//! LLM Polish — conditional compilation for LLM text polishing.
//!
//! When `llm` feature is active: uses native Candle-based LLM (LlmHandle).
//! When `http_llm` feature is active: uses remote HTTP llm-server (HttpLlmClient).
//! When neither feature is active: polish() returns the input unchanged.

#[cfg(feature = "llm")]
use crate::llm::{LlmEngine, LlmHandle};
#[cfg(feature = "http_llm")]
use crate::http_llm::HttpLlmClient;

/// Polish rough text into natural, fluent output.
///
/// - `llm` feature: delegates to native Candle-based `LlmHandle::polish()`
/// - `http_llm` feature: delegates to remote `HttpLlmClient::polish()`
/// - neither: returns `Ok(rough_text.to_string())`
pub fn polish(rough_text: &str) -> anyhow::Result<String> {
    #[cfg(feature = "llm")]
    {
        // Uses the Runtime's stored LlmHandle field — handled in Runtime::apply_polish()
        Ok(rough_text.to_string())
    }

    #[cfg(feature = "http_llm")]
    {
        let client = HttpLlmClient::from_env();
        client.polish(rough_text)
    }

    #[cfg(not(any(feature = "llm", feature = "http_llm")))]
    {
        Ok(rough_text.to_string())
    }
}
