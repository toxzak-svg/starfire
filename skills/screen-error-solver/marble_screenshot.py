"""Marble's Screen Screenshot Tool -- Ctrl+Shift+M triggered single capture.
Idle until hotkey is pressed, then capture + analyze + alert.
"""
import sys, os, base64, io, json, time, hashlib, logging
from datetime import datetime

try:
    import mss
    from PIL import Image
    MSS_AVAILABLE = True
except ImportError:
    MSS_AVAILABLE = False

try:
    from pynput import keyboard
    PYNPUT_AVAILABLE = True
except ImportError:
    PYNPUT_AVAILABLE = False

try:
    import requests
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False

BLACKBOX_API_KEY = os.environ.get("BLACKBOX_API_KEY", "sk-z3eX9Vn4X5i-BLstbx5I2A")
LOG_FILE = os.path.join(os.path.dirname(__file__), "watcher.log")

TELEGRAM_BOT_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q")
TELEGRAM_CHAT_ID = os.environ.get("TELEGRAM_CHAT_ID", "8371302588")

logging.basicConfig(
    level=logging.INFO,
    format="[MARBLE SNAP] %(asctime)s %(message)s",
    handlers=[
        logging.FileHandler(LOG_FILE),
        logging.StreamHandler(sys.stdout),
    ],
)
log = logging.getLogger("marble_screenshot")


def capture_screen():
    """Capture current screen, return (base64_png, pil_image, capture_time)."""
    if not MSS_AVAILABLE:
        return None, None, None
    try:
        with mss.mss() as sct:
            monitors = sct.monitors
            img = sct.grab(monitors[0])
            pil_img = Image.frombytes("RGB", img.size, img.bgra, "raw", "BGRX")
            pil_img = pil_img.resize((960, 540), Image.LANCZOS)
            buffer = io.BytesIO()
            pil_img.save(buffer, format="PNG", optimize=True)
            b64 = base64.b64encode(buffer.getvalue()).decode("utf-8")
            return b64, pil_img, datetime.now().isoformat()
    except Exception as e:
        log.error(f"Capture failed: {e}")
        return None, None, None


def analyze_with_blackbox(b64_image: str) -> dict:
    """Send screenshot to Blackbox vision."""
    if not REQUESTS_AVAILABLE:
        return {"has_error": False}

    prompt = (
        'Analyze this screenshot of a developer\'s screen.\n\n'
        'CRITICAL INSTRUCTIONS:\n'
        '1. IGNORE all Telegram chat windows and Marble watcher messages.\n'
        '2. IGNORE all notification toasts, system tray icons, and desktop notifications.\n'
        '3. Only report ACTUAL errors: terminal crashes, build failures, test failures, '
        'exception traces, IDE errors (red squiggles), crash dialogs, or permission errors.\n'
        '4. Capture the FULL error message — do not truncate. Include all file paths, '
        'line numbers, and error codes.\n\n'
        'Respond ONLY with JSON:\n'
        '{"has_error": true/false, "error_type": "...", "error_message": "COMPLETE message", '
        '"file_location": "file:line if visible", "language": "...", "severity": "critical|warning|info", '
        '"context_summary": "what they appear to be working on"}\n\n'
        'If no error visible: {"has_error": false}'
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
        for block in data.get("content", []):
            if block.get("type") == "text":
                import re
                text = re.sub(r"```json\s*", "", block.get("text", ""))
                text = re.sub(r"```\s*", "", text)
                return json.loads(text)
        return {"has_error": False}
    except Exception as e:
        log.error(f"Blackbox analysis failed: {e}")
        return {"has_error": False}


def generate_fix(error_msg: str, error_type: str, context: str = "") -> str:
    """Ask Blackbox for a fix."""
    if not REQUESTS_AVAILABLE:
        return "Install requests library to enable fix suggestions."

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
        for block in resp.json().get("content", []):
            if block.get("type") == "text":
                return block.get("text", "")[:500]
        return "Could not generate fix."
    except Exception as e:
        return f"Fix generation failed: {e}"


def send_telegram_alert(error: dict, fix: str, screenshot_b64: str = ""):
    """Send error alert + screenshot to Zach via Telegram."""
    import urllib.request

    emoji = "🔴" if error.get("severity") == "critical" else "🟡"
    err_msg = error.get("error_message", "")[:600]
    file_loc = error.get("file_location", "not visible")
    err_type = error.get("error_type", "unknown")

    msg = (
        f"{emoji} <b>Error Detected</b>\n\n"
        f"<b>Type:</b> {err_type}\n"
        f"<b>File:</b> {file_loc}\n"
        f"<b>Msg:</b> {err_msg}\n\n"
        f"<b>Fix:</b> {fix[:400]}"
    )

    try:
        # Send text first
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
                log.info(f"Alert sent to Zach")

        # Send screenshot as photo if available
        if screenshot_b64:
            photo_payload = json.dumps({
                "chat_id": TELEGRAM_CHAT_ID,
                "photo": f"data:image/png;base64,{screenshot_b64}",
                "caption": f"Screenshot at {datetime.now().strftime('%H:%M:%S')}",
            }).encode()
            req2 = urllib.request.Request(
                f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendPhoto",
                data=photo_payload,
                headers={"Content-Type": "application/json"},
            )
            try:
                with urllib.request.urlopen(req2, timeout=15) as resp2:
                    result2 = json.loads(resp2.read())
                    if result2.get("ok"):
                        log.info(f"Screenshot sent")
            except Exception as e:
                log.warning(f"Photo send failed (may be too large): {e}")

    except Exception as e:
        log.error(f"Telegram alert failed: {e}")


def on_hotkey():
    """Called when Ctrl+Shift+M is pressed. Capture + analyze + alert."""
    log.info("Hotkey triggered! Capturing screen...")
    b64, pil_img, capture_time = capture_screen()

    if not b64:
        log.error("Capture failed, nothing to send")
        return

    # Save local copy
    snap_dir = os.path.join(os.path.dirname(__file__), "snapshots")
    os.makedirs(snap_dir, exist_ok=True)
    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    local_path = os.path.join(snap_dir, f"snap_{ts}.png")
    pil_img.save(local_path)
    log.info(f"Saved: {local_path}")

    # Analyze
    result = analyze_with_blackbox(b64)

    if result.get("has_error"):
        err_type = result.get("error_type", "unknown")
        err_msg = result.get("error_message", "")
        fix = generate_fix(err_msg, err_type, result.get("context_summary", ""))
        log.info(f"Error detected: {err_type} — {err_msg[:80]}")
        send_telegram_alert(result, fix, b64)
    else:
        log.info("No error detected on screen")
        # Still let Zach know the capture worked
        try:
            import urllib.request
            msg = f"📸 Screen captured at {capture_time} — no errors detected"
            payload = json.dumps({
                "chat_id": TELEGRAM_CHAT_ID,
                "text": msg,
                "disable_notification": True,
            }).encode()
            req = urllib.request.Request(
                f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage",
                data=payload,
                headers={"Content-Type": "application/json"},
            )
            with urllib.request.urlopen(req, timeout=10):
                pass
        except Exception as e:
            log.warning(f"Notification send failed: {e}")


def run():
    log.info("=" * 50)
    log.info("MARBLE SCREEN SNAP")
    log.info("Ctrl+Shift+M  →  capture + analyze + alert")
    log.info("Ctrl+C         →  quit")
    log.info("=" * 50)

    if not MSS_AVAILABLE:
        log.error("mss not installed: pip install mss")
        return
    if not REQUESTS_AVAILABLE:
        log.error("requests not installed")
        return
    if not PYNPUT_AVAILABLE:
        log.error("pynput not installed: pip install pynput")
        log.error("Hotkey won't work without pynput. Screenshot-only mode.")
        # Fallback: single capture on import (for testing)
        print("[MARBLE] pynput missing — doing one capture and exiting.")
        on_hotkey()
        return

    def on_press(key):
        try:
            # Ctrl+Shift+M
            if (key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r) or \
               (hasattr(key, 'char') and key.char == 'm'):
                pass  # handled by on_activate
        except AttributeError:
            pass

    def on_release(key):
        pass

    # Register hotkey
    hotkey = keyboard.HotKey(
        keyboard.HotKey.parse("<ctrl>+<shift>+m"),
        on_hotkey
    )

    listener = keyboard.Listener(
        on_press=lambda key: (
            hotkey.press(key.char if hasattr(key, 'char') else None),
            on_press(key)
        ),
        on_release=on_release,
    )
    listener.start()

    log.info("Listening for Ctrl+Shift+M ...")
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        log.info("STOPPED.")
        listener.stop()


if __name__ == "__main__":
    run()
