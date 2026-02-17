---
name: twitter-search
description: "Cost-optimised Twitter/X search with a multi-tier fallback chain: official API → Apify scraper → Nitter → web search."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🐦"
    primaryEnv: TWITTER_BEARER_TOKEN
    requires:
      env: []
---

# Twitter/X Multi-Tier Search

Searches Twitter/X using a waterfall of providers ordered by cost and
reliability, falling through to cheaper tiers when the preferred API is
unavailable or rate-limited.

## Tier Chain

| Tier | Provider | Cost | Rate Limit |
|------|----------|------|------------|
| 1 | Twitter API v2 (Bearer) | ~$0.001/req | 500k tweets/mo (Basic) |
| 2 | Apify Twitter Scraper | ~$0.003/req | pay-per-use |
| 3 | Nitter public instance | free | fragile |
| 4 | Web search fallback | free | limited |

## Setup

Set at least one credential (all are optional — skill degrades gracefully):

```
rustyclaw vault set twitter_bearer_token   # Twitter API v2 Bearer Token
rustyclaw vault set apify_api_token        # Apify.com API token
rustyclaw skills link-secret twitter-search twitter_bearer_token
rustyclaw skills link-secret twitter-search apify_api_token
```

## Instructions for Agent

When asked to search Twitter/X:

### Tier 1 — Twitter API v2

If `TWITTER_BEARER_TOKEN` is set:
```
GET https://api.twitter.com/2/tweets/search/recent
  ?query={encoded_query}
  &max_results=10
  &tweet.fields=created_at,author_id,public_metrics,text
Authorization: Bearer {token}
```
On 429 (rate limit) or 403 (insufficient access), fall to Tier 2.

### Tier 2 — Apify Scraper

If `APIFY_API_TOKEN` is set and Tier 1 failed:
```
POST https://api.apify.com/v2/acts/quacker~twitter-scraper/run-sync-get-dataset-items
  ?token={APIFY_API_TOKEN}
Body: { "searchTerms": ["{query}"], "maxTweets": 10 }
```
On error or missing token, fall to Tier 3.

### Tier 3 — Nitter

Try a public Nitter instance:
```
GET https://nitter.net/search?q={encoded_query}&f=tweets
```
Parse `<div class="tweet-content">` entries.
On failure (instance down), fall to Tier 4.

### Tier 4 — Web Search Fallback

Use `web_search` tool with query `site:twitter.com OR site:x.com {query}`.
Extract tweet URLs and text snippets from results.

### Result Format

For each result, normalise to:
```json
{
  "text": "tweet text",
  "author": "@handle",
  "date": "2025-02-10",
  "url": "https://x.com/handle/status/123",
  "likes": 42,
  "retweets": 7,
  "tier_used": "twitter_api_v2"
}
```

Present results as a numbered list with author, date, and full text.
Always note which tier was used and why lower tiers were skipped.

## Cost Tracking

Log each search attempt with tier used, result count, and estimated cost
to the API cost tracking table if available.

## Dependencies

- At least one of: `TWITTER_BEARER_TOKEN`, `APIFY_API_TOKEN`, or internet access.
- `video-idea-pipeline` uses this skill for trend research.
