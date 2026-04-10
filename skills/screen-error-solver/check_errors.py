import json, sys

log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"
with open(log_path, "r") as f:
    lines = f.readlines()

total = len(lines)
unnotified = []
for line in lines:
    try:
        entry = json.loads(line)
        if not entry.get("notified", False):
            unnotified.append(entry)
    except:
        pass

print(f"Total errors: {total}")
print(f"Unnotified: {len(unnotified)}")
for e in unnotified:
    print(f"  [{e['id']}] {e['error_type']}: {e['error_message'][:80]}")
    print(f"    severity: {e['severity']}")
