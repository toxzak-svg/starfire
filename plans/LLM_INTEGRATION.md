# LLM Integration Plan — Qwen2-0.5B via llama-server

## Architecture

```
Starfire (Rust)
  └─► lib/llm/   ──► ureq HTTP client ──► localhost:8080
                                            └─► llama-server (subprocess)
                                                  └─► Qwen2-0.5B Q4_K_M.gguf
```

**Why this approach:**
- llama-server is a standalone binary — no CGO, no compile pain
- HTTP interface is clean and well-tested
- Starfire talks to it as a client — no threading issues
- Can be swapped for native Rust GGUF later without changing the interface
- 20-30 tok/sec on CPU — fast enough for interactive use

## llama-server Setup

### 1. Download Qwen2-0.5B GGUF
```bash
# Download Qwen2-0.5B Q4_K_M (352 MB) from HuggingFace
curl -L -o qwen2-0.5b-q4_k_m.gguf \
  https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF/resolve/main/qwen2-0.5b-instruct-q4_k_m.gguf
```

Alternative (if HuggingFace blocked): use TheBloke's mirror.

### 2. Download llama-server (Windows binary)
```bash
# Download from GitHub releases
curl -L -o llama-server.exe \
  https://github.com/ggml-org/llama.cpp/releases/latest/download/llama-server-windows-x64.exe
```

Or build from source with `cmake --build . --target llama-server`.

### 3. Test manually
```bash
./llama-server.exe -m qwen2-0.5b-q4_k_m.gguf -c 2048 --host 127.0.0.1 --port 8080
```

Test with curl:
```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"What is 2+2?"}]}'
```

## Starfire lib/llm Module

### Interface Design

```rust
// lib/llm/mod.rs

pub struct LlmEngine {
    base_url: String,   // "http://127.0.0.1:8080"
    model: String,      // "qwen2-0.5b"
    stream: bool,
}

impl LlmEngine {
    /// Create a new LLM engine (does NOT start the server)
    pub fn new(base_url: &str, model: &str) -> Self { ... }

    /// Check if the LLM server is running and healthy
    pub async fn health_check(&self) -> bool { ... }

    /// Generate a completion (non-streaming, simplest interface)
    pub async fn generate(&self, prompt: &str) -> anyhow::Result<String> { ... }

    /// Generate with a chat template
    pub async fn chat(&self, messages: &[ChatMessage]) -> anyhow::Result<String> { ... }

    /// Generate with system prompt override
    pub async fn generate_with_system(
        &self,
        system: &str,
        user: &str,
    ) -> anyhow::Result<String> { ... }

    /// Stream tokens (for interactive output)
    pub async fn stream_generate(
        &self,
        prompt: &str,
        on_token: impl Fn(String),
    ) -> anyhow::Result<()> { ... }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,    // "user" | "assistant" | "system"
    pub content: String,
}
```

### Integration Points

1. **Conversation layer** (`lib/conversation/`): After Quanot processes input and generates a response, the LLM polishes/fluencies it
2. **Book sweep results**: LLM reads section content and generates natural language
3. **Memory**: LLM helps format memories into readable summaries

### Startup Sequence

Starfire checks for LLM availability on startup:
1. Try to connect to `http://127.0.0.1:8080/health`
2. If unreachable, log warning but continue (LLM is optional enhancement)
3. If reachable, expose LLM capability in status

### Process Management

In `star/bin/main.rs` or a new `star/bin/llm_manager.rs`:

```rust
pub struct LlmManager {
    server_path: PathBuf,
    model_path: PathBuf,
    port: u16,
    child: Option<Child>,
}

impl LlmManager {
    /// Start llama-server as a background subprocess
    pub fn start(&mut self) -> anyhow::Result<()> {
        let child = std::process::Command::new(&self.server_path)
            .args([
                "-m", self.model_path.to_str().unwrap(),
                "-c", "2048",
                "--host", "127.0.0.1",
                "--port", &self.port.to_string(),
                "--log-disable",
            ])
            .spawn()?;
        self.child = Some(child);
        Ok(())
    }

    /// Stop the server
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}
```

## Files to Create

```
projects/starfire/lib/llm/
├── mod.rs          — LlmEngine, ChatMessage, public API
├── client.rs       — HTTP client wrapper (ureq-based)
├── chat.rs         — chat completion API calls
└── tests.rs        — unit tests (mock HTTP responses)

projects/starfire/scripts/
├── download_model.sh    — download Qwen2-0.5B GGUF
├── download_llama.cpp.sh — download llama-server binary
└── start-llm.sh         — start llama-server with correct args
```

## Quantization Notes

- Q4_K_M: 352 MB, good quality/size tradeoff
- Q8_0: 600 MB, near-float quality (if space permits)
- Q2_K: 200 MB, very aggressive (for embedded/MCU use)
- Qwen2-0.5B native: 1 GB

For Railway deployment (1 GB RAM):
- Q4_K_M is the sweet spot
- Leave headroom for Starfire (~200-300 MB)
- Total: ~650 MB out of 1 GB

## Status
- [x] Plan written
- [x] Build lib/llm module ✅ (245 tests passing)
- [ ] Download Qwen2-0.5B GGUF
- [ ] Download/compile llama-server
- [ ] Wire into conversation layer
- [ ] Test end-to-end
