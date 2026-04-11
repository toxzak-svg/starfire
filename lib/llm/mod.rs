//! LLM — Bonsai-8B Q1_0_g128 via Candle (native Rust, no server)
//!
//! Starfire uses a 1-bit quantized LLM (Bonsai-8B Q1_0_g128) for:
//! - Fluency: makes Starfire's output sound natural
//! - Gap filling: fills knowledge gaps that Quanot can't handle
//! - Text generation: polishes rough reasoning into readable prose
//!
//! The LLM runs natively via candle-core + candle-transformers — no subprocess,
//! no HTTP sidecar, fully in-process. Model: `models/bonsai-8b/Bonsai-8B.gguf`
//!
//! # Architecture
//!
//! - Tokenizer: loaded from GGUF metadata via `TokenizerFromGguf` trait
//! - Model: quantized GPT-2/BPE vocabulary, 254 Q1_0_g128 tensors
//! - Generation: autoregressive with KV-cache, temperature + top-p sampling
//! - Polish pipeline: rough KG output → model → fluent natural text

pub mod client;
pub mod chat;
pub mod polish;

use anyhow::{Context as AnyhowContext, Result as AnyhowResult};
use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_core::quantized::tokenizer::TokenizerFromGguf;
use candle_transformers::models::quantized_qwen3 as qlm;
use candle_transformers::generation::LogitsProcessor;
use std::path::Path;
use tokenizers::Tokenizer;

/// Vocabulary size for Bonsai-8B.
const BONSAI_VOCAB_SIZE: usize = 151_669;

/// Maximum new tokens to generate in a single response.
const MAX_NEW_TOKENS: usize = 256;

/// Default generation temperature (0.7 = balanced creativity/coherence).
const DEFAULT_TEMPERATURE: f64 = 0.7;

/// LLM inference engine — loads Bonsai-8B GGUF via Candle and runs inference.
pub struct LlmEngine {
    /// Model weights wrapped in Arc<Mutex> for thread-safe interior mutability
    /// (forward() needs &mut self internally for KV-cache updates).
    model: std::sync::Arc<std::sync::Mutex<qlm::ModelWeights>>,
    tokenizer: Tokenizer,
    device: Device,
    vocab_size: usize,
    bos_token_id: Option<u32>,
    eos_token_id: Option<u32>,
    max_seq_len: usize,
}

impl LlmEngine {
    /// Create a new LLM engine from a GGUF file path.
    /// This loads both the model weights and the embedded tokenizer.
    pub fn new(gguf_path: &Path) -> AnyhowResult<Self> {
        let device = Device::Cpu;

        // Open file and read GGUF content (contains both model tensors
        // and the embedded tokenizer metadata).
        let mut file = std::fs::File::open(gguf_path)
            .with_context(|| format!("Failed to open GGUF file: {:?}", gguf_path))?;
        let content = gguf_file::Content::read(&mut file)
            .context("Failed to read GGUF file content")?;

        // Load the embedded tokenizer from GGUF metadata.
        // Reads tokenizer.ggml.model, tokenizer.ggml.tokens, tokenizer.ggml.merges.
        let tokenizer = Tokenizer::from_gguf(&content)
            .context("Failed to load tokenizer from GGUF — file may be corrupted or use an unsupported format")?;

        // Re-open file for model weight loading (tensor data section).
        let mut file = std::fs::File::open(gguf_path)
            .with_context(|| format!("Failed to reopen GGUF file: {:?}", gguf_path))?;
        let content2 = gguf_file::Content::read(&mut file)?;
        let model = qlm::ModelWeights::from_gguf(content2, &mut file, &device)
            .context("Failed to load model weights from GGUF")?;

        // Extract special token IDs from the tokenizer vocab.
        let vocab = tokenizer.get_vocab(true);
        let bos_token_id = vocab.get("<s>").copied();
        let eos_token_id = vocab.get("</s>").copied();

        Ok(Self {
            model: std::sync::Arc::new(std::sync::Mutex::new(model)),
            tokenizer,
            device,
            vocab_size: BONSAI_VOCAB_SIZE,
            bos_token_id,
            eos_token_id,
            max_seq_len: 4096,
        })
    }

    /// Check if a GGUF file looks like Bonsai (Q1_0_g128 quantization).
    pub fn is_bonsai(gguf_path: &Path) -> AnyhowResult<bool> {
        let mut file = std::fs::File::open(gguf_path)?;
        let content = gguf_file::Content::read(&mut file)?;
        Ok(content.tensor_infos.values().any(|t| matches!(
            t.ggml_dtype,
            candle_core::quantized::GgmlDType::Q1_0_g128
        )))
    }

    /// Tokenize a string into token IDs using the GGUF-embedded tokenizer.
    fn tokenize(&self, text: &str) -> AnyhowResult<Vec<u32>> {
        let encoding = self.tokenizer.encode(text, false)
            .map_err(|e| anyhow::anyhow!("Tokenizer encode error: {}", e))?;
        Ok(encoding.get_ids().to_vec())
    }

    /// Decode a single token ID to a string.
    fn decode_token(&self, token_id: u32) -> String {
        if let Ok(decoded) = self.tokenizer.decode(&[token_id], true) {
            decoded
        } else {
            String::new()
        }
    }

    /// Autoregressive text generation with temperature and top-p sampling.
    ///
    /// Processes the prompt through the model, then generates tokens
    /// one-at-a-time with KV-cache acceleration until EOS or max_new_tokens.
    fn generate_impl(
        &mut self,
        prompt: &str,
        max_new_tokens: usize,
        temperature: f64,
        top_p: Option<f64>,
    ) -> AnyhowResult<String> {
        // Tokenize the prompt.
        let mut token_ids = self.tokenize(prompt)?;

        // Optionally prepend BOS token.
        if let Some(bos) = self.bos_token_id {
            token_ids.insert(0, bos);
        }

        // Build logits processor for sampling.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(42);
        let mut logits_processor = LogitsProcessor::new(seed, Some(temperature), top_p);

        let eos = self.eos_token_id.unwrap_or(0);
        let mut generated = Vec::new();
        let prompt_len = token_ids.len();

        for step in 0..max_new_tokens {
            // index_pos: the position of the LAST token in the full sequence
            // that this forward pass will attend over.
            let index_pos = prompt_len.saturating_sub(1) + step;

            // Build input: full prompt on step 0, single last token thereafter.
            let input_ids: Vec<u32> = if step == 0 {
                token_ids.clone()
            } else {
                // Append the last generated token to our running sequence.
                if let Some(&last) = token_ids.last() {
                    token_ids.push(last);
                }
                vec![*token_ids.last().unwrap_or(&0)]
            };

            // Run forward pass.
            let input_tensor = Tensor::new(input_ids.as_slice(), &self.device)
                .map_err(|e| anyhow::anyhow!("Failed to create input tensor: {}", e))?
                .reshape(&[1, input_ids.len()])?;
            let logits = self.model.lock().unwrap().forward(&input_tensor, index_pos)
                .map_err(|e| anyhow::anyhow!("Forward pass failed at step {}: {}", step, e))?;

            // Extract logits for the last position → [vocab_size] Tensor, then sample.
            let last_logits = logits
                .squeeze(0)
                .map_err(|e| anyhow::anyhow!("Squeeze[0] failed: {}", e))?
                .squeeze(0)
                .map_err(|e| anyhow::anyhow!("Squeeze[1] failed: {}", e))?;

            // Sample next token directly from the Tensor.
            let next_token = logits_processor
                .sample(&last_logits)
                .map_err(|e| anyhow::anyhow!("Sampling failed: {}", e))? as u32;

            // Stop on EOS.
            if next_token == eos && eos != 0 {
                break;
            }

            generated.push(next_token);
            token_ids.push(next_token);
        }

        // Decode generated tokens to a string.
        let text = self.tokenizer.decode(&generated, true)
            .unwrap_or_else(|_| {
                generated.iter()
                    .map(|&t| self.decode_token(t))
                    .collect::<String>()
                    .trim()
                    .to_string()
            });

        Ok(text.trim().to_string())
    }

    /// Generate a text completion for a prompt (no chat formatting).
    pub fn generate(&mut self, prompt: &str) -> AnyhowResult<String> {
        self.generate_impl(prompt, MAX_NEW_TOKENS, DEFAULT_TEMPERATURE, Some(0.9))
    }

    /// Generate with custom parameters.
    pub fn generate_with(
        &mut self,
        prompt: &str,
        max_new_tokens: usize,
        temperature: f64,
    ) -> AnyhowResult<String> {
        self.generate_impl(prompt, max_new_tokens, temperature, Some(0.9))
    }

    /// Generate a chat completion from messages.
    pub fn chat(&mut self, messages: &[ChatMessage]) -> AnyhowResult<String> {
        let prompt = build_bonsai_prompt(messages);
        self.generate(&prompt)
    }

    /// Generate with streaming callback — yields tokens one at a time.
    ///
    /// Callback is called after each new token is decoded. Return `false` from
    /// the callback to abort early. Tokens are decoded as strings (may be
    /// multi-char or empty depending on tokenizer behavior).
    ///
    /// # Example
    /// ```ignore
    /// engine.generate_stream("Hello", 20, 0.7, Some(0.9), |tok| {
    ///     print!("{}", tok);
    ///     true  // keep going
    /// })?;
    /// ```
    pub fn generate_stream<F>(
        &mut self,
        prompt: &str,
        max_new_tokens: usize,
        temperature: f64,
        top_p: Option<f64>,
        mut callback: F,
    ) -> AnyhowResult<()>
    where
        F: FnMut(&str) -> bool,
    {
        let mut token_ids = self.tokenize(prompt)?;

        if let Some(bos) = self.bos_token_id {
            token_ids.insert(0, bos);
        }

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(42);
        let mut logits_processor = LogitsProcessor::new(seed, Some(temperature), top_p);

        let eos = self.eos_token_id.unwrap_or(0);

        for step in 0..max_new_tokens {
            let index_pos = token_ids.len() - 1;

            let input_ids: Vec<u32> = if step == 0 {
                token_ids.clone()
            } else {
                vec![*token_ids.last().unwrap_or(&0)]
            };

            let input_tensor = Tensor::new(input_ids.as_slice(), &self.device)?
                .reshape(&[1, input_ids.len()])?;
            let logits = self
                .model
                .lock()
                .unwrap()
                .forward(&input_tensor, index_pos)?;

            let last_logits = logits.squeeze(0)?.squeeze(0)?;
            let next_token = logits_processor.sample(&last_logits)? as u32;

            if next_token == eos && eos != 0 {
                break;
            }

            let token_str = self.decode_token(next_token);
            if !token_str.is_empty() {
                if !callback(&token_str) {
                    break; // caller aborted
                }
            }

            token_ids.push(next_token);
        }

        Ok(())
    }

    /// Generate a chat completion with streaming — callbacks with each token.
    pub fn chat_stream<F>(&mut self, messages: &[ChatMessage], callback: F) -> AnyhowResult<()>
    where
        F: FnMut(&str) -> bool,
    {
        let prompt = build_bonsai_prompt(messages);
        self.generate_stream(&prompt, MAX_NEW_TOKENS, DEFAULT_TEMPERATURE, Some(0.9), callback)
    }

    /// Polish rough text with streaming — callbacks with each token.
    ///
    /// System prompt is omitted from streaming output to avoid polluting the
    /// returned text. The model still sees it internally.
    pub fn polish_stream<F>(&mut self, rough_text: &str, callback: F) -> AnyhowResult<()>
    where
        F: FnMut(&str) -> bool,
    {
        let system = "You are a text polish engine. Rewrite the following text to be \
             more fluent and natural-sounding while preserving all factual \
             content and meaning. Be concise. Do not add new information. \
             Do not summarize. Maintain the same level of detail.";
        let messages = &[
            ChatMessage::system(system),
            ChatMessage::user(rough_text),
        ];
        self.chat_stream(messages, callback)
    }

    /// Polish a rough Starfire output into fluent, natural-sounding text.
    ///
    /// Primary voice integration point. Flow:
    /// 1. Reasoning layer produces rough output (may be stilted)
    /// 2. `polish()` sends it to Bonsai with a "text polish engine" prompt
    /// 3. Bonsai rewrites it naturally
    /// 4. Result returned as final response
    ///
    /// No new information added, no summarization — just fluent expression.
    pub fn polish(&mut self, rough_text: &str) -> AnyhowResult<String> {
        self.generate_with_system(
            "You are a text polish engine. Rewrite the following text to be \
             more fluent and natural-sounding while preserving all factual \
             content and meaning. Be concise. Do not add new information. \
             Do not summarize. Maintain the same level of detail.",
            rough_text,
        )
    }

    /// Generate with an explicit system prompt + user message.
    pub fn generate_with_system(
        &mut self,
        system: &str,
        user: &str,
    ) -> AnyhowResult<String> {
        let messages = &[
            ChatMessage::system(system),
            ChatMessage::user(user),
        ];
        self.chat(messages)
    }

    /// Health check — returns true if model responds correctly to a forward pass.
    pub fn health_check(&mut self) -> bool {
        if let Ok(tokens) = self.tokenize("Hello") {
            if !tokens.is_empty() {
                let first_token = tokens[0];
                if let Ok(input) = Tensor::new(&[first_token], &self.device) {
                    if let Ok(reshaped) = input.reshape(&[1, 1]) {
                        return self.model.lock().unwrap().forward(&reshaped, 0).is_ok();
                    }
                }
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

    /// Construct an LlmEngine from Arc-wrapped parts (allows LlmHandle to
    /// return a fresh engine from its cache without deep-cloning weights).
    pub fn from_arc_parts(
        model: std::sync::Arc<std::sync::Mutex<qlm::ModelWeights>>,
        tokenizer: Tokenizer,
        device: Device,
        vocab_size: usize,
        bos_token_id: Option<u32>,
        eos_token_id: Option<u32>,
        max_seq_len: usize,
    ) -> Self {
        Self {
            model,
            tokenizer,
            device,
            vocab_size,
            bos_token_id,
            eos_token_id,
            max_seq_len,
        }
    }
}

/// A single chat message (role + content).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self { role: "system".to_string(), content: content.to_string() }
    }

    pub fn user(content: &str) -> Self {
        Self { role: "user".to_string(), content: content.to_string() }
    }

    pub fn assistant(content: &str) -> Self {
        Self { role: "assistant".to_string(), content: content.to_string() }
    }
}

/// Build a Bonsai-compatible prompt from chat messages.
fn build_bonsai_prompt(messages: &[ChatMessage]) -> String {
    let mut parts = Vec::new();

    for msg in messages {
        match msg.role.as_str() {
            "system" => {
                parts.push(format!("System: {}", msg.content.trim()));
            }
            "user" => {
                parts.push(format!("User: {}", msg.content.trim()));
            }
            "assistant" => {
                parts.push(format!("Assistant: {}", msg.content.trim()));
            }
            _ => {
                parts.push(format!("{}: {}", msg.role, msg.content.trim()));
            }
        }
    }

    parts.join("\n")
}

/// Called once at startup to init candle internals.
fn init() {
    // candle-core is side-effect-free on CPU; no explicit init needed.
}

/// A handle that can be shared across threads. Loading is deferred —
/// call `.load()` to get an actual `LlmEngine`. The engine is cached
/// so repeated calls return the same instance without reloading GGUF.
pub struct LlmHandle {
    gguf_path: std::path::PathBuf,
    /// Cached engine. Uses `parking_lot::Mutex` (which is `Send + Sync`) so
    /// the handle is `Clone + Send + Sync`. Wrapping in Arc so clones of the
    /// handle share the same cached engine.
    cached: std::sync::Arc<parking_lot::Mutex<Option<LlmEngine>>>,
}

impl LlmHandle {
    /// Load (or return cached) the Bonsai-8B model.
    /// First call does GGUF parsing + tensor loading (~5-10s).
    /// Subsequent calls return the cached engine immediately.
    pub fn load(&self) -> AnyhowResult<LlmEngine> {
        // Fast path: already loaded.
        {
            let guard = self.cached.lock();
            if let Some(ref engine) = *guard {
                // Return a fresh instance with shared Arc-wrapped fields so the
                // caller gets mutable access (for generation loop) while the
                // cache retains ownership.
                return Ok(LlmEngine::from_arc_parts(
                    engine.model.clone(),
                    engine.tokenizer.clone(),
                    engine.device.clone(),
                    engine.vocab_size,
                    engine.bos_token_id,
                    engine.eos_token_id,
                    engine.max_seq_len,
                ));
            }
        }

        // Slow path: load and cache.
        let engine = LlmEngine::new(&self.gguf_path)?;
        let cached_engine = LlmEngine::from_arc_parts(
            engine.model.clone(),
            engine.tokenizer.clone(),
            engine.device.clone(),
            engine.vocab_size,
            engine.bos_token_id,
            engine.eos_token_id,
            engine.max_seq_len,
        );
        {
            let mut guard = self.cached.lock();
            *guard = Some(cached_engine);
        }
        Ok(engine)
    }

    /// Create a handle from a GGUF file path.
    pub fn new(gguf_path: &Path) -> Self {
        Self {
            gguf_path: gguf_path.to_path_buf(),
            cached: std::sync::Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    /// Create an LLM handle at the standard Bonsai-8B location for a data dir.
    pub fn for_data_dir(data_dir: &Path) -> Self {
        Self::new(&data_dir.join("models/bonsai-8b/Bonsai-8B.gguf"))
    }

    /// Chat with streaming — callback receives each token as it's generated.
    /// Return `false` from callback to abort early.
    pub fn chat_stream<F>(&self, messages: &[ChatMessage], callback: F) -> AnyhowResult<()>
    where
        F: FnMut(&str) -> bool + Send + 'static,
    {
        let mut engine = self.load()?;
        engine.chat_stream(messages, callback)
    }

    /// Polish rough text with streaming — callback receives each token.
    /// Return `false` from callback to abort early.
    pub fn polish_stream<F>(&self, rough_text: &str, callback: F) -> AnyhowResult<()>
    where
        F: FnMut(&str) -> bool + Send + 'static,
    {
        let mut engine = self.load()?;
        engine.polish_stream(rough_text, callback)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
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
    fn test_model_size() {
        let path = Path::new("models/bonsai-8b/Bonsai-8B.gguf");
        if path.exists() {
            let size = LlmEngine::model_size_human(path);
            println!("Model size: {}", size);
            assert!(size.contains("GB") || size.contains("MB"));
        }
    }

    #[test]
    fn test_build_bonsai_prompt_system_user() {
        let messages = &[
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("What is 2+2?"),
        ];
        let prompt = build_bonsai_prompt(messages);
        assert!(prompt.contains("You are a helpful assistant"));
        assert!(prompt.contains("User:"));
        assert!(prompt.contains("What is 2+2?"));
    }

    #[test]
    fn test_generate_stream_yields_tokens() {
        let path = Path::new("models/bonsai-8b/Bonsai-8B.gguf");
        if !path.exists() {
            println!("SKIPPED (no GGUF at {:?})", path);
            return;
        }
        let mut engine = LlmEngine::new(path).unwrap();
        let mut tokens = Vec::new();
        let mut token_count = 0;
        engine
            .generate_stream(
                "Say hello in exactly three words.",
                30,
                0.7,
                Some(0.9),
                |tok| {
                    token_count += 1;
                    tokens.push(tok.to_string());
                    true // keep going
                },
            )
            .unwrap();
        println!(
            "Got {} tokens: {:?}",
            token_count,
            &tokens.iter().take(5).collect::<Vec<_>>()
        );
        assert!(token_count > 0, "Should yield at least some tokens");
        // Should have aborted after 30 tokens max
        assert!(token_count <= 30);
    }

    #[test]
    fn test_generate_stream_abort_early() {
        let path = Path::new("models/bonsai-8b/Bonsai-8B.gguf");
        if !path.exists() {
            println!("SKIPPED (no GGUF at {:?})", path);
            return;
        }
        let mut engine = LlmEngine::new(path).unwrap();
        let mut token_count = 0;
        engine
            .generate_stream("Tell me a long story.", 100, 0.7, Some(0.9), |_tok| {
                token_count += 1;
                if token_count >= 5 {
                    false // abort after 5 tokens
                } else {
                    true
                }
            })
            .unwrap();
        assert_eq!(token_count, 5, "Should have aborted after 5 tokens");
    }
}
