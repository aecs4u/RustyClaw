---
name: business-council
description: "Daily multi-agent business council: spawns specialist sub-agents for strategy, market trends, and operations to produce a consolidated business insights brief."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🏛️"
    primaryEnv: ANTHROPIC_API_KEY
    requires:
      env:
        - ANTHROPIC_API_KEY
routines:
  - name: business-council
    trigger: cron
    expression: "0 0 9 * * MON-FRI *"
    description: "Run weekdays at 09:00"
---

# Business Council (Multi-Agent)

Orchestrates a panel of specialist AI sub-agents — each with a defined role
and scoped context — that collaborate to produce a consolidated daily business
insights brief.

> **Cost note:** Estimated ~$1.50–$2.00 per run using Claude Sonnet. Budget
> approximately $40–48/month if run daily on weekdays.

## Council Roles

| Agent | Role | Context |
|-------|------|---------|
| Strategist | Long-term direction, priorities | CRM trends + KB recent articles |
| Analyst | Market trends, competitor movements | Twitter search + web headlines |
| Ops Director | Execution status, blockers | Recent routines log + health check |
| Synthesiser | Consolidates into actionable brief | All three agent outputs |

## Setup

Requires an Anthropic API key with sufficient quota for 4 sequential API calls:

```
rustyclaw vault set anthropic_api_key
rustyclaw skills link-secret business-council anthropic_api_key
```

### Schedule as a Routine

```
rustyclaw routines add business-council \
  --trigger cron \
  --cron "0 0 9 * * MON-FRI *" \
  --prompt "Run business-council"
```

### Optional Config

```toml
[business_council]
model       = "claude-sonnet-4-5-20250929"  # model for sub-agents
max_tokens  = 1024                           # per sub-agent response
subjects    = ["SaaS growth", "AI tooling", "developer tools"]
```

## Instructions for Agent

### Phase 1 — Gather Context

Before spawning sub-agents, assemble context packages:

1. **CRM context:** Query `interactions` for the last 7 days, summarise key
   contacts and topics discussed.
2. **Market context:** Run `twitter-search` for each configured subject;
   fetch top 5 recent results per subject.
3. **KB context:** Run `knowledge-base-search` for each subject; return top 3
   items added in the last 7 days.
4. **Ops context:** Read the last `daily-health-check` report and the last
   7 days of `routine_executions` from `routines.db`.

### Phase 2 — Sub-Agent Calls

Call the Anthropic API three times in sequence (or parallel if safe):

**Strategist prompt:**
```
You are the Chief Strategy Officer. Based on the CRM and KB context provided,
identify the top 3 strategic priorities for this week. Be specific, concise,
and actionable. Context: {crm_context} {kb_context}
```

**Analyst prompt:**
```
You are the Market Analyst. Based on the trending topics and competitor signals
provided, summarise the top 3 market developments relevant to {subjects}.
Highlight any threats or opportunities. Context: {market_context}
```

**Ops Director prompt:**
```
You are the Operations Director. Based on the system health and routine
execution logs, identify any blockers, failures, or improvements needed.
Suggest concrete next steps. Context: {ops_context}
```

### Phase 3 — Synthesise

Call the Synthesiser with all three outputs:
```
You are the Executive Assistant. Synthesise the following reports from the
Strategy, Market, and Operations agents into a single daily brief (max 400 words).
Format as: Executive Summary, Key Actions (numbered), Flags/Risks.
{strategist_output} {analyst_output} {ops_output}
```

### Phase 4 — Deliver

Output the synthesised brief to stdout as markdown.
Save to `knowledge-base-add` with type `snippet`, tags `business-council, {date}`.

## Output Format

```markdown
# Business Council Brief — {date}

## Executive Summary
{2-3 sentence overview}

## Key Actions
1. {action 1}
2. {action 2}
3. {action 3}

## Market Signals
{bullet points from Analyst}

## Operational Flags
{bullet points from Ops Director}

## Risks
{any flagged risks}
```

## Dependencies

- `ANTHROPIC_API_KEY` (required — no fallback)
- `twitter-search`, `knowledge-base-search`, `crm-daily-sync` (for context)
- `daily-health-check` (for ops context)
