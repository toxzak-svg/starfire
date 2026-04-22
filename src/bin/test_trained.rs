use star::language_model::{CharRNN, Vocabulary, generate::{self, GenerateConfig}};

fn main() {
    println!("Loading your trained model...\n");

    let vocab = Vocabulary::new();
    let model_path = "data/star_model.bin";

    let mut model = match CharRNN::load(model_path) {
        Ok(m) => {
            println!("Loaded! {} parameters", m.num_params());
            m
        }
        Err(e) => {
            eprintln!("Load failed: {}", e);
            return;
        }
    };

    println!("\nGenerating from 'Hello':");
    let config = GenerateConfig {
        max_length: 100,
        temperature: 0.8,
        top_k: 0,
        seed: Some(42),
    };
    let result = generate::generate(&mut model, &vocab, "Hello", config);
    println!("{}", result.trim());
}