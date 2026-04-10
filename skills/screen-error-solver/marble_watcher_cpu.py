"""
Marble's Screen Watcher -- CPU-only edition.
No API calls. Pure OCR + regex pattern matching.
Runs as a background process, logs errors for Marble to pick up on heartbeat.
"""
import sys, os, base64, io, json, time, logging, hashlib, re, subprocess
from datetime import datetime
from pathlib import Path

# ── Dependencies ──────────────────────────────────────────────────────────────
try:
    import mss
    from PIL import Image
    MSS_AVAILABLE = True
except ImportError:
    MSS_AVAILABLE = False

try:
    import pytesseract
    pytesseract.pytesseract.tesseract_cmd = r"C:\Program Files\Tesseract-OCR\tesseract.exe"
    PYTESS_AVAILABLE = True
except ImportError:
    PYTESS_AVAILABLE = False

TELEGRAM_BOT_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q")
TELEGRAM_CHAT_ID = os.environ.get("TELEGRAM_CHAT_ID", "")  # Set via env or below


def notify_telegram(message: str, entry_id: str = ""):
    """Send a message via the BotShoppe Telegram bot and mark pending entry as notified."""
    if not TELEGRAM_CHAT_ID:
        log.warning("TELEGRAM_CHAT_ID not set — skipping notification")
        return False
    try:
        import urllib.request, urllib.parse
        url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
        data = urllib.parse.urlencode({
            "chat_id": TELEGRAM_CHAT_ID,
            "text": message,
            "parse_mode": "HTML",
        }).encode()
        req = urllib.request.Request(url, data=data, method="POST")
        with urllib.request.urlopen(req, timeout=10) as resp:
            success = resp.status == 200
            if success and entry_id:
                mark_notified(entry_id)
            return success
    except Exception as e:
        log.error(f"Telegram notify failed: {e}")
        return False


def mark_notified(entry_id: str):
    """Update notified=True for a pending entry."""
    try:
        if not PENDING_FILE.exists():
            return
        pending = json.loads(PENDING_FILE.read_text(encoding="utf-8"))
        if isinstance(pending, dict):
            for key, entry in pending.items():
                if entry.get("id") == entry_id:
                    entry["notified"] = True
        elif isinstance(pending, list):
            for entry in pending:
                if entry.get("id") == entry_id:
                    entry["notified"] = True
        PENDING_FILE.write_text(json.dumps(pending, indent=2), encoding="utf-8")
        log.info(f"Marked entry {entry_id} as notified")
    except Exception as e:
        log.error(f"Failed to mark notified: {e}")

# ── Paths ────────────────────────────────────────────────────────────────────
SCRIPT_DIR = Path(__file__).parent.resolve()
LOG_FILE = SCRIPT_DIR / "marble_screen_errors.log"
ERROR_HISTORY = SCRIPT_DIR / "error_history.jsonl"
PENDING_FILE = SCRIPT_DIR / "pending_approvals.json"
WATCHER_LOG = SCRIPT_DIR / "watcher.log"

# ── Logging ──────────────────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO,
    format="[MARBLE WATCHER CPU] %(asctime)s %(message)s",
    handlers=[
        logging.FileHandler(str(WATCHER_LOG)),
        logging.StreamHandler(sys.stdout),
    ],
)
log = logging.getLogger("marble_watcher_cpu")

# ── Error patterns (CPU-friendly regex) ─────────────────────────────────────
ERROR_PATTERNS = [
    # Python
    (re.compile(r"Traceback \(most recent call last\)", re.I), "Python Traceback", "critical"),
    (re.compile(r"(\w+Error): (.+?)(?:\n|$)", re.I), "Python Error", "critical"),
    (re.compile(r"ModuleNotFoundError: No module named '(\w+)'", re.I), "Import Error", "critical"),
    (re.compile(r"ImportError:", re.I), "Import Error", "critical"),
    (re.compile(r"SyntaxError:", re.I), "Syntax Error", "critical"),
    (re.compile(r"AttributeError:", re.I), "Attribute Error", "critical"),
    (re.compile(r"TypeError:", re.I), "Type Error", "critical"),
    (re.compile(r"ValueError:", re.I), "Value Error", "critical"),
    (re.compile(r"RuntimeError:", re.I), "Runtime Error", "critical"),
    # Rust
    (re.compile(r"error\[E\d+\]:", re.I), "Rust Compiler Error", "critical"),
    (re.compile(r"panicked at", re.I), "Rust Panic", "critical"),
    (re.compile(r"BUILD FAILED", re.I), "Build Failure", "critical"),
    # Shell / OS
    (re.compile(r"Permission denied", re.I), "Permission Error", "critical"),
    (re.compile(r"No such file or directory", re.I), "File Not Found", "critical"),
    (re.compile(r"command not found", re.I), "Command Not Found", "critical"),
    (re.compile(r"segmentation fault", re.I), "Segfault", "critical"),
    (re.compile(r"SIGSEGV", re.I), "Segfault Signal", "critical"),
    # HTTP / network
    (re.compile(r"[45]\d{2} (Server Error|Internal Server Error|Bad Request|Not Found)", re.I), "HTTP Error", "warning"),
    (re.compile(r"ECONNREFUSED|ERR_CONNECTION_REFUSED", re.I), "Connection Refused", "warning"),
    # Dev tooling
    (re.compile(r"FAILED|FAILURE", re.I), "Build/Test Failure", "critical"),
    (re.compile(r"npm ERR!", re.I), "NPM Error", "critical"),
    (re.compile(r"cargo error", re.I), "Cargo Error", "critical"),
    (re.compile(r"error: failed to", re.I), "Build Error", "critical"),
    # Misc critical
    (re.compile(r"exited with code [1-9]\d*", re.I), "Non-zero Exit", "warning"),
    (re.compile(r"OOMKilled|CUDA out of memory", re.I), "Memory Error", "critical"),
    (re.compile(r"fatal:", re.I), "Git Fatal", "critical"),
]

# Errors to IGNORE (too noisy, not actionable)
IGNORE_PATTERNS = [
    re.compile(r"notified:\s*true", re.I),
    re.compile(r"HEARTBEAT_OK", re.I),
    re.compile(r"^\s*$"),  # blank lines
]

# ── Config ───────────────────────────────────────────────────────────────────
COOLDOWN = 60      # seconds between same-error alerts
INTERVAL = 10      # seconds between captures
BURST_FILE = SCRIPT_DIR / "burst_trigger.txt"
LAST_ERROR_FILE = SCRIPT_DIR / "last_error_hash.txt"


def is_recent_error(error_hash: str) -> bool:
    """Return True if this error hash was seen in the last COOLDOWN seconds."""
    try:
        if not LAST_ERROR_FILE.exists():
            return False
        stored_hash, stored_time = LAST_ERROR_FILE.read_text().strip().split("|")
        age = time.time() - float(stored_time)
        return stored_hash == error_hash and age < COOLDOWN
    except:
        return False


def mark_error(error_hash: str):
    """Store the hash and timestamp of the last seen error."""
    try:
        LAST_ERROR_FILE.write_text(f"{error_hash}|{time.time()}")
    except:
        pass


def capture_screen():
    """Capture current screen. Returns PIL Image or None."""
    if not MSS_AVAILABLE:
        log.error("mss not available")
        return None
    try:
        with mss.mss() as sct:
            monitors = sct.monitors
            img = sct.grab(monitors[0])
            pil_img = Image.frombytes("RGB", img.size, img.bgra, "raw", "BGRX")
            return pil_img
    except Exception as e:
        log.error(f"Capture failed: {e}")
        return None


def extract_text(pil_img) -> str:
    """Extract text from PIL image using pytesseract (CPU-only OCR)."""
    if not PYTESS_AVAILABLE:
        log.warning("pytesseract not available — skipping OCR")
        return ""

    try:
        # Try fast mode first (basic), fall back to slow if needed
        text = pytesseract.image_to_string(pil_img, config="--psm 6 -c tessedit_char_whitelist=0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_:./\\[](){}<>!?@#$%^&*|~`\"'+=,-;\n")
        return text
    except Exception as e:
        log.error(f"OCR failed: {e}")
        return ""


def scan_for_errors(text: str) -> list[dict]:
    """Scan extracted text for error patterns. Returns list of found errors."""
    errors = []
    for pattern, label, severity in ERROR_PATTERNS:
        # Skip if matches ignore patterns
        if any(ig.search(text) for ig in IGNORE_PATTERNS):
            continue
        match = pattern.search(text)
        if match:
            errors.append({
                "pattern_label": label,
                "severity": severity,
                "matched_text": match.group(0)[:300],
                "full_context": text[:2000],  # capture surrounding context
            })
    return errors


def do_burst_record(seconds: int = 5):
    """Do a focused burst recording session — no cooldown, no rate limiting."""
    log.info(f"Burst recording: {seconds}s starting now...")
    start = time.time()  
    while time.time() - start < seconds:
        img = capture_screen()
        if img:
            text = extract_text(img)
            if text.strip():
                errors = scan_for_errors(text)
                for err in errors:
                    err_hash = hashlib.md5(
                        (err["pattern_label"] + err["matched_text"][:100]).encode()
                    ).hexdigest()[:12]
                    log.warning(f"[BURST] Error detected: [{err['pattern_label']}] {err['matched_text'][:100]}")
                    entry_id = write_log_entry(err)
                    notify_telegram(f"[BURST ALERT] [{err['pattern_label']}] {err['matched_text'][:200]}", entry_id)
        time.sleep(2)  # capture every 2s during burst
    log.info(f"Burst recording done. Captured {(time.time() - start):.0f}s")


def write_log_entry(error: dict, screenshot_b64: str = ""):
    """Write a new error entry to the log file."""
    entry_id = hashlib.md5(
        (error.get("pattern_label", "") + error.get("matched_text", "")[:100]).encode()
    ).hexdigest()[:12]

    entry = {
        "id": entry_id,
        "timestamp": datetime.now().isoformat(),
        "error_type": error.get("pattern_label", "unknown"),
        "error_message": error.get("matched_text", ""),
        "severity": error.get("severity", "warning"),
        "context": error.get("full_context", "")[:500],
        "screenshot_b64": screenshot_b64,
        "notified": False,
    }

    # Append to main log
    try:
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception as e:
        log.error(f"Failed to write log: {e}")

    # Add to pending approvals
    try:
        pending = []
        if PENDING_FILE.exists():
            try:
                pending = json.loads(PENDING_FILE.read_text(encoding="utf-8"))
                if isinstance(pending, dict):
                    pending = list(pending.values())
            except:
                pending = []
        pending.append(entry)
        PENDING_FILE.write_text(json.dumps(pending, indent=2), encoding="utf-8")
    except Exception as e:
        log.error(f"Failed to update pending approvals: {e}")

    return entry_id


def run_loop():
    """Main capture → OCR → detect → log loop."""
    log.info("Starting CPU-only screen watcher")
    log.info(f"mss available: {MSS_AVAILABLE}, pytesseract available: {PYTESS_AVAILABLE}")

    if not MSS_AVAILABLE:
        log.error("mss not installed. Run: pip install mss")
        return

    if not PYTESS_AVAILABLE:
        log.warning("pytesseract not installed. OCR disabled — will use pattern matching on text-only sources.")
        log.warning("Run: pip install pytesseract && download tesseract binary")

    consecutive_failures = 0
    last_alert_hash = ""

    while True:
        try:
            img = capture_screen()
            if img is None:
                time.sleep(INTERVAL)
                continue

            # Optionally downscale for faster OCR
            img_small = img.resize((img.width // 2, img.height // 2), Image.LANCZOS)

            # OCR: extract text from screen
            text = extract_text(img_small)
            if not text.strip():
                time.sleep(INTERVAL)
                continue

            # Scan for errors
            found_errors = scan_for_errors(text)

            if found_errors:
                for err in found_errors:
                    err_hash = hashlib.md5(
                        (err["pattern_label"] + err["matched_text"][:100]).encode()
                    ).hexdigest()[:12]

                    # Cooldown check
                    if is_recent_error(err_hash):
                        continue

                    log.warning(f"Error detected: [{err['pattern_label']}] {err['matched_text'][:100]}")
                    entry_id = write_log_entry(err)
                    entry_id_for_tg = notify_telegram(f"[SCREEN ERROR] [{err['pattern_label']}] {err['matched_text'][:200]}", entry_id)
                    if not entry_id_for_tg:
                        # Telegram failed — still mark notified in log to avoid spam
                        mark_notified(entry_id)
                    mark_error(err_hash)
                    consecutive_failures = 0
            else:
                consecutive_failures = 0

            time.sleep(INTERVAL)

        except KeyboardInterrupt:
            log.info("Received interrupt — shutting down")
            break
        except Exception as e:
            log.error(f"Loop error: {e}")
            consecutive_failures += 1
            if consecutive_failures > 5:
                log.error("Too many consecutive failures — restarting")
                consecutive_failures = 0
            time.sleep(INTERVAL)

        # ── Burst mode trigger ──────────────────────────────────────────
        try:
            if BURST_FILE.exists():
                content = BURST_FILE.read_text().strip()
                if content:
                    seconds = int(content)
                    seconds = max(3, min(seconds, 30))  # clamp 3-30s
                    log.info(f"Burst trigger found: {seconds}s")
                    do_burst_record(seconds)
                    BURST_FILE.write_text("")
                else:
                    try: BURST_FILE.unlink()
                    except: pass
        except Exception as e:
            log.error(f"Burst check error: {e}")


if __name__ == "__main__":
    print("Marble Screen Watcher — CPU Edition")
    print("Press Ctrl+C to stop")
    run_loop()
