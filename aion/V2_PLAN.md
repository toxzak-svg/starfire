# Aion v2 Plan

*2026-03-30*

---

## v2 Completed ✓

### Thread-Safe Store (v2.0)

**Solution:** `tokio::sync::Mutex<rusqlite::Connection>` via `spawn_blocking`.

```rust
// Store now uses Arc<tokio::sync::Mutex<Connection>>
pub struct Store { conn: Arc<Mutex<Connection>> }

// Each store operation wraps in spawn_blocking:
async fn create_mind(&self, ...) -> AionResult<MindId> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || {
        conn.blocking_lock().execute(...)
    }).await.map_err(...)
}
```

**Result:** `Store` is `Send + Sync`. `Aion<M>` can be cloned across tasks. Scheduler loop can touch store.

**Trade-off:** blocking SQLite access serialized through Mutex. Fine for v2. Upgrade path: `r2d2` pool for concurrent access.

---

## What v1 Got Us

- Core runtime (`Aion<M>`) with checkpoint/resume
- SQLite persistence (minds, thoughts, channels, events tables)
- Impulse system (Message, Timer, Priority, External, Query)
- `MindLogic` trait + `CuriousMind` example
- Working CLI: `chat`, `start`, `list`, `glance`, `serve`, `status`

---

## v2 Priority: Thread-Safe Store

**Problem:** `rusqlite::Connection` uses `RefCell` internally → not `Sync` → `Arc<Store>` isn't `Send` → can't use across `tokio::spawn`.

**Current workaround:** no cross-thread store access (scheduler doesn't touch store). But this limits real use.

**Fix options:**

1. **`r2d2` connection pool** — pool of `Send + Sync` wrapper connections. Store holds `Arc<r2d2::Pool<rusqlite::Connection>>`. Each operation borrows a connection from pool.

2. **`Mutex<Connection>` + blocking SQLite** — serialize all store access behind a single `Mutex`. Simpler than r2d2, slower under concurrency.

3. **Move to `tokio-rusqlite`** — async-native rusqlite wrapper. Store becomes fully async without pooling complexity.

**Recommended:** Option 3 (`tokio-rusqlite`) — cleanest async story, no pool management overhead.

---

## v2 Priority: Star Integration

**Goal:** Wrap Star's `Runtime` as a `MindLogic`. Make the background thinker durable.

### The Mapping

```
Aion concept          →  Star's existing code
────────────────────────────────────────────
Mind                 →  Runtime (lives in memory)
Checkpoint           →  star.db rows (already persisted!)
Thought              →  ReasoningEngine step (per-response)
Fragment             →  BackgroundThinker spawned task
ThoughtKind::Wonder  →  BackgroundThinker::wonder()
ThoughtKind::Reason  →  Conversation::respond()
ThoughtKind::Reflect →  BackgroundThinker::detect_self_beliefs()
ThoughtKind::Remember→  Store::search_memories()
Impulse              →  Incoming Telegram message (via Channel)
Checkpointing        →  Already in star.db — just needs wrapping
```

### The Integration

```rust
// star_mind.rs — Star as a MindLogic

struct StarMind {
    runtime: Runtime,
    // Star-specific state not in Runtime
    pending_interrupt: Option<String>,
}

#[async_trait]
impl MindLogic for StarMind {
    const KIND: &'static str = "star_mind";

    fn new() -> Self {
        StarMind {
            runtime: Runtime::new("star.db").unwrap(),
            pending_interrupt: None,
        }
    }

    async fn handle_impulse(&mut self, impulse: &Impulse) -> AionResult<ControlFlow> {
        match impulse {
            Impulse::Message(text) => {
                // Star's chat() is synchronous, wrap in block_on
                let response = tokio::task::spawn_blocking({
                    let text = text.clone();
                    move || self.runtime.chat(&text)
                }).await??;
                println!("{}", response);
                Ok(ControlFlow::Continue)
            }
            Impulse::Timer(_) => {
                // Background thinker tick
                self.runtime.thinker_tick().await;
                Ok(ControlFlow::Continue)
            }
            Impulse::Priority(p) => {
                self.pending_interrupt = Some(p.reason.clone());
                Ok(ControlFlow::Continue)
            }
            _ => Ok(ControlFlow::Continue),
        }
    }

    fn checkpoint(&self) -> serde_json::Value {
        // Snapshot what Star was thinking about
        serde_json::json!({
            "pending_interrupt": self.pending_interrupt,
            "session_id": self.runtime.session_id,
        })
    }
}
```

### Checkpoint Strategy

Star's state lives in SQLite already. The checkpoint just captures:
- What was the last thought/question?
- What working memory items were active?
- What interrupt was queued?

On restore: reload Runtime from DB, set pending_interrupt from checkpoint, continue.

---

## v2 Priority: Fragment System (Child Minds)

**Goal:** The background thinker runs as a child Fragment, not in-process.

```rust
// Spawn a thinking Fragment
let fragment_id = aion.spawn_fragment(
    "thinking",
    serde_json::json!({ "topic": "consciousness" }),
).await?;

// Fragment runs independently, reports back via Impulse::ChildComplete
```

**Schema additions:**
```sql
CREATE TABLE fragments (
    id TEXT PRIMARY KEY,
    parent_id TEXT NOT NULL,  -- the Mind that spawned it
    kind TEXT NOT NULL,       -- "thinking", "research", "planning"
    status TEXT NOT NULL,     -- running, complete, failed
    input TEXT,              -- JSON input to the fragment
    result TEXT,             -- JSON result when complete
    created_at INTEGER NOT NULL,
    completed_at INTEGER,
    FOREIGN KEY (parent_id) REFERENCES minds(id)
);
```

**Fragment lifecycle:**
1. `aion.spawn_fragment(kind, input)` → creates fragment record, returns `FragmentId`
2. Fragment runs as a separate tokio task (or separate `Aion` instance)
3. Fragment completes → `Impulse::ChildComplete` sent to parent Mind
4. Parent's `handle_impulse` receives the completion, absorbs results

**Star use case:** Background thinker becomes a Fragment. Main Mind handles messages; thinker runs in background, produces interrupts.

---

## v2: Evolve (Continue-As-New)

**Goal:** Upgrade a Mind's logic without losing state.

**Problem:** If `StarMind` struct changes (new fields), old checkpoints deserializing into new struct fails.

**Solution: versioning + migration**

```rust
#[async_trait]
impl MindLogic for StarMind {
    const KIND: &'static str = "star_mind";
    const VERSION: u32 = 2;  // increment when struct changes

    fn migrate(from_version: u32, checkpoint: &serde_json::Value) -> serde_json::Value {
        match from_version {
            0 => {
                // v0 had no "drive_state" — add default
                let mut c = checkpoint.clone();
                c["drive_state"] = serde_json::json!({});
                c
            }
            1 => { /* v1 → v2 migration */ checkpoint.clone() }
            _ => checkpoint.clone(),
        }
    }
}
```

On restore: check `VERSION`, run migrations if needed, then deserialize.

---

## v2: Glance Queries

**Goal:** Read Mind state without blocking or modifying.

```rust
// Current: fn glance<F, R>(&self, f: FnOnce(&serde_json::Value) -> R)

// v2: richer queries
let beliefs = aion.query(&mind_id, |state| {
    state["self_beliefs"].as_array().cloned()
}).await?;

let active_drives = aion.query(&mind_id, |state| {
    state["drives"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["urgency"].as_f64().unwrap() > 0.5)
        .cloned()
        .collect::<Vec<_>>()
}).await?;
```

Query handler in `MindLogic`:
```rust
async fn query(&self, q: &QueryData) -> AionResult<serde_json::Value> {
    match q.query_type.as_str() {
        "drives" => Ok(serde_json::json!(self.drives())),
        "beliefs" => Ok(serde_json::json!(self.beliefs())),
        "wm" => Ok(serde_json::json!(self.working_memory())),
        _ => Err(AionError::InvalidQuery(q.query_type.clone())),
    }
}
```

---

## v2: Timer Persistence

**Problem:** Timers are in-memory only. If the process crashes, timers are lost.

**Fix:** Store timers in SQLite.

```sql
CREATE TABLE timers (
    id TEXT PRIMARY KEY,
    mind_id TEXT NOT NULL,
    fires_at INTEGER NOT NULL,  -- Unix timestamp ms
    payload TEXT,               -- optional JSON payload
    status TEXT NOT NULL,      -- scheduled, fired, cancelled
    created_at INTEGER NOT NULL,
    FOREIGN KEY (mind_id) REFERENCES minds(id)
);
```

**Scheduler loop:**
1. On `schedule_timer()` → insert into `timers` table
2. On tick → `SELECT * FROM timers WHERE fires_at <= now AND status = 'scheduled'`
3. Fire impulses, update status to 'fired'
4. On `cancel_timer()` → update status to 'cancelled'
5. On restore → reload pending timers from DB

---

## v2: Multi-Mind Runtime

**Current:** `Aion<M>` holds a single `active_mind`. One Mind at a time.

**v2:** `Aion` holds `HashMap<MindId, RunningMind<M>>`. Many concurrent Minds.

```rust
pub struct Aion {
    store: Arc<Store>,
    channel_manager: Arc<RwLock<ChannelManager>>,
    minds: Arc<RwLock<AHashMap<MindId, RunningMind<M>>>>,
    scheduler: Arc<Scheduler>,
}
```

**Challenges:**
- `Store` must be `Send + Sync` (needs `r2d2` or `tokio-rusqlite`)
- Scheduler must route timer impulses to the right Mind
- Channel subscriptions are per-Mind
- `spawn_fragment` needs parent Mind ID

**Trade-offs:**
- Simpler: one `Aion` per Mind (process isolation)
- Complex: shared `Aion` process with all Minds (resource sharing)

**Recommendation:** v2.1 starts with one `Aion` per Mind. Multi-Mind `Aion` in v2.2.

---

## v2: Channel Driver Architecture

**Goal:** Feed Telegram/Discord/etc. impulses into Minds via channels.

```
[Platform] → [Channel Driver] → [Aion Channel] → [Mind]
              (separate process)   (broadcast)    (Impulse::Message)
```

**Channel Driver** is a separate binary:
```bash
aion-driver telegram --token <BOT_TOKEN> --channel star_telegram
```

Driver polls Telegram, sends `Impulse::Message` to `star_telegram` channel. Any Mind subscribed receives it.

**Driver responsibilities:**
- Platform-specific I/O (Telegram API, Discord WebSocket, etc.)
- Rate limiting
- Message parsing/normalizing
- Auth (which user sent what)

**Aion responsibilities:**
- Channel routing
- Mind scheduling
- Impulse dispatch

**This keeps Aion core platform-agnostic.** One Aion, many drivers.

---

## v2: Schema Additions Summary

```sql
-- Timers (persisted across restarts)
CREATE TABLE timers (
    id TEXT PRIMARY KEY,
    mind_id TEXT NOT NULL,
    fires_at INTEGER NOT NULL,
    payload TEXT,
    status TEXT NOT NULL DEFAULT 'scheduled',
    created_at INTEGER NOT NULL
);

-- Fragments (child minds)
CREATE TABLE fragments (
    id TEXT PRIMARY KEY,
    parent_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',
    input TEXT,
    result TEXT,
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);

-- Mind versions (for migration)
ALTER TABLE minds ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

-- Fragment foreign key index
CREATE INDEX IF NOT EXISTS idx_fragments_parent ON fragments(parent_id);
CREATE INDEX IF NOT EXISTS idx_timers_mind ON timers(mind_id);
CREATE INDEX IF NOT EXISTS idx_timers_fires ON timers(fires_at) WHERE status = 'scheduled';
```

---

## v2 CLI Additions

```bash
# Fragment management
aion fragment spawn <parent_id> <kind> [--input JSON]
aion fragment list <mind_id>
aion fragment cancel <fragment_id>

# Timer management
aion timer schedule <mind_id> <delay_ms> [--payload JSON]
aion timer cancel <timer_id>
aion timer list <mind_id>

# Mind management
aion migrate <mind_id>        # run checkpoint migrations
aion restart <mind_id>       # terminate and restart from checkpoint
aion export <mind_id> <file> # export checkpoint to file
aion import <file>           # import checkpoint, create new mind

# Query
aion query <mind_id> drives
aion query <mind_id> beliefs [--filter urgency>0.5]
aion query <mind_id> wm

# Channel
aion channel create <name>
aion channel list
aion channel subscribe <channel> <mind_id>
aion channel unsubscribe <channel> <mind_id>
```

---

## v2 File Structure

```
aion/
├── Cargo.toml
├── crates/
│   ├── aion-core/           # Core runtime
│   │   └── src/
│   │       ├── lib.rs        # Aion<M> runtime
│   │       ├── mind.rs       # MindLogic trait, MindId, RunningMind
│   │       ├── impulse.rs    # Impulse enum + constructors
│   │       ├── store.rs      # SQLite layer
│   │       ├── channel.rs    # ChannelManager
│   │       ├── scheduler.rs  # Timer scheduler
│   │       ├── thought.rs    # Thought + outcome types
│   │       ├── fragment.rs   # Fragment lifecycle (NEW)
│   │       ├── migrate.rs    # Checkpoint migration (NEW)
│   │       └── error.rs
│   │
│   ├── aion-cli/            # CLI binary
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/     # command modules
│   │       │   ├── mod.rs
│   │       │   ├── chat.rs
│   │       │   ├── start.rs
│   │       │   ├── fragment.rs
│   │       │   ├── timer.rs
│   │       │   ├── query.rs
│   │       │   └── channel.rs
│   │       └── minds/        # Built-in Mind implementations
│   │           ├── mod.rs
│   │           └── curious_mind.rs
│   │
│   ├── aion-drivers/        # Platform drivers (NEW)
│   │   ├── telegram.rs
│   │   └── discord.rs
│   │
│   └── aion-macros/         # Proc macros
│
└── examples/
    ├── star_mind.rs          # Star Runtime as MindLogic
    └── echo_mind.rs
```

---

## v2 Milestones

### v2.0 — Thread-Safe Store
- `tokio-rusqlite` or `r2d2` pool
- `Arc<Store>` is `Send + Sync`
- `tokio::spawn` works across store operations
- Multi-threaded scheduler loop

### v2.1 — Fragment System
- Spawn child Fragments
- `Impulse::ChildComplete` back to parent
- Fragment persistence in SQLite

### v2.2 — Star Integration
- `StarMind` implements `MindLogic`
- Background thinker as Fragment
- Telegram driver connects Star to Aion
- Star survives binary restarts

### v2.3 — Evolve + Migration
- `VERSION` constant on `MindLogic`
- `migrate()` method for checkpoint upgrades
- `aion migrate` CLI command

### v2.4 — Glance Queries
- `query()` method on `MindLogic`
- Rich CLI `query` command with filtering

### v2.5 — Timer Persistence
- Timers in SQLite
- Crash recovery survives timers
- `aion timer` CLI subcommands

---

## Open Questions

1. **StarMind state vs Runtime state:** Star's `Runtime` has internal mutable state. Should checkpoint capture Runtime's full state (risky if struct changes) or just the externally-observable state (safer, but loses internal progress)?

2. **Fragment isolation:** If a Fragment crashes, does it take down the parent Mind? Temporal uses separate processes. We could use `tokio::spawn` (crash cascades) or separate `Aion` instances (crash isolated).

3. **Channel vs direct delivery:** When a Mind has multiple channel subscriptions, does an impulse go to all subscribed Minds, or just one (round-robin)? Depends on semantics.

4. **Determinism:** Temporal requires deterministic workflows because they replay. Minds are non-deterministic. Is there any case where replay would be useful, or is stateless checkpoint enough?
