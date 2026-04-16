//! IntentionCNN — classifies "I'm X" utterances into intent categories.
//!
//! Handles: name introduction, state reports, apologies, intents, and chitchat.
//! Loaded behind the `llm` feature gate (candle-core + candle-nn inference).

#[cfg(feature = "llm")]
use candle_core::{Module, Result as CResult, Tensor, Device};

/// Intention categories from the CNN classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImIntention {
    Name     = 0,
    State    = 1,
    Apology  = 2,
    Intent   = 3,
    Chitchat = 4,
    Other    = 5,
}

impl ImIntention {
    pub fn from_idx(idx: usize) -> Self {
        match idx {
            0 => ImIntention::Name,
            1 => ImIntention::State,
            2 => ImIntention::Apology,
            3 => ImIntention::Intent,
            4 => ImIntention::Chitchat,
            _ => ImIntention::Other,
        }
    }
}

/// Returns true if text matches the "I'm X" pattern.
pub fn is_im_utterance(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.starts_with("i'm ") || lower.starts_with("im ") || lower.starts_with("i am ")
}

/// Extract the name after "I'm" or "I am".
pub fn extract_name(text: &str) -> Option<String> {
    let text = text.trim();
    let prefix_and_rest = if text.to_lowercase().starts_with("i'm ") {
        text.strip_prefix("I'm ").or(text.strip_prefix("i'm "))
    } else if text.to_lowercase().starts_with("im ") {
        text.strip_prefix("Im ").or(text.strip_prefix("im "))
    } else if text.to_lowercase().starts_with("i am ") {
        text.strip_prefix("I am ").or(text.strip_prefix("i am "))
    } else {
        None
    };

    prefix_and_rest.and_then(|rest| {
        let name = rest.split_whitespace().next().unwrap_or(rest)
            .trim_matches(|c: char| c.is_ascii_punctuation() && c != '\'');
        if name.is_empty() || name.len() > 30 { None } else { Some(name.to_string()) }
    })
}

// ─── Model architecture (matches intention_cnn.ipynb) ───────────────────────

const VOCAB_SIZE: usize = 76;
const EMBED_DIM: usize = 64;
const SEQ_LEN: usize = 48;

// Character vocabulary (must match the order used during training)
const VOCAB: &str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'\"-.,!?;:\n\t0123456789";

fn char_to_idx(c: char) -> usize {
    VOCAB.find(c).unwrap_or(0)
}

/// Character-level tokenization matching the Python encoder.
fn encode(text: &str) -> Vec<u32> {
    let mut ids: Vec<u32> = text.chars().map(|c| char_to_idx(c) as u32).collect();
    if ids.len() >= SEQ_LEN {
        return ids[..SEQ_LEN].to_vec();
    }
    ids.resize(SEQ_LEN, 0);
    ids
}

/// IntentionCNN with weights stored as raw tensors (no candle-nn layer wrappers).
#[cfg(feature = "llm")]
struct IntentionCNN {
    embedding_w: Tensor,
    // conv weights: (out_channels, in_channels, kernel_size)
    conv3_w: Tensor,
    conv3_b: Tensor,
    conv4_w: Tensor,
    conv4_b: Tensor,
    conv5_w: Tensor,
    conv5_b: Tensor,
    // batch norm: running stats + affine params
    bn3_mean: Tensor,
    bn3_var: Tensor,
    bn3_gamma: Tensor,
    bn3_beta: Tensor,
    bn4_mean: Tensor,
    bn4_var: Tensor,
    bn4_gamma: Tensor,
    bn4_beta: Tensor,
    bn5_mean: Tensor,
    bn5_var: Tensor,
    bn5_gamma: Tensor,
    bn5_beta: Tensor,
    fc1_w: Tensor,
    fc1_b: Tensor,
    fc2_w: Tensor,
    fc2_b: Tensor,
}

#[cfg(feature = "llm")]
impl IntentionCNN {
    /// Load from a safetensors file path.
    fn load(path: &std::path::Path) -> CResult<Self> {
        let tensors = candle_core::safetensors::load(path, &Device::Cpu)?;
        let t = |name: &str| {
            tensors.get(name).cloned()
                .ok_or_else(|| candle_core::Error::Msg(format!("missing tensor: {name}")))
        };
        Ok(Self {
            embedding_w: t("embedding.weight")?,
            conv3_w: t("conv3.weight")?,
            conv3_b: t("conv3.bias")?,
            conv4_w: t("conv4.weight")?,
            conv4_b: t("conv4.bias")?,
            conv5_w: t("conv5.weight")?,
            conv5_b: t("conv5.bias")?,
            bn3_mean: t("bn3.running_mean")?,
            bn3_var: t("bn3.running_var")?,
            bn3_gamma: t("bn3.weight")?,
            bn3_beta: t("bn3.bias")?,
            bn4_mean: t("bn4.running_mean")?,
            bn4_var: t("bn4.running_var")?,
            bn4_gamma: t("bn4.weight")?,
            bn4_beta: t("bn4.bias")?,
            bn5_mean: t("bn5.running_mean")?,
            bn5_var: t("bn5.running_var")?,
            bn5_gamma: t("bn5.weight")?,
            bn5_beta: t("bn5.bias")?,
            fc1_w: t("fc1.weight")?,
            fc1_b: t("fc1.bias")?,
            fc2_w: t("fc2.weight")?,
            fc2_b: t("fc2.bias")?,
        })
    }

    /// Inline batch-norm inference pass: (x - mean) / sqrt(var + eps) * gamma + beta
    fn bn(&self, x: &Tensor, mean: &Tensor, var: &Tensor, gamma: &Tensor, beta: &Tensor) -> CResult<Tensor> {
        let target_shape: Vec<usize> = x
            .dims()
            .iter()
            .enumerate()
            .map(|(idx, v)| if idx == 1 { *v } else { 1 })
            .collect();
        let ts = target_shape.as_slice();
        let eps = 1e-5f64;
        x.broadcast_sub(&mean.reshape(ts)?)?
            .broadcast_div(&((var.reshape(ts)? + eps)?.sqrt()?))?
            .broadcast_mul(&gamma.reshape(ts)?)?
            .broadcast_add(&beta.reshape(ts)?)
    }

    /// Max-pool then squeeze the last two dims to (batch, channels).
    fn maxpool_squeeze(&self, x: &Tensor) -> CResult<Tensor> {
        x.max_keepdim(2)?.squeeze(2)?.squeeze(1)
    }
}

#[cfg(feature = "llm")]
impl Module for IntentionCNN {
    fn forward(&self, x: &Tensor) -> CResult<Tensor> {
        // x: (batch, seq_len)
        let x = self.embedding_w.embedding(&x)?; // (batch, seq_len, embed_dim)
        let x = x.permute((0, 2, 1))?; // (batch, embed_dim, seq_len)

        // Conv branch 3 — kernel_size=3, padding=1
        let c3 = x.conv1d(&self.conv3_w, 1, 1, 1, 1)?.broadcast_add(&self.conv3_b.reshape(&[128usize])?)?;
        let c3 = self.bn(&c3, &self.bn3_mean, &self.bn3_var, &self.bn3_gamma, &self.bn3_beta)?;
        let c3 = c3.relu()?;
        let c3 = self.maxpool_squeeze(&c3)?; // (batch, 128)

        // Conv branch 4 — kernel_size=4, padding=2
        let c4 = x.conv1d(&self.conv4_w, 2, 1, 1, 1)?.broadcast_add(&self.conv4_b.reshape(&[128usize])?)?;
        let c4 = self.bn(&c4, &self.bn4_mean, &self.bn4_var, &self.bn4_gamma, &self.bn4_beta)?;
        let c4 = c4.relu()?;
        let c4 = self.maxpool_squeeze(&c4)?;

        // Conv branch 5 — kernel_size=5, padding=2
        let c5 = x.conv1d(&self.conv5_w, 2, 1, 1, 1)?.broadcast_add(&self.conv5_b.reshape(&[128usize])?)?;
        let c5 = self.bn(&c5, &self.bn5_mean, &self.bn5_var, &self.bn5_gamma, &self.bn5_beta)?;
        let c5 = c5.relu()?;
        let c5 = self.maxpool_squeeze(&c5)?;

        // Concatenate, ReLU, fc1, ReLU, fc2
        let x = Tensor::cat(&[&c3, &c4, &c5], 1)?; // (batch, 384)
        let x = x.relu()?;
        let x = x.matmul(&self.fc1_w)?.broadcast_add(&self.fc1_b)?;
        let x = x.relu()?;
        let x = x.matmul(&self.fc2_w)?.broadcast_add(&self.fc2_b)?;
        Ok(x)
    }
}

/// Raw rule-based fallback — always available.
fn rule_based_classify(text: &str) -> (ImIntention, f32) {
    let lower = text.to_lowercase();

    // Handle "I apologize" / "I'm sorry" → apology
    if lower.contains("apologize") || lower.contains("sorry") {
        return (ImIntention::Apology, 0.85);
    }

    // Handle "I will X" / "I'll X" / "I am going to X" → intent
    if lower.starts_with("i will ") || lower.starts_with("i'll ") || lower.starts_with("i am going to ") || lower.starts_with("i'm going to ") {
        return (ImIntention::Intent, 0.85);
    }

    if is_im_utterance(text) {
        // Phrases that look like conversational fillers — NOT name, NOT state, but chitchat
        let chitchat_phrases = ["just saying", "kidding", "joking", "being sarcastic"];
        if chitchat_phrases.iter().any(|p| lower.contains(p)) {
            return (ImIntention::Chitchat, 0.7);
        }

        if let Some(name) = extract_name(text) {
            // Treat as Name only if it starts with uppercase AND is NOT a common state word
            let state_words = ["tired", "happy", "sad", "angry", "scared", "sick", "cold", "hot", "hungry", "thirsty", "sleepy", "bored", "fine", "okay", "alright", "good", "great", "bad", "wrong"];
            let starts_uppercase = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            let is_state = state_words.contains(&name.to_lowercase().as_str());
            if starts_uppercase && !is_state {
                return (ImIntention::Name, 0.9);
            }
        }
        // Otherwise it's a state report
        (ImIntention::State, 0.75)
    } else {
        (ImIntention::Other, 0.0)
    }
}

/// IntentionCNN classifier — loads safetensors weights behind the `llm` feature.
#[cfg(feature = "llm")]
pub struct ImIntentionClassifier {
    model: Option<IntentionCNN>,
}

#[cfg(feature = "llm")]
impl ImIntentionClassifier {
    pub fn new() -> CResult<Self> {
        Ok(Self { model: None })
    }

    /// Load model weights from a safetensors file.
    pub fn load(&mut self, path: &std::path::Path) -> CResult<()> {
        let model = IntentionCNN::load(path)?;
        self.model = Some(model);
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    pub fn classify(&self, text: &str) -> (ImIntention, f32) {
        let model = match &self.model {
            Some(m) => m,
            None => return rule_based_classify(text),
        };

        let ids = encode(text);
        let ids_tensor = match Tensor::new(ids.as_slice(), &Device::Cpu) {
            Ok(t) => t,
            Err(_) => return rule_based_classify(text),
        };
        let input = match ids_tensor.reshape((1, SEQ_LEN)) {
            Ok(t) => t,
            Err(_) => return rule_based_classify(text),
        };

        let output = match model.forward(&input) {
            Ok(o) => o,
            Err(_) => return rule_based_classify(text),
        };

        // Compute softmax probabilities
        let probs = match candle_nn::ops::softmax(&output, 1) {
            Ok(p) => p,
            Err(_) => return rule_based_classify(text),
        };

        // Get argmax and confidence
        let idx_tensor = match probs.argmax(1) {
            Ok(t) => t,
            Err(_) => return rule_based_classify(text),
        };
        let idx = match idx_tensor.to_scalar::<f32>() {
            Ok(v) => v as usize,
            Err(_) => return rule_based_classify(text),
        };
        // Extract probability for predicted class
        let probs_flat = match probs.flatten_all() {
            Ok(f) => f,
            Err(_) => return rule_based_classify(text),
        };
        let probs_vec: Vec<f32> = match probs_flat.to_vec1() {
            Ok(v) => v,
            Err(_) => return rule_based_classify(text),
        };
        let confidence = probs_vec.get(idx).copied().unwrap_or(0.0);

        (ImIntention::from_idx(idx), confidence)
    }
}

/// Stub classifier — rule-based fallback when `llm` feature is disabled.
#[cfg(not(feature = "llm"))]
pub struct ImIntentionClassifier;

#[cfg(not(feature = "llm"))]
impl ImIntentionClassifier {
    pub fn new() -> Result<Self, candle_core::Error> {
        Ok(Self)
    }
    pub fn load(&self, _: &std::path::Path) -> Result<(), candle_core::Error> {
        Ok(())
    }
    pub fn is_loaded(&self) -> bool {
        false
    }
    pub fn classify(&self, text: &str) -> (ImIntention, f32) {
        rule_based_classify(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    struct TestCase {
        input: &'static str,
        expected: ImIntention,
    }

    // Load the real model once for all ML inference tests
    static ML_CLF: LazyLock<ImIntentionClassifier> = LazyLock::new(|| {
        ImIntentionClassifier::new().expect("failed to load intention_cnn model")
    });

    fn run_cases(cases: &[TestCase]) {
        let classifier = &*ML_CLF;
        for tc in cases {
            let (result, _conf) = classifier.classify(tc.input);
            assert_eq!(
                result, tc.expected,
                "classify(\"{}\") = {:?}, expected {:?}",
                tc.input, result, tc.expected
            );
        }
    }

    #[test]
    fn test_im_utterance_detection() {
        assert!(is_im_utterance("I'm John"));
        assert!(is_im_utterance("im going"));
        assert!(is_im_utterance("I am tired"));
        assert!(!is_im_utterance("Hello there"));
        assert!(!is_im_utterance("What's up"));
    }

    #[test]
    fn test_name_extraction() {
        assert_eq!(extract_name("I'm John"), Some("John".to_string()));
        assert_eq!(extract_name("I'm Sarah O'Connor"), Some("Sarah".to_string()));
        assert_eq!(extract_name("i'm alex"), Some("alex".to_string()));
        assert_eq!(extract_name("I am tired"), Some("tired".to_string()));
        assert_eq!(extract_name("Hello"), None);
    }

    #[test]
    fn test_name_intention() {
        let cases = &[
            TestCase { input: "I'm John",         expected: ImIntention::Name },
            TestCase { input: "I'm Sarah",        expected: ImIntention::Name },
            TestCase { input: "I'm O'Connor",     expected: ImIntention::Name },
            TestCase { input: "Im Zachary",       expected: ImIntention::Name },
            TestCase { input: "i am Max",          expected: ImIntention::Name },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_state_intention() {
        let cases = &[
            TestCase { input: "I'm tired",        expected: ImIntention::State },
            TestCase { input: "I'm happy",         expected: ImIntention::State },
            TestCase { input: "I'm frustrated",    expected: ImIntention::State },
            TestCase { input: "I'm hungry",        expected: ImIntention::State },
            TestCase { input: "Im cold",           expected: ImIntention::State },
            TestCase { input: "i am busy",         expected: ImIntention::State },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_apology_intention() {
        let cases = &[
            TestCase { input: "I'm sorry",         expected: ImIntention::Apology },
            TestCase { input: "I'm so sorry",      expected: ImIntention::Apology },
            TestCase { input: "I apologize",       expected: ImIntention::Apology },
            TestCase { input: "I'm sorry about that", expected: ImIntention::Apology },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_intent_intention() {
        let cases = &[
            TestCase { input: "I'm going to the store",  expected: ImIntention::Intent },
            TestCase { input: "I'm going to try",        expected: ImIntention::Intent },
            TestCase { input: "I will go",               expected: ImIntention::Intent },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_other_intention() {
        let cases = &[
            TestCase { input: "I'm just saying",    expected: ImIntention::Chitchat },
            TestCase { input: "What's the plan",    expected: ImIntention::Other },
            TestCase { input: "Hello",              expected: ImIntention::Other },
            TestCase { input: "How are you",         expected: ImIntention::Other },
        ];
        run_cases(cases);
    }
}
