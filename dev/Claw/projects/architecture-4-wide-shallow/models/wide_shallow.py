"""
Wide and Shallow Transformer Model Implementation

Core architecture: 8K-16K attention heads across 2-4 layers.
Each head has its own MLP projection head.
Uses grouped-query attention to reduce KV overhead.
Heavy residual connections for information flow.
"""

import math
from typing import Optional, Tuple
import torch
import torch.nn as nn
import torch.nn.functional as F
from einops import rearrange


class GroupedQueryAttention(nn.Module):
    """
    Grouped Query Attention (GQA) for massively wide heads.
    
    Instead of having separate KV for each head, we share KV across groups.
    num_heads: 8192 (8K) - number of query heads
    num_kv_heads: 64 - number of key/value heads  
    groups_per_kv: 128 - how many query heads share each KV head
    """
    
    def __init__(
        self,
        hidden_size: int,
        num_heads: int,
        num_kv_heads: int,
        head_dim: int = 64,
        dropout: float = 0.0,
    ):
        super().__init__()
        
        self.hidden_size = hidden_size
        self.num_heads = num_heads
        self.num_kv_heads = num_kv_heads
        self.head_dim = head_dim
        self.groups_per_kv = num_heads // num_kv_heads
        self.dropout = dropout
        
        # Ensure heads divide evenly
        assert num_heads % num_kv_heads == 0, \
            f"num_heads ({num_heads}) must be divisible by num_kv_heads ({num_kv_heads})"
        
        # Q projection - massive for 8K heads
        self.q_proj = nn.Linear(hidden_size, num_heads * head_dim, bias=False)
        
        # KV projection - much smaller due to grouping
        self.k_proj = nn.Linear(hidden_size, num_kv_heads * head_dim, bias=False)
        self.v_proj = nn.Linear(hidden_size, num_kv_heads * head_dim, bias=False)
        
        # Output projection
        self.o_proj = nn.Linear(num_heads * head_dim, hidden_size, bias=False)
        
        # Dropout
        self.attn_dropout = nn.Dropout(dropout)
        
    def forward(
        self,
        hidden_states: torch.Tensor,
        attention_mask: Optional[torch.Tensor] = None,
        position_embeddings: Optional[torch.Tensor] = None,
        past_key_value: Optional[Tuple[torch.Tensor, torch.Tensor]] = None,
        use_cache: bool = False,
    ) -> Tuple[torch.Tensor, Optional[Tuple[torch.Tensor, torch.Tensor]]]:
        """
        Args:
            hidden_states: [batch, seq_len, hidden_size]
            attention_mask: [batch, 1, seq_len, seq_len] or [batch, seq_len]
            position_embeddings: [1, seq_len, head_dim] RoPE embeddings
            past_key_value: Tuple of (past_keys, past_values) for KV cache
            use_cache: Whether to return KV cache for inference
            
        Returns:
            output: [batch, seq_len, hidden_size]
            present_key_value: Optional tuple of (keys, values) for caching
        """
        batch_size, seq_len, _ = hidden_states.shape
        
        # Project to Q, K, V
        # Q: [batch, seq_len, num_heads * head_dim]
        # K, V: [batch, seq_len, num_kv_heads * head_dim]
        q = self.q_proj(hidden_states)
        k = self.k_proj(hidden_states)
        v = self.v_proj(hidden_states)
        
        # Reshape Q: [batch, seq_len, num_heads, head_dim]
        q = q.view(batch_size, seq_len, self.num_heads, self.head_dim)
        # Reshape K, V: [batch, seq_len, num_kv_heads, head_dim]
        k = k.view(batch_size, seq_len, self.num_kv_heads, self.head_dim)
        v = v.view(batch_size, seq_len, self.num_kv_heads, self.head_dim)
        
        # Handle KV cache
        if past_key_value is not None:
            past_keys, past_values = past_key_value
            k = torch.cat([past_keys, k], dim=1)
            v = torch.cat([past_values, v], dim=1)
        
        # Store cache for inference
        present_key_value = (k, v) if use_cache else None
        
        # Expand K, V to match Q heads via broadcasting
        # K, V: [batch, kv_seq_len, num_kv_heads, head_dim] 
        # -> [batch, kv_seq_len, num_heads, head_dim]
        if seq_len > 1 or past_key_value is not None:
            # Repeat KV heads for each group
            k = repeat_kv(k, self.groups_per_kv)  # [batch, kv_seq_len, num_heads, head_dim]
            v = repeat_kv(v, self.groups_per_kv)
        
        # Apply position embeddings (RoPE)
        if position_embeddings is not None:
            # position_embeddings: [1, seq_len, head_dim]
            # Apply rotation using complex number trick
            q = apply_rotary_pos_emb(q, position_embeddings)
            k = apply_rotary_pos_emb(k, position_embeddings)
        
        # Compute attention scores
        # Q: [batch, num_heads, seq_len, head_dim]
        # K: [batch, num_heads, kv_seq_len, head_dim]
        q = q.transpose(1, 2)
        k = k.transpose(1, 2)
        v = v.transpose(1, 2)
        
        # Attention: [batch, num_heads, seq_len, kv_seq_len]
        attn_weights = torch.matmul(q, k.transpose(-2, -1)) / math.sqrt(self.head_dim)
        
        # Apply attention mask
        if attention_mask is not None:
            if attention_mask.dim() == 2:
                # [batch, seq_len] -> [batch, 1, seq_len, seq_len]
                attention_mask = attention_mask.unsqueeze(1).unsqueeze(2)
            elif attention_mask.dim() == 3:
                # [batch, seq_len, kv_seq_len] -> [batch, 1, seq_len, kv_seq_len]
                attention_mask = attention_mask.unsqueeze(1)
            
            # Convert to large negative for masked positions
            attention_mask = (1.0 - attention_mask) * torch.finfo(attn_weights.dtype).min
            attn_weights = attn_weights + attention_mask
        
        # Softmax and dropout
        attn_weights = F.softmax(attn_weights, dim=-1)
        attn_weights = self.attn_dropout(attn_weights)
        
        # Apply attention to values
        # attn: [batch, num_heads, seq_len, kv_seq_len]
        # v: [batch, num_heads, kv_seq_len, head_dim]
        # out: [batch, num_heads, seq_len, head_dim]
        attn_output = torch.matmul(attn_weights, v)
        
        # Reshape: [batch, seq_len, num_heads, head_dim]
        attn_output = attn_output.transpose(1, 2).contiguous()
        attn_output = attn_output.reshape(batch_size, seq_len, self.num_heads * self.head_dim)
        
        # Final projection
        output = self.o_proj(attn_output)
        
        return output, present_key_value


def repeat_kv(x: torch.Tensor, n_rep: int) -> torch.Tensor:
    """Repeat KV heads to match number of Q heads."""
    batch_size, seq_len, num_kv_heads, head_dim = x.shape
    if n_rep == 1:
        return x
    return (
        x.unsqueeze(3)
        .expand(batch_size, seq_len, num_kv_heads, n_rep, head_dim)
        .reshape(batch_size, seq_len, num_kv_heads * n_rep, head_dim)
    )


def apply_rotary_pos_emb(q: torch.Tensor, cos: torch.Tensor, sin: torch.Tensor) -> torch.Tensor:
    """Apply rotary position embeddings to query and keys."""
    # q: [batch, num_heads, seq_len, head_dim]
    # cos, sin: [1, seq_len, head_dim]
    # Using complex number representation
    # Alternatively use the simpler form:
    q_real = q[..., : q.shape[-1] // 2]
    q_imag = q[..., q.shape[-1] // 2:]
    
    # Reshape cos/sin for broadcasting: [1, 1, seq_len, head_dim]
    cos = cos.unsqueeze(0).unsqueeze(0)
    sin = sin.unsqueeze(0).unsqueeze(0)
    
    # (q_real + i*q_imag) * (cos + i*sin) = 
    # q_real*cos - q_imag*sin + i*(q_real*sin + q_imag*cos)
    out_real = q_real * cos - q_imag * sin
    out_imag = q_real * sin + q_imag * cos
    
    return torch.cat([out_real, out_imag], dim=-1)


class HeadwiseMLP(nn.Module):
    """
    Headwise MLP projection - each attention head gets its own MLP.
    
    This is different from standard transformer FFN where there's one 
    large MLP shared across all heads. Here we have smaller MLPs per head.
    
    This is more parameter-efficient for the wide architecture since
    we don't need to project all the way to intermediate_size per head.
    """
    
    def __init__(
        self,
        hidden_size: int,
        num_heads: int,
        head_dim: int = 64,
        intermediate_size: Optional[int] = None,
        activation: str = "silu",
    ):
        super().__init__()
        
        self.hidden_size = hidden_size
        self.num_heads = num_heads
        self.head_dim = head_dim
        self.intermediate_size = intermediate_size or hidden_size
        
        # Project down to per-head dimension, then up
        # This keeps parameters manageable with 8K heads
        self.gate_proj = nn.Linear(hidden_size, num_heads * head_dim, bias=False)
        self.up_proj = nn.Linear(hidden_size, num_heads * head_dim, bias=False)
        self.down_proj = nn.Linear(num_heads * head_dim, hidden_size, bias=False)
        
        self.act_fn = nn.SiLU() if activation == "silu" else nn.GELU()
        
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Args:
            x: [batch, seq_len, hidden_size]
        Returns:
            [batch, seq_len, hidden_size]
        """
        # Project to per-head space: [batch, seq_len, num_heads * head_dim]
        gate = self.gate_proj(x)
        up = self.up_proj(x)
        
        # Apply activation
        gate = self.act_fn(gate)
        
        # Element-wise multiply
        intermediate = gate * up
        
        # Project back
        output = self.down_proj(intermediate)
        
        return output


class WideShallowDecoderLayer(nn.Module):
    """
    Single decoder layer for the wide and shallow transformer.
    
    Contains:
    - Grouped Query Attention (8K+ heads)
    - Headwise MLP
    - Pre-norm residual connections
    """
    
    def __init__(
        self,
        hidden_size: int,
        num_heads: int,
        num_kv_heads: int,
        head_dim: int = 64,
        intermediate_size: Optional[int] = None,
        dropout: float = 0.0,
        activation: str = "silu",
        layer_norm_eps: float = 1e-5,
    ):
        super().__init__()
        
        self.hidden_size = hidden_size
        self.num_heads = num_heads
        
        # Self-attention with grouped query attention
        self.self_attn = GroupedQueryAttention(
            hidden_size=hidden_size,
            num_heads=num_heads,
            num_kv_heads=num_kv_heads,
            head_dim=head_dim,
            dropout=dropout,
        )
        
        # Headwise MLP - each head gets projection capability
        self.mlp = HeadwiseMLP(
            hidden_size=hidden_size,
            num_heads=num_heads,
            head_dim=head_dim,
            intermediate_size=intermediate_size,
            activation=activation,
        )
        
        # Pre-norm residual layers
        self.input_layernorm = nn.LayerNorm(hidden_size, eps=layer_norm_eps)
        self.post_attention_layernorm = nn.LayerNorm(hidden_size, eps=layer_norm_eps)
        
    def forward(
        self,
        hidden_states: torch.Tensor,
        attention_mask: Optional[torch.Tensor] = None,
        position_embeddings: Optional[torch.Tensor] = None,
        past_key_value: Optional[Tuple[torch.Tensor, torch.Tensor]] = None,
        use_cache: bool = False,
    ) -> Tuple[torch.Tensor, Optional[Tuple[torch.Tensor, torch.Tensor]]]:
        """
        Args:
            hidden_states: [batch, seq_len, hidden_size]
        Returns:
            output: [batch, seq_len, hidden_size]
            present_key_value: Optional KV cache
        """
        # Pre-norm architecture
        # First: self-attention with residual
        residual = hidden_states
        hidden_states = self.input_layernorm(hidden_states)
        
        # Self-attention
        attn_output, present_key_value = self.self_attn(
            hidden_states=hidden_states,
            attention_mask=attention_mask,
            position_embeddings=position_embeddings,
            past_key_value=past_key_value,
            use_cache=use_cache,
        )
        
        # Residual connection
        hidden_states = residual + attn_output
        
        # Second: MLP with residual
        residual = hidden_states
        hidden_states = self.post_attention_layernorm(hidden_states)
        
        mlp_output = self.mlp(hidden_states)
        hidden_states = residual + mlp_output
        
        return hidden_states, present_key_value


class WideShallowRotaryEmbedding(nn.Module):
    """
    Rotary Position Embedding (RoPE) for the wide architecture.
    
    Pre-computed embeddings for efficiency with long sequences.
    """
    
    def __init__(self, head_dim: int, max_seq_len: int = 4096, base: int = 10000):
        super().__init__()
        self.head_dim = head_dim
        self.max_seq_len = max_seq_len
        self.base = base
        
        # Create position indices
        inv_freq = 1.0 / (self.base ** (torch.arange(0, head_dim, 2).float() / head_dim))
        self.register_buffer("inv_freq", inv_freq, persistent=False)
        
        # Compute embeddings
        self._set_cos_sin_cache(max_seq_len)
        
    def _set_cos_sin_cache(self, seq_len: int):
        self.max_seq_len_cached = seq_len
        t = torch.arange(self.max_seq_len_cached, device=self.inv_freq.device)
        freqs = torch.einsum("i,j->ij", t, self.inv_freq)  # [seq_len, head_dim/2]
        emb = torch.cat([freqs, freqs], dim=-1)  # [seq_len, head_dim]
        
        self.register_buffer("cos_cached", emb.cos(), persistent=False)
        self.register_buffer("sin_cached", emb.sin(), persistent=False)
        
    def forward(self, seq_len: int, device: torch.device):
        if seq_len > self.max_seq_len_cached:
            self._set_cos_sin_cache(seq_len)
        
        return (
            self.cos_cached[:seq_len].to(device),
            self.sin_cached[:seq_len].to(device),
        )


class WideShallowTransformer(nn.Module):
    """
    Wide and Shallow Transformer Model
    
    Core architecture:
    - 8K-16K attention heads across 2-4 layers
    - Grouped-query attention to reduce KV overhead
    - Each head has its own MLP projection head
    - Heavy residual connections for information flow
    
    This goes against the trend of deeper models - instead, it captures
    local patterns directly without repeated refinement.
    """
    
    def __init__(
        self,
        vocab_size: int = 50257,
        num_layers: int = 4,
        num_heads: int = 32,
        num_kv_heads: int = 8,
        hidden_size: int = 4096,
        intermediate_size: Optional[int] = None,
        head_dim: Optional[int] = None,
        max_position_embeddings: int = 4096,
        dropout: float = 0.0,
        embed_dropout: float = 0.0,
        activation: str = "silu",
        layer_norm_eps: float = 1e-5,
        use_cache: bool = True,
    ):
        super().__init__()
        
        self.vocab_size = vocab_size
        self.num_layers = num_layers
        self.num_heads = num_heads
        self.num_kv_heads = num_kv_heads
        self.hidden_size = hidden_size
        # Compute head_dim from hidden_size and num_heads if not provided
        self.head_dim = head_dim or (hidden_size // num_heads)
        self.max_position_embeddings = max_position_embeddings
        self.use_cache = use_cache
        
        # Recompute hidden_size if head_dim was provided
        if head_dim:
            self.hidden_size = num_heads * head_dim
        
        # Token embeddings
        self.embed_tokens = nn.Embedding(vocab_size, hidden_size)
        self.embed_dropout = nn.Dropout(embed_dropout)
        
        # Rotary embeddings
        self.rotary_emb = WideShallowRotaryEmbedding(
            head_dim=self.head_dim,
            max_seq_len=max_position_embeddings,
        )
        
        # Stack of decoder layers
        self.layers = nn.ModuleList([
            WideShallowDecoderLayer(
                hidden_size=hidden_size,
                num_heads=num_heads,
                num_kv_heads=num_kv_heads,
                head_dim=self.head_dim,
                intermediate_size=intermediate_size,
                dropout=dropout,
                activation=activation,
                layer_norm_eps=layer_norm_eps,
            )
            for _ in range(num_layers)
        ])
        
        # Final layer norm
        self.norm = nn.LayerNorm(hidden_size, eps=layer_norm_eps)
        
        # Output head
        self.lm_head = nn.Linear(hidden_size, vocab_size, bias=False)
        
        # Tie weights with embeddings
        self.lm_head.weight = self.embed_tokens.weight
        
        # Initialize weights
        self._init_weights()
        
    def _init_weights(self):
        """Initialize weights - critical for wide architectures."""
        # Use small_init for wide layers to prevent collapse
        for module in self.modules():
            if isinstance(module, nn.Linear):
                # Small init - scales with inverse of width
                std = (self.hidden_size ** -0.5) 
                if hasattr(module, 'mlp_to_gate'):
                    std *= (2 * self.num_layers) ** -0.5
                nn.init.normal_(module.weight, std=std)
                if module.bias is not None:
                    nn.init.zeros_(module.bias)
            elif isinstance(module, nn.Embedding):
                nn.init.normal_(module.weight, std=0.02)
                
    def forward(
        self,
        input_ids: torch.Tensor,
        attention_mask: Optional[torch.Tensor] = None,
        labels: Optional[torch.Tensor] = None,
        past_key_values: Optional[Tuple[Tuple[torch.Tensor, torch.Tensor], ...]] = None,
        use_cache: bool = False,
    ) -> dict:
        """
        Args:
            input_ids: [batch, seq_len]
            attention_mask: [batch, seq_len]
            labels: [batch, seq_len] for language modeling
            past_key_values: Cached KV for autoregressive generation
            use_cache: Whether to use caching for inference
            
        Returns:
            Dictionary with loss and logits
        """
        batch_size, seq_len = input_ids.shape
        
        # Get embeddings
        hidden_states = self.embed_tokens(input_ids)
        hidden_states = self.embed_dropout(hidden_states)
        
        # Create causal attention mask
        if attention_mask is None:
            attention_mask = torch.ones_like(input_ids)
        
        # Create proper 4D attention mask for causal LM
        # [batch, 1, seq_len, seq_len]
        causal_mask = torch.triu(
            torch.ones(seq_len, seq_len, device=input_ids.device, dtype=torch.bool),
            diagonal=1
        ).unsqueeze(0).unsqueeze(0)
        
        # Combine with explicit attention mask
        if attention_mask is not None:
            # Convert to proper shape [batch, 1, seq_len, seq_len]
            mask_expanded = attention_mask.unsqueeze(1).unsqueeze(2)
            # 1 = attend, 0 = don't attend
            combined_mask = torch.where(mask_expanded & ~causal_mask, 0.0, 1.0)
        else:
            combined_mask = torch.where(~causal_mask, 0.0, 1.0)
        
        # Get position embeddings
        cos, sin = self.rotary_emb(seq_len, hidden_states.device)
        
        # Process through layers
        present_key_values = [] if use_cache else None
        for i, layer in enumerate(self.layers):
            # Get past key values for this layer if caching
            layer_past = None
            if past_key_values is not None:
                layer_past = past_key_values[i]
            
            hidden_states, present_kv = layer(
                hidden_states=hidden_states,
                attention_mask=combined_mask,
                position_embeddings=(cos, sin),
                past_key_value=layer_past,
                use_cache=use_cache,
            )
            
            if use_cache and present_kv is not None:
                present_key_values.append(present_kv)
        
        # Final norm
        hidden_states = self.norm(hidden_states)
        
        # Compute logits
        logits = self.lm_head(hidden_states)
        
        loss = None
        if labels is not None:
            # Shift for next-token prediction
            shift_logits = logits[..., :-1, :].contiguous()
            shift_labels = labels[..., 1:].contiguous()
            
            # Compute loss
            loss_fct = nn.CrossEntropyLoss(ignore_index=-100)
            loss = loss_fct(
                shift_logits.view(-1, self.vocab_size),
                shift_labels.view(-1)
            )
        
        return {
            "loss": loss,
            "logits": logits,
            "past_key_values": present_key_values,
        }
    
    def generate(
        self,
        input_ids: torch.Tensor,
        max_new_tokens: int = 100,
        temperature: float = 1.0,
        top_k: Optional[int] = None,
    ) -> torch.Tensor:
        """
        Autoregressive generation.
        
        Args:
            input_ids: [batch, seq_len] - prompt tokens
            max_new_tokens: Number of new tokens to generate
            temperature: Sampling temperature (1.0 = greedy)
            top_k: If set, sample from top-k only
            
        Returns:
            Generated tokens [batch, seq_len + max_new_tokens]
        """
        self.eval()
        
        past_key_values = None
        
        for _ in range(max_new_tokens):
            # Forward with caching
            outputs = self.forward(
                input_ids=input_ids,
                past_key_values=past_key_values,
                use_cache=True,
            )
            
            # Get logits for last token
            next_token_logits = outputs["logits"][:, -1, :] / temperature
            
            # Apply top-k filtering
            if top_k is not None:
                v, _ = torch.topk(next_token_logits, min(top_k, next_token_logits.size(-1)))
                next_token_logits[next_token_logits < v[:, [-1]]] = float('-inf')
            
            # Sample
            probs = F.softmax(next_token_logits, dim=-1)
            next_token = torch.multinomial(probs, num_samples=1)
            
            # Append to sequence
            input_ids = torch.cat([input_ids, next_token], dim=1)
            past_key_values = outputs["past_key_values"]
            
            # Stop if all sequences hit EOS (if vocab has EOS)
            # Note: GPT-2 doesn't have explicit EOS, so we just generate
            
        return input_ids


def create_wide_shallow_model(config: dict) -> WideShallowTransformer:
    """Factory function to create model from config dictionary."""
    model_config = config.get("model", config)
    
    return WideShallowTransformer(
        vocab_size=model_config.get("vocab_size", 50257),
        num_layers=model_config.get("num_layers", 4),
        num_heads=model_config.get("num_heads", 8192),
        num_kv_heads=model_config.get("num_kv_heads", 64),
        hidden_size=model_config.get("hidden_size", 4096),
        intermediate_size=model_config.get("intermediate_size"),
        head_dim=model_config.get("head_dim", 64),
        max_position_embeddings=model_config.get("max_position_embeddings", 4096),
        dropout=model_config.get("dropout", 0.0),
        embed_dropout=model_config.get("embed_dropout", 0.0),
        activation=model_config.get("activation", "silu"),
    )
