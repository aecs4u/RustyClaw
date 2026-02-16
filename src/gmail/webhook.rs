//! Webhook HTTP server for Gmail Pub/Sub notifications.

use super::GmailConfig;

#[cfg(feature = "gmail")]
use axum::{routing::post, Router};

/// Start webhook HTTP server to receive Pub/Sub notifications
#[cfg(feature = "gmail")]
pub async fn start_webhook_server(_config: &GmailConfig) -> anyhow::Result<()> {
    // TODO: Create axum Router with POST handler
    // - Verify Pub/Sub message signature
    // - Parse notification payload
    // - Extract history ID
    // - Fetch new messages via Gmail API
    // - Process and route emails
    // - Send acknowledgment
    anyhow::bail!("Webhook server not yet implemented")
}

/// Handle incoming Pub/Sub notification
#[cfg(feature = "gmail")]
async fn handle_pubsub_notification(
    // body: axum::Json<PubSubMessage>,
) -> axum::response::Result<String> {
    // TODO: Process notification
    Ok("OK".to_string())
}

/// Pub/Sub message structure
#[cfg(feature = "gmail")]
#[derive(serde::Deserialize)]
struct PubSubMessage {
    message: PubSubMessageData,
    subscription: String,
}

#[cfg(feature = "gmail")]
#[derive(serde::Deserialize)]
struct PubSubMessageData {
    data: String, // Base64-encoded
    message_id: String,
    publish_time: String,
}
