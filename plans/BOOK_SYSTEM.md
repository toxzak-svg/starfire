# Starfire Book System — SPEC

## Concept

A hierarchical, bookmark-driven knowledge system for Starfire that behaves like a **personal library**: instant tab-switching between specialized knowledge domains (Railway deployments, medical symptoms, project context), where each "book" has sections of varying information density. The LLM never sees the whole library — only the paged-in sections relevant to the current focus.

Drawing from: MemGPT's virtual context paging, MemoryOS's heat-tracking, GraphRAG's hierarchical retrieval.

## Core Metaphor

```
World Wide Web          →    Starfire Library System
─────────────────────────────────────────────────────
URL (bookmark string)   →    `railway:logs:errors:last10`
Website                 →    Book "Railway Deployment"
Page                    →    Section (dense/light/packed)
Browser tabs            →    Active Focus Stack
Browser history         →    Starfire memory (memories/crumbs)
Search engine           →    Semantic sweep (Quanot-driven)
```

## Design Principles

1. **Information density is explicit** — each section declares its density (HIGH/MEDIUM/LOW)
2. **Bookmark strings are stable** — never change them, only extend books
3. **Books are additive, never overwritten** — new info appends, old stashes
4. **LLM sees only what's paged in** — never the full book unless requested
5. **Stashing is instant** — state is serialized, not re-computed

## Data Structures

### Bookmark String Format
```
{book}:{chapter}:{section}:{subsection}
```
Examples:
- `railway:env:secrets` — HIGH density
- `railway:logs:errors:railway_errors` — MEDIUM density
- `railway:status:current` — LOW density
- `medical:symptoms:fever:child` — HIGH density
- `project:starfire:module:causal` — MEDIUM density

### Book
```rust
pub struct Book {
    pub id: BookId,
    pub name: String,
    pub bookmark_prefix: String,    // e.g. "railway"
    pub chapters: Vec<Chapter>,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
}

pub struct Chapter {
    pub id: ChapterId,
    pub name: String,
    pub bookmark_prefix: String,    // e.g. "env"
    pub sections: Vec<Section>,
    pub density: Density,           // aggregate density hint
}

pub struct Section {
    pub id: SectionId,
    pub bookmark: String,           // full bookmark: "railway:env:secrets"
    pub content: String,           // raw content
    pub tokens: usize,             // token count (for load budgeting)
    pub density: Density,
    pub last_accessed: i64,
    pub version: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Density {
    /// Full detail — load everything: variable names, values, states
    High,
    /// Summary — key patterns, error signatures, state summaries
    Medium,
    /// Compressed — status codes, boolean flags, single facts
    Low,
    /// Packed — raw data dump, load on explicit request only
    Packed,
}
```

### Focus (Active Tab)
```rust
pub struct Focus {
    pub id: FocusId,
    pub book_id: BookId,
    pub pinned_sections: Vec<SectionId>,   // always-loaded sections
    pub active_chapters: Vec<ChapterId>,   // currently relevant chapters
    pub density_filter: Density,            // minimum density to load
    pub thread_id: Option<String>,         // if stashed, thread string
    pub stashed_at: Option<i64>,
}

impl Focus {
    /// Stash focus to a thread string — serializes state to a retrieval key
    pub fn stash(&mut self) -> String {
        // Generate: "thread:{book}:{chapter1,chapter2}:{pinned}:{timestamp}"
        let thread = format!(
            "thread:{}:{:?}:{:?}:{}",
            self.book_id,
            self.active_chapters,
            self.pinned_sections,
            crate::now_timestamp()
        );
        self.thread_id = Some(thread.clone());
        self.stashed_at = Some(crate::now_timestamp());
        thread
    }

    /// Restore focus from thread string
    pub fn restore(thread: &str) -> Option<Focus> { ... }
}
```

### Library Manifest (lightweight index)
```rust
pub struct LibraryManifest {
    pub books: HashMap<BookId, BookHeader>,  // all book metadata, no content
    pub active_focus: Option<FocusId>,       // currently active focus
    pub focus_stack: Vec<FocusId>,           // tab stack
}

pub struct BookHeader {
    pub id: BookId,
    pub name: String,
    pub bookmark_prefix: String,
    pub chapter_count: usize,
    pub section_count: usize,
    pub total_tokens: usize,
    pub density_breakdown: HashMap<Density, usize>,
    pub last_accessed: i64,
}
```

## Core Operations

### 1. Page In (load sections for a focus)
```rust
pub fn page_in(focus: &Focus, max_tokens: usize) -> Vec<Section> {
    // 1. Get pinned sections (always load)
    // 2. Load chapters by density filter
    // 3. Budget remaining tokens by density priority: HIGH > MEDIUM > LOW
    // 4. Return loaded sections + whether truncation occurred
}
```

### 2. Stash Focus (tab switch)
```rust
pub fn stash_focus(library: &mut Library, focus_id: FocusId) -> String {
    // 1. Serialize focus state to thread string
    // 2. Push to focus_stack
    // 3. Clear active focus
    // 4. Return thread string for later retrieval
}
```

### 3. Restore Focus (tab switch back)
```rust
pub fn restore_focus(library: &mut Library, thread: &str) -> Option<Focus> {
    // 1. Parse thread string
    // 2. Reconstruct focus
    // 3. Page in sections
    // 4. Return focus
}
```

### 4. Sweep (gather info across books)
```rust
pub fn sweep(library: &Library, query: &str, max_results: usize) -> Vec<SweepResult> {
    // Uses Quanot's semantic matching to find relevant sections across books
    // Returns: (bookmark, relevance_score, density, snippet)
}
```

### 5. Write Section (add knowledge)
```rust
pub fn write_section(
    library: &mut Library,
    bookmark: &str,
    content: &str,
    density: Density,
) -> Result<SectionId> {
    // 1. Parse bookmark → book/chapter/section
    // 2. If book doesn't exist, create it
    // 3. Append or update section
    // 4. Return new section ID
}
```

## Database Schema (SQLite)

```sql
CREATE TABLE books (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    bookmark_prefix TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0
);

CREATE TABLE chapters (
    id TEXT PRIMARY KEY,
    book_id TEXT NOT NULL REFERENCES books(id),
    name TEXT NOT NULL,
    bookmark_prefix TEXT NOT NULL,
    density TEXT NOT NULL,
    UNIQUE(book_id, bookmark_prefix)
);

CREATE TABLE sections (
    id TEXT PRIMARY KEY,
    chapter_id TEXT NOT NULL REFERENCES chapters(id),
    bookmark TEXT UNIQUE NOT NULL,
    content TEXT NOT NULL,
    tokens INTEGER NOT NULL,
    density TEXT NOT NULL,
    last_accessed INTEGER NOT NULL,
    version INTEGER DEFAULT 1
);

CREATE TABLE focuses (
    id TEXT PRIMARY KEY,
    book_id TEXT NOT NULL REFERENCES books(id),
    thread_id TEXT,
    stashed_at INTEGER,
    density_filter TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE focus_pins (
    focus_id TEXT NOT NULL REFERENCES focuses(id),
    section_id TEXT NOT NULL REFERENCES sections(id),
    PRIMARY KEY (focus_id, section_id)
);

CREATE VIRTUAL TABLE section_fts USING fts5(
    bookmark,
    content,
    content='sections',
    content_rowid='rowid'
);
```

## Information Density Profiles

### Railway Deployment
```
env:secrets        → HIGH  (env vars, API keys, actual values)
env:config_patterns → MEDIUM (common patterns, sanitized examples)
logs:errors:last10 → MEDIUM (last 10 errors with context)
logs:errors:summary → LOW   (error count, frequency, trend)
status:current     → LOW   (URL health: 200/500/timeout)
status:history     → PACKED (full history dump, load on demand)
```

### Medical
```
symptoms:fever     → HIGH  (temp ranges, duration, triggers, child/adult)
symptoms:fever:summary → MEDIUM (quick reference)
anatomy:body_region → MEDIUM (connected systems)
drug:interactions  → HIGH  (specific drug pairs, severity)
```

### Project (Starfire)
```
module:{name}      → MEDIUM (API surface, key structs)
module:{name}:spec → HIGH   (full spec, pub fn signatures)
module:{name}:test → MEDIUM (test coverage, status)
status:current     → LOW   (compiles? tests pass? 239/239)
```

## Speed Guarantees

| Operation | Target | Mechanism |
|-----------|--------|-----------|
| Manifest load | <1ms | HashMap, no I/O |
| Bookmark lookup | <1ms | Trie over bookmark prefixes |
| Section load (1) | <5ms | SQLite row fetch |
| Sweep (top-k) | <100ms | Quanot semantic match + SQLite FTS |
| Focus page-in (4 sections) | <20ms | Batched SQLite queries |

## Module Structure

```
lib/book/
├── mod.rs          — Library, Book, Focus, Density, public API
├── db.rs           — SQLite persistence layer
├── manifest.rs     — Lightweight manifest (no content loading)
├── pager.rs        — page_in(), density budgeting, token counting
├── sweep.rs        — Quanot-driven semantic sweep across books
├── thread.rs        — stash/restore focus threads
└── tests.rs        — unit tests
```

## Relation to Existing Systems

- **Starfire memory** (`lib/persistence`) — raw storage, not hierarchical. Book system layers on top.
- **Quanot** — used for semantic sweep. Learning engine provides relevance scoring.
- **World Model** — entities/relations already form a graph. Book sections can reference world-model entities.
- **CrumbStore** — bookmarks are stable paths, crumbs are volatile snapshots. Books outlast sessions.
- **Goals** — a Book can track a goal's context. Focus stacks on top of goal tracking.
- **Learning** — few-shot examples can be auto-generated from Book sections.

## Out of Scope (v1)

- Book merging / deduplication
- Multi-writer conflict resolution
- Book versioning / diff
- Semantic embedding index (use Quanot for now)
- Book sharing / export
