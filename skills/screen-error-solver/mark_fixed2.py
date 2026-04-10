import json

log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"

with open(log_path, "r") as f:
    lines = f.readlines()

# Keywords that mean the error is FIXED
fix_keywords = [
    "API server not running on port 8080",
    "Socket Access Permission Error",
    "An attempt was made to access a socket",
    "SIGKILL",
    "IDENTITY.md not found",
    "SOUL.md not found",
    "Star API server started on port 8081",
    "Star API server was started on port 8081",
]

# Keywords that mean it's a false alarm / not actionable
false_alarm = [
    ".venv",
    "lm-evaluation-harness",
    "spawnSync /bin/sh ENOENT",
    "shell env fallback failed",
]

marked = 0
with open(log_path, "w") as f:
    for line in lines:
        try:
            entry = json.loads(line)
            msg = entry.get("error_message", "") + entry.get("context_summary", "")
            
            # Mark fixed as notified
            if any(kw in msg for kw in fix_keywords):
                entry["notified"] = True
                marked += 1
            # Mark false alarms as notified
            if any(kw in msg for kw in false_alarm):
                entry["notified"] = True
                marked += 1
            f.write(json.dumps(entry) + "\n")
        except:
            f.write(line)

print(f"Marked {marked} entries as notified")

# Count remaining
with open(log_path, "r") as f:
    remaining = sum(1 for line in f if '"notified": false' in line)
print(f"Remaining unnotified: {remaining}")
