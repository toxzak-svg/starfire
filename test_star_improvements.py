#!/usr/bin/env python3
"""Interact with Star using a pseudo-TTY"""
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

def chat(star_path, data_dir, messages, timeout=25):
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
    
    time.sleep(3)  # Wait for initialization
    
    results = []
    
    for msg in messages:
        sys.stderr.write(f"\n>>> SENDING: {msg}\n")
        write_all(master_fd, (msg + "\n").encode())
        
        deadline = time.time() + timeout
        output = read_until(master_fd, deadline)
        
        decoded = output.decode('utf-8', errors='replace')
        
        # Extract lines that don't start with ANSI codes or Memory/storage lines
        lines = decoded.split('\n')
        response_lines = []
        skip_until_prompt = True
        for line in lines:
            # Skip lines with ANSI codes (INFO, WARN, color codes)
            if '[2m' in line or '[32m' in line or '[33m' in line or '[31m' in line:
                continue
            # Skip lines with Memory storage
            if line.startswith('Memory '):
                continue
            # Skip separator lines
            if line.startswith('─') or line.startswith('═'):
                skip_until_prompt = True
                continue
            # After prompt marker, collect response
            if line.startswith('> '):
                skip_until_prompt = False
                continue
            if skip_until_prompt:
                continue
            # Stop at next prompt marker or quit
            if line.startswith('Type /') or line.startswith('> '):
                break
            response_lines.append(line)
        
        response = '\n'.join(response_lines).strip()
        results.append(response)
        sys.stderr.write(f"=== RESPONSE ===\n{response}\n=== END ===\n")
    
    # Quit
    write_all(master_fd, b"/quit\n")
    time.sleep(2)
    
    try:
        os.read(master_fd, 4096)
    except OSError:
        pass
    
    os.close(master_fd)
    os.waitpid(pid, 0)
    
    return results

if __name__ == "__main__":
    star_path = "/home/zach/.openclaw/workspace/life/life/target/release/star"
    data_dir = "/home/zach/.openclaw/workspace/life/life"
    
    # Test messages
    messages = [
        "Hey Star, how are you today?",
        "I've been thinking about attention mechanisms lately.",
        "What's been on your mind lately?",
        "Can you tell me something you're curious about?",
        "What's the most interesting thing you've learned recently?",
        "Do you think consciousness is something you can have?",
        "I'm working on a project involving neural networks.",
        "Tell me about yourself.",
    ]
    
    results = chat(star_path, data_dir, messages)
    
    print("\n\n=== CONVERSATION SUMMARY ===")
    for msg, resp in zip(messages, results):
        print(f"\nUser: {msg}")
        print(f"Star: {resp}")
