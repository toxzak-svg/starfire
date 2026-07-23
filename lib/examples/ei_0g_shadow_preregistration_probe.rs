#[cfg(not(feature = "emerging-intelligence-shadow"))]
fn main() {
    eprintln!("enable --features emerging-intelligence-shadow");
}

#[cfg(feature = "emerging-intelligence-shadow")]
#[path = "../emerging_intelligence/shadow.rs"]
mod shadow;

#[cfg(feature = "emerging-intelligence-shadow")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = shadow::run_synthetic_preregistration_probe()?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report.classification != "PREREGISTRATION_PASS" {
        return Err("EI-0G shadow preregistration probe failed".into());
    }
    Ok(())
}
