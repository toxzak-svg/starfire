# Personal AI Agent — Starfire OS
**Vision Document** | Created: 2026-04-05  
**Goal:** Build an AI agent that runs on your phone and coordinates your entire connected life.

---

## The Vision

An AI that runs locally on your phone, knows your world, thinks autonomously, and acts on your behalf across all your devices and services.

Not a chatbot. Not a wrapper. An **intelligent agent that is the operating system for your life**.

### What It Does

**At Home:**
- Coordinates all smart devices (Home Assistant integration)
- Manages your network, your files, your screens
- Your laptop, tablet, TV, speakers — all visible to the agent
- You talk to it like a capable household AI

**Away:**
- Stays connected to your home network remotely
- Acts on your behalf even when you're not there
- Accesses your devices, files, systems from anywhere

**The Key Insight:**
Permissions. You give it permission to act across your devices. It uses that permission intelligently. No single platform ecosystem can do this — Apple only works with Apple, Google only with Google. An open, local-first agent can bridge all of it.

---

## Technical Architecture

### On-Device Stack

```
Phone (Primary Agent Hub)
├── Starfire (Reasoning + Memory + Identity)
├── Bonsai-8B Q1_0_g128 (Local LLM — runs on CPU)
│   └── Quantized for mobile inference
├── Network Discovery Module
│   ├── mDNS — find devices on local network
│   ├── UPnP — smart device discovery
│   └── WiFi Direct — peer-to-peer
├── Bluetooth Controller
│   ├── Proximity detection
│   ├── Car/IoT device pairing
│   └── Low-energy peripheral comms
├── Home Assistant Bridge
│   ├── All smart devices via local API
│   └── Scenes, automations, entity control
├── Remote Access Tunnel
│   ├── Tailscale / Cloudflare Tunnel
│   └── Access home from anywhere
└── Context Memory
    ├── Personal preferences
    ├── Device registry
    ├── Conversation history
    └── Behavioral patterns
```

### Device Network

```
Phone Agent
    │
    ├──→ Laptop (file access, screen, compute offload)
    ├──→ Home Server (NAS, media, backup)
    ├──→ Smart Devices (via Home Assistant)
    │       ├── Lights, thermostat, locks
    │       ├── Cameras, sensors
    │       └── Entertainment system
    ├──→ Car (via Bluetooth / USB)
    │       ├── Status, location
    │       └── Cabin preferences
    └──→ Other Phones (proximity P2P)
```

---

## Product Phases

### Phase 1 — Personal Starfire (Now)
**What:** Fine-tune Starfire's reasoning stack on Zach's personal data.  
**Goal:** A model that knows Zach, thinks like Zach, acts for Zach.  
**Data sources:** ChatGPT export, coding traces, conversation history, memos.

### Phase 2 — Mobile Starfire  
**What:** Get Starfire running on phone (iOS/Android).  
**Stack:** Core ML / Android NNAPI for on-device inference.  
**Model:** Quantized personal Starfire — runs on phone CPU.

### Phase 3 — Network Discovery
**What:** Agent discovers and catalogs devices on local network.  
**Stack:** mDNS/Bonjour, UPnP SSDP discovery.  
**Registry:** SQLite DB of known devices, capabilities, access methods.

### Phase 4 — Home Assistant Integration
**What:** Full Home Assistant connection — control everything.  
**Stack:** HA REST API (local, no cloud).  
**Scope:** All entities, scenes, automations exposed to agent reasoning.

### Phase 5 — Remote Access
**What:** Access home network from anywhere.  
**Stack:** Cloudflare Tunnel or Tailscale.  
**Security:** Agent credentials, not user credentials — agent acts on behalf.

### Phase 6 — Bluetooth Mesh
**What:** Low-energy proximity interactions.  
**Use cases:** Car approach detection, wearable context, nearby device P2P.

---

## Training Data Plan

### Data Sources — EXTRACTED ✅

| Source | Items | Status |
|--------|-------|--------|
| ChatGPT Export | 912 train | ✅ Done |
| Git Commits (starfire/candle/star) | 444 train | ✅ Done |
| Starfire Memories | 89 train | ✅ Done |
| Starfire Conversations | 68 train | ✅ Done |
| Perplexity AI Research | 27 train | ✅ Done |
| Nova Chat | 7 train | ✅ Done |
| **Total** | **1,548 train + 210 eval** | ✅ Done |

**Dataset size:** 5.7 MB training + 0.6 MB eval

### Processing Pipeline

Pipeline: `training/dataset_builder.py` → `training/extract_starfire.py` → `training/extract_perplexity.py` → `training/build_full_dataset.py`

Final output: `data/processed/training/train.jsonl` + `eval.jsonl`

### Next: Training

**Base Model:** Bonsai-8B Q1_0_g128
**Method:** LoRA fine-tuning (rank=16, alpha=32)
**Hardware needed:** GPU — not available locally, needs cloud (runpod/vast.ai)

See `training/config.yaml` for full training config once hardware is available.

---

## Open Questions

1. **Quantization after fine-tuning?** Train full precision, then quantize to Q1_0_g128
2. **Mobile inference engine?** Need Core ML (iOS) / NNAPI (Android) support
3. **Home Assistant version?** Needs local API access — cloud-connected HA not ideal
4. **Security model?** Agent credentials vs user credentials — how does delegation work?
5. **How does the agent discover devices?** Manual pairing vs automatic discovery?
6. **What does "acting on your behalf" actually mean?** Define the permission scope clearly

---

## Files

- `plans/personal-agent.md` — this document
- `plans/bonsai-integration.md` — LLM integration (Phase 0 prerequisite)
- `data/raw/` — raw data before processing
- `data/processed/` — formatted training data
- `training/` — training scripts and configs
