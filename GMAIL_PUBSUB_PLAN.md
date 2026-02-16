# Gmail Pub/Sub Integration Plan

## Overview

Implement Gmail webhook automation to enable RustyClaw to receive and respond to emails automatically via Gmail API and Cloud Pub/Sub.

## Architecture

### Components

1. **Gmail API Client**
   - OAuth 2.0 authentication flow
   - Read/send email capabilities
   - Label and filter management
   - Message threading support

2. **Cloud Pub/Sub Integration**
   - Subscribe to Gmail push notifications
   - Webhook endpoint to receive notifications
   - Message acknowledgment and delivery tracking

3. **Email Processing Pipeline**
   - Parse incoming email notifications
   - Extract sender, subject, body
   - Route to appropriate session/handler
   - Generate and send responses

4. **Configuration**
   - Gmail API credentials (OAuth client ID/secret)
   - Pub/Sub topic and subscription
   - Email filters and routing rules
   - Auto-response templates

## Implementation Steps

### Phase 1: Gmail API Client (Core)

**Dependencies:**
```toml
google-gmail1 = "5.0"  # Gmail API v1
yup-oauth2 = "8.0"     # OAuth 2.0
hyper = "0.14"         # HTTP client
```

**Files:**
- `src/gmail/mod.rs` - Main Gmail client
- `src/gmail/auth.rs` - OAuth flow
- `src/gmail/messages.rs` - Email read/send
- `src/config.rs` - Add GmailConfig struct

**Features:**
- [ ] OAuth 2.0 device flow for CLI auth
- [ ] Token storage in secrets vault
- [ ] Read emails (by ID, query)
- [ ] Send emails (plain text, HTML)
- [ ] Thread/conversation support

### Phase 2: Pub/Sub Webhook Endpoint

**Dependencies:**
```toml
warp = "0.3"           # Web framework for webhook
tokio = "1.0"          # Async runtime (already present)
```

**Files:**
- `src/gmail/pubsub.rs` - Pub/Sub client
- `src/gmail/webhook.rs` - HTTP webhook handler
- `src/gateway/gmail_handler.rs` - Integration with gateway

**Features:**
- [ ] Webhook HTTP server (POST /gmail/webhook)
- [ ] Verify Pub/Sub signature
- [ ] Parse notification payload
- [ ] Fetch full message via Gmail API
- [ ] Forward to gateway for processing

### Phase 3: Email Processing & Routing

**Files:**
- `src/gmail/processor.rs` - Email content extraction
- `src/gmail/router.rs` - Route emails to sessions

**Features:**
- [ ] Extract plain text/HTML from emails
- [ ] Parse sender, subject, thread ID
- [ ] Match emails to existing conversations
- [ ] Create new sessions for new threads
- [ ] Generate AI responses
- [ ] Send replies maintaining thread

### Phase 4: Configuration & Setup

**Config Example:**
```toml
[gmail]
enabled = true
credentials_path = "~/.rustyclaw/credentials/gmail_oauth.json"
webhook_port = 8080
webhook_path = "/gmail/webhook"
pubsub_topic = "projects/PROJECT_ID/topics/gmail-notifications"
pubsub_subscription = "projects/PROJECT_ID/subscriptions/rustyclaw"

[[gmail.filters]]
from = "*@example.com"
action = "auto_respond"
template = "Thanks for your email. I'll get back to you soon."

[[gmail.filters]]
label = "IMPORTANT"
action = "notify_session"
session_id = "main"
```

**Setup Commands:**
```bash
rustyclaw gmail setup          # OAuth flow
rustyclaw gmail webhook start  # Start webhook server
rustyclaw gmail test           # Send test email
```

## Complexity Estimate

- **Lines of Code:** ~1500-2000 lines
- **Dependencies:** 5-7 new crates
- **Time Estimate:** 15-20 hours
- **Prerequisites:**
  - Google Cloud Project
  - Gmail API enabled
  - OAuth 2.0 credentials
  - Cloud Pub/Sub topic/subscription configured

## Security Considerations

1. **OAuth Tokens:** Store refresh tokens in encrypted vault
2. **Webhook Endpoint:** Verify Pub/Sub signatures
3. **Email Content:** Sanitize before passing to AI
4. **Rate Limiting:** Respect Gmail API quotas
5. **PII Handling:** Email addresses, names, content

## Testing Strategy

1. **Unit Tests:** Mock Gmail API responses
2. **Integration Tests:** Real Gmail API with test account
3. **Webhook Tests:** Simulate Pub/Sub notifications
4. **E2E Tests:** Send email → receive → respond flow

## Alternatives & Simplifications

### Minimal Implementation (No Pub/Sub)
- Poll Gmail API periodically instead of webhooks
- Simpler setup (no Cloud Pub/Sub required)
- Higher latency, more API quota usage
- Still requires OAuth setup

### Email Tool Only (No Automation)
- Add `email_read` and `email_send` tools
- Agent can manually check/send emails
- No automatic responses
- Much simpler implementation (~500 LOC)

## Recommendation

Given the complexity and dependencies, consider:

1. **Start with Email Tools** - Implement basic read/send capabilities
2. **Add Polling** - Check for new emails on demand or interval
3. **Defer Pub/Sub** - Add real-time webhooks in future iteration

This provides immediate value while deferring the complex webhook infrastructure.
