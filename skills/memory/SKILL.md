# Memory Skill

## Core Scripts

### `remember.js` — Record something new
```bash
node skills/memory/scripts/remember.js <type> <text> [importance 1-5] [tags...]
```

**Types:**
- `episodic` — something that happened (dated automatically)
- `semantic` — a fact or preference about Zachary
- `procedural` — a lesson or workflow pattern

**Importance:** 1-5 (5 = critical, 1 = forget soon)

**Examples:**
```bash
# Zachary said remember this about project X
node skills/memory/scripts/remember.js semantic "Zachary prefers dark mode" 4 preference UI

# A decision was made
node skills/memory/scripts/remember.js episodic "Chose PostgreSQL over MongoDB" 5 decision database project-x

# I made a mistake and learned from it
node skills/memory/scripts/remember.js procedural "Don't use rm -rf in this workspace" 5 mistake safety
```

### `recall.js` — Search memory
```bash
node skills/memory/scripts/recall.js [search <query>] [limit]
```

**Examples:**
```bash
# Search all memory types
node skills/memory/scripts/recall.js search "project-x"

# Get recent topics
node skills/memory/scripts/recall.js
```

## Memory Types

| Type | Location | What it stores | Decay |
|------|----------|-----------------|-------|
| **Episodic** | `memory/episodic/YYYY-MM-DD.jsonl` | Events, conversations, milestones | Compresses after 30 days |
| **Semantic** | `memory/semantic/*.jsonl` | Facts, preferences, "remember this" | Never (importance-based) |
| **Procedural** | `memory/procedural/*.jsonl` | Workflows, lessons, patterns | Never (importance-based) |

## Importance Scale

| Score | When to use |
|-------|-------------|
| 5 | Major decisions, emotional moments, critical lessons |
| 4 | Significant preferences stated, nontrivial help |
| 3 | Normal useful interactions (default) |
| 2 | Casual mentions, potentially useful |
| 1 | Low-value, will decay quickly |

## Workflow

1. **Session start:** `recall search <current-project>` to load relevant context
2. **During session:** `remember` things worth keeping as they come up
3. **Session end:** If significant, `remember episodic` the milestone
4. **Periodic review:** Update `MEMORY.md` from daily files

## Index

The `memory/index.json` tracks topics and tags for fast lookup. Updated automatically on every `remember`.

## Decay

Low-importance entries (1-2) decay and get purged. High-importance (4-5) are permanent. This prevents memory bloat while keeping what matters.