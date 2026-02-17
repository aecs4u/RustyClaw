//! LINE messenger.
//!
//! Supports outbound push messaging through LINE Messaging API.
//! Inbound messaging is webhook-driven, so `receive_messages` currently
//! returns no messages.

use crate::messengers::{Message, Messenger, SendOptions};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

const LINE_API_BASE: &str = "https://api.line.me/v2/bot";

#[derive(Debug, Deserialize)]
struct LineSendResponse {
    #[serde(default)]
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub token: Option<String>,
    pub default_to: Option<String>,
    pub api_base: String,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            token: None,
            default_to: None,
            api_base: LINE_API_BASE.to_string(),
        }
    }
}

pub struct LineMessenger {
    name: String,
    config: LineConfig,
    http: reqwest::Client,
    connected: bool,
}

impl LineMessenger {
    pub fn new(name: String, config: LineConfig) -> Self {
        Self {
            name,
            config,
            http: reqwest::Client::new(),
            connected: false,
        }
    }

    fn api_base(&self) -> String {
        self.config.api_base.trim_end_matches('/').to_string()
    }

    fn resolve_to(&self, recipient: &str) -> Result<String> {
        if !recipient.trim().is_empty() {
            return Ok(recipient.to_string());
        }
        self.config
            .default_to
            .clone()
            .context("LINE requires recipient user/group/room id or default_to")
    }
}

#[async_trait]
impl Messenger for LineMessenger {
    fn name(&self) -> &str {
        &self.name
    }

    fn messenger_type(&self) -> &str {
        "line"
    }

    async fn initialize(&mut self) -> Result<()> {
        let _token = self
            .config
            .token
            .as_ref()
            .context("LINE requires token (channel access token)")?;
        self.connected = true;
        Ok(())
    }

    async fn send_message(&self, recipient: &str, content: &str) -> Result<String> {
        self.send_message_with_options(SendOptions {
            recipient,
            content,
            ..Default::default()
        })
        .await
    }

    async fn send_message_with_options(&self, opts: SendOptions<'_>) -> Result<String> {
        let token = self
            .config
            .token
            .as_ref()
            .context("LINE token is not configured")?;
        let to = self.resolve_to(opts.recipient)?;

        let url = format!("{}/message/push", self.api_base());
        let resp = self
            .http
            .post(&url)
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "to": to,
                "messages": [
                    {
                        "type": "text",
                        "text": opts.content,
                    }
                ],
            }))
            .send()
            .await
            .context("Failed to send LINE push message")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("LINE send failed ({}): {}", status, text);
        }

        // LINE push endpoint usually returns 200 with empty body.
        let _maybe: Result<LineSendResponse, _> = resp.json().await;
        Ok(format!("line-{}", chrono::Utc::now().timestamp_millis()))
    }

    async fn receive_messages(&self) -> Result<Vec<Message>> {
        // LINE inbound messages are delivered via webhook callbacks.
        Ok(Vec::new())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_type() {
        let m = LineMessenger::new("line-main".to_string(), LineConfig::default());
        assert_eq!(m.messenger_type(), "line");
    }
}

