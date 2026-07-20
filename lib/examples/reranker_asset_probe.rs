//! Render asset probe for the bundled Star identity and trained voice checkpoint.
//!
//! This proves the files shipped in the Docker build context are present and
//! that the checkpoint can be parsed by the exact loader used by `Runtime`.

use serde_json::json;
use star::language_model::{CharRnnBackend, RerankerBackend};
use std::path::Path;

fn main() {
    let identity_path = Path::new("IDENTITY.md");
    let checkpoint_path = Path::new("models/ckpt_e28_b500.pt");

    let identity = std::fs::read_to_string(identity_path)
        .expect("bundled IDENTITY.md must be readable during the Render build");
    assert!(identity.contains("# IDENTITY.md — Star"));
    assert!(identity.contains("**Name:** Star"));

    let checkpoint_size = std::fs::metadata(checkpoint_path)
        .expect("bundled reranker checkpoint must exist during the Render build")
        .len();
    assert!(checkpoint_size > 1_000_000, "checkpoint is unexpectedly small");

    let backend = CharRnnBackend::load_from_checkpoint(checkpoint_path)
        .expect("bundled reranker checkpoint must load with CharRnnBackend");

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gate_passed": backend.name() == "char_rnn",
            "identity_present": true,
            "identity_is_full": true,
            "checkpoint_present": true,
            "checkpoint_size_bytes": checkpoint_size,
            "checkpoint_loadable": true,
            "backend": backend.name(),
        }))
        .expect("serialize asset probe report")
    );
}
