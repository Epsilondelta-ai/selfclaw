pub mod channel;
pub mod cli;
pub mod discord;
pub mod gateway;
pub mod message;
pub mod queue;
pub mod slack;
pub mod telegram;
pub mod webchat;
pub mod ws;

pub use channel::{ChannelError, ChannelHandle};
pub use gateway::Gateway;
pub use message::{ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage};
pub use queue::ChatQueue;
pub use ws::{WebSocketServer, WsMessageType, WsProtocolMessage};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
