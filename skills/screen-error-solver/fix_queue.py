import json, os, hashlib
from datetime import datetime

QUEUE_FILE = os.path.join(os.environ.get("MARBLE_WATCHER_DIR", r"C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver"), "fix_queue.json")

def enqueue_fix(error: dict, fix: str, screenshot_b64: str = ""):
    """Add a fix task to the queue for Marble to process on next heartbeat."""
    # Use full message for hash — [:80] truncates differently per capture causing duplicate entries
    msg_for_hash = error.get("error_message", "") or ""
    entry = {
        "id": hashlib.md5(f"{error.get('error_type','')}:{msg_for_hash[:200]}".encode()).hexdigest()[:12],
        "timestamp": datetime.now().isoformat(),
        "error_type": error.get("error_type", "unknown"),
        "error_message": error.get("error_message", ""),
        "severity": error.get("severity", "warning"),
        "file_location": error.get("file_location", ""),
        "language": error.get("language", ""),
        "context_summary": error.get("context_summary", ""),
        "fix": fix,
        "screenshot_b64": screenshot_b64[:200] if screenshot_b64 else "",
        "status": "pending",  # pending | fixed | failed
        "notified": True,
    }
    try:
        queue = []
        if os.path.exists(QUEUE_FILE):
            with open(QUEUE_FILE, "r") as f:
                queue = json.load(f)
        # Don't duplicate — check by id AND by full fingerprint
        fp = f"{error.get('error_type','')}:{msg_for_hash}"
        if not any(e.get("id") == entry["id"] and e.get("status") == "pending" for e in queue):
            queue.append(entry)
            with open(QUEUE_FILE, "w") as f:
                json.dump(queue, f, indent=2)
            print(f"[FIX QUEUE] Added: [{error.get('error_type','')}] {msg_for_hash[:60]}")
    except Exception as e:
        print(f"[FIX QUEUE] Failed to enqueue: {e}")
