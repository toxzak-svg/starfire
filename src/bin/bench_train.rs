//! Quick micro-benchmark for the language model training loop.
//!
//! Usage:
//!   cargo run --release --bin bench_train -- [batches]
//!
//! Runs `batches` mini-batches (default 5) on the default config and prints
//! wall-clock per-batch timings. Compares forward+backward+apply timing
//! before and after the flat-storage refactor.

use star::language_model::model::{CharRNN, ModelConfig};
use star::language_model::vocabulary::Vocabulary;
use std::time::Instant;

fn main() {
    let batches: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let config = ModelConfig::default();
    let vocab = Vocabulary::new();
    let mut model = CharRNN::new(config.clone());

    println!("Config: {:?}", config);
    println!("Params: {}", model.num_params());
    println!("Batches: {}", batches);
    println!();

    // Synthesize a deterministic training corpus
    let seq_length = 128;
    let batch_size = 32;
    let total_seqs = batch_size * batches + 4; // small surplus
    let sequences: Vec<Vec<usize>> = (0..total_seqs)
        .map(|i| {
            (0..seq_length)
                .map(|j| (i * 31 + j * 7 + 13) % vocab.size())
                .collect()
        })
        .collect();

    // Warm-up (drop)
    {
        model.reset_hidden();
        let acts = model.forward_sequence(&sequences[0]);
        let grads = model.backward_sequence(&sequences[0], &acts, &sequences[0]);
        model.apply_gradients(&grads, 0.001, 5.0);
    }

    let mut forward_total = std::time::Duration::ZERO;
    let mut backward_total = std::time::Duration::ZERO;
    let mut apply_total = std::time::Duration::ZERO;

    for batch_idx in 0..batches {
        let start = batch_start(batch_idx * batch_size, batch_size, &sequences);
        let end = (start + batch_size).min(sequences.len());

        model.reset_hidden();

        // Forward + backward + apply per sequence (matches train.rs shape)
        let f_start = Instant::now();
        let mut acts = model.forward_sequence(&sequences[start]);
        forward_total += f_start.elapsed();

        let b_start = Instant::now();
        let grads = model.backward_sequence(&sequences[start], &acts, &sequences[start]);
        backward_total += b_start.elapsed();

        let a_start = Instant::now();
        model.apply_gradients(&grads, 0.001, 5.0);
        apply_total += a_start.elapsed();

        // Touch the rest of the batch (sequentially, like train.rs)
        for s in start + 1..end {
            let f = Instant::now();
            acts = model.forward_sequence(&sequences[s]);
            forward_total += f.elapsed();

            let b = Instant::now();
            let g = model.backward_sequence(&sequences[s], &acts, &sequences[s]);
            backward_total += b.elapsed();

            let a = Instant::now();
            model.apply_gradients(&g, 0.001, 5.0);
            apply_total += a.elapsed();
        }
    }

    let total = forward_total + backward_total + apply_total;
    let seqs = batch_size * batches;
    let per_seq = total / seqs as u32;
    let per_batch = total / batches as u32;

    println!("=== Timing ({} sequences, {} batches) ===", seqs, batches);
    println!("forward_sequence:  {:?}", forward_total);
    println!("backward_sequence: {:?}", backward_total);
    println!("apply_gradients:   {:?}", apply_total);
    println!("total:             {:?}", total);
    println!();
    println!("per sequence:      {:?}", per_seq);
    println!("per batch (32 seq): {:?}", per_batch);
}

fn batch_start(i: usize, _bs: usize, seqs: &[Vec<usize>]) -> usize {
    i.min(seqs.len().saturating_sub(1))
}