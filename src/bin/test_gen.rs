//! Test generation from trained model

use star::language_model::{CharRNN, Vocabulary, generate::{self, GenerateConfig}};
use std::io::{Read, Write};

fn main() {
    println!("Testing trained model generation...\n");

    let vocab = Vocabulary::new();
    println!("Vocabulary size: {}", vocab.size());

    let model_path = "data/star_model_new.bin";
    let mut model = match CharRNN::load(model_path) {
        Ok(m) => {
            println!("Model loaded successfully!");
            println!("Parameters: {}\n", m.num_params());
            m
        }
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
            std::process::exit(1);
        }
    };

    let prompts = vec![
        "Zachary: hello\nStar:",
        "Zachary: what is the sky?\nStar:",
        "Zachary: how are you?\nStar:",
        "Zachary: tell me about yourself\nStar:",
    ];

    let config = GenerateConfig {
        max_length: 100,
        temperature: 0.3,
        top_k: 5,
        seed: Some(42),
    };

    println!("Generating responses with temperature={}, top_k={}, max_length={}\n",
        config.temperature, config.top_k, config.max_length);

    for prompt in prompts {
        println!("Prompt: {}", prompt.replace("\n", "\\n"));
        println!("---");

        let response = generate::generate(&mut model, &vocab, prompt, config.clone());

        let response_clean = response.lines()
            .filter(|l| !l.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");

        println!("Generated: {}", response_clean);
        println!();
    }

    println!("\n--- Interactive mode (type 'quit' to exit) ---\n");
    loop {
        print!("You: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() || input == "quit" {
            break;
        }

        let prompt = format!("Zachary: {}\nStar:", input);
        let config = GenerateConfig {
            max_length: 150,
            temperature: 0.5,
            top_k: 10,
            seed: None,
        };

        let response = generate::generate(&mut model, &vocab, &prompt, config);
        let clean = response.lines().filter(|l| !l.trim().is_empty()).take(2).collect::<Vec<_>>().join(" ");

        println!("Star: {}\n", clean);
    }
}
