"""Start/stop/status for Marble's hotkey screen watcher."""
import sys, subprocess, time
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PID_FILE = SCRIPT_DIR / "watcher_hotkey.pid"
LOG_FILE = SCRIPT_DIR / "watcher_hotkey.log"

def start_background():
    """Launch the hotkey watcher in a detached background process."""
    import os
    os.environ["TELEGRAM_CHAT_ID"] = "8371302588"
    script = SCRIPT_DIR / "marble_watcher_hotkey.py"
    with open(LOG_FILE, "w") as log:
        proc = subprocess.Popen(
            [sys.executable, str(script)],
            stdout=log, stderr=subprocess.STDOUT,
            cwd=str(SCRIPT_DIR), detached=True
        )
    PID_FILE.write_text(str(proc.pid))
    print(f"Marble watcher started (PID {proc.pid})")
    print(f"  Hotkey: Ctrl+Shift+M")
    print(f"  Log: {LOG_FILE}")

def stop():
    if PID_FILE.exists():
        pid = int(PID_FILE.read_text().strip())
        try:
            subprocess.call(["taskkill", "/PID", str(pid), "/F"])
            print(f"Killed PID {pid}")
        except:
            print(f"Could not kill PID {pid}")
        PID_FILE.unlink()
    else:
        print("No PID file found -- not running?")

def status():
    if PID_FILE.exists():
        pid = int(PID_FILE.read_text().strip())
        print(f"Watcher running as PID {pid}")
    else:
        print("Watcher not running (no PID file)")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python run_watcher_manager.py start|stop|status")
        sys.exit(1)
    cmd = sys.argv[1].lower()
    if cmd == "start":
        start_background()
    elif cmd == "stop":
        stop()
    elif cmd == "status":
        status()
    else:
        print(f"Unknown command: {cmd}")
