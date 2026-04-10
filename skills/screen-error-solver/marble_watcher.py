"""Marble's Screen Watcher -- background service edition.
Runs as a persistent background process, logs errors to marble_screen_errors.log
for Marble to pick up on heartbeat.
"""
import sys, os, base64, io, json, time, logging, hashlib, threading
from datetime import datetime

# Block import errors from crashing the whole process
try:
    import mss
    from PIL import Image
    MSS_AVAILABLE = True
except ImportError:
    MSS_AVAILABLE = False

try:
    import requests
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False

BLACKBOX_API_KEY = os.environ.get("BLACKBOX_API_KEY", "sk-z3eX9Vn4X5i-BLstbx5I2A")
LOG_FILE = os.path.join(os.path.dirname(__file__), "marble_screen_errors.log")
ERROR_LOG = os.path.join(os.path.dirname(__file__), "error_history.jsonl")

TELEGRAM_BOT_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q")
TELEGRAM_CHAT_ID = os.environ.get("TELEGRAM_CHAT_ID", "8371302588")  # Zach's Telegram ID

logging.basicConfig(
    level=logging.INFO,
    format="[MARBLE WATCHER] %(asctime)s %(message)s",
    handlers=[
        logging.FileHandler(os.path.join(os.path.dirname(__file__), "watcher.log")),
        logging.StreamHandler(sys.stdout),
    ],
)
log = logging.getLogger("marble_watcher")

ERROR_PATTERNS = [
    (__import__("re").compile(r"Traceback \(most recent call last\)"), "python_traceback", "critical"),
    ( __import__("re").compile(r"(\w+Error): (.+)"), "python_error", "critical"),
    ( __import__("re").compile(r"ModuleNotFoundError: No module named '(.+)'"), "import_error", "critical"),
    ( __import__("re").compile(r"error\[E(\d+)\]: (.+)"), "rust_error", "critical"),
    ( __import__("re").compile(r"panicked at (.+)"), "rust_panic", "critical"),
    ( __import__("re").compile(r"BUILD FAILED"), "build_failure", "critical"),
    ( __import__("re").compile(r"FAILED|FAILURE"), "generic_failure", "critical"),
    ( __import__("re").compile(r"npm ERR!"), "npm_error", "critical"),
    ( __import__("re").compile(r"cargo error"), "cargo_error", "critical"),
    ( __import__("re").compile(r"command not found"), "command_not_found", "critical"),
    ( __import__("re").compile(r"No such file or directory"), "file_not_found", "critical"),
    ( __import__("re").compile(r"Permission denied"), "permission_denied", "critical"),
    ( __import__("re").compile(r"segmentation fault"), "segfault", "critical"),
    ( __import__("re").compile(r"(5\d{2}) ([\w\s]+)"), "http_server_error", "critical"),
    ( __import__("re").compile(r"ECONNREFUSED|ERR_CONNECTION_REFUSED"), "connection_refused", "critical"),
    ( __import__("re").compile(r"exited with code (\d+)"), "docker_exit", "critical"),
    ( __import__("re").compile(r"OOMKilled"), "oom_killed", "critical"),
    ( __import__("re").compile(r"CUDA out of memory"), "cuda_oom", "critical"),
    ( __import__("re").compile(r"fatal: (.+)"), "git_fatal", "critical"),
]

COOLDOWN = 60  # seconds between same-error alerts
INTERVAL = 4  # seconds between captures


def capture_screen():
    """Capture current screen, return (base64_png, pil_image) or (None, None)."""
    if not MSS_AVAILABLE:
        return None, None
    try:
        with mss.mss() as sct:
            monitors = sct.monitors
            img = sct.grab(monitors[0])
            pil_img = Image.frombytes("RGB", img.size, img.bgra, "raw", "BGRX")
            # Downscale to 960x540 for Blackbox
            pil_img = pil_img.resize((960, 540), Image.LANCZOS)
            buffer = io.BytesIO()
            pil_img.save(buffer, format="PNG", optimize=True)
            b64 = base64.b64encode(buffer.getvalue()).decode("utf-8")
            return b64, pil_img
    except Exception as e:
        log.error(f"Capture failed: {e}")
        return None, None


def analyze_with_blackbox(b64_image: str) -> dict:
    """Send screenshot to Blackbox vision. Returns dict with has_error and details."""
    if not REQUESTS_AVAILABLE:
        return {"has_error": False}

    prompt = (
        'Analyze this screenshot of a developer\'s screen.\n\n'
        'CRITICAL INSTRUCTIONS:\n'
        '1. IGNORE all Telegram chat windows — do NOT report the Marble watcher\'s own messages or Telegram UI as errors.\n'
        '2. IGNORE all notification toasts/popups.\n'
        '3. Only report ACTUAL errors: terminal crashes, build failures, test failures, exception traces, IDE errors (red squiggles), crash dialogs, or permission errors.\n'
        '4. IMPORTANT: Capture the FULL error message text — do not truncate. If an error is visible, transcribe the COMPLETE message including all file paths, line numbers, and error codes.\n'
        '5. If you see the Marble Watcher\'s own Telegram messages about errors, ignore them — these are the watcher\'s own notifications, not real errors.\n\n'
        'Respond ONLY with JSON:\n'
        '{"has_error": true/false, "error_type": "...", "error_message": "COMPLETE error message with no truncation", '
        '"file_location": "file:line if visible", "language": "...", "severity": "critical|warning|info", '
        '"context_summary": "..."}\n\n'
        'If no real error visible: {"has_error": false}'
    )

    try:
        resp = requests.post(
            "https://api.blackbox.ai/v1/messages",
            headers={
                "Authorization": f"Bearer {BLACKBOX_API_KEY}",
                "anthropic-version": "2023-06-01",
            },
            json={
                "model": "blackboxai/anthropic/claude-sonnet-4.6",
                "messages": [{
                    "role": "user",
                    "content": [
                        {"type": "text", "text": prompt},
                        {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": b64_image}},
                    ],
                }],
                "max_tokens": 600,
            },
            timeout=45,
        )
        resp.raise_for_status()
        data = resp.json()
        content = data.get("content", [])
        if isinstance(content, list):
            for block in content:
                if block.get("type") == "text":
                    text = block.get("text", "")
                    import re
                    text = re.sub(r"```json\s*", "", text)
                    text = re.sub(r"```\s*", "", text)
                    return json.loads(text)
        return {"has_error": False}
    except Exception as e:
        log.error(f"Blackbox analysis failed: {e}")
        return {"has_error": False}


def generate_fix(error_msg: str, error_type: str, context: str = "") -> str:
    """Ask Blackbox for a fix."""
    if not REQUESTS_AVAILABLE:
        return "Install pytesseract for OCR-based detection, or check the error manually."

    prompt = (
        f"A developer hit this error:\n"
        f"Type: {error_type}\n"
        f"Message: {error_msg}\n"
        f"Context: {context[:500]}\n\n"
        f"Give a ONE-sentence cause and the exact fix."
    )

    try:
        resp = requests.post(
            "https://api.blackbox.ai/v1/messages",
            headers={
                "Authorization": f"Bearer {BLACKBOX_API_KEY}",
                "anthropic-version": "2023-06-01",
            },
            json={
                "model": "blackboxai/anthropic/claude-sonnet-4.6",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 300,
            },
            timeout=30,
        )
        resp.raise_for_status()
        data = resp.json()
        for block in data.get("content", []):
            if block.get("type") == "text":
                return block.get("text", "")[:500]
        return "Could not generate fix."
    except Exception as e:
        return f"Fix generation failed: {e}"


def send_telegram_alert(error: dict, fix: str):
    """Send error alert directly to Zach via Telegram DM."""
def send_approval_request(error: dict, fix: str, screenshot_b64: str = ""):
    """Send approval request to Zach via Telegram with inline Fix/Ignore buttons.
    Writes to pending_approvals.json so Marble can process on heartbeat.
    """
    import urllib.request
    emoji = "🔴" if error.get("severity") == "critical" else "🟡"
    err_msg = error.get('error_message', '')[:800]
    file_loc = error.get('file_location', 'not visible')
    err_type = error.get('error_type', 'unknown')
    entry_id = hashlib.md5(f"{error.get('error_type','')}:{err_msg[:200]}".encode()).hexdigest()[:12]

    # Store pending approval so Marble can act on it
    APPROVAL_FILE = os.path.join(os.path.dirname(__file__), "pending_approvals.json")
    try:
        approvals = []
        if os.path.exists(APPROVAL_FILE):
            with open(APPROVAL_FILE) as f:
                approvals = json.load(f)
        approvals.append({
            "id": entry_id,
            "timestamp": datetime.now().isoformat(),
            "error": error,
            "fix": fix,
            "screenshot_b64": screenshot_b64[:200] if screenshot_b64 else "",
            "status": "pending",  # pending | approved | ignored
        })
        with open(APPROVAL_FILE, "w") as f:
            json.dump(approvals, f, indent=2)
    except Exception as e:
        log.error(f"Failed to write approval file: {e}")

    # Send Telegram message with inline buttons
    try:
        msg = (
            f"{emoji} <b>Error Found</b> — want me to fix it?\n\n"
            f"<b>Type:</b> {err_type}\n"
            f"<b>File:</b> {file_loc}\n"
            f"<b>Msg:</b> {err_msg}\n\n"
            f"<b>Fix:</b> {fix[:500]}\n\n"
            f"React with ✅ to approve or ❌ to ignore."
        )
        payload = json.dumps({
            "chat_id": TELEGRAM_CHAT_ID,
            "text": msg,
            "parse_mode": "HTML",
        }).encode()
        req = urllib.request.Request(
            f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage",
            data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            if result.get("ok"):
                log.info(f"Approval request sent for {entry_id}")
            else:
                log.warning(f"Telegram approval send failed: {result}")
    except Exception as e:
        log.error(f"Telegram approval request failed: {e}")
        payload = json.dumps({
            "chat_id": TELEGRAM_CHAT_ID,
            "text": msg,
            "parse_mode": "HTML",
            "disable_notification": False,
        }).encode()
        req = urllib.request.Request(
            f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage",
            data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            if result.get("ok"):
                log.info(f"Telegram alert sent to Zach")
            else:
                log.warning(f"Telegram send failed: {result}")
    except Exception as e:
        log.error(f"Telegram alert failed: {e}")


def run():
    log.info("STARTING Marble Screen Watcher")
    log.info(f"Interval: {INTERVAL}s | Cooldown: {COOLDOWN}s")

    if not MSS_AVAILABLE:
        log.error("mss not installed: pip install mss")
        return
    if not REQUESTS_AVAILABLE:
        log.error("requests not installed")
        return

    recent_errors = {}  # hash -> timestamp
    captures = 0

    log.info("Screen watcher running. Marble will ping you when errors appear.")

    while True:
        try:
            b64, _ = capture_screen()
            if b64:
                captures += 1
                result = analyze_with_blackbox(b64)

                if result.get("has_error"):
                    err_type = result.get("error_type", "unknown")
                    err_msg = result.get("error_message", "")
                    # Use full message for hash to avoid dupes from truncation
                    key = hashlib.md5(f"{err_type}:{err_msg[:200]}".encode()).hexdigest()[:12]
                    now = time.time()

                    # Smart cooldown: only re-alert if error was absent then came back
                    # OR if it's been COOLDOWN since first_seen AND since last alert
                    if key not in recent_errors:
                        # New error — record and ask approval
                        recent_errors[key] = {"first": now, "last": now, "alerted_at": 0}
                        fix = generate_fix(err_msg, err_type, result.get("context_summary", ""))
                        send_approval_request(result, fix, b64)
                    else:
                        entry = recent_errors[key]
                        entry["last"] = now
                        time_since_first = now - entry["first"]
                        time_since_alert = now - entry["alerted_at"]
                        # Re-alert only if: error persisted for >2 min AND >2 min since last alert
                        if time_since_first > 120 and time_since_alert > COOLDOWN:
                            entry["alerted_at"] = now
                            fix = generate_fix(err_msg, err_type, result.get("context_summary", ""))
                            send_approval_request(result, fix, b64)
                        else:
                            log.debug(f"Cooldown active for {err_type[:40]} (first={time_since_first:.0f}s ago, alert={time_since_alert:.0f}s ago)")
                else:
                    # No error detected — clear errors that are "resolved" (not currently on screen)
                    # This way, a NEW appearance of the same error is treated as genuinely new
                    resolved_keys = [k for k, v in recent_errors.items() if now - v["last"] > 30]
                    for k in resolved_keys:
                        del recent_errors[k]
                        log.debug(f"Error {k[:12]} resolved (absent for 30s), tracking reset")

                if captures % 30 == 0:
                    log.info(f"Still watching... ({captures} frames analyzed)")

            time.sleep(INTERVAL)

        except KeyboardInterrupt:
            log.info(f"STOPPED. Analyzed {captures} frames.")
            break
        except Exception as e:
            log.error(f"Loop error: {e}")
            time.sleep(5)


if __name__ == "__main__":
    run()
