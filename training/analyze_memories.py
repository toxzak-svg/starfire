import sqlite3
from pathlib import Path
from collections import Counter

db = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\star\star.db')
conn = sqlite3.connect(db)
cur = conn.cursor()

# Sample memories by domain
cur.execute("SELECT domain, COUNT(*) FROM memories GROUP BY domain ORDER BY COUNT(*) DESC LIMIT 20")
print("Memory domains:")
for domain, count in cur.fetchall():
    print(f"  {domain}: {count}")

# Sample actual content by domain
print("\n--- Sample memories by domain ---")
for domain in ['episodic', 'semantic', 'identity', 'procedural', 'general']:
    cur.execute("SELECT content FROM memories WHERE domain = ? AND content IS NOT NULL LIMIT 2", (domain,))
    rows = cur.fetchall()
    if rows:
        print(f"\n[{domain}]:")
        for content, in rows:
            print(f"  {str(content)[:150]}...")

# Total with actual content
cur.execute("SELECT COUNT(*) FROM memories WHERE content IS NOT NULL AND LENGTH(content) > 20")
print(f"\nMemories with real content: {cur.fetchone()[0]}")

conn.close()
