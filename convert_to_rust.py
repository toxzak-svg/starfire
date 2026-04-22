#!/usr/bin/env python3
"""Convert PyTorch checkpoint to Rust binary format."""

import torch
import struct
import sys
import os

def save_rust_binary(checkpoint, output_path):
    """Save model weights in exact Rust CharRNN binary format.

    Rust reads:
    - Config: 5 u32/f32 values
    - Embedding: read_f32_vec (u64 len + data)
    - Per LSTM layer:
      - i_weight: read_f32_vec
      - i_bias: single f32 (NOT a vector!)
      - f_weight: read_f32_vec
      - f_bias: single f32
      - c_weight: read_f32_vec
      - c_bias: single f32
      - o_weight: read_f32_vec
      - o_bias: single f32
    - output_weight: read_f32_vec
    - output_bias: read_f32_vec
    """

    state_dict = checkpoint['model']
    config = checkpoint.get('config', {})
    vocab_size = config.get('VOCAB_SIZE', 227)
    embedding_dim = config.get('EMBEDDING_DIM', 64)
    hidden_size = config.get('HIDDEN_SIZE', 256)
    num_layers = config.get('NUM_LAYERS', 2)
    dropout = config.get('DROPOUT', 0.1)

    print(f"Config: vocab={vocab_size}, embed={embedding_dim}, hidden={hidden_size}, layers={num_layers}")

    with open(output_path, 'wb') as f:
        def w_u32(v): f.write(struct.pack('<I', v))
        def w_f32(v): f.write(struct.pack('<f', v))
        def w_vec(v):
            f.write(struct.pack('<Q', len(v)))
            for x in v: f.write(struct.pack('<f', x))

        # Config: vocab_size, embedding_dim, hidden_size, num_layers as u32, dropout as f32
        w_u32(vocab_size)
        w_u32(embedding_dim)
        w_u32(hidden_size)
        w_u32(num_layers)
        w_f32(dropout)

        # Embedding: (vocab_size * embedding_dim)
        emb = state_dict['embedding.weight'].numpy().flatten()
        print(f"Embedding: {emb.shape}, expect {vocab_size * embedding_dim}")
        w_vec(emb.tolist())

        # LSTM cells
        for layer in range(num_layers):
            prefix = f'lstm_cells.{layer}.'

            # i_weight: (hidden_size, input_size + hidden_size)
            # First layer: input_size = embedding_dim, Other layers: input_size = hidden_size
            input_size = embedding_dim if layer == 0 else hidden_size
            total = input_size + hidden_size
            expected_iw = hidden_size * total
            w = state_dict[f'{prefix}i_weight'].numpy().flatten()
            print(f"Layer {layer} i_weight: {w.shape}, expect {expected_iw}")
            w_vec(w.tolist())

            # i_bias: Vec<f32> (hidden_size,)
            b = state_dict[f'{prefix}i_bias'].numpy().flatten()
            print(f"Layer {layer} i_bias: {b.shape}, expect {hidden_size}")
            w_vec(b.tolist())

            # f_weight
            w = state_dict[f'{prefix}f_weight'].numpy().flatten()
            w_vec(w.tolist())

            # f_bias: Vec<f32> (hidden_size,)
            b = state_dict[f'{prefix}f_bias'].numpy().flatten()
            w_vec(b.tolist())

            # c_weight
            w = state_dict[f'{prefix}c_weight'].numpy().flatten()
            w_vec(w.tolist())

            # c_bias: Vec<f32> (hidden_size,)
            b = state_dict[f'{prefix}c_bias'].numpy().flatten()
            w_vec(b.tolist())

            # o_weight
            w = state_dict[f'{prefix}o_weight'].numpy().flatten()
            w_vec(w.tolist())

            # o_bias: Vec<f32> (hidden_size,)
            b = state_dict[f'{prefix}o_bias'].numpy().flatten()
            w_vec(b.tolist())

        # Output layer: (vocab_size, hidden_size)
        out_w = state_dict['output_weight'].numpy().flatten()
        print(f"Output weight: {out_w.shape}, expect {hidden_size * vocab_size}")
        w_vec(out_w.tolist())

        # Output bias: (vocab_size,)
        out_b = state_dict['output_bias'].numpy().flatten()
        print(f"Output bias: {out_b.shape}, expect {vocab_size}")
        w_vec(out_b.tolist())

    size_mb = os.path.getsize(output_path) / 1024 / 1024
    print(f"\nSaved to {output_path} ({size_mb:.2f} MB)")

def main():
    if len(sys.argv) < 3:
        print("Usage: python convert_to_rust.py <checkpoint.pt> <output.bin>")
        sys.exit(1)

    ckpt_path = sys.argv[1]
    output_path = sys.argv[2]

    print(f"Loading checkpoint from {ckpt_path}...")
    checkpoint = torch.load(ckpt_path, map_location='cpu')
    print(f"Keys: {list(checkpoint.keys())}")

    print(f"\nConverting to Rust binary format...")
    save_rust_binary(checkpoint, output_path)
    print("Done!")

if __name__ == '__main__':
    main()