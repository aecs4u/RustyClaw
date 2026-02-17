---
name: video-idea-pipeline
description: "Automated content research pipeline: discovers trending topics, validates ideas against the knowledge base, and creates structured video production tasks."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🎬"
    requires:
      bins: []
      env: []
---

# Video Idea Pipeline

Runs an automated content research pipeline to surface high-potential video
ideas, validate them against existing knowledge, and produce ready-to-act
production tasks.

## What This Skill Does

1. **Trend discovery** — searches Twitter/X and web for trending topics in
   configured subject areas.
2. **Knowledge gap analysis** — checks the knowledge base for existing coverage
   to avoid duplicate content.
3. **Competitive research** — searches YouTube (via web) for existing videos on
   the topic; estimates gap opportunity.
4. **Idea scoring** — scores each candidate on: trending score, knowledge gap,
   competition density, estimated production effort.
5. **Task generation** — creates structured production tasks for the top 3 ideas.

## Setup

Configure subject areas and optional YouTube API:

```toml
[video_pipeline]
subjects = ["rust programming", "AI tools", "developer productivity"]
min_score = 0.6          # minimum idea score (0–1)
max_ideas_per_run = 5    # cap output
```

Optional (improves competition research):
```
rustyclaw vault set youtube_api_key
rustyclaw skills link-secret video-idea-pipeline youtube_api_key
```

## Instructions for Agent

### Step 1 — Gather Trending Topics

For each configured subject, invoke `twitter-search` with:
- `"{subject} tutorial"` — find what people are asking about
- `"{subject} tip"` — find engagement-driving formats
- filter to tweets with > 50 likes in the last 7 days

Also run `web_search` for `"{subject}" site:reddit.com "how to"` to surface
questions on r/rust, r/MachineLearning, etc.

### Step 2 — KB Deduplication

For each candidate topic, call `knowledge-base-search` with the topic title.
If a very similar item (similarity > 0.85) exists and was added in the last
30 days, skip this topic (already researched).

### Step 3 — Competition Research

For each remaining candidate:
```
web_search: 'site:youtube.com "{topic}" tutorial'
```
Count results. Score competition:
- 0–5 results → low competition (score: 1.0)
- 6–20 results → medium (score: 0.6)
- 20+ results → high (score: 0.3)

### Step 4 — Score and Rank

```
final_score = (twitter_engagement * 0.4) +
              (kb_novelty * 0.3) +
              (competition_score * 0.3)
```

Filter to `final_score >= min_score`, take top N.

### Step 5 — Generate Tasks

For each top idea, output a structured task:

```markdown
## Video Idea: {title}

**Score:** {score:.2}
**Trending:** {top_tweet_example}
**Competition:** {n} existing videos

### Production Checklist
- [ ] Research & outline (est. 2h)
- [ ] Record (est. 3h)
- [ ] Edit (est. 4h)
- [ ] Thumbnail + description (est. 1h)

### Key Points to Cover
{3-5 bullet points from trend research}

### Reference Sources
{top 3 URLs from knowledge base or web search}
```

Save each task to `knowledge-base-add` with type `snippet` and tags
`video-idea, {subject}, {date}`.

## Dependencies

- `twitter-search` skill (for trend data)
- `knowledge-base-search` + `knowledge-base-add` (for deduplication and storage)
- YouTube Data API v3 (optional — web search fallback used otherwise)
