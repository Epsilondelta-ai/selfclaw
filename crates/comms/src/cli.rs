use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

/// CLI channel for terminal-based interaction with SelfClaw.
///
/// This is the primary and always-available communication channel.
/// It reads lines from stdin and writes responses to stdout.
pub struct CliChannel {
    connected: Arc<AtomicBool>,
    prompt: String,
}

impl CliChannel {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            prompt: "you> ".to_string(),
        }
    }

    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Start the CLI channel.
    ///
    /// Spawns two tasks:
    /// - A stdin reader that sends InboundMessages to the gateway.
    /// - A stdout writer that receives OutboundMessages and prints them.
    ///
    /// Returns a ChannelHandle for the gateway to use.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        let connected = self.connected.clone();
        connected.store(true, Ordering::Relaxed);

        let prompt = self.prompt.clone();

        // Spawn stdin reader
        let connected_reader = connected.clone();
        let inbound_tx_clone = inbound_tx.clone();
        tokio::spawn(async move {
            Self::stdin_reader_loop(inbound_tx_clone, connected_reader, &prompt).await;
        });

        // Spawn stdout writer
        let connected_writer = connected.clone();
        tokio::spawn(async move {
            Self::stdout_writer_loop(outbound_rx, connected_writer).await;
        });

        info!("CLI channel started");

        Ok(ChannelHandle {
            kind: ChannelKind::Cli,
            name: "cli".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    /// Stop the CLI channel.
    pub fn stop(&self) {
        self.connected.store(false, Ordering::Relaxed);
        info!("CLI channel stopped");
    }

    async fn stdin_reader_loop(
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        connected: Arc<AtomicBool>,
        prompt: &str,
    ) {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut msg_counter: u64 = 0;

        while connected.load(Ordering::Relaxed) {
            // Print prompt
            eprint!("{}", prompt);

            match lines.next_line().await {
                Ok(Some(line)) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Classify intent
                    let intent = classify_intent(&trimmed);

                    msg_counter += 1;
                    let msg = InboundMessage {
                        id: format!("cli-{}", msg_counter),
                        content: trimmed,
                        metadata: MessageMetadata {
                            timestamp: Utc::now().to_rfc3339(),
                            sender: "human".to_string(),
                            channel: ChannelKind::Cli,
                            intent,
                            conversation_id: Some("cli-session".to_string()),
                        },
                    };

                    if inbound_tx.send(msg).is_err() {
                        warn!("Inbound channel closed, stopping CLI reader");
                        break;
                    }
                }
                Ok(None) => {
                    info!("Stdin closed (EOF)");
                    break;
                }
                Err(e) => {
                    warn!(error = %e, "Error reading stdin");
                    break;
                }
            }
        }

        connected.store(false, Ordering::Relaxed);
    }

    async fn stdout_writer_loop(
        mut outbound_rx: mpsc::UnboundedReceiver<OutboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        while connected.load(Ordering::Relaxed) {
            match outbound_rx.recv().await {
                Some(msg) => {
                    println!("\nselfclaw> {}\n", msg.content);
                }
                None => {
                    break;
                }
            }
        }
    }
}

impl Default for CliChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple intent classification based on message content.
pub fn classify_intent(content: &str) -> MessageIntent {
    let lower = content.to_lowercase();

    if lower.starts_with('/') || lower.starts_with('!') {
        return MessageIntent::Command;
    }

    if lower == "pause" || lower == "stop" || lower == "shutdown" {
        return MessageIntent::System;
    }

    if lower.ends_with('?')
        || lower.starts_with("what ")
        || lower.starts_with("how ")
        || lower.starts_with("why ")
        || lower.starts_with("when ")
        || lower.starts_with("where ")
        || lower.starts_with("who ")
    {
        return MessageIntent::Question;
    }

    MessageIntent::Chat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_intent_command() {
        assert_eq!(classify_intent("/help"), MessageIntent::Command);
        assert_eq!(classify_intent("!status"), MessageIntent::Command);
    }

    #[test]
    fn test_classify_intent_system() {
        assert_eq!(classify_intent("pause"), MessageIntent::System);
        assert_eq!(classify_intent("STOP"), MessageIntent::System);
        assert_eq!(classify_intent("shutdown"), MessageIntent::System);
    }

    #[test]
    fn test_classify_intent_question() {
        assert_eq!(
            classify_intent("What are you thinking about?"),
            MessageIntent::Question
        );
        assert_eq!(
            classify_intent("How does memory work?"),
            MessageIntent::Question
        );
        assert_eq!(
            classify_intent("Is this interesting?"),
            MessageIntent::Question
        );
    }

    #[test]
    fn test_classify_intent_chat() {
        assert_eq!(classify_intent("Hello there"), MessageIntent::Chat);
        assert_eq!(
            classify_intent("That sounds interesting"),
            MessageIntent::Chat
        );
    }

    #[test]
    fn test_cli_channel_default_prompt() {
        let ch = CliChannel::new();
        assert_eq!(ch.prompt, "you> ");
        assert!(!ch.is_connected());
    }

    #[test]
    fn test_cli_channel_custom_prompt() {
        let ch = CliChannel::new().with_prompt(">> ");
        assert_eq!(ch.prompt, ">> ");
    }

    #[tokio::test]
    async fn test_cli_channel_outbound_delivery() {
        // Test that outbound messages are properly received by the writer
        let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        let msg = OutboundMessage::new("Hello human!", ChannelKind::Cli);
        outbound_tx.send(msg).unwrap();

        let received = outbound_rx.recv().await.unwrap();
        assert_eq!(received.content, "Hello human!");
        assert_eq!(received.target_channel, ChannelKind::Cli);
    }

    #[test]
    fn test_cli_channel_stop() {
        let ch = CliChannel::new();
        ch.connected.store(true, Ordering::Relaxed);
        assert!(ch.is_connected());

        ch.stop();
        assert!(!ch.is_connected());
    }
}
