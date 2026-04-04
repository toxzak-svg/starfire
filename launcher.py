#!/usr/bin/env python3
"""
Starfire Launcher
=================
Simple launcher for Starfire + QuaNot.

Usage:
    python launcher.py status  - Check system status
    python launcher.py chat    - Start chat
    python launcher.py quanot - Run QuaNot demo
"""

import sys
import os
import subprocess
import argparse
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent
QUANOT_DIR = PROJECT_ROOT / "quanot"

# Find Starfire binary (try multiple locations)
STARFIRE_BIN = PROJECT_ROOT / "target" / "release" / "star.exe"
if not STARFIRE_BIN.exists():
    STARFIRE_BIN = PROJECT_ROOT / "target" / "release" / "star"
if not STARFIRE_BIN.exists():
    STARFIRE_BIN = PROJECT_ROOT / "target" / "debug" / "star.exe"
if not STARFIRE_BIN.exists():
    STARFIRE_BIN = PROJECT_ROOT / "target" / "debug" / "star"


def check_quanot():
    """Check QuaNot setup."""
    venv_python = QUANOT_DIR / ".venv" / "Scripts" / "python.exe"
    if not venv_python.exists():
        venv_python = QUANOT_DIR / ".venv" / "bin" / "python"
    return venv_python.exists(), str(venv_python)


def status():
    """Show system status."""
    print("\n" + "=" * 50)
    print("Starfire Status")
    print("=" * 50)
    
    # Starfire
    if STARFIRE_BIN.exists():
        print("[OK] Starfire: {}".format(STARFIRE_BIN))
    else:
        print("[X] Starfire: Not found")
        print("  Run: cargo build --release")
    
    # QuaNot
    ready, python = check_quanot()
    if ready:
        print("[OK] QuaNot: Ready ({})".format(python))
    else:
        print("[X] QuaNot: Not setup")
        print("  Run: ./build.sh quanot")
    
    print("=" * 50)


def chat():
    """Start Starfire chat."""
    if not STARFIRE_BIN.exists():
        print("Starfire not found. Build with: cargo build --release")
        return
    
    subprocess.run([str(STARFIRE_BIN), "chat"])


def quanot():
    """Run QuaNot demo."""
    ready, python = check_quanot()
    if not ready:
        print("QuaNot not setup. Run: ./build.sh quanot")
        return
    
    subprocess.run([python, str(QUANOT_DIR / "src" / "main.py")])


def main():
    parser = argparse.ArgumentParser(description="Starfire Launcher")
    parser.add_argument("command", choices=["status", "chat", "quanot"])
    args = parser.parse_args()
    
    if args.command == "status":
        status()
    elif args.command == "chat":
        chat()
    elif args.command == "quanot":
        quanot()


if __name__ == "__main__":
    main()
