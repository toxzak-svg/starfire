"""Global Workspace Language Model implementation."""

import math
import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import Dict, Optional, Tuple
from dataclasses import dataclass

from .config import GWLConfig


class Processor(nn.Module):
    """A specialized processor module that can read from and write to the workspace.
    
    Each processor has a unique perspective on the input and competes with 
    other processors for access to the workspace.
    """
    
    def __init__(self, config: GWLConfig):
        super().__init__()
        self.config = config
        
        # Input projection: map from embed_dim to processor's internal representation
        self.input_proj = nn.Linear(config.embed_dim, config.processor_config__hidden_dim)
        
        # Processor core based on type
        if config.processor_type == 'mlp':
            self.core = nn.Sequential(
                nn.Linear(config.processor_config__hidden_dim, config.processor_config__hidden_dim),
                nn.GELU(),
                nn.Linear(config.processor_config__hidden_dim, config.processor_config__hidden_dim),
            )
        elif config.processor_type == 'lstm':
            self.core = nn.LSTM(
                config.processor_config__hidden_dim,
                config.processor_config__hidden_dim,
                num_layers=1,
                batch_first=True,
            )
        elif config.processor_type == 'transformer':
            encoder_layer = nn.TransformerEncoderLayer(
                d_model=config.processor_config__hidden_dim,
                nhead=4,
                dim_feedforward=config.processor_config__hidden_dim * 2,
                dropout=config.dropout_prob if config.use_dropout else 0,
                batch_first=True,
            )
            self.core = nn.TransformerEncoder(encoder_layer, num_layers=1)
        
        # Output projection: map from processor hidden dim to workspace dim
        self.output_proj = nn.Linear(config.processor_config__hidden_dim, config.workspace_dim)
        
        # Attention/salience computation
        self.salience = nn.Linear(config.workspace_dim, 1)
        
    def forward(self, x: torch.Tensor, workspace: Optional[torch.Tensor] = None) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Args:
            x: Input embeddings [batch, seq_len, embed_dim]
            workspace: Optional workspace for recurrent processing [batch, workspace_dim]
            
        Returns:
            processor_output: Output to write to workspace [batch, seq_len, workspace_dim]
            salience: Attention weights [batch, seq_len]
        """
        batch_size, seq_len, _ = x.shape
        
        # Project and process
        h = self.input_proj(x)
        
        if self.config.processor_type == 'lstm':
            if workspace is not None:
                # Use workspace as hidden state
                h, _ = self.core(h, (workspace.unsqueeze(0).repeat(1, 1, 1), 
                                      workspace.unsqueeze(0).repeat(1, 1, 1)))
            else:
                h, _ = self.core(h)
        else:
            h = self.core(h)
        
        # Project to workspace dimension
        output = self.output_proj(h)
        
        # Compute salience/attention
        salience = torch.sigmoid(self.salience(output))
        
        return output, salience


class Workspace(nn.Module):
    """The shared workspace that broadcasts information to all processors.
    
    The workspace maintains a persistent state and receives inputs from
    winning processors in each competition step.
    """
    
    def __init__(self, config: GWLConfig):
        super().__init__()
        self.config = config
        
        # Workspace state
        self.state = nn.Parameter(torch.randn(1, config.workspace_dim) * 0.02)
        
        # Update network
        if config.use_workspace_gating:
            self.update_gate = nn.Linear(config.workspace_dim * 2, config.workspace_dim)
            self.update_candidate = nn.Linear(config.workspace_dim * 2, config.workspace_dim)
        
        # Layer norm
        if config.use_layer_norm:
            self.layer_norm = nn.LayerNorm(config.workspace_dim)
        
        # Dropout
        self.dropout = nn.Dropout(config.dropout_prob) if config.use_dropout else nn.Identity()
        
    def forward(self, inputs: torch.Tensor, competition_weights: torch.Tensor) -> torch.Tensor:
        """
        Args:
            inputs: Weighted inputs from processors [batch, num_processors, workspace_dim]
            competition_weights: How much each processor contributes [batch, num_processors]
            
        Returns:
            workspace: Updated workspace state [batch, workspace_dim]
        """
        batch_size = inputs.shape[0]
        
        # Weighted sum of processor outputs
        weighted = (inputs * competition_weights.unsqueeze(-1)).sum(dim=1)
        
        # Expand to batch size
        current_workspace = self.state.expand(batch_size, -1)
        
        if self.config.use_workspace_gating:
            # Gated update
            combined = torch.cat([current_workspace, weighted], dim=-1)
            gate = torch.sigmoid(self.update_gate(combined))
            candidate = torch.tanh(self.update_candidate(combined))
            new_workspace = current_workspace + gate * (candidate - current_workspace)
        else:
            # Direct residual update
            new_workspace = current_workspace + weighted
        
        # Apply normalization and dropout
        if hasattr(self, 'layer_norm'):
            new_workspace = self.layer_norm(new_workspace)
        new_workspace = self.dropout(new_workspace)
        
        return new_workspace


class CompetitionMechanism(nn.Module):
    """Determines which processors get to write to the workspace."""
    
    def __init__(self, config: GWLConfig):
        super().__init__()
        self.config = config
        
    def forward(self, processor_outputs: torch.Tensor, processor_saliences: torch.Tensor) -> torch.Tensor:
        """
        Args:
            processor_outputs: [batch, num_processors, workspace_dim]
            processor_saliences: [batch, num_processors, seq_len]
            
        Returns:
            competition_weights: [batch, num_processors]
        """
        batch_size = processor_outputs.shape[0]
        num_processors = processor_outputs.shape[1]
        
        # Aggregate salience across sequence dimension
        # Take mean or max across seq_len
        salience_agg = processor_saliences.mean(dim=2)  # [batch, num_processors]
        
        # Compute relevance of each processor's output
        relevance = processor_outputs.norm(dim=-1)  # [batch, num_processors]
        
        # Combine salience and relevance
        scores = salience_agg * relevance
        
        # Apply competition type
        if self.config.competition_type == 'softmax':
            weights = F.softmax(scores, dim=-1)
        elif self.config.competition_type == 'sigmoid':
            weights = torch.sigmoid(scores)
            weights = weights / (weights.sum(dim=-1, keepdim=True) + 1e-8)
        elif self.config.competition_type == 'top_k':
            k = min(8, num_processors)
            top_k_vals, top_k_idx = torch.topk(scores, k, dim=-1)
            weights = torch.zeros_like(scores)
            weights.scatter_(1, top_k_idx, F.softmax(top_k_vals, dim=-1))
        else:
            weights = F.softmax(scores, dim=-1)
        
        return weights


class GlobalWorkspaceLM(nn.Module):
    """Global Workspace Language Model.
    
    This model implements Global Workspace Theory where:
    1. Input embeddings are processed by multiple specialized "processor" modules
    2. Processors compete for attention based on their salience
    3. Winning processors write to a shared "workspace"
    4. Workspace state is broadcast back to all processors
    5. This process repeats for several steps
    6. Final workspace state is used for language modeling
    """
    
    def __init__(self, config: GWLConfig):
        super().__init__()
        self.config = config
        
        # Embedding layer
        self.embedding = nn.Embedding(config.vocab_size, config.embed_dim)
        
        # Positional encoding (learnable)
        self.pos_encoding = nn.Parameter(torch.randn(1, 2048, config.embed_dim) * 0.02)
        
        # Processor bank
        self.processors = nn.ModuleList([
            Processor(config) for _ in range(config.num_processors)
        ])
        
        # Competition mechanism
        self.competition = CompetitionMechanism(config)
        
        # Workspace
        self.workspace = Workspace(config)
        
        # Output head
        self.output_proj = nn.Linear(config.workspace_dim, config.vocab_size)
        
        # Tie weights with embedding if possible
        if config.embed_dim == config.workspace_dim:
            self.output_proj.weight = self.embedding.weight
            
        # Dropout on input
        self.input_dropout = nn.Dropout(config.dropout_prob) if config.use_dropout else nn.Identity()
        
    def forward(self, input_ids: torch.Tensor, target_ids: Optional[torch.Tensor] = None) -> Dict[str, Optional[torch.Tensor]]:
        """
        Args:
            input_ids: Input token IDs [batch, seq_len]
            target_ids: Target token IDs for computing loss [batch, seq_len]
            
        Returns:
            Dictionary containing:
                - loss: Cross-entropy loss (if target_ids provided)
                - logits: Output logits [batch, seq_len, vocab_size]
                - workspace: Final workspace state [batch, workspace_dim]
        """
        batch_size, seq_len = input_ids.shape
        
        # Embedding
        x = self.embedding(input_ids)
        
        # Add positional encoding
        if seq_len <= self.pos_encoding.shape[1]:
            x = x + self.pos_encoding[:, :seq_len, :]
        else:
            # Fallback if sequence too long
            x = x + self.pos_encoding[:, :self.pos_encoding.shape[1], :]
        
        x = self.input_dropout(x)
        
        # Initialize workspace
        workspace = self.workspace.state.expand(batch_size, -1)
        
        # Iterative competition and workspace update
        for step in range(self.config.num_steps):
            # Process input through all processors
            processor_outputs = []
            processor_saliences = []
            
            for processor in self.processors:
                output, salience = processor(x, workspace)
                processor_outputs.append(output)
                processor_saliences.append(salience)
            
            # Stack along processor dimension
            processor_outputs = torch.stack(processor_outputs, dim=1)  # [batch, num_procs, seq_len, workspace_dim]
            processor_saliences = torch.stack(processor_saliences, dim=1)  # [batch, num_procs, seq_len]
            
            # Aggregate across sequence for competition
            # Use mean pooling to get per-processor scores
            proc_output_mean = processor_outputs.mean(dim=2)  # [batch, num_procs, workspace_dim]
            
            # Compute competition weights
            competition_weights = self.competition(proc_output_mean, processor_saliences)
            
            # Weighted sum of processor outputs
            workspace_input = (processor_outputs.mean(dim=2) * competition_weights.unsqueeze(-1)).sum(dim=1)
            
            # Update workspace
            workspace = self.workspace(processor_outputs.mean(dim=2).permute(0, 2, 1).reshape(batch_size, -1), 
                                       competition_weights)
            # Fix: need to reshape properly
            proc_outputs_agg = processor_outputs.mean(dim=2)  # [batch, num_procs, workspace_dim]
            workspace = self.workspace(proc_outputs_agg.reshape(batch_size, -1), competition_weights)
        
        # Use final workspace for language modeling
        # Expand workspace to match sequence length
        workspace_expanded = workspace.unsqueeze(1).expand(-1, seq_len, -1)
        
        # Combine input with workspace
        combined = x + workspace_expanded
        
        # Project to vocabulary
        logits = self.output_proj(combined)
        
        loss = None
        if target_ids is not None:
            # Shift for next-token prediction
            # logits[:, :-1, :] should predict target_ids[:, 1:]
            # But for simplicity, use standard causal LM loss
            loss_fct = nn.CrossEntropyLoss()
            # Flatten for loss computation
            logits_flat = logits.view(-1, self.config.vocab_size)
            targets_flat = target_ids.view(-1)
            loss = loss_fct(logits_flat, targets_flat)
        
        return {
            "loss": loss,
            "logits": logits,
            "workspace": workspace,
        }
    
    def generate(self, prompt_ids: torch.Tensor, max_new_tokens: int = 50, 
                 temperature: float = 1.0, top_k: int = 50) -> torch.Tensor:
        """
        Generate text given a prompt.
        
        Args:
            prompt_ids: Input token IDs [batch, seq_len]
            max_new_tokens: Maximum number of tokens to generate
            temperature: Sampling temperature (higher = more random)
            top_k: Top-k sampling parameter
            
        Returns:
            Generated token IDs including prompt
        """
        self.eval()
        
        with torch.no_grad():
            generated = prompt_ids
            
            for _ in range(max_new_tokens):
                # Forward pass
                outputs = self.forward(generated)
                logits = outputs["logits"]
                
                # Get logits for last position
                next_token_logits = logits[:, -1, :] / temperature
                
                # Top-k filtering
                if top_k > 0:
                    v, _ = torch.topk(next_token_logits, min(top_k, next_token_logits.size(-1)))
                    next_token_logits[next_token_logits < v[:, [-1]]] = float('-inf')
                
                # Sample
                probs = F.softmax(next_token_logits, dim=-1)
                next_token = torch.multinomial(probs, num_samples=1)
                
                # Append
                generated = torch.cat([generated, next_token], dim=1)
                
                # Stop if all sequences hit EOS (if we had one)
                # For now, just generate max_new_tokens
                
        return generated