import json

log_path = r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_screen_errors.log"

with open(log_path, "r") as f:
    lines = f.readlines()

marked = 0
with open(log_path, "w") as f:
    for line in lines:
        try:
            entry = json.loads(line)
            if not entry.get("notified", False):
                entry["notified"] = True
                marked += 1
            f.write(json.dumps(entry) + "\n")
        except:
            f.write(line)

print(f"Marked {marked} as notified")
with open(log_path, "r") as f:
    remaining = sum(1 for line in f if '"notified": false' in line)
print(f"Remaining unnotified: {remaining}")
