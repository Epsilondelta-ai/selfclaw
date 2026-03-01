use serde::{Deserialize, Serialize};

/// Identifies which channel a message came from or should be sent to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelKind {
    Cli,
    Discord,
    Telegram,
    Slack,
    WebChat,
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelKind::Cli => write!(f, "cli"),
            ChannelKind::Discord => write!(f, "discord"),
            ChannelKind::Telegram => write!(f, "telegram"),
            ChannelKind::Slack => write!(f, "slack"),
            ChannelKind::WebChat => write!(f, "webchat"),
        }
    }
}

/// Classification of intent behind a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MessageIntent {
    /// A general conversational message.
    #[default]
    Chat,
    /// A command or instruction to the agent.
    Command,
    /// A question directed at the agent.
    Question,
    /// A response to a previous agent message.
    Reply,
    /// System-level signal (pause, stop, etc).
    System,
}


impl MessageIntent {
    pub fn display_str(&self) -> &'static str {
        match self {
            MessageIntent::Chat => "chat",
            MessageIntent::Command => "command",
            MessageIntent::Question => "question",
            MessageIntent::Reply => "reply",
            MessageIntent::System => "system",
        }
    }
}

/// Metadata attached to every message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub timestamp: String,
    pub sender: String,
    pub channel: ChannelKind,
    pub intent: MessageIntent,
    /// Optional conversation or thread identifier for context.
    pub conversation_id: Option<String>,
}

/// A message received from a human via any channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub id: String,
    pub content: String,
    pub metadata: MessageMetadata,
}

/// A message to be sent to a human via a specific channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub content: String,
    pub target_channel: ChannelKind,
    /// If replying to a specific inbound message, its ID.
    pub reply_to: Option<String>,
    /// Optional conversation or thread identifier.
    pub conversation_id: Option<String>,
}

impl OutboundMessage {
    pub fn new(content: impl Into<String>, channel: ChannelKind) -> Self {
        Self {
            content: content.into(),
            target_channel: channel,
            reply_to: None,
            conversation_id: None,
        }
    }

    pub fn reply(content: impl Into<String>, channel: ChannelKind, reply_to: String) -> Self {
        Self {
            content: content.into(),
            target_channel: channel,
            reply_to: Some(reply_to),
            conversation_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_kind_display() {
        assert_eq!(ChannelKind::Cli.to_string(), "cli");
        assert_eq!(ChannelKind::Discord.to_string(), "discord");
        assert_eq!(ChannelKind::Telegram.to_string(), "telegram");
        assert_eq!(ChannelKind::Slack.to_string(), "slack");
        assert_eq!(ChannelKind::WebChat.to_string(), "webchat");
    }

    #[test]
    fn test_default_intent_is_chat() {
        assert_eq!(MessageIntent::default(), MessageIntent::Chat);
    }

    #[test]
    fn test_outbound_message_new() {
        let msg = OutboundMessage::new("hello", ChannelKind::Cli);
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.target_channel, ChannelKind::Cli);
        assert!(msg.reply_to.is_none());
    }

    #[test]
    fn test_outbound_message_reply() {
        let msg = OutboundMessage::reply("response", ChannelKind::Discord, "msg-123".to_string());
        assert_eq!(msg.content, "response");
        assert_eq!(msg.target_channel, ChannelKind::Discord);
        assert_eq!(msg.reply_to.as_deref(), Some("msg-123"));
    }

    #[test]
    fn test_inbound_message_serde_roundtrip() {
        let msg = InboundMessage {
            id: "test-1".to_string(),
            content: "Hello SelfClaw".to_string(),
            metadata: MessageMetadata {
                timestamp: "2026-03-01T12:00:00Z".to_string(),
                sender: "human-1".to_string(),
                channel: ChannelKind::Telegram,
                intent: MessageIntent::Chat,
                conversation_id: Some("conv-42".to_string()),
            },
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: InboundMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-1");
        assert_eq!(deserialized.content, "Hello SelfClaw");
        assert_eq!(deserialized.metadata.channel, ChannelKind::Telegram);
        assert_eq!(deserialized.metadata.intent, MessageIntent::Chat);
        assert_eq!(
            deserialized.metadata.conversation_id.as_deref(),
            Some("conv-42")
        );
    }
}
