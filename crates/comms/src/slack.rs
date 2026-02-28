use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use selfclaw_config::SlackConfig;

use crate::channel::{ChannelError, ChannelHandle};
use crate::message::{
    ChannelKind, InboundMessage, MessageIntent, MessageMetadata, OutboundMessage,
};

const SLACK_API_BASE: &str = "https://slack.com/api";

/// Slack channel for communicating via Slack Web API.
///
/// Uses conversations.history for polling messages and chat.postMessage for sending.
/// For production use, Slack's Socket Mode (via app_token) would be preferred for
/// real-time messaging, but this implementation uses polling for simplicity.
pub struct SlackChannel {
    config: SlackConfig,
    connected: Arc<AtomicBool>,
    client: reqwest::Client,
}

impl SlackChannel {
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            connected: Arc::new(AtomicBool::new(false)),
            client: reqwest::Client::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Start the Slack channel.
    pub fn start(
        &self,
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    ) -> Result<ChannelHandle, ChannelError> {
        if self.config.bot_token.is_empty() {
            return Err(ChannelError::AuthFailed(
                "Slack bot_token is empty".to_string(),
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

        info!("Slack channel started");

        Ok(ChannelHandle {
            kind: ChannelKind::Slack,
            name: "slack".to_string(),
            outbound_tx,
            connected: true,
        })
    }

    pub fn stop(&self) {
        self.connected.store(false, Ordering::Relaxed);
        info!("Slack channel stopped");
    }

    /// Poll Slack channels for new messages using conversations.history.
    async fn poll_messages(
        client: reqwest::Client,
        token: &str,
        channel_ids: &[String],
        inbound_tx: mpsc::UnboundedSender<InboundMessage>,
        connected: Arc<AtomicBool>,
    ) {
        let mut last_timestamps: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        while connected.load(Ordering::Relaxed) {
            for channel_id in channel_ids {
                let url = format!("{}/conversations.history", SLACK_API_BASE);

                let mut params = vec![
                    ("channel", channel_id.as_str()),
                    ("limit", "10"),
                ];

                let oldest_binding;
                if let Some(ts) = last_timestamps.get(channel_id) {
                    oldest_binding = ts.clone();
                    params.push(("oldest", &oldest_binding));
                }

                match client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .query(&params)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            if data["ok"].as_bool() != Some(true) {
                                warn!(
                                    error = data["error"].as_str().unwrap_or("unknown"),
                                    "Slack API returned error"
                                );
                                continue;
                            }

                            if let Some(messages) = data["messages"].as_array() {
                                // Messages come newest-first, reverse for chronological order
                                for msg in messages.iter().rev() {
                                    // Skip bot messages
                                    if msg.get("bot_id").is_some() {
                                        continue;
                                    }

                                    let ts = msg["ts"].as_str().unwrap_or_default().to_string();
                                    let text =
                                        msg["text"].as_str().unwrap_or_default().to_string();
                                    let user =
                                        msg["user"].as_str().unwrap_or("unknown").to_string();

                                    if text.is_empty() {
                                        continue;
                                    }

                                    last_timestamps.insert(channel_id.clone(), ts.clone());

                                    let inbound = InboundMessage {
                                        id: format!("slack-{}-{}", channel_id, ts),
                                        content: text,
                                        metadata: MessageMetadata {
                                            timestamp: Utc::now().to_rfc3339(),
                                            sender: user,
                                            channel: ChannelKind::Slack,
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
                    }
                    Ok(resp) => {
                        warn!(status = %resp.status(), "Slack API HTTP error");
                    }
                    Err(e) => {
                        warn!(error = %e, "Slack API request failed");
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    /// Send outbound messages via Slack chat.postMessage.
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
                        let url = format!("{}/chat.postMessage", SLACK_API_BASE);

                        let mut body = serde_json::json!({
                            "channel": channel_id,
                            "text": msg.content
                        });

                        // Support threading via reply_to (as thread_ts)
                        if let Some(ref thread_ts) = msg.reply_to {
                            body["thread_ts"] = serde_json::json!(thread_ts);
                        }

                        match client
                            .post(&url)
                            .header("Authorization", format!("Bearer {}", token))
                            .json(&body)
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                    if data["ok"].as_bool() != Some(true) {
                                        warn!(
                                            error = data["error"].as_str().unwrap_or("unknown"),
                                            "Slack postMessage returned error"
                                        );
                                    } else {
                                        info!("Message sent to Slack channel {}", channel_id);
                                    }
                                }
                            }
                            Ok(resp) => {
                                warn!(
                                    status = %resp.status(),
                                    "Failed to send Slack message"
                                );
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to send Slack message");
                            }
                        }
                    } else {
                        warn!("Slack outbound message missing conversation_id (channel_id)");
                    }
                }
                None => break,
            }
        }
    }
}

/// Build a SlackChannel from config, returning None if not enabled.
pub fn from_config(config: &SlackConfig) -> Option<SlackChannel> {
    if config.enabled {
        Some(SlackChannel::new(config.clone()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_channel_creation() {
        let config = SlackConfig {
            enabled: true,
            bot_token: "xoxb-test".to_string(),
            app_token: "xapp-test".to_string(),
            allowed_channel_ids: vec!["C123".to_string()],
        };
        let ch = SlackChannel::new(config);
        assert!(!ch.is_connected());
    }

    #[test]
    fn test_slack_from_config_disabled() {
        let config = SlackConfig::default();
        assert!(from_config(&config).is_none());
    }

    #[test]
    fn test_slack_from_config_enabled() {
        let config = SlackConfig {
            enabled: true,
            bot_token: "token".to_string(),
            app_token: String::new(),
            allowed_channel_ids: vec![],
        };
        assert!(from_config(&config).is_some());
    }

    #[tokio::test]
    async fn test_slack_start_fails_without_token() {
        let config = SlackConfig {
            enabled: true,
            bot_token: String::new(),
            app_token: String::new(),
            allowed_channel_ids: vec![],
        };
        let ch = SlackChannel::new(config);
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = ch.start(tx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bot_token"));
    }

    #[test]
    fn test_slack_stop() {
        let config = SlackConfig {
            enabled: true,
            bot_token: "token".to_string(),
            app_token: String::new(),
            allowed_channel_ids: vec![],
        };
        let ch = SlackChannel::new(config);
        ch.connected.store(true, Ordering::Relaxed);

        ch.stop();
        assert!(!ch.is_connected());
    }
}
