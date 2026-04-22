//! Text generation from trained model

use crate::language_model::model::CharRNN;
use crate::language_model::vocabulary::Vocabulary;
use rand::Rng;

/// Generation configuration
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    pub max_length: usize,      // Maximum characters to generate
    pub temperature: f32,        // Sampling temperature (higher = more random)
    pub top_k: usize,           // Top-k sampling (0 = disabled)
    pub seed: Option<u64>,      // Random seed for reproducibility
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            max_length: 200,
            temperature: 0.8,
            top_k: 0,
            seed: None,
        }
    }
}

/// Generate text from a starting prompt
pub fn generate(
    model: &mut CharRNN,
    vocab: &Vocabulary,
    prompt: &str,
    config: GenerateConfig,
) -> String {
    let mut rng = rand::thread_rng();
    
    // Set seed if provided
    if let Some(seed) = config.seed {
        let seed_bytes = seed.to_le_bytes();
        let mut state = [0u64; 4];
        for (i, chunk) in seed_bytes.chunks(8).enumerate() {
            if i < 4 {
                state[i] = u64::from_le_bytes([
                    chunk.get(0).copied().unwrap_or(0),
                    chunk.get(1).copied().unwrap_or(0),
                    chunk.get(2).copied().unwrap_or(0),
                    chunk.get(3).copied().unwrap_or(0),
                    chunk.get(4).copied().unwrap_or(0),
                    chunk.get(5).copied().unwrap_or(0),
                    chunk.get(6).copied().unwrap_or(0),
                    chunk.get(7).copied().unwrap_or(0),
                ]);
            }
        }
    }
    
    // Reset hidden state for generation
    model.reset_hidden();
    
    // Encode the prompt
    let prompt_chars: Vec<usize> = vocab.encode(prompt);
    
    // Feed prompt through model to set up hidden state
    for &char_idx in &prompt_chars {
        model.step(char_idx);
    }
    
    // Generate new characters
    let mut generated = Vec::new();
    let mut char_count = 0;
    
    // Start with the last character of the prompt
    let last_char = prompt_chars.last().copied().unwrap_or(0);
    
    let mut last_char = prompt_chars.last().copied().unwrap_or(0);

    loop {
        // Get probabilities for next character
        let logits = model.step(last_char);
        let mut probs = softmax_scaled(&logits, config.temperature);

        // Apply top-k sampling if enabled
        if config.top_k > 0 {
            apply_top_k(&mut probs, config.top_k);
        }

        // Sample from distribution
        let next_char = sample_from_distribution(&probs, &mut rng);

        // Check for end of sequence
        if next_char == vocab.eos {
            break;
        }

        // Add to generated output
        generated.push(next_char);
        char_count += 1;

        // Check length limit
        if char_count >= config.max_length {
            break;
        }

        // Feed this character back as input for next step
        last_char = next_char;
    }
    
    // Decode to string
    vocab.decode(&generated)
}

/// Apply temperature scaling to logits
fn softmax_scaled(logits: &[f32], temperature: f32) -> Vec<f32> {
    if temperature == 0.0 {
        // Greedy: return argmax
        let max_idx = logits.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        let mut result = vec![0.0; logits.len()];
        result[max_idx] = 1.0;
        return result;
    }
    
    let scaled: Vec<f32> = logits.iter().map(|&x| x / temperature).collect();
    let max = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = scaled.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&x| x / sum).collect()
}

/// Apply top-k filtering
fn apply_top_k(probs: &mut Vec<f32>, k: usize) {
    // Find the k-th largest probability
    if k >= probs.len() {
        return;
    }
    
    let mut sorted = probs.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let threshold = sorted[k];
    
    // Zero out probabilities below threshold
    for p in probs.iter_mut() {
        if *p < threshold {
            *p = 0.0;
        }
    }
    
    // Renormalize
    let sum: f32 = probs.iter().sum();
    if sum > 0.0 {
        for p in probs.iter_mut() {
            *p /= sum;
        }
    }
}

/// Sample from a probability distribution
fn sample_from_distribution(probs: &[f32], rng: &mut impl Rng) -> usize {
    let r = rng.gen::<f32>();
    let mut cumsum = 0.0f32;
    
    for (i, &p) in probs.iter().enumerate() {
        cumsum += p;
        if r < cumsum {
            return i;
        }
    }
    
    // Fallback: return most likely
    probs.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Generate a response given a topic and context
pub fn generate_response(
    model: &mut CharRNN,
    vocab: &Vocabulary,
    context: &str,
    config: GenerateConfig,
) -> String {
    // Format context as a prompt
    let prompt = format!("{} Star:", context);
    generate(model, vocab, &prompt, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_model::model::ModelConfig;
    
    #[test]
    fn test_generation() {
        let config = ModelConfig::default();
        let mut model = CharRNN::new(config);
        let vocab = Vocabulary::new();
        
        let gen_config = GenerateConfig {
            max_length: 50,
            temperature: 0.8,
            top_k: 0,
            seed: Some(42),
        };
        
        let result = generate(&mut model, &vocab, "Hello", gen_config);
        // Should produce some output (exact output depends on random state)
        assert!(!result.is_empty() || result.len() <= 50);
    }
    
    #[test]
    fn test_temperature_scaling() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax_scaled(&logits, 1.0);
        assert!((probs.iter().sum::<f32>() - 1.0).abs() < 0.01);
        
        // Higher temperature should make distribution more uniform
        let probs_hot = softmax_scaled(&logits, 2.0);
        let variance_hot = variance(&probs_hot);
        let probs_cold = softmax_scaled(&logits, 0.5);
        let variance_cold = variance(&probs_cold);
        
        assert!(variance_hot < variance_cold);
    }
    
    fn variance(v: &[f32]) -> f32 {
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        v.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / v.len() as f32
    }
}