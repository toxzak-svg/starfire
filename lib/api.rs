//! Star HTTP API Server
//!
//! A simple HTTP wrapper around Star's reasoning engine.

use crate::{Runtime, Memory};
use crate::persistence::MemoryDomain;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Start the Star HTTP API server.
pub fn start(runtime: Arc<Mutex<Runtime>>, host: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    info!("Starting Star API server at http://{}", addr);

    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
    info!("Star API ready at http://{}/", addr);

    for request in server.incoming_requests() {
        if let Err(e) = handle_request(&runtime, request) {
            warn!("Request failed: {}", e);
        }
    }
    
    Ok(())
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn handle_request(runtime: &Arc<Mutex<Runtime>>, mut request: tiny_http::Request) -> Result<()> {
    let path = request.url().to_string();
    let method = request.method().as_str().to_string();
    
    // Handle CORS preflight immediately
    if method == "OPTIONS" {
        let response = tiny_http::Response::from_string(String::new())
            .with_status_code(204)
            .with_header(header("Access-Control-Allow-Origin", "*"))
            .with_header(header("Access-Control-Allow-Methods", "GET,POST,OPTIONS"))
            .with_header(header("Access-Control-Allow-Headers", "Content-Type"));
        match request.respond(response) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("IO error: {}", e)),
        }
    } else {
        // Read request body
        let mut body = Vec::new();
        request.as_reader().read_to_end(&mut body).map_err(|e| anyhow::anyhow!("Read error: {}", e))?;
        let body_str = String::from_utf8_lossy(&body).to_string();

        // Build response based on route
        let (status, response_body) = match (method.as_str(), path.as_str()) {
            ("POST", "/reason") => (200, handle_reason(runtime, &body_str)),
            ("POST", "/chat") => (200, handle_chat(runtime, &body_str)),
            ("POST", "/remember") => (200, handle_remember(runtime, &body_str)),
            ("GET", "/identity") => (200, handle_identity(runtime)),
            ("GET", "/memory/stats") => (200, handle_memory_stats(runtime)),
            ("GET", "/health") => (200, r#"{"status":"ok"}"#.to_string()),
            ("GET", "/cognitive") => (200, handle_cognitive(runtime)),
            ("GET", "/metacog") => (200, handle_metacog(runtime)),
            ("GET", "/metacog/insight") => (200, handle_metacog_insight(runtime)),
            ("GET", "/think") => (200, handle_think(runtime)),
            ("GET", "/thought") => (200, handle_thought(runtime)),
            ("GET", "/") => (200, r#"{"name":"Star","version":"0.1","endpoints":["/reason","/chat","/remember","/identity","/memory/stats","/health","/cognitive","/metacog","/metacog/insight","/think","/thought","/webhook/telegram"]}"#.to_string()),
            ("POST", "/webhook/telegram") => (200, handle_webhook_telegram(runtime, &body_str)),
            _ => {
                warn!("Unknown route: {} {}", method, path);
                (404, r#"{"error":"Not found"}"#.to_string())
            }
        };

        // Send response (consumes request)
        let response = tiny_http::Response::from_data(response_body.into_bytes())
            .with_status_code(status)
            .with_header(header("Content-Type", "application/json"))
            .with_header(header("Access-Control-Allow-Origin", "*"));
        match request.respond(response) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("IO error: {}", e)),
        }
    }
}

fn handle_reason(runtime: &Arc<Mutex<Runtime>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct ReasonRequest {
        query: String,
        memories: Option<Vec<String>>,
    }

    let req: ReasonRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Invalid request: {}"}}"#, e),
    };

    let memories: Vec<Memory> = req.memories
        .unwrap_or_default()
        .into_iter()
        .map(|s| Memory::new(&s, MemoryDomain::Episodic, 0.5))
        .collect();

    let mut rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    // Use the runtime's reason method — this connects to the fully initialized
    // reasoning engine with seed knowledge and the memory store, instead of
    // spinning up a fresh empty engine each request (which was the old bug).
    let result = rt_guard.reason(&req.query, &memories);

    serde_json::json!({
        "answer": result.answer,
        "confidence": format!("{:?}", result.confidence).to_lowercase(),
        "confidence_score": result.confidence_score,
        "reasoning_chain": result.reasoning_chain,
    }).to_string()
}

fn handle_cognitive(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };
    
    let cog = rt_guard.cognition();
    let reasoning_trace: Vec<serde_json::Value> = cog.reasoning_trace.iter().map(|step| {
        serde_json::json!({
            "input": step.input,
            "conclusion": step.conclusion,
            "chain": step.chain,
            "confidence": format!("{:?}", step.confidence).to_lowercase(),
            "timestamp": step.timestamp,
        })
    }).collect();
    
    let response = serde_json::json!({
        "current_focus": cog.current_focus,
        "certainty": cog.certainty,
        "open_questions": cog.open_questions,
        "last_reasoning": cog.last_reasoning,
        "reasoning_trace": reasoning_trace,
    });
    
    serde_json::to_string(&response).unwrap_or_else(|e| format!(r#"{{"error":"{}",}}"#, e))
}

fn handle_metacog(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };
    
    let metacog = rt_guard.metacognition_ref();
    
    // Get reasoning history
    let reasoning_history: Vec<serde_json::Value> = metacog.reasoning_history().iter().rev().take(10).map(|r| {
        serde_json::json!({
            "query": r.query,
            "conclusion": r.conclusion,
            "confidence": format!("{:?}", r.confidence).to_lowercase(),
            "was_surprising": r.was_surprising,
            "timestamp": r.timestamp,
        })
    }).collect();
    
    // Get beliefs
    let beliefs: Vec<serde_json::Value> = metacog.all_beliefs().iter().map(|(topic, belief)| {
        serde_json::json!({
            "topic": topic,
            "content": belief.content,
            "confidence": format!("{:?}", belief.confidence_state).to_lowercase(),
        })
    }).collect();
    
    // Get surprising conclusions
    let surprising: Vec<String> = metacog.surprising_conclusions().iter().map(|r| r.conclusion.clone()).collect();
    
    // Get top knowledge gap
    let top_gap = metacog.top_gap().map(|g| serde_json::json!({
        "topic": g.topic,
        "importance": g.importance,
        "investigated": g.investigated,
        "progress": g.progress,
    }));
    
    let response = serde_json::json!({
        "beliefs": beliefs,
        "reasoning_history": reasoning_history,
        "surprising_conclusions": surprising,
        "top_gap": top_gap,
        "curiosity_topics": metacog.curiosity_topics(),
    });
    
    serde_json::to_string(&response).unwrap_or_else(|e| format!(r#"{{"error":"{}",}}"#, e))
}

fn handle_metacog_insight(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    // Phase 2 (voice-refine 2026-06-21): generate_insight() now returns a
    // structured InsightIntent. We expose both the structured fields (for
    // the voice engine to consume) and the legacy formatted prose (for the
    // HTTP endpoint to return as before).
    let insight = rt_guard.metacognition_ref().generate_insight();

    let (has_insight, kind_str, topic, formatted) = match insight {
        Some(i) => (
            true,
            Some(format!("{:?}", i.kind)),
            i.topic.clone(),
            Some(i.format()),
        ),
        None => (false, None, None, None),
    };

    serde_json::json!({
        "has_insight": has_insight,
        "kind": kind_str,
        "topic": topic,
        "insight": formatted,
    })
    .to_string()
}

fn handle_chat(runtime: &Arc<Mutex<Runtime>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct ChatRequest {
        message: String,
    }

    let req: ChatRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Invalid request: {}"}}"#, e),
    };

    let mut rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    match rt_guard.chat(&req.message) {
        Ok(response) => serde_json::json!({ "response": response }).to_string(),
        Err(e) => format!(r#"{{"error":"Chat error: {}"}}"#, e),
    }
}

fn handle_remember(runtime: &Arc<Mutex<Runtime>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct RememberRequest {
        topic: String,
        limit: Option<usize>,
    }

    let req: RememberRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Invalid request: {}"}}"#, e),
    };

    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    let memories = rt_guard.get_memories(&req.topic, req.limit.unwrap_or(5));

    let results: Vec<serde_json::Value> = memories.iter().map(|m| {
        serde_json::json!({
            "content": m.content,
            "domain": format!("{:?}", m.domain).to_lowercase(),
            "importance": m.importance,
            "confidence": m.current_confidence(crate::now_timestamp()),
        })
    }).collect();

    serde_json::to_string(&results).unwrap_or_else(|e| format!(r#"{{"error":"Serialization: {}"}}"#, e))
}

fn handle_identity(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    serde_json::json!({
        "name": "Star",
        "summary": rt_guard.identity_summary(),
        "relationship": rt_guard.relationship_to_zachary(),
        "session_id": rt_guard.session_id(),
    }).to_string()
}

fn handle_memory_stats(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    let snap = rt_guard.store_snapshot();

    serde_json::json!({
        "memory_count": snap.memory_count,
        "beliefs_count": snap.beliefs_count,
        "sessions_count": snap.sessions_count,
        "domain_breakdown": snap.domain_breakdown,
    }).to_string()
}

fn handle_think(runtime: &Arc<Mutex<Runtime>>) -> String {
    let mut rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    let thought = rt_guard.think();

    let kind_str = match &thought.kind {
        crate::runtime::ThoughtKind::Question(q) => {
            serde_json::json!({ "type": "question", "text": q })
        }
        crate::runtime::ThoughtKind::Insight(i) => {
            serde_json::json!({ "type": "insight", "text": i })
        }
        crate::runtime::ThoughtKind::Connection(c) => {
            serde_json::json!({ "type": "connection", "text": c })
        }
    };

    serde_json::json!({
        "thought": kind_str,
        "topic": thought.topic,
        "confidence": format!("{:?}", thought.confidence).to_lowercase(),
        "generated_by": thought.generated_by,
        "tentative_answer": thought.tentative_answer,
    }).to_string()
}

/// Get Star's last autonomous thought (for external observers).
/// This is what Star is "thinking about" between conversations.
fn handle_thought(runtime: &Arc<Mutex<Runtime>>) -> String {
    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    match rt_guard.last_autonomous_thought() {
        Some(thought) => {
            let kind_str = match &thought.kind {
                crate::runtime::ThoughtKind::Question(q) => {
                    serde_json::json!({ "type": "question", "text": q })
                }
                crate::runtime::ThoughtKind::Insight(i) => {
                    serde_json::json!({ "type": "insight", "text": i })
                }
                crate::runtime::ThoughtKind::Connection(c) => {
                    serde_json::json!({ "type": "connection", "text": c })
                }
            };
            serde_json::json!({
                "thought": kind_str,
                "topic": thought.topic,
                "confidence": format!("{:?}", thought.confidence).to_lowercase(),
                "generated_by": thought.generated_by,
            }).to_string()
        }
        None => {
            r#"{"thought":null,"message":"Star has no pending autonomous thoughts"}"#.to_string()
        }
    }
}

/// Handle incoming Telegram webhook updates.
fn handle_webhook_telegram(runtime: &Arc<Mutex<Runtime>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct TgUpdate {
        update_id: u64,
        message: Option<TgMessage>,
    }

    #[derive(serde::Deserialize)]
    struct TgMessage {
        message_id: u64,
        chat: TgChat,
        text: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct TgChat {
        id: i64,
    }

    let update: TgUpdate = match serde_json::from_str(body) {
        Ok(u) => u,
        Err(e) => return format!(r#"{{"error":"Failed to parse update: {}"}}"#, e),
    };

    let message = match update.message {
        Some(m) => m,
        None => return r#"{"ok":true,"response":"no message"}"#.to_string(),
    };

    let text = match message.text {
        Some(t) if !t.is_empty() => t,
        _ => return r#"{"ok":true,"response":"no text"}"#.to_string(),
    };

    let chat_id = message.chat.id;
    let message_id = message.message_id;

    // Forward to Star's chat
    let star_response = {
        let mut rt_guard = match runtime.lock() {
            Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
            Ok(r) => r,
        };
        rt_guard.chat(&text).unwrap_or_else(|e| format!("Error: {}", e))
    };

    // Send response back to Telegram
    if let Ok(token) = std::env::var("TELEGRAM_BOT_TOKEN") {
        let send_url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": star_response,
            "reply_to_message_id": message_id,
        });

        // Spawn a thread for the Telegram API call (non-blocking)
        std::thread::spawn(move || {
            let _ = ureq::post(&send_url)
                .set("Content-Type", "application/json")
                .send_string(&serde_json::to_string(&payload).unwrap_or_default());
        });
    }

    serde_json::json!({
        "ok": true,
        "response": star_response,
        "chat_id": chat_id,
        "update_id": update.update_id,
    }).to_string()
}
