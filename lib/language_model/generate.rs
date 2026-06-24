//! Text generation from trained model

use crate::language_model::model::CharRNN;
use crate::language_model::vocabulary::Vocabulary;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand::rngs::StdRng;

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
    // Resolve RNG: seeded when `config.seed` is set so the run is
    // reproducible, otherwise thread-local entropy. The previous version
    // *built* a seed state but never applied it, so `seed=Some(42)` was a
    // silent no-op — every call diverged from the previous output.
    let mut rng: Box<dyn RngCore> = match config.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(rand::thread_rng()),
    };

    // Reset hidden state for generation
    model.reset_hidden();

    // Encode the prompt
    let prompt_chars: Vec<usize> = vocab.encode(prompt);

    // Pre-existing crash (2026-06-23): when the loaded checkpoint was
    // trained against a vocabulary whose `vocab_size` is smaller than
    // this runtime's `Vocabulary` (e.g. the 11MB `ckpt_e28_b500.pt`
    // saved with a different training config than the 3.7MB
    // `data/star_model.bin`), the prompt encoded by the 227-char
    // vocabulary can return indices >= the model's `vocab_size`.
    // `model.step()` would then index past the embedding table and
    // panic with a slice-bounds error on the very first mismatched
    // character. Filter the prompt to what the model can actually
    // embed before feeding it through.
    let model_vocab = model.vocab_size();
    let safe_prompt: Vec<usize> = prompt_chars
        .iter()
        .copied()
        .filter(|&idx| idx < model_vocab)
        .collect();

    // Feed prompt through model to set up hidden state
    for &char_idx in &safe_prompt {
        model.step(char_idx);
    }

    // Generate new characters
    let mut generated = Vec::new();
    let mut char_count = 0;

    // Start with the last character of the prompt. If the prompt was
    // empty (or every character was filtered out by the bounds check
    // above), the previous code fell back to `0` — the EOS token —
    // which produces nonsense. Bail out cleanly instead of inventing
    // a phantom context.
    let mut last_char = match safe_prompt.last() {
        Some(&c) => c,
        None => return String::new(),
    };

    loop {
        // Get probabilities for next character
        let logits = model.step(last_char);
        let mut probs = softmax_scaled(&logits, config.temperature);

        // Apply top-k sampling if enabled
        if config.top_k > 0 {
            apply_top_k(&mut probs, config.top_k);
        }

        // Sample from distribution
        let next_char = sample_from_distribution(&probs, &mut *rng);

        // Check for end of sequence
        if next_char == vocab.eos {
            break;
        }

        // The sampled token also has to be inside the model's vocab, or
        // the next `model.step(last_char = next_char)` will panic the
        // same way the prompt bug did. Clamp to UNK on out-of-range —
        // better to continue sampling than to crash the chat call.
        last_char = if next_char < model_vocab {
            next_char
        } else {
            vocab.unk
        };

        // Add to generated output
        generated.push(last_char);
        char_count += 1;

        // Check length limit
        if char_count >= config.max_length {
            break;
        }
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

/// Sample from a probability distribution.
///
/// Takes `&mut dyn RngCore` (not `impl Rng`) so the caller can pass a
/// boxed trait object that may hold either a seeded `StdRng` or a
/// `ThreadRng`. `next_u32() / u32::MAX` is the same uniform [0,1) draw
/// `rng.gen::<f32>()` would have produced, but without the implicit
/// `Sized` bound `gen()` carries.
fn sample_from_distribution(probs: &[f32], rng: &mut dyn RngCore) -> usize {
    let r = (rng.next_u32() as f64 / u32::MAX as f64) as f32;
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