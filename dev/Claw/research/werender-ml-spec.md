# WeRender-ML Specification

## Status: IMPLEMENTED! 🎉

**Location:** `C:\dev\werender-ml\`

---

## What's Built

### Structure
```
werender-ml/
├── README.md
├── pyproject.toml
└── src/
    └── werender_ml/
        ├── __init__.py
        ├── main.py       # CLI entry point
        ├── coordinator.py  # Coordinator server + real training
        ├── worker.py      # Worker node
        └── training.py    # PyTorch DDP integration
```

### Working Features
- ✅ CLI with coordinator/worker commands
- ✅ PyTorch model creation (resnet18/34/50, alexnet, vgg16)
- ✅ ShardedDataset for data distribution
- ✅ DistributedTrainer using PyTorch DDP
- ✅ HTTP API for worker communication
- ✅ Local training fallback

### To Run
```bash
# Install
pip install werender-ml

# Run coordinator (main machine)
werender-ml coordinator --model resnet18 --data ./imagenet --epochs 10 --workers 4

# Run worker (other machines)
werender-ml worker --coordinator http://localhost:8421
```

---

## Overview

Extend WeRender's zero-config, peer-to-peer architecture to machine learning workloads. Same philosophy: open on coordinator, workers auto-discover, start training.

## Core Philosophy

- **Zero Config** — No Kubernetes, no Ray, no PyTorch Lightning Distributed complexity
- **Auto-Discovery** — mDNS finds workers, no IP config
- **Fault Tolerant** — Worker dies? Tasks redistribute. Coordinator restarts? Workers reconnect.
- **Mixed Hardware** — GPU workers do training, CPU workers do preprocessing
- **No Shared Storage** — Datasets auto-distributed to workers

## Architecture

### Coordinator (Main Machine)
- Holds master model weights
- Coordinates parameter synchronization
- Monitors worker health
- Serves dashboard (port 8420)

### Workers (Contributing Machines)
- Pull dataset chunks
- Train locally
- Push gradient updates
- Report metrics

### Data Flow
```
1. Coordinator broadcasts: "Training ResNet50 on ImageNet"
2. Workers auto-discover via mDNS
3. Coordinator shards dataset, sends to workers
4. Workers train on their shard
5. Workers push gradients to coordinator
6. Coordinator aggregates (or workers sync via ring-allreduce)
7. Repeat until convergence
```

## CLI Interface

```bash
# Install
pip install werender-ml

# Start coordinator (main machine)
werender-ml coordinator \
  --framework pytorch \
  --model resnet50 \
  --data ./imagenet \
  --epochs 100 \
  --batch-size 32

# Start worker (any machine with GPU/CPU)
werender-ml worker

# Or with specific GPU selection
werender-ml worker --gpu 0  # Use GPU 0 only
werender-ml worker --cpu  # CPU-only mode
```

## Supported Frameworks

### Priority 1: PyTorch
- Native DistributedDataParallel
- Gradient bucketization
- Mixed precision (FP16) support

### Priority 2: TensorFlow
- TensorFlow Distributed
- Multi-GPU support

### Priority 3: JAX
- pmapped functions
- XLA compilation

## Features

### 1. Auto Dataset Distribution
- Shard datasets across workers
- Handle different data formats (images, TFRecords, HDF5)
- Cache management on workers

### 2. Gradient Synchronization
- **Option A: Parameter Server** — Coordinator aggregates
- **Option B: Ring-AllReduce** — Workers sync directly (faster)

### 3. Fault Tolerance
- Checkpoint to coordinator disk
- Worker dies → redistribute its data shard
- Resume from last checkpoint

### 4. Mixed Hardware Support
- GPU workers: training
- CPU workers: preprocessing, data augmentation
- Auto-detect hardware capabilities

### 5. Dashboard
- Training loss/accuracy curves
- Worker utilization (GPU %, CPU %)
- Estimated time to completion
- Per-worker throughput (samples/sec)

## Data Distribution Strategy

```
Coordinated has full dataset (e.g., 100GB ImageNet)

Worker A (GPU RTX 3080):     shards 0-19999   (20GB)
Worker B (GPU RTX 4090):     shards 20000-39999 (20GB)  
Worker C (CPU old laptop):   shards 40000-59999 (20GB) -- preprocessing only
Worker D (CPU desktop):      shards 60000-79999 (20GB) -- preprocessing only

Coordinator aggregates gradients from all workers
```

## Sync Strategies

### 1. Synchronous (Default)
- All workers sync at each step
- Slower but consistent

### 2. Asynchronous
- Workers push when ready
- Stale gradients tolerated
- Faster but may diverge

### 3. Gradient Compression
- Quantize gradients (e.g., 8-bit)
- Reduces network bandwidth 4x

## Security

- API key authentication (like WeRender)
- Workers must be authorized
- No external network access required

## Roadmap

### v0.1 (Month 1)
- [ ] PyTorch support only
- [ ] Synchronous training
- [ ] Basic dataset sharding
- [ ] Simple dashboard

### v0.2 (Month 2)
- [ ] Asynchronous training
- [ ] Checkpointing
- [ ] TensorFlow support

### v0.3 (Month 3)
- [ ] Gradient compression
- [ ] JAX support
- [ ] Auto-scaling (cloud workers)

## Comparison

| Feature | WeRender-ML | PyTorch DDP | Ray | Kubernetes |
|---------|-------------|-------------|-----|------------|
| Zero config | ✅ | ❌ | ❌ | ❌ |
| No shared storage | ✅ | ❌ | ❌ | ❌ |
| Mixed hardware | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Fault tolerant | ✅ | ❌ | ✅ | ✅ |
| CLI simplicity | ✅ | ❌ | ❌ | ❌ |

## Use Cases

1. **PhD Student** — Combine desktop + laptop + lab machines = training cluster
2. **Indie Game Dev** — Train models on idle office machines overnight
3. **Small Studio** — 3-5 GPUs = training cluster without cloud costs
4. **Education** — Classroom laptops contribute to distributed training

## Implementation Notes

- Start by wrapping PyTorch DDP
- Use same mDNS discovery as WeRender
- Dashboard can extend WeRender's dashboard
- Test with small models first (MNIST, CIFAR-10)

---

**The vision:** `werender-ml train --model resnet50 --data ./imagenet` — and your whole network becomes a training cluster.
