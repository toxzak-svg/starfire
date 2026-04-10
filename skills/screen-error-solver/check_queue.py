import json
with open(r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\fix_queue.json") as f:
    q = json.load(f)
print(f"Queue size: {len(q)}")
for e in q:
    print(f"  [{e['status']}] {e['error_type']}: {e['error_message'][:80]}")
