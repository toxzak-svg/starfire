import json, sys
log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"
with open(log_path, "r") as f:
    lines = f.readlines()
print(f"Total errors logged: {len(lines)}")
unnotified = [l for l in lines if '"notified": false' in l]
print(f"Unnotified: {len(unnotified)}")
if unnotified:
    last = json.loads(unnotified[-1])
    print(f"Latest error: [{last['error_type']}] {last['error_message'][:80]}")
# Mark all as notified
with open(log_path, "w") as f:
    for line in lines:
        if '"notified": false' in line:
            entry = json.loads(line)
            entry["notified"] = True
            f.write(json.dumps(entry) + "\n")
        else:
            f.write(line)
print("All entries marked as notified.")
