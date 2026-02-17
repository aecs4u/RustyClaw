---
name: hourly-db-backup
description: "Automated hourly backup of all RustyClaw SQLite databases to Google Drive using rclone."
version: "0.1.0"
author: "aecs4u"
linked_secrets:
  - rclone_config
metadata:
  openclaw:
    emoji: "🗄️"
    primaryEnv: RCLONE_REMOTE
    requires:
      bins:
        - rclone
      env:
        - RCLONE_REMOTE
routines:
  - name: hourly-db-backup
    trigger: cron
    expression: "0 30 * * * * *"
    description: "Run every hour at HH:30 (offset from code backup)"
---

# Hourly Database Backup

Backs up all SQLite databases used by RustyClaw (CRM, knowledge base, structured
memory, routines) to Google Drive via `rclone` every hour.

## What This Skill Does

1. Discovers all `*.db` files under the workspace data directory.
2. Creates timestamped copies in a temp directory.
3. Uploads the copies to `{RCLONE_REMOTE}:rustyclaw-backups/{date}/`.
4. Cleans up temp copies after upload.
5. Retains the last 30 days of backups on Drive (prunes older folders).

## Setup

### Install rclone

```bash
curl https://rclone.org/install.sh | sudo bash
rclone config   # follow wizard to configure "gdrive" remote
```

### Set Environment Variables

```toml
# config.toml
[backup]
rclone_remote = "gdrive"          # name of your rclone remote
db_backup_path = "rustyclaw-backups"  # folder name on Drive
```

Or set via environment:

```bash
export RCLONE_REMOTE=gdrive
```

### Schedule as a Routine

```
rustyclaw routines add hourly-db-backup \
  --trigger cron \
  --cron "0 30 * * * * *" \
  --prompt "Run hourly-db-backup"
```

## Instructions for Agent

When executing this skill:

1. Discover databases:
   ```bash
   find {data_dir} -name "*.db" -type f
   ```
   Typical files: `crm.db`, `kb.db`, `memory.db`, `routines.db`.
2. For each `*.db` file, create a safe copy using SQLite's online backup API:
   ```bash
   sqlite3 {db_path} ".backup /tmp/backup/{name}-{timestamp}.db"
   ```
3. Upload all copies:
   ```bash
   rclone copy /tmp/backup/ {RCLONE_REMOTE}:rustyclaw-backups/{YYYY-MM-DD}/
   ```
4. Clean up `/tmp/backup/`:
   ```bash
   rm -rf /tmp/backup/
   ```
5. Prune old backups (retain last 30 days):
   ```bash
   rclone delete --min-age 30d {RCLONE_REMOTE}:rustyclaw-backups/
   ```
6. Report:
   ```
   Backed up {n} databases ({total_size}) to {remote}:rustyclaw-backups/{date}/
   ```

## Error Handling

- If rclone is not installed: log a clear error, do not fail silently.
- If a database is locked (WAL mode write in progress): use SQLite's `.backup`
  command which handles concurrent access safely.
- If upload fails: keep local copies in a `failed_backups/` directory and
  retry on the next run.

## Dependencies

- `rclone` on PATH with a configured Google Drive remote.
- `sqlite3` CLI for safe hot backups (optional — file copy works if DBs are idle).
