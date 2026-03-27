# WeRender-ML

**Zero-Config Distributed ML Training**

Turn any collection of networked computers into a machine learning training cluster instantly.

## Installation

```bash
pip install torch  # Install PyTorch first
pip install werender-ml
```

## Quick Start

### 1. Start Coordinator (Main Machine)

```bash
werender-ml coordinator --model resnet18 --data ./imagenet --epochs 100
```

### 2. Start Workers (Other Machines)

```bash
werender-ml worker
```

Workers automatically discover the coordinator via mDNS and join the training cluster.

## Features

- 🔍 **Zero Config** — Auto-discovery via mDNS
- 🔐 **API Key Security** — Secure authentication
- 🧠 **PyTorch Native** — Uses PyTorch DDP under the hood
- 💪 **Fault Tolerant** — Workers can crash, training continues
- 🔄 **Mixed Hardware** — GPUs train, CPUs preprocess
- 📊 **Dashboard** — Real-time training metrics

## Requirements

- Python 3.10+
- PyTorch 2.0+
- Network connectivity between machines

## Architecture

### Coordinator (Main Machine)
- Holds master model
- Coordinates training
- Aggregates gradients
- Serves dashboard

### Workers (Contributing Machines)
- Pull data shards
- Train locally
- Push gradients

## Roadmap

- [ ] PyTorch DDP integration
- [ ] Dataset sharding
- [ ] Checkpointing
- [ ] TensorFlow support
- [ ] Gradient compression
- [ ] Cloud worker support

## License

MIT
