"""Process pending error-fix approvals from Zach.

On heartbeat: read pending_approvals.json, check for approved entries,
spawn coding agent for each approved fix, mark done/failed.
"""
import json, os, sys

APPROVALS_FILE = os.path.join(
    os.environ.get("MARBLE_WATCHER_DIR",
        r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver"),
    "pending_approvals.json"
)

def load_approvals():
    if not os.path.exists(APPROVALS_FILE):
        return []
    with open(APPROVALS_FILE) as f:
        return json.load(f)

def save_approvals(entries):
    with open(APPROVALS_FILE, "w") as f:
        json.dump(entries, f, indent=2)

def process_approvals():
    entries = load_approvals()
    if not entries:
        return "No pending approvals"

    pending = [e for e in entries if e.get("status") == "pending"]
    if not pending:
        return f"No pending approvals ({len(entries)} total)"

    entry = pending[0]
    eid = entry.get("id", "?")
    error = entry.get("error", {})
    fix = entry.get("fix", "")
    err_type = error.get("error_type", "unknown")
    err_msg = error.get("error_message", "")[:300]
    file_loc = error.get("file_location", "not visible")
    lang = error.get("language", "")
    context = error.get("context_summary", "")

    print(f"[APPROVAL] Processing approved fix: {eid} — {err_type}")

    # Build the coding task
    task = f"""Auto-fix this error (approved by Zach):

## Error
- Type: {err_type}
- Message: {err_msg}
- File/Location: {file_loc}
- Language: {lang}

## Context
{context}

## Suggested Fix
{fix[:2000]}

Instructions:
1. Find the relevant source files from the file location
2. Apply the fix
3. If it's a test failure: run the specific test to verify
4. If it's a build error: try to compile
5. Commit with a descriptive message like "fix: {err_type} - {err_msg[:80]}"
6. Report what you did

Working directory: C:\\Users\\Zwmar\\.openclaw\\workspace
Be thorough — this was approved by Zach, so he wants it fixed.
"""

    # Mark as in_progress
    entry["status"] = "in_progress"
    save_approvals(entries)

    return f"APPROVED_FIX:{eid}", task, entry

if __name__ == "__main__":
    result = process_approvals()
    if isinstance(result, tuple):
        eid, task, entry = result
        print(f"[APPROVAL] Ready to spawn agent for {eid}")
        print(f"[TASK]\n{task[:500]}...")
    else:
        print(result)
