import sqlite3
from pathlib import Path

STAR_DIR = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\star')
train_db = STAR_DIR / "training.db"

conn = sqlite3.connect(train_db)
conn.set_trace_callback(None)
cur = conn.cursor()

# Look at a single conversation
cur.execute("SELECT id FROM conversations LIMIT 1")
cid, = cur.fetchone()

cur.execute("SELECT speaker, content FROM turns WHERE conversation_id = ? ORDER BY turn_index", (cid,))
rows = cur.fetchall()

print(f"Conversation {cid}:")
for speaker, content in rows:
    c = str(content)[:80] if content else "None"
    print(f"  {speaker}: {c}...")

# Count pairs properly
pairs_found = 0
current_user = None
for speaker, content in rows:
    if not content:
        continue
    if speaker == 'zachary':
        current_user = content
    elif speaker == 'star' and current_user:
        pairs_found += 1
        current_user = None

print(f"\nPairs found: {pairs_found}")

conn.close()
