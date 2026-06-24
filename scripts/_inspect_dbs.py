import sqlite3, os, json

base = r"C:\Users\Zwmar\projects\starfire"
for db in ["training.db", "star.db", "voice.db", "library.db"]:
    p = os.path.join(base, db)
    if not os.path.exists(p):
        print(f"--- {db}: missing ---")
        continue
    print(f"--- {db} ({os.path.getsize(p)} bytes) ---")
    try:
        c = sqlite3.connect(p)
        cur = c.cursor()
        cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
        for (t,) in cur.fetchall():
            n = cur.execute(f"SELECT COUNT(*) FROM {t}").fetchone()[0]
            print(f"  {t}: {n} rows")
            try:
                cols = [r[1] for r in cur.execute(f"PRAGMA table_info({t})").fetchall()]
                print(f"    cols: {cols}")
            except Exception as e:
                print(f"    cols err: {e}")
        c.close()
    except Exception as e:
        print(f"  err: {e}")
