---
name: weekly-memory-synthesis
description: "Weekly consolidation of daily notes and interaction logs into long-term structured memory, pruning redundant entries and surfacing durable insights."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🧠"
    requires:
      bins: []
      env: []
routines:
  - name: weekly-memory-synthesis
    trigger: cron
    expression: "0 0 22 * * SUN *"
    description: "Run every Sunday at 22:00"
---

# Weekly Memory Synthesis

Consolidates the past week's daily notes, interaction logs, and knowledge base
additions into durable long-term memory entries, pruning ephemeral details while
preserving key facts, decisions, and relationships.

## What This Skill Does

1. **Aggregates** all structured memory entries, KB snippets, and CRM
   interactions from the past 7 days.
2. **Deduplicates** semantically similar entries (cosine similarity > 0.90).
3. **Promotes** high-signal entries to permanent memory with updated timestamps.
4. **Prunes** entries marked as ephemeral that are now older than 14 days.
5. **Synthesises** a weekly digest: top facts, decisions, and contacts
   encountered this week.
6. **Archives** the raw weekly data to a yearly archive table before pruning.

## Setup

### Schedule as a Routine

```
rustyclaw routines add weekly-memory-synthesis \
  --trigger cron \
  --cron "0 0 22 * * SUN *" \
  --prompt "Run weekly-memory-synthesis"
```

## Instructions for Agent

When executing this skill:

### Step 1 — Gather This Week's Data

```sql
-- Structured memory entries from this week
SELECT id, key, value, importance, created_at
FROM memory_entries
WHERE created_at >= unixepoch('now', '-7 days')
ORDER BY importance DESC;

-- KB items added this week
SELECT id, title, summary, tags, added_at
FROM kb_items
WHERE added_at >= unixepoch('now', '-7 days');

-- CRM interactions this week
SELECT c.name, i.type, i.subject, i.summary, i.ts
FROM interactions i
JOIN contacts c ON c.email = i.contact_email
WHERE i.ts >= unixepoch('now', '-7 days')
ORDER BY i.ts DESC;
```

### Step 2 — Deduplicate Memory Entries

For each pair of memory entries, compute embedding similarity.
If similarity > 0.90, merge: keep the entry with higher `importance`;
append the other's value as a note; delete the duplicate.

### Step 3 — Promote High-Signal Entries

Entries that appear in 3+ days of the week or have `importance >= 4`
are promoted: set `permanent = 1`, update `last_confirmed = now`.

### Step 4 — Prune Ephemeral Entries

```sql
DELETE FROM memory_entries
WHERE permanent = 0
  AND created_at < unixepoch('now', '-14 days')
  AND importance < 3;
```

Before deleting, copy rows to `memory_archive`:
```sql
INSERT INTO memory_archive SELECT *, unixepoch() AS archived_at FROM memory_entries WHERE ...;
```

### Step 5 — Synthesise Weekly Digest

Write a brief structured summary to the knowledge base:

```markdown
## Week of {YYYY-MM-DD}

### Key Facts Learned
{top 5 new permanent memory entries}

### Decisions Made
{entries tagged 'decision' or 'commitment'}

### Notable Contacts
{top 5 contacts by interaction frequency this week}

### KB Additions
{n} items saved — top 3: {titles}

### Memory Stats
- New entries this week: {n}
- Entries promoted to permanent: {n}
- Entries pruned: {n}
- Total memory size: {n} entries
```

Save this digest via `knowledge-base-add` with tags `weekly-digest, {date}`.

## Database Schema

Requires `memory_entries` and `memory_archive` tables from the Structured
Memory feature. Creates `memory_archive` if absent:

```sql
CREATE TABLE IF NOT EXISTS memory_archive (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key         TEXT,
    value       TEXT,
    importance  INTEGER,
    created_at  INTEGER,
    archived_at INTEGER
);
```

## Dependencies

- Structured Memory (SQLite backend) must be enabled.
- `knowledge-base-add` for storing the weekly digest.
- Local embeddings (for deduplication — falls back to key matching if absent).
