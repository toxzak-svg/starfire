"""
FABQ-RC Dequantization Kernel — Rust
Reads FABQ-RC quantized layers and dequantizes to f32 for inference.

Format per layer (from FABQ-RC v2 pipeline):
  - int8_weights:  int8 ndarray, shape (channels, blocksize)  [1 channel gets int8]
  - binary_weights: int8 ndarray, shape (7, blocksize)       [7 channels get binary]
  - int8_scales:   f32 ndarray, shape (channels_per_block,) = (8,)
  - binary_scales: f32 ndarray, shape (7, blocksize//8)     [one scale per 8 columns]
  - allocation:   dict[int->str] channel_id -> "int8"|"binary"
  - blocksize:    int

Dequantization:
  For int8 channels: value * scale
  For binary channels: (bits packed in int8) -> unpacked * binary_scale
"""

import numpy as np
import json, os
from pathlib import Path

DATA_DIR = Path("fabq_rc_outputs")
LAYER_NAME = "model.layers.0.self_attn.q_proj"  # example

def load_layer(layer_name: str):
    """Load all FABQ-RC data for one layer."""
    layer_dir = DATA_DIR / "quantized_layers"
    
    # Load summary JSON
    summary_path = layer_dir / f"{layer_name}_summary.json"
    with open(summary_path) as f:
        summary = json.load(f)
    
    # Load npz
    npz_path = layer_dir / f"{layer_name}.npz"
    npz = np.load(npz_path)
    
    int8_weights = npz["int8_weights"]        # (1, blocksize) or (channels, blocksize)
    binary_weights = npz["binary_weights"]    # (7, blocksize)
    int8_scales = npz["int8_scales"]          # (8,) or (n_int8_channels,)
    binary_scales = npz["binary_scales"]     # (7, blocksize//8)
    
    allocation = summary["allocation"]  # {channel_idx: "int8"|"binary"}
    blocksize = summary["blocksize"]
    
    return {
        "int8_weights": int8_weights,
        "binary_weights": binary_weights,
        "int8_scales": int8_scales,
        "binary_scales": binary_scales,
        "allocation": allocation,
        "blocksize": blocksize,
        "shape": summary["weight_shape"],
    }


def dequantize_layer(layer_name: str) -> np.ndarray:
    """
    Full dequantization of one FABQ-RC layer.
    Returns f32 weight matrix.
    """
    d = load_layer(layer_name)
    
    alloc = d["allocation"]  # {channel_idx: "int8"|"binary"}
    blocksize = d["blocksize"]
    n_channels = len(alloc)
    
    # Determine which channels are int8 vs binary
    int8_channels = [ch for ch, t in alloc.items() if t == "int8"]
    binary_channels = [ch for ch, t in alloc.items() if t == "binary"]
    
    # Shape: (n_channels, blocksize_per_channel)
    # We reshape into (n_channels, original_dim)
    # For now, blocksize = inner_dim, n_channels = outer_dim
    
    n_int8 = len(int8_channels)
    n_binary = len(binary_channels)
    
    int8_w = d["int8_weights"]  # (n_int8, blocksize)
    binary_w = d["binary_weights"]  # (n_binary, blocksize)
    
    # Build output (n_channels, blocksize)
    out = np.zeros((n_channels, blocksize), dtype=np.float32)
    
    # Int8 channels: multiply by scales directly
    for i, ch in enumerate(int8_channels):
        scale = d["int8_scales"][i] if n_int8 > 1 else d["int8_scales"][0]
        out[ch] = int8_w[i].astype(np.float32) * scale
    
    # Binary channels: unpack bits and multiply by binary scales
    # binary_weights packed as (n_binary, blocksize) with values 0 or 1
    # binary_scales: (n_binary, blocksize//8) — one scale per 8 columns
    for i, ch in enumerate(binary_channels):
        # Unpack bits: each byte has 8 binary values
        bw = binary_w[i]  # (blocksize,) int8, values 0 or 1
        bs = d["binary_scales"][i]  # (blocksize//8,)
        # Reshape: 8 values per scale
        n_blocks = blocksize // 8
        bw_reshaped = bw.reshape(n_blocks, 8)  # (n_blocks, 8)
        # Multiply each column by its scale
        result = bw_reshaped.astype(np.float32) * bs.reshape(-1, 1)  # (n_blocks, 8)
        out[ch] = result.flatten()
    
    return out


# ── Quick test ─────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("FABQ-RC Dequantization Kernel — Python reference")
    print()
    
    # Load one layer as smoke test
    layer = "model.layers.0.self_attn.q_proj"
    d = load_layer(layer)
    print(f"Layer: {layer}")
    print(f"  Shape: {d['shape']}")
    print(f"  Blocksize: {d['blocksize']}")
    print(f"  Allocation: {d['allocation']}")
    print(f"  int8_weights shape: {d['int8_weights'].shape}")
    print(f"  binary_weights shape: {d['binary_weights'].shape}")
    print(f"  int8_scales shape: {d['int8_scales'].shape}")
    print(f"  binary_scales shape: {d['binary_scales'].shape}")
    
    w = dequantize_layer(layer)
    print(f"  Dequantized shape: {w.shape}")
    print(f"  Dequantized dtype: {w.dtype}")
    print(f"  Sample values: {w[0,:8]}")
