#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY"""
import os
import pty
import select
import time
import sys
import fcntl
import struct
import termios

def write_all(fd, data):
    """Write data to fd, handling partial writes"""
    while data:
        n = os.write(fd, data)
        data = data[n:]

def main():
    messages = [
        "Hello Star, how are you today?",
        "What have you been thinking about?",
        "What are you curious about?",
        "What have you been researching?",
        "Tell me about yourself",
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
        os.execv("/home/zach/.openclaw/workspace/life/target/release/star", 
                 ["/home/zach/.openclaw/workspace/life/target/release/star", 
                  "--data-dir", "/home/zach/.openclaw/workspace/life/life", "chat"])
        os._exit(1)
    
    # Parent
    os.close(slave_fd)
    
    # Set master to non-blocking
    fl = fcntl.fcntl(master_fd, fcntl.F_GETFL)
    fcntl.fcntl(master_fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    
    output = b""
    time.sleep(3)  # Wait for initialization
    
    for msg in messages:
        print(f"SENDING: {msg}", file=sys.stderr)
        write_all(master_fd, (msg + "\n").encode())
        time.sleep(4)
        
        # Read available output
        while True:
            ready, _, _ = select.select([master_fd], [], [], 0.5)
            if not ready:
                break
            try:
                data = os.read(master_fd, 4096)
                if data:
                    output += data
                else:
                    break
            except OSError:
                break
    
    # Send quit
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    
    # Read remaining output
    while True:
        ready, _, _ = select.select([master_fd], [], [], 0.5)
        if not ready:
            break
        try:
            data = os.read(master_fd, 4096)
            if data:
                output += data
            else:
                break
        except OSError:
            break
    
    os.close(master_fd)
    os.waitpid(pid, 0)
    
    print("=== STAR OUTPUT ===", file=sys.stderr)
    print(output.decode('utf-8', errors='replace'), file=sys.stderr)
    print("=== END ===", file=sys.stderr)

if __name__ == "__main__":
    main()
