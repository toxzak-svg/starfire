//! Minimal test - just try loading and generating from the smallest possible model

use star::language_model::{CharRNN, Vocabulary, generate::{self, GenerateConfig}, model::ModelConfig};
use std::fs;

fn main() {
    println!("Testing model load and generate...\n");

    let vocab = Vocabulary::new();
    println!("Vocab size: {}", vocab.size());

    // Check model file size
    let model_path = "data/star_model.bin";
    match fs::metadata(model_path) {
        Ok(meta) => println!("Model file size: {} bytes", meta.len()),
        Err(e) => {
            eprintln!("No model file found: {}", e);
            return;
        }
    }

    // Load model - wrap in smaller scope to limit memory
    let mut model = match CharRNN::load(model_path) {
        Ok(m) => {
            println!("Model loaded! {} parameters", m.num_params());
            m
        }
        Err(e) => {
            eprintln!("Load failed: {}", e);
            return;
        }
    };

    println!("\nGenerating from prompt: 'Hello'");

    let config = GenerateConfig {
        max_length: 30,  // very short
        temperature: 1.0,
        top_k: 0,
        seed: Some(123),
    };

    let result = generate::generate(&mut model, &vocab, "Hello", config);
    println!("Result: {}", result.trim());
}