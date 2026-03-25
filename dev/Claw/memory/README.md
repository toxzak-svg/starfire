# Memory — Schema & Usage

This folder holds the agent’s **episodic** (what happened) and **daily** logs. Long-term curated memory lives in **`MEMORY.md`** (workspace root).

## Files

| File | Purpose |
|------|--------|
| **`YYYY-MM-DD.md`** | Daily log: raw notes for that day. Agent reads today + yesterday each session. |
| **`index.json`** | (Optional) Index of memory entries for associative recall by topic/query. |
| **`episodes.jsonl`** | (Optional) One JSON object per line: episodic events with type, date, importance, tags. |
| **`semantic.jsonl`** | (Optional) One JSON object per line: semantic facts (preferences, decisions) from consolidation. |
| **`consolidation-log.md`** | (Optional) Log of when and what was consolidated. |
| **`heartbeat-state.json`** | Track last check times for heartbeat (email, calendar, etc.). See AGENTS.md. |

## Memory types (human-like)

- **Episodic** — Something that happened: a conversation, a decision, an event. Has a date and often a short summary.
- **Semantic** — Stable facts: user preferences, timezone, “remember this” facts. Key–value style.
- **Procedural** — How to do something: “deploy to Cloudflare”, “run tests”. Topic + summary.

When you write to MEMORY.md or add structured entries, tag or type them accordingly so retrieval (and any future tooling) can use them.

## Entry shapes (for scripts / optional tooling)

**Episodic** (event):

```json
{"id": "ep-2026-03-11-a1b2", "type": "episodic", "date": "2026-03-11T14:00:00Z", "summary": "User asked to plan human-like memory.", "tags": ["planning", "memory"], "importance": 0.8, "source": "session"}
```

**Semantic** (fact/preference):

```json
{"id": "sem-2026-03-11-x9y8", "type": "semantic", "key": "user_timezone", "value": "User prefers Europe/Berlin.", "date": "2026-03-11", "importance": 1}
```

**Procedural** (how-to):

```json
{"id": "proc-2026-03-11-p1q2", "type": "procedural", "topic": "deploy_cloudflare", "summary": "Use Cloudflare skill; wrangler deploy.", "date": "2026-03-11", "importance": 0.7}
```

If you add **index.json**, each entry can be summarized there with `id`, `type`, `date`, `importance`, `tags`, `summary` (or `key`/`value` for semantic) so a recall step can search by topic or query without parsing every file.

## Usage

- **Every session:** Agent reads `memory/YYYY-MM-DD.md` for today and yesterday (and MEMORY.md in main session).
- **After something important:** Append to the day’s `YYYY-MM-DD.md`; optionally add an episodic or semantic entry to MEMORY.md’s structured section or to `episodes.jsonl` / `semantic.jsonl` if you use them.
- **“Remember this”:** Update MEMORY.md and optionally add a semantic entry with high importance (e.g. 1).
- **Recall by topic:** If `index.json` (or episodes/semantic JSONL) exists, search by tags or summary/key when loading context for the current turn.

Create `YYYY-MM-DD.md` for today if it doesn’t exist. Create other optional files only when you add tooling or want structured recall.
