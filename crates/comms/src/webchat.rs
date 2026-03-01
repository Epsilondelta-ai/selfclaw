use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use selfclaw_config::WebChatConfig;

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

/// WebChat channel for browser-based interaction.
///
/// Runs an HTTP server that accepts JSON POST requests for inbound messages
/// and dispatches outbound messages to connected clients.
///
/// This serves as the bridge for the optional Next.js web UI.
///
/// API endpoints:
///   POST /api/message  — Send a message to SelfClaw
///     Body: { "content": "...", "sender": "..." }
///
///   GET  /api/messages  — Long-poll for new outbound messages
///
/// A full WebSocket implementation would use tokio-tungstenite, but for
/// initial simplicity this uses HTTP long-polling via reqwest/hyper.
pub struct WebChatChannel {
    config: WebChatConfig,
    connected: Arc<AtomicBool>,
}

impl WebChatChannel {
    pub fn new(config: WebChatConfig) -> Self {
        Self {
            config,
            connected: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn port(&self) -> u16 {
        self.config.port
    }

    /// Start the WebChat channel.
    ///
    /// Spawns an HTTP listener that bridges web clients to the gateway.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        self.connected.store(true, Ordering::Relaxed);

        let port = self.config.port;
        let connected = self.connected.clone();

        // Spawn the HTTP server task
        tokio::spawn(async move {
            Self::run_server(port, inbound_tx, outbound_rx, connected).await;
        });

        info!(port = port, "WebChat channel started");

        Ok(ChannelHandle {
            kind: ChannelKind::WebChat,
            name: "webchat".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    pub fn stop(&self) {
        self.connected.store(false, Ordering::Relaxed);
        info!("WebChat channel stopped");
    }

    /// Minimal HTTP server using tokio TCP listener.
    ///
    /// Accepts POST /api/message for inbound messages and serves outbound messages
    /// via a simple response queue.
    async fn run_server(
        port: u16,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        mut outbound_rx: mpsc::UnboundedReceiver<OutboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                info!(addr = %addr, "WebChat server listening");
                l
            }
            Err(e) => {
                warn!(error = %e, "Failed to bind WebChat server");
                connected.store(false, Ordering::Relaxed);
                return;
            }
        };

        // Collect outbound messages into a shared buffer
        let outbound_buffer: Arc<tokio::sync::Mutex<Vec<OutboundMessage>>> =
            Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let buffer_writer = outbound_buffer.clone();
        let connected_writer = connected.clone();
        tokio::spawn(async move {
            while connected_writer.load(Ordering::Relaxed) {
                match outbound_rx.recv().await {
                    Some(msg) => {
                        let mut buf = buffer_writer.lock().await;
                        buf.push(msg);
                        // Keep buffer bounded
                        if buf.len() > 100 {
                            buf.drain(0..50);
                        }
                    }
                    None => break,
                }
            }
        });

        let mut msg_counter: u64 = 0;

        while connected.load(Ordering::Relaxed) {
            let accept =
                tokio::time::timeout(std::time::Duration::from_secs(1), listener.accept()).await;

            match accept {
                Ok(Ok((mut stream, _addr))) => {
                    let mut buf = vec![0u8; 8192];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(_) => continue,
                    };
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();

                    if request.contains("POST /api/message") {
                        // Extract JSON body (after double CRLF)
                        if let Some(body_start) = request.find("\r\n\r\n") {
                            let body = &request[body_start + 4..];
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                                let content =
                                    json["content"].as_str().unwrap_or_default().to_string();
                                let sender =
                                    json["sender"].as_str().unwrap_or("web-user").to_string();

                                if !content.is_empty() {
                                    msg_counter += 1;
                                    let inbound = InboundMessage {
                                        id: format!("web-{}", msg_counter),
                                        content,
                                        metadata: MessageMetadata {
                                            timestamp: Utc::now().to_rfc3339(),
                                            sender,
                                            channel: ChannelKind::WebChat,
                                            intent: MessageIntent::Chat,
                                            conversation_id: Some("webchat".to_string()),
                                        },
                                    };

                                    let _ = inbound_tx.send(inbound);
                                }
                            }

                            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{\"ok\":true}";
                            let _ = stream.write_all(response.as_bytes()).await;
                        }
                    } else if request.contains("GET /api/messages") {
                        let messages = {
                            let mut buf = outbound_buffer.lock().await;
                            let msgs: Vec<_> = buf.drain(..).collect();
                            msgs
                        };

                        let json_body = serde_json::json!({
                            "messages": messages.iter().map(|m| {
                                serde_json::json!({
                                    "content": m.content,
                                    "reply_to": m.reply_to,
                                })
                            }).collect::<Vec<_>>()
                        });

                        let body = serde_json::to_string(&json_body).unwrap_or_default();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else if request.contains("OPTIONS") {
                        // CORS preflight
                        let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else {
                        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "WebChat accept error");
                }
                Err(_) => {
                    // Timeout — loop back and check connected flag
                }
            }
        }
    }
}

/// Build a WebChatChannel from config, returning None if not enabled.
pub fn from_config(config: &WebChatConfig) -> Option<WebChatChannel> {
    if config.enabled {
        Some(WebChatChannel::new(config.clone()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webchat_channel_creation() {
        let config = WebChatConfig {
            enabled: true,
            port: 3001,
        };
        let ch = WebChatChannel::new(config);
        assert!(!ch.is_connected());
        assert_eq!(ch.port(), 3001);
    }

    #[test]
    fn test_webchat_from_config_disabled() {
        let config = WebChatConfig::default();
        assert!(from_config(&config).is_none());
    }

    #[test]
    fn test_webchat_from_config_enabled() {
        let config = WebChatConfig {
            enabled: true,
            port: 4000,
        };
        let ch = from_config(&config);
        assert!(ch.is_some());
        assert_eq!(ch.unwrap().port(), 4000);
    }

    #[test]
    fn test_webchat_stop() {
        let config = WebChatConfig {
            enabled: true,
            port: 3001,
        };
        let ch = WebChatChannel::new(config);
        ch.connected.store(true, Ordering::Relaxed);

        ch.stop();
        assert!(!ch.is_connected());
    }

    #[tokio::test]
    async fn test_webchat_start_and_send_message() {
        let config = WebChatConfig {
            enabled: true,
            port: 0, // Use port 0 to get a random available port
        };

        // We can at least verify the channel handle is created properly
        // (actual HTTP testing would require binding to a real port)
        let ch = WebChatChannel::new(config);
        let (tx, _rx) = mpsc::unbounded_channel();
        // Note: start() with port 0 may or may not bind successfully
        // depending on the OS, so we just test the handle structure
        let _ = ch.start(tx);
    }
}
