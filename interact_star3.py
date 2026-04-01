#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY, with line-by-line reading"""
import os
import pty
import select
import time
import sys
import fcntl
import select as sel

def write_all(fd, data):
    while data:
        n = os.write(fd, data)
        data = data[n:]

def main():
    messages = [
        "Hello Star, how are you today?",
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
        os.execv("/home/zach/.openclaw/workspace/life/target/release/star", 
                 ["/home/zach/.openclaw/workspace/life/target/release/star", 
                  "--data-dir", "/home/zach/.openclaw/workspace/life/life", "chat"])
        os._exit(1)
    
    os.close(slave_fd)
    
    # Set master to non-blocking
    fl = fcntl.fcntl(master_fd, fcntl.F_GETFL)
    fcntl.fcntl(master_fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    
    output = []
    time.sleep(4)  # Wait for initialization
    
    print(f"SENDING: {messages[0]}", file=sys.stderr)
    write_all(master_fd, (messages[0] + "\n").encode())
    
    # Poll for output for up to 30 seconds
    deadline = time.time() + 30
    last_output_time = time.time()
    
    while time.time() < deadline:
        ready, _, _ = sel.select([master_fd], [], [], 0.5)
        if ready:
            try:
                data = os.read(master_fd, 8192)
                if data:
                    output.append(data)
                    print(f"Read {len(data)} bytes", file=sys.stderr)
                    last_output_time = time.time()
            except OSError:
                break
        else:
            # No data available
            if time.time() - last_output_time > 3 and output:
                # No new output for 3 seconds after we've seen some
                break
    
    # Send quit
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    
    try:
        data = os.read(master_fd, 8192)
        if data:
            output.append(data)
    except OSError:
        pass
    
    os.close(master_fd)
    os.waitpid(pid, 0)
    
    full_output = b"".join(output).decode('utf-8', errors='replace')
    
    print("=== STAR OUTPUT ===", file=sys.stderr)
    print(full_output, file=sys.stderr)
    print("=== END ===", file=sys.stderr)

if __name__ == "__main__":
    import termios
    main()
