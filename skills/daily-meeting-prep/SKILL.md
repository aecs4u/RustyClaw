---
name: daily-meeting-prep
description: "Automated pre-meeting briefings for today's calendar events using CRM contact history and recent email threads."
version: "0.1.0"
author: "aecs4u"
linked_secrets:
  - gmail_oauth_token
  - google_calendar_oauth_token
metadata:
  openclaw:
    emoji: "📅"
    primaryEnv: GMAIL_CLIENT_ID
    requires:
      env:
        - GMAIL_CLIENT_ID
        - GMAIL_CLIENT_SECRET
      config:
        - google_calendar_id
routines:
  - name: daily-meeting-prep
    trigger: cron
    expression: "0 0 7 * * * *"
    description: "Run every day at 07:00 before meetings start"
---

# Daily Meeting Prep

Generates an automated daily briefing for each calendar event scheduled today,
combining CRM contact history and recent email threads for each attendee.

## What This Skill Does

1. Fetches today's calendar events from Google Calendar.
2. For each event, retrieves all attendees from the CRM database.
3. For each attendee, fetches:
   - Recent interaction history (last 5 interactions from `crm.db`)
   - Any open email threads in the last 7 days
4. Produces a concise briefing per meeting:
   - Meeting title, time, attendees
   - Per-attendee: last contact date, topics discussed, open items
   - Suggested talking points based on recent context

## Setup

Requires `crm-daily-sync` to have run beforehand (schedule it at 06:00, this
skill at 07:00).

### Schedule as a Routine

```
rustyclaw routines add daily-meeting-prep \
  --trigger cron \
  --cron "0 0 7 * * * *" \
  --prompt "Run daily-meeting-prep"
```

### Required Secrets

```
rustyclaw vault set gmail_oauth_token
rustyclaw vault set google_calendar_oauth_token
rustyclaw skills link-secret daily-meeting-prep gmail_oauth_token
rustyclaw skills link-secret daily-meeting-prep google_calendar_oauth_token
```

## Instructions for Agent

When executing this skill:

1. Call `calendar_list_events` with `time_min: today 00:00`, `time_max: today 23:59`.
2. For each event with at least one external attendee:
   a. Look up each attendee in `contacts` by email.
   b. Fetch their last 5 rows from `interactions` ordered by `ts DESC`.
   c. Use `gmail_search_threads` with `from:attendee_email newer_than:7d` to
      find recent threads.
3. Compose a briefing block for each meeting:
   ```
   ## 14:00 – Quarterly Review (45 min)
   Attendees: Alice Brown (Acme), Bob Lee (internal)

   **Alice Brown** (alice@acme.com)
   Last seen: 2025-01-28 via email "Q4 deliverables"
   Open threads: "Integration timeline" (3 messages, unanswered)
   Suggested: Follow up on the integration timeline question.

   **Bob Lee** — internal, no open items.
   ```
4. Output the full briefing as markdown to stdout.
5. If any meeting has an attendee not in the CRM, note them as "new contact —
   no history available".

## Dependencies

- `crm-daily-sync` (must run before this skill, typically at 06:00)
- Gmail OAuth and Google Calendar API credentials
