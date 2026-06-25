//! Training binary for Star's language model
//!
//! Usage:
//!   cargo run --bin train_model -- --data data/training.txt --epochs 10

use star::language_model::{CharRNN, Vocabulary, train::{self, TrainConfig}, model::ModelConfig};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "train_model")]
#[command(about = "Train Star's character-level language model")]
struct Args {
    /// Training data file
    #[arg(long, default_value = "all_personal_training.txt")]
    data: PathBuf,

    /// Number of epochs
    #[arg(long, default_value = "10")]
    epochs: usize,

    /// Sequence length for BPTT
    #[arg(long, default_value = "128")]
    seq_length: usize,

    /// Batch size
    #[arg(long, default_value = "32")]
    batch_size: usize,

    /// Learning rate
    #[arg(long, default_value = "0.001")]
    learning_rate: f32,

    /// Hidden layer size
    #[arg(long, default_value = "256")]
    hidden_size: usize,

    /// Embedding dimension
    #[arg(long, default_value = "64")]
    embedding_dim: usize,

    /// Number of LSTM layers
    #[arg(long, default_value = "2")]
    num_layers: usize,

    /// Where to save the model
    #[arg(long, default_value = "data/star_model.bin")]
    output: PathBuf,

    /// Save model every N batches
    #[arg(long, default_value = "500")]
    save_every: usize,

    /// Resume training from an existing checkpoint at --output
    #[arg(long, default_value = "false")]
    resume: bool,
}

fn main() {
    let args = Args::parse();
    
    println!("Star Language Model Training");
    println!("============================");
    println!("Data file: {:?}", args.data);
    println!("Epochs: {}", args.epochs);
    println!("Sequence length: {}", args.seq_length);
    println!("Batch size: {}", args.batch_size);
    println!("Hidden size: {}", args.hidden_size);
    println!();
    
    // Create vocabulary
    let vocab = Vocabulary::new();
    println!("Vocabulary size: {}", vocab.size());
    
    // Load training data
    let sequences = train::parse_conversation_file(&args.data);
    println!("Loaded {} conversations", sequences.len());
    
    if sequences.is_empty() {
        eprintln!("No training sequences found!");
        std::process::exit(1);
    }
    
    // Create model configuration
    let config = ModelConfig {
        vocab_size: vocab.size(),
        embedding_dim: args.embedding_dim,
        hidden_size: args.hidden_size,
        num_layers: args.num_layers,
        dropout: 0.1,
    };
    
    println!("Creating model with {} parameters...", config.vocab_size);
    let mut model = if args.resume && args.output.exists() {
        println!("Resuming from existing checkpoint: {:?}", args.output);
        let loaded = CharRNN::load(args.output.to_string_lossy().as_ref())
            .unwrap_or_else(|e| {
                eprintln!("Failed to load checkpoint {:?}: {}", args.output, e);
                std::process::exit(1);
            });
        println!("Loaded checkpoint with {} parameters", loaded.num_params());
        loaded
    } else {
        let m = CharRNN::new(config);
        println!("Model has {} parameters", m.num_params());
        m
    };
    println!();
    
    // Create training configuration
    let train_config = TrainConfig {
        seq_length: args.seq_length,
        batch_size: args.batch_size,
        epochs: args.epochs,
        learning_rate: args.learning_rate,
        grad_clip: 5.0,
        save_every: args.save_every,
        model_path: args.output.to_string_lossy().to_string(),
    };
    
    // Train
    println!("Starting training...");
    if let Err(e) = train::train(&mut model, &sequences, &vocab, train_config) {
        eprintln!("Training failed: {}", e);
        std::process::exit(1);
    }
    
    println!("Training complete!");
}