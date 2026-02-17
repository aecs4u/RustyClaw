---
name: crm-search
description: "Hybrid SQL + semantic search for contacts and interaction history in the local Personal CRM database."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🔍"
    requires:
      bins: []
      env: []
---

# CRM Search

Provides hybrid SQL and semantic search over contacts and interaction history stored by `crm-daily-sync`.

## What This Skill Does

- Searches contacts by name, email, or organisation (exact and fuzzy).
- Searches interaction history by topic, keyword, or semantic similarity.
- Returns ranked results combining keyword relevance and embedding distance.
- Supports filters: date range, interaction type (email/meeting), organisation.

## Instructions for Agent

When the user asks to find, look up, or search for a contact or conversation:

1. Parse the query for:
   - Named entity (person, company) → SQL `LIKE '%name%'` on `contacts`
   - Topic or abstract description → semantic search using `search_embeddings`
   - Date context ("last week", "in January") → filter `interactions.ts`
2. Run the SQL query first (fast path):
   ```sql
   SELECT c.name, c.email, c.organisation, c.last_seen
   FROM contacts c
   WHERE c.name LIKE '%{query}%'
      OR c.email LIKE '%{query}%'
      OR c.organisation LIKE '%{query}%'
   ORDER BY c.last_seen DESC
   LIMIT 10;
   ```
3. If SQL returns fewer than 3 results and local embeddings are available,
   run semantic search over `interactions.summary` using `search_embeddings`.
4. Merge and deduplicate results; rank by recency and relevance score.
5. Present results as a formatted list:
   - Contact name, email, organisation
   - Last seen date
   - Most recent interaction summary (truncated to 150 chars)
6. If no results found, suggest running `crm-daily-sync` to refresh data.

## Example Queries

- "Find John Smith" → SQL name search
- "Who did I meet about the API integration?" → semantic search
- "Contacts at Acme Corp last month" → SQL + date filter
- "Anyone I haven't spoken to in 30 days?" → stale contact query

## Dependencies

- `crm-daily-sync` must have run at least once to populate the database.
- Local embeddings skill (optional, for semantic search).
