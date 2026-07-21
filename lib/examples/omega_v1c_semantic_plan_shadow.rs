//! ΩV1-C matched-shadow semantic response-plan probe.

use anyhow::Result;
use star::omega_v1_semantic_plan::run_shadow_migration;

fn main() -> Result<()> {
    let report = run_shadow_migration()?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.gate_passed {
        anyhow::bail!("ΩV1-C semantic response-plan shadow gate failed");
    }
    Ok(())
}
