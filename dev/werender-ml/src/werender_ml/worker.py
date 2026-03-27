"""WeRender-ML Worker.

Participates in distributed ML training.
"""

import json
import time
import requests
import torch
import torch.nn as nn
from typing import Optional


class MLWorker:
    """ML Training Worker."""
    
    def __init__(
        self,
        coordinator_url: str,
        api_key: Optional[str] = None,
        gpu_id: Optional[int] = None,
        cpu_only: bool = False
    ):
        self.coordinator_url = coordinator_url.rstrip("/")
        self.api_key = api_key
        self.gpu_id = gpu_id
        self.cpu_only = cpu_only
        
        # Determine device
        if cpu_only:
            self.device = torch.device("cpu")
        elif gpu_id is not None and torch.cuda.is_available():
            self.device = torch.device(f"cuda:{gpu_id}")
        elif torch.cuda.is_available():
            self.device = torch.device("cuda")
        else:
            self.device = torch.device("cpu")
        
        self.worker_id = None
        self.connected = False
        self.running = False
        
        # Training state
        self.model = None
        self.current_epoch = 0
    
    def get_system_info(self) -> dict:
        """Get system information."""
        info = {
            "gpu": str(self.device),
            "gpu_available": torch.cuda.is_available(),
            "gpu_count": torch.cuda.device_count() if torch.cuda.is_available() else 0,
            "cpu_count": 1,  # Simplified
            "memory_total": 0,
        }
        
        if torch.cuda.is_available():
            try:
                info["memory_total"] = torch.cuda.get_device_properties(0).total_memory
            except:
                pass
        
        return info
    
    def connect(self) -> bool:
        """Connect to coordinator."""
        # Register with coordinator
        url = f"{self.coordinator_url}/register"
        
        data = {
            "worker_id": f"worker-{int(time.time())}",
            "info": self.get_system_info()
        }
        
        try:
            response = requests.post(url, json=data, timeout=10)
            if response.status_code == 200:
                result = response.json()
                self.worker_id = result.get("worker_id")
                self.connected = True
                print(f" Connected to coordinator as {self.worker_id}")
                return True
            else:
                print(f" Failed to connect: {response.status_code}")
                return False
        except Exception as e:
            print(f" Connection error: {e}")
            return False
    
    def fetch_task(self) -> Optional[dict]:
        """Fetch next training task from coordinator."""
        url = f"{self.coordinator_url}/task"
        
        try:
            response = requests.get(
                url,
                headers={"X-Worker-ID": self.worker_id},
                timeout=5
            )
            if response.status_code == 200:
                return response.json()
            return None
        except:
            return None
    
    def train_epoch(self, task: dict):
        """Train for one epoch (placeholder)."""
        print(f" Training epoch {self.current_epoch + 1}")
        
        # In full implementation:
        # 1. Download data shard
        # 2. Train model
        # 3. Push gradients back
        
        # Placeholder: simulate training
        for step in range(10):
            time.sleep(0.3)
            if step % 3 == 0:
                print(f"   Step {step + 1}/10 - Loss: {1.5 - step * 0.1:.4f}")
        
        self.current_epoch += 1
    
    def run(self):
        """Main worker loop."""
        if not self.connected:
            print(" Not connected to coordinator")
            return
        
        self.running = True
        print(f"⏳ Waiting for training tasks...")
        
        while self.running:
            # Fetch task
            task = self.fetch_task()
            
            if task:
                if task.get("type") == "train":
                    self.train_epoch(task)
                elif task.get("type") == "shutdown":
                    print(" Coordinator shutdown, exiting")
                    break
            else:
                time.sleep(2)  # Wait before checking again
    
    def stop(self):
        """Stop the worker."""
        self.running = False
