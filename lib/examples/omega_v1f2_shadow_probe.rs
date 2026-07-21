use anyhow::{bail, Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let model_path = std::env::var("OMEGA_V1F2_MODEL_PATH")
        .map(PathBuf::from)
        .context("OMEGA_V1F2_MODEL_PATH must identify the exported F1R1 model")?;
    let report = star::omega_v1f2_shadow::run_builder_probe(&model_path)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.gate_passed {
        bail!("ΩV1-F2 builder probe failed");
    }
    Ok(())
}
