use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use selfclaw_config::TelegramConfig;

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

const TELEGRAM_API_BASE: &str = "https://api.telegram.org";

/// Telegram channel for communicating via Telegram Bot API.
///
/// Uses long polling with getUpdates for receiving messages and
/// sendMessage for sending responses.
pub struct TelegramChannel {
    config: TelegramConfig,
    connected: Arc<AtomicBool>,
    client: reqwest::Client,
}

impl TelegramChannel {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            connected: Arc::new(AtomicBool::new(false)),
            client: reqwest::Client::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Start the Telegram channel.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        if self.config.bot_token.is_empty() {
            return Err(ChannelError::AuthFailed(
                "Telegram bot_token is empty".to_string(),
            ));
        }

        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMessage>();

        self.connected.store(true, Ordering::Relaxed);

        // Spawn long-polling update loop
        let connected_poller = self.connected.clone();
        let client = self.client.clone();
        let token = self.config.bot_token.clone();
        let allowed_chats = self.config.allowed_chat_ids.clone();
        tokio::spawn(async move {
            Self::poll_updates(client, &token, &allowed_chats, inbound_tx, connected_poller).await;
        });

        // Spawn outbound sender
        let connected_sender = self.connected.clone();
        let client = self.client.clone();
        let token = self.config.bot_token.clone();
        tokio::spawn(async move {
            Self::send_messages(client, &token, outbound_rx, connected_sender).await;
        });

        info!("Telegram channel started");

        Ok(ChannelHandle {
            kind: ChannelKind::Telegram,
            name: "telegram".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    pub fn stop(&self) {
        self.connected.store(false, Ordering::Relaxed);
        info!("Telegram channel stopped");
    }

    /// Long-poll for updates from Telegram Bot API.
    async fn poll_updates(
        client: reqwest::Client,
        token: &str,
        allowed_chats: &[i64],
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        let mut offset: i64 = 0;

        while connected.load(Ordering::Relaxed) {
            let url = format!("{}/bot{}/getUpdates", TELEGRAM_API_BASE, token);

            let body = serde_json::json!({
                "offset": offset,
                "timeout": 30,
                "allowed_updates": ["message"]
            });

            match client.post(&url).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(updates) = data["result"].as_array() {
                            for update in updates {
                                let update_id = update["update_id"].as_i64().unwrap_or(0);
                                offset = update_id + 1;

                                if let Some(msg) = update.get("message") {
                                    let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);

                                    // Filter by allowed chat IDs if configured
                                    if !allowed_chats.is_empty()
                                        && !allowed_chats.contains(&chat_id)
                                    {
                                        continue;
                                    }

                                    let text =
                                        msg["text"].as_str().unwrap_or_default().to_string();
                                    if text.is_empty() {
                                        continue;
                                    }

                                    let sender_name = msg["from"]["first_name"]
                                        .as_str()
                                        .unwrap_or("unknown")
                                        .to_string();

                                    let msg_id = msg["message_id"].as_i64().unwrap_or(0);

                                    let inbound = InboundMessage {
                                        id: format!("tg-{}-{}", chat_id, msg_id),
                                        content: text,
                                        metadata: MessageMetadata {
                                            timestamp: Utc::now().to_rfc3339(),
                                            sender: sender_name,
                                            channel: ChannelKind::Telegram,
                                            intent: MessageIntent::Chat,
                                            conversation_id: Some(chat_id.to_string()),
                                        },
                                    };

                                    if inbound_tx.send(inbound).is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(resp) => {
                    warn!(status = %resp.status(), "Telegram API error");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(e) => {
                    warn!(error = %e, "Telegram API request failed");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Send outbound messages via Telegram sendMessage API.
    async fn send_messages(
        client: reqwest::Client,
        token: &str,
        mut outbound_rx: mpsc::UnboundedReceiver<OutboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        while connected.load(Ordering::Relaxed) {
            match outbound_rx.recv().await {
                Some(msg) => {
                    if let Some(ref chat_id) = msg.conversation_id {
                        let url = format!("{}/bot{}/sendMessage", TELEGRAM_API_BASE, token);

                        let mut body = serde_json::json!({
                            "chat_id": chat_id,
                            "text": msg.content
                        });

                        // Support reply-to
                        if let Some(ref reply_id) = msg.reply_to {
                            if let Ok(id) = reply_id
                                .strip_prefix(&format!("tg-{}-", chat_id))
                                .unwrap_or(reply_id)
                                .parse::<i64>()
                            {
                                body["reply_to_message_id"] = serde_json::json!(id);
                            }
                        }

                        match client.post(&url).json(&body).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                info!("Message sent to Telegram chat {}", chat_id);
                            }
                            Ok(resp) => {
                                warn!(
                                    status = %resp.status(),
                                    "Failed to send Telegram message"
                                );
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to send Telegram message");
                            }
                        }
                    } else {
                        warn!("Telegram outbound message missing conversation_id (chat_id)");
                    }
                }
                None => break,
            }
        }
    }
}

/// Build a TelegramChannel from config, returning None if not enabled.
pub fn from_config(config: &TelegramConfig) -> Option<TelegramChannel> {
    if config.enabled {
        Some(TelegramChannel::new(config.clone()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_channel_creation() {
        let config = TelegramConfig {
            enabled: true,
            bot_token: "123:ABC".to_string(),
            allowed_chat_ids: vec![12345],
        };
        let ch = TelegramChannel::new(config);
        assert!(!ch.is_connected());
    }

    #[test]
    fn test_telegram_from_config_disabled() {
        let config = TelegramConfig::default();
        assert!(from_config(&config).is_none());
    }

    #[test]
    fn test_telegram_from_config_enabled() {
        let config = TelegramConfig {
            enabled: true,
            bot_token: "token".to_string(),
            allowed_chat_ids: vec![],
        };
        assert!(from_config(&config).is_some());
    }

    #[tokio::test]
    async fn test_telegram_start_fails_without_token() {
        let config = TelegramConfig {
            enabled: true,
            bot_token: String::new(),
            allowed_chat_ids: vec![],
        };
        let ch = TelegramChannel::new(config);
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = ch.start(tx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bot_token"));
    }

    #[test]
    fn test_telegram_stop() {
        let config = TelegramConfig {
            enabled: true,
            bot_token: "token".to_string(),
            allowed_chat_ids: vec![],
        };
        let ch = TelegramChannel::new(config);
        ch.connected.store(true, Ordering::Relaxed);

        ch.stop();
        assert!(!ch.is_connected());
    }
}
