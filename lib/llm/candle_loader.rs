//! Candle Loader — Load Bonsai/GGUF quantized models via candle-core
//!
//! Uses candle's native quantized GGUF support to load Q1_0_g128 (Bonsai) models
//! directly in-process without any external server or subprocess.
//!
//! The `candle` crate is sourced from `projects/candle/` which is already part
//! of the workspace, so this links against it as a local path dependency.

use candle_core::{Device, Result, Tensor};
use candle_core::quantized::gguf_file;
use std::path::Path;

/// Load a GGUF file and return its tensor metadata (headers only).
/// This is fast and uses minimal memory — just the file index.
pub fn load_gguf_metadata(path: &Path) -> Result<GgufMetadata> {
    let mut file = std::fs::File::open(path)?;
    let content = gguf_file::Content::read(&mut file)?;

    let mut q1_0_g128_tensors = Vec::new();
    let mut other_dtypes = std::collections::HashMap::new();

    for (name, info) in &content.tensor_infos {
        let dtype_name = format!("{:?}", info.ggml_dtype);
        if matches!(
            info.ggml_dtype,
            candle_core::quantized::GgmlDType::Q1_0_g128
        ) {
            q1_0_g128_tensors.push(TensorInfo {
                name: name.clone(),
                shape: info.shape.clone(),
            });
        } else {
            *other_dtypes.entry(dtype_name).or_insert(0) += 1;
        }
    }

    Ok(GgufMetadata {
        gguf_version: format!("{:?}", content.magic),
        total_tensors: content.tensor_infos.len(),
        q1_0_g128_count: q1_0_g128_tensors.len(),
        q1_0_g128_tensors,
        other_dtypes,
        tensor_data_offset: content.tensor_data_offset,
    })
}

/// Load a specific tensor by name (reads actual weight data).
pub fn load_tensor(
    path: &Path,
    tensor_name: &str,
    offset: u64,
) -> Result<Tensor> {
    let device = Device::Cpu;
    let mut file = std::fs::File::open(path)?;
    let content = gguf_file::Content::read(&mut file)?;

    let info = content.tensor_infos
        .get(tensor_name)
        .ok_or_else(|| candle_core::Error::Msg(format!(
            "Tensor not found: {}", tensor_name
        )))?;

    info.read(&mut file, offset, &device)
}

/// Check if a GGUF file uses Q1_0_g128 (Bonsai) quantization.
pub fn is_bonsai_model(path: &Path) -> Result<bool> {
    let metadata = load_gguf_metadata(path)?;
    Ok(metadata.q1_0_g128_count > 0)
}

/// Get the model file size in human-readable form.
pub fn model_size_human(path: &Path) -> String {
    let bytes = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    let mb = bytes as f64 / 1_048_576.0;
    let gb = mb / 1024.0;
    if gb >= 1.0 {
        format!("{:.1} GB", gb)
    } else {
        format!("{:.0} MB", mb)
    }
}

#[derive(Debug)]
pub struct GgufMetadata {
    pub gguf_version: String,
    pub total_tensors: usize,
    pub q1_0_g128_count: usize,
    pub q1_0_g128_tensors: Vec<TensorInfo>,
    pub other_dtypes: std::collections::HashMap<String, usize>,
    pub tensor_data_offset: u64,
}

#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonsai_detection() {
        // Path to the downloaded Bonsai model
        let path = Path::new("models/bonsai-8b/Bonsai-8B.gguf");

        if !path.exists() {
            println!("SKIPPED: Bonsai model not found at {:?}", path);
            return;
        }

        let metadata = load_gguf_metadata(path).unwrap();
        println!("GGUF version: {}", metadata.gguf_version);
        println!("Total tensors: {}", metadata.total_tensors);
        println!("Q1_0_g128 tensors: {}", metadata.q1_0_g128_count);
        println!("Other dtypes: {:?}", metadata.other_dtypes);
        println!("Model size: {}", model_size_human(path));

        assert!(metadata.q1_0_g128_count > 0, "Expected Q1_0_g128 tensors");
    }
}
