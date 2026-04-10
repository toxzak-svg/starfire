import json
log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"
with open(log_path, "r") as f:
    lines = f.readlines()
print(f"Total entries: {len(lines)}")
for i, line in enumerate(lines):
    entry = json.loads(line)
    print(f"{'UNNOTIFIED' if not entry.get('notified') else 'notified':12} [{entry['error_type']}] {entry['error_message'][:60]}")
