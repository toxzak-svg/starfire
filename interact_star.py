#!/usr/bin/env python3
"""Interact with Star binary via stdin/stdout"""
import subprocess
import sys
import time

messages = [
    "Hello Star, how are you today?",
    "What have you been thinking about?",
    "What are you curious about?",
    "What have you been researching?",
    "Tell me about yourself",
]

proc = subprocess.Popen(
    ["/home/zach/.openclaw/workspace/life/target/release/star", "--data-dir", "/home/zach/.openclaw/workspace/life/life", "chat"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    bufsize=1,  # line buffered
)

# Wait for initialization
time.sleep(3)

for msg in messages:
    print(f"SENDING: {msg}", file=sys.stderr)
    proc.stdin.write(msg + "\n")
    proc.stdin.flush()
    time.sleep(4)  # wait for response

proc.stdin.write("/quit\n")
proc.stdin.flush()
time.sleep(2)

proc.stdin.close()
time.sleep(2)
proc.wait(timeout=5)

print("=== FULL OUTPUT ===", file=sys.stderr)
