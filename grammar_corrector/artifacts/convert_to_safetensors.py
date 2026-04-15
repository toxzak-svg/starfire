#!/usr/bin/env python3
"""Convert PyTorch intention_cnn checkpoint to Safetensors for Rust loading."""
import os
import torch
from safetensors.torch import save_file

ARTIFACTS = os.path.dirname(os.path.abspath(__file__))
CKPT_PATH = os.path.join(ARTIFACTS, "intention_cnn.pt.zip")
OUT_PATH = os.path.join(ARTIFACTS, "intention_cnn.safetensors")

ckpt = torch.load(CKPT_PATH, map_location="cpu", weights_only=False)
state = ckpt["state_dict"]

# Filter out training-only items (num_batches_tracked is for training, not inference)
filtered = {k: v for k, v in state.items() if "num_batches_tracked" not in k}

print(f"Original keys: {len(state)}")
print(f"Filtered keys: {len(filtered)}")
print("Keys:", list(filtered.keys()))

save_file(filtered, OUT_PATH)
size_mb = os.path.getsize(OUT_PATH) / 1024 / 1024
print(f"Saved {len(filtered)} tensors → {OUT_PATH} ({size_mb:.2f} MB)")
