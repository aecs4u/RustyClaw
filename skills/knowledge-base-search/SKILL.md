---
name: knowledge-base-search
description: "Semantic and keyword search across articles, PDFs, videos, and snippets stored in the local knowledge base."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "🔎"
    requires:
      bins: []
      env: []
---

# Knowledge Base — Search

Searches the local knowledge base built by `knowledge-base-add` using a
combination of keyword matching and semantic (embedding) similarity.

## What This Skill Does

- Accepts a free-form query in natural language.
- Runs keyword search (FTS5 MATCH or LIKE) for fast exact matches.
- Runs embedding similarity search for conceptual matches.
- Merges and re-ranks results by a combined relevance score.
- Returns title, source URL, summary, tags, and a relevant excerpt per result.

## Instructions for Agent

When the user asks to "search my notes", "find articles about X",
"what do I know about Y", or similar:

1. Generate an embedding for the query using `embed_text`.
2. Run keyword search:
   ```sql
   SELECT id, source, type, title, summary, tags
   FROM kb_items
   WHERE title LIKE '%{query}%' OR tags LIKE '%{query}%' OR text LIKE '%{query}%'
   ORDER BY added_at DESC
   LIMIT 20;
   ```
3. Run semantic search: compute cosine similarity between the query embedding
   and each stored embedding; select top-10 by similarity > 0.65 threshold.
4. Merge both result sets, deduplicate by `id`, and compute combined score:
   `score = 0.4 * keyword_rank + 0.6 * semantic_similarity`
5. Return the top 5 results formatted as:
   ```
   1. **{title}** ({type}) — {date}
      {summary}
      Source: {source}
      Tags: {tags}
   ```
6. If fewer than 3 results, suggest running `knowledge-base-add` on relevant
   content.

## Example Queries

- "embeddings in Rust" → finds articles and PDFs on the topic
- "that YouTube video about vector databases" → semantic match on video transcript
- "my notes on customer onboarding" → searches snippets and PDFs
- "anything from last week about competitors" → combines recency filter + semantic

## Dependencies

- `knowledge-base-add` must have run to populate the database.
- Local embeddings skill (recommended — falls back to keyword-only if absent).
