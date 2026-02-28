use thiserror::Error;
use tokio::sync::mpsc;

use crate::message::{ChannelKind, OutboundMessage};

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("channel not connected: {0}")]
    NotConnected(String),

    #[error("send failed: {0}")]
    SendFailed(String),

    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("channel error: {0}")]
    Other(String),
}

/// Handle for a running channel — holds the sender for outbound messages
/// and metadata about the channel.
///
/// Each concrete channel (CLI, Discord, Telegram, etc.) produces a ChannelHandle
/// when started. The Gateway uses these handles to route outbound messages.
pub struct ChannelHandle {
    pub kind: ChannelKind,
    pub name: String,
    pub outbound_tx: mpsc::UnboundedSender<OutboundMessage>,
    pub connected: bool,
}

impl std::fmt::Debug for ChannelHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelHandle")
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("connected", &self.connected)
            .finish()
    }
}

impl ChannelHandle {
    pub fn send(&self, message: OutboundMessage) -> Result<(), ChannelError> {
        self.outbound_tx.send(message).map_err(|e| {
            ChannelError::SendFailed(format!("channel {} closed: {}", self.name, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ChannelKind;

    #[test]
    fn test_channel_error_display() {
        let err = ChannelError::NotConnected("discord".to_string());
        assert_eq!(err.to_string(), "channel not connected: discord");

        let err = ChannelError::SendFailed("timeout".to_string());
        assert_eq!(err.to_string(), "send failed: timeout");

        let err = ChannelError::AuthFailed("invalid token".to_string());
        assert_eq!(err.to_string(), "authentication failed: invalid token");
    }

    #[test]
    fn test_channel_handle_send() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = ChannelHandle {
            kind: ChannelKind::Cli,
            name: "test-cli".to_string(),
            outbound_tx: tx,
            connected: true,
        };

        let msg = OutboundMessage::new("hello", ChannelKind::Cli);
        handle.send(msg).unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "hello");
        assert_eq!(received.target_channel, ChannelKind::Cli);
    }

    #[test]
    fn test_channel_handle_send_on_closed_channel() {
        let (tx, rx) = mpsc::unbounded_channel::<OutboundMessage>();
        drop(rx);

        let handle = ChannelHandle {
            kind: ChannelKind::Discord,
            name: "test-discord".to_string(),
            outbound_tx: tx,
            connected: false,
        };

        let msg = OutboundMessage::new("hello", ChannelKind::Discord);
        let result = handle.send(msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_handle_debug() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let handle = ChannelHandle {
            kind: ChannelKind::Cli,
            name: "test".to_string(),
            outbound_tx: tx,
            connected: true,
        };
        let debug = format!("{:?}", handle);
        assert!(debug.contains("test"));
        assert!(debug.contains("Cli"));
    }
}
