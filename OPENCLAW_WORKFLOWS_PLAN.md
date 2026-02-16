# OpenClaw-Inspired Workflows Implementation Plan

## Executive Summary

This plan implements 10 major workflows inspired by OpenClaw's advanced usage patterns:
1. **Hybrid Database Foundation** - Standardized SQLite + vector storage
2. **Personal CRM** - Contact management with Gmail/Calendar integration
3. **Knowledge Base** - Article/document storage with semantic search
4. **Cost Tracking** - API usage and cost monitoring
5. **Video Idea Pipeline** - Content research and task creation
6. **Twitter/X Search** - Multi-tier fallback chain
7. **Meeting Prep** - Automated daily briefings
8. **Business Council** - Multi-agent meta-analysis
9. **Backup & Health** - Automated system maintenance
10. **Self-Improvement** - Memory synthesis and validation

**Total estimated effort:** 18-22 weeks (~350-450 hours)
**Dependencies:** Gmail feature (PR #34), Telegram/Slack messengers, multi-agent orchestration

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    RustyClaw Gateway                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ WebSocket    │  │ Gmail        │  │ Telegram     │      │
│  │ Sessions     │  │ Webhook      │  │ Bot          │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                ┌───────────▼────────────┐                    │
│                │   Skill Orchestrator   │                    │
│                └───────────┬────────────┘                    │
│                            │                                 │
│         ┌──────────────────┼──────────────────┐             │
│         │                  │                  │             │
│    ┌────▼────┐      ┌──────▼──────┐    ┌─────▼─────┐       │
│    │ Skills  │      │ Hybrid DB   │    │ External  │       │
│    │ Engine  │◄─────┤ (SQLite +   │───►│ APIs      │       │
│    └─────────┘      │  Vector)    │    │ (Gmail,   │       │
│                     └─────────────┘    │  Twitter) │       │
│                                        └───────────┘       │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation Infrastructure (Weeks 1-3)

### 1.1 Hybrid Database System

**Goal:** Create reusable database abstraction with SQL + vector search.

**Components:**

#### `src/database/mod.rs` (NEW)
```rust
pub mod hybrid;
pub mod sqlite;
pub mod vector;

pub use hybrid::HybridDatabase;
```

#### `src/database/hybrid.rs` (NEW)
Core abstraction layer:
```rust
#[async_trait]
pub trait HybridDatabase: Send + Sync {
    /// Insert record with auto-embedding generation
    async fn insert(&self, table: &str, record: &Record) -> Result<i64>;

    /// Traditional SQL query
    async fn query_sql(&self, query: &str, params: &[Value]) -> Result<Vec<Record>>;

    /// Semantic search using vector similarity
    async fn search_semantic(&self, query: &str, limit: usize) -> Result<Vec<(Record, f32)>>;

    /// Hybrid: SQL filters + semantic ranking
    async fn search_hybrid(
        &self,
        sql_where: &str,
        semantic_query: &str,
        limit: usize,
    ) -> Result<Vec<(Record, f32)>>;

    /// Batch operations for efficiency
    async fn insert_batch(&self, table: &str, records: &[Record]) -> Result<Vec<i64>>;
}

pub struct Record {
    pub id: Option<i64>,
    pub fields: HashMap<String, Value>,
    pub embedding: Option<Vec<f32>>,  // Optional vector
    pub embedding_text: Option<String>,  // Text to embed
}
```

#### `src/database/sqlite.rs` (NEW)
SQLite implementation with rusqlite:
```rust
pub struct SqliteBackend {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteBackend {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn create_table(&self, schema: &TableSchema) -> Result<()>;
}
```

#### `src/database/vector.rs` (NEW)
Vector storage options:
```rust
pub enum VectorBackend {
    SQLiteVSS,      // SQLite extension (simple, local)
    Qdrant,         // Dedicated vector DB (scalable)
    PostgresPgvector, // If user wants PostgreSQL
}

pub struct VectorStore {
    backend: VectorBackend,
}

impl VectorStore {
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    pub async fn search_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<(i64, f32)>>;
}
```

**Dependencies to add:**
```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
sqlite-vss = "0.1"  # Vector search extension
tokio-rusqlite = "0.5"  # Async wrapper
serde_rusqlite = "0.35"

# Optional: For dedicated vector DB
qdrant-client = { version = "1.0", optional = true }

[features]
qdrant = ["dep:qdrant-client"]
```

**Configuration:**
```toml
# config.toml
[database]
backend = "sqlite"  # or "qdrant"
path = "~/.rustyclaw/databases/"

[database.vector]
backend = "sqlite_vss"  # or "qdrant"
embedding_model = "voyage-2"  # or "text-embedding-3-small"
embedding_provider = "voyage"  # or "openai"
dimension = 1536  # Depends on model
```

**Tests to write:**
- `tests/database_hybrid.rs` - Integration tests for hybrid queries
- Test SQL-only queries
- Test vector-only queries
- Test hybrid queries (SQL filter + semantic ranking)
- Test batch inserts
- Test concurrent access

**Estimated effort:** 40 hours (1 week)
**Risk:** Medium - Vector search integration complexity

---

### 1.2 Complete Gmail Feature (Phases 2-4 from PR #34)

**Goal:** OAuth authentication, API client, webhook server.

**Status:** Foundation complete (PR #34), need implementation.

#### Phase 2: OAuth Authentication (~8 hours)

**File:** `src/gmail/auth.rs`

Implement device flow:
```rust
pub async fn create_authenticator(
    credentials_path: &Path,
) -> Result<Authenticator<HttpsConnector<HttpConnector>>> {
    let secret = yup_oauth2::read_application_secret(credentials_path).await?;

    let auth = DeviceFlowAuthenticator::builder(secret)
        .persist_tokens_to_disk(token_cache_path())
        .build()
        .await?;

    // Display device code to user
    eprintln!("Please visit: {} and enter code: {}", auth.verification_url, auth.user_code);

    Ok(auth)
}
```

#### Phase 3: Gmail API Client (~12 hours)

**File:** `src/gmail/client.rs`

Full implementation:
```rust
pub struct GmailClient {
    auth: Authenticator<HttpsConnector<HttpConnector>>,
    http: HyperClient<HttpsConnector<HttpConnector>>,
}

impl GmailClient {
    pub async fn list_emails(&self, query: &str, max_results: u32) -> Result<Vec<Email>> {
        let token = self.auth.token(&GMAIL_SCOPES).await?;
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages?q={}&maxResults={}",
            urlencoding::encode(query),
            max_results
        );

        let req = hyper::Request::builder()
            .uri(&url)
            .header("Authorization", format!("Bearer {}", token.as_str()))
            .body(Body::empty())?;

        let resp = self.http.request(req).await?;
        let body = hyper::body::to_bytes(resp.into_body()).await?;
        let messages: MessagesListResponse = serde_json::from_slice(&body)?;

        // Fetch full details for each message
        let mut emails = Vec::new();
        for msg in messages.messages {
            emails.push(self.read_email(&msg.id).await?);
        }

        Ok(emails)
    }

    pub async fn read_email(&self, message_id: &str) -> Result<Email> {
        // GET /users/me/messages/{id}
        // Parse headers, body
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str, thread_id: Option<&str>) -> Result<String> {
        // Construct RFC 2822 message
        // Base64url encode
        // POST /users/me/messages/send
    }
}
```

#### Phase 4: Webhook Server (~10 hours)

**File:** `src/gmail/webhook.rs`

Implement Pub/Sub handler:
```rust
pub async fn start_webhook_server(config: &GmailConfig) -> Result<()> {
    let app = Router::new()
        .route(&config.webhook_path, post(handle_pubsub_notification));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.webhook_port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handle_pubsub_notification(
    Json(payload): Json<PubSubMessage>,
) -> Result<String, StatusCode> {
    // Verify signature
    // Decode base64 data
    // Extract historyId
    // Fetch new messages via Gmail API
    // Route to appropriate session based on filters

    Ok("OK".to_string())
}
```

**Estimated effort:** 30 hours (1 week)
**Risk:** Low - Well-documented Gmail API

---

### 1.3 Cost Tracking Infrastructure

**Goal:** Log all API usage and costs for monitoring.

**Components:**

#### `src/usage/mod.rs` (NEW)
```rust
pub struct UsageTracker {
    db: SqliteBackend,
}

pub struct UsageRecord {
    pub timestamp: i64,
    pub workflow: String,
    pub service: String,  // 'anthropic', 'openai', 'gmail_api'
    pub model: Option<String>,  // 'claude-opus-4-6', etc.
    pub tokens_input: Option<i64>,
    pub tokens_output: Option<i64>,
    pub cost_usd: f64,
    pub metadata: serde_json::Value,
}

impl UsageTracker {
    pub async fn log(&self, record: UsageRecord) -> Result<()>;
    pub async fn query(&self, filter: UsageQuery) -> Result<Vec<UsageRecord>>;
    pub async fn summary(&self, period: TimePeriod) -> Result<UsageSummary>;
}
```

#### Database Schema
```sql
CREATE TABLE usage_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    workflow TEXT NOT NULL,
    service TEXT NOT NULL,
    model TEXT,
    tokens_input INTEGER,
    tokens_output INTEGER,
    cost_usd REAL NOT NULL,
    metadata TEXT,  -- JSON
    INDEX idx_timestamp (timestamp),
    INDEX idx_workflow (workflow),
    INDEX idx_service (service)
);

CREATE TABLE cost_budgets (
    id INTEGER PRIMARY KEY,
    service TEXT UNIQUE,
    monthly_limit_usd REAL NOT NULL,
    alert_threshold REAL DEFAULT 0.8  -- Alert at 80%
);
```

#### Wrapper Functions
Wrap all external API calls:
```rust
// src/api/anthropic.rs
pub async fn call_claude(prompt: &str, model: &str) -> Result<String> {
    let start = Instant::now();
    let response = anthropic_sdk::messages::create(...)
        .await?;

    // Log usage
    USAGE_TRACKER.log(UsageRecord {
        timestamp: Utc::now().timestamp(),
        workflow: current_workflow_context(),
        service: "anthropic".to_string(),
        model: Some(model.to_string()),
        tokens_input: Some(response.usage.input_tokens),
        tokens_output: Some(response.usage.output_tokens),
        cost_usd: calculate_anthropic_cost(model, &response.usage),
        metadata: json!({
            "duration_ms": start.elapsed().as_millis(),
        }),
    }).await?;

    Ok(response.content[0].text.clone())
}
```

**Configuration:**
```toml
[usage_tracking]
enabled = true
database_path = "~/.rustyclaw/databases/usage.db"

[[usage_tracking.budgets]]
service = "anthropic"
monthly_limit_usd = 100.0
alert_threshold = 0.8

[[usage_tracking.budgets]]
service = "gmail_api"
monthly_limit_usd = 10.0
alert_threshold = 0.9
```

**Skills to create:**
- `skills/cost_report.md` - Query usage by time period
- `skills/budget_alerts.md` - Check if approaching limits

**Estimated effort:** 20 hours (0.5 week)
**Risk:** Low - Straightforward logging

---

## Phase 2: Core Workflows (Weeks 4-7)

### 2.1 Personal CRM

**Goal:** Track contacts from Gmail/Calendar with semantic search.

#### Database Schema
```sql
CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    name TEXT,
    company TEXT,
    role TEXT,
    first_contact_date INTEGER,
    last_contact_date INTEGER,
    interaction_count INTEGER DEFAULT 0,
    interaction_timeline TEXT,  -- JSON array of interactions
    notes TEXT,
    embedding BLOB,  -- Vector embedding of all context
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE interactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_id INTEGER NOT NULL,
    interaction_type TEXT NOT NULL,  -- 'email_sent', 'email_received', 'meeting'
    subject TEXT,
    snippet TEXT,
    full_content TEXT,
    timestamp INTEGER NOT NULL,
    embedding BLOB,
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);

CREATE TABLE calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT UNIQUE,
    summary TEXT,
    description TEXT,
    start_time INTEGER,
    end_time INTEGER,
    participants TEXT,  -- JSON array of email addresses
    created_at INTEGER
);
```

#### Skills

**Skill:** `skills/crm_daily_sync.md`
```markdown
# CRM Daily Synchronization

## Trigger
- Cron: Daily at 6 AM
- Manual: User command `/crm sync`

## Steps

### 1. Gmail Ingestion
- Query Gmail API: `after:yesterday` (emails from last 24h)
- For each email:
  - Extract From, To, CC addresses
  - Parse subject and body
  - Create interaction record

### 2. Contact Extraction
- For each email address found:
  - Check if contact exists in `contacts` table
  - If new:
    - Extract name from email display name
    - Attempt company detection from email domain
    - Use Claude Haiku to classify role (if domain is known company)
    - Generate embedding from all available context
    - Insert into `contacts`
  - If existing:
    - Update `last_contact_date`
    - Increment `interaction_count`
    - Append to `interaction_timeline`
    - Regenerate embedding with new context

### 3. Calendar Integration
- Query Google Calendar API: events from last 24h
- For each meeting:
  - Extract participants
  - Update contact records with meeting interaction
  - Store event in `calendar_events`

### 4. Deduplication
- Merge duplicate contacts (same email, variations of name)
- Use semantic search to find near-duplicates (same company/role)

### 5. Classification (Batch)
- For new contacts without role/company:
  - Batch classify with Claude Haiku (cheap)
  - Prompt: "Based on email domain and name, what is likely role/company?"
  - Update records

### 6. Reporting
- Send Telegram notification:
  ```
  📇 CRM Daily Sync Complete
  - New contacts: 12
  - Updated contacts: 45
  - Meetings processed: 8
  - Total contacts: 1,247
  ```

## Error Handling
- If Gmail quota exceeded: Skip and retry in 1 hour
- If Calendar unavailable: Continue with email only
- Log all errors to `cron_logs` table

## Performance
- Target: Complete in <5 minutes
- Batch embedding generation (max 100 texts per API call)
- Use connection pooling for database
```

**Skill:** `skills/crm_search.md`
```markdown
# CRM Search

## Trigger
User queries like:
- "Who do I know at [Company]?"
- "When did I last talk to [Person]?"
- "Find contacts in [Industry]"
- "Show me people I haven't contacted in 30 days"

## Steps

### 1. Parse Query Intent
Use Claude to extract:
- Search type: person, company, industry, relationship
- Filters: time range, last contact, interaction count
- Semantic query text

### 2. Execute Hybrid Search
```rust
let results = crm_db.search_hybrid(
    sql_where: "last_contact_date < (now() - 30 days)",
    semantic_query: "software engineers at startups",
    limit: 10
).await?;
```

### 3. Format Results
For each contact:
```
👤 [Name] - [Role] at [Company]
   📧 [email]
   🕐 Last contact: [date] ([X] days ago)
   💬 Last interaction: "[subject/snippet]"
   📊 Total interactions: [count]
```

### 4. Offer Actions
- "View full timeline"
- "Draft follow-up email"
- "Schedule meeting"
```

**Skill:** `skills/meeting_prep.md`
```markdown
# Daily Meeting Prep

## Trigger
Cron: Daily at 7 AM

## Steps

### 1. Query Calendar
- Fetch today's calendar events
- Filter out:
  - All-day events
  - Events with no external participants
  - Internal team meetings (configurable email domains)

### 2. For Each Meeting
- Extract participant emails
- Look up contacts in CRM
- For each participant:
  - Get last interaction date and subject
  - Get company context
  - Get interaction timeline (last 5 interactions)

### 3. Generate Briefing
Use Claude Sonnet to create briefing:
```
Meeting: [Event summary]
Time: [Time]
Participants: [List]

Context for each participant:
- [Name] ([Role] at [Company])
  - Last contact: [X days ago] - "[Subject]"
  - Background: [Company description, past interactions]
  - Suggested topics: [AI-generated suggestions]
```

### 4. Deliver
- Send to Telegram "meeting_prep" topic
- One message per meeting
- Include action buttons:
  - "View full contact timeline"
  - "Draft agenda"
  - "Postpone meeting"
```

**Implementation files:**
- `src/workflows/crm/mod.rs` - Core CRM logic
- `src/workflows/crm/sync.rs` - Gmail/Calendar sync
- `src/workflows/crm/search.rs` - Hybrid search queries
- `src/workflows/crm/meeting_prep.rs` - Briefing generator
- `skills/crm_daily_sync.md`
- `skills/crm_search.md`
- `skills/meeting_prep.md`

**Tests:**
- Unit tests for contact deduplication
- Integration test: Gmail sync with mock API
- Test semantic search accuracy
- Test meeting prep generation

**Estimated effort:** 60 hours (1.5 weeks)
**Risk:** Medium - Requires Gmail + Calendar OAuth

---

### 2.2 Knowledge Base

**Goal:** Store articles/documents with semantic search.

#### Database Schema
```sql
CREATE TABLE knowledge_base (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT,
    title TEXT NOT NULL,
    author TEXT,
    source_type TEXT NOT NULL,  -- 'article', 'pdf', 'video', 'tweet'
    content_text TEXT NOT NULL,
    content_html TEXT,
    date_published INTEGER,
    date_added INTEGER NOT NULL,
    tags TEXT,  -- JSON array
    summary TEXT,
    embedding BLOB,
    metadata TEXT  -- JSON with source-specific fields
);

CREATE TABLE knowledge_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    knowledge_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding BLOB NOT NULL,
    FOREIGN KEY (knowledge_id) REFERENCES knowledge_base(id),
    UNIQUE(knowledge_id, chunk_index)
);
```

#### Skills

**Skill:** `skills/kb_add.md`
```markdown
# Add to Knowledge Base

## Trigger
- User sends URL in Telegram "knowledge_base" topic
- User sends file attachment (PDF, TXT, MD)
- Email with [KB] tag (via Gmail filter)

## Steps

### 1. Source Detection
Identify content type:
- URL pattern → article, YouTube, Twitter, GitHub, PDF
- File extension → PDF, TXT, MD, DOCX
- Email → forwarded article, newsletter

### 2. Content Extraction

**For URLs:**
```rust
match url_type {
    URLType::Article => {
        // Use readability-rs or goose
        let article = extract_article(&url).await?;
        (article.title, article.text, article.html)
    },
    URLType::YouTube => {
        // Use youtube-transcript-api
        let transcript = fetch_youtube_transcript(&video_id).await?;
        (video.title, transcript, None)
    },
    URLType::Twitter => {
        // Use FX Twitter API (free tier)
        let tweet = fetch_tweet(&tweet_id).await?;
        (format!("Tweet by {}", tweet.author), tweet.text, None)
    },
    URLType::PDF => {
        // Download and extract with pdf-extract
        let pdf_path = download(&url).await?;
        let text = extract_pdf_text(&pdf_path)?;
        (filename, text, None)
    },
}
```

**For Files:**
- PDF: `pdf-extract` crate
- DOCX: `docx-rs` crate
- Plain text: Read directly

### 3. Content Processing

**Chunking:**
```rust
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    // Split into chunks of ~500 tokens with 50 token overlap
    // Preserve sentence boundaries
    // Return chunks
}
```

**Embedding:**
- Generate embeddings for each chunk
- Use Voyage AI (`voyage-2`, 1536 dimensions, $0.10/1M tokens)
- Batch process chunks (up to 128 per request)

### 4. Storage
- Insert into `knowledge_base` table
- Insert chunks into `knowledge_chunks` table
- Generate overall summary with Claude Haiku

### 5. Response
Send to Telegram:
```
✅ Added to Knowledge Base
📄 [Title]
🔗 [URL]
📊 Processed 12 chunks
🏷️ Auto-tags: [AI, Anthropic, Claude 4.6]
💬 Summary: [One-sentence summary]
```

## Performance
- Target: <30 seconds for typical article
- Parallel: Download + extraction while generating embeddings
```

**Skill:** `skills/kb_search.md`
```markdown
# Search Knowledge Base

## Trigger
User queries like:
- "Find articles about Opus 4.6"
- "What did I save about agent security?"
- "Show me YouTube videos about LangChain"

## Steps

### 1. Parse Query
Extract:
- Search text (semantic)
- Filters: source_type, date range, tags
- Limit (default 5)

### 2. Generate Query Embedding
```rust
let query_embedding = vector_store.embed_text(query_text).await?;
```

### 3. Hybrid Search
```rust
let results = kb_db.search_hybrid(
    sql_where: "source_type = 'article' AND date_added > X",
    semantic_query: query_text,
    limit: 5
).await?;
```

### 4. Rank Results
- Combine SQL filter + vector similarity
- Re-rank with Claude if needed (query intent matching)

### 5. Format Response
For each result:
```
📄 [Title]
👤 [Author] | 🗓️ [Date]
🔗 [URL]
📝 [Summary]
⭐ Relevance: [similarity score]

[Most relevant chunk excerpt]
```

### 6. Follow-up Actions
- "Show full content"
- "Find similar articles"
- "Create video idea from this"
```

**Implementation files:**
- `src/workflows/knowledge_base/mod.rs`
- `src/workflows/knowledge_base/extractors.rs` - Content extraction
- `src/workflows/knowledge_base/chunking.rs` - Text chunking
- `src/workflows/knowledge_base/search.rs` - Hybrid search
- `skills/kb_add.md`
- `skills/kb_search.md`

**Dependencies to add:**
```toml
[dependencies]
url = "2.5"
reqwest = { version = "0.11", features = ["json", "stream"] }
readability = "0.3"  # Article extraction
pdf-extract = "0.7"
lopdf = "0.32"  # PDF parsing
html2text = "0.12"
```

**Tests:**
- Test extraction for various URL types
- Test chunking preserves context
- Test semantic search accuracy
- Integration test: End-to-end add + search

**Estimated effort:** 50 hours (1.25 weeks)
**Risk:** Medium - Content extraction reliability varies by source

---

### 2.3 Backup & Health Check Automation

**Goal:** Automated system maintenance and monitoring.

#### Skills

**Skill:** `skills/backup_code.md`
```markdown
# Hourly Code Backup

## Trigger
Cron: Every hour at :00

## Steps
1. Check Git status in `~/.rustyclaw/`
2. If changes detected:
   - `git add -A`
   - `git commit -m "Auto-backup $(date +%Y-%m-%d %H:%M)"`
   - `git push origin main`
3. Log result to `cron_logs` table
4. If failure: Send alert to Telegram

## Error Handling
- If push fails (network): Retry in 15 minutes
- If merge conflict: Alert user, skip auto-commit
```

**Skill:** `skills/backup_databases.md`
```markdown
# Hourly Database Backup

## Trigger
Cron: Every hour at :05

## Steps
1. Get all databases from `~/.rustyclaw/databases/`
2. For each database:
   - Create timestamped backup: `[dbname]_YYYY-MM-DD_HH.db`
   - Sync to Google Drive with rclone:
     ```bash
     rclone sync ~/.rustyclaw/databases/ \
                  gdrive:rustyclaw-backups/databases/ \
                  --exclude "*.db-shm" \
                  --exclude "*.db-wal"
     ```
3. Cleanup: Delete local backups older than 7 days
4. Log result to `cron_logs`

## Configuration
```toml
[backup]
enabled = true
remote = "gdrive:rustyclaw-backups"
retention_days = 7
```
```

**Skill:** `skills/health_check.md`
```markdown
# Daily Platform Health Check

## Trigger
Cron: Daily at 9 AM

## Checks

### 1. Gateway Status
- Check process is running: `pgrep rustyclaw-gateway`
- Check WebSocket responsive: Connect to ws://localhost:PORT
- Check memory usage: Alert if >1GB

### 2. Database Integrity
```rust
for db in databases {
    conn.execute("PRAGMA integrity_check", [])?;
    let size_mb = db.metadata()?.len() / 1_000_000;
    // Alert if size grew >100MB in 24h (potential leak)
}
```

### 3. API Quotas
```rust
// Check remaining credits
let anthropic_budget = check_budget("anthropic").await?;
let gmail_quota = check_gmail_quota().await?;

if anthropic_budget.remaining_pct < 0.2 {
    alert("⚠️ Anthropic budget at {}%", anthropic_budget.remaining_pct * 100);
}
```

### 4. Disk Space
```bash
df -h ~/.rustyclaw | awk 'NR==2 {print $5}' | sed 's/%//'
```
Alert if >90% full

### 5. Failed Cron Jobs
```sql
SELECT COUNT(*) FROM cron_logs
WHERE status = 'failed'
  AND timestamp > (now() - 86400);
```

### 6. Session Health
- Count active sessions
- Check for sessions >7 days old (potential leaks)
- Check WebSocket connection count

## Output
Send to Telegram "system_health" topic:
```
✅ Platform Health: GOOD
━━━━━━━━━━━━━━━━━━━
🟢 Gateway: Up 14 days
💾 Databases: 2.4 GB (OK)
💰 API Costs (24h): $3.42
🚫 Failed jobs: 0
🔗 Active sessions: 3
💽 Disk usage: 45% (OK)

📊 API Budget Status:
  Anthropic: $67/$100 (67%)
  Gmail: $2.40/$10 (24%)
```
```

**Implementation files:**
- `skills/backup_code.md`
- `skills/backup_databases.md`
- `skills/health_check.md`
- `src/cron/backup.rs` - Backup logic
- `src/cron/health.rs` - Health check logic

**Cron configuration:**
```toml
[[cron_jobs]]
name = "backup_code"
schedule = "0 * * * *"  # Every hour
skill = "backup_code"

[[cron_jobs]]
name = "backup_databases"
schedule = "5 * * * *"  # Every hour at :05
skill = "backup_databases"

[[cron_jobs]]
name = "health_check"
schedule = "0 9 * * *"  # Daily at 9 AM
skill = "health_check"
```

**Prerequisites:**
```bash
# Install rclone
sudo apt install rclone
rclone config  # Configure Google Drive remote named "gdrive"

# Initialize Git repo
cd ~/.rustyclaw
git init
git remote add origin https://github.com/yourusername/rustyclaw-instance.git
git add -A
git commit -m "Initial commit"
git push -u origin main
```

**Estimated effort:** 25 hours (0.6 weeks)
**Risk:** Low - Straightforward automation

---

## Phase 3: Advanced Workflows (Weeks 8-12)

### 3.1 Video Idea Pipeline

**Goal:** Research + task creation for content ideas.

#### Database Schema
```sql
CREATE TABLE video_ideas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    source_url TEXT,
    pitch TEXT NOT NULL,
    hooks TEXT NOT NULL,  -- JSON array of 5 hooks
    outline TEXT NOT NULL,  -- Markdown outline
    research_sources TEXT NOT NULL,  -- JSON array of URLs
    status TEXT NOT NULL DEFAULT 'pitched',  -- 'pitched', 'approved', 'in_production', 'published'
    task_url TEXT,  -- Asana/GitHub issue URL
    created_at INTEGER NOT NULL,
    published_video_id TEXT,
    embedding BLOB
);

CREATE TABLE video_research (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_idea_id INTEGER NOT NULL,
    source_type TEXT NOT NULL,  -- 'article', 'tweet', 'video', 'web'
    url TEXT NOT NULL,
    title TEXT,
    snippet TEXT,
    relevance_score REAL,
    FOREIGN KEY (video_idea_id) REFERENCES video_ideas(id)
);
```

#### Skills

**Skill:** `skills/video_idea_create.md`
```markdown
# Create Video Idea

## Trigger
- User says: "video idea: [topic]"
- User sends URL with: "make a video about this"
- Slack message tagged with @RustyClaw + "video"
- Knowledge base auto-triggers (configurable)

## Steps

### 1. Parse Input
Extract:
- Topic/intent
- Source URL (if provided)
- Context from conversation

### 2. Research Phase (Parallel Execution)

**Twitter/X Research:**
```rust
let tweets = twitter_api.search(
    query: format!("{} (min_faves:100 OR min_retweets:50)", topic),
    max_results: 20,
    recency_days: 30
).await?;

// Filter for high engagement
let top_tweets = tweets.into_iter()
    .filter(|t| t.likes + t.retweets > 200)
    .take(10)
    .collect();
```

**Web Research:**
```rust
let web_results = brave_search(
    query: format!("{} latest news 2026", topic),
    count: 10
).await?;

// Use Claude to score relevance
for result in web_results {
    let score = rate_relevance(&result, &topic).await?;
    if score > 0.7 {
        research_sources.push(result);
    }
}
```

**Knowledge Base Search:**
```rust
let kb_results = knowledge_base.search_semantic(
    query: topic,
    limit: 5
).await?;

// Include saved articles as context
```

### 3. Duplicate Check
```sql
SELECT * FROM video_ideas
WHERE embedding IS NOT NULL
ORDER BY vector_distance(embedding, ?) ASC
LIMIT 3;
```

If similar idea exists (similarity >0.85):
- Alert user: "Similar idea from [date]: [title]"
- Ask: Continue anyway? (A) Yes (B) No

### 4. Generate Video Pitch
Use Claude Opus 4.6:
```
Context:
- Topic: {topic}
- Research sources: {sources}
- Knowledge base articles: {kb_results}
- Twitter trends: {top_tweets}

Task:
Create a compelling video pitch with:
1. 3 angle variations (different approaches to the topic)
2. For the best angle:
   - 5 hook options (first 15 seconds)
   - 3-part outline (intro/body/conclusion)
   - Key points to cover
   - Call-to-action suggestion
3. All source links formatted as markdown
```

### 5. Create Task
**Asana Integration:**
```rust
let task = asana_client.create_task(CreateTaskRequest {
    workspace: config.asana_workspace_id,
    project: config.asana_video_pipeline_project_id,
    name: format!("Video: {}", pitch_title),
    notes: format!("
# Video Pitch

{}

# Hooks

{}

# Outline

{}

# Research Sources

{}
", pitch, hooks, outline, sources),
    due_on: Some(Utc::now() + Duration::days(14)),
}).await?;
```

**Alternative: GitHub Issues**
```rust
let issue = github_client.create_issue(
    owner: "yourusername",
    repo: "video-ideas",
    title: format!("[VIDEO] {}", pitch_title),
    body: /* same as above */,
    labels: vec!["video-idea", "researched"],
).await?;
```

### 6. Store & Notify

**Database:**
```rust
db.execute(
    "INSERT INTO video_ideas (topic, pitch, hooks, outline, research_sources, task_url, embedding) VALUES (?, ?, ?, ?, ?, ?, ?)",
    params![topic, pitch_json, hooks_json, outline, sources_json, task_url, embedding],
)?;
```

**Telegram:**
```
🎥 Video Idea Created!

📌 Topic: [Topic]
🎯 Pitch: [First 100 chars...]

🪝 Top Hook: "[Hook #1]"

📋 Task: [Asana/GitHub link]
⏱️ Research time: 28 seconds

[View Full Pitch] [Start Production] [Reject]
```

**Slack (Optional):**
Post summary to team channel with link to task

## Performance Target
- Total execution: <60 seconds
- Research phase: <30 seconds (parallel)
- Pitch generation: <20 seconds
```

**Implementation files:**
- `src/workflows/video_pipeline/mod.rs`
- `src/workflows/video_pipeline/research.rs` - Multi-source research
- `src/workflows/video_pipeline/pitch_generator.rs` - Claude-based pitch creation
- `src/integrations/asana.rs` - Asana API client
- `src/integrations/github_issues.rs` - GitHub issues API
- `skills/video_idea_create.md`

**Dependencies:**
```toml
[dependencies]
# For Asana
asana-rs = "0.1"  # Or use reqwest directly

# For GitHub
octocrab = "0.38"
```

**Configuration:**
```toml
[video_pipeline]
enabled = true
default_task_system = "asana"  # or "github"

[video_pipeline.asana]
access_token = "YOUR_TOKEN"
workspace_id = "YOUR_WORKSPACE"
project_id = "YOUR_PROJECT"

[video_pipeline.github]
token = "ghp_YOUR_TOKEN"
owner = "yourusername"
repo = "video-ideas"
```

**Estimated effort:** 40 hours (1 week)
**Risk:** Medium - Depends on external integrations (Asana/GitHub, Twitter API)

---

### 3.2 Twitter/X Multi-Tier Search

**Goal:** Cost-optimized Twitter search with fallback chain.

#### Implementation

**File:** `src/integrations/twitter/mod.rs`
```rust
pub enum TwitterTier {
    FxTwitter,      // Free, single tweet only
    TwitterApiIo,   // $0.15/1k tweets, search enabled
    OfficialV2,     // $0.005/tweet, expensive
    XArchTool,      // Grok via XAI
}

pub struct TwitterSearcher {
    config: TwitterConfig,
    usage_tracker: Arc<UsageTracker>,
}

impl TwitterSearcher {
    pub async fn search(&self, query: TwitterQuery) -> Result<Vec<Tweet>> {
        // Try each tier in order
        for tier in &self.config.enabled_tiers {
            match self.try_tier(tier, &query).await {
                Ok(tweets) => {
                    self.log_usage(tier, tweets.len()).await?;
                    return Ok(tweets);
                },
                Err(e) if e.is_quota_exceeded() => {
                    eprintln!("[twitter] {} quota exceeded, trying next tier", tier);
                    continue;
                },
                Err(e) => return Err(e),
            }
        }

        Err(anyhow!("All Twitter search tiers exhausted"))
    }

    async fn try_tier(&self, tier: &TwitterTier, query: &TwitterQuery) -> Result<Vec<Tweet>> {
        match tier {
            TwitterTier::FxTwitter => {
                // Only works for single tweet URL
                if let Some(tweet_id) = query.extract_tweet_id() {
                    self.fetch_fx_twitter(tweet_id).await
                } else {
                    Err(anyhow!("FxTwitter only supports single tweet lookups"))
                }
            },
            TwitterTier::TwitterApiIo => {
                self.search_twitterapi_io(query).await
            },
            TwitterTier::OfficialV2 => {
                // Check budget first
                let budget = self.check_budget("twitter_official").await?;
                if budget.remaining_usd < 5.0 {
                    return Err(anyhow!("Twitter API v2 budget too low"));
                }
                self.search_official_v2(query).await
            },
            TwitterTier::XArchTool => {
                self.search_via_grok(query).await
            },
        }
    }
}
```

**Configuration:**
```toml
[twitter]
enabled = true
enabled_tiers = ["fx_twitter", "twitterapi_io", "official_v2", "xarch"]

[twitter.fx_twitter]
enabled = true

[twitter.twitterapi_io]
enabled = true
api_key = "YOUR_KEY"
cost_per_thousand = 0.15

[twitter.official_v2]
enabled = true
bearer_token = "YOUR_TOKEN"
cost_per_tweet = 0.005
monthly_budget_usd = 50.0

[twitter.xarch]
enabled = true
xai_api_key = "YOUR_KEY"
```

**Skill:** `skills/twitter_search.md`
```markdown
# Twitter Search

## Trigger
User queries like:
- "Search Twitter for reactions to Claude 4.6"
- "Find tweets about [topic]"
- "Show me what people are saying about [company]"

## Steps
1. Parse query and intent
2. Execute multi-tier search (automatic fallback)
3. Filter results by engagement (likes + retweets > threshold)
4. Format results with clickable links
5. Log usage and costs

## Response Format
```
🐦 Twitter Search Results (via TwitterAPI.io - $0.03)

1. @username (❤️ 450 🔄 120)
   "[Tweet text...]"
   🔗 https://twitter.com/...

2. @username (❤️ 230 🔄 89)
   "[Tweet text...]"
   🔗 https://twitter.com/...

💰 Cost: $0.03 | 📊 20 tweets searched
```
```

**Estimated effort:** 30 hours (0.75 weeks)
**Risk:** Medium - API availability and rate limits

---

## Phase 4: Meta-Workflows (Weeks 13-16)

### 4.1 Business Council (Multi-Agent Meta-Analysis)

**Goal:** Multi-agent collaboration for business insights.

This is the most complex workflow, requiring multi-agent orchestration.

#### Architecture

```
┌─────────────────────────────────────────────┐
│         Business Council Orchestrator        │
│  ┌────────────────────────────────────────┐ │
│  │  Phase 1: Signal Collection            │ │
│  │  - YouTube analytics                   │ │
│  │  - CRM health                          │ │
│  │  - Cron reliability                    │ │
│  │  - Usage/costs                         │ │
│  │  - Email volume                        │ │
│  └────────────┬───────────────────────────┘ │
│               ▼                              │
│  ┌────────────────────────────────────────┐ │
│  │  Phase 2: Draft Analysis               │ │
│  │  (Claude Sonnet 4.5 - Fast)           │ │
│  └────────────┬───────────────────────────┘ │
│               ▼                              │
│  ┌────────────────────────────────────────┐ │
│  │  Phase 3: Multi-Agent Council          │ │
│  │  ┌──────────────┐  ┌──────────────┐   │ │
│  │  │ Growth       │  │ Revenue      │   │ │
│  │  │ Strategist   │  │ Guardian     │   │ │
│  │  └──────────────┘  └──────────────┘   │ │
│  │  ┌──────────────┐  ┌──────────────┐   │ │
│  │  │ Skeptical    │  │ Team         │   │ │
│  │  │ Operator     │  │ Dynamics     │   │ │
│  │  └──────────────┘  └──────────────┘   │ │
│  │  (All Claude Sonnet 4.5)              │ │
│  └────────────┬───────────────────────────┘ │
│               ▼                              │
│  ┌────────────────────────────────────────┐ │
│  │  Phase 4: Consensus & Ranking          │ │
│  │  (Claude Opus 4.6 - Best reasoning)   │ │
│  └────────────┬───────────────────────────┘ │
│               ▼                              │
│  ┌────────────────────────────────────────┐ │
│  │  Phase 5: Daily Report                 │ │
│  │  (Telegram notification)               │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

#### Database Schema
```sql
CREATE TABLE business_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date INTEGER NOT NULL,
    signal_type TEXT NOT NULL,  -- 'youtube', 'crm', 'cron', 'costs', 'email'
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    metric_change_pct REAL,  -- % change from previous period
    confidence REAL NOT NULL,  -- 0-100
    context TEXT,  -- JSON with additional details
    INDEX idx_date (date),
    INDEX idx_type (signal_type)
);

CREATE TABLE council_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date INTEGER NOT NULL,
    draft_analysis TEXT NOT NULL,
    agent_outputs TEXT NOT NULL,  -- JSON with all 4 agent responses
    final_report TEXT NOT NULL,
    recommendations TEXT NOT NULL,  -- JSON array
    created_at INTEGER NOT NULL
);
```

#### Implementation

**Skill:** `skills/business_council.md`
```markdown
# Daily Business Council

## Trigger
Cron: 2 AM daily

## Phase 1: Signal Collection (15-20 minutes)

Aggregate metrics from all sources:

### YouTube Analytics
```rust
let yt_signals = vec![
    Signal {
        metric_name: "views_24h",
        value: youtube_api.get_channel_views(days=1).await?,
        change_pct: calculate_change_vs_avg(),
        confidence: 95.0,
    },
    Signal {
        metric_name: "subscriber_growth",
        value: youtube_api.get_subscriber_growth(days=1).await?,
        change_pct: calculate_change_vs_avg(),
        confidence: 90.0,
    },
    // ... more metrics
];
```

### CRM Health
```sql
SELECT
    COUNT(*) as new_contacts_24h,
    COUNT(CASE WHEN last_contact_date < (now() - 2592000) THEN 1 END) as stale_contacts,
    AVG(interaction_count) as avg_interactions
FROM contacts
WHERE first_contact_date > (now() - 86400);
```

### Cron Reliability
```sql
SELECT
    COUNT(CASE WHEN status='failed' THEN 1 END) as failures_24h,
    COUNT(CASE WHEN status='success' THEN 1 END) as successes_24h
FROM cron_logs
WHERE timestamp > (now() - 86400);
```

### Cost Tracking
```sql
SELECT
    SUM(cost_usd) as total_cost_24h,
    workflow,
    service
FROM usage_log
WHERE timestamp > (now() - 86400)
GROUP BY workflow, service
ORDER BY total_cost_24h DESC
LIMIT 20;
```

### Email Volume
```rust
let email_signals = gmail_client.list_emails("after:yesterday", 1000).await?;
Signal {
    metric_name: "emails_received_24h",
    value: email_signals.len() as f64,
    confidence: 100.0,
}
```

**Rank signals by confidence and select top 200**

## Phase 2: Draft Analysis (5 minutes)

Use Claude Sonnet 4.5 (fast, cheap):
```
You are a business analyst reviewing daily metrics.

Signals (top 200):
{signals_json}

Task:
1. Identify the 5 most significant changes (positive or negative)
2. Note any unusual patterns or anomalies
3. Suggest 3-5 areas that warrant deeper investigation

Keep response concise (max 500 words).
```

## Phase 3: Multi-Agent Council (10-15 minutes)

Launch 4 agents in parallel with different system prompts:

**Agent 1: Growth Strategist**
```
You are a growth strategist focused on audience and revenue expansion.

Draft analysis:
{draft}

Raw signals:
{signals}

Task:
Review the draft and signals. Identify:
1. Opportunities for audience growth
2. Underutilized channels or content types
3. Trends that could be capitalized on
4. Revenue growth strategies

Provide 2-3 specific, actionable recommendations.
```

**Agent 2: Revenue Guardian**
```
You are a financial analyst focused on costs, ROI, and sustainability.

Draft analysis:
{draft}

Raw signals:
{signals}

Task:
Review the draft and signals. Identify:
1. Cost overruns or inefficiencies
2. Workflows with poor ROI
3. Budget risks
4. Revenue optimization opportunities

Provide 2-3 specific, actionable recommendations.
Flag any immediate financial concerns.
```

**Agent 3: Skeptical Operator**
```
You are a skeptical operator who challenges assumptions and identifies risks.

Draft analysis:
{draft}

Other agents' recommendations:
{agent1_output}
{agent2_output}

Task:
Review the draft and other agents' recommendations. Identify:
1. Flaws in reasoning or assumptions
2. Overlooked risks
3. Recommendations that may backfire
4. Alternative perspectives

Provide constructive criticism and 1-2 counter-recommendations.
```

**Agent 4: Team Dynamics Architect**
```
You are a team health specialist focused on workload, burnout, and collaboration.

Draft analysis:
{draft}

Raw signals:
{signals}

Task:
Review the draft and signals. Identify:
1. Workload balance issues
2. Signs of burnout or inefficiency
3. Collaboration gaps
4. Process improvements

Provide 2-3 specific, actionable recommendations focused on team health.
```

**Execute agents concurrently:**
```rust
let (output1, output2, output3, output4) = tokio::join!(
    call_agent("growth_strategist", &context),
    call_agent("revenue_guardian", &context),
    call_agent("skeptical_operator", &context),
    call_agent("team_dynamics", &context),
);
```

## Phase 4: Consensus Building (5 minutes)

Use Claude Opus 4.6 (best reasoning):
```
You are a council moderator synthesizing insights from 4 specialist agents.

Draft analysis:
{draft}

Agent outputs:
- Growth Strategist: {output1}
- Revenue Guardian: {output2}
- Skeptical Operator: {output3}
- Team Dynamics: {output4}

Task:
1. Reconcile disagreements between agents
2. Rank all recommendations by (impact × feasibility)
3. Produce final report with:
   - Top 3-5 priorities (actionable insights)
   - Supporting evidence from signals
   - Dissenting opinions (if significant)
   - Recommended next steps

Format as markdown suitable for Telegram.
```

## Phase 5: Delivery & Storage

**Store in database:**
```rust
db.execute(
    "INSERT INTO council_reports (date, draft_analysis, agent_outputs, final_report, recommendations) VALUES (?, ?, ?, ?, ?)",
    params![date, draft, agents_json, final_report, recommendations_json],
)?;
```

**Send to Telegram:**
```
📊 Daily Business Council Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━

🗓️ {date}

🔥 Top Priorities

1. [Priority 1 with emoji]
   Impact: ★★★★★
   [Details...]

2. [Priority 2]
   Impact: ★★★★☆
   [Details...]

💡 Opportunities
- [Opportunity 1]
- [Opportunity 2]

⚠️ Risks
- [Risk 1]

📈 Key Metrics (24h)
- Views: {views} ({change}%)
- Costs: ${costs}
- Emails: {email_count}

[View Full Report] [View Signals] [Dismiss]
```

## Performance & Cost

**Timing:**
- Signal collection: 15-20 min
- Draft analysis: 5 min
- Multi-agent council: 10-15 min (parallel)
- Consensus: 5 min
- **Total: 35-45 minutes**

**Cost per run:**
- Sonnet 4.5 (draft): ~$0.20
- Sonnet 4.5 (4 agents × 2000 tokens each): ~$0.80
- Opus 4.6 (consensus): ~$0.60
- **Total: ~$1.60/day = $48/month**

## Error Handling
- If any agent fails: Continue with remaining agents
- If consensus fails: Send draft + individual agent outputs
- If signal collection incomplete: Note missing sources in report
```

**Implementation files:**
- `src/workflows/business_council/mod.rs`
- `src/workflows/business_council/signals.rs` - Signal collection
- `src/workflows/business_council/agents.rs` - Multi-agent orchestration
- `src/workflows/business_council/consensus.rs` - Opus-based synthesis
- `skills/business_council.md`

**Tests:**
- Unit test: Signal ranking by confidence
- Unit test: Agent prompt generation
- Integration test: Multi-agent execution (with mocked Claude API)
- Integration test: Report generation

**Estimated effort:** 70 hours (1.75 weeks)
**Risk:** High - Complex multi-agent orchestration, high API costs

---

### 4.2 Memory Synthesis & Markdown Validation

**Goal:** Self-improvement through automated learning.

#### Skills

**Skill:** `skills/weekly_memory_synthesis.md`
```markdown
# Weekly Memory Synthesis

## Trigger
Cron: Sunday 11 PM

## Steps

### 1. Collect Daily Notes
Read all files from `~/.rustyclaw/memory/daily/YYYY-MM-DD.md` for past 7 days

### 2. Extract Patterns
Use Claude Sonnet 4.5:
```
Review these daily notes from the past week:

{daily_notes}

Extract and categorize:

1. **User Preferences**
   - Communication style preferences
   - Workflow patterns
   - Tools and commands frequently used

2. **Successful Patterns**
   - Workflows that worked well
   - Effective problem-solving approaches
   - Useful tool combinations

3. **Mistakes & Learnings**
   - Errors made
   - Wrong assumptions
   - Better approaches discovered

4. **System Improvements**
   - Feature requests
   - Configuration changes
   - New skills needed

Keep each category concise. Focus on patterns, not individual events.
```

### 3. Update MEMORY.md
```rust
let mut memory = fs::read_to_string("~/.rustyclaw/memory/MEMORY.md")?;

// Update each section
memory = update_section(&memory, "## User Preferences", &preferences);
memory = update_section(&memory, "## Successful Patterns", &successes);
memory = update_section(&memory, "## Mistakes to Avoid", &mistakes);

// Ensure <200 lines
if memory.lines().count() > 200 {
    memory = condense_memory(&memory, 200)?;
}

fs::write("~/.rustyclaw/memory/MEMORY.md", memory)?;
```

### 4. Update Learnings Folder
```rust
// Append new mistakes to dedicated file
let mistakes_file = "~/.rustyclaw/memory/learnings/mistakes.md";
fs::OpenOptions::new()
    .append(true)
    .open(mistakes_file)?
    .write_all(format!("\n## Week of {}\n{}", date, mistakes).as_bytes())?;
```

### 5. Archive Daily Notes
```bash
mkdir -p ~/.rustyclaw/memory/archive/$(date +%Y-%m)
mv ~/.rustyclaw/memory/daily/*.md ~/.rustyclaw/memory/archive/$(date +%Y-%m)/
```

### 6. Report
Send to Telegram:
```
🧠 Weekly Memory Synthesis Complete

📝 Synthesized 7 daily notes
✨ Extracted 12 new patterns
⚠️ Added 3 mistakes to avoid
📦 Archived to memory/archive/2026-02

Updated MEMORY.md: 187/200 lines
```
```

**Skill:** `skills/daily_markdown_validation.md`
```markdown
# Daily Markdown File Validation

## Trigger
Cron: 3 AM daily

## Steps

### 1. Collect All Markdown Files
```rust
let mut md_files = Vec::new();
md_files.extend(glob("~/.rustyclaw/skills/*.md")?);
md_files.extend(glob("~/.rustyclaw/config/*.md")?);
md_files.extend(glob("~/.rustyclaw/memory/*.md")?);
```

### 2. Load Best Practices
```rust
// Store Anthropic's prompt engineering guide locally
let anthropic_guide = fs::read_to_string(
    "~/.rustyclaw/docs/anthropic_prompt_guide.md"
)?;

// Store RustyClaw-specific patterns
let rustyclaw_patterns = fs::read_to_string(
    "~/.rustyclaw/docs/rustyclaw_best_practices.md"
)?;
```

### 3. Validate Each File
For each markdown file, use Claude Sonnet 4.5:
```
You are validating RustyClaw configuration files against best practices.

File: {filename}
Content:
{file_content}

Best Practices:
{anthropic_guide}
{rustyclaw_patterns}

Check for:
1. **Claude 4.6 Prompt Guidelines**
   - Avoid ALL CAPS, excessive bold, "CRITICAL" language
   - Use clear, concise language
   - Proper XML tag usage

2. **Skill Structure** (for skills/*.md)
   - Clear "Trigger" section
   - Step-by-step "Steps" section
   - Expected "Output" section

3. **Syntax Errors**
   - Invalid cron schedules
   - Broken markdown formatting
   - Outdated API patterns

4. **Consistency**
   - Consistent terminology
   - No conflicting instructions

Respond with:
- Status: OK | WARNINGS | ERRORS
- Issues found (if any)
- Suggested fixes
```

### 4. Aggregate Results
```rust
let mut report = ValidationReport {
    total_files: md_files.len(),
    ok: 0,
    warnings: 0,
    errors: 0,
    suggestions: Vec::new(),
};

for result in validation_results {
    match result.status {
        Status::OK => report.ok += 1,
        Status::Warnings => {
            report.warnings += 1;
            report.suggestions.push(result);
        },
        Status::Errors => {
            report.errors += 1;
            report.suggestions.push(result);
        },
    }
}
```

### 5. Send Report
Send to Telegram "system_health" topic:
```
📝 Markdown Validation Report
━━━━━━━━━━━━━━━━━━━━━━━━

✅ OK: 18 files
⚠️ Warnings: 3 files
❌ Errors: 0 files

Suggestions:

1. skills/crm_sync.md (⚠️ Warning)
   Line 42: Uses deprecated yup-oauth2 API
   Suggestion: Update to yup-oauth2 v10.0 pattern

2. memory/MEMORY.md (⚠️ Warning)
   File is 215 lines (limit: 200)
   Suggestion: Condense "Successful Patterns" section

3. skills/video_idea.md (⚠️ Warning)
   Lines 12-15: Uses ALL CAPS for emphasis
   Suggestion: Opus 4.6 doesn't need caps, use clear language

[Auto-fix All] [Review] [Dismiss]
```

### 6. Optional Auto-Fix
If user approves:
```rust
for suggestion in report.suggestions {
    if suggestion.auto_fixable {
        apply_fix(&suggestion)?;
    }
}
```
```

**Implementation files:**
- `src/memory/synthesis.rs`
- `src/validation/markdown_validator.rs`
- `skills/weekly_memory_synthesis.md`
- `skills/daily_markdown_validation.md`

**Estimated effort:** 35 hours (0.9 weeks)
**Risk:** Low - Straightforward text processing

---

## Phase 5: Messenger Integration (Weeks 17-18)

### 5.1 Telegram Bot Implementation

**Goal:** Primary interface with per-topic routing.

**Dependencies:**
```toml
[dependencies]
teloxide = { version = "0.13", features = ["macros"] }
```

**Configuration:**
```toml
[messengers.telegram]
enabled = true
bot_token = "YOUR_BOT_TOKEN"
allowed_users = [123456789]  # Your Telegram user ID

# Per-topic session routing
[[messengers.telegram.topics]]
topic = "knowledge_base"
session_id = "kb-main"
description = "Article and document storage"

[[messengers.telegram.topics]]
topic = "crm"
session_id = "contacts-main"
description = "Personal CRM and meeting prep"

[[messengers.telegram.topics]]
topic = "video_ideas"
session_id = "content-pipeline"
description = "Video research and task creation"

[[messengers.telegram.topics]]
topic = "system_health"
session_id = "monitoring-main"
description = "Cron jobs, backups, health checks"

[[messengers.telegram.topics]]
topic = "general"
session_id = "general-main"
description = "General conversation"

[session]
# Match OpenClaw's 1-year expiration
expiration_hours = 8760  # 365 days
```

**Implementation:**

**File:** `src/messengers/telegram/mod.rs`
```rust
use teloxide::prelude::*;

pub struct TelegramBot {
    bot: Bot,
    config: TelegramConfig,
    gateway_tx: mpsc::Sender<GatewayMessage>,
}

impl TelegramBot {
    pub async fn start(config: TelegramConfig, gateway_tx: mpsc::Sender<GatewayMessage>) -> Result<()> {
        let bot = Bot::new(&config.bot_token);

        let handler = Update::filter_message()
            .branch(
                dptree::entry()
                    .filter(|msg: Message| Self::is_authorized(&msg, &config))
                    .endpoint(Self::handle_message)
            );

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![config, gateway_tx])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    fn is_authorized(msg: &Message, config: &TelegramConfig) -> bool {
        if let Some(user) = msg.from() {
            config.allowed_users.contains(&user.id.0)
        } else {
            false
        }
    }

    async fn handle_message(
        bot: Bot,
        msg: Message,
        config: TelegramConfig,
        gateway_tx: mpsc::Sender<GatewayMessage>,
    ) -> ResponseResult<()> {
        // Determine session based on topic
        let session_id = if let Some(thread_id) = msg.thread_id {
            // Map thread_id to session_id via config
            config.topics.iter()
                .find(|t| t.thread_id == thread_id)
                .map(|t| t.session_id.clone())
                .unwrap_or_else(|| "general-main".to_string())
        } else {
            "general-main".to_string()
        };

        // Extract message content
        let content = msg.text().unwrap_or("");

        // Handle file attachments
        let attachments = extract_attachments(&msg).await?;

        // Send to gateway
        gateway_tx.send(GatewayMessage {
            session_id,
            content: content.to_string(),
            attachments,
            reply_to: Some(TelegramReplyHandle {
                bot: bot.clone(),
                chat_id: msg.chat.id,
                thread_id: msg.thread_id,
            }),
        }).await?;

        Ok(())
    }
}

async fn extract_attachments(msg: &Message) -> Result<Vec<Attachment>> {
    let mut attachments = Vec::new();

    // Handle documents (PDFs, etc.)
    if let Some(document) = &msg.document() {
        attachments.push(Attachment::Document {
            file_id: document.file.id.clone(),
            file_name: document.file_name.clone(),
            mime_type: document.mime_type.clone(),
        });
    }

    // Handle photos
    if let Some(photo) = msg.photo() {
        let largest = photo.iter().max_by_key(|p| p.width).unwrap();
        attachments.push(Attachment::Photo {
            file_id: largest.file.id.clone(),
        });
    }

    // Handle URLs in message
    if let Some(text) = msg.text() {
        for url in extract_urls(text) {
            attachments.push(Attachment::Url(url));
        }
    }

    Ok(attachments)
}
```

**Estimated effort:** 35 hours (0.9 weeks)
**Risk:** Low - Well-documented teloxide library

---

## Implementation Summary

### Timeline

| Phase | Weeks | Components |
|-------|-------|------------|
| **Phase 1: Foundation** | 1-3 | Hybrid DB, Gmail completion, Cost tracking |
| **Phase 2: Core Workflows** | 4-7 | CRM, Knowledge Base, Backup/Health |
| **Phase 3: Advanced** | 8-12 | Video Pipeline, Twitter Search |
| **Phase 4: Meta-Workflows** | 13-16 | Business Council, Memory/Validation |
| **Phase 5: Messengers** | 17-18 | Telegram Bot |

**Total: 18 weeks (~4.5 months)**

### Effort Breakdown

| Component | Hours | Risk |
|-----------|-------|------|
| Hybrid Database System | 40 | Medium |
| Gmail OAuth + API + Webhook | 30 | Low |
| Cost Tracking | 20 | Low |
| Personal CRM | 60 | Medium |
| Knowledge Base | 50 | Medium |
| Backup & Health | 25 | Low |
| Video Idea Pipeline | 40 | Medium |
| Twitter Multi-Tier Search | 30 | Medium |
| Business Council | 70 | High |
| Memory Synthesis & Validation | 35 | Low |
| Telegram Bot | 35 | Low |
| **Total** | **435 hours** | |

### Dependencies

```
Hybrid DB ──┬──> CRM
            ├──> Knowledge Base
            ├──> Video Pipeline
            └──> Business Council

Gmail ──────┬──> CRM
            └──> Knowledge Base (email ingestion)

Cost Tracking ──> Business Council

CRM ─────────┬──> Meeting Prep
             └──> Business Council

Knowledge Base ──> Video Pipeline

Telegram Bot ───> All workflows (interface)
```

### Critical Path

1. **Hybrid Database** (Week 1) - Blocks CRM, KB, Video, Council
2. **Gmail Feature** (Week 2) - Blocks CRM
3. **Cost Tracking** (Week 2-3) - Blocks Council
4. **CRM** (Week 4-5) - Blocks Meeting Prep, Council
5. **Knowledge Base** (Week 6-7) - Blocks Video Pipeline
6. **Video Pipeline** (Week 8-9) - No blockers
7. **Business Council** (Week 13-15) - Needs all signals
8. **Telegram Bot** (Week 17-18) - Can be done anytime

---

## Testing Strategy

### Unit Tests
- Database: CRUD operations, hybrid queries
- Signal collection: Metric calculation accuracy
- Agent prompts: Correct context injection
- Cost calculation: Pricing accuracy

### Integration Tests
- Gmail sync: Mock API, verify contact extraction
- Knowledge Base: End-to-end add + search
- Video pipeline: Research + task creation
- Business Council: Multi-agent execution (mocked Claude API)

### Performance Tests
- Database: Query performance with 10k+ records
- Vector search: Latency with 1M+ embeddings
- Concurrent sessions: 10+ active sessions

### Load Tests
- Gateway: 100 concurrent WebSocket connections
- Telegram: 50 messages/second
- API quotas: Verify rate limiting works

---

## Rollout Strategy

### Phase 1: Foundation (Weeks 1-3)
- Deploy hybrid database
- Complete Gmail feature
- Enable cost tracking
- **Validation:** Can store data, track costs

### Phase 2: Single Workflow MVP (Weeks 4-5)
- Deploy CRM only
- Test with real Gmail data
- Iterate based on usage
- **Validation:** CRM works end-to-end

### Phase 3: Expand Workflows (Weeks 6-12)
- Add one workflow at a time
- Test each independently
- Integrate with CRM
- **Validation:** Each workflow functional

### Phase 4: Advanced Features (Weeks 13-16)
- Deploy Business Council
- Enable memory synthesis
- **Validation:** Meta-analysis produces insights

### Phase 5: Interface (Weeks 17-18)
- Deploy Telegram bot
- Test per-topic routing
- **Validation:** Can use via Telegram

---

## Success Criteria

### Foundation
- [ ] Hybrid database supports SQL + vector queries
- [ ] Gmail OAuth works with token refresh
- [ ] Cost tracking logs all API calls
- [ ] Databases backed up hourly

### Core Workflows
- [ ] CRM syncs daily, tracks 100+ contacts
- [ ] Knowledge base stores 50+ articles with semantic search
- [ ] Meeting prep generates daily briefings
- [ ] Backup completes in <5 minutes

### Advanced Workflows
- [ ] Video pipeline creates task in <60 seconds
- [ ] Twitter search uses cost-optimized tiers
- [ ] Health check runs daily, alerts on issues

### Meta-Workflows
- [ ] Business Council produces actionable insights
- [ ] Memory synthesis maintains <200 line MEMORY.md
- [ ] Markdown validation catches errors

### Interface
- [ ] Telegram bot routes to correct sessions
- [ ] Per-topic expiration works (1 year)
- [ ] File attachments processed correctly

---

## Future Enhancements (Out of Scope)

- **Slack integration** - Similar to Telegram
- **Voice interface** - Whisper transcription
- **YouTube analytics** - Channel tracking
- **Fathom integration** - Meeting transcripts
- **Asana/HubSpot** - Bidirectional sync
- **Image/Video generation** - Replicate API
- **Multi-user support** - Team collaboration

---

## Risk Mitigation

### High-Risk Components

**Business Council (Multi-Agent)**
- Risk: Complex orchestration, high costs
- Mitigation: Extensive mocking in tests, cost budgets, graceful degradation

**Gmail API Quotas**
- Risk: Quota exceeded during sync
- Mitigation: Exponential backoff, hourly sync instead of real-time

**Vector Search Performance**
- Risk: Slow queries with large datasets
- Mitigation: Benchmark with realistic data, consider dedicated Qdrant server

### Dependencies on External Services

| Service | Risk | Mitigation |
|---------|------|------------|
| Gmail API | Quota limits | Caching, batch requests |
| Twitter API | Rate limits, costs | Multi-tier fallback |
| Anthropic API | Costs, downtime | Budget alerts, model fallback |
| Asana/GitHub | API changes | Version pinning, graceful errors |

---

## Cost Estimates

### Monthly Operating Costs

| Service | Usage | Cost |
|---------|-------|------|
| **Anthropic API** | | |
| - Business Council (daily) | Sonnet + Opus | $48 |
| - Other workflows | ~2M tokens/month | $30 |
| **Gmail API** | 10k requests/day | $5 |
| **Twitter API** | Mixed tiers | $15 |
| **Voyage AI** (embeddings) | 5M tokens/month | $5 |
| **VPS** (Optional) | Dedicated server | $10 |
| **Total** | | **$113/month** |

Similar to OpenClaw user's ~$150/month budget.

---

## Next Steps

To begin implementation, recommend starting with:

**Week 1: Hybrid Database Foundation**
1. Create `src/database/` module structure
2. Implement SQLite backend with rusqlite
3. Add SQLite-VSS for vector search
4. Write unit tests for CRUD + hybrid queries
5. Document API with examples

**Week 2: Complete Gmail Feature**
1. Implement OAuth device flow in `src/gmail/auth.rs`
2. Implement API client in `src/gmail/client.rs`
3. Test with real Gmail account
4. Deploy webhook server (optional for now)

**Week 3: Cost Tracking**
1. Create usage_log database schema
2. Wrap Anthropic API calls with logging
3. Create cost reporting skill
4. Test with real API usage

Would you like me to:
- **A)** Create feature branches for Phase 1 (hybrid-db, gmail-complete, cost-tracking)
- **B)** Start implementing the Hybrid Database foundation
- **C)** Write detailed technical specs for a specific workflow (e.g., CRM)
