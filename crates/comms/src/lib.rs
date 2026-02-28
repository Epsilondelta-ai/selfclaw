pub mod message;
pub mod channel;
pub mod gateway;
pub mod queue;
pub mod cli;
pub mod discord;
pub mod telegram;
pub mod slack;
pub mod webchat;
pub mod ws;

pub use message::{ChannelKind, InboundMessage, OutboundMessage, MessageMetadata, MessageIntent};
pub use channel::{ChannelError, ChannelHandle};
pub use gateway::Gateway;
pub use queue::ChatQueue;
pub use ws::{WebSocketServer, WsProtocolMessage, WsMessageType};

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
