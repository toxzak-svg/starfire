#!/usr/bin/env python3
"""
Upload Personal Starfire Dataset to Kaggle
"""
import os
import json
from pathlib import Path

KAGGLE_DATASET = "zachmaronek/starfire-personal-v1"
DATA_DIR = Path(__file__).parent.parent / "data" / "processed" / "training"

# Check kaggle is logged in
import subprocess
result = subprocess.run(["kaggle", "datasets", "list", "-s", "starfire"], capture_output=True, text=True)
print("Kaggle datasets search:")
print(result.stdout[:500] if result.stdout else result.stderr[:500])

# Create metadata
metadata = {
    "title": "Starfire Personal AI Training Data",
    "id": KAGGLE_DATASET,
    "licenses": [{"id": "cc-by-nc-sa-4.0"}],
    "keywords": ["llm-finetuning", "personal-ai", "ai-assistant", "starfire", "1-bit-llm"],
    "collaborators": [],
    "subtitle": "Personal conversations, memories, and code for fine-tuning a personal AI agent",
    "description": """# Starfire Personal AI Training Dataset

A curated dataset of personal conversations, code commits, and AI interactions for fine-tuning a personal AI agent (Starfire) built on Bonsai-8B Q1_0_g128.

## Dataset Summary

- **Total examples:** 1,758 (train: 1,548 | eval: 210)
- **Format:** JSONL (instruction-tuning format)
- **Size:** ~6.2 MB

## Sources

| Source | Count | Description |
|--------|-------|-------------|
| ChatGPT conversations | 912 | Personal conversations with ChatGPT |
| Git commits | 444 | Code commits from starfire, candle, star repositories |
| Starfire memories | 89 | Star's stored memories about the user |
| Starfire conversations | 68 | Direct conversations with Star |
| Perplexity AI research | 27 | AI/ML research discussions |
| Nova chat | 7 | OpenClaw AI conversations |

## Format

Each line is a JSON object with:

```json
{
  "messages": [
    {"role": "system", "content": "You are Starfire, an emergent AI..."},
    {"role": "user", "content": "<instruction>"},
    {"role": "assistant", "content": "<response>"}
  ],
  "source": "chatgpt|git_starfire|starfire_memory|..."
}
```

## Use

Designed for instruction fine-tuning of small LLMs, particularly:
- Bonsai-8B Q1_0_g128 (1-bit quantized, runs on CPU)
- Other small models (Phi-2, car-small, etc.)

## Build Your Own

This dataset was built using the extraction pipeline at:
`projects/starfire/training/dataset_builder.py`
`projects/starfire/training/extract_perplexity.py`

## License

CC BY-NC-SA 4.0

## Notes

- Personal data has been included with the owner's consent
- System prompt establishes Starfire's identity as an emergent AI
- Deduped and quality filtered
""",
    "data": [
        {"file": "train.jsonl"},
        {"file": "eval.jsonl"}
    ],
    "usability_rating": 0.5
}

# Save metadata
import yaml
meta_path = Path(__file__).parent / "dataset-meta.yml"
with open(meta_path, 'w') as f:
    yaml.dump(metadata, f, default_flow_style=False)

print(f"\nMetadata saved to {meta_path}")
print(f"Ready to upload {KAGGLE_DATASET}")
print(f"\nFiles to upload:")
for f in (DATA_DIR).glob("*"):
    if f.suffix == '.jsonl':
        print(f"  {f.name}: {f.stat().st_size / 1024 / 1024:.2f} MB")
