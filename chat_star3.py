#!/usr/bin/env python3
"""Interact with Star using pipe approach with readline"""
import subprocess
import sys
import time
import os

def run():
    star_proc = subprocess.Popen(
        ["/home/zach/.openclaw/workspace/life/target/release/star",
         "--data-dir", "/home/zach/.openclaw/workspace/life", "chat"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd="/home/zach/.openclaw/workspace/life"
    )
    
    # Wait for init
    time.sleep(3)
    
    # Read init output
    init = star_proc.stderr.read(4000)
    # print("INIT:", init[:500])
    
    messages = [
        "Hello Star, how are you today?",
        "What have you been thinking about lately?",
        "What's been on your mind?",
        "What are you curious about right now?",
        "What have you been researching?",
        "Tell me about yourself",
    ]
    
    full_output = []
    for msg in messages:
        print(f"\nME: {msg}")
        # Send message
        star_proc.stdin.write((msg + "\n").encode())
        star_proc.stdin.flush()
        time.sleep(2)
        
        # Try to read stderr for the response (logging goes there)
        # Actually the response goes to stdout
        # Let's read stdout
        import select
        outputs = []
        while True:
            ready, _, _ = select.select([star_proc.stdout], [], [], 0.5)
            if ready:
                data = star_proc.stdout.read(4096)
                if data:
                    outputs.append(data)
                    print(f"  [stdout chunk {len(data)} bytes]")
                else:
                    break
            else:
                break
                
        combined = b"".join(outputs)
        text = combined.decode('utf-8', errors='replace')
        # Filter ANSI
        import re
        text = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', text)
        print(f"STAR: {text[:500]}")
        full_output.append(text)
    
    # Quit
    star_proc.stdin.write(b"/quit\n")
    star_proc.stdin.flush()
    time.sleep(1)
    star_proc.stdin.close()
    star_proc.wait()
    return full_output

if __name__ == "__main__":
    run()
