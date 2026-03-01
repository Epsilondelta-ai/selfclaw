use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use selfclaw_config::DiscordConfig;

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord channel for communicating with humans via a Discord bot.
///
/// Uses the Discord REST API via reqwest:
/// - Receives messages by polling with GET /channels/{id}/messages
/// - Sends messages with POST /channels/{id}/messages
pub struct DiscordChannel {
    config: DiscordConfig,
    connected: Arc<AtomicBool>,
    client: reqwest::Client,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            connected: Arc::new(AtomicBool::new(false)),
            client: reqwest::Client::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Start the Discord channel.
    ///
    /// Spawns tasks for polling messages and sending responses.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        if self.config.bot_token.is_empty() {
            return Err(ChannelError::AuthFailed(
                "Discord bot_token is empty".to_string(),
            ));
        }

        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        self.connected.store(true, Ordering::Relaxed);

        // Spawn message poller
        let connected_poller = self.connected.clone();
        let client = self.client.clone();
        let token = self.config.bot_token.clone();
        let channel_ids = self.config.allowed_channel_ids.clone();
        tokio::spawn(async move {
            Self::poll_messages(client, &token, &channel_ids, inbound_tx, connected_poller).await;
        });

        // Spawn outbound sender
        let connected_sender = self.connected.clone();
        let client = self.client.clone();
        let token = self.config.bot_token.clone();
        tokio::spawn(async move {
            Self::send_messages(client, &token, outbound_rx, connected_sender).await;
        });

        info!("Discord channel started");

        Ok(ChannelHandle {
            kind: ChannelKind::Discord,
            name: "discord".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    pub fn stop(&self) {
        self.connected.store(false, Ordering::Relaxed);
        info!("Discord channel stopped");
    }

    /// Poll Discord channels for new messages.
    async fn poll_messages(
        client: reqwest::Client,
        token: &str,
        channel_ids: &[String],
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        let mut last_message_ids: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        while connected.load(Ordering::Relaxed) {
            for channel_id in channel_ids {
                let url = format!(
                    "{}/channels/{}/messages?limit=10",
                    DISCORD_API_BASE, channel_id
                );

                let mut req = client
                    .get(&url)
                    .header("Authorization", format!("Bot {}", token));

                if let Some(after) = last_message_ids.get(channel_id) {
                    req = req.query(&[("after", after.as_str())]);
                }

                match req.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(messages) = resp.json::<Vec<serde_json::Value>>().await {
                            for msg in messages.iter().rev() {
                                let msg_id = msg["id"].as_str().unwrap_or_default().to_string();
                                let content =
                                    msg["content"].as_str().unwrap_or_default().to_string();
                                let author = msg["author"]["username"]
                                    .as_str()
                                    .unwrap_or("unknown")
                                    .to_string();

                                // Skip bot messages
                                if msg["author"]["bot"].as_bool().unwrap_or(false) {
                                    continue;
                                }

                                if content.is_empty() {
                                    continue;
                                }

                                last_message_ids.insert(channel_id.clone(), msg_id.clone());

                                let inbound = InboundMessage {
                                    id: format!("discord-{}", msg_id),
                                    content,
                                    metadata: MessageMetadata {
                                        timestamp: Utc::now().to_rfc3339(),
                                        sender: author,
                                        channel: ChannelKind::Discord,
                                        intent: MessageIntent::Chat,
                                        conversation_id: Some(channel_id.clone()),
                                    },
                                };

                                if inbound_tx.send(inbound).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Ok(resp) => {
                        warn!(status = %resp.status(), "Discord API error");
                    }
                    Err(e) => {
                        warn!(error = %e, "Discord API request failed");
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    /// Send outbound messages to Discord.
    async fn send_messages(
        client: reqwest::Client,
        token: &str,
        mut outbound_rx: mpsc::UnboundedReceiver<OutboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        while connected.load(Ordering::Relaxed) {
            match outbound_rx.recv().await {
                Some(msg) => {
                    if let Some(ref channel_id) = msg.conversation_id {
                        let url = format!("{}/channels/{}/messages", DISCORD_API_BASE, channel_id);

                        let body = serde_json::json!({
                            "content": msg.content
                        });

                        match client
                            .post(&url)
                            .header("Authorization", format!("Bot {}", token))
                            .json(&body)
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                info!("Message sent to Discord channel {}", channel_id);
                            }
                            Ok(resp) => {
                                warn!(
                                    status = %resp.status(),
                                    "Failed to send Discord message"
                                );
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to send Discord message");
                            }
                        }
                    } else {
                        warn!("Discord outbound message missing conversation_id (channel_id)");
                    }
                }
                None => break,
            }
        }
    }
}

/// Build a DiscordChannel from config, returning None if not enabled.
pub fn from_config(config: &DiscordConfig) -> Option<DiscordChannel> {
    if config.enabled {
        Some(DiscordChannel::new(config.clone()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_channel_creation() {
        let config = DiscordConfig {
            enabled: true,
            bot_token: "test-token".to_string(),
            allowed_channel_ids: vec!["123".to_string()],
        };
        let ch = DiscordChannel::new(config);
        assert!(!ch.is_connected());
    }

    #[test]
    fn test_discord_from_config_disabled() {
        let config = DiscordConfig::default();
        assert!(from_config(&config).is_none());
    }

    #[test]
    fn test_discord_from_config_enabled() {
        let config = DiscordConfig {
            enabled: true,
            bot_token: "token".to_string(),
            allowed_channel_ids: vec![],
        };
        assert!(from_config(&config).is_some());
    }

    #[tokio::test]
    async fn test_discord_start_fails_without_token() {
        let config = DiscordConfig {
            enabled: true,
            bot_token: String::new(),
            allowed_channel_ids: vec![],
        };
        let ch = DiscordChannel::new(config);
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = ch.start(tx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bot_token"));
    }

    #[test]
    fn test_discord_stop() {
        let config = DiscordConfig {
            enabled: true,
            bot_token: "token".to_string(),
            allowed_channel_ids: vec![],
        };
        let ch = DiscordChannel::new(config);
        ch.connected.store(true, Ordering::Relaxed);
        assert!(ch.is_connected());

        ch.stop();
        assert!(!ch.is_connected());
    }
}
