---
name: hourly-code-backup
description: "Automated hourly Git backup of the RustyClaw workspace, skills, and configuration files to a remote repository."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "💾"
    requires:
      bins:
        - git
routines:
  - name: hourly-code-backup
    trigger: cron
    expression: "0 0 * * * * *"
    description: "Run every hour on the hour"
---

# Hourly Code Backup

Performs an automated Git commit and push of all changed files in the
RustyClaw workspace — including skills, configuration, and custom routines.

## What This Skill Does

1. Checks `git status` in the workspace directory for uncommitted changes.
2. If changes exist, stages all modified files (`git add -A`).
3. Commits with an automatic timestamped message.
4. Pushes to the configured `backup` remote (defaults to `origin`).
5. Reports the number of files backed up or "nothing to commit".

## Setup

### Configure a Backup Remote

```bash
# If using origin as backup (default):
git remote -v   # verify origin is set

# Or add a dedicated backup remote:
git remote add backup git@github.com:youruser/rustyclaw-backup.git
```

Set the remote name in config if different from `origin`:

```toml
[backup]
git_remote = "backup"   # default: "origin"
git_branch = "main"     # default: "main"
```

### Schedule as a Routine

```
rustyclaw routines add hourly-code-backup \
  --trigger cron \
  --cron "0 0 * * * * *" \
  --prompt "Run hourly-code-backup"
```

## Instructions for Agent

When executing this skill:

1. Run `git -C {workspace_dir} status --porcelain` to detect changes.
2. If output is empty, output "Nothing to commit." and exit.
3. Run `git -C {workspace_dir} add -A`.
4. Run `git -C {workspace_dir} commit -m "Auto-backup {iso_timestamp}"`.
5. Run `git -C {workspace_dir} push {remote} {branch}`.
6. Report:
   ```
   Backed up {n} files to {remote}/{branch} at {timestamp}.
   ```
7. If `git push` fails (network, auth), log the error and do NOT retry —
   the next hourly run will pick up the same changes.

## Error Handling

- On push failure: log the error message, keep the local commit, exit cleanly.
- On merge conflict: log a warning, skip this run — do not force-push.
- On missing remote: log a warning and suggest running setup.

## Dependencies

- `git` must be on PATH.
- SSH key or HTTPS credentials configured for the remote.
