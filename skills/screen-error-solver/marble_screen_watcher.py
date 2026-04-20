"""
Marble Screen Watcher - Instant screenshot on Ctrl+Shift+M
Sends screenshots directly to Marble via Telegram.
"""
import time
import mss
import keyboard
import os
from pathlib import Path

# Config - adjust these
HOTKEY = "ctrl+shift+m"
SCREENSHOT_DIR = Path(os.environ.get("TEMP", "C:/Users/Zwmar/AppData/Local/Temp"))
MAX_KEEP = 5  # keep only last N screenshots to avoid filling disk

last_screenshot_path = None

def take_and_send_screenshot():
    global last_screenshot_path
    
    # Take screenshot
    with mss.mss() as sct:
        monitor = sct.monitors[1]  # primary monitor
        screenshot = sct.grab(monitor)
    
    # Save to temp file
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    filename = f"screen_{timestamp}.png"
    filepath = SCREENSHOT_DIR / filename
    mss.tools.to_png(screenshot.rgb, screenshot.size, output=str(filepath))
    last_screenshot_path = filepath
    
    # Clean up old screenshots
    screenshots = sorted(SCREENSHOT_DIR.glob("screen_*.png"))
    while len(screenshots) > MAX_KEEP:
        old = screenshots.pop(0)
        try:
            old.unlink()
        except Exception:
            pass
    
    return filepath

def main():
    print(f"[Marble Screen Watcher] Ready. Press {HOTKEY.upper()} to capture screen.")
    print("Screenshot will be sent directly to Marble on Telegram.")
    print("Press Ctrl+C to stop.")
    
    while True:
        try:
            keyboard.wait(HOTKEY)
            filepath = take_and_send_screenshot()
            print(f"[Captured] {filepath.name}")
        except KeyboardInterrupt:
            print("\n[Stopped]")
            break
        except Exception as e:
            print(f"[Error] {e}")

if __name__ == "__main__":
    main()
