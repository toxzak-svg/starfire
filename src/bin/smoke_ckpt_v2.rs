//! Smoke test for ckpt_e28_b500_v2.bin
//!
//! Loads the converted PyTorch checkpoint (via convert_to_rust.py) and
//! verifies that:
//!   1. The Rust loader accepts it cleanly (no panic, no allocation blow-up).
//!   2. The reported num_params matches the documented config:
//!        vocab=227, embed=64, hidden=256, layers=2 → 926,883 parameters.
//!   3. vocab_size() reports 227.
//!   4. A forward pass + sampling round produces non-empty output.
//!
//! Run with: cargo run --bin smoke_ckpt_v2 -- models/ckpt_e28_b500_v2.bin

use star::language_model::{CharRNN, Vocabulary, generate::{self, GenerateConfig}};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let model_path = args.get(1).cloned().unwrap_or_else(|| {
        "models/ckpt_e28_b500_v2.bin".to_string()
    });

    println!("=== smoke_ckpt_v2 ===");
    println!("Loading: {}", model_path);

    // Expected for documented config (vocab=227, embed=64, hidden=256, layers=2):
    //   embedding          = 227 * 64                 = 14,528
    //   lstm layer 0       = 4*256*(64+256) + 4*256   = 328,704
    //   lstm layer 1       = 4*256*(256+256) + 4*256  = 525,312
    //   output (W + b)     = 256*227 + 227            = 58,339
    //   TOTAL                                       = 926,883
    const EXPECTED_PARAMS: usize = 926_883;
    const EXPECTED_VOCAB: usize = 227;

    let mut model = match CharRNN::load(&model_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("LOAD FAILED: {} (kind: {:?})", e, e.kind());
            return ExitCode::from(1);
        }
    };

    let reported = model.num_params();
    let vocab = model.vocab_size();

    println!("Reported num_params: {}", reported);
    println!("Expected (default cfg): {}", EXPECTED_PARAMS);
    println!("vocab_size(): {}", vocab);

    if vocab != EXPECTED_VOCAB {
        eprintln!("FAIL: vocab_size = {} but expected {}", vocab, EXPECTED_VOCAB);
        return ExitCode::from(2);
    }
    println!("[ok] vocab_size matches expected ({})", EXPECTED_VOCAB);

    if reported != EXPECTED_PARAMS {
        eprintln!("FAIL: num_params = {} but expected {}", reported, EXPECTED_PARAMS);
        return ExitCode::from(3);
    }
    println!("[ok] num_params matches expected");

    // Smoke-generate a short response to confirm forward pass works end-to-end.
    let vocab_map = Vocabulary::new();
    let cfg = GenerateConfig {
        max_length: 30,
        temperature: 0.8,
        top_k: 0,
        seed: Some(42),
    };
    let result = generate::generate(&mut model, &vocab_map, "Zachary: hi\nStar:", cfg);
    println!("Sample generation (seed=42): {:?}", result);

    if result.is_empty() {
        eprintln!("FAIL: generation produced empty output");
        return ExitCode::from(4);
    }
    println!("[ok] generation produced output");

    println!("\nALL CHECKS PASSED");
    ExitCode::SUCCESS
}