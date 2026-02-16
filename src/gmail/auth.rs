//! OAuth 2.0 authentication for Gmail API.
//!
//! Handles device flow authentication and token management.

#[cfg(feature = "gmail")]
use yup_oauth2::{authenticator::Authenticator, DeviceFlowAuthenticator};

/// Gmail API OAuth scopes
pub const GMAIL_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/gmail.send",
    "https://www.googleapis.com/auth/gmail.modify",
];

/// Initialize Gmail OAuth authenticator (stub - returns error until implemented)
#[cfg(feature = "gmail")]
pub async fn create_authenticator(
    _credentials_path: &std::path::Path,
) -> anyhow::Result<()> {
    // TODO: Implement device flow authentication
    // - Load OAuth client secret from credentials_path
    // - Create DeviceFlowAuthenticator with appropriate connector
    // - Store refresh token in secrets vault
    // - Return authenticator for API calls
    //
    // Example:
    // let secret = oauth2::read_application_secret(credentials_path).await?;
    // let auth = DeviceFlowAuthenticator::builder(secret)
    //     .persist_tokens_to_disk("token_cache.json")
    //     .build()
    //     .await?;
    // Ok(auth)
    anyhow::bail!("Gmail OAuth not yet implemented")
}
