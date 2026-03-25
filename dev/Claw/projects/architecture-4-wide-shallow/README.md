# Wide and Shallow Transformer

**Core intuition:** Go against the trend of deeper models — instead, go massively wide with 8K-16K attention heads but only 2-4 layers. Capture local patterns directly without repeated refinement.

## Architecture

- **8K-16K attention heads** across 2-4 layers
- **Grouped-query attention** to reduce KV overhead
- Each head has its own MLP projection head
- Heavy residual connections for information flow

## Why It Could Work

- GPUs are massively parallel — width maps better to hardware than depth
- Local patterns (syntax, n-gram statistics) captured directly
- Different statistical structure than deep stacks

## Training Challenges

- Initialization matters — wide layers can collapse to noise
- Need different learning rate schedules
- Risk: might not capture long-range reasoning

## Benchmark

Compare same-param transformer on code completion, math, long-context reasoning.

---

## Project Structure

```
architecture-4-wide-shallow/
├── README.md              # This file
├── SPEC.md               # Detailed specification
├── config.yaml            # Hyperparameters
├── requirements.txt      # Python dependencies
├── models/
│   ├── __init__.py
│   └── wide_shallow.py   # Core model implementation
├── train.py              # Training script
└── benchmark.py          # Evaluation script
```

## Quick Start

### Installation

```bash
pip install -r requirements.txt
```

### Training (Full Model - requires A100/H100)

```bash
python train.py --config config.yaml
```

### Training (Small Model - fits on 30GB GPU)

```bash
python train.py --config config_small.yaml
```

### Benchmarking

```bash
python benchmark.py --config config.yaml
```

## Hardware Requirements

| GPU | Memory | Config to Use |
|-----|--------|---------------|
| A100/H100 | 80GB | config.yaml |
| P5000 | 30GB | config_small.yaml |

### Paperspace Gradient Setup

1. Create Gradient notebook with P5000 (or A100 for full model)
2. Upload project files
3. Run: `pip install -r requirements.txt`
4. Run: `python train.py --config config_small.yaml`

## Model Components

| Component | Description |
|-----------|-------------|
| `WideShallowTransformer` | Main model class |
| `GroupedQueryAttention` | Multi-head attention with 8K+ heads |
| `HeadwiseMLP` | Per-head MLP projection |
| `WideShallowDecoderLayer` | Single decoder layer |
| `WideShallowRotaryEmbedding` | RoPE position embeddings |

## Configuration

Key parameters in `config.yaml`:

```yaml
model:
  num_layers: 4          # Shallow: 2-4 layers
  num_heads: 8192        # Wide: 8K-16K heads
  num_kv_heads: 64       # GQA: KV heads
  hidden_size: 4096      # Model dimension

training:
  learning_rate: 1.0e-4  # Lower than standard
  max_steps: 100000
```

## Architecture Comparison

| Aspect | Standard Transformer | Wide-Shallow |
|--------|---------------------|--------------|
| Layers | 12-96 | 2-4 |
| Heads | 16-128 | 8K-16K |
| Depth | Deep | Shallow |
| Width | Narrow | Massive |

## Key Design Decisions

1. **Grouped-Query Attention (GQA)**: With 8K query heads but only 64 KV heads, we achieve 128x memory reduction vs standard MHA.

2. **Headwise MLP**: Each attention head gets its own small MLP projection, more parameter-efficient than traditional FFN.

3. **Pre-Norm Residual**: Heavy residual connections help gradient flow in shallow networks.

4. **Small Init**: Critical initialization strategy to prevent wide layer collapse.

## Hardware Utilization

The wide architecture maps better to GPU parallelism:
- 8K+ heads = high parallelism per layer
- Fewer layers = less sequential computation
- GQA = smaller KV cache = faster inference

## Risks & Limitations

- **Not proven** — experimental architecture
- May not capture long-range dependencies
- Training instability with wide networks
- Hardware constraints with massive width

## License

MIT
