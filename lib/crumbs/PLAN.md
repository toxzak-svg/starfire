# CrumbStore → Starfire Integration Plan
**Created: 2026-04-05**

---

## Architecture Decision: Why NOT PyO3

Two options for integrating crumbs into Starfire:

| Approach | Pros | Cons |
|----------|------|------|
| **PyO3 / python3-rs** | Direct Rust→Python calls | Heavy extra dependency; Python interpreter still needed |
| **Subprocess (CLI)** | Simple, already works | IPC overhead; startup latency on every call |
| **Rust reimplementation** | Fast, zero-dep, idiomatic | Dual-maintenance of Python + Rust crumb logic |

**Decision: Rust reimplementation** — but thin and complementary.

Starfire already uses SQLite (rusqlite). CrumbStore uses JSON files. They coexist. The Python CrumbStore stays for Zach's CLI use and the existing 44 tests. Starfire gets its own native Rust CrumbStore that reads/writes the same `~/.openclaw/crumbs/` files — meaning both can operate on the same crumb store without conflicts.

---

## Dual-Maintenance Strategy

```
~/.openclaw/crumbs/         ← Shared storage
     ├── index.json         ← Shared index
     ├── 0001_crumb_*.json  ← Individual crumb files
     └── ...

Python CrumbStore ◄──────────────────────────────► Rust CrumbStore
(zach's CLI + tests)                             (Starfire integration)
```

Both read/write the same JSON files. This means:
- Zach uses Python CLI for manual crumb operations
- Starfire uses Rust CrumbStore for programmatic crumb operations
- No conflict, no duplication, no PyO3 dependency

---

## File Structure (Post-Integration)

```
~/.openclaw/crumbs/
├── index.json                    ← shared index
├── 0001_crumb_*.json             ← crumb files
└── (Python and Rust both read/write here)

projects/starfire/lib/crumbs/     ← NEW Rust CrumbStore
├── Cargo.toml
├── src/
│   ├── lib.rs                    ← public API
│   ├── types.rs                  ← Crumb, CrumbMeta, SearchQuery structs
│   ├── index.rs                  ← index.json read/write
│   ├── file.rs                   ← individual crumb file read/write
│   └── search.rs                 ← keyword + tag search logic
└── tests/
    ├── test_crumbs.rs            ← unit tests

projects/starfire/lib/
├── lib.rs                        ← add: pub mod crumbs;
└── ...
```

---

## CrumbData Types (Rust)

```rust
// src/types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crumb {
    pub id: String,
    #[serde(rename = "type")]
    pub crumb_type: String,      // "local", "web", "gist", "notion"
    pub created: String,         // ISO 8601
    pub updated: String,
    pub location: String,
    pub display_location: String,
    pub topic: String,
    pub summary: String,
    pub relevance: Vec<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub topic: Option<String>,
    pub crumb_type: Option<String>,
    pub author: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrumbIndex {
    pub version: u32,
    pub updated: String,
    pub total: usize,
    pub crumbs: HashMap<String, CrumbMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrumbMeta {
    pub topic: String,
    pub tags: Vec<String>,
    pub created: String,
    pub location: String,
    #[serde(rename = "type")]
    pub crumb_type: String,
    pub author: String,
}
```

---

## Starfire CrumbStore API

```rust
// src/lib.rs

/// Initialize the crumb store (ensure directory + index exist)
pub fn init() -> Result<()>;

### Create a crumb
pub fn create(
    location: &str,
    topic: &str,
    summary: &str,
    relevance: Vec<String>,
    tags: Vec<String>,
    author: &str,
) -> Result<Crumb>;

/// Search crumbs
pub fn search(query: SearchQuery) -> Result<Vec<Crumb>>;

/// Get a crumb by ID
pub fn get(crumb_id: &str) -> Result<Crumb>;

/// Update a crumb
pub fn update(crumb_id: &str, updates: CrumbUpdate) -> Result<Crumb>;

/// Delete a crumb
pub fn delete(crumb_id: &str) -> Result<()>;

/// List all crumbs
pub fn list_all(limit: usize) -> Result<Vec<Crumb>>;

/// Get statistics
pub fn stats() -> Result<CrumbStats>;
```

---

## Integration Points with Starfire Modules

### 1. WorldModel → CrumbStore
When WorldModel encounters an external reference (URL, file, paper), it can drop a crumb:

```rust
// In world_model/mod.rs or a new world_model/crumbs.rs
pub fn remember_location(location: &str, topic: &str, summary: &str) {
    CrumbStore::create(
        location,
        topic,
        summary,
        relevance: vec![topic.to_lowercase()],
        tags: vec!["world-model".to_string()],
        author: "starfire",
    ).ok();
}
```

### 2. Research → CrumbStore
When Research finds a useful URL:

```rust
// In research/mod.rs
pub fn note_find(url: &str, title: &str, summary: &str) {
    CrumbStore::create(
        location: url,
        topic: title,
        summary: summary,
        relevance: vec!["research".to_string()],
        tags: vec!["research".to_string()],
        author: "starfire",
    ).ok();
}
```

### 3. Curiosity → CrumbStore
When Curiosity identifies a knowledge gap, it can leave a crumb marking what needs investigation:

```rust
// In curiosity/mod.rs
pub fn leave_probe_crumb(topic: &str, gap_description: &str) {
    CrumbStore::create(
        location: "internal://curiosity-gap",
        topic: &format!("investigate: {}", topic),
        summary: gap_description,
        relevance: vec!["curiosity".to_string(), "gap".to_string()],
        tags: vec!["curiosity".to_string(), "investigate".to_string()],
        author: "starfire",
    ).ok();
}
```

### 4. memory/memory.rs Integration
When a Memory is formed about an external entity, also create a crumb:

```rust
// In persistence/memory.rs
impl Memory {
    pub fn with_crumb_location(mut self, location: &str) -> Self {
        // Leave a crumb at the external location pointing back to this memory
        if let Ok(crumb) = CrumbStore::create(...) {
            self.provenance = Some(crumb.id);
        }
        self
    }
}
```

### 5. cognition.rs Integration
In the main cognition loop, expose crumb operations:

```rust
// In cognition.rs
pub fn recall_related_crumbs(context: &str) -> Vec<Crumb> {
    CrumbStore::search(SearchQuery {
        query: Some(context.to_string()),
        limit: 5,
        ..Default::default()
    }).unwrap_or_default()
}
```

---

## Cargo Dependencies (crumbs/Cargo.toml)

```toml
[package]
name = "star-crumbs"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
dirs = { workspace = true }
chrono = { version = "0.4", optional = true }  # for timestamp parsing only

[dev-dependencies]
tempfile = "3"
```

No new heavy dependencies. Uses Starfire's workspace deps.

---

## Implementation Phases

### Phase 1: Rust CrumbStore Core ✅ Plan (Today)
- [ ] `lib/crumbs/Cargo.toml`
- [ ] `lib/crumbs/src/types.rs` — Crumb, CrumbIndex, SearchQuery
- [ ] `lib/crumbs/src/index.rs` — index.json read/write with locking
- [ ] `lib/crumbs/src/file.rs` — individual crumb file read/write
- [ ] `lib/crumbs/src/search.rs` — keyword + tag search
- [ ] `lib/crumbs/src/lib.rs` — public API (create, get, search, update, delete, list, stats)
- [ ] `lib/crumbs/tests/test_crumbs.rs` — unit tests
- [ ] Add `pub mod crumbs;` to `lib.rs`

### Phase 2: Starfire Integration ✅ Plan
- [ ] Wire CrumbStore into WorldModel (leave crumb when external reference found)
- [ ] Wire into Research module
- [ ] Wire into Curiosity module (leave probe crumbs for knowledge gaps)
- [ ] Add `crumbs` field to `SearchQuery` in cognition for cross-system recall
- [ ] Add `CrumbsBridge` — lets existing modules create crumbs without depending on crumb types directly

### Phase 3: CLI Parity ✅ Plan
- [ ] `lib/crumbs/src/cli.rs` — Rust CLI (`star-crumbs create/get/search/list/delete/stats`)
- [ ] Mirror the Python CLI interface exactly so Zach can use either

### Phase 4: File Locking + Safety ✅ Plan
- [ ] File-based locking for concurrent access (two sessions writing simultaneously)
- [ ] Handle malformed JSON gracefully (skip corrupted crumbs, log warning)
- [ ] Atomic writes (write to temp, rename — prevents corruption on crash)

### Phase 5: Advanced ✅ Plan
- [ ] Semantic search using embeddings (optional, if Starfire has an embedding model available)
- [ ] Expiration enforcement (skip expired crumbs on search)
- [ ] Thread support (parent_id field for crumb chains)

---

## Key Design Decisions

### 1. File-based, not SQLite
Crumbs are external pointers — they live in the filesystem where the resources they reference also live. JSON files are human-readable and Git-diffable. SQLite would be overkill.

### 2. Author field = "starfire" for programmatic crumbs
Python CLI creates crumbs with `author: "marble"`. Starfire creates with `author: "starfire"`. This distinguishes human-created vs. agent-created crumbs in search results.

### 3. Non-fatal integration
When CrumbStore operations fail (file not found, disk full, JSON parse error), Starfire logs the error but doesn't crash. Crumbs are side channels, not critical path.

### 4. `internal://` URI scheme for agent-generated crumbs
When Starfire creates a crumb about an internal thought (curiosity gap, reasoning trace), it uses `internal://` as the location type. This makes it clear the crumb points to Starfire's own cognitive artifacts, not an external resource.

### 5. Tag convention: module origin
When Starfire creates crumbs, it tags with the originating module:
- `world-model` — from the WorldModel module
- `curiosity` — from the Curiosity module
- `research` — from the Research module
- `marble` — from Python CLI (human-created)

This lets you filter crumbs by "who left them" and understand the provenance chain.

---

## Memory vs. Crumbs: Complementary Roles

| | Starfire Memory (SQLite) | CrumbStore (JSON) |
|--|--|--|
| **Stores** | Beliefs, reasoning traces, episodic facts | External references (URLs, files) |
| **Updated by** | Starfire cognition | Starfire + Marble (CLI) |
| **Format** | Structured (Belief, domain, confidence) | Unstructured (topic, summary, tags) |
| **Query** | Structured SQL queries | Keyword/tag search |
| **Purpose** | Internal cognitive state | External world pointers |
| **TTL** | Memory decay, consolidation | Permanent until deleted |

**The workflow:**
1. Starfire reads a URL via Research or WorldModel
2. Creates a Memory about the content (internal knowledge)
3. Creates a Crumb pointing to the URL (external pointer)
4. Future session: queries Memory for what Starfire *knows*, queries Crumbs for where she *found* things

---

## Testing Strategy

### Unit tests (Rust)
- CRUD operations
- Search (all filter combinations)
- Index consistency
- File locking
- Malformed JSON recovery

### Integration tests
- Python + Rust read/write same files (cross-compatibility)
- Starfire cognition loop creates crumb → Python CLI reads it
- Python CLI creates crumb → Starfire reads it

### Property-based tests
- Generated crumb IDs are unique
- Search results respect limit
- Index count matches file count

---

## What to Build First

1. **`lib/crumbs/src/types.rs`** — Define the data structures first
2. **`lib/crumbs/src/lib.rs`** — Core API signatures
3. **`lib/crumbs/src/file.rs`** — Read/write individual crumbs
4. **`lib/crumbs/src/index.rs`** — Index management
5. **`lib/crumbs/src/search.rs`** — Search logic
6. **Wire into WorldModel** — First integration point
