//! Gmail API client for reading and sending emails.

use super::Email;

/// Gmail API client
pub struct GmailClient {
    // TODO: Add authenticator and HTTP client
}

impl GmailClient {
    /// Create new Gmail client with OAuth authenticator
    pub async fn new() -> anyhow::Result<Self> {
        // TODO: Initialize with authenticator from auth.rs
        anyhow::bail!("GmailClient not yet implemented")
    }

    /// Read email by message ID
    pub async fn read_email(&self, _message_id: &str) -> anyhow::Result<Email> {
        // TODO: GET https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}
        // - Fetch message details
        // - Parse headers (From, To, Subject)
        // - Extract plain text and HTML body
        // - Return Email struct
        anyhow::bail!("read_email not yet implemented")
    }

    /// Send email
    pub async fn send_email(
        &self,
        _to: &str,
        _subject: &str,
        _body: &str,
        _thread_id: Option<&str>,
    ) -> anyhow::Result<String> {
        // TODO: POST https://gmail.googleapis.com/gmail/v1/users/me/messages/send
        // - Construct RFC 2822 email message
        // - Base64url encode
        // - Send via API
        // - Return message ID
        anyhow::bail!("send_email not yet implemented")
    }

    /// List recent emails matching query
    pub async fn list_emails(&self, _query: &str, _max_results: u32) -> anyhow::Result<Vec<Email>> {
        // TODO: GET https://gmail.googleapis.com/gmail/v1/users/me/messages?q={query}
        // - Search with query
        // - Fetch full message details for each result
        // - Return list of emails
        anyhow::bail!("list_emails not yet implemented")
    }
}
