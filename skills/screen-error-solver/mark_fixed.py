import json

log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"

with open(log_path, "r") as f:
    lines = f.readlines()

fixed_ids = {
    "af38120497a2",  # API server 8080 - now on 8081
    "7dd30ea52cf9",  # API server not running - now fixed
    "d5413185bc95",  # Socket 10013 on 8080 - now on 8081
    "3042b65bbdb38",  # Socket permission - now on 8081
    "26566117c964",  # Cargo build error - wrong dir, not reproducible
    "2cdd39adfc50d",  # API not running - now fixed
    "33b46b8e6b52",  # API not running - now fixed
    "4f916213fd25",  # SIGKILL - watcher running fine now
    "133d6ce0e534",  # SIGKILL - watcher running fine now
    "314bf8b39d1d",  # IDENTITY.md - now copied
    "a6dccaf1e3b9",  # IDENTITY/SOUL not found - now fixed
    "16ce488e39fe",  # Health check - should rerun
    "4c65268561f5",  # Star needs attention - API now on 8081
}

marked = 0
with open(log_path, "w") as f:
    for line in lines:
        try:
            entry = json.loads(line)
            eid = entry.get("id", "")[:12]
            if eid in fixed_ids:
                entry["notified"] = True
                marked += 1
            f.write(json.dumps(entry) + "\n")
        except:
            f.write(line)

print(f"Marked {marked} entries as notified")
