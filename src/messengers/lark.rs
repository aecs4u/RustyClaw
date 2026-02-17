//! Feishu/Lark messenger.
//!
//! Supports:
//! - Incoming webhook mode (`webhook_url`)
//! - Bot API mode (`token` + recipient chat id)
//!
//! Inbound messaging on Feishu/Lark is webhook/event-stream based, so
//! `receive_messages` currently returns no messages.

use crate::messengers::{Message, Messenger, SendOptions};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

const LARK_API_BASE: &str = "https://open.feishu.cn/open-apis";

#[derive(Debug, Deserialize)]
struct LarkSendResponse {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    data: Option<LarkSendData>,
}

#[derive(Debug, Deserialize)]
struct LarkSendData {
    #[serde(default)]
    message_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LarkConfig {
    pub token: Option<String>,
    pub webhook_url: Option<String>,
    pub default_chat_id: Option<String>,
    pub api_base: String,
}

impl Default for LarkConfig {
    fn default() -> Self {
        Self {
            token: None,
            webhook_url: None,
            default_chat_id: None,
            api_base: LARK_API_BASE.to_string(),
        }
    }
}

pub struct LarkMessenger {
    name: String,
    config: LarkConfig,
    http: reqwest::Client,
    connected: bool,
}

impl LarkMessenger {
    pub fn new(name: String, config: LarkConfig) -> Self {
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

    fn resolve_chat_id(&self, recipient: &str) -> Result<String> {
        if !recipient.trim().is_empty() {
            return Ok(recipient.to_string());
        }
        self.config
            .default_chat_id
            .clone()
            .context("Lark requires recipient chat id or default_chat_id")
    }
}

#[async_trait]
impl Messenger for LarkMessenger {
    fn name(&self) -> &str {
        &self.name
    }

    fn messenger_type(&self) -> &str {
        "lark"
    }

    async fn initialize(&mut self) -> Result<()> {
        if self.config.webhook_url.is_some() {
            self.connected = true;
            return Ok(());
        }

        // In bot API mode we only require a token at initialization.
        let _token = self
            .config
            .token
            .as_ref()
            .context("Lark API mode requires token")?;

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
        if let Some(webhook_url) = &self.config.webhook_url {
            let resp = self
                .http
                .post(webhook_url)
                .json(&serde_json::json!({
                    "msg_type": "text",
                    "content": {
                        "text": opts.content,
                    }
                }))
                .send()
                .await
                .context("Failed to send Lark webhook message")?;

            if !resp.status().is_success() {
                anyhow::bail!("Lark webhook send failed: {}", resp.status());
            }

            return Ok(format!("lark-{}", chrono::Utc::now().timestamp_millis()));
        }

        let token = self
            .config
            .token
            .as_ref()
            .context("Lark token is not configured")?;
        let chat_id = self.resolve_chat_id(opts.recipient)?;

        let url = format!("{}/im/v1/messages?receive_id_type=chat_id", self.api_base());
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json; charset=utf-8")
            .header("X-Request-ID", format!("rustyclaw-{}", chrono::Utc::now().timestamp_millis()))
            .json(&serde_json::json!({
                "receive_id": chat_id,
                "msg_type": "text",
                "content": serde_json::json!({ "text": opts.content }).to_string(),
            }))
            .send()
            .await
            .context("Failed to send Lark API message")?;

        if !resp.status().is_success() {
            anyhow::bail!("Lark API send failed: {}", resp.status());
        }

        let body: LarkSendResponse = resp
            .json()
            .await
            .context("Failed to parse Lark send response")?;

        if body.code != 0 {
            anyhow::bail!(
                "Lark API returned error code {}: {}",
                body.code,
                body.msg.unwrap_or_else(|| "unknown".to_string())
            );
        }

        Ok(body
            .data
            .and_then(|d| d.message_id)
            .unwrap_or_else(|| format!("lark-{}", chrono::Utc::now().timestamp_millis())))
    }

    async fn receive_messages(&self) -> Result<Vec<Message>> {
        // Lark bot events are delivered via webhooks.
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
    fn test_lark_type() {
        let m = LarkMessenger::new("lark-main".to_string(), LarkConfig::default());
        assert_eq!(m.messenger_type(), "lark");
    }
}
