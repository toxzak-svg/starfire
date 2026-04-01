#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY, with longer response timeouts"""
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

def read_available(fd, timeout=15.0):
    """Read whatever is available on fd for up to timeout seconds.
    Returns when no data for 2 consecutive seconds."""
    output = []
    fl = fcntl.fcntl(fd, fcntl.F_GETFL)
    fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    
    deadline = time.time() + timeout
    last_read_time = time.time()
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.5)
        if ready:
            try:
                data = os.read(fd, 4096)
                if data:
                    output.append(data)
                    last_read_time = time.time()
                else:
                    break
            except OSError:
                break
        else:
            # Nothing ready in 0.5s - check if we should give up
            if output and (time.time() - last_read_time) > 5.0:
                # Got some output and waited 5s with nothing new
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
        # Child
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
    
    output = []
    time.sleep(8)  # Wait for initialization (this takes ~6s)
    
    for msg in messages:
        print(f"\n>>> SENDING: {msg}", file=sys.stderr)
        write_all(master_fd, (msg + "\n").encode())
        
        # Read response with generous timeout
        response = read_available(master_fd, timeout=20.0)
        if response:
            text = response.decode('utf-8', errors='replace')
            print(f"\n--- STAR RESPONSE ---\n{text}\n--- END ---", file=sys.stderr)
            output.append(text)
        else:
            print(f"\n--- (no response after 20s) ---", file=sys.stderr)
        
        time.sleep(1)
    
    # Send quit
    print(f"\n>>> SENDING: /quit", file=sys.stderr)
    write_all(master_fd, b"/quit\n")
    time.sleep(3)
    
    try:
        data = os.read(master_fd, 4096)
        if data:
            print(f"(final: {data.decode('utf-8', errors='replace')})", file=sys.stderr)
    except OSError:
        pass
    
    os.close(master_fd)
    os.waitpid(pid, 0)
    
    print(f"\n=== CAPTURED {len(output)} RESPONSES ===", file=sys.stderr)
    for i, resp in enumerate(output):
        print(f"\n--- Response {i+1} ---\n{resp}", file=sys.stderr)

if __name__ == "__main__":
    main()
