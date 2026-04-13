"""
Marble's Screen Watcher -- Hotkey-Triggered Assist (Burst Mode)
==============================================================
Zach hits Ctrl+Shift+M -> captures BURST of screens -> sends sequence to Marble on Telegram
Marble analyzes and replies on Telegram with help.

No API vision calls needed during capture -- just sends screenshots to Telegram.
Marble (me) sees it and responds. Simple and reliable.

Burst: 5 shots @ 0.3s interval (~1.5s total capture window)
"""
import sys, os, base64, io, json, time, logging, hashlib, re, argparse
from datetime import datetime
from pathlib import Path

# ── Dependencies ──────────────────────────────────────────────────────────────
import mss
from PIL import Image

try:
    import pytesseract
    pytesseract.pytesseract.tesseract_cmd = r"C:\Program Files\Tesseract-OCR\tesseract.exe"
    PYTESS_AVAILABLE = True
except ImportError:
    PYTESS_AVAILABLE = False

try:
    from pynput import keyboard
    PINPUT_AVAILABLE = True
except ImportError:
    PINPUT_AVAILABLE = False

# ── Telegram config ──────────────────────────────────────────────────────────
TELEGRAM_BOT_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q")
TELEGRAM_CHAT_ID = os.environ.get("TELEGRAM_CHAT_ID", "8371302588")  # Ton0Fun

# ── Burst config ─────────────────────────────────────────────────────────────
BURST_COUNT = 5          # number of shots in a burst
BURST_INTERVAL = 0.3    # seconds between shots

# ── Paths ───────────────────────────────────────────────────────────────────
SCRIPT_DIR = Path(__file__).parent.resolve()
LOG_FILE = SCRIPT_DIR / "marble_screen_errors.log"
WATCHER_LOG = SCRIPT_DIR / "watcher.log"

# ── Logging ─────────────────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO,
    format="[MARBLE HOTKEY] %(asctime)s %(message)s",
    handlers=[
        logging.FileHandler(str(WATCHER_LOG)),
        logging.StreamHandler(sys.stdout),
    ],
)
log = logging.getLogger("marble_watcher_hotkey")

# ── Helpers ─────────────────────────────────────────────────────────────────
def capture_screen(region=None):
    """Capture screen (or region). Returns PIL Image."""
    try:
        with mss.mss() as sct:
            monitors = sct.monitors
            if region:
                x, y, w, h = region
                img = sct.grab((x, y, x + w, y + h))
            else:
                img = sct.grab(monitors[0])
            pil_img = Image.frombytes("RGB", img.size, img.bgra, "raw", "BGRX")
            return pil_img
    except Exception as e:
        log.error(f"Capture failed: {e}")
        return None


def extract_text(pil_img) -> str:
    """OCR extract visible text from screen."""
    if not PYTESS_AVAILABLE:
        return ""
    try:
        return pytesseract.image_to_string(
            pil_img,
            config="--psm 6 -c tessedit_char_whitelist=0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz _:./\\[](){}<>!?@#$%^&*|~`\"'+=,-;\n"
        )
    except Exception as e:
        log.error(f"OCR failed: {e}")
        return ""


def img_to_b64(pil_img) -> str:
    """Convert PIL image to base64 JPEG."""
    buf = io.BytesIO()
    pil_img.save(buf, format="JPEG", quality=85)
    return base64.b64encode(buf.getvalue()).decode()


def notify_telegram(msg: str, img_b64: str = "", photo_index: int = 0, photo_total: int = 1) -> bool:
    """Send text + optional screenshot to Telegram."""
    try:
        import urllib.request, urllib.parse

        if img_b64:
            # Send photo with caption
            url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendPhoto"
            boundary = "MarbleScreenWatch"
            # Caption: include burst position if multi-shot
            if photo_total > 1:
                caption_prefix = f"[{photo_index+1}/{photo_total}] "
            else:
                caption_prefix = ""
            full_caption = caption_prefix + msg[:1024]

            body = (
                f"--{boundary}\r\n"
                f"Content-Disposition: form-data; name=\"chat_id\"\r\n\r\n{TELEGRAM_CHAT_ID}\r\n"
                f"--{boundary}\r\n"
                f"Content-Disposition: form-data; name=\"caption\"\r\n\r\n{full_caption}\r\n"
                f"--{boundary}\r\n"
                f"Content-Disposition: form-data; name=\"photo\"; filename=\"screen.jpg\"\r\n"
                f"Content-Type: image/jpeg\r\n\r\n"
            )
            body = body.encode() + base64.b64decode(img_b64) + f"\r\n--{boundary}--\r\n".encode()
            headers = {
                "Content-Type": f"multipart/form-data; boundary={boundary}",
            }
            req = urllib.request.Request(url, data=body, headers=headers, method="POST")
        else:
            url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
            data = urllib.parse.urlencode({
                "chat_id": TELEGRAM_CHAT_ID,
                "text": msg,
                "parse_mode": "HTML",
            }).encode()
            req = urllib.request.Request(url, data=data, method="POST")

        with urllib.request.urlopen(req, timeout=20) as resp:
            return resp.status == 200
    except Exception as e:
        log.error(f"Telegram notify failed: {e}")
        return False


def do_assist_burst():
    """Capture a burst of screens and send them all to Telegram."""
    log.info(f"=== Burst assist triggered ({BURST_COUNT} shots @ {BURST_INTERVAL}s interval) ===")
    print(f"\n[MARBLE] Capturing burst ({BURST_COUNT} shots @ {BURST_INTERVAL}s interval)...")

    shots = []
    for i in range(BURST_COUNT):
        img = capture_screen()
        if img is None:
            log.warning(f"Burst shot {i+1} failed, continuing...")
            continue

        text = extract_text(img)
        text_preview = text[:200].replace("\n", " ") if text else "(no text detected)"
        shots.append((img, text, text_preview, i))
        log.info(f"  Shot {i+1}/{BURST_COUNT}: {img.size} | OCR: {len(text)} chars")

        if i < BURST_COUNT - 1:
            time.sleep(BURST_INTERVAL)

    if not shots:
        notify_telegram("Marble: couldn't capture your screen -- mss error")
        return

    # Send first shot with full context
    img0, text0, preview0, idx0 = shots[0]
    img0_b64 = img_to_b64(img0)
    ocr_summary = f"OCR: {text0[:500].replace(chr(10), ' ')}" if text0 else "(no text detected)"

    if len(shots) == 1:
        msg = f"<b>[Screen Assist]</b>\nI see: {preview0}\n\nWhat do you need?"
        sent = notify_telegram(msg, img0_b64)
    else:
        msg = (
            f"<b>[Screen Assist — {len(shots)} shots]</b>\n"
            f"I see (shot 1/{len(shots)}): {preview0}\n\n"
            f"Sequence coming... ({len(shots)} total shots)\n"
            f"{ocr_summary[:200]}"
        )
        sent = notify_telegram(msg, img0_b64, photo_index=0, photo_total=len(shots))

    if not sent:
        log.error("Failed to send first shot to Telegram")
        return

    print(f"[MARBLE] Shot 1/{len(shots)} sent to Telegram")

    # Send remaining shots
    for i in range(1, len(shots)):
        img_i, text_i, preview_i, idx_i = shots[i]
        img_i_b64 = img_to_b64(img_i)
        msg_i = f"[{i+1}/{len(shots)}] I see: {preview_i}"
        notify_telegram(msg_i, img_i_b64, photo_index=i, photo_total=len(shots))
        print(f"[MARBLE] Shot {i+1}/{len(shots)} sent to Telegram")
        time.sleep(0.2)  # small delay between sends to avoid rate limit

    print(f"[MARBLE] All {len(shots)} shots sent to Telegram -- I'll analyze and reply shortly!")
    log.info(f"Burst assist complete ({len(shots)} shots sent)")


# ── Alias for backward compat ──────────────────────────────────────────────────
def do_assist():
    """Single-shot assist (backward compat alias). Use burst mode by default."""
    do_assist_burst()


# ── Hotkey listener ────────────────────────────────────────────────────────────
def run_hotkey_listener():
    """Listen for Ctrl+Shift+M in background."""
    if not PINPUT_AVAILABLE:
        log.error("pynput not available")
        print("[MARBLE] ERROR: pynput not installed. Run: pip install pynput")
        return

    print("[MARBLE] Hotkey listener active -- press Ctrl+Shift+M to trigger assist (burst mode)")
    print(f"[MARBLE] Burst: {BURST_COUNT} shots @ {BURST_INTERVAL}s interval")
    print("[MARBLE] Screenshot sequence comes to me on Telegram -- I'll reply with analysis!")
    log.info(f"Hotkey listener started (Ctrl+Shift+M, burst={BURST_COUNT}x{BURST_INTERVAL}s)")

    from pynput import keyboard

    def on_activate():
        do_assist_burst()

    with keyboard.GlobalHotKeys({
        '<ctrl>+<shift>+m': on_activate,
    }) as h:
        h.join()


# ── Entry point ─────────────────────────────────────────────────────────────
if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--test", action="store_true", help="Test capture + OCR and exit")
    parser.add_argument("--count", type=int, default=BURST_COUNT, help="Number of burst shots")
    parser.add_argument("--interval", type=float, default=BURST_INTERVAL, help="Interval between shots")
    args = parser.parse_args()

    print("-" * 60)
    print("  Marble Screen Watcher -- Burst Mode")
    print("  Press Ctrl+Shift+M to capture + assist (5-shot burst)")
    print("-" * 60)

    if args.test:
        print("Testing capture...")
        img = capture_screen()
        if img:
            print(f"  [OK] Capture: {img.size}")
            text = extract_text(img)
            print(f"  [OK] OCR: {len(text)} chars")
            if text:
                print(f"  Sample: {text[:200]}")
        else:
            print("  [X] Capture failed")
        sys.exit(0)

    run_hotkey_listener()
