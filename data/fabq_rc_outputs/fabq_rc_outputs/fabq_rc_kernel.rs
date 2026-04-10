//! fabq_rc_kernel — FABQ-RC Dequantization + MatMul for TinyLlama
//!
//! Reads FABQ-RC quantized layers from .npz files (NumPy format) and performs
//! fast dequantized matrix multiplication.
//!
//! Format per layer:
//!   - int8_weights:  i8 array, shape [n_int8, blocksize]  — one int8 channel
//!   - binary_weights: i8 array, shape [n_binary, blocksize] — seven binary channels
//!   - int8_scales:   f32 array, shape [n_int8] or [8]
//!   - binary_scales: f32 array, shape [n_binary, blocksize/8]
//!   - allocation:    JSON map channel_idx -> "int8"|"binary"
//!   - blocksize:     integer blocksize for this layer
//!
//! BPW: 8/8 = 1.0 for int8 channels, 1/8 = 0.125 for binary channels
//! Expected: ~0.5-1.0 bpw total depending on int8 fraction

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Layer metadata from the FABQ-RC quantization run
#[derive(Debug, Clone)]
pub struct LayerMeta {
    /// {channel_idx: "int8"|"binary"}
    pub allocation: HashMap<usize, String>,
    pub blocksize: usize,
    pub weight_shape: (usize, usize), // (out_features, in_features)
    pub n_int8: usize,
    pub n_binary: usize,
}

impl LayerMeta {
    /// Load layer metadata from JSON summary file
    pub fn from_summary_json(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let v: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let alloc_raw = v["allocation"].as_object().unwrap();
        let mut allocation = HashMap::new();
        for (k, val) in alloc_raw {
            let idx: usize = k.parse()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let t = val.as_str().unwrap();
            allocation.insert(idx, t.to_string());
        }
        
        let blocksize = v["blocksize"].as_u64().unwrap() as usize;
        
        let shape_arr = v["weight_shape"].as_array().unwrap();
        let out_f = shape_arr[0].as_u64().unwrap() as usize;
        let in_f = shape_arr[1].as_u64().unwrap() as usize;
        
        let n_int8 = allocation.values().filter(|v| *v == "int8").count();
        let n_binary = allocation.values().filter(|v| *v == "binary").count();
        
        Ok(Self {
            allocation,
            blocksize,
            weight_shape: (out_f, in_f),
            n_int8,
            n_binary,
        })
    }
}

/// FABQ-RC quantized layer weights
#[derive(Debug)]
pub struct FabqRcLayer {
    pub meta: LayerMeta,
    /// int8_weights[i] = [blocksize] f32 values for channel i (after dequant)
    pub int8_dequant: Vec<Vec<f32>>,
    /// binary_weights[i] = [blocksize] f32 values for channel i (after dequant)
    pub binary_dequant: Vec<Vec<f32>>,
}

impl FabqRcLayer {
    /// Load and dequantize a layer from .npz + _summary.json
    ///
    /// Note: This reads the raw .npz which is a ZIP archive containing
    /// individual .npy files. Reading .npy from Rust requires the ndarray-npy
    /// crate or equivalent. For a production kernel, use a dedicated
    /// .npz reading crate (e.g. `ndarray-npy`).
    ///
    /// Simplified here: we show the structure and provide Python-side
    /// dequantization as reference. The actual Rust kernel reads pre-dequantized
    /// binary files (see `fabq_rc_export.py`).
    pub fn load_from_dir(_dir: &Path) -> std::io::Result<Self> {
        todo!("Use ndarray-npy or pre-exported binary format. See fabq_rc_export.py")
    }
}

/// Dequantize int8 channel: value * scale
#[inline(always)]
pub fn dequant_int8(values: &[i8], scale: f32) -> Vec<f32> {
    values.iter().map(|v| (*v as f32) * scale).collect()
}

/// Dequantize binary channel: unpack bits, multiply by per-8-element scales
///
/// binary_weights: packed as [blocksize] i8, each value is 0 or 1
/// binary_scales:  [blocksize/8] f32, one scale per 8 columns
#[inline(always)]
pub fn dequant_binary(packed: &[i8], scales: &[f32], blocksize: usize) -> Vec<f32> {
    let n_blocks = blocksize / 8;
    let mut out = vec![0.0_f32; blocksize];
    
    for block_idx in 0..n_blocks {
        let scale = scales[block_idx];
        for bit in 0..8 {
            let pos = block_idx * 8 + bit;
            if pos < blocksize {
                out[pos] = if packed[pos] != 0 { scale } else { 0.0 };
            }
        }
    }
    
    out
}

/// Compute dot product of two f32 slices
#[inline(always)]
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// matmul: C = A * B, where B is FABQ-RC dequantized
/// A: [m, k] row-major, B: [k, n] row-major, C: [m, n] row-major
pub fn matmul_fabq_rc(a: &[f32], b: &[f32], c: &mut [f32], m: usize, k: usize, n: usize) {
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0_f32;
            let a_row = &a[i * k..];
            let b_col: Vec<f32> = (0..k).map(|p| b[p * n + j]).collect();
            sum = dot(&a_row[..k], &b_col);
            c[i * n + j] = sum;
        }
    }
}

/// Verify dequantization math matches Python reference
/// Call from Python via pyo3 or from tests.
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_dequant() {
        // blocksize=16, 2 blocks of 8
        let packed: Vec<i8> = vec![1, 0, 1, 1, 0, 0, 1, 0,  1, 1, 0, 0, 1, 1, 1, 0];
        let scales: Vec<f32> = vec![0.5, 0.25];
        let result = dequant_binary(&packed, &scales, 16);
        
        assert_eq!(result[0], 0.5);   // 1 * 0.5
        assert_eq!(result[1], 0.0);   // 0 * 0.5
        assert_eq!(result[8], 0.25);  // 1 * 0.25
        assert_eq!(result[15], 0.0); // 0 * 0.25
    }
    
    #[test]
    fn test_int8_dequant() {
        let values: Vec<i8> = vec![10, -5, 0, 20];
        let scale = 0.1;
        let result = dequant_int8(&values, scale);
        
        assert_eq!(result[0], 1.0);
        assert_eq!(result[1], -0.5);
        assert_eq!(result[3], 2.0);
    }
    
    #[test]
    fn test_dot() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![0.5, 1.0, 1.5];
        assert_eq!(dot(&a, &b), 7.0);
    }
}
