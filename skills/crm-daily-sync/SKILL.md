---
name: crm-daily-sync
description: "Daily synchronization of Gmail and Google Calendar to extract and track contacts in a local CRM database."
version: "0.1.0"
author: "aecs4u"
linked_secrets:
  - gmail_oauth_token
  - google_calendar_oauth_token
metadata:
  openclaw:
    emoji: "📇"
    primaryEnv: GMAIL_CLIENT_ID
    requires:
      env:
        - GMAIL_CLIENT_ID
        - GMAIL_CLIENT_SECRET
    config:
      - google_calendar_id
routines:
  - name: crm-daily-sync
    trigger: cron
    expression: "0 0 6 * * * *"
    description: "Run every day at 06:00"
---

# CRM Daily Sync

Synchronises Gmail and Google Calendar to extract and track contacts in a local Personal CRM SQLite database.

## What This Skill Does

1. Fetches the last 24 hours of Gmail messages and calendar events.
2. Extracts unique contacts (name, email, organisation) using NER and heuristics.
3. Upserts contacts into the local `crm.db` SQLite database.
4. Stores interaction history: email threads and meeting summaries per contact.
5. Flags stale contacts (no interaction in > 30 days) for follow-up.

## Setup

### Required Secrets

Add these to your RustyClaw vault before enabling:

```
rustyclaw vault set gmail_oauth_token       # OAuth2 access token for Gmail
rustyclaw vault set google_calendar_oauth_token  # OAuth2 token for Calendar
```

Then link them to this skill:

```
rustyclaw skills link-secret crm-daily-sync gmail_oauth_token
rustyclaw skills link-secret crm-daily-sync google_calendar_oauth_token
```

### Required Config (`config.toml`)

```toml
[google]
client_id     = "your-client-id.apps.googleusercontent.com"
client_secret = "your-client-secret"
calendar_id   = "primary"
```

### Schedule as a Routine

```
rustyclaw routines add crm-daily-sync \
  --trigger cron \
  --cron "0 0 6 * * * *" \
  --prompt "Run crm-daily-sync"
```

## Instructions for Agent

When executing this skill:

1. Use the `gmail_list_messages` tool to fetch messages from the last 24 hours
   (`after: yesterday`).
2. For each thread, extract sender/recipient names and email addresses.
3. Use the `calendar_list_events` tool to fetch today's and tomorrow's events.
4. For each event, extract attendee names and email addresses.
5. Upsert every new contact into the SQLite CRM table `contacts`:
   - Fields: `email` (PK), `name`, `organisation`, `last_seen`, `source`
6. Insert a row into `interactions` for each email/meeting:
   - Fields: `contact_email`, `type` (email|meeting), `subject`, `ts`, `summary`
7. Generate a brief deduplication report:
   - How many new contacts added
   - How many existing contacts updated
   - Any stale contacts (> 30 days since `last_seen`)
8. Store the embeddings for each interaction summary using `embed_text` if
   the local embeddings skill is active.

## Database Schema

The skill expects (and will create if absent) these tables:

```sql
CREATE TABLE IF NOT EXISTS contacts (
    email        TEXT PRIMARY KEY,
    name         TEXT,
    organisation TEXT,
    last_seen    INTEGER,  -- unix timestamp
    source       TEXT      -- 'gmail' | 'calendar' | 'manual'
);

CREATE TABLE IF NOT EXISTS interactions (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_email TEXT REFERENCES contacts(email),
    type          TEXT,    -- 'email' | 'meeting'
    subject       TEXT,
    ts            INTEGER, -- unix timestamp
    summary       TEXT
);
```

## Dependencies

- `knowledge-base-add` (optional — saves meeting summaries to KB)
- Gmail OAuth credentials
- Google Calendar API access
