---
name: daily-health-check
description: "Daily automated monitoring of the RustyClaw gateway, SQLite databases, external APIs, and system resources."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🩺"
    requires:
      bins:
        - curl
routines:
  - name: daily-health-check
    trigger: cron
    expression: "0 0 8 * * * *"
    description: "Run every day at 08:00"
---

# Daily Platform Health Check

Runs a comprehensive daily health check across all RustyClaw components:
gateway service, SQLite databases, external API reachability, and host
system resources.

## What This Skill Does

1. **Gateway check** — verifies the RustyClaw gateway is responding.
2. **Database integrity** — runs `PRAGMA integrity_check` on all `*.db` files.
3. **Database sizes** — warns if any DB exceeds a size threshold.
4. **API reachability** — pings configured external API endpoints (Gmail,
   Calendar, ClawHub, optional third-party APIs).
5. **System resources** — checks disk usage, memory, and load average.
6. **Cost tracking** — reports yesterday's API token spend if available.
7. Outputs a health report and flags any WARNING or CRITICAL items.

## Setup

### Schedule as a Routine

```
rustyclaw routines add daily-health-check \
  --trigger cron \
  --cron "0 0 8 * * * *" \
  --prompt "Run daily-health-check"
```

### Optional Config

```toml
[health_check]
db_size_warn_mb   = 500    # warn if DB exceeds this size
disk_warn_pct     = 85     # warn if disk usage exceeds this %
api_endpoints     = [      # additional URLs to ping
  "https://api.anthropic.com",
  "https://clawhub.ai",
]
```

## Instructions for Agent

When executing this skill, produce a structured health report:

### 1. Gateway Status

```bash
curl -sf http://localhost:{gateway_port}/health
```
Report OK / UNREACHABLE.

### 2. Database Integrity

For each `*.db` in the data directory:
```bash
sqlite3 {db} "PRAGMA integrity_check;" 2>&1
sqlite3 {db} "PRAGMA page_count;" 2>&1
```
Report OK / CORRUPT. Warn if size > configured threshold.

### 3. API Reachability

For each configured endpoint, run:
```bash
curl -sf --max-time 5 {url} -o /dev/null -w "%{http_code}"
```
Report HTTP status. Flag non-2xx as WARNING, connection refused as CRITICAL.

### 4. System Resources

```bash
df -h /           # disk usage
free -m           # memory
uptime            # load average
```
Warn if disk > 85%, flag memory < 256 MB free as WARNING.

### 5. Cost Report (if available)

Query today's rows from the cost tracking table:
```sql
SELECT provider, SUM(tokens_in) as in, SUM(tokens_out) as out, SUM(cost_usd) as cost
FROM api_usage
WHERE date = date('now', '-1 day')
GROUP BY provider;
```

### 6. Report Format

```
=== RustyClaw Health Report — {date} ===

GATEWAY:    ✅ OK
DATABASES:  ✅ crm.db (12 MB) | ✅ kb.db (48 MB) | ✅ memory.db (4 MB)
APIS:       ✅ anthropic.com | ✅ clawhub.ai | ⚠️  gmail (503)
DISK:       ✅ 42% used (/dev/sda1)
MEMORY:     ✅ 1.2 GB free
LOAD:       ✅ 0.45 (1m avg)
COST:       anthropic: $0.12 yesterday

Overall: ✅ HEALTHY  [or ⚠️ WARNINGS DETECTED / 🔴 CRITICAL]
```

Emit a WARNING log entry for each non-OK item. Emit a CRITICAL alert if
the gateway is unreachable or any database is corrupt.

## Dependencies

- `curl` on PATH.
- `sqlite3` CLI for integrity checks.
- Cost tracking table (optional — skip section if absent).
