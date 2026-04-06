import sqlite3
import os
from pathlib import Path

STAR_DIR = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\star')

for db_path in [STAR_DIR / "training.db", STAR_DIR / "star.db"]:
    if not db_path.exists():
        continue
    print(f"\n{'='*60}")
    print(f"DB: {db_path}")
    conn = sqlite3.connect(db_path)
    conn.set_trace_callback(None)
    cur = conn.cursor()
    
    cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
    for table, in cur.fetchall():
        cur.execute(f"PRAGMA table_info({table})")
        cols = [(r[1], r[2]) for r in cur.fetchall()]
        cur.execute(f"SELECT COUNT(*) FROM {table}")
        count, = cur.fetchone()
        print(f"\n{table} ({count} rows):")
        for col_name, col_type in cols:
            print(f"  {col_name}: {col_type}")
    
    conn.close()
