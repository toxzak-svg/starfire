use star::language_model::{CharRNN, Vocabulary, generate::{self, GenerateConfig}};

fn main() {
    println!("Testing save/load cycle...\n");

    let vocab = Vocabulary::new();
    let config = star::language_model::model::ModelConfig::default();

    let mut model = CharRNN::new(config.clone());
    println!("Created model with {} parameters", model.num_params());

    let path = "data/test_model.bin";
    model.save(path).expect("Save failed");
    println!("Saved to {}", path);

    let file_size = std::fs::metadata(path).expect("Failed to get metadata").len();
    println!("File size: {} bytes", file_size);

    drop(model);

    let loaded = CharRNN::load(path).expect("Load failed");
    println!("Loaded model with {} parameters", loaded.num_params());

    println!("\nGenerating from 'Hello':");
    let gen_config = GenerateConfig {
        max_length: 50,
        temperature: 1.0,
        top_k: 0,
        seed: Some(42),
    };
    let mut loaded = loaded;
    let result = generate::generate(&mut loaded, &vocab, "Hello", gen_config);
    println!("Result: {}", result.trim());

    std::fs::remove_file(path).ok();
    println!("\nSave/load cycle works!");
}