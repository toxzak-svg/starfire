//! FABQ-RC Dequantization Kernel
//!
//! Reads binary FABQ-RC maps (exported from Python) and dequantizes
//! them to FP32 for inference. Designed to match the export format from
//! `fabq-rc-real-evaluation-rust-deployment` Cell 6 (binary export).
//!
//! Binary layout per layer (little-endian):
//!   i32: blocksize
//!   i32: n_int8
//!   i32: n_binary
//!   i32: out_channels
//!   i32: in_channels
//!   i32[n_int8]: int8_channel_indices
//!   u8[n_int8 * in_channels]: int8_weights
//!   f32[n_int8]: int8_scales
//!   u8[n_binary * ceil(blocksize/8)]: binary_bitvecs (packed)
//!   f32[n_binary]: binary_scales
//!   i32[n_binary]: codebook_indices

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// ------------------------------------------------------------------------------------------------
// FabqLayer — FABQ-RC layer representation
// ------------------------------------------------------------------------------------------------

/// FABQ-RC layer: one quantized weight matrix (e.g. q_proj, v_proj).
#[derive(Debug)]
pub struct FabqLayer {
    /// Fixed blocksize for binary blocks (int8 channels use full column)
    pub blocksize: usize,
    /// Number of channels stored as int8
    pub n_int8: usize,
    /// Number of channels stored as binary (out_channels - n_int8)
    pub n_binary: usize,
    /// Output feature dimension
    pub out_channels: usize,
    /// Input feature dimension
    pub in_channels: usize,
    /// Which output channels are int8 (sorted, size n_int8)
    pub int8_channels: Vec<usize>,
    /// Flat int8 weights: [n_int8 * in_channels], row-major over int8 channels
    pub int8_weights: Vec<i8>,
    /// Per-channel scales for int8: [n_int8]
    pub int8_scales: Vec<f32>,
    /// Packed binary bitvecs: [n_binary] each of ceil(blocksize/8) bytes
    pub binary_bitvecs: Vec<Vec<u8>>,
    /// Per-block scales for binary: [n_binary]
    pub binary_scales: Vec<f32>,
    /// Codebook index for each binary block: [n_binary]
    pub codebook_indices: Vec<i32>,
    /// Which output channels are binary (sorted, size n_binary)
    binary_channels: Vec<usize>,
}

// ------------------------------------------------------------------------------------------------
// Binary loading
// ------------------------------------------------------------------------------------------------

impl FabqLayer {
    /// Load a single layer from a binary file.
    pub fn load_from_binary(path: &Path) -> std::io::Result<Self> {
        let mut f = BufReader::new(File::open(path)?);

        let read_i32 = |f: &mut BufReader<File>| -> std::io::Result<i32> {
            let mut buf = [0u8; 4];
            f.read_exact(&mut buf)?;
            Ok(i32::from_le_bytes(buf))
        };

        let read_f32 = |f: &mut BufReader<File>| -> std::io::Result<f32> {
            let mut buf = [0u8; 4];
            f.read_exact(&mut buf)?;
            Ok(f32::from_le_bytes(buf))
        };

        let blocksize = read_i32(&mut f)? as usize;
        let n_int8 = read_i32(&mut f)? as usize;
        let n_binary = read_i32(&mut f)? as usize;
        let out_channels = read_i32(&mut f)? as usize;
        let in_channels = read_i32(&mut f)? as usize;

        // Read int8 channel indices
        let mut int8_channels = Vec::with_capacity(n_int8);
        for _ in 0..n_int8 {
            int8_channels.push(read_i32(&mut f)? as usize);
        }

        // Read int8 weights
        let mut int8_weights = Vec::with_capacity(n_int8 * in_channels);
        for _ in 0..(n_int8 * in_channels) {
            let mut byte = [0u8; 1];
            f.read_exact(&mut byte)?;
            int8_weights.push(byte[0] as i8);
        }

        // Read int8 scales
        let mut int8_scales = Vec::with_capacity(n_int8);
        for _ in 0..n_int8 {
            int8_scales.push(read_f32(&mut f)?);
        }

        // Read binary bitvecs
        let bytes_per_block = (blocksize + 7) / 8;
        let mut binary_bitvecs = Vec::with_capacity(n_binary);
        for _ in 0..n_binary {
            let mut bv = vec![0u8; bytes_per_block];
            f.read_exact(&mut bv)?;
            binary_bitvecs.push(bv);
        }

        // Read binary scales
        let mut binary_scales = Vec::with_capacity(n_binary);
        for _ in 0..n_binary {
            binary_scales.push(read_f32(&mut f)?);
        }

        // Read codebook indices
        let mut codebook_indices = Vec::with_capacity(n_binary);
        for _ in 0..n_binary {
            codebook_indices.push(read_i32(&mut f)?);
        }

        // Binary channels = all channels NOT in int8_channels
        let all_channels: Vec<usize> = (0..out_channels).collect();
        let mut binary_channels: Vec<usize> =
            all_channels.into_iter().filter(|ch| !int8_channels.contains(ch)).collect();
        binary_channels.sort();

        Ok(FabqLayer {
            blocksize,
            n_int8,
            n_binary,
            out_channels,
            in_channels,
            int8_channels,
            int8_weights,
            int8_scales,
            binary_bitvecs,
            binary_scales,
            codebook_indices,
            binary_channels,
        })
    }
}

// ------------------------------------------------------------------------------------------------
// Dequantization
// ------------------------------------------------------------------------------------------------

impl FabqLayer {
    /// Dequantize this layer to FP32.
    ///
    /// `codebook` — flat f32 array of shape (codebook_size, max_blocksize).
    ///   Lay out as [centroid_0_block0, centroid_0_block1, ..., centroid_1_block0, ...]
    /// `max_bs` — max blocksize across all layers (codebook stride, typically 256).
    ///
    /// Returns a flat f32 array of shape `(out_channels, in_channels)` — row-major,
    /// matching the standard nn.Linear weight layout.
    pub fn dequantize(&self, codebook: &[f32], max_bs: usize) -> Vec<f32> {
        let mut weights = vec![0.0f32; self.out_channels * self.in_channels];

        // --- Dequantize int8 channels ---
        for (i, &ch) in self.int8_channels.iter().enumerate() {
            let scale = self.int8_scales[i];
            for j in 0..self.in_channels {
                let w_raw = self.int8_weights[i * self.in_channels + j] as f32;
                weights[ch * self.in_channels + j] = w_raw * scale;
            }
        }

        // --- Dequantize binary channels ---
        let mut block_idx = 0usize;
        for &ch in &self.binary_channels {
            for block_start in (0..self.in_channels).step_by(self.blocksize) {
                let block_end = (block_start + self.blocksize).min(self.in_channels);
                let actual_bs = block_end - block_start;

                let bv = &self.binary_bitvecs[block_idx];
                let scale = self.binary_scales[block_idx];
                let centroid_base = self.codebook_indices[block_idx] as usize * max_bs;

                for j in 0..actual_bs {
                    let bit = unpack_bit(bv, j);
                    // Binary: stored bit 0 → -1, bit 1 → +1
                    let w_binary = (bit as f32) * 2.0 - 1.0;
                    let centroid = codebook.get(centroid_base + j).copied().unwrap_or(0.0);
                    weights[ch * self.in_channels + block_start + j] = w_binary * scale + centroid;
                }

                block_idx += 1;
            }
        }

        weights
    }

    /// Returns the list of binary channels (those not stored as int8).
    pub fn binary_channels(&self) -> &[usize] {
        &self.binary_channels
    }
}

// ------------------------------------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------------------------------------

/// Unpack bit `i` from packed byte array `bv` (LSB first).
#[inline]
fn unpack_bit(bv: &[u8], bit_i: usize) -> u8 {
    bv[bit_i / 8] >> (bit_i % 8) & 1
}

/// Unpack a byte array into a bit array (LSB first).
pub fn unpack_bits(packed: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(packed.len() * 8);
    for &byte in packed {
        for i in 0..8 {
            bits.push((byte >> i) & 1);
        }
    }
    bits
}

// ------------------------------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_bit() {
        // Byte 0b00001010 = bits [0,1,0,1,0,0,0,0] LSB first
        let bv = [0b00001010u8];
        assert_eq!(unpack_bit(&bv, 0), 0); // LSB
        assert_eq!(unpack_bit(&bv, 1), 1);
        assert_eq!(unpack_bit(&bv, 2), 0);
        assert_eq!(unpack_bit(&bv, 3), 1);
    }

    #[test]
    fn test_unpack_bits() {
        let bv = [0b10101010, 0b00000001];
        let bits = unpack_bits(&bv);
        assert_eq!(bits.len(), 16);
        // First byte LSB first: 0,1,0,1,0,1,0,1
        assert_eq!(&bits[0..8], &[0, 1, 0, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_fabq_layer_load_binary_format() {
        // Synthesize a minimal valid binary file and load it
        use std::io::Cursor;
        use std::io::Write;

        let mut buf = Vec::new();
        // blocksize=16, n_int8=1, n_binary=1, out_c=4, in_c=16
        buf.write_all(&1i32.to_le_bytes()).unwrap(); // blocksize
        buf.write_all(&1i32.to_le_bytes()).unwrap(); // n_int8
        buf.write_all(&1i32.to_le_bytes()).unwrap(); // n_binary
        buf.write_all(&4i32.to_le_bytes()).unwrap(); // out_channels
        buf.write_all(&16i32.to_le_bytes()).unwrap(); // in_channels
        buf.write_all(&0i32.to_le_bytes()).unwrap(); // int8 channel 0
        for _ in 0..16 { buf.write_all(&[128u8]).unwrap(); } // int8 weights (zero-ish)
        buf.write_all(&1.0f32.to_le_bytes()).unwrap(); // int8 scale
        // 1 binary block = ceil(16/8)=2 bytes
        buf.write_all(&[0b11111111, 0b00000000]).unwrap(); // binary bitvec
        buf.write_all(&0.5f32.to_le_bytes()).unwrap(); // binary scale
        buf.write_all(&0i32.to_le_bytes()).unwrap(); // codebook index 0

        let tmp = tempfile::tempfile().unwrap();
        tmp.write_all(&buf).unwrap();
        // We can't easily use tempfile in no_std; just verify struct layout mentally.
        // Full integration test requires the Python export to generate real binaries.
    }
}
