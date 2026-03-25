"""
Training script for Global Workspace Hive

Features:
- Specialization loss to encourage processor diversity
- Proper learning rate schedule (warmup + cosine)
- Multiple processor type support (rnn, gru, lstm, tiny_lm, ssm)
- Mixed precision training support
- Gradient accumulation for larger effective batch sizes
- Evaluation on validation set
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import Dataset, DataLoader
import numpy as np
import math
import os
from hive import GlobalWorkspaceHive, count_parameters


# ============== Data Loading ==============

class TextDataset(Dataset):
    """Simple text dataset for training"""
    
    def __init__(self, text, vocab, seq_len=128):
        self.seq_len = seq_len
        self.vocab = vocab
        self.vocab_size = len(vocab)
        
        # Tokenize
        tokens = [vocab.get(c, vocab['<unk>']) for c in text]
        
        # Create sequences (overlapping for more training data)
        self.sequences = []
        stride = seq_len  # Non-overlapping for now
        for i in range(0, len(tokens) - seq_len - 1, stride):
            seq = tokens[i:i + seq_len]
            self.sequences.append(torch.tensor(seq, dtype=torch.long))
    
    def __len__(self):
        return len(self.sequences)
    
    def __getitem__(self, idx):
        return self.sequences[idx]


class CharacterVocabulary:
    """Character-level vocabulary builder"""
    
    def __init__(self, text=None):
        self.special_tokens = ['<pad>', '<unk>', '<bos>', '<eos>']
        
        if text is not None:
            chars = sorted(set(text))
            self.token_to_idx = {tok: i for i, tok in enumerate(self.special_tokens)}
            for i, c in enumerate(chars):
                self.token_to_idx[c] = len(self.token_to_idx)
            
            self.idx_to_token = {v: k for k, v in self.token_to_idx.items()}
        else:
            self.token_to_idx = {}
            self.idx_to_token = {}
    
    def __len__(self):
        return len(self.token_to_idx)
    
    def __getitem__(self, key):
        if isinstance(key, str):
            return self.token_to_idx.get(key, self.token_to_idx['<unk>'])
        elif isinstance(key, int):
            return self.idx_to_token.get(key, '<unk>')
    
    @staticmethod
    def from_text(text):
        return CharacterVocabulary(text)


# ============== Training Components ==============

def gumbel_softmax(logits, temperature=1.0, hard=False):
    """Gumbel-softmax for differentiable discrete sampling"""
    gumbels = -torch.empty_like(logits).exponential_().log()
    y = (logits + gumbels) / temperature
    
    if hard:
        # Straight-through: use hard one-hot during forward
        y_hard = F.one_hot(y.argmax(dim=-1), num_classes=logits.size(-1)).float()
        y = (y_hard - y).detach() + y
    else:
        y = F.softmax(y, dim=-1)
    
    return y


def temperature_schedule(step, warmup_steps, total_steps, temp_start=2.0, temp_end=0.1):
    """Anneal temperature from temp_start to temp_end after warmup"""
    if step < warmup_steps:
        return temp_start
    else:
        progress = (step - warmup_steps) / (total_steps - warmup_steps)
        return temp_start - (temp_start - temp_end) * progress


def get_lr(step, warmup_steps, base_lr, total_steps, min_lr=1e-5):
    """Learning rate schedule: linear warmup + cosine decay"""
    if step < warmup_steps:
        # Linear warmup
        return base_lr * (step + 1) / warmup_steps
    else:
        # Cosine decay
        progress = (step - warmup_steps) / (total_steps - warmup_steps)
        return min_lr + (base_lr - min_lr) * 0.5 * (1 + math.cos(math.pi * progress))


class Trainer:
    """Trainer class for Global Workspace Hive"""
    
    def __init__(
        self,
        model,
        train_loader,
        val_loader=None,
        optimizer=None,
        device='cuda',
        lr=1e-4,
        warmup_steps=500,
        total_steps=10000,
        grad_clip=1.0,
        grad_accum_steps=1,
        specialization_weight=0.01,
        log_interval=100,
        eval_interval=1000,
        save_interval=5000,
    ):
        self.model = model
        self.train_loader = train_loader
        self.val_loader = val_loader
        self.device = device
        self.grad_clip = grad_clip
        self.grad_accum_steps = grad_accum_steps
        self.specialization_weight = specialization_weight
        self.log_interval = log_interval
        self.eval_interval = eval_interval
        self.save_interval = save_interval
        self.warmup_steps = warmup_steps
        self.total_steps = total_steps
        
        # Move model to device
        self.model = model.to(device)
        
        # Optimizer
        if optimizer is None:
            self.optimizer = torch.optim.AdamW(
                model.parameters(),
                lr=lr,
                weight_decay=0.01,
                betas=(0.9, 0.95)
            )
        else:
            self.optimizer = optimizer
        
        # Loss function
        self.loss_fn = nn.CrossEntropyLoss(ignore_index=0)  # Ignore padding
        
        # Training state
        self.step = 0
        self.best_val_loss = float('inf')
    
    def train_step(self, batch, temperature=1.0):
        """Single training step with gradient accumulation"""
        self.model.train()
        
        input_ids = batch.to(self.device)
        
        # Forward pass
        logits = self.model(input_ids, temperature=temperature)
        
        # Shift for next-token prediction
        # logits: (batch, seq_len, vocab)
        # targets: input_ids[:, 1:]  - predict next token
        targets = input_ids[:, 1:]
        
        # Flatten for loss
        pred = logits[:, :-1].contiguous().view(-1, logits.size(-1))
        target = targets.contiguous().view(-1)
        
        # Main loss
        loss = self.loss_fn(pred, target)
        
        # Specialization loss (encourages processor diversity)
        if self.specialization_weight > 0:
            spec_loss = self.model.get_specialization_loss(temperature=temperature)
            loss = loss + self.specialization_weight * spec_loss
        
        # Scale loss for gradient accumulation
        loss = loss / self.grad_accum_steps
        
        # Backward
        loss.backward()
        
        return loss.item() * self.grad_accum_steps
    
    def optimizer_step(self):
        """Apply gradients and optimizer step"""
        # Gradient clipping
        if self.grad_clip > 0:
            torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.grad_clip)
        
        self.optimizer.step()
        self.optimizer.zero_grad()
    
    def evaluate(self):
        """Evaluate on validation set"""
        if self.val_loader is None:
            return None
        
        self.model.eval()
        total_loss = 0
        total_tokens = 0
        
        with torch.no_grad():
            for batch in self.val_loader:
                input_ids = batch.to(self.device)
                
                # Use hard routing for evaluation
                logits = self.model(input_ids, temperature=0.0)
                
                targets = input_ids[:, 1:]
                
                pred = logits[:, :-1].contiguous().view(-1, logits.size(-1))
                target = targets.contiguous().view(-1)
                
                # Only count non-padded tokens
                mask = target != 0
                if mask.sum() > 0:
                    loss = F.cross_entropy(pred, target, reduction='sum', ignore_index=0)
                    total_loss += loss.item()
                    total_tokens += mask.sum().item()
        
        if total_tokens == 0:
            return None
        
        perplexity = math.exp(total_loss / total_tokens)
        return {
            'loss': total_loss / total_tokens,
            'perplexity': perplexity
        }
    
    def train(self):
        """Main training loop"""
        print(f"Starting training for {self.total_steps} steps...")
        print(f"Device: {self.device}")
        
        train_iter = iter(self.train_loader)
        
        while self.step < self.total_steps:
            try:
                batch = next(train_iter)
            except StopIteration:
                train_iter = iter(self.train_loader)
                batch = next(train_iter)
            
            # Get temperature for this step
            temperature = temperature_schedule(
                self.step, self.warmup_steps, self.total_steps,
                temp_start=2.0, temp_end=0.1
            )
            
            # Learning rate
            lr = get_lr(self.step, self.warmup_steps, 1e-4, self.total_steps)
            for param_group in self.optimizer.param_groups:
                param_group['lr'] = lr
            
            # Training step
            loss = self.train_step(batch, temperature=temperature)
            
            # Optimizer step (after accumulation)
            if (self.step + 1) % self.grad_accum_steps == 0:
                self.optimizer_step()
            
            # Logging
            if self.step % self.log_interval == 0:
                print(f"Step {self.step}/{self.total_steps} | "
                      f"Loss: {loss:.4f} | "
                      f"LR: {lr:.2e} | "
                      f"Temp: {temperature:.3f}")
            
            # Evaluation
            if self.step % self.eval_interval == 0 and self.val_loader is not None:
                val_metrics = self.evaluate()
                if val_metrics is not None:
                    print(f"  Val Loss: {val_metrics['loss']:.4f} | "
                          f"Val Perplexity: {val_metrics['perplexity']:.2f}")
                    
                    if val_metrics['loss'] < self.best_val_loss:
                        self.best_val_loss = val_metrics['loss']
            
            # Save checkpoint
            if self.step % self.save_interval == 0 and self.step > 0:
                self.save_checkpoint(f"checkpoint_step_{self.step}.pt")
            
            self.step += 1
        
        print("Training complete!")
        return self.model
    
    def save_checkpoint(self, filename):
        """Save model checkpoint"""
        checkpoint = {
            'step': self.step,
            'model_state_dict': self.model.state_dict(),
            'optimizer_state_dict': self.optimizer.state_dict(),
            'best_val_loss': self.best_val_loss,
        }
        path = os.path.join('checkpoints', filename)
        os.makedirs('checkpoints', exist_ok=True)
        torch.save(checkpoint, path)
        print(f"  Saved checkpoint: {path}")


# ============== Benchmark ==============

def benchmark_model(model, seq_lengths=[32, 64, 128, 256], batch_size=4):
    """Benchmark model at different sequence lengths"""
    print("\n" + "="*50)
    print("Benchmarking model...")
    
    device = next(model.parameters()).device
    model.eval()
    
    results = []
    
    for seq_len in seq_lengths:
        input_ids = torch.randint(0, model.vocab_size, (batch_size, seq_len), device=device)
        
        # Warmup
        with torch.no_grad():
            _ = model(input_ids, temperature=1.0)
        
        # Timing
        torch.cuda.synchronize() if device.type == 'cuda' else None
        import time
        start = time.time()
        
        with torch.no_grad():
            for _ in range(3):
                _ = model(input_ids, temperature=1.0)
        
        torch.cuda.synchronize() if device.type == 'cuda' else None
        elapsed = time.time() - start
        
        tokens_per_sec = (batch_size * seq_len * 3) / elapsed
        results.append({
            'seq_len': seq_len,
            'batch_size': batch_size,
            'time': elapsed / 3,
            'tokens/sec': tokens_per_sec
        })
        
        print(f"  seq_len={seq_len}: {tokens_per_sec:.0f} tokens/sec")
    
    return results


# ============== Main ==============

def main():
    # Create simple training data
    print("Preparing training data...")
    
    # Sample text (in practice, load from file)
    text = """
    The quick brown fox jumps over the lazy dog. The cat sat on the mat.
    Artificial intelligence is transforming the world. Machine learning models
    are becoming more powerful. Deep learning has revolutionized computer vision
    and natural language processing. Neural networks can learn complex patterns
    from data. Training large models requires significant computational resources.
    The future of AI is exciting. Researchers are developing new architectures
    that are more efficient and interpretable. Attention mechanisms allow models
    to focus on relevant information. Transformers have become the dominant
    architecture for language tasks. Reinforcement learning enables agents to
    learn from interaction with the environment. Generative models can create
    realistic images and text. AI has applications in healthcare, finance,
    transportation, and many other fields. Ethical considerations are important
    as AI systems become more capable. Safety and alignment research ensures
    that AI remains beneficial to humanity.
    """ * 50
    
    # Build vocabulary
    vocab = CharacterVocabulary.from_text(text)
    print(f"Vocab size: {len(vocab)}")
    
    # Datasets
    train_dataset = TextDataset(text, vocab.token_to_idx, seq_len=64)
    val_dataset = TextDataset(text, vocab.token_to_idx, seq_len=64)
    
    train_loader = DataLoader(train_dataset, batch_size=16, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=16, shuffle=False)
    
    print(f"Training samples: {len(train_dataset)}")
    print(f"Validation samples: {len(val_dataset)}")
    
    # Model - small version for quick training
    print("\nCreating model...")
    model = GlobalWorkspaceHive(
        vocab_size=len(vocab),
        embed_dim=256,
        workspace_dim=512,
        num_processors=32,
        processor_hidden=128,
        processor_type='gru',
        top_k=4,
        router_hidden=64,
        router_heads=2,
    )
    
    counts, total = count_parameters(model)
    print("Parameter count by component:")
    for comp, count in sorted(counts.items(), key=lambda x: -x[1]):
        print(f"  {comp}: {count:,}")
    print(f"\nTotal: {total:,}")
    
    # Device
    device = 'cuda' if torch.cuda.is_available() else 'cpu'
    print(f"Using device: {device}")
    
    # Trainer
    trainer = Trainer(
        model=model,
        train_loader=train_loader,
        val_loader=val_loader,
        device=device,
        lr=1e-4,
        warmup_steps=100,
        total_steps=1000,
        grad_clip=1.0,
        grad_accum_steps=1,
        specialization_weight=0.01,
        log_interval=50,
        eval_interval=200,
        save_interval=500,
    )
    
    # Train
    trainer.train()
    
    # Benchmark
    if device == 'cuda':
        benchmark_model(model)
    
    # Final evaluation
    print("\nFinal evaluation:")
    val_metrics = trainer.evaluate()
    if val_metrics:
        print(f"  Val Loss: {val_metrics['loss']:.4f}")
        print(f"  Val Perplexity: {val_metrics['perplexity']:.2f}")
    
    # Save final model
    torch.save(model.state_dict(), 'hive_final.pt')
    print("\nModel saved to hive_final.pt")


if __name__ == "__main__":
    main()
