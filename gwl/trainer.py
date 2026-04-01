"""Training utilities for Global Workspace Language Model."""

import torch
from torch.utils.data import Dataset
from typing import List, Optional


class TextDataset(Dataset):
    """Dataset for text tokenization and sequence creation.
    
    This dataset takes a list of token IDs and creates sequences
    of a specified length for language model training.
    """
    
    def __init__(self, tokens: List[int], seq_len: int = 128, stride: int = 64):
        """
        Args:
            tokens: List of token IDs
            seq_len: Length of each sequence
            stride: Step size between sequences (for overlapping sequences)
        """
        self.tokens = tokens
        self.seq_len = seq_len
        self.stride = stride
        
        # Create input-target pairs with stride
        self.sequences = []
        for i in range(0, len(tokens) - seq_len, stride):
            input_seq = tokens[i:i + seq_len]
            target_seq = tokens[i:i + seq_len]
            self.sequences.append((input_seq, target_seq))
        
    def __len__(self) -> int:
        return len(self.sequences)
    
    def __getitem__(self, idx: int):
        input_seq, target_seq = self.sequences[idx]
        return (
            torch.tensor(input_seq, dtype=torch.long),
            torch.tensor(target_seq, dtype=torch.long),
        )


class GWLTrainer:
    """Trainer class for GWL model with convenience methods."""
    
    def __init__(self, model, train_dataset: Dataset, val_dataset: Optional[Dataset] = None,
                 config=None, device: str = "cuda"):
        """
        Args:
            model: The GWL model to train
            train_dataset: Training dataset
            val_dataset: Optional validation dataset
            config: Training configuration
            device: Device to train on
        """
        self.model = model
        self.train_dataset = train_dataset
        self.val_dataset = val_dataset
        self.config = config
        self.device = device
        
        # Move model to device
        self.model = self.model.to(device)
        
    def get_train_loader(self, batch_size: int, num_workers: int = 4, shuffle: bool = True):
        """Create training data loader."""
        from torch.utils.data import DataLoader
        return DataLoader(
            self.train_dataset,
            batch_size=batch_size,
            shuffle=shuffle,
            num_workers=num_workers,
            pin_memory=True,
        )
    
    def get_val_loader(self, batch_size: int, num_workers: int = 2):
        """Create validation data loader."""
        from torch.utils.data import DataLoader
        if self.val_dataset is None:
            return None
        return DataLoader(
            self.val_dataset,
            batch_size=batch_size,
            shuffle=False,
            num_workers=num_workers,
            pin_memory=True,
        )
    
    def train_step(self, batch, optimizer, scaler=None):
        """Single training step."""
        input_ids, target_ids = batch
        input_ids = input_ids.to(self.device, non_blocking=True)
        target_ids = target_ids.to(self.device, non_blocking=True)
        
        self.model.train()
        
        if scaler is not None:
            from torch.cuda.amp import autocast
            with autocast():
                outputs = self.model(input_ids, target_ids)
                loss = outputs["loss"]
        else:
            outputs = self.model(input_ids, target_ids)
            loss = outputs["loss"]
        
        if loss is None:
            return None
        
        if scaler is not None:
            scaler.scale(loss).backward()
            return loss.item()
        else:
            loss.backward()
            return loss.item()
    
    @torch.no_grad()
    def evaluate(self, val_loader):
        """Evaluate on validation set."""
        self.model.eval()
        total_loss = 0
        num_batches = 0
        
        for batch in val_loader:
            input_ids, target_ids = batch
            input_ids = input_ids.to(self.device)
            target_ids = target_ids.to(self.device)
            
            outputs = self.model(input_ids, target_ids)
            if outputs["loss"] is not None:
                total_loss += outputs["loss"].item()
                num_batches += 1
        
        return total_loss / max(num_batches, 1)
    
    def save_checkpoint(self, path: str, step: int, optimizer=None, **kwargs):
        """Save model checkpoint."""
        checkpoint = {
            "step": step,
            "model_state_dict": self.model.state_dict(),
        }
        if optimizer is not None:
            checkpoint["optimizer_state_dict"] = optimizer.state_dict()
        checkpoint.update(kwargs)
        torch.save(checkpoint, path)
    
    def load_checkpoint(self, path: str, optimizer=None):
        """Load model checkpoint."""
        checkpoint = torch.load(path, map_location=self.device)
        self.model.load_state_dict(checkpoint["model_state_dict"])
        if optimizer is not None and "optimizer_state_dict" in checkpoint:
            optimizer.load_state_dict(checkpoint["optimizer_state_dict"])
        return checkpoint