#!/usr/bin/env python3
"""Interact with Star binary via stdin/stdout - simplified"""
import subprocess
import sys
import time
import os
import select
import signal

messages = [
    "hello",
    "what is intelligence",
    "what do you think about consciousness",
    "tell me about yourself",
    "tell me about philosophy",
]

proc = subprocess.Popen(
    ["/home/zach/.openclaw/workspace/life/target/release/star", "--data-dir", "/home/zach/.openclaw/workspace/life/life", "chat"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    bufsize=0,
)

# Set stdout to non-blocking
import fcntl
fl = fcntl.fcntl(proc.stdout.fileno(), fcntl.F_GETFL)
fcntl.fcntl(proc.stdout.fileno(), fcntl.F_SETFL, fl | os.O_NONBLOCK)

print("Waiting for init...", file=sys.stderr, flush=True)
time.sleep(5)

def read_output(timeout=5.0):
    result = ""
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([proc.stdout], [], [], 0.2)
        if ready:
            try:
                chunk = os.read(proc.stdout.fileno(), 4096)
                if chunk:
                    text = chunk.decode('utf-8', errors='replace')
                    result += text
            except OSError:
                break
        elif result:
            break
    return result

for msg in messages:
    print(f"\n=== SENDING: '{msg}' ===", file=sys.stderr, flush=True)
    proc.stdin.write((msg + "\n").encode())
    proc.stdin.flush()
    
    response = read_output(8.0)
    if response:
        print(f"--- RESPONSE ---", file=sys.stderr, flush=True)
        print(response, file=sys.stderr, flush=True)
        print(f"--- END ---", file=sys.stderr, flush=True)
    else:
        print(f"(no response)", file=sys.stderr, flush=True)

print("\n=== QUITTING ===", file=sys.stderr, flush=True)
proc.stdin.write(b"/quit\n")
proc.stdin.flush()
time.sleep(2)

# Drain remaining output
remaining = read_output(3.0)
if remaining:
    print(f"(remaining: {remaining})", file=sys.stderr)

proc.stdin.close()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
    proc.wait()
print("Done.", file=sys.stderr)
