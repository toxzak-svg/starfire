import sqlite3
import os

for db_path in [
    r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\star.db',
    r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\training.db', 
    r'C:\Users\Zwmar\.openclaw\workspace\projects\star\star.db',
    r'C:\Users\Zwmar\.openclaw\workspace\projects\star\training.db',
    r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\voice.db',
]:
    if os.path.exists(db_path):
        try:
            conn = sqlite3.connect(db_path)
            cur = conn.cursor()
            cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
            tables = [r[0] for r in cur.fetchall()]
            print(f"\n{db_path}:")
            print(f"  Tables: {tables}")
            for table in tables:
                cur.execute(f"SELECT COUNT(*) FROM {table}")
                count = cur.fetchone()[0]
                print(f"  {table}: {count} rows")
            conn.close()
        except Exception as e:
            print(f"\n{db_path}: ERROR {e}")
