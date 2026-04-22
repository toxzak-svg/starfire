//! Training pipeline for character-level language model

use crate::language_model::model::{CharRNN, ModelConfig};
use crate::language_model::vocabulary::Vocabulary;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Training hyperparameters
#[derive(Debug, Clone)]
pub struct TrainConfig {
    pub seq_length: usize,      // Sequence length for truncated BPTT
    pub batch_size: usize,      // Number of sequences per batch
    pub epochs: usize,          // Training epochs
    pub learning_rate: f32,    // Learning rate
    pub grad_clip: f32,        // Gradient clipping threshold
    pub save_every: usize,     // Save model every N batches
    pub model_path: String,    // Where to save the model
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            seq_length: 128,
            batch_size: 32,
            epochs: 10,
            learning_rate: 0.001,
            grad_clip: 5.0,
            save_every: 500,
            model_path: "data/star_model.bin".to_string(),
        }
    }
}

/// Parse conversation data from training file
/// Format: "Zachary: message\nStar: response\n"
pub fn parse_conversation_file(path: &Path) -> Vec<String> {
    let file = File::open(path).expect("Failed to open training file");
    let reader = BufReader::new(file);
    let mut sequences = Vec::new();
    let mut current = String::new();
    
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        
        // Skip empty lines
        if line.trim().is_empty() {
            if !current.is_empty() {
                sequences.push(current.clone());
                current.clear();
            }
            continue;
        }
        
        // Transform labels: "user:" -> "Zachary:", "assistant:" -> "Star:"
        let transformed = if line.starts_with("user:") {
            line.replacen("user:", "Zachary:", 1)
        } else if line.starts_with("assistant:") {
            line.replacen("assistant:", "Star:", 1)
        } else {
            line
        };
        
        current.push_str(&transformed);
        current.push('\n');
    }
    
    if !current.is_empty() {
        sequences.push(current);
    }
    
    sequences
}

/// Prepare training data from sequences
pub fn prepare_training_data(
    sequences: &[String],
    vocab: &Vocabulary,
    seq_length: usize,
) -> Vec<Vec<usize>> {
    let mut sequences_encoded: Vec<Vec<usize>> = Vec::new();
    
    for seq in sequences {
        let encoded = vocab.encode_with_eos(seq);
        
        // If sequence is longer than seq_length, chunk it
        if encoded.len() > seq_length {
            for chunk in encoded.chunks(seq_length) {
                let mut padded = chunk.to_vec();
                if padded.len() < seq_length {
                    padded.resize(seq_length, vocab.pad);
                }
                sequences_encoded.push(padded);
            }
        } else {
            // Pad shorter sequences
            let mut padded = encoded;
            padded.resize(seq_length, vocab.pad);
            sequences_encoded.push(padded);
        }
    }
    
    sequences_encoded
}

/// Compute cross-entropy loss
pub fn cross_entropy_loss(logits: &[f32], target: usize) -> f32 {
    // Softmax is already applied, so just negative log likelihood
    let exp_sum: f32 = logits.iter()
        .map(|&x| x.exp())
        .sum();
    
    let target_logit = logits[target];
    -(target_logit - exp_sum.ln())
}

/// Simple gradient descent update
pub fn sgd_update(param: &mut [f32], grad: &[f32], lr: f32) {
    for (p, g) in param.iter_mut().zip(grad.iter()) {
        *p -= lr * g;
    }
}

/// Train the model using BPTT
pub fn train(
    model: &mut CharRNN,
    sequences: &[String],
    vocab: &Vocabulary,
    config: TrainConfig,
) -> Result<(), String> {
    let training_data = prepare_training_data(sequences, vocab, config.seq_length);

    if training_data.is_empty() {
        return Err("No training data available".to_string());
    }

    println!("Training with {} sequences", training_data.len());
    println!("Learning rate: {}, Gradient clip: {}", config.learning_rate, config.grad_clip);

    for epoch in 0..config.epochs {
        let mut total_loss = 0.0f32;
        let mut num_sequences = 0;

        // Shuffle training data
        let mut indices: Vec<usize> = (0..training_data.len()).collect();
        use rand::seq::SliceRandom;
        indices.shuffle(&mut rand::thread_rng());

        for batch_start in (0..training_data.len()).step_by(config.batch_size) {
            let batch_end = (batch_start + config.batch_size).min(training_data.len());
            let batch_size_actual = batch_end - batch_start;

            // Reset hidden state for each new batch
            model.reset_hidden();

            // Accumulate gradients over batch
            for &seq_idx in &indices[batch_start..batch_end] {
                let seq = &training_data[seq_idx];

                // Forward pass through sequence
                let activations = model.forward_sequence(seq);

                // Compute loss
                let mut seq_loss = 0.0f32;
                for t in 0..seq.len().saturating_sub(1) {
                    let probs = softmax(&activations.output_logits[t]);
                    let target = seq[t + 1];
                    seq_loss += -probs[target].ln();
                }
                total_loss += seq_loss;
                num_sequences += 1;

                // Backward pass
                let gradients = model.backward_sequence(seq, &activations, seq);

                // Apply gradients immediately (simpler than accumulating)
                model.apply_gradients(&gradients, config.learning_rate, config.grad_clip);
            }

            let avg_loss = total_loss / num_sequences as f32;
            let batch_num = batch_start / config.batch_size;
            let total_batches = training_data.len() / config.batch_size;

            if batch_num % 10 == 0 || batch_start + batch_size_actual >= training_data.len() {
                println!(
                    "Epoch {} Batch {}/{} Loss: {:.4}",
                    epoch + 1,
                    batch_num,
                    total_batches,
                    avg_loss
                );
            }

            // Save periodically
            if batch_start > 0 && batch_start % config.save_every < batch_size_actual {
                if let Err(e) = model.save(&config.model_path) {
                    eprintln!("Failed to save model: {}", e);
                } else {
                    println!("Model saved to {}", config.model_path);
                }
            }
        }

        let avg_epoch_loss = total_loss / num_sequences as f32;
        println!("Epoch {} complete. Avg loss: {:.4}", epoch + 1, avg_epoch_loss);
    }

    // Final save
    if let Err(e) = model.save(&config.model_path) {
        eprintln!("Failed to save final model: {}", e);
    } else {
        println!("Final model saved to {}", config.model_path);
    }

    Ok(())
}

/// Minimal softmax for training
fn softmax(v: &[f32]) -> Vec<f32> {
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = v.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&x| x / sum).collect()
}