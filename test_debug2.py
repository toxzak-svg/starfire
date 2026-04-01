#!/usr/bin/env python3
"""Print raw output for debugging"""
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

def read_until(fd, deadline):
    output = []
    last_output_time = time.time()
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.5)
        if ready:
            try:
                data = os.read(fd, 8192)
                if data:
                    output.append(data)
                    last_output_time = time.time()
            except OSError:
                break
        else:
            if time.time() - last_output_time > 2 and output:
                break
    return b"".join(output)

if __name__ == "__main__":
    star_path = "/home/zach/.openclaw/workspace/life/life/target/release/star"
    data_dir = "/home/zach/.openclaw/workspace/life/life"
    
    messages = [
        "I've been thinking about attention mechanisms lately.",
        "What's been on your mind?",
        "Can you tell me something you're curious about?",
        "What's the most interesting thing you've learned recently?",
        "Do you think consciousness is something you can have?",
        "Tell me about yourself.",
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
        os.execv(star_path, 
                 [star_path, "--data-dir", data_dir, "chat"])
        os._exit(1)
    
    os.close(slave_fd)
    
    fl = fcntl.fcntl(master_fd, fcntl.F_GETFL)
    fcntl.fcntl(master_fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    
    time.sleep(3)
    
    for msg in messages:
        sys.stderr.write(f"\n{'='*50}\nSENDING: {msg}\n{'='*50}\n")
        write_all(master_fd, (msg + "\n").encode())
        time.sleep(6)
        output = read_until(master_fd, time.time() + 10)
        decoded = output.decode('utf-8', errors='replace')
        sys.stderr.write(decoded + "\n")
    
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    os.close(master_fd)
    os.waitpid(pid, 0)
