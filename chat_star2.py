#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY — debug version"""
import os
import pty
import select
import time
import sys
import fcntl
import termios
import re

def write_all(fd, data):
    while data:
        n = os.write(fd, data)
        data = data[n:]

def read_with_timeout(fd, timeout=2.0, chunk=4096):
    output = []
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.2)
        if ready:
            try:
                data = os.read(fd, chunk)
                if data:
                    output.append(data)
                    deadline = time.time() + timeout  # reset on activity
            except OSError:
                break
        elif output:
            # No data but have some - short pause
            time.sleep(0.1)
            if time.time() > deadline:
                break
    return b"".join(output)

def clean_ansi(data):
    return re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', data.decode('utf-8', errors='replace'))

def main():
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
        os.execv("/home/zach/.openclaw/workspace/life/target/release/star", 
                 ["/home/zach/.openclaw/workspace/life/target/release/star", 
                  "--data-dir", "/home/zach/.openclaw/workspace/life", "chat"])
        os._exit(1)
    
    os.close(slave_fd)
    
    # Wait for init
    time.sleep(4)
    initial = read_with_timeout(master_fd, 2.0)
    print(f"INITIAL OUTPUT ({len(initial)} bytes):")
    print(clean_ansi(initial)[:500])
    print("---")
    
    # Send first message
    msg = "Hello Star, how are you today?"
    print(f"SENDING: {msg!r}")
    write_all(master_fd, (msg + "\n").encode())
    time.sleep(1)
    resp = read_with_timeout(master_fd, 8.0)
    print(f"RESPONSE ({len(resp)} bytes):")
    print(clean_ansi(resp))
    print("===")
    
    msg2 = "What have you been thinking about?"
    print(f"SENDING: {msg2!r}")
    write_all(master_fd, (msg2 + "\n").encode())
    time.sleep(1)
    resp2 = read_with_timeout(master_fd, 8.0)
    print(f"RESPONSE2 ({len(resp2)} bytes):")
    print(clean_ansi(resp2))
    print("===")
    
    msg3 = "What are you curious about?"
    print(f"SENDING: {msg3!r}")
    write_all(master_fd, (msg3 + "\n").encode())
    time.sleep(1)
    resp3 = read_with_timeout(master_fd, 8.0)
    print(f"RESPONSE3 ({len(resp3)} bytes):")
    print(clean_ansi(resp3))
    print("===")
    
    msg4 = "Tell me about yourself"
    print(f"SENDING: {msg4!r}")
    write_all(master_fd, (msg4 + "\n").encode())
    time.sleep(1)
    resp4 = read_with_timeout(master_fd, 8.0)
    print(f"RESPONSE4 ({len(resp4)} bytes):")
    print(clean_ansi(resp4))
    print("===")
    
    write_all(master_fd, b"/quit\n")
    time.sleep(1)
    os.close(master_fd)
    os.waitpid(pid, 0)
    print("DONE")

if __name__ == "__main__":
    main()
