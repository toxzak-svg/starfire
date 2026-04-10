"""
fabq_rc_export.py

Converts FABQ-RC .npz quantized layers to a binary format
readable directly by the Rust kernel — no NumPy dependency at runtime.

Binary format per layer (fabqrc extension):
  header:
    magic:          4 bytes  = b'FQRC'
    version:        u16      = 1
    allocation_len: u32      = number of channels
    blocksize:      u32
    out_features:   u32
    in_features:    u32
  per-channel (for each of allocation_len channels):
    channel_id:     u8
    type_flag:      u8       = 0 for int8, 1 for binary
    if int8:
      scale:        f32
      data:         i8[blocksize]
    if binary:
      n_blocks:     u32      = blocksize / 8
      scales:       f32[n_blocks]
      packed:       u8[blocksize / 8]  (8 bits per byte, little-endian)
"""

import numpy as np
import json
import struct
import os
from pathlib import Path

FABQRC_MAGIC = b'FQRC'
FABQRC_VERSION = 1

DATA_DIR = Path("fabq_rc_outputs")
OUTPUT_DIR = Path("fabq_rc_bin")
LAYERS_DIR = DATA_DIR / "quantized_layers"


def npz_to_fabqrc(layer_name: str, output_path: Path):
    """Convert a single layer's .npz to .fabqrc binary format."""
    npz_path = LAYERS_DIR / f"{layer_name}.npz"
    summary_path = LAYERS_DIR / f"{layer_name}_summary.json"
    
    npz = np.load(npz_path)
    with open(summary_path) as f:
        summary = json.load(f)
    
    alloc: dict = summary["allocation"]
    blocksize: int = summary["blocksize"]
    shape: list = summary["weight_shape"]  # [out_features, in_features]
    out_features, in_features = shape[0], shape[1]
    
    int8_w = npz["int8_weights"]
    binary_w = npz["binary_weights"]
    int8_scales = npz["int8_scales"]
    binary_scales = npz["binary_scales"]
    
    n_int8 = sum(1 for t in alloc.values() if t == "int8")
    n_binary = sum(1 for t in alloc.values() if t == "binary")
    
    with open(output_path, "wb") as f:
        # ── Header ────────────────────────────────────────────────────────────
        f.write(FABQRC_MAGIC)
        f.write(struct.pack("<H", FABQRC_VERSION))          # version u16
        f.write(struct.pack("<I", len(alloc)))               # allocation_len u32
        f.write(struct.pack("<I", blocksize))                # blocksize u32
        f.write(struct.pack("<I", out_features))             # out_features u32
        f.write(struct.pack("<I", in_features))              # in_features u32
        
        # ── Per-channel data ─────────────────────────────────────────────────
        int8_idx = 0
        binary_idx = 0
        
        # Channels are 0-7, iterate in order
        for ch in sorted(alloc.keys()):
            ch_type = alloc[ch]
            f.write(struct.pack("<B", ch))                    # channel_id u8
            f.write(struct.pack("<B", 0 if ch_type == "int8" else 1))  # type_flag
            
            if ch_type == "int8":
                scale = float(int8_scales[int8_idx] if n_int8 > 1 else int8_scales[0])
                f.write(struct.pack("<f", scale))             # scale f32
                # Write int8 data as raw i8 bytes
                data = int8_w[int8_idx].tobytes()
                f.write(data)
                int8_idx += 1
            else:
                n_blocks = blocksize // 8
                f.write(struct.pack("<I", n_blocks))          # n_blocks u32
                # Write scales
                f.write(binary_scales[binary_idx].tobytes())
                # Pack binary into bytes (8 bits per byte)
                packed_row = binary_w[binary_idx]
                packed_bytes = bytearray()
                for byte_idx in range(n_blocks):
                    byte_val = 0
                    for bit in range(8):
                        pos = byte_idx * 8 + bit
                        if pos < blocksize and packed_row[pos] != 0:
                            byte_val |= (1 << bit)
                    packed_bytes.append(byte_val)
                f.write(bytes(packed_bytes))
                binary_idx += 1
    
    size = output_path.stat().st_size
    print(f"  {layer_name}: {size/1024:.1f} KB  (blocksize={blocksize}, int8={n_int8}, binary={n_binary})")
    return size


def export_all(output_dir: Path = OUTPUT_DIR):
    """Export all quantized layers to .fabqrc binary format."""
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Find all _summary.json files
    summary_files = sorted(LAYERS_DIR.glob("*_summary.json"))
    print(f"Found {len(summary_files)} layers to export")
    print()
    
    total_size = 0
    for sf in summary_files:
        layer_name = sf.stem  # "model.layers.0.self_attn.q_proj_summary"
        # Remove _summary suffix (already removed in stem since suffix is _summary.json)
        actual_name = layer_name.replace("_summary", "")
        out_path = output_dir / f"{actual_name}.fabqrc"
        try:
            total_size += npz_to_fabqrc(actual_name, out_path)
        except Exception as e:
            print(f"  ERROR {actual_name}: {e}")
    
    print()
    print(f"Total: {total_size/1e6:.2f} MB")


if __name__ == "__main__":
    print("FABQ-RC Binary Exporter")
    print("="*60)
    export_all()
