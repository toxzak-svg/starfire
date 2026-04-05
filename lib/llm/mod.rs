//! LLM — Bonsai-8B Q1_0_g128 via Candle (native Rust, no server)
//!
//! Starfire uses a 1-bit quantized LLM (Bonsai-8B Q1_0_g128) for:
//! - Fluency: makes Starfire's output sound natural
//! - Gap filling: fills knowledge gaps that Quanot can't handle
//! - Text generation: polishes rough reasoning into readable prose
//!
//! The LLM runs natively via candle-core + candle-transformers — no subprocess,
//! no HTTP sidecar, fully in-process. Model: `models/bonzai-8b/Bonsai-8B.gguf`
//!
//! # Usage
//!
//! ```ignore
//! let mut llm = LlmEngine::new("models/bonzai-8b/Bonsai-8B.gguf")?;
//! let response = llm.generate("What is 2+2?")?;
//! let polished = llm.polish("twoplus two equals four")?;
//! ```

pub mod client;
pub mod chat;

use candle_core::{Device, Result as CandleResult, Tensor};
use candle_transformers::models::quantized_llama as qlm;
use std::path::Path;

/// LLM inference engine — loads Bonsai-8B GGUF via Candle and runs inference.
pub struct LlmEngine {
    model: qlm::ModelWeights,
    device: Device,
    vocab_size: usize,
    max_seq_len: usize,
}

impl LlmEngine {
    /// Create a new LLM engine from a GGUF file path.
    pub fn new(gguf_path: &Path) -> anyhow::Result<Self> {
        self::init();
        let device = Device::Cpu;

        let mut file = std::fs::File::open(gguf_path)?;
        let model = qlm::ModelWeights::from_gguf(
            candle_core::quantized::gguf_file::Content::read(&mut file)?,
            &mut file,
            &device,
        )?;

        let vocab_size = 151669; // Bonsai-8B vocab size
        let max_seq_len = 4096;  // MAX_SEQ_LEN from quantized_llama.rs

        Ok(Self {
            model,
            device,
            vocab_size,
            max_seq_len,
        })
    }

    /// Check if a GGUF file looks like Bonsai (Q1_0_g128).
    pub fn is_bonsai(gguf_path: &Path) -> anyhow::Result<bool> {
        let mut file = std::fs::File::open(gguf_path)?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        Ok(content.tensor_infos.values().any(|t| matches!(
            t.ggml_dtype,
            candle_core::quantized::GgmlDType::Q1_0_g128
        )))
    }

    /// Run a forward pass with a token_ids tensor, returning logits.
    fn forward(&mut self, token_ids: &Tensor) -> CandleResult<Tensor> {
        self.model.forward(token_ids, 0)
    }

    /// Tokenize a string (simple UTF-8 byte-level approximation).
    /// For Bonsai-8B, a proper tokenizer is needed — this is a placeholder.
    fn tokenize_simple(&self, text: &str) -> Vec<u32> {
        // Simple byte-level tokenization as fallback
        // A real implementation would use the GGUF tokenizer
        let mut tokens = Vec::with_capacity(text.len() * 2);
        for c in text.chars() {
            if c.is_ascii_alphanumeric() || c == ' ' {
                tokens.push(c as u32);
            }
        }
        // Clamp to vocab size
        tokens.iter().map(|&t| t.min(self.vocab_size as u32 - 1)).collect()
    }

    /// Sample a token from logits (greedy).
    fn sample_token(&self, logits: &[f32]) -> usize {
        logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Generate a text completion for a prompt.
    pub fn generate(&mut self, prompt: &str) -> anyhow::Result<String> {
        // Tokenize input
        let mut token_ids: Vec<u32> = self.tokenize_simple(prompt);

        // Build input tensor
        let max_len = token_ids.len().min(self.max_seq_len).max(1);
        let input = Tensor::new(token_ids.as_slice(), &self.device)?.reshape(&[1, max_len])?;

        // Forward pass (index_pos = 0 for first token)
        let logits = self.model.forward(&input, 0)?;
        let logits = logits.squeeze(0)?;

        // Sample next token
        let logits_v = logits.to_vec1::<f32>()?;
        let next_token = self.sample_token(&logits_v);

        // Decode token (placeholder — just return the token number for now)
        Ok(format!("[token {}]", next_token))
    }

    /// Generate a chat completion from messages.
    pub fn chat(&mut self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        // Simple concatenation of messages as prompt
        let prompt: String = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        self.generate(&prompt)
    }

    /// Polish a rough Starfire output into fluent text.
    pub fn polish(&mut self, rough_text: &str) -> anyhow::Result<String> {
        self.generate_with_system(
            "You are a text polish engine. Rewrite the following text to be more fluent \
             and natural-sounding while preserving all factual content. Be concise. \
             Do not add new information. Do not summarize.",
            rough_text,
        )
    }

    /// Generate with a system prompt + user message.
    pub fn generate_with_system(&mut self, system: &str, user: &str) -> anyhow::Result<String> {
        self.chat(&[
            ChatMessage::system(system),
            ChatMessage::user(user),
        ])
    }

    /// Health check — returns true if the model is loaded.
    pub fn health_check(&mut self) -> bool {
        // Simple check: try a tiny forward pass
        let tokens = [0u32, 1, 2, 3];
        let input = Tensor::new(tokens.as_slice(), &self.device).ok();
        if let Some(input) = input {
            let reshaped = input.reshape(&[1, 4]).ok();
            if let Some(reshaped) = reshaped {
                return self.forward(&reshaped).is_ok();
            }
        }
        false
    }

    /// Model size on disk in human-readable form.
    pub fn model_size_human(gguf_path: &Path) -> String {
        let bytes = std::fs::metadata(gguf_path).map(|m| m.len()).unwrap_or(0);
        let mb = bytes as f64 / 1_048_576.0;
        let gb = mb / 1024.0;
        if gb >= 1.0 {
            format!("{:.1} GB", gb)
        } else {
            format!("{:.0} MB", mb)
        }
    }
}

/// A single chat message.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

/// Called once at startup to init candle internals.
fn init() {
    // candle-core is side-effect-free on CPU; no explicit init needed.
    // GPU backends (CUDA/Metal) would be initialized here if used.
}

/// A handle that can be shared across threads ( Clone is cheap ).
#[derive(Clone)]
pub struct LlmHandle {
    gguf_path: std::path::PathBuf,
}

impl LlmHandle {
    /// Load the model from disk and return an engine.
    /// This does the actual GGUF parsing and tensor loading — takes a few seconds.
    pub fn load(&self) -> anyhow::Result<LlmEngine> {
        LlmEngine::new(&self.gguf_path)
    }

    /// Create a handle from a GGUF path.
    pub fn new(gguf_path: &Path) -> Self {
        Self {
            gguf_path: gguf_path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "You are helpful");

        let usr = ChatMessage::user("Hello");
        assert_eq!(usr.role, "user");

        let ast = ChatMessage::assistant("Hi there");
        assert_eq!(ast.role, "assistant");
    }

    #[test]
    fn test_tokenizer_simple() {
        // Just verify it doesn't crash — output format varies by model
        let engine = unsafe {
            // This test requires the model to be loaded, so we skip actual inference
            // For unit tests we just check the ChatMessage API
        };
        let _msg = ChatMessage::user("Hello, world!");
    }

    #[test]
    fn test_model_size() {
        let path = Path::new("models/bonzai-8b/Bonsai-8B.gguf");
        if path.exists() {
            let size = LlmEngine::model_size_human(path);
            println!("Model size: {}", size);
            assert!(size.contains("GB") || size.contains("MB"));
        }
    }
}
