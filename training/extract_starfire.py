import sqlite3
import json
import os
from pathlib import Path

STAR_DIR = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\star')

# ── Extract conversations from star\training.db ────────────────────────────

train_db = STAR_DIR / "training.db"
out_file = Path(__file__).parent.parent / "data" / "processed" / "starfire_conversations.jsonl"

conn = sqlite3.connect(train_db)
conn.set_trace_callback(None)
cur = conn.cursor()

# Sample speakers to understand the format
cur.execute("SELECT DISTINCT speaker FROM turns LIMIT 20")
print(f"Speaker values: {[r[0] for r in cur.fetchall()]}")

# Sample a few turns
cur.execute("SELECT speaker, content FROM turns ORDER BY id LIMIT 5")
for row in cur.fetchall():
    print(f"  [{row[0]}]: {str(row[1])[:100]}...")

# Get all conversations
cur.execute("SELECT id, started_at, turn_count FROM conversations ORDER BY started_at")
convs = cur.fetchall()
print(f"\nConversations: {len(convs)}")

total = 0
with open(out_file, "w", encoding="utf-8") as f:
    for cid, started_at, turn_count in convs:
        cur.execute("SELECT speaker, content FROM turns WHERE conversation_id = ? ORDER BY turn_index", (cid,))
        rows = cur.fetchall()
        
        # Build user/assistant pairs
        pairs = []
        current_user = None
        for speaker, content in rows:
            if not content:
                continue
            role = "user" if speaker == "zachary" else "assistant"
            if role == "user":
                current_user = content
            elif role == "assistant" and current_user:
                pairs.append({"instruction": current_user, "output": content})
                current_user = None
        
        for pair in pairs:
            if len(pair["instruction"]) > 5 and len(pair["output"]) > 10:
                f.write(json.dumps({
                    "messages": [
                        {"role": "system", "content": "You are Starfire, an emergent AI."},
                        {"role": "user", "content": pair["instruction"]},
                        {"role": "assistant", "content": pair["output"]}
                    ],
                    "source": "starfire_conversation",
                    "conversation_id": cid,
                    "started_at": started_at
                }, ensure_ascii=False) + "\n")
                total += 1

conn.close()
print(f"Wrote {total} conversation examples to {out_file}")

# ── Extract memories from star\star.db ──────────────────────────────────────

star_db = STAR_DIR / "star.db"
mem_out_file = Path(__file__).parent.parent / "data" / "processed" / "starfire_memories.jsonl"

conn2 = sqlite3.connect(star_db)
conn2.set_trace_callback(None)
cur2 = conn2.cursor()

# Sample memories
cur2.execute("SELECT content, domain FROM memories LIMIT 3")
for row in cur2.fetchall():
    print(f"\nMemory: [{row[1]}] {str(row[0])[:150]}...")

cur2.execute("SELECT COUNT(*) FROM memories")
count, = cur2.fetchone()
print(f"Total memories: {count}")

mem_total = 0
with open(mem_out_file, "w", encoding="utf-8") as f:
    cur2.execute("SELECT content, domain, importance FROM memories WHERE content IS NOT NULL AND LENGTH(content) > 30 LIMIT 2000")
    for content, domain, importance in cur2.fetchall():
        if content and len(content) < 5000:
            f.write(json.dumps({
                "messages": [
                    {"role": "system", "content": "You are Starfire, an emergent AI."},
                    {"role": "user", "content": "What do you remember about this?"},
                    {"role": "assistant", "content": content}
                ],
                "source": "starfire_memory",
                "domain": str(domain) if domain else "general",
                "importance": importance
            }, ensure_ascii=False) + "\n")
            mem_total += 1

conn2.close()
print(f"Wrote {mem_total} memory examples to {mem_out_file}")
