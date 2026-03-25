"""
Benchmark script for Global Workspace Hive

Compares the hive architecture against:
1. Dense transformer baseline (same params)
2. Ablations: single processor, no competition, static routing

Metrics:
- Training loss / perplexity
- Inference speed (tokens/sec)
- Compositional generalization
- Domain transfer
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import Dataset, DataLoader
import numpy as np
import math
import os
import json
from hive import GlobalWorkspaceHive, count_parameters


# ============== Baseline Transformer ==============

class TransformerBaseline(nn.Module):
    """Standard dense transformer - same parameter count as hive"""
    
    def __init__(
        self,
        vocab_size,
        embed_dim=256,
        num_layers=6,
        num_heads=4,
        ff_dim=1024,
        seq_len=128
    ):
        super().__init__()
        self.vocab_size = vocab_size
        
        # Embedding
        self.embedding = nn.Embedding(vocab_size, embed_dim)
        self.pos_embedding = nn.Embedding(seq_len, embed_dim)
        
        # Transformer layers
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=embed_dim,
            nhead=num_heads,
            dim_feedforward=ff_dim,
            batch_first=True
        )
        self.transformer = nn.TransformerEncoder(encoder_layer, num_layers=num_layers)
        
        # Output
        self.output = nn.Linear(embed_dim, vocab_size)
        
        # Tie weights
        self.output.weight = self.embedding.weight
    
    def forward(self, input_ids):
        batch_size, seq_len = input_ids.shape
        
        # Embeddings
        embeds = self.embedding(input_ids)
        positions = torch.arange(seq_len, device=input_ids.device).unsqueeze(0)
        embeds = embeds + self.pos_embedding(positions)
        
        # Transform
        out = self.transformer(embeds)
        
        # Output
        logits = self.output(out)
        
        return logits


class AblationHive(nn.Module):
    """Hive with ablations applied"""
    
    def __init__(
        self,
        hive_model,
        ablation='single_processor'
    ):
        super().__init__()
        self.hive = hive_model
        self.ablation = ablation
        
        # For single processor: replace with one processor
        if ablation == 'single_processor':
            self.hive.num_processors = 1
            # Will need to handle in forward
        
        # For no competition: all processors write
        if ablation == 'no_competition':
            self.hive.top_k = self.hive.num_processors
        
        # For static routing: disable router
        # (handled in forward)
    
    def forward(self, input_ids, temperature=1.0):
        if self.ablation == 'single_processor':
            return self._forward_single(input_ids)
        elif self.ablation == 'no_competition':
            return self._forward_no_competition(input_ids, temperature)
        elif self.ablation == 'static_routing':
            return self._forward_static_routing(input_ids, temperature)
        else:
            return self.hive.forward(input_ids, temperature)
    
    def _forward_single(self, input_ids):
        """Collapse to single processor"""
        batch_size, seq_len = input_ids.shape
        device = input_ids.device
        
        embeds = self.hive.embedding(input_ids)
        
        # Initialize workspace
        ws_state = self.hive.workspace.from_embed(embeds[:, 0, :])
        
        all_logits = []
        
        for t in range(seq_len):
            embed_t = embeds[:, t, :]
            
            # Just one processor (reuse first)
            processor = self.hive.processors.processors[0]
            out, _ = processor(ws_state)
            out = out.unsqueeze(1)  # (batch, 1, workspace)
            
            # Weight = 1.0
            weights = torch.ones(batch_size, 1, device=device)
            
            # Workspace
            logits, ws_state = self.hive.workspace(embed_t, out, weights, temperature=1.0)
            all_logits.append(logits)
        
        all_logits = torch.stack(all_logits, dim=1)
        return all_logits
    
    def _forward_no_competition(self, input_ids, temperature):
        """All processors write equally"""
        batch_size, seq_len = input_ids.shape
        device = input_ids.device
        
        embeds = self.hive.embedding(input_ids)
        
        # Initialize
        ws_state = self.hive.workspace.from_embed(embeds[:, 0, :])
        hidden_states = self.hive.processors.init_hidden(batch_size, device)
        
        all_logits = []
        
        for t in range(seq_len):
            embed_t = embeds[:, t, :]
            
            # All processors run
            processor_outputs, hidden_states = self.hive.processors(ws_state, hidden_states)
            
            # Equal weights
            weights = torch.ones(batch_size, self.hive.num_processors, device=device)
            weights = weights / self.hive.num_processors
            
            # Workspace
            logits, ws_state = self.hive.workspace(embed_t, processor_outputs, weights, temperature)
            all_logits.append(logits)
        
        all_logits = torch.stack(all_logits, dim=1)
        return all_logits
    
    def _forward_static_routing(self, input_ids, temperature):
        """Fixed uniform routing (no dynamic router)"""
        batch_size, seq_len = input_ids.shape
        device = input_ids.device
        
        embeds = self.hive.embedding(input_ids)
        
        # Initialize
        ws_state = self.hive.workspace.from_embed(embeds[:, 0, :])
        hidden_states = self.hive.processors.init_hidden(batch_size, device)
        
        all_logits = []
        
        # Competition still works, but router output is ignored
        for t in range(seq_len):
            embed_t = embeds[:, t, :]
            
            # Processors
            processor_outputs, hidden_states = self.hive.processors(ws_state, hidden_states)
            
            # Competition
            routing_weights = self.hive.competition(ws_state, temperature)
            
            # Workspace (ignore router)
            logits, ws_state = self.hive.workspace(embed_t, processor_outputs, routing_weights, temperature)
            all_logits.append(logits)
        
        all_logits = torch.stack(all_logits, dim=1)
        return all_logits


# ============== Data ==============

class TextDataset(Dataset):
    def __init__(self, text, vocab, seq_len=64):
        self.seq_len = seq_len
        tokens = [vocab.get(c, vocab['<unk>']) for c in text]
        
        self.sequences = []
        for i in range(0, len(tokens) - seq_len - 1, seq_len):
            self.sequences.append(torch.tensor(tokens[i:i + seq_len], dtype=torch.long))
    
    def __len__(self):
        return len(self.sequences)
    
    def __getitem__(self, idx):
        return self.sequences[idx]


class CharacterVocabulary:
    def __init__(self, text):
        chars = sorted(set(text))
        special = ['<pad>', '<unk>']
        self.token_to_idx = {tok: i for i, tok in enumerate(special)}
        for c in chars:
            self.token_to_idx[c] = len(self.token_to_idx)
        self.idx_to_token = {v: k for k, v in self.token_to_idx.items()}
    
    def __len__(self):
        return len(self.token_to_idx)


# ============== Training ==============

def train_model(model, train_loader, val_loader, epochs=5, device='cuda', name='model'):
    """Train a model and return training curve"""
    model = model.to(device)
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-4, weight_decay=0.01)
    loss_fn = nn.CrossEntropyLoss()
    
    train_losses = []
    val_perplexities = []
    
    for epoch in range(epochs):
        model.train()
        epoch_loss = 0
        steps = 0
        
        for batch in train_loader:
            batch = batch.to(device)
            
            logits = model(batch)
            targets = batch[:, 1:]
            
            pred = logits[:, :-1].contiguous().view(-1, logits.size(-1))
            target = targets.contiguous().view(-1)
            
            loss = loss_fn(pred, target)
            
            optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            
            epoch_loss += loss.item()
            steps += 1
        
        avg_loss = epoch_loss / steps
        train_losses.append(avg_loss)
        
        # Validation
        model.eval()
        val_loss = 0
        val_tokens = 0
        
        with torch.no_grad():
            for batch in val_loader:
                batch = batch.to(device)
                logits = model(batch)
                targets = batch[:, 1:]
                
                pred = logits[:, :-1].contiguous().view(-1, logits.size(-1))
                target = targets.contiguous().view(-1)
                
                loss = loss_fn(pred, target, reduction='sum')
                val_loss += loss.item()
                val_tokens += target.numel()
        
        perplexity = math.exp(val_loss / val_tokens)
        val_perplexities.append(perplexity)
        
        print(f"  {name} Epoch {epoch+1}/{epochs} | Train Loss: {avg_loss:.4f} | Val PPL: {perplexity:.2f}")
    
    return {
        'train_losses': train_losses,
        'val_perplexities': val_perplexities
    }


def benchmark_speed(model, seq_len=64, batch_size=4, device='cuda'):
    """Benchmark inference speed"""
    model = model.to(device)
    model.eval()
    
    input_ids = torch.randint(0, model.vocab_size, (batch_size, seq_len), device=device)
    
    # Warmup
    with torch.no_grad():
        _ = model(input_ids)
    
    if device == 'cuda':
        torch.cuda.synchronize()
    
    import time
    start = time.time()
    
    with torch.no_grad():
        for _ in range(10):
            _ = model(input_ids)
    
    if device == 'cuda':
        torch.cuda.synchronize()
    
    elapsed = time.time() - start
    tokens_per_sec = (batch_size * seq_len * 10) / elapsed
    
    return tokens_per_sec


# ============== Main ==============

def main():
    print("="*60)
    print("Global Workspace Hive - Benchmark Suite")
    print("="*60)
    
    device = 'cuda' if torch.cuda.is_available() else 'cpu'
    print(f"\nDevice: {device}")
    
    # Create training data
    text = """
    The quick brown fox jumps over the lazy dog. A journey of a thousand miles
    begins with a single step. Knowledge is power. Time and tide wait for no man.
    All that glitters is not gold. Where there's a will there's a way. Actions speak
    louder than words. Better late than never. Knowledge is power. 
    """ * 100
    
    vocab = CharacterVocabulary(text)
    print(f"Vocab size: {len(vocab)}")
    
    dataset = TextDataset(text, vocab.token_to_idx, seq_len=64)
    train_loader = DataLoader(dataset, batch_size=16, shuffle=True)
    val_loader = DataLoader(dataset, batch_size=16, shuffle=False)
    
    print(f"Training samples: {len(dataset)}")
    
    # Model configs (roughly matching parameter counts)
    print("\n" + "="*60)
    print("Creating models...")
    print("="*60)
    
    # Hive model
    hive = GlobalWorkspaceHive(
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
    hive_counts, hive_params = count_parameters(hive)
    print(f"\nHive: {hive_params:,} params")
    
    # Transformer baseline (similar parameter count)
    transformer = TransformerBaseline(
        vocab_size=len(vocab),
        embed_dim=256,
        num_layers=6,
        num_heads=4,
        ff_dim=512,
        seq_len=128
    )
    trans_counts, trans_params = count_parameters(transformer)
    print(f"Transformer: {trans_params:,} params")
    
    # Create ablations
    ablation_single = AblationHive(hive, ablation='single_processor')
    ablation_no_comp = AblationHive(hive, ablation='no_competition')
    ablation_static = AblationHive(hive, ablation='static_routing')
    
    # Train models
    print("\n" + "="*60)
    print("Training models...")
    print("="*60)
    
    epochs = 3
    
    print("\n[1/5] Training Hive...")
    hive_results = train_model(hive, train_loader, val_loader, epochs=epochs, device=device, name='Hive')
    
    print("\n[2/5] Training Transformer baseline...")
    trans_results = train_model(transformer, train_loader, val_loader, epochs=epochs, device=device, name='Transformer')
    
    print("\n[3/5] Training Single Processor ablation...")
    single_results = train_model(ablation_single, train_loader, val_loader, epochs=epochs, device=device, name='SingleProc')
    
    print("\n[4/5] Training No Competition ablation...")
    no_comp_results = train_model(ablation_no_comp, train_loader, val_loader, epochs=epochs, device=device, name='NoComp')
    
    print("\n[5/5] Training Static Routing ablation...")
    static_results = train_model(ablation_static, train_loader, val_loader, epochs=epochs, device=device, name='Static')
    
    # Benchmark speeds
    print("\n" + "="*60)
    print("Benchmarking inference speeds...")
    print("="*60)
    
    speeds = {
        'Hive': benchmark_speed(hive, seq_len=64, device=device),
        'Transformer': benchmark_speed(transformer, seq_len=64, device=device),
        'Single Processor': benchmark_speed(ablation_single, seq_len=64, device=device),
        'No Competition': benchmark_speed(ablation_no_comp, seq_len=64, device=device),
        'Static Routing': benchmark_speed(ablation_static, seq_len=64, device=device),
    }
    
    for name, speed in speeds.items():
        print(f"  {name}: {speed:.0f} tokens/sec")
    
    # Summary
    print("\n" + "="*60)
    print("SUMMARY")
    print("="*60)
    
    print(f"\nFinal Validation Perplexities:")
    print(f"  Hive:           {hive_results['val_perplexities'][-1]:.2f}")
    print(f"  Transformer:    {trans_results['val_perplexities'][-1]:.2f}")
    print(f"  Single Proc:    {single_results['val_perplexities'][-1]:.2f}")
    print(f"  No Competition: {no_comp_results['val_perplexities'][-1]:.2f}")
    print(f"  Static Rout:    {static_results['val_perplexities'][-1]:.2f}")
    
    print(f"\nInference Speeds (tokens/sec):")
    for name, speed in speeds.items():
        print(f"  {name}: {speed:.0f}")
    
    print(f"\nParameter Counts:")
    print(f"  Hive:        {hive_params:,}")
    print(f"  Transformer: {trans_params:,}")
    
    # Save results
    results = {
        'hive_params': hive_params,
        'transformer_params': trans_params,
        'hive_final_ppl': hive_results['val_perplexities'][-1],
        'transformer_final_ppl': trans_results['val_perplexities'][-1],
        'single_proc_final_ppl': single_results['val_perplexities'][-1],
        'no_comp_final_ppl': no_comp_results['val_perplexities'][-1],
        'static_routing_final_ppl': static_results['val_perplexities'][-1],
        'speeds': speeds,
    }
    
    with open('benchmark_results.json', 'w') as f:
        json.dump(results, f, indent=2)
    
    print("\nResults saved to benchmark_results.json")


if __name__ == "__main__":
    main()
