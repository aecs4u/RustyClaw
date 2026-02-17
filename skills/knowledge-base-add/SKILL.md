---
name: knowledge-base-add
description: "Save articles, PDFs, videos, and tweets to a searchable local knowledge base with automatic embeddings and tagging."
version: "0.1.0"
author: "aecs4u"
metadata:
  openclaw:
    emoji: "📚"
    requires:
      bins: []
      env: []
    primaryEnv: JINA_API_KEY
---

# Knowledge Base — Add Content

Saves a piece of content (article URL, PDF path, YouTube video, tweet/post) to
a local SQLite knowledge base with automatic text extraction, embedding, and
topic tagging.

## What This Skill Does

- Accepts: URLs (articles, YouTube), local PDF/Markdown paths, raw text snippets.
- Extracts clean text (strips ads/nav for articles, transcribes video if
  `yt-dlp` is available, parses PDF with `pdftotext` or Jina API).
- Generates an embedding vector using local embeddings (or Jina's Reader API).
- Stores content in `kb.db` with full text, embedding, source URL, and tags.
- Auto-tags using a small summarisation pass (title, 3-5 keyword tags).

## Setup

No required environment variables — works offline with local embeddings.

Set `JINA_API_KEY` for higher-quality PDF extraction and remote embeddings:

```
rustyclaw vault set jina_api_key
rustyclaw skills link-secret knowledge-base-add jina_api_key
```

Optional binaries for richer extraction (install on host):
- `pdftotext` (poppler-utils) — PDF text extraction
- `yt-dlp` — YouTube transcript extraction

## Instructions for Agent

When the user says "save this", "add to KB", "remember this article", or
similar:

1. Identify the content type from the input:
   - URL matching `youtube.com` or `youtu.be` → video
   - URL ending `.pdf` or local path ending `.pdf` → PDF
   - Any other URL → article
   - Raw text / markdown → snippet
2. Extract text:
   - Article: use `fetch_url` then strip HTML to plain text.
   - PDF (local): run `pdftotext {path} -` or call Jina Reader API if key set.
   - YouTube: attempt `yt-dlp --skip-download --write-auto-sub --sub-format vtt`
     then parse VTT captions; fall back to description if unavailable.
   - Snippet: use text as-is.
3. Generate a 1–2 sentence summary and 3–5 keyword tags using the agent's
   reasoning (no extra API call needed).
4. Generate embedding via `embed_text` with the summary + title.
5. Insert into `kb_items`:
   ```sql
   INSERT INTO kb_items (source, type, title, text, summary, tags, embedding, added_at)
   VALUES (?, ?, ?, ?, ?, ?, ?, unixepoch());
   ```
6. Confirm: "Saved: '{title}' [tags: {tags}]"

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS kb_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    source     TEXT,    -- URL or file path
    type       TEXT,    -- 'article' | 'pdf' | 'video' | 'snippet'
    title      TEXT,
    text       TEXT,    -- full extracted text
    summary    TEXT,    -- 1-2 sentence summary
    tags       TEXT,    -- comma-separated keyword tags
    embedding  BLOB,    -- float32 vector, local embeddings format
    added_at   INTEGER  -- unix timestamp
);
```

## Dependencies

- Local embeddings skill (recommended — falls back to keyword-only if absent)
- `knowledge-base-search` (companion search skill)
