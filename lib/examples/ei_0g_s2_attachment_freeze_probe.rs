#[cfg(not(feature = "emerging-intelligence-shadow-attachment"))]
fn main() {
    eprintln!("enable --features emerging-intelligence-shadow-attachment");
}

#[cfg(feature = "emerging-intelligence-shadow-attachment")]
#[path = "../emerging_intelligence/shadow_attachment.rs"]
mod shadow_attachment;

#[cfg(feature = "emerging-intelligence-shadow-attachment")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = shadow_attachment::run_attachment_freeze_probe()?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report.classification != "FREEZE_PASS" {
        return Err("EI-0G-S2 attachment freeze probe failed".into());
    }
    Ok(())
}
