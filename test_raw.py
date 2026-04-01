#!/usr/bin/env python3
"""Debug test - print raw output"""
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
        ready, _, _ = select.select([fd], [], [], 0.3)
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
    
    # Send greeting
    msg = "Hey Star, how are you today?"
    write_all(master_fd, (msg + "\n").encode())
    time.sleep(5)
    output = read_until(master_fd, time.time() + 10)
    print("=== RAW AFTER GREETING ===")
    print(output.decode('utf-8', errors='replace'))
    print("=== END ===\n")
    
    # Send statement about attention
    msg2 = "I've been thinking about attention mechanisms lately."
    write_all(master_fd, (msg2 + "\n").encode())
    time.sleep(5)
    output2 = read_until(master_fd, time.time() + 10)
    print("=== RAW AFTER ATTENTION ===")
    print(output2.decode('utf-8', errors='replace'))
    print("=== END ===\n")
    
    # Send "what's been on your mind"
    msg3 = "What's been on your mind?"
    write_all(master_fd, (msg3 + "\n").encode())
    time.sleep(5)
    output3 = read_until(master_fd, time.time() + 10)
    print("=== RAW AFTER MIND ===")
    print(output3.decode('utf-8', errors='replace'))
    print("=== END ===\n")
    
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    os.close(master_fd)
    os.waitpid(pid, 0)
