"""Run mind-agent as a persistent Windows/Linux service.

This script auto-restarts if it crashes, logs output, and runs quietly.
"""

import subprocess
import sys
import time
import os
from pathlib import Path

# Configuration
SERVICE_NAME = "MindAgent"
RESTART_DELAY = 5  # seconds to wait before restarting
MAX_RESTARTS = 10  # Give up after this many restarts in a row

# Log to file
LOG_FILE = Path(os.environ.get('LOG_FILE', 'mind-agent.log'))


def log(message: str):
    """Log to file with timestamp."""
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    log_line = f"[{timestamp}] {message}\n"
    with open(LOG_FILE, "a") as f:
        f.write(log_line)
    print(log_line.strip())


def main():
    """Run mind-agent with auto-restart."""
    log(f"Starting {SERVICE_NAME}")
    
    restarts = 0
    
    while restarts < MAX_RESTARTS:
        try:
            # Import and run
            from mind_agent import AgentMind
            
            agent = AgentMind()
            
            # Teach it basic facts
            agent.remember("Your name is Zach", importance=0.9)
            agent.remember("You created me", importance=0.9)
            agent.remember("We work on AI research", importance=0.8)
            
            log("Mind-agent initialized and ready!")
            
            # Simple loop - wait for input
            while True:
                time.sleep(10)  # Check every 10 seconds
                
        except KeyboardInterrupt:
            log("Shutting down (keyboard interrupt)")
            break
            
        except Exception as e:
            restarts += 1
            log(f"Error: {e}")
            log(f"Restarting in {RESTART_DELAY} seconds... ({restarts}/{MAX_RESTARTS})")
            time.sleep(RESTART_DELAY)
    
    log(f"Gave up after {MAX_RESTARTS} restarts")


if __name__ == "__main__":
    main()
