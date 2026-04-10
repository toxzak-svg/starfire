//! LLM Inference HTTP Server
//!
//! Wraps Bonsai-8B (or any GGUF model) behind an OpenAI-compatible REST API.
//! Runs as a standalone service; star-core calls it over HTTP.
//!
//! ## Endpoints
//!
//! - `GET  /health`              — health check (200 OK)
//! - `POST /v1/chat/completions`  — OpenAI-compatible chat completions
//! - `POST /v1/completions`       — OpenAI-compatible completions
//!
//! ## Environment
//!
//! - `GGUF_PATH`   — path to the GGUF model file (default: /models/bonzai-8b/Bonsai-8B.gguf)
//! - `PORT`        — HTTP port (default: 8081)
//! - `HOST`        — bind host (default: 0.0.0.0)

use std::io::Read;
use std::sync::Mutex;
use candle_core::{Device, Result as CandleResult, Tensor};
use candle_transformers::models::quantized_llama as qlm;
use serde::{Deserialize, Serialize};

// ============================================================
// Shared engine state
// ============================================================

struct AppState {
    engine: Mutex<Option<LlmEngine>>,
}

struct LlmEngine {
    model: qlm::ModelWeights,
    device: Device,
    vocab_size: usize,
    max_seq_len: usize,
}

impl LlmEngine {
    fn new(gguf_path: &str) -> anyhow::Result<Self> {
        let device = Device::Cpu;
        let mut file = std::fs::File::open(gguf_path)?;
        let model = qlm::ModelWeights::from_gguf(
            candle_core::quantized::gguf_file::Content::read(&mut file)?,
            &mut file,
            &device,
        )?;
        let vocab_size = 151669;
        let max_seq_len = 4096;
        Ok(Self { model, device, vocab_size, max_seq_len })
    }

    fn generate(&mut self, prompt: &str, max_new_tokens: u32) -> anyhow::Result<String> {
        let mut token_ids = self.tokenize(prompt);
        let original_len = token_ids.len();

        for _ in 0..max_new_tokens.min(512) {
            let input_len = token_ids.len().min(self.max_seq_len);
            let input_tokens: Vec<u32> = token_ids.iter().rev().take(input_len).rev().cloned().collect();
            let input = Tensor::new(input_tokens.as_slice(), &self.device)?
                .reshape(&[1, input_tokens.len()])?;

            let logits = self.model.forward(&input, 0)?.squeeze(0)?;
            let logits_v = logits.to_vec1::<f32>()?;

            let next = self.sample_token(&logits_v);
            if next == 0 || next >= self.vocab_size { break; }

            token_ids.push(next as u32);

            if token_ids.len() - original_len >= self.max_seq_len { break; }
        }

        Ok(self.detokenize(&token_ids))
    }

    fn chat(&mut self, messages: &[ChatMessageIn], max_new_tokens: u32) -> anyhow::Result<String> {
        let system = messages.iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let prompt: String = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let full = if system.is_empty() {
            prompt
        } else {
            format!("System: {}\n{}", system, prompt)
        };

        self.generate(&full, max_new_tokens)
    }

    fn tokenize(&self, text: &str) -> Vec<u32> {
        // Byte-level approximation — replace with proper tokenizer for production
        let mut tokens = Vec::with_capacity(text.len() * 2);
        for byte in text.bytes() {
            let t = match byte {
                b if b.is_ascii_alphabetic() || b == b' ' => byte as u32,
                _ => byte as u32,
            };
            tokens.push(t.min(self.vocab_size as u32 - 1));
        }
        tokens
    }

    fn detokenize(&self, token_ids: &[u32]) -> String {
        // Byte-level approximation
        let bytes: Vec<u8> = token_ids.iter()
            .filter(|&&t| t < 256)
            .map(|&t| t as u8)
            .collect();
        String::from_utf8_lossy(&bytes).to_string()
    }

    fn sample_token(&self, logits: &[f32]) -> usize {
        logits.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn health_check(&self) -> bool {
        let tokens = [0u32, 1, 2, 3];
        if let Ok(input) = Tensor::new(tokens.as_slice(), &self.device) {
            if let Ok(reshaped) = input.reshape(&[1, 4]) {
                return self.model.forward(&reshaped, 0).is_ok();
            }
        }
        false
    }
}

// ============================================================
// HTTP server (tiny_http is already in workspace deps)
// ============================================================

fn handle_health(state: &AppState, resp_writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let healthy = {
        let guard = state.engine.lock().unwrap();
        guard.as_ref().map(|e| e.health_check()).unwrap_or(false)
    };

    if healthy {
        write_response(resp_writer, 200, "OK", "{\"status\":\"healthy\"}")
    } else {
        write_response(resp_writer, 503, "Service Unavailable", "{\"status\":\"model_not_loaded\"}")
    }
}

fn handle_chat(state: &AppState, body: &[u8], resp_writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let Ok(request) = serde_json::from_slice::<ChatCompletionsRequest>(body) else {
        return write_response(resp_writer, 400, "Bad Request", "{\"error\":\"invalid request body\"}");
    };

    let max_tokens = request.max_tokens.unwrap_or(256);

    let generated = {
        let mut guard = state.engine.lock().unwrap();
        match guard.as_mut() {
            Some(engine) => {
                match engine.chat(&request.messages, max_tokens) {
                    Ok(text) => text,
                    Err(e) => {
                        return write_response(resp_writer, 500, "Internal Server Error",
                            &format!("{{\"error\":\"generation failed: {}\"}}", e));
                    }
                }
            }
            None => {
                return write_response(resp_writer, 503, "Service Unavailable",
                    "{\"error\":\"model not loaded\"}");
            }
        }
    };

    let response = ChatCompletionsResponse {
        id: format!("chatcmpl-{}", uuid_simple()),
        object: "chat.completion".to_string(),
        created: unix_timestamp(),
        model: request.model.clone(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessageOut { role: "assistant".to_string(), content: generated },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        },
    };

    let json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    write_response(resp_writer, 200, "OK", &json)
}

fn handle_completion(state: &AppState, body: &[u8], resp_writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let Ok(request) = serde_json::from_slice::<CompletionRequest>(body) else {
        return write_response(resp_writer, 400, "Bad Request", "{\"error\":\"invalid request body\"}");
    };

    let max_tokens = request.max_tokens.unwrap_or(256);

    let generated = {
        let mut guard = state.engine.lock().unwrap();
        match guard.as_mut() {
            Some(engine) => {
                match engine.generate(&request.prompt, max_tokens) {
                    Ok(text) => text,
                    Err(e) => {
                        return write_response(resp_writer, 500, "Internal Server Error",
                            &format!("{{\"error\":\"generation failed: {}\"}}", e));
                    }
                }
            }
            None => {
                return write_response(resp_writer, 503, "Service Unavailable",
                    "{\"error\":\"model not loaded\"}");
            }
        }
    };

    let response = CompletionResponse {
        id: format!("cmpl-{}", uuid_simple()),
        object: "text_completion".to_string(),
        created: unix_timestamp(),
        model: request.model.clone(),
        choices: vec![CompletionChoice {
            index: 0,
            text: generated,
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        },
    };

    let json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    write_response(resp_writer, 200, "OK", &json)
}

fn write_response(w: &mut dyn std::io::Write, status: u16, status_text: &str, body: &str) -> std::io::Result<()> {
    let body_len = body.len();
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status, status_text, body_len, body
    );
    w.write_all(response.as_bytes())
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn uuid_simple() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:032x}", rng.gen::<u128>())
}

// ============================================================
// OpenAI-compatible request/response types
// ============================================================

#[derive(Debug, Deserialize)]
struct ChatMessageIn {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessageIn>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionsResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct ChatMessageOut {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatChoice {
    index: usize,
    message: ChatMessageOut,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    echo: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<CompletionChoice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct CompletionChoice {
    index: usize,
    text: String,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_tokens: Option<u32>,
}

// ============================================================
// Main
// ============================================================

fn main() -> anyhow::Result<()> {
    // Simple logger
    println!("[llm-server] starting...");

    let gguf_path = std::env::var("GGUF_PATH").unwrap_or_else(|_| "/models/bonsai-8b/Bonsai-8B.gguf".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8081".to_string()).parse().unwrap_or(8081);

    println!("[llm-server] loading model from: {}", gguf_path);

    // Load model eagerly — takes a few seconds
    let engine = LlmEngine::new(&gguf_path)?;
    println!("[llm-server] model loaded successfully");

    let state = std::sync::Arc::new(AppState {
        engine: Mutex::new(Some(engine)),
    });

    let addr = format!("{}:{}", host, port);
    println!("[llm-server] listening on http://{}", addr);

    let server = tiny_http::Server::http(&addr).map_err(|e| anyhow::anyhow!("failed to bind: {}", e))?;

    for request in server.incoming_requests() {
        let state_clone = state.clone();
        std::thread::spawn(move || {
            let path = request.url().to_string();
            let method = request.method().to_string();

            let mut resp_body = Vec::new();
            request.as_reader().read_to_end(&mut resp_body).ok();

            let mut writer = Vec::new();
            let result = match (method.as_str(), path.as_str()) {
                ("GET", "/health") | ("GET", "/v1/health") => {
                    handle_health(&state_clone, &mut writer)
                }
                ("POST", "/v1/chat/completions") => {
                    handle_chat(&state_clone, &resp_body, &mut writer)
                }
                ("POST", "/v1/completions") => {
                    handle_completion(&state_clone, &resp_body, &mut writer)
                }
                _ => {
                    write_response(&mut writer, 404, "Not Found",
                        &format!("{{\"error\":\"unknown endpoint: {} {}\"}}", method, path))
                }
            };

            if let Err(e) = result {
                eprintln!("[llm-server] handler error: {}", e);
            }

            let _ = request.as_response().map(|r| r.into_writer()).and_then(|mut w| {
                w.write_all(&writer)
            });
        });
    }

    Ok(())
}
