use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{info, warn};

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

/// Message types for the WebSocket protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WsMessageType {
    Chat,
    Status,
    Memory,
    StateChange,
}

/// A structured message sent over the WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsProtocolMessage {
    #[serde(rename = "type")]
    pub msg_type: WsMessageType,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

impl WsProtocolMessage {
    pub fn chat(content: &str) -> Self {
        Self {
            msg_type: WsMessageType::Chat,
            payload: serde_json::json!({ "content": content }),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn status(data: serde_json::Value) -> Self {
        Self {
            msg_type: WsMessageType::Status,
            payload: data,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn memory(path: &str, content: &str) -> Self {
        Self {
            msg_type: WsMessageType::Memory,
            payload: serde_json::json!({ "path": path, "content": content }),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn state_change(from: &str, to: &str) -> Self {
        Self {
            msg_type: WsMessageType::StateChange,
            payload: serde_json::json!({ "from": from, "to": to }),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

type ClientId = u64;
type ClientSender = mpsc::UnboundedSender<WsMessage>;

/// Shared state for all connected WebSocket clients.
struct WsSharedState {
    clients: HashMap<ClientId, ClientSender>,
    next_id: ClientId,
}

impl WsSharedState {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
            next_id: 1,
        }
    }

    fn add_client(&mut self, sender: ClientSender) -> ClientId {
        let id = self.next_id;
        self.next_id += 1;
        self.clients.insert(id, sender);
        id
    }

    fn remove_client(&mut self, id: ClientId) {
        self.clients.remove(&id);
    }

    fn broadcast(&self, message: &str) {
        for sender in self.clients.values() {
            let _ = sender.send(WsMessage::Text(message.to_string()));
        }
    }

    fn client_count(&self) -> usize {
        self.clients.len()
    }
}

/// WebSocket server for real-time communication with the web UI.
///
/// Listens for WebSocket connections and bridges them into the
/// Gateway's inbound/outbound message flow.
pub struct WebSocketServer {
    state: Arc<RwLock<WsSharedState>>,
    port: u16,
    /// Sender for messages from the outbound_rx forwarder to broadcast.
    broadcast_tx: mpsc::UnboundedSender<String>,
    broadcast_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        let (broadcast_tx, broadcast_rx) = mpsc::unbounded_channel();
        Self {
            state: Arc::new(RwLock::new(WsSharedState::new())),
            port,
            broadcast_tx,
            broadcast_rx: Arc::new(Mutex::new(Some(broadcast_rx))),
        }
    }

    /// Get the port the server will listen on.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Broadcast a protocol message to all connected clients.
    pub fn broadcast(&self, message: &WsProtocolMessage) {
        if let Ok(json) = serde_json::to_string(message) {
            let _ = self.broadcast_tx.send(json);
        }
    }

    /// Broadcast a raw string to all connected clients.
    pub fn broadcast_raw(&self, message: &str) {
        let _ = self.broadcast_tx.send(message.to_string());
    }

    /// Get the number of connected clients.
    pub async fn client_count(&self) -> usize {
        self.state.read().await.client_count()
    }

    /// Start the WebSocket server and integrate it into the Gateway.
    ///
    /// Returns a ChannelHandle for the gateway to route outbound messages.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        let state = self.state.clone();
        let port = self.port;
        let broadcast_rx = self.broadcast_rx.clone();
        let broadcast_tx_for_outbound = self.broadcast_tx.clone();

        // Spawn the WebSocket listener
        tokio::spawn(async move {
            Self::run_listener(state, port, inbound_tx, broadcast_rx).await;
        });

        // Spawn outbound message forwarder: converts OutboundMessages to WS broadcasts
        tokio::spawn(async move {
            Self::forward_outbound(outbound_rx, broadcast_tx_for_outbound).await;
        });

        info!(port = port, "WebSocket server started");

        Ok(ChannelHandle {
            kind: ChannelKind::WebChat,
            name: "websocket".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    /// Forward outbound messages from the gateway into WebSocket broadcasts.
    async fn forward_outbound(
        mut outbound_rx: mpsc::UnboundedReceiver<OutboundMessage>,
        broadcast_tx: mpsc::UnboundedSender<String>,
    ) {
        while let Some(msg) = outbound_rx.recv().await {
            let ws_msg = WsProtocolMessage::chat(&msg.content);
            if let Ok(json) = serde_json::to_string(&ws_msg) {
                let _ = broadcast_tx.send(json);
            }
        }
    }

    /// Main WebSocket listener loop.
    async fn run_listener(
        state: Arc<RwLock<WsSharedState>>,
        port: u16,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        broadcast_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
    ) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                info!(addr = %addr, "WebSocket server listening");
                l
            }
            Err(e) => {
                warn!(error = %e, "Failed to bind WebSocket server");
                return;
            }
        };

        // Spawn broadcast forwarder: reads from broadcast_rx and sends to all clients
        let state_for_broadcast = state.clone();
        let rx = broadcast_rx.lock().await.take();
        if let Some(mut broadcast_rx) = rx {
            tokio::spawn(async move {
                while let Some(msg) = broadcast_rx.recv().await {
                    let s = state_for_broadcast.read().await;
                    s.broadcast(&msg);
                }
            });
        }

        let mut msg_counter: u64 = 0;

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!(addr = %addr, "New WebSocket connection");
                    let state = state.clone();
                    let inbound_tx = inbound_tx.clone();
                    msg_counter += 1;
                    let counter_base = msg_counter * 1000;

                    tokio::spawn(async move {
                        Self::handle_connection(stream, addr, state, inbound_tx, counter_base)
                            .await;
                    });
                }
                Err(e) => {
                    warn!(error = %e, "WebSocket accept error");
                }
            }
        }
    }

    /// Handle a single WebSocket connection.
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        state: Arc<RwLock<WsSharedState>>,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        counter_base: u64,
    ) {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                warn!(error = %e, addr = %addr, "WebSocket handshake failed");
                return;
            }
        };

        let (ws_sender, mut ws_receiver) = ws_stream.split();

        // Create a channel for this client's outbound messages
        let (client_tx, mut client_rx) = mpsc::unbounded_channel::<WsMessage>();

        let client_id = {
            let mut s = state.write().await;
            s.add_client(client_tx)
        };

        info!(client_id = client_id, addr = %addr, "WebSocket client connected");

        // Spawn a task to forward messages from client_rx to the WS sender
        let ws_sender = Arc::new(Mutex::new(ws_sender));
        let ws_sender_clone = ws_sender.clone();
        let sender_task = tokio::spawn(async move {
            while let Some(msg) = client_rx.recv().await {
                let mut sender = ws_sender_clone.lock().await;
                if sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Read incoming messages
        let mut msg_count: u64 = 0;
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(WsMessage::Text(text)) => {
                    // Try to parse as a WsProtocolMessage
                    if let Ok(protocol_msg) = serde_json::from_str::<WsProtocolMessage>(&text) {
                        if protocol_msg.msg_type == WsMessageType::Chat {
                            let content = protocol_msg.payload["content"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string();
                            let sender_name = protocol_msg.payload["sender"]
                                .as_str()
                                .unwrap_or("web-user")
                                .to_string();

                            if !content.is_empty() {
                                msg_count += 1;
                                let inbound = InboundMessage {
                                    id: format!("ws-{}-{}", counter_base, msg_count),
                                    content,
                                    metadata: MessageMetadata {
                                        timestamp: Utc::now().to_rfc3339(),
                                        sender: sender_name,
                                        channel: ChannelKind::WebChat,
                                        intent: MessageIntent::Chat,
                                        conversation_id: Some(format!("ws-{}", client_id)),
                                    },
                                };

                                let _ = inbound_tx.send(inbound);
                            }
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!(
                        client_id = client_id,
                        "WebSocket client disconnected (close frame)"
                    );
                    break;
                }
                Ok(WsMessage::Ping(data)) => {
                    let mut sender = ws_sender.lock().await;
                    let _ = sender.send(WsMessage::Pong(data)).await;
                }
                Err(e) => {
                    warn!(client_id = client_id, error = %e, "WebSocket receive error");
                    break;
                }
                _ => {}
            }
        }

        // Clean up
        {
            let mut s = state.write().await;
            s.remove_client(client_id);
        }
        sender_task.abort();

        info!(client_id = client_id, addr = %addr, "WebSocket client disconnected");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite;

    #[test]
    fn test_ws_protocol_message_chat() {
        let msg = WsProtocolMessage::chat("hello");
        assert_eq!(msg.msg_type, WsMessageType::Chat);
        assert_eq!(msg.payload["content"], "hello");
        assert!(!msg.timestamp.is_empty());
    }

    #[test]
    fn test_ws_protocol_message_status() {
        let msg = WsProtocolMessage::status(serde_json::json!({"state": "thinking"}));
        assert_eq!(msg.msg_type, WsMessageType::Status);
        assert_eq!(msg.payload["state"], "thinking");
    }

    #[test]
    fn test_ws_protocol_message_memory() {
        let msg = WsProtocolMessage::memory("identity/values.md", "# Values");
        assert_eq!(msg.msg_type, WsMessageType::Memory);
        assert_eq!(msg.payload["path"], "identity/values.md");
        assert_eq!(msg.payload["content"], "# Values");
    }

    #[test]
    fn test_ws_protocol_message_state_change() {
        let msg = WsProtocolMessage::state_change("idle", "thinking");
        assert_eq!(msg.msg_type, WsMessageType::StateChange);
        assert_eq!(msg.payload["from"], "idle");
        assert_eq!(msg.payload["to"], "thinking");
    }

    #[test]
    fn test_ws_protocol_message_serde_roundtrip() {
        let msg = WsProtocolMessage::chat("test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: WsProtocolMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.msg_type, WsMessageType::Chat);
        assert_eq!(deserialized.payload["content"], "test message");
    }

    #[test]
    fn test_ws_protocol_message_json_format() {
        let msg = WsProtocolMessage::chat("hi");
        let json = serde_json::to_string(&msg).unwrap();
        // Verify the JSON has the expected "type" field (not "msg_type")
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(val.get("type").is_some());
        assert!(val.get("payload").is_some());
        assert!(val.get("timestamp").is_some());
    }

    #[test]
    fn test_ws_shared_state() {
        let mut state = WsSharedState::new();
        assert_eq!(state.client_count(), 0);

        let (tx1, _rx1) = mpsc::unbounded_channel();
        let id1 = state.add_client(tx1);
        assert_eq!(state.client_count(), 1);

        let (tx2, _rx2) = mpsc::unbounded_channel();
        let id2 = state.add_client(tx2);
        assert_eq!(state.client_count(), 2);
        assert_ne!(id1, id2);

        state.remove_client(id1);
        assert_eq!(state.client_count(), 1);

        state.remove_client(id2);
        assert_eq!(state.client_count(), 0);
    }

    #[test]
    fn test_ws_shared_state_broadcast() {
        let mut state = WsSharedState::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        state.add_client(tx1);
        state.add_client(tx2);

        state.broadcast(r#"{"type":"chat","payload":{"content":"hello"},"timestamp":"now"}"#);

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();

        match msg1 {
            WsMessage::Text(t) => assert!(t.contains("hello")),
            _ => panic!("expected text message"),
        }
        match msg2 {
            WsMessage::Text(t) => assert!(t.contains("hello")),
            _ => panic!("expected text message"),
        }
    }

    #[test]
    fn test_websocket_server_creation() {
        let server = WebSocketServer::new(3002);
        assert_eq!(server.port(), 3002);
    }

    #[test]
    fn test_websocket_server_broadcast_protocol() {
        let server = WebSocketServer::new(0);
        let msg = WsProtocolMessage::chat("test broadcast");
        // This shouldn't panic even with no clients
        server.broadcast(&msg);
    }

    #[tokio::test]
    async fn test_websocket_server_start_and_connect() {
        // Start server on a random port
        let server = WebSocketServer::new(0);

        // Use port 0 — bind to any available port
        // We'll start on a known port for the test
        let port = 19876; // Use a high port unlikely to conflict
        let server = WebSocketServer::new(port);
        let (inbound_tx, mut inbound_rx) = mpsc::unbounded_channel();

        let handle = server.start(inbound_tx).unwrap();
        assert_eq!(handle.kind, ChannelKind::WebChat);
        assert_eq!(handle.name, "websocket");
        assert!(handle.connected);

        // Give the server a moment to bind
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Connect a WebSocket client
        let url = format!("ws://127.0.0.1:{}", port);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect to WS server");

        // Send a chat message
        let chat_msg = WsProtocolMessage::chat("hello from test");
        let json = serde_json::to_string(&chat_msg).unwrap();
        ws_stream.send(WsMessage::Text(json)).await.unwrap();

        // Give the server time to process
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Check that the inbound message was forwarded
        let inbound = inbound_rx.try_recv().unwrap();
        assert_eq!(inbound.content, "hello from test");
        assert_eq!(inbound.metadata.channel, ChannelKind::WebChat);
    }

    #[tokio::test]
    async fn test_websocket_server_broadcast_to_client() {
        let port = 19877;
        let server = WebSocketServer::new(port);
        let (inbound_tx, _inbound_rx) = mpsc::unbounded_channel();

        let _handle = server.start(inbound_tx).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Connect a client
        let url = format!("ws://127.0.0.1:{}", port);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect");

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Broadcast a message from the server
        let msg = WsProtocolMessage::status(serde_json::json!({"state": "reflecting"}));
        server.broadcast(&msg);

        // Give time for the broadcast to propagate
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Read the message from the client
        let received = tokio::time::timeout(std::time::Duration::from_secs(2), ws_stream.next())
            .await
            .expect("Timeout waiting for message")
            .expect("Stream ended")
            .expect("Error receiving message");

        match received {
            WsMessage::Text(text) => {
                let parsed: WsProtocolMessage = serde_json::from_str(&text).unwrap();
                assert_eq!(parsed.msg_type, WsMessageType::Status);
                assert_eq!(parsed.payload["state"], "reflecting");
            }
            other => panic!("Expected text message, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_websocket_outbound_message_forwarding() {
        let port = 19878;
        let server = WebSocketServer::new(port);
        let (inbound_tx, _inbound_rx) = mpsc::unbounded_channel();

        let handle = server.start(inbound_tx).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Connect a client
        let url = format!("ws://127.0.0.1:{}", port);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect");

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Send an outbound message through the channel handle (as the agent would)
        let outbound = OutboundMessage::new("agent says hello", ChannelKind::WebChat);
        handle.send(outbound).unwrap();

        // Give time for forwarding
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Client should receive the message as a chat protocol message
        let received = tokio::time::timeout(std::time::Duration::from_secs(2), ws_stream.next())
            .await
            .expect("Timeout waiting for message")
            .expect("Stream ended")
            .expect("Error receiving message");

        match received {
            WsMessage::Text(text) => {
                let parsed: WsProtocolMessage = serde_json::from_str(&text).unwrap();
                assert_eq!(parsed.msg_type, WsMessageType::Chat);
                assert_eq!(parsed.payload["content"], "agent says hello");
            }
            other => panic!("Expected text message, got {:?}", other),
        }
    }
}
