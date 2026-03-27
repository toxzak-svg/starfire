# WeRender-Inference

**Zero-Config Distributed LLM Inference**

Turn any collection of networked computers into a distributed LLM inference cluster.

## Installation

```bash
# Get llama.cpp binary first
# Download from: https://github.com/ggerganov/llama.cpp/releases

pip install werender-inference
```

## Quick Start

### 1. Start Server (Main Machine)

```bash
werender-inference start --model ./models/llama-3-8b-instruct-q4.gguf --port 8080
```

### 2. Add Workers (Other Machines)

```bash
werender-inference worker --coordinator http://192.168.1.100:8080
```

### 3. Chat

```bash
werender-inference chat --prompt "Hello!"
```

## Features

- Zero config - auto-discovery (coming soon)
- Load balancing across workers
- OpenAI-compatible API
- Streaming support
- Multiple model support

## Requirements

- llama.cpp binary (llama-cli.exe)
- Python 3.10+
- Network connectivity between machines

## Architecture

```
User Request
    ↓
Coordinator (load balancer)
    ↓
Worker 1 ←→ Worker 2 ←→ Worker 3
    ↓
Response
```

## Roadmap

- [ ] mDNS auto-discovery
- [ ] Load balancing
- [ ] Worker health monitoring
- [ ] Model sharding

## License

MIT
