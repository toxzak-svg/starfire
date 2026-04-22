# Memory Schema

## Three Memory Types

### Episodic (memory/episodic/)
**What happened when.** Raw events, conversations, milestones.
- One JSONL file per day: `YYYY-MM-DD.jsonl`
- Each entry: `{ts, type, summary, importance, tags}`
- Importance: 1-5 (5 = major decision, emotional moment, mistake learned from)

### Semantic (memory/semantic/)
**Facts and preferences.** Things that are true and stable.
- `facts.jsonl` — verified facts about Zachary and the world
- `preferences.jsonl` — stated likes, dislikes, working style
- `remember.jsonl` — things Zachary said "remember this" about

### Procedural (memory/procedural/)
**How to do X.** Patterns, workflows, lessons.
- `workflows.jsonl` — how things work in this workspace
- `lessons.jsonl` — mistakes and what I learned from them
- `patterns.jsonl` — recurring situations and how I handle them

## Index System

`memory/index.json` — searchable reference:
```json
{
  "topics": {
    "project-x": {"last": "2026-04-15", "tags": ["important", "active"]},
    "zachary-preferences": {"last": "2026-04-20", "tags": ["semantic"]}
  }
}
```

Updated on every remember/record.

## Importance Scoring

| Score | Meaning |
|-------|---------|
| 5 | Major decision, emotional moment, mistake learned |
| 4 | Significant preference stated, nontrivial help provided |
| 3 | Normal useful interaction |
| 2 | Casual mention, potentially useful |
| 1 | Low-value, decays fast |

## Decay Rules

- Importance 1: purge after 7 days
- Importance 2: purge after 30 days
- Importance 3: purge after 90 days
- Importance 4-5: never purge
- All episodic entries: compress to summary after 30 days

## Conflict Detection

When recording semantic that contradicts existing:
1. Mark old entry as "superseded"
2. Add new entry with "supersedes: [ref]"
3. Log in lessons if it's a belief change