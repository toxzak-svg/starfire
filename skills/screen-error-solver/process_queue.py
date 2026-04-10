"""Process the fix queue — spawn coding agents for pending critical fixes."""
import json, os, sys

QUEUE_FILE = os.path.join(os.path.dirname(__file__), "fix_queue.json")

def process_queue():
    if not os.path.exists(QUEUE_FILE):
        return "Queue file not found"

    with open(QUEUE_FILE, "r") as f:
        queue = json.load(f)

    pending = [e for e in queue if e.get("status") == "pending" and e.get("severity") == "critical"]
    if not pending:
        return f"No pending critical fixes ({len(queue)} total entries)"

    entry = pending[0]
    error_type = entry.get("error_type", "unknown")
    error_msg = entry.get("error_message", "")[:300]
    fix = entry.get("fix", "")
    file_loc = entry.get("file_location", "")
    lang = entry.get("language", "")
    context = entry.get("context_summary", "")

    print(f"[FIX QUEUE] Processing: {error_type}")
    print(f"[FIX QUEUE] Message: {error_msg[:100]}")
    print(f"[FIX QUEUE] Fix: {fix[:200]}")

    # Build coding task for the agent
    task = f"""Auto-fix this error found on the developer's screen:

## Error
- Type: {error_type}
- Message: {error_msg}
- File/Location: {file_loc}
- Language: {lang}

## Context
{context}

## Suggested Fix
{fix}

Instructions:
1. Read the relevant source files
2. Apply the fix
3. If it's a test failure, run the test to verify
4. If it's a build error, try to compile
5. Commit your changes with a descriptive message
6. Report what you did

Working directory: C:\\Users\\Zwmar\\.openclaw\\workspace
Project: find the relevant project from the file location
"""

    # Mark as in-progress
    entry["status"] = "in_progress"
    with open(QUEUE_FILE, "w") as f:
        json.dump(queue, f, indent=2)

    return task

if __name__ == "__main__":
    result = process_queue()
    print(result)
    if result.startswith("No pending"):
        sys.exit(0)
    else:
        print("\n[SPAWNING CODING AGENT...]")
        # The actual spawn happens in the cron job handler
