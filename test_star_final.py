#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY with proper timing"""
import os
import pty
import select
import time
import sys
import fcntl
import termios

def write_all(fd, data):
    while data:
        n = os.write(fd, data)
        data = data[n:]

def read_all(fd, timeout=10.0):
    """Read until EOF or timeout"""
    output = []
    fl = fcntl.fcntl(fd, fcntl.F_GETFL)
    fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.5)
        if ready:
            try:
                data = os.read(fd, 8192)
                if data:
                    output.append(data)
                else:
                    break
            except OSError:
                break
        else:
            # Nothing ready for 0.5s
            if output:
                break
    return b"".join(output)

def main():
    messages = [
        "hello star",
        "what is intelligence",
        "what do you think about consciousness",
        "tell me about yourself",
        "tell me about philosophy",
    ]

    master_fd, slave_fd = pty.openpty()
    
    pid = os.fork()
    
    if pid == 0:
        os.close(master_fd)
        os.setsid()
        fcntl.ioctl(slave_fd, termios.TIOCSCTTY, 0)
        os.dup2(slave_fd, 0)
        os.dup2(slave_fd, 1)
        os.dup2(slave_fd, 2)
        if slave_fd > 2:
            os.close(slave_fd)
        os.execv("/home/zach/.openclaw/workspace/life/life/target/release/star", 
                 ["/home/zach/.openclaw/workspace/life/life/target/release/star", 
                  "--data-dir", "/home/zach/.openclaw/workspace/life/life", "chat"])
        os._exit(1)
    
    os.close(slave_fd)
    
    # Read initialization output (up to 5 seconds)
    init_output = read_all(master_fd, timeout=5.0)
    print(f"[INIT OUTPUT - {len(init_output)} bytes]:", file=sys.stderr)
    print(init_output.decode('utf-8', errors='replace'), file=sys.stderr)
    
    responses = []
    for msg in messages:
        print(f"\n>>> SENDING: {msg}", file=sys.stderr)
        write_all(master_fd, (msg + "\n").encode())
        
        # Wait for Star's response
        response = read_all(master_fd, timeout=15.0)
        if response:
            text = response.decode('utf-8', errors='replace')
            # Strip ANSI codes and initialization text for cleaner output
            lines = text.split('\n')
            print(f"\n--- STAR RAW ---")
            print(text)
            print(f"--- END ---")
            responses.append(text)
        else:
            print(f"(no response)", file=sys.stderr)
            responses.append("")
        
        time.sleep(0.5)
    
    # Send quit
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    
    try:
        os.close(master_fd)
    except:
        pass
    os.waitpid(pid, 0)
    
    print(f"\n\n=== SUMMARY: {len(responses)} responses captured ===", file=sys.stderr)
    for i, r in enumerate(responses):
        # Extract just Star's response text (skip the echoed input)
        lines = r.split('\n')
        print(f"\n[Response {i+1}]:")
        for line in lines:
            if line.strip() and not line.startswith('>') and not line.startswith('─'):
                print(f"  {line}")

if __name__ == "__main__":
    main()
