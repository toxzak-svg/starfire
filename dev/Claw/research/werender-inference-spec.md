# WeRender-Inference Specification

## Overview

Turn any collection of networked computers into a distributed LLM inference cluster. Zero config, auto-discovery, just works.

## The Pitch

**"Your own local ChatGPT in 30 seconds"**

```
# Main machine (has the model)
werender-inference start --model llama-3-8b-instruct-q4 --port 8080

# Worker machines (contribute GPU/CPU)
werender-inference worker
```

Workers auto-discover via mDNS. Requests load-balance across all machines. Faster inference, longer context, zero config.

## Architecture

### Components

1. **Coordinator** — Load balancer, manages workers, serves API
2. **Worker** — Runs llama.cpp, handles inference requests
3. **Client** — Sends prompts, receives completions

### Data Flow

```
User Prompt
    ↓
Coordinator (load balancer)
    ↓
Worker 1 (GPU) ←→ Worker 2 (GPU) ←→ Worker 3 (CPU)
    ↓
Responses aggregated or streamed back
```

## Features

### Must Have
- [ ] Auto-discovery (mDNS like WeRender)
- [ ] Load balancing across workers
- [ ] llama.cpp integration
- [ ] Streaming responses
- [ ] Multiple model support

### Nice to Have
- [ ] Model sharding (different workers load different layers)
- [ ] KV cache sharing between workers
- [ ] API compatible with OpenAI
- [ ] Web UI dashboard
- [ ] Token tracking per worker

## CLI Interface

```bash
# Install
pip install werender-inference

# Start inference server (coordinator)
werender-inference start \
  --model meta-llama-3-8b-instruct-q4_k_m.gguf \
  --port 8080 \
  --context 8192

# Add workers
werender-inference worker

# Or with specific GPU
werender-inference worker --gpu 0

# Query
curl http://localhost:8080/v1/chat/completions \
  -d '{"messages":[{"role":"user","content":"Hello!"}]}'
```

## How llama.cpp RPC Works

```cpp
// Server mode
./llama-cli -m model.gguf --rpc server --host 0.0.0.0:50052

// Client mode (delegates to server)
./llama-cli -m model.gguf --rpc 192.168.1.100:50052 -n 256 "prompt"
```

The key: RPC mode splits the model across machines! Some layers on one, some on another.

## Implementation

### Step 1: Wrapper Script
- Python CLI that wraps llama.cpp
- Handles model downloading (from HuggingFace)
- Manages processes

### Step 2: Coordinator
- Flask/FastAPI server
- Worker registry (via mDNS or manual)
- Request distribution (round-robin or fastest-response)

### Step 3: Integration
- Use llama.cpp as the engine
- Communicate via RPC or HTTP

## Comparison

| Feature | WeRender-Inference | llama.cpp alone | LocalAI |
|---------|---------------------|-----------------|---------|
| Distributed | Yes | Manual | Yes |
| Zero config | Yes | No | No |
| Auto-discovery | Yes | No | No |
| Load balancing | Yes | No | Partial |
| OpenAI API | Yes | No | Yes |

## Use Cases

1. **Home lab** — 3-4 GPUs = faster local LLM
2. **Office** — Idle machines help with inference
3. **Classroom** — Students contribute CPUs
4. **Demo** — "Look, my laptops are running ChatGPT!"

## Roadmap

### v0.1
- [ ] Basic CLI wrapper for llama.cpp
- [ ] Start/stop inference server
- [ ] Simple worker registration

### v0.2
- [ ] mDNS auto-discovery
- [ ] Load balancing
- [ ] OpenAI API compatibility

### v0.3
- [ ] Model sharding
- [ ] Dashboard
- [ ] Streaming

## Tech Stack

- **Python** — CLI and coordinator
- **llama.cpp** — Inference engine
- **FastAPI** — Coordinator API
- **zeroconf** — mDNS discovery
- **httpx** — HTTP client for workers

---

**Vision:** `werender-inference start --model llama-3-8b` → your whole network is now a ChatGPT competitor.
