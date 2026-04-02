#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY — full conversation"""
import os
import pty
import select
import time
import sys
import fcntl
import termios
import signal

def write_all(fd, data):
    while data:
        n = os.write(fd, data)
        data = data[n:]

def read_available(fd, timeout=2.0):
    """Read all available data from fd within timeout"""
    output = []
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.1)
        if ready:
            try:
                data = os.read(fd, 4096)
                if data:
                    output.append(data)
            except OSError:
                break
        elif output:
            # No data but we have some - small pause then continue
            time.sleep(0.1)
    return b"".join(output)

def expect(fd, needle, timeout=10.0):
    """Wait until needle appears in output or timeout"""
    buf = b""
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.2)
        if ready:
            try:
                data = os.read(fd, 1024)
                if data:
                    buf += data
                    if needle in buf:
                        return buf
            except OSError:
                break
        else:
            if buf:
                break
    return buf

def send_and_get(fd, message, timeout=15.0):
    """Send a message and get response up to next prompt"""
    write_all(fd, (message + "\n").encode())
    buf = b""
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.3)
        if ready:
            try:
                data = os.read(fd, 2048)
                if data:
                    buf += data
                    # Check if we got a prompt (Star is waiting for input)
                    if b"> " in buf and len(buf) > 10:
                        # Got response, might be done
                        # Wait a tiny bit for trailing output
                        extra = read_available(fd, 1.0)
                        return buf + extra
            except OSError:
                break
        elif buf:
            # No more data coming
            break
    return buf

def main():
    messages = [
        "Hello Star, how are you today?",
        "What have you been thinking about lately?",
        "What's been on your mind?",
        "What are you curious about right now?",
        "What have you been researching?",
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
                  "--data-dir", "/home/zach/.openclaw/workspace/life", "chat"])
        os._exit(1)
    
    os.close(slave_fd)
    
    # Wait for initialization
    time.sleep(3)
    
    # Clear initial output
    try:
        os.read(master_fd, 65536)
    except OSError:
        pass
    
    print("=== INTERACTING WITH STAR ===\n")
    
    for msg in messages:
        print(f"ME: {msg}")
        response = send_and_get(master_fd, msg)
        text = response.decode('utf-8', errors='replace')
        # Clean up ANSI codes roughly
        import re
        text = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', text)
        # Print only the actual response (after the > prompt)
        lines = text.split('\n')
        response_lines = []
        in_response = False
        for line in lines:
            if '>' in line and 'Star' not in line and '═' not in line:
                in_response = True
                continue
            if in_response:
                response_lines.append(line)
        resp_text = '\n'.join(response_lines).strip()
        print(f"STAR: {resp_text}")
        print()
        time.sleep(1)
    
    # Send quit
    print("ME: /quit")
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    
    os.close(master_fd)
    os.waitpid(pid, 0)
    print("=== CONVERSATION END ===")

if __name__ == "__main__":
    main()
