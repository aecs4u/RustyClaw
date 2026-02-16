//! Gmail integration module for email automation via Gmail API.
//!
//! This module provides:
//! - OAuth 2.0 authentication with Gmail API
//! - Reading and sending emails
//! - Webhook endpoint for Cloud Pub/Sub notifications
//! - Email processing and routing
//!
//! Requires the `gmail` feature to be enabled.

#[cfg(feature = "gmail")]
pub mod auth;
#[cfg(feature = "gmail")]
pub mod client;
#[cfg(feature = "gmail")]
pub mod webhook;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Gmail integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailConfig {
    /// Whether Gmail integration is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Path to OAuth 2.0 credentials JSON file
    pub credentials_path: Option<PathBuf>,

    /// Port for webhook HTTP server
    #[serde(default = "default_webhook_port")]
    pub webhook_port: u16,

    /// Path for webhook endpoint (e.g., "/gmail/webhook")
    #[serde(default = "default_webhook_path")]
    pub webhook_path: String,

    /// Google Cloud Pub/Sub topic name
    pub pubsub_topic: Option<String>,

    /// Google Cloud Pub/Sub subscription name
    pub pubsub_subscription: Option<String>,

    /// Email processing filters
    #[serde(default)]
    pub filters: Vec<EmailFilter>,
}

impl Default for GmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            credentials_path: None,
            webhook_port: default_webhook_port(),
            webhook_path: default_webhook_path(),
            pubsub_topic: None,
            pubsub_subscription: None,
            filters: Vec::new(),
        }
    }
}

fn default_webhook_port() -> u16 {
    8080
}

fn default_webhook_path() -> String {
    "/gmail/webhook".to_string()
}

/// Email filter for routing and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFilter {
    /// Filter name/description
    pub name: String,

    /// Match emails from this address (supports wildcards)
    pub from: Option<String>,

    /// Match emails with this label
    pub label: Option<String>,

    /// Match emails with this subject pattern
    pub subject: Option<String>,

    /// Action to take when filter matches
    pub action: EmailAction,
}

/// Action to take when email filter matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmailAction {
    /// Automatically respond with template
    AutoRespond { template: String },

    /// Forward to specific session
    NotifySession { session_id: String },

    /// Ignore the email
    Ignore,
}

/// Email message representation
#[derive(Debug, Clone)]
pub struct Email {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body_plain: String,
    pub body_html: Option<String>,
    pub timestamp: i64,
}

// ── Stub implementations (to be completed) ──────────────────────────────

#[cfg(feature = "gmail")]
pub async fn init_gmail(_config: &GmailConfig) -> anyhow::Result<()> {
    // TODO: Initialize Gmail client with OAuth
    anyhow::bail!("Gmail integration not yet fully implemented")
}

#[cfg(feature = "gmail")]
pub async fn start_webhook(_config: &GmailConfig) -> anyhow::Result<()> {
    // TODO: Start webhook HTTP server
    anyhow::bail!("Gmail webhook server not yet implemented")
}

#[cfg(not(feature = "gmail"))]
pub async fn init_gmail(_config: &GmailConfig) -> anyhow::Result<()> {
    anyhow::bail!("Gmail feature not enabled. Recompile with --features gmail")
}

#[cfg(not(feature = "gmail"))]
pub async fn start_webhook(_config: &GmailConfig) -> anyhow::Result<()> {
    anyhow::bail!("Gmail feature not enabled. Recompile with --features gmail")
}
