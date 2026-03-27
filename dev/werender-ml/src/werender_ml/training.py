"""PyTorch DDP Training Integration.

Real distributed training using PyTorch Distributed Data Parallel.
"""

import os
import json
import asyncio
import torch
import torch.nn as nn
import torch.distributed as dist
import torch.multiprocessing as mp
from torch.nn.parallel import DistributedDataParallel as DDP
from torch.utils.data import DataLoader, DistributedSampler
from torch.utils.data.dataset import Dataset
from pathlib import Path
from typing import Optional, List
import random
import numpy as np


class MockDataset(Dataset):
    """Mock dataset for testing."""
    
    def __init__(self, size: int = 1000, num_classes: int = 10):
        self.size = size
        self.num_classes = num_classes
        # Mock data: random images
        self.data = [torch.randn(3, 224, 224) for _ in range(size)]
        self.labels = [random.randint(0, num_classes - 1) for _ in range(size)]
    
    def __len__(self):
        return self.size
    
    def __getitem__(self, idx):
        return self.data[idx], self.labels[idx]


class ShardedDataset:
    """Dataset that can be sharded across workers."""
    
    def __init__(self, data_path: str, num_shards: int, shard_id: int):
        self.data_path = Path(data_path)
        self.num_shards = num_shards
        self.shard_id = shard_id
        
        # For now, create mock data
        # In full version, would load actual data
        self.samples_per_shard = 1000 // num_shards
        self.dataset = MockDataset(size=self.samples_per_shard)
        
        print(f" Worker {shard_id}: Loaded {len(self.dataset)} samples")
    
    def get_loader(self, batch_size: int, num_workers: int = 0):
        """Get DataLoader for this shard."""
        return DataLoader(
            self.dataset,
            batch_size=batch_size,
            shuffle=False,  # Sampler handles this
            num_workers=num_workers,
            pin_memory=True
        )


class DistributedTrainer:
    """Handles actual PyTorch DDP training."""
    
    def __init__(
        self,
        model: nn.Module,
        train_loader: DataLoader,
        optimizer: torch.optim.Optimizer,
        device: torch.device,
        rank: int,
        world_size: int
    ):
        self.model = model.to(device)
        self.train_loader = train_loader
        self.optimizer = optimizer
        self.device = device
        self.rank = rank
        self.world_size = world_size
        
        # Wrap in DDP
        self.model = DDP(self.model, device_ids=None if device.type == "cpu" else [device])
        
        # Loss
        self.criterion = nn.CrossEntropyLoss()
    
    def train_epoch(self, epoch: int) -> float:
        """Train for one epoch."""
        self.model.train()
        total_loss = 0.0
        num_batches = 0
        
        for batch_idx, (data, target) in enumerate(self.train_loader):
            data, target = data.to(self.device), target.to(self.device)
            
            # Forward pass
            output = self.model(data)
            loss = self.criterion(output, target)
            
            # Backward pass
            self.optimizer.zero_grad()
            loss.backward()
            self.optimizer.step()
            
            total_loss += loss.item()
            num_batches += 1
            
            if self.rank == 0 and batch_idx % 10 == 0:
                print(f"   [Rank {self.rank}] Batch {batch_idx}/{len(self.train_loader)} - Loss: {loss.item():.4f}")
        
        # Average loss across all ranks
        avg_loss = total_loss / num_batches
        if self.world_size > 1:
            dist.all_reduce(torch.tensor(avg_loss, device=self.device))
            avg_loss /= self.world_size
        
        return avg_loss


def setup_distributed(rank: int, world_size: int, master_addr: str = "127.0.0.1", master_port: int = 29500):
    """Setup PyTorch distributed training."""
    os.environ["MASTER_ADDR"] = master_addr
    os.environ["MASTER_PORT"] = str(master_port)
    
    dist.init_process_group(
        backend="gloo",  # Use gloo for CPU, nccl for GPU
        rank=rank,
        world_size=world_size
    )


def cleanup_distributed():
    """Cleanup distributed training."""
    dist.destroy_process_group()


def create_model(model_name: str, num_classes: int = 10) -> nn.Module:
    """Create a model by name."""
    # Try torchvision models first
    try:
        import torchvision.models as models
        
        model_registry = {
            "resnet18": models.resnet18,
            "resnet34": models.resnet34,
            "resnet50": models.resnet50,
            "alexnet": models.alexnet,
            "vgg16": models.vgg16,
        }
        
        if model_name.lower() in model_registry:
            model = model_registry[model_name.lower()]()
            
            # Modify final layer for custom num_classes
            if hasattr(model, "fc"):
                model.fc = nn.Linear(model.fc.in_features, num_classes)
            elif hasattr(model, "classifier"):
                if isinstance(model.classifier, nn.Sequential):
                    model.classifier[-1] = nn.Linear(model.classifier[-1].in_features, num_classes)
            
            return model
    except ImportError:
        pass
    
    # Fallback: simple CNN
    return nn.Sequential(
        nn.Conv2d(3, 64, 3, padding=1),
        nn.ReLU(),
        nn.MaxPool2d(2),
        nn.Conv2d(64, 128, 3, padding=1),
        nn.ReLU(),
        nn.MaxPool2d(2),
        nn.Conv2d(128, 256, 3, padding=1),
        nn.ReLU(),
        nn.AdaptiveAvgPool2d((1, 1)),
        nn.Flatten(),
        nn.Linear(256, num_classes)
    )


def worker_main(
    rank: int,
    world_size: int,
    model_name: str,
    data_path: str,
    epochs: int,
    batch_size: int,
    lr: float,
    master_addr: str,
    master_port: int
):
    """Main function for worker process."""
    # Setup distributed
    setup_distributed(rank, world_size, master_addr, master_port)
    
    # Create model
    model = create_model(model_name)
    
    # Create dataset shard
    dataset = ShardedDataset(data_path, world_size, rank)
    train_loader = dataset.get_loader(batch_size)
    
    # Optimizer
    optimizer = torch.optim.SGD(model.parameters(), lr=lr, momentum=0.9)
    
    # Device
    device = torch.device("cpu")  # Start with CPU
    if torch.cuda.is_available():
        device = torch.device(f"cuda:{rank % torch.cuda.device_count()}")
    
    # Trainer
    trainer = DistributedTrainer(model, train_loader, optimizer, device, rank, world_size)
    
    # Training loop
    for epoch in range(epochs):
        loss = trainer.train_epoch(epoch)
        if rank == 0:
            print(f" Epoch {epoch + 1}/{epochs} - Loss: {loss:.4f}")
    
    # Cleanup
    cleanup_distributed()
    
    if rank == 0:
        print(" Training complete!")


def spawn_workers(
    world_size: int,
    model_name: str,
    data_path: str,
    epochs: int,
    batch_size: int,
    lr: float
):
    """Spawn worker processes."""
    # Use multiprocessing
    world_size = min(world_size, mp.cpu_count())
    
    print(f" Starting {world_size} worker processes...")
    
    # Find free port
    import socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(("", 0))
    master_port = s.getsockname()[1]
    s.close()
    master_addr = "127.0.0.1"
    
    print(f" Master: {master_addr}:{master_port}")
    
    # Spawn processes
    mp.spawn(
        worker_main,
        args=(world_size, model_name, data_path, epochs, batch_size, lr, master_addr, master_port),
        nprocs=world_size,
        join=True
    )


# Export
__all__ = [
    "DistributedTrainer",
    "ShardedDataset",
    "create_model",
    "spawn_workers",
    "setup_distributed",
    "cleanup_distributed"
]
