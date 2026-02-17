---
name: daily-markdown-validation
description: "Automated daily validation of all skill SKILL.md files and config TOML files against schema and best-practice rules."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "✅"
    requires:
      bins: []
      env: []
routines:
  - name: daily-markdown-validation
    trigger: cron
    expression: "0 0 3 * * * *"
    description: "Run every day at 03:00 (low-traffic window)"
---

# Daily Markdown Validation

Validates all `SKILL.md` files and `config.toml` against schema rules and
best practices. Reports violations, warnings, and auto-fixable issues.

## What This Skill Does

1. Discovers all `SKILL.md` files under configured skills directories.
2. Validates frontmatter against the SKILL.md schema.
3. Validates `config.toml` against known keys and value types.
4. Checks instructions body for common issues (broken links, empty sections).
5. Produces a validation report with ERROR / WARNING / INFO items.
6. Optionally auto-fixes trivially correctable issues (trailing whitespace,
   missing newline at EOF, wrong version format).

## Setup

### Schedule as a Routine

```
rustyclaw routines add daily-markdown-validation \
  --trigger cron \
  --cron "0 0 3 * * * *" \
  --prompt "Run daily-markdown-validation"
```

## Validation Rules

### SKILL.md Frontmatter Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `name` present | ERROR | Skill must have a name field |
| `name` slug format | ERROR | Only alphanumeric, hyphens, underscores |
| `description` present | WARNING | Description strongly recommended |
| `version` semver | WARNING | Should follow `"X.Y.Z"` format |
| `author` present | INFO | Author field recommended for sharing |
| `metadata.openclaw` valid | WARNING | Known keys only; no typos |
| `requires.bins` resolvable | WARNING | Binaries listed should exist on PATH |
| `requires.env` documented | INFO | Env vars should be described in body |

### SKILL.md Body Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `# {name}` heading | WARNING | Body should start with H1 matching name |
| `## Instructions` present | WARNING | Skill should have Instructions section |
| No broken markdown links | INFO | `[text](url)` — URL should be reachable |
| No empty sections | INFO | Sections with no content are noise |

### config.toml Rules

| Rule | Severity | Description |
|------|----------|-------------|
| Known top-level keys | WARNING | Unexpected keys may be typos |
| Model names valid | WARNING | Provider model IDs should be recognised |
| Port values in range | ERROR | Ports must be 1–65535 |
| Token values non-empty strings | WARNING | Empty token strings usually a mistake |

## Instructions for Agent

### Discovery

Collect all SKILL.md paths:
```bash
find {skills_dirs} -name "SKILL.md" -type f
```
Also collect `config.toml` from the workspace root.

### Validation Pass

For each SKILL.md:
1. Parse frontmatter with the YAML parser.
2. Check each rule above; collect violations as `{path}: [{severity}] {message}`.
3. Parse the markdown body; check H1, sections, links.

For `config.toml`:
1. Parse with a TOML parser.
2. Check known keys, value types, and port ranges.

### Auto-Fix Pass (if `--fix` flag is set)

Apply these safe transformations:
- Strip trailing whitespace from all lines.
- Ensure file ends with exactly one newline.
- Normalise version strings to `"X.Y.Z"` quoted format.
- Add missing `description` with placeholder text and add INFO note.

### Report

```
=== Skill Validation Report — {date} ===

Checked {n} SKILL.md files + config.toml

ERRORS (must fix):
  crm-daily-sync/SKILL.md: [ERROR] 'name' field missing from frontmatter

WARNINGS (should fix):
  twitter-search/SKILL.md: [WARNING] 'description' field not present
  config.toml: [WARNING] Unknown key 'llm.unkown_setting' (typo?)

INFO:
  business-council/SKILL.md: [INFO] 'author' field empty

Summary: {e} errors, {w} warnings, {i} info
```

Exit with status 1 if any ERROR items are present.
Store the report in the knowledge base with tags `validation, {date}`.

## Dependencies

- Access to the skills directory (read-only is sufficient).
- `config.toml` in the workspace root.
- No external APIs required.
