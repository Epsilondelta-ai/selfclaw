use std::collections::HashMap;

use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{ChannelKind, InboundMessage, OutboundMessage};

/// The Gateway is the central message router.
///
/// It manages multiple communication channels and provides a unified
/// interface for the agent to send/receive messages across all platforms.
///
/// Architecture:
/// ```text
///  Discord ──┐
///  Telegram ─┤
///  Slack ────┤──→ Gateway ──→ Agent (inbound_rx)
///  CLI ──────┤
///  WebChat ──┘
///             ←── Agent sends OutboundMessage to specific channel
/// ```
pub struct Gateway {
    /// Handles to each connected channel, keyed by ChannelKind.
    channels: HashMap<ChannelKind, ChannelHandle>,

    /// Sender that all channels push inbound messages into.
    inbound_tx: mpsc::UnboundedSender<InboundMessage>,

    /// Receiver for the agent to consume inbound messages.
    inbound_rx: Option<mpsc::UnboundedReceiver<InboundMessage>>,
}

impl Gateway {
    /// Create a new Gateway.
    pub fn new() -> Self {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        Self {
            channels: HashMap::new(),
            inbound_tx,
            inbound_rx: Some(inbound_rx),
        }
    }

    /// Get the sender for inbound messages (given to channels so they can forward messages).
    pub fn inbound_sender(&self) -> mpsc::UnboundedSender<InboundMessage> {
        self.inbound_tx.clone()
    }

    /// Take the inbound receiver (can only be called once — the agent owns it).
    pub fn take_inbound_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<InboundMessage>> {
        self.inbound_rx.take()
    }

    /// Register a channel handle with the gateway.
    pub fn register_channel(&mut self, handle: ChannelHandle) {
        info!(channel = %handle.name, kind = %handle.kind, "Channel registered with gateway");
        self.channels.insert(handle.kind.clone(), handle);
    }

    /// Send a message to a specific channel.
    pub fn send(&self, message: OutboundMessage) -> Result<(), ChannelError> {
        let target = &message.target_channel;

        if let Some(handle) = self.channels.get(target) {
            if handle.connected {
                handle.send(message)
            } else {
                Err(ChannelError::NotConnected(format!(
                    "channel {} is registered but not connected",
                    handle.name
                )))
            }
        } else {
            Err(ChannelError::NotConnected(format!(
                "no channel registered for {:?}",
                target
            )))
        }
    }

    /// Broadcast a message to all connected channels.
    pub fn broadcast(&self, content: &str) -> Vec<Result<(), ChannelError>> {
        self.channels
            .iter()
            .filter(|(_, handle)| handle.connected)
            .map(|(kind, handle)| {
                let msg = OutboundMessage::new(content, kind.clone());
                handle.send(msg)
            })
            .collect()
    }

    /// Returns the list of registered channel kinds.
    pub fn registered_channels(&self) -> Vec<&ChannelKind> {
        self.channels.keys().collect()
    }

    /// Returns the number of registered channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of connected channels.
    pub fn connected_count(&self) -> usize {
        self.channels.values().filter(|h| h.connected).count()
    }

    /// Remove a channel from the gateway.
    pub fn remove_channel(&mut self, kind: &ChannelKind) -> Option<ChannelHandle> {
        let handle = self.channels.remove(kind);
        if let Some(ref h) = handle {
            warn!(channel = %h.name, "Channel removed from gateway");
        }
        handle
    }

    /// Check if a specific channel kind is registered.
    pub fn has_channel(&self, kind: &ChannelKind) -> bool {
        self.channels.contains_key(kind)
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handle(kind: ChannelKind, connected: bool) -> (ChannelHandle, mpsc::UnboundedReceiver<OutboundMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = ChannelHandle {
            kind: kind.clone(),
            name: format!("test-{}", kind),
            outbound_tx: tx,
            connected,
        };
        (handle, rx)
    }

    #[test]
    fn test_new_gateway_empty() {
        let gw = Gateway::new();
        assert_eq!(gw.channel_count(), 0);
        assert_eq!(gw.connected_count(), 0);
    }

    #[test]
    fn test_register_channel() {
        let mut gw = Gateway::new();
        let (handle, _rx) = make_handle(ChannelKind::Cli, true);

        gw.register_channel(handle);
        assert_eq!(gw.channel_count(), 1);
        assert!(gw.has_channel(&ChannelKind::Cli));
        assert!(!gw.has_channel(&ChannelKind::Discord));
    }

    #[test]
    fn test_register_multiple_channels() {
        let mut gw = Gateway::new();
        let (h1, _r1) = make_handle(ChannelKind::Cli, true);
        let (h2, _r2) = make_handle(ChannelKind::Discord, true);
        let (h3, _r3) = make_handle(ChannelKind::Telegram, false);

        gw.register_channel(h1);
        gw.register_channel(h2);
        gw.register_channel(h3);

        assert_eq!(gw.channel_count(), 3);
        assert_eq!(gw.connected_count(), 2);
    }

    #[test]
    fn test_send_to_registered_channel() {
        let mut gw = Gateway::new();
        let (handle, mut rx) = make_handle(ChannelKind::Cli, true);
        gw.register_channel(handle);

        let msg = OutboundMessage::new("hello from agent", ChannelKind::Cli);
        gw.send(msg).unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "hello from agent");
    }

    #[test]
    fn test_send_to_unregistered_channel_fails() {
        let gw = Gateway::new();
        let msg = OutboundMessage::new("hello", ChannelKind::Discord);
        let result = gw.send(msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_send_to_disconnected_channel_fails() {
        let mut gw = Gateway::new();
        let (handle, _rx) = make_handle(ChannelKind::Slack, false);
        gw.register_channel(handle);

        let msg = OutboundMessage::new("hello", ChannelKind::Slack);
        let result = gw.send(msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_broadcast() {
        let mut gw = Gateway::new();
        let (h1, mut r1) = make_handle(ChannelKind::Cli, true);
        let (h2, mut r2) = make_handle(ChannelKind::Discord, true);
        let (h3, _r3) = make_handle(ChannelKind::Telegram, false); // disconnected

        gw.register_channel(h1);
        gw.register_channel(h2);
        gw.register_channel(h3);

        let results = gw.broadcast("broadcast message");

        // Only 2 connected channels should receive the broadcast
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));

        let m1 = r1.try_recv().unwrap();
        assert_eq!(m1.content, "broadcast message");

        let m2 = r2.try_recv().unwrap();
        assert_eq!(m2.content, "broadcast message");
    }

    #[test]
    fn test_remove_channel() {
        let mut gw = Gateway::new();
        let (handle, _rx) = make_handle(ChannelKind::Cli, true);
        gw.register_channel(handle);

        assert!(gw.has_channel(&ChannelKind::Cli));
        let removed = gw.remove_channel(&ChannelKind::Cli);
        assert!(removed.is_some());
        assert!(!gw.has_channel(&ChannelKind::Cli));
        assert_eq!(gw.channel_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_channel() {
        let mut gw = Gateway::new();
        let removed = gw.remove_channel(&ChannelKind::Discord);
        assert!(removed.is_none());
    }

    #[test]
    fn test_take_inbound_receiver_once() {
        let mut gw = Gateway::new();
        let rx = gw.take_inbound_receiver();
        assert!(rx.is_some());

        let rx2 = gw.take_inbound_receiver();
        assert!(rx2.is_none());
    }

    #[test]
    fn test_inbound_message_flow() {
        let mut gw = Gateway::new();
        let tx = gw.inbound_sender();
        let mut rx = gw.take_inbound_receiver().unwrap();

        let inbound = InboundMessage {
            id: "msg-1".to_string(),
            content: "hello agent".to_string(),
            metadata: crate::message::MessageMetadata {
                timestamp: "2026-03-01T12:00:00Z".to_string(),
                sender: "user-1".to_string(),
                channel: ChannelKind::Discord,
                intent: crate::message::MessageIntent::Chat,
                conversation_id: None,
            },
        };

        tx.send(inbound).unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(received.id, "msg-1");
        assert_eq!(received.content, "hello agent");
    }

    #[test]
    fn test_registered_channels_list() {
        let mut gw = Gateway::new();
        let (h1, _r1) = make_handle(ChannelKind::Cli, true);
        let (h2, _r2) = make_handle(ChannelKind::Slack, true);
        gw.register_channel(h1);
        gw.register_channel(h2);

        let kinds = gw.registered_channels();
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&&ChannelKind::Cli));
        assert!(kinds.contains(&&ChannelKind::Slack));
    }
}
