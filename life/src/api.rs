//! Star HTTP API Server
//!
//! A simple HTTP wrapper around Star's reasoning engine.

use crate::{Runtime, ReasoningEngine, Memory};
use crate::persistence::MemoryDomain;
use anyhow::Result;
use std::io::Read;
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
            ("POST", "/remember") => (200, handle_remember(runtime, &body_str)),
            ("GET", "/identity") => (200, handle_identity(runtime)),
            ("GET", "/memory/stats") => (200, handle_memory_stats(runtime)),
            ("GET", "/health") => (200, r#"{"status":"ok"}"#.to_string()),
            ("GET", "/") => (200, r#"{"name":"Star","version":"0.1","endpoints":["/reason","/remember","/identity","/memory/stats","/health"]}"#.to_string()),
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
        .map(|s| Memory::new(s, MemoryDomain::Episodic, 0.5))
        .collect();

    let rt_guard = match runtime.lock() {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Lock poisoned: {}"}}"#, e),
    };

    let mut engine = ReasoningEngine::new();
    let result = engine.reason(&req.query, &memories);

    serde_json::json!({
        "answer": result.answer,
        "confidence": format!("{:?}", result.confidence).to_lowercase(),
        "confidence_score": result.confidence_score,
        "reasoning_chain": result.reasoning_chain,
    }).to_string()
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
            "confidence": m.current_confidence(chrono::Utc::now().timestamp()),
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
