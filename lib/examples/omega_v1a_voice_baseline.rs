//! Executable ΩV1-A frozen voice baseline gate.

use star::omega_v1_voice_baseline::run_frozen_baseline;

fn main() {
    let report = match run_frozen_baseline() {
        Ok(report) => report,
        Err(error) => {
            eprintln!("ΩV1-A evaluator error: {error:#}");
            std::process::exit(1);
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .expect("serializing the frozen ΩV1-A report cannot fail")
    );

    if !report.gate_passed {
        std::process::exit(1);
    }
}
