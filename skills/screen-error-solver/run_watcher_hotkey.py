"""
Marble Screen Watcher -- Hotkey Edition
======================================
Press Ctrl+Shift+M anytime to share your screen with Marble.
She'll analyze it and reply to you on Telegram.

Usage:
  python run_watcher_hotkey.py              # hotkey mode (silent until triggered)
  python run_watcher_hotkey.py --test     # test capture + OCR

Exit: Ctrl+C
"""
import sys, subprocess
from pathlib import Path

import os
os.environ["TELEGRAM_CHAT_ID"] = "8371302588"

script = Path(__file__).parent / "marble_watcher_hotkey.py"
sys.exit(subprocess.call([sys.executable, str(script)] + sys.argv[1:]))
