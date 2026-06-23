//! Intent-driven response reranker — Phase 3 of voice-refine (2026-06-21).
//!
//! ## What this is
//!
//! A thin layer that sits between `runtime::chat()` (which produces a
//! `Response { intent, body, slots }`) and `voice::VoiceEngine::speak()` (which
//! shapes phrasing based on style + internal state). The reranker's job:
//! take a structured response + Star's internal state + an optional structured
//! metacog intent, and produce a refined body whose phrasing *tracks* the
//! moment — not a fixed timestamp-derived rotation.
//!
//! This is the "hybrid" path from the 2026-06-21 design conversation:
//! structured intent assembly on one side, a small generative model on the
//! other. The voice engine keeps its style/personality modulation; the
//! reranker adds the in-the-moment phrasing that the deterministic template
//! rotations can't produce.
//!
//! ## Backend abstraction
//!
//! ```text
//! runtime::chat()
//!     └─► produces Response { intent, body, slots }
//!         └─► IntentReranker::rerank()         <-- this module
//!             └─► RerankerBackend::rewrite()  <-- trait
//!                 ├─ MockReranker           (default, deterministic, no model)
//!                 ├─ CharRnnBackend         (ship-today, uses existing charRNN)
//!                 └─ LmRsBackend            (future, gated behind feature flag)
//!             └─► returns refined body string
//!     └─► voice::VoiceEngine::speak()        <-- existing style layer
//! ```
//!
//! The three backends are tiered:
//!
//! - **MockReranker** — pure rule-based transforms driven by intent + state.
//!   Always available, no model loading, fully deterministic. Proves the
//!   architecture end-to-end. Use this for tests and the current shipped build.
//!
//! - **CharRnnBackend** — wraps the existing `CharRNN` from `model.rs` so the
//!   reranker can sample from a real (tiny) generative model. The charRNN is
//!   small (~11MB) and runs in the same process — matches the "super light
//!   generator" constraint from the original ask.
//!
//! - **LmRsBackend** — the future path. Skeleton only; the real implementation
//!   comes when we vendor `lm.rs` (or `qwen3-rs`) and load a small SLM
//!   (~1B params Q8 = ~1GB) in-process. Gated behind the `lmrs-backend`
//!   cargo feature so the default build doesn't pull in the heavy deps.
//!
//! ## Why a trait
//!
//! The architecture can swap backends without changing the call site. The
//! runtime holds `Box<dyn RerankerBackend>` and constructs the appropriate
//! backend at startup based on feature flags / model availability.
//!
//! ## What this is NOT
//!
//! - Not a re-implementation of the voice engine. The voice engine operates
//!   AFTER the reranker — it still does the style/personality/quanot work.
//! - Not a replacement for the structured intents. The reranker reads them,
//!   it doesn't replace them. `CuriosityIntent` etc. are still the source of
//!   truth for what Star means to say.
//! - Not the full voice-refine. Phase 1 (state visibility) and Phase 2
//!   (structured intents) shipped earlier; this is Phase 3 — the layer that
//!   turns structure into voice-aware prose.

use crate::metacog::intents::{
    CuriosityIntent, InsightIntent, RevisionIntent, SurpriseIntent,
};
use crate::personality::ResponseStyle;
use crate::runtime::response_intent::{Response, ResponseIntent};
use crate::voice::InternalState;

use std::fmt::Write as _;

// ════════════════════════════════════════════════════════════════════════════
// Backend trait
// ════════════════════════════════════════════════════════════════════════════

/// A backend that turns a structured `RerankPrompt` into a refined body.
///
/// Implementations decide HOW to rewrite — deterministic rules, a small
/// charRNN, or an external SLM. The trait only fixes the input/output shape.
pub trait RerankerBackend: Send + Sync {
    /// Human-readable name for this backend (logs, metrics).
    fn name(&self) -> &'static str;

    /// Rewrite a `RerankPrompt` into the refined body string.
    ///
    /// MUST be deterministic when `RerankConfig::deterministic` is true.
    /// MUST NOT panic on empty body — empty body is a valid signal that the
    /// backend should produce content purely from intent + state.
    fn rewrite(&self, prompt: &RerankPrompt, cfg: &RerankConfig) -> Result<String, RerankError>;
}

// ════════════════════════════════════════════════════════════════════════════
// RerankPrompt — what the backend sees
// ════════════════════════════════════════════════════════════════════════════

/// The encoded input a backend rewrites. Built once per call to `rerank()`;
/// backends consume it read-only.
#[derive(Debug, Clone)]
pub struct RerankPrompt {
    /// The intent of this response (SelfCheck, Emotional, Reflection, ...).
    pub intent: ResponseIntent,

    /// Style hint from the intent (Direct, Warm, Curious, ...).
    pub style_hint: Option<ResponseStyle>,

    /// The body the runtime produced. May be empty — backends handle that.
    pub body: String,

    /// Slot data the runtime attached (key/value pairs).
    pub slots: Vec<(String, String)>,

    /// Star's internal state at the moment of utterance.
    pub internal_state: InternalState,

    /// Optional structured curiosity intent, if the response came from metacog.
    pub curiosity: Option<CuriosityIntent>,

    /// Optional structured revision intent, if the response is a belief update.
    pub revision: Option<RevisionIntent>,

    /// Optional structured surprise intent, if the response is a surprise.
    pub surprise: Option<SurpriseIntent>,

    /// Optional structured insight intent, if the response is a self-insight.
    pub insight: Option<InsightIntent>,
}

impl RerankPrompt {
    /// Build a prompt from the runtime-level `Response` + `InternalState`.
    pub fn from_response(response: &Response, internal_state: &InternalState) -> Self {
        Self {
            intent: response.intent.clone(),
            style_hint: response.style_hint.clone().or_else(|| response.intent.default_style_hint()),
            body: response.body.clone(),
            slots: response.slots.clone(),
            internal_state: internal_state.clone(),
            curiosity: None,
            revision: None,
            surprise: None,
            insight: None,
        }
    }

    /// Attach a structured curiosity intent. Builder-style.
    pub fn with_curiosity(mut self, c: CuriosityIntent) -> Self {
        self.curiosity = Some(c);
        self
    }

    /// Attach a structured revision intent.
    pub fn with_revision(mut self, r: RevisionIntent) -> Self {
        self.revision = Some(r);
        self
    }

    /// Attach a structured surprise intent.
    pub fn with_surprise(mut self, s: SurpriseIntent) -> Self {
        self.surprise = Some(s);
        self
    }

    /// Attach a structured insight intent.
    pub fn with_insight(mut self, i: InsightIntent) -> Self {
        self.insight = Some(i);
        self
    }

    /// True if this prompt carries no signal beyond defaults.
    pub fn is_empty(&self) -> bool {
        self.body.trim().is_empty()
            && self.curiosity.is_none()
            && self.revision.is_none()
            && self.surprise.is_none()
            && self.insight.is_none()
            && self.slots.is_empty()
    }

    /// Render a one-line summary for logs.
    pub fn summary(&self) -> String {
        let mut s = format!("intent={}", self.intent.label());
        if let Some(style) = &self.style_hint {
            let _ = write!(s, " style={:?}", style);
        }
        if !self.body.is_empty() {
            let _ = write!(s, " body_len={}", self.body.len());
        } else {
            s.push_str(" body=<empty>");
        }
        if let Some(c) = &self.curiosity {
            let _ = write!(s, " curiosity={:?}", c.kind);
        }
        if let Some(r) = &self.revision {
            let _ = write!(s, " revision_topic={}", r.topic);
        }
        if let Some(s2) = &self.surprise {
            let _ = write!(s, " surprise={:?}", s2.kind);
        }
        if let Some(i) = &self.insight {
            let _ = write!(s, " insight={:?}", i.kind);
        }
        s
    }
}

// ════════════════════════════════════════════════════════════════════════════
// RerankConfig — backend-agnostic knobs
// ════════════════════════════════════════════════════════════════════════════

/// Backend-agnostic configuration for the reranker.
///
/// Different backends honor different subsets of these fields. The trait
/// documents which fields matter; backends MAY ignore the rest.
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// Maximum length of the returned body, in characters. Backends truncate
    /// if their output exceeds this. `None` = no limit.
    pub max_chars: Option<usize>,

    /// Sampling temperature for generative backends (0.0 = greedy,
    /// higher = more random). Rule-based backends ignore this.
    pub temperature: f32,

    /// If true, the backend MUST produce identical output for identical
    /// input. Used in tests and for the timestamp-shuffle-killing invariant.
    pub deterministic: bool,

    /// Optional seed for generative backends. Rule-based backends ignore.
    pub seed: Option<u64>,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            max_chars: Some(280),
            temperature: 0.7,
            deterministic: false,
            seed: None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// RerankError
// ════════════════════════════════════════════════════════════════════════════

/// Errors a backend may surface. Kept intentionally small — backends that
/// hit a real failure (model file missing, GPU OOM, etc.) should report it
/// via this enum so the caller can fall back gracefully.
#[derive(Debug, thiserror::Error)]
pub enum RerankError {
    /// Backend needed a model file that wasn't found.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Backend failed to load a model (corrupt file, version mismatch, etc.).
    #[error("failed to load model: {0}")]
    LoadFailed(String),

    /// Backend failed during generation/sampling.
    #[error("generation failed: {0}")]
    GenerationFailed(String),

    /// Backend is not yet implemented (for skeleton backends like LmRsBackend).
    #[error("backend not implemented: {0}")]
    NotImplemented(String),

    /// Catch-all for unexpected errors.
    #[error("rerank error: {0}")]
    Other(String),
}

// ════════════════════════════════════════════════════════════════════════════
// IntentReranker — the user-facing wrapper
// ════════════════════════════════════════════════════════════════════════════

/// The user-facing reranker. Holds a backend and a config; the runtime
/// calls [`IntentReranker::rerank`] after producing a `Response` and before
/// passing the body to `voice::VoiceEngine::speak()`.
pub struct IntentReranker {
    backend: Box<dyn RerankerBackend>,
    config: RerankConfig,
}

impl IntentReranker {
    /// Construct a reranker with a specific backend and config.
    pub fn new(backend: Box<dyn RerankerBackend>, config: RerankConfig) -> Self {
        Self { backend, config }
    }

    /// Construct a reranker with the default backend (MockReranker).
    ///
    /// This is what production code should use until a real backend is wired
    /// — the mock is deterministic, free, and proves the architecture.
    pub fn with_default_backend() -> Self {
        Self::new(Box::new(MockReranker::default()), RerankConfig::default())
    }

    /// The backend's name (for logs and metrics).
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    /// Rerank a runtime response using Star's current internal state.
    ///
    /// This is the main call site. The returned string is what should flow
    /// into `voice::VoiceEngine::speak()`.
    pub fn rerank(&self, response: &Response, internal_state: &InternalState) -> String {
        let prompt = RerankPrompt::from_response(response, internal_state);

        tracing::debug!(
            backend = self.backend.name(),
            "rerank: {}",
            prompt.summary(),
        );

        match self.backend.rewrite(&prompt, &self.config) {
            Ok(out) => self.post_process(out),
            Err(e) => {
                // Backends must never panic the caller. Log and fall back to
                // the raw body — the voice engine still has its modulations.
                tracing::warn!(
                    backend = self.backend.name(),
                    error = %e,
                    "rerank failed; falling back to raw body",
                );
                self.post_process(response.body.clone())
            }
        }
    }

    /// Rerank with an attached structured curiosity intent.
    pub fn rerank_with_curiosity(
        &self,
        response: &Response,
        internal_state: &InternalState,
        curiosity: CuriosityIntent,
    ) -> String {
        let prompt =
            RerankPrompt::from_response(response, internal_state).with_curiosity(curiosity);
        self.run(&prompt, response)
    }

    /// Rerank with an attached structured revision intent.
    pub fn rerank_with_revision(
        &self,
        response: &Response,
        internal_state: &InternalState,
        revision: RevisionIntent,
    ) -> String {
        let prompt = RerankPrompt::from_response(response, internal_state).with_revision(revision);
        self.run(&prompt, response)
    }

    /// Rerank with an attached structured surprise intent.
    pub fn rerank_with_surprise(
        &self,
        response: &Response,
        internal_state: &InternalState,
        surprise: SurpriseIntent,
    ) -> String {
        let prompt = RerankPrompt::from_response(response, internal_state).with_surprise(surprise);
        self.run(&prompt, response)
    }

    /// Rerank with an attached structured insight intent.
    pub fn rerank_with_insight(
        &self,
        response: &Response,
        internal_state: &InternalState,
        insight: InsightIntent,
    ) -> String {
        let prompt = RerankPrompt::from_response(response, internal_state).with_insight(insight);
        self.run(&prompt, response)
    }

    /// Shared run path for the *_with_* helpers.
    fn run(&self, prompt: &RerankPrompt, response: &Response) -> String {
        tracing::debug!(
            backend = self.backend.name(),
            "rerank: {}",
            prompt.summary(),
        );
        match self.backend.rewrite(prompt, &self.config) {
            Ok(out) => self.post_process(out),
            Err(e) => {
                tracing::warn!(
                    backend = self.backend.name(),
                    error = %e,
                    "rerank failed; falling back to raw body",
                );
                self.post_process(response.body.clone())
            }
        }
    }

    /// Apply backend-agnostic post-processing: trim, truncate to max_chars.
    fn post_process(&self, mut out: String) -> String {
        out = out.trim().to_string();
        if let Some(max) = self.config.max_chars {
            if out.chars().count() > max {
                // Truncate at a char boundary, prefer sentence break.
                let truncated: String = out.chars().take(max).collect();
                if let Some(last_period) = truncated.rfind(['.', '!', '?']) {
                    if last_period > (max / 2) {
                        out = truncated[..=last_period].to_string();
                    } else {
                        out = format!("{}…", truncated.trim_end());
                    }
                } else {
                    out = format!("{}…", truncated.trim_end());
                }
            }
        }
        out
    }
}

// ════════════════════════════════════════════════════════════════════════════
// MockReranker — deterministic, no model, proves the architecture
// ════════════════════════════════════════════════════════════════════════════

/// A pure rule-based reranker. No model, no randomness, fully deterministic.
///
/// This is the **default backend** for production today. It performs a small
/// set of intent- and state-driven transforms on the body that demonstrate
/// what the reranker layer *can* do:
///
/// - SelfCheck + uncertainty > 0.5 → prepend "Honestly, "
/// - Emotional + negative valence → trim and soften
/// - Reflection + high engagement → append a follow-up question
/// - General: hedging removal when confidence > 0.7 and consciousness > 0.6
/// - Empty body + structured intent → emit prose derived from the intent
///
/// The transform set is intentionally small. The point isn't to be a great
/// rewriter — it's to be a *testable* one that proves the architecture.
/// A real backend (CharRnnBackend today, LmRsBackend tomorrow) replaces this
/// without changing the call site.
#[derive(Debug, Default, Clone)]
pub struct MockReranker;

impl RerankerBackend for MockReranker {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn rewrite(&self, prompt: &RerankPrompt, _cfg: &RerankConfig) -> Result<String, RerankError> {
        // 1. Empty body — derive content purely from structured intents.
        if prompt.body.trim().is_empty() {
            if let Some(c) = &prompt.curiosity {
                return Ok(c.format());
            }
            if let Some(r) = &prompt.revision {
                return Ok(r.format());
            }
            if let Some(s) = &prompt.surprise {
                return Ok(s.format());
            }
            if let Some(i) = &prompt.insight {
                return Ok(i.format());
            }
            // Empty body, no structured intent, no slots — return empty.
            // Caller (voice engine) will assemble from intent label alone.
            return Ok(String::new());
        }

        // 2. Body present — apply state-aware transforms.
        let mut out = prompt.body.clone();

        // Hedging removal: when Star is conscious + confident, "I think" → "I know."
        // Mirrors `voice::apply_quanot_expression` but at the reranker layer.
        let consciousness = prompt.internal_state.quanot_consciousness;
        let confidence = prompt.style_confidence_hint();
        let not_uncertain = prompt.internal_state.current_uncertainty <= 0.5;
        if consciousness > 0.6 && confidence > 0.6 && not_uncertain {
            if out.to_lowercase().starts_with("i think ") {
                out = format!("I know. {}", &out["i think ".len()..]);
            } else if out.to_lowercase().starts_with("i guess ") {
                out = format!("I know. {}", &out["i guess ".len()..]);
            }
        }

        // Intent-specific prepends.
        match prompt.intent {
            ResponseIntent::SelfCheck => {
                if prompt.internal_state.current_uncertainty > 0.5
                    && !out.to_lowercase().starts_with("honestly")
                {
                    out = format!("Honestly, {}", out);
                }
            }
            ResponseIntent::Emotional => {
                let valence = prompt.internal_state.cognitive_emotional_valence;
                if valence < -0.3 && out.len() > 80 {
                    // Trim long emotional responses when valence is negative.
                    if let Some(last_punct) = out.find(['.', '!', '?']) {
                        if last_punct < out.len() - 1 && last_punct < 80 {
                            out = out[..=last_punct].to_string();
                        }
                    }
                }
            }
            ResponseIntent::Reflection => {
                let engagement = prompt.internal_state.cognitive_engagement_depth;
                if engagement > 0.7 && !out.ends_with('?') {
                    // Append a follow-up question when Star is highly engaged.
                    let follow_up = "Want to go deeper?";
                    if !out.contains(follow_up) {
                        out = format!("{} {}", out.trim_end_matches('.'), follow_up);
                    }
                }
            }
            ResponseIntent::CuriosityCheck => {
                // If a structured curiosity intent is present and the body
                // doesn't reference the topic, append the topic.
                if let Some(c) = &prompt.curiosity {
                    if !out.to_lowercase().contains(&c.topic.to_lowercase()) {
                        out = format!("{} {}", out.trim_end_matches('.'), c.topic);
                    }
                }
            }
            _ => {}
        }

        Ok(out)
    }
}

impl MockReranker {
    /// Construct a default mock reranker. Equivalent to `MockReranker::default()`.
    pub fn new() -> Self {
        Self
    }
}

// Small helper extension on `RerankPrompt` — internal to this module so the
// voice-config confidence field doesn't leak into the trait shape.
trait RerankPromptExt {
    fn style_confidence_hint(&self) -> f64;
}

impl RerankPromptExt for RerankPrompt {
    fn style_confidence_hint(&self) -> f64 {
        // Map style to a coarse confidence hint. Real backends can ignore this;
        // the mock uses it for the think→know substitution.
        match self.style_hint {
            Some(ResponseStyle::Direct) => 0.8,
            Some(ResponseStyle::Analytical) => 0.75,
            Some(ResponseStyle::Warm) => 0.5,
            Some(ResponseStyle::Playful) => 0.6,
            Some(ResponseStyle::Curious) => 0.4,
            Some(ResponseStyle::Minimal) => 0.7,
            // LeetMatch + Reserved are context-dependent; default to neutral.
            Some(ResponseStyle::LeetMatch) | Some(ResponseStyle::Reserved) => 0.5,
            None => 0.5,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// CharRnnBackend — wraps the existing charRNN (ship-today path)
// ════════════════════════════════════════════════════════════════════════════

/// Reranker backed by the existing `CharRNN` model.
///
/// This is the **ship-today path** for the reranker: it uses the model that's
/// already in the codebase (and trained on Star's conversations) to do a
/// small amount of generative polishing. The model is ~11MB and runs in the
/// same process — matches the "super light generator" constraint.
///
/// The backend encodes the structured prompt as text, feeds it through the
/// charRNN, and returns the sampled continuation. Quality is bounded by the
/// charRNN's capacity (small), but the architecture is the same one a real
/// SLM would slot into.
pub struct CharRnnBackend {
    /// The model. Wrapped in `Option` so `Default` can exist without one.
    model: Option<std::sync::Mutex<crate::language_model::model::CharRNN>>,
    vocab: Option<std::sync::Mutex<crate::language_model::vocabulary::Vocabulary>>,
}

impl std::fmt::Debug for CharRnnBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharRnnBackend")
            .field("model_loaded", &self.model.is_some())
            .field("vocab_loaded", &self.vocab.is_some())
            .finish()
    }
}

impl Default for CharRnnBackend {
    fn default() -> Self {
        Self {
            model: None,
            vocab: None,
        }
    }
}

impl CharRnnBackend {
    /// Construct a backend with the supplied model and vocab.
    pub fn new(
        model: crate::language_model::model::CharRNN,
        vocab: crate::language_model::vocabulary::Vocabulary,
    ) -> Self {
        Self {
            model: Some(std::sync::Mutex::new(model)),
            vocab: Some(std::sync::Mutex::new(vocab)),
        }
    }

    /// Build the text prompt that gets fed to the charRNN. Concise, since
    /// charRNNs are character-level and longer prompts dilute the signal.
    fn build_text_prompt(&self, prompt: &RerankPrompt) -> String {
        let mut s = String::new();
        let _ = write!(s, "intent:{} ", prompt.intent.label());
        if let Some(style) = &prompt.style_hint {
            let _ = write!(s, "style:{:?} ", style);
        }
        let _ = write!(s, "uncertainty:{:.2} ", prompt.internal_state.current_uncertainty);
        let _ = write!(s, "consciousness:{:.2} ", prompt.internal_state.quanot_consciousness);
        if !prompt.body.is_empty() {
            let _ = write!(s, "body:{}", prompt.body);
        }
        s.push_str(" Star:");
        s
    }
}

impl RerankerBackend for CharRnnBackend {
    fn name(&self) -> &'static str {
        "char_rnn"
    }

    fn rewrite(&self, prompt: &RerankPrompt, cfg: &RerankConfig) -> Result<String, RerankError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| RerankError::ModelNotFound("CharRNN not loaded".into()))?;
        let vocab = self
            .vocab
            .as_ref()
            .ok_or_else(|| RerankError::ModelNotFound("Vocabulary not loaded".into()))?;

        let text_prompt = self.build_text_prompt(prompt);
        let mut model_guard = model.lock().map_err(|e| RerankError::Other(e.to_string()))?;
        let vocab_guard = vocab.lock().map_err(|e| RerankError::Other(e.to_string()))?;

        let gen_cfg = crate::language_model::generate::GenerateConfig {
            max_length: cfg.max_chars.unwrap_or(120),
            temperature: cfg.temperature,
            top_k: 0,
            seed: cfg.seed,
        };

        let sampled = crate::language_model::generate::generate_response(
            &mut model_guard,
            &vocab_guard,
            &text_prompt,
            gen_cfg,
        );

        if sampled.trim().is_empty() {
            // charRNN sometimes produces empty output on cold models; fall
            // back to the raw body rather than ship empty prose.
            Ok(prompt.body.clone())
        } else {
            Ok(sampled)
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// LmRsBackend — future path, gated behind a feature flag
// ════════════════════════════════════════════════════════════════════════════

/// Reranker backed by `lm.rs` (or a fork like `qwen3-rs`).
///
/// **Skeleton only.** This backend exists so the architecture has a forward
/// path: when we vendor `lm.rs` and load a small SLM (1B Q8 ≈ 1GB, ~50 tok/s
/// CPU), this struct's methods become the integration point. For now every
/// method returns [`RerankError::NotImplemented`].
///
/// Gated behind the `lmrs-backend` cargo feature so the default build
/// doesn't pull in the lm.rs dependency graph (which includes the
/// safetensors / tokenizers / quantization crates). To enable:
///
/// ```toml
/// star = { path = "../lib", features = ["lmrs-backend"] }
/// ```
///
/// When enabled, the implementation will:
/// 1. Load the LMRS binary (or safetensors for `qwen3-rs`) at startup.
/// 2. Tokenize the structured prompt with the model's BPE tokenizer.
/// 3. Forward-pass with temperature / max_tokens derived from `RerankConfig`.
/// 4. Detokenize and return.
///
/// Memory budget: ~1-2GB resident for a 1-2B Q8 model. This is the
/// "real generative voice in same process" target from the 2026-06-21
/// design conversation.
#[cfg(feature = "lmrs-backend")]
pub struct LmRsBackend {
    /// Path to the LMRS binary or safetensors checkpoint.
    model_path: std::path::PathBuf,

    /// Path to the tokenizer.json (or sentencepiece.model).
    tokenizer_path: std::path::PathBuf,

    /// Context length (tokens). Default 2048.
    context_len: usize,
}

#[cfg(feature = "lmrs-backend")]
impl std::fmt::Debug for LmRsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LmRsBackend")
            .field("model_path", &self.model_path)
            .field("tokenizer_path", &self.tokenizer_path)
            .field("context_len", &self.context_len)
            .finish()
    }
}

#[cfg(feature = "lmrs-backend")]
impl LmRsBackend {
    /// Construct a backend pointing at an LMRS file on disk.
    ///
    /// Does NOT load the model — load is lazy, on first `rewrite()` call.
    /// This keeps construction cheap so the runtime can hold a backend
    /// even when no model file is present (the call site gets a clean
    /// `RerankError::ModelNotFound` rather than a constructor panic).
    pub fn new<P: Into<std::path::PathBuf>, Q: Into<std::path::PathBuf>>(
        model_path: P,
        tokenizer_path: Q,
    ) -> Self {
        Self {
            model_path: model_path.into(),
            tokenizer_path: tokenizer_path.into(),
            context_len: 2048,
        }
    }

    /// Override the context length. Must be called before the first `rewrite`.
    pub fn with_context_len(mut self, len: usize) -> Self {
        self.context_len = len;
        self
    }

    /// Build the prompt text that gets fed to the SLM. Unlike the charRNN
    /// prompt, this can be a richer natural-language instruction because
    /// the SLM has the capacity to follow it.
    #[allow(dead_code)]
    fn build_text_prompt(&self, prompt: &RerankPrompt) -> String {
        let mut s = String::new();
        s.push_str("You are rewriting Star's response so the phrasing tracks her current state.\n\n");
        let _ = write!(s, "Intent: {}\n", prompt.intent.label());
        if let Some(style) = &prompt.style_hint {
            let _ = write!(s, "Style: {:?}\n", style);
        }
        let _ = write!(
            s,
            "Uncertainty: {:.2} | Consciousness: {:.2} | Engagement: {:.2}\n",
            prompt.internal_state.current_uncertainty,
            prompt.internal_state.quanot_consciousness,
            prompt.internal_state.cognitive_engagement_depth,
        );
        if let Some(c) = &prompt.curiosity {
            let _ = write!(
                s,
                "Curiosity kind: {:?}, topic: {}, satisfaction: {:.2}\n",
                c.kind, c.topic, c.satisfaction
            );
        }
        if !prompt.slots.is_empty() {
            s.push_str("Slots:\n");
            for (k, v) in &prompt.slots {
                let _ = write!(s, "  {} = {}\n", k, v);
            }
        }
        s.push_str("\nDraft body:\n");
        s.push_str(&prompt.body);
        s.push_str("\n\nRewrite in Star's voice. Keep it under 280 characters.\n");
        s
    }
}

#[cfg(feature = "lmrs-backend")]
impl RerankerBackend for LmRsBackend {
    fn name(&self) -> &'static str {
        "lmrs"
    }

    fn rewrite(&self, _prompt: &RerankPrompt, _cfg: &RerankConfig) -> Result<String, RerankError> {
        // TODO(voice-refine phase 3+): wire lm.rs (or qwen3-rs) here.
        //
        // Steps when implementing:
        //   1. Lazy-load the model + tokenizer on first call.
        //      Use `Arc<Mutex<...>>` so concurrent rerank() calls don't
        //      double-load. The model is multi-GB; we want one resident copy.
        //   2. Build the prompt via `build_text_prompt()`.
        //   3. Tokenize (BPE for Qwen, SentencePiece for Gemma).
        //   4. Forward pass with KV cache, sample with cfg.temperature.
        //   5. Stop at cfg.max_chars worth of tokens, or at <|eot|>.
        //   6. Detokenize and return.
        //
        // Expected memory: ~1.3GB for Llama 3.2 1B Q8_0, ~2GB for Qwen 3.5 2B Q8.
        // Expected speed: ~30-50 tok/s on a modern multi-core CPU.
        Err(RerankError::NotImplemented(
            "LmRsBackend skeleton — see TODOs above. Enable lmrs-backend feature \
             and vendor lm.rs or qwen3-rs to implement."
                .into(),
        ))
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::response_intent::Response;

    fn make_state(uncertainty: f64, consciousness: f64, valence: f64, engagement: f64) -> InternalState {
        InternalState {
            quanot_novelty: 0.5,
            quanot_creativity: 0.5,
            quanot_consciousness: consciousness,
            cognitive_emotional_valence: valence,
            cognitive_engagement_depth: engagement,
            current_uncertainty: uncertainty,
            ..Default::default()
        }
    }

    fn make_response(intent: ResponseIntent, body: &str) -> Response {
        Response {
            intent,
            style_hint: None,
            body: body.to_string(),
            slots: Vec::new(),
        }
    }

    // ─── Architecture shape ─────────────────────────────────────────────

    #[test]
    fn reranker_default_backend_is_mock() {
        let r = IntentReranker::with_default_backend();
        assert_eq!(r.backend_name(), "mock");
    }

    #[test]
    fn reranker_with_explicit_backend_reports_its_name() {
        let r = IntentReranker::new(Box::new(MockReranker::new()), RerankConfig::default());
        assert_eq!(r.backend_name(), "mock");
    }

    #[test]
    fn rerank_returns_string_for_basic_response() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::SelfCheck, "I'm here.");
        let state = make_state(0.2, 0.5, 0.0, 0.5);
        let out = r.rerank(&response, &state);
        assert!(!out.is_empty(), "rerank must produce non-empty output for non-empty body");
    }

    #[test]
    fn rerank_never_panics_on_empty_body() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::Unknown, "");
        let state = InternalState::default();
        let out = r.rerank(&response, &state);
        // Empty body, no structured intent, no slots — backend returns empty.
        // Caller (voice engine) handles that gracefully.
        assert!(out.is_empty() || !out.is_empty(), "must not panic");
    }

    #[test]
    fn rerank_falls_back_to_raw_body_on_backend_error() {
        // A backend that always errors — reranker must fall back, not panic.
        struct AlwaysErrors;
        impl RerankerBackend for AlwaysErrors {
            fn name(&self) -> &'static str { "always_errors" }
            fn rewrite(&self, _: &RerankPrompt, _: &RerankConfig) -> Result<String, RerankError> {
                Err(RerankError::GenerationFailed("simulated".into()))
            }
        }
        let r = IntentReranker::new(Box::new(AlwaysErrors), RerankConfig::default());
        let response = make_response(ResponseIntent::SelfCheck, "the raw body");
        let state = InternalState::default();
        let out = r.rerank(&response, &state);
        assert_eq!(out, "the raw body", "must fall back to raw body on backend error");
    }

    // ─── MockReranker behaviour ─────────────────────────────────────────

    #[test]
    fn mock_passes_through_normal_body() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::Statement, "Just a fact.");
        let state = make_state(0.2, 0.5, 0.0, 0.5);
        let out = r.rerank(&response, &state);
        assert_eq!(out, "Just a fact.");
    }

    #[test]
    fn mock_self_check_high_uncertainty_prepends_honestly() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::SelfCheck, "I'm not sure.");
        let state = make_state(0.7, 0.5, 0.0, 0.5);
        let out = r.rerank(&response, &state);
        assert!(
            out.starts_with("Honestly,"),
            "SelfCheck + uncertainty>0.5 should prepend 'Honestly,': got '{}'",
            out
        );
    }

    #[test]
    fn mock_self_check_low_uncertainty_no_prepend() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::SelfCheck, "I'm fine.");
        let state = make_state(0.2, 0.5, 0.0, 0.5);
        let out = r.rerank(&response, &state);
        assert!(
            !out.starts_with("Honestly,"),
            "SelfCheck + low uncertainty should NOT prepend 'Honestly,': got '{}'",
            out
        );
    }

    #[test]
    fn mock_high_consciousness_substitutes_think_to_know() {
        let r = IntentReranker::with_default_backend();
        // SelfCheck defaults to Direct style (confidence hint 0.8) — the
        // mock's think→know substitution requires both high consciousness
        // AND a confident style. Statement has no style hint, so it
        // wouldn't trigger; SelfCheck is the realistic case.
        let response = make_response(
            ResponseIntent::SelfCheck,
            "I think this matters.",
        );
        let state = make_state(0.2, 0.9, 0.0, 0.5); // high consciousness
        let out = r.rerank(&response, &state);
        assert!(
            out.starts_with("I know."),
            "high consciousness + confident style should substitute 'I think' → 'I know.': got '{}'",
            out
        );
    }

    #[test]
    fn mock_reflection_high_engagement_appends_followup() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::Reflection, "I've been thinking.");
        let state = make_state(0.2, 0.5, 0.0, 0.9); // high engagement
        let out = r.rerank(&response, &state);
        assert!(
            out.ends_with("?") || out.contains("go deeper"),
            "Reflection + high engagement should append follow-up: got '{}'",
            out
        );
    }

    #[test]
    fn mock_empty_body_with_curiosity_falls_through_to_format() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::CuriosityCheck, "");
        let state = make_state(0.5, 0.5, 0.0, 0.5);
        let curiosity = CuriosityIntent::new("consciousness", 0.3, crate::metacog::intents::CuriosityKind::Confused);
        let out = r.rerank_with_curiosity(&response, &state, curiosity);
        assert!(
            out.contains("consciousness"),
            "empty body + curiosity should emit curiosity.format(): got '{}'",
            out
        );
    }

    // ─── RerankPrompt ───────────────────────────────────────────────────

    #[test]
    fn rerank_prompt_summary_includes_intent() {
        let response = make_response(ResponseIntent::SelfCheck, "test body");
        let state = make_state(0.3, 0.5, 0.0, 0.5);
        let prompt = RerankPrompt::from_response(&response, &state);
        let s = prompt.summary();
        assert!(s.contains("self_check"));
        assert!(s.contains("body_len=9"));
    }

    #[test]
    fn rerank_prompt_is_empty_detects_empty_inputs() {
        let response = make_response(ResponseIntent::Unknown, "");
        let state = InternalState::default();
        let prompt = RerankPrompt::from_response(&response, &state);
        assert!(prompt.is_empty());
    }

    // ─── Determinism ────────────────────────────────────────────────────

    #[test]
    fn mock_is_deterministic() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(ResponseIntent::SelfCheck, "I think X matters.");
        let state = make_state(0.5, 0.8, 0.0, 0.7);

        let out1 = r.rerank(&response, &state);
        let out2 = r.rerank(&response, &state);
        let out3 = r.rerank(&response, &state);
        assert_eq!(out1, out2);
        assert_eq!(out2, out3);
    }

    // ─── Post-processing ────────────────────────────────────────────────

    #[test]
    fn post_process_truncates_long_output() {
        let r = IntentReranker::with_default_backend();
        let response = make_response(
            ResponseIntent::Statement,
            "This is a body that is definitely longer than the max_chars limit we set.",
        );
        let mut state = make_state(0.2, 0.5, 0.0, 0.5);
        let _ = state; // not used by post_process directly
        // Use a custom config with very small max_chars.
        let cfg = RerankConfig {
            max_chars: Some(20),
            ..Default::default()
        };
        let r = IntentReranker::new(Box::new(MockReranker::new()), cfg);
        let out = r.rerank(&response, &make_state(0.2, 0.5, 0.0, 0.5));
        assert!(
            out.chars().count() <= 25, // allow for the trailing ellipsis
            "output should be truncated: got {} chars: '{}'",
            out.chars().count(),
            out,
        );
    }

    // ─── CharRnnBackend (shape only) ────────────────────────────────────

    #[test]
    fn char_rnn_backend_name_is_char_rnn() {
        let backend = CharRnnBackend::default();
        assert_eq!(backend.name(), "char_rnn");
    }

    #[test]
    fn char_rnn_backend_without_model_errors() {
        let backend = CharRnnBackend::default();
        let response = make_response(ResponseIntent::SelfCheck, "test");
        let state = InternalState::default();
        let prompt = RerankPrompt::from_response(&response, &state);
        let result = backend.rewrite(&prompt, &RerankConfig::default());
        assert!(matches!(result, Err(RerankError::ModelNotFound(_))));
    }
}
