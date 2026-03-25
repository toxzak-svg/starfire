# Wide and Shallow Transformer - Specification

## Overview

This is a transformer architecture that trades depth for width - using 8K-16K attention heads across only 2-4 layers to capture local patterns directly without repeated refinement.

## Core Design Decisions

### 1. Massive Width (8K-16K Attention Heads)

The key innovation is using many more attention heads than typical transformers:
- Standard: 16-128 heads
- This architecture: 8,192-16,384 heads

Each head can attend to different aspects of the input simultaneously, capturing diverse local patterns in parallel.

### 2. Grouped-Query Attention (GQA)

With 8K+ heads, standard MHA would require massive KV cache. We use GQA to reduce this:
- Query heads: 8,192
- Key/Value heads: 64
- Groups per KV: 128 (each KV head shared across 128 query heads)

This dramatically reduces memory while maintaining representation quality.

### 3. Headwise MLP

Instead of one large FFN shared across all heads, each head gets its own small MLP projection:
- Projects hidden → num_heads * head_dim
- Applies SiLU activation
- Projects back to hidden

This is more parameter-efficient than traditional FFN for wide architectures.

### 4. Pre-Norm Residual Architecture

Using pre-norm (RMSNorm/LayerNorm before attention/MLP) with heavy residual connections:
- Helps gradient flow in shallow networks
- Enables direct information bypass from input to output

### 5. Rotary Position Embeddings (RoPE)

Efficient position encoding that works well with the attention mechanism:
- Pre-computed sin/cos embeddings
- Applied to queries and keys
- Supports long sequences (4K+)

## Architecture Parameters

| Component | Value | Notes |
|-----------|-------|-------|
| Layers | 2-4 | Very shallow vs typical 12-96 |
| Attention Heads | 8,192-16,384 | Massive width |
| Hidden Size | 4,096 | Must equal heads × head_dim |
| Head Dimension | 64 | Standard |
| KV Heads | 64 | For GQA |
| Max Seq Length | 4,096 | Can extend to 16K+ |
| Vocab Size | 50,257 | GPT-2 tokenizer compatible |

## Memory & Compute Analysis

For 8K heads, hidden_size=4096, 4 layers:

**Parameters:**
- Embeddings: ~205M
- Attention (Q/KV/O): ~2.1B  
- MLP (gate/up/down): ~1.0B
- LayerNorms: ~0.03M
- **Total: ~3.3B parameters**

**KV Cache with GQA:**
- Without GQA: 8,192 × seq_len × head_dim × 2 (K+V) = ~4GB per layer
- With GQA: 64 × seq_len × head_dim × 2 = ~32MB per layer
- **Memory reduction: 128x**

## Training Considerations

### Initialization
Critical for wide architectures - use small_init to prevent collapse:
```python
std = (hidden_size ** -0.5) * (2 * num_layers) ** -0.5
```

### Learning Rate
- Lower base LR than deep models (1e-4 vs 1e-3)
- Warmup: 1000 steps
- Cosine decay to min_lr

### Challenges
1. **Initialization collapse** - wide layers can collapse to noise
2. **Memory pressure** - even with GQA, large models need GPU clusters
3. **Unknown optimal width** - 8K may be too few or too many
4. **Limited receptive field** - shallow networks may not capture long-range dependencies

## Files

| File | Purpose |
|------|---------|
| `config.yaml` | Hyperparameters |
| `models/wide_shallow.py` | Core model implementation |
| `train.py` | Training loop |
| `benchmark.py` | Evaluation scripts |
| `requirements.txt` | Dependencies |

## Usage

### Training
```bash
python train.py --config config.yaml
```

### Benchmarking
```bash
python benchmark.py --config config.yaml
```

### Inference
```python
from models import create_wide_shallow_model
import yaml

with open('config.yaml') as f:
    config = yaml.safe_load(f)

model = create_wide_shallow_model(config)
input_ids = tokenizer.encode("Hello, world!")
output = model.generate(input_ids, max_new_tokens=50)
```

## Expected Performance

Based on the design philosophy:

| Task | Expected Performance |
|------|---------------------|
| Code completion | Good - local patterns important |
| Syntax/n-gram | Good - direct capture |
| Math reasoning | Unknown - may struggle with depth |
| Long-range reasoning | Risk - shallow network |

## Comparison with Standard Transformer

| Aspect | Standard | Wide-Shallow |
|--------|----------|--------------|
| Depth | 12-96 layers | 2-4 layers |
| Width | 16-128 heads | 8K-16K heads |
| Parameters | Similar | Similar |
| FLOPs/token | Similar | Similar |
| Memory (KV) | High | Low (GQA) |
| Hardware utilization | Depth-oriented | Width-oriented |

## Risks & Limitations

1. **Not proven** - this is an experimental architecture
2. **May not scale** - local patterns may not be sufficient
3. **Hardware constraints** - requires large GPU memory
4. **Training instability** - wide networks are harder to train
