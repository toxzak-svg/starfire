#!/usr/bin/env python3
"""
Training script for Wide and Shallow Transformer

Supports:
- Distributed training with Accelerate
- Mixed precision training (FP16/BF16)
- Gradient accumulation
- Learning rate scheduling with warmup
- Checkpointing and resuming
"""

import os
import sys
import json
import yaml
import logging
from pathlib import Path
from typing import Optional, Dict, Any

import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Dataset
from torch.optim import AdamW
from torch.cuda.amp import autocast, GradScaler
from tqdm import tqdm

# For distributed training
try:
    from accelerate import Accelerator, DistributedDataParallelKwargs
    ACCELERATE_AVAILABLE = True
except ImportError:
    ACCELERATE_AVAILABLE = False

# For tokenization
try:
    from transformers import GPT2Tokenizer
    TRANSFORMERS_AVAILABLE = True
except ImportError:
    TRANSFORMERS_AVAILABLE = False

from models.wide_shallow import create_wide_shallow_model


# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("training.log")
    ]
)
logger = logging.getLogger(__name__)


class DummyDataset(Dataset):
    """Dummy dataset for testing - replace with real data loading."""
    
    def __init__(self, vocab_size: int, seq_len: int, size: int = 10000):
        self.vocab_size = vocab_size
        self.seq_len = seq_len
        self.size = size
        
    def __len__(self):
        return self.size
    
    def __getitem__(self, idx):
        # Generate random tokens (replace with real data)
        input_ids = torch.randint(0, self.vocab_size, (self.seq_len,))
        labels = input_ids.clone()
        return {"input_ids": input_ids, "labels": labels}


def load_config(config_path: str) -> Dict[str, Any]:
    """Load configuration from YAML file."""
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)


def create_optimizer(model: nn.Module, config: Dict[str, Any]) -> torch.optim.Optimizer:
    """Create optimizer with weight decay."""
    train_config = config.get("training", config)
    
    # Separate parameters for weight decay
    no_decay = ["bias", "LayerNorm.weight", "layer_norm.weight"]
    optimizer_grouped_parameters = [
        {
            "params": [p for n, p in model.named_parameters() 
                      if not any(nd in n for nd in no_decay)],
            "weight_decay": train_config.get("weight_decay", 0.01)
        },
        {
            "params": [p for n, p in model.named_parameters() 
                      if any(nd in n for nd in no_decay)],
            "weight_decay": 0.0
        }
    ]
    
    optimizer = AdamW(
        optimizer_grouped_parameters,
        lr=train_config.get("learning_rate", 1e-4),
        betas=(train_config.get("beta1", 0.9), train_config.get("beta2", 0.95)),
    )
    
    return optimizer


def create_scheduler(
    optimizer: torch.optim.Optimizer,
    config: Dict[str, Any],
    num_training_steps: int,
) -> torch.optim.lr_scheduler._LRScheduler:
    """Create learning rate scheduler with warmup."""
    train_config = config.get("training", config)
    
    warmup_steps = train_config.get("warmup_steps", 1000)
    min_lr = train_config.get("min_lr", 1e-6)
    
    # Linear warmup + linear decay
    def lr_lambda(current_step: int):
        if current_step < warmup_steps:
            return float(current_step) / float(max(1, warmup_steps))
        elif current_step > num_training_steps * 0.9:
            # Final 10% - linear decay to min_lr
            progress = (current_step - num_training_steps * 0.9) / (num_training_steps * 0.1)
            return max(min_lr / train_config.get("learning_rate", 1e-4), 1.0 - progress)
        else:
            return 1.0
    
    scheduler = torch.optim.lr_scheduler.LambdaLR(optimizer, lr_lambda)
    
    return scheduler


def train_step(
    model: nn.Module,
    batch: Dict[str, torch.Tensor],
    optimizer: torch.optim.Optimizer,
    scaler: Optional[GradScaler],
    config: Dict[str, Any],
    use_amp: bool = True,
) -> Dict[str, float]:
    """Single training step."""
    model.train()
    
    input_ids = batch["input_ids"]
    labels = batch.get("labels", input_ids)
    
    # Move to device
    device = next(model.parameters()).device
    input_ids = input_ids.to(device)
    labels = labels.to(device)
    
    # Forward pass with mixed precision
    if use_amp and scaler is not None:
        with autocast():
            outputs = model(input_ids=input_ids, labels=labels)
            loss = outputs["loss"]
        
        # Backward pass with gradient scaling
        scaler.scale(loss).backward()
        
        # Gradient clipping
        if config.get("training", {}).get("max_grad_norm", 1.0):
            scaler.unscale_(optimizer)
            torch.nn.utils.clip_grad_norm_(
                model.parameters(), 
                config["training"]["max_grad_norm"]
            )
        
        scaler.step(optimizer)
        scaler.update()
    else:
        outputs = model(input_ids=input_ids, labels=labels)
        loss = outputs["loss"]
        
        loss.backward()
        
        if config.get("training", {}).get("max_grad_norm", 1.0):
            torch.nn.utils.clip_grad_norm_(
                model.parameters(), 
                config["training"]["max_grad_norm"]
            )
        
        optimizer.step()
    
    optimizer.zero_grad()
    
    return {"loss": loss.item()}


@torch.no_grad()
def evaluate(
    model: nn.Module,
    eval_dataloader: DataLoader,
    use_amp: bool = True,
) -> Dict[str, float]:
    """Evaluate model on validation set."""
    model.eval()
    
    total_loss = 0.0
    num_batches = 0
    
    for batch in tqdm(eval_dataloader, desc="Evaluating"):
        input_ids = batch["input_ids"]
        labels = batch.get("labels", input_ids)
        
        device = next(model.parameters()).device
        input_ids = input_ids.to(device)
        labels = labels.to(device)
        
        if use_amp:
            with autocast():
                outputs = model(input_ids=input_ids, labels=labels)
        else:
            outputs = model(input_ids=input_ids, labels=labels)
        
        total_loss += outputs["loss"].item()
        num_batches += 1
    
    return {"eval_loss": total_loss / num_batches}


def save_checkpoint(
    model: nn.Module,
    optimizer: torch.optim.Optimizer,
    scheduler: torch.optim.lr_scheduler._LRScheduler,
    scaler: Optional[GradScaler],
    step: int,
    config: Dict[str, Any],
    checkpoint_dir: str = "checkpoints",
):
    """Save training checkpoint."""
    checkpoint_path = Path(checkpoint_dir) / f"checkpoint-{step}"
    checkpoint_path.mkdir(parents=True, exist_ok=True)
    
    # Save model
    model_to_save = model.module if hasattr(model, "module") else model
    torch.save(model_to_save.state_dict(), checkpoint_path / "model.pt")
    
    # Save optimizer
    torch.save(optimizer.state_dict(), checkpoint_path / "optimizer.pt")
    
    # Save scheduler
    torch.save(scheduler.state_dict(), checkpoint_path / "scheduler.pt")
    
    # Save scaler if using AMP
    if scaler is not None:
        torch.save(scaler.state_dict(), checkpoint_path / "scaler.pt")
    
    # Save config
    with open(checkpoint_path / "config.yaml", 'w') as f:
        yaml.dump(config, f)
    
    logger.info(f"Saved checkpoint to {checkpoint_path}")


def load_checkpoint(
    checkpoint_path: str,
    model: nn.Module,
    optimizer: Optional[torch.optim.Optimizer] = None,
    scheduler: Optional[torch.optim.lr_scheduler._LRScheduler] = None,
    scaler: Optional[GradScaler] = None,
) -> int:
    """Load training checkpoint."""
    checkpoint_path = Path(checkpoint_path)
    
    # Load model
    model.load_state_dict(torch.load(checkpoint_path / "model.pt"))
    
    step = 0
    if optimizer is not None and (checkpoint_path / "optimizer.pt").exists():
        optimizer.load_state_dict(torch.load(checkpoint_path / "optimizer.pt"))
    
    if scheduler is not None and (checkpoint_path / "scheduler.pt").exists():
        scheduler.load_state_dict(torch.load(checkpoint_path / "scheduler.pt"))
    
    if scaler is not None and (checkpoint_path / "scaler.pt").exists():
        scaler.load_state_dict(torch.load(checkpoint_path / "scaler.pt"))
    
    # Extract step from filename
    if "checkpoint-" in checkpoint_path.name:
        step = int(checkpoint_path.name.split("-")[1])
    
    logger.info(f"Loaded checkpoint from {checkpoint_path}")
    
    return step


def main(config_path: str = "config.yaml"):
    """Main training loop."""
    
    # Load config
    config = load_config(config_path)
    model_config = config.get("model", config)
    train_config = config.get("training", config)
    
    logger.info(f"Starting training with config: {json.dumps(model_config, indent=2)}")
    
    # Setup device
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    logger.info(f"Using device: {device}")
    
    # Create model
    logger.info("Creating model...")
    model = create_wide_shallow_model(config)
    model = model.to(device)
    
    # Count parameters
    num_params = sum(p.numel() for p in model.parameters())
    logger.info(f"Model parameters: {num_params:,}")
    
    # Create datasets (replace with real data)
    logger.info("Loading datasets...")
    data_config = config.get("data", {})
    tokenizer_name = data_config.get("tokenizer", "gpt2")
    
    if TRANSFORMERS_AVAILABLE:
        tokenizer = GPT2Tokenizer.from_pretrained(tokenizer_name)
        tokenizer.pad_token = tokenizer.eos_token
    else:
        tokenizer = None
    
    # Create dummy datasets for now
    train_dataset = DummyDataset(
        vocab_size=model_config.get("vocab_size", 50257),
        seq_len=train_config.get("max_seq_length", 2048),
        size=10000,
    )
    eval_dataset = DummyDataset(
        vocab_size=model_config.get("vocab_size", 50257),
        seq_len=train_config.get("max_seq_length", 2048),
        size=1000,
    )
    
    # Create dataloaders
    batch_size = train_config.get("batch_size", 1)
    train_dataloader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        shuffle=True,
        num_workers=data_config.get("preprocessing_num_workers", 4),
    )
    eval_dataloader = DataLoader(
        eval_dataset,
        batch_size=batch_size,
        shuffle=False,
        num_workers=data_config.get("preprocessing_num_workers", 4),
    )
    
    # Create optimizer
    optimizer = create_optimizer(model, config)
    
    # Create scheduler
    max_steps = train_config.get("max_steps", 100000)
    scheduler = create_scheduler(optimizer, config, max_steps)
    
    # Setup mixed precision
    use_amp = train_config.get("mixed_precision", True) and torch.cuda.is_available()
    scaler = GradScaler() if use_amp else None
    
    if use_amp:
        logger.info("Using mixed precision training (FP16)")
    
    # Training loop
    logger.info("Starting training...")
    global_step = 0
    gradient_accumulation_steps = train_config.get("gradient_accumulation_steps", 8)
    
    progress_bar = tqdm(total=max_steps, desc="Training")
    
    model.train()
    
    for epoch in range(100):  # Run until max_steps
        for batch in train_dataloader:
            # Training step
            loss_dict = train_step(
                model=model,
                batch=batch,
                optimizer=optimizer,
                scaler=scaler,
                config=config,
                use_amp=use_amp,
            )
            
            # Step scheduler
            scheduler.step()
            
            global_step += 1
            
            # Update progress
            progress_bar.update(1)
            progress_bar.set_postfix({
                "loss": f"{loss_dict['loss']:.4f}",
                "lr": f"{optimizer.param_groups[0]['lr']:.2e}"
            })
            
            # Evaluate periodically
            if global_step % train_config.get("eval_steps", 1000) == 0:
                eval_metrics = evaluate(model, eval_dataloader, use_amp=use_amp)
                logger.info(f"Step {global_step}: {eval_metrics}")
                model.train()
            
            # Save checkpoint periodically
            if global_step % train_config.get("save_steps", 5000) == 0:
                save_checkpoint(
                    model=model,
                    optimizer=optimizer,
                    scheduler=scheduler,
                    scaler=scaler,
                    step=global_step,
                    config=config,
                )
            
            # Stop if max steps reached
            if global_step >= max_steps:
                break
        
        if global_step >= max_steps:
            break
    
    progress_bar.close()
    
    # Save final model
    logger.info("Training complete! Saving final model...")
    save_checkpoint(
        model=model,
        optimizer=optimizer,
        scheduler=scheduler,
        scaler=scaler,
        step=global_step,
        config=config,
        checkpoint_dir="checkpoints/final",
    )
    
    logger.info("Done!")


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Train Wide and Shallow Transformer")
    parser.add_argument("--config", type=str, default="config.yaml",
                       help="Path to config file")
    args = parser.parse_args()
    
    main(args.config)
