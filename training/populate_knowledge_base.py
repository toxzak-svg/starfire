#!/usr/bin/env python3
"""
Populate Star's Knowledge Base with personal context from training data.
Extracts key facts and insights from the dataset and adds them to Star's memory.
"""
import json
from pathlib import Path

STARFIRE_LIB = Path.home() / ".openclaw" / "workspace" / "projects" / "starfire"
DATA_DIR = STARFIRE_LIB / "data" / "processed" / "training"

# Facts to extract from the data
# These will be added to the knowledge base for Star to use in conversations

FACTS_ABOUT_ZACH = [
    # From ChatGPT conversations
    ("Zach is working on OaaS (Operations as a Service)", "Zach is building a business around operations automation"),
    ("Zach is researching GPU infrastructure", "Zach has set up remote GPU servers with WireGuard tunnels"),
    ("Zach is interested in 1-bit LLM models", "Zach is exploring Bonsai and 1-bit quantization for local AI"),
    ("Zach is building Starfire as a personal AI agent", "Starfire is Zach's AI project with reasoning and memory"),
    ("Zach has worked on car-small (DonkeyCar ML model)", "Zach trained car-small, an 840K param Mamba SSM controller"),
    ("Zach is interested in quantization research", "Zach has been experimenting with quantization of transformers and SSMs"),
    ("Zach has a cat named Sky", "Sky is a grey tabby cat with golden eyes"),
    ("Zach's Telegram handle is @Ton0Fun", "Zach's username on Telegram is Ton0Fun"),
]

FACTS_ABOUT_PROJECTS = [
    ("Starfire has a book system for hierarchical knowledge", "Books → Chapters → Sections with density tiers (High/Medium/Low/Packed)"),
    ("Starfire has a causal discovery module", "Finds causal edges from temporal patterns in data"),
    ("Starfire has a curiosity system", "Autonomous probing of knowledge gaps between conversations"),
    ("Starfire has a metacognition module", "Knows what it knows, tracks confidence explicitly"),
    ("Starfire uses Bonsai-8B Q1_0_g128 via Candle", "1-bit quantized LLM running on CPU"),
    ("Quanot lives inside Starfire", "Quantum-inspired symbolic reasoning engine"),
    ("Starfire has a CLI (star) and HTTP API", "star chat, star doctor commands; REST API on port 8080"),
    ("Felix is the R&D lab", "Python CLI + Next.js dashboard for experiments"),
]

# Additional context about what Zach cares about
ZACH_VALUES = [
    "Building things that run locally without GPU dependency",
    "Creating AI that has genuine personality, not just templated responses",
    "Pushing the boundary on 1-bit / extreme quantization",
    "Making AI that can actually act on the world (agents, not just chat)",
    "Privacy-first: data stays local, no cloud dependency",
]

print("Personal context extracted from training data:")
print(f"  Facts about Zach: {len(FACTS_ABOUT_ZACH)}")
print(f"  Facts about projects: {len(FACTS_ABOUT_PROJECTS)}")
print(f"  Values/interests: {len(ZACH_VALUES)}")

# Now write this as a knowledge base document
# The Book system uses sections with bookmark keys

OUTPUT = STARFIRE_LIB / "data" / "processed" / "personal_context_kb.json"

kb_data = {
    "about_zach": FACTS_ABOUT_ZACH,
    "about_projects": FACTS_ABOUT_PROJECTS,
    "zach_values": ZACH_VALUES,
}

with open(OUTPUT, 'w', encoding='utf-8') as f:
    json.dump(kb_data, f, indent=2, ensure_ascii=False)

print(f"\nSaved knowledge base to: {OUTPUT}")

# Also create a summary document that could be added to the Book system
SUMMARY = f"""
# Personal Context for Star

## About Zach
{ZACH_VALUES}

## Projects

### Starfire
Starfire is Zach's primary AI project — an emergent AI with reasoning, memory, and personality.
Built in Rust with modular subsystems: causal discovery, metacognition, curiosity, book system.

### Bonsai-8B
Zach is running Bonsai-8B Q1_0_g128 via Candle — a 1-bit quantized model that runs on CPU.
This is the LLM powering Starfire's generation.

### Quanot
Quantum-inspired symbolic reasoning engine living inside Starfire.

### car-small
840K parameter Mamba SSM trained for DonkeyCar — an edge AI controller.

### Felix
Zach's R&D lab — Python CLI + Next.js dashboard for experiments.

## Personal
- Cat: Sky (grey tabby, golden eyes)
- Telegram: @Ton0Fun
- Based in EDT timezone

## What's Important to Zach
- GPU-free AI that runs on any device
- Genuine AI personality, not just wrapper
- Open, personal, local-first
"""

summary_path = STARFIRE_LIB / "data" / "processed" / "personal_context.md"
with open(summary_path, 'w', encoding='utf-8') as f:
    f.write(SUMMARY)

print(f"Saved summary to: {summary_path}")
print(f"\nThis context can now be added to Star's Book system or used to populate memory.")
