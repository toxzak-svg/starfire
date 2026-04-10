import json

log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"

with open(log_path, "r") as f:
    lines = f.readlines()

# Errors that are fixed/resolved
fix_keywords = [
    "API server started on port 8081",
    "SIGKILL",
    "SpawnSync /bin/sh ENOENT",
    ".venv",
    "lm-evaluation-harness",
    "Enterprise features",
    "Draft Session off",
    "allow_pickle",
]

marked = 0
with open(log_path, "w") as f:
    for line in lines:
        try:
            entry = json.loads(line)
            msg = entry.get("error_message", "") + entry.get("context_summary", "")
            if any(kw in msg for kw in fix_keywords):
                entry["notified"] = True
                marked += 1
            f.write(json.dumps(entry) + "\n")
        except:
            f.write(line)

print(f"Marked {marked}")

with open(log_path, "r") as f:
    remaining = sum(1 for l in f if '"notified": false' in l)
print(f"Remaining unnotified: {remaining}")
