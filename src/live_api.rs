//! Live Integration 1 for the deployed HTTP surface.
//!
//! The existing API remains the protected content producer. This module runs it
//! on a loopback port, then gives successful `/chat` responses one production
//! path through a typed semantic plan, persistent VoiceState, deterministic
//! rendering, and an inspectable append-only trace.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use star::omega_v1_semantic_plan::{
    ClaimProvenance, DetailBudget, DialoguePolicy, EmotionalPosition, EpistemicConfidence,
    GroundedClaim, InitiativeLevel, PlanConfidenceLevel, PlanIntent, ProhibitedImplication,
    ResponseStance, SemanticOperation, SemanticOperationKind, SemanticResponsePlan,
};
use star::runtime::response_intent::{self, ResponseIntent};
use star::voice_state::{
    BasisPoints, BoundedVoiceDelta, VoiceDebugProjection, VoiceDimension, VoiceEvidenceKind,
    VoiceEvidenceRef, VoiceRevisionEvent, VoiceRevisionReason, VoiceRevisionTarget, VoiceState,
};
use star::{api, now_timestamp, Runtime};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{info, warn};

const LIVE_STATE_FILE: &str = "live_voice_state.json";
const LIVE_TRACE_FILE: &str = "live_chat_trace.jsonl";
const LIVE_SESSION_ID: &str = "starfire-http-live-v1";
const LIVE_PIPELINE: &str = "live-integration-1";

#[derive(Debug, Clone, Serialize)]
struct LiveTrace {
    trace_id: String,
    timestamp: i64,
    turn: u64,
    response_intent: String,
    semantic_plan: SemanticResponsePlan,
    voice_before: VoiceDebugProjection,
    voice_after: VoiceDebugProjection,
    raw_response: String,
    rendered_response: String,
}

struct LiveIntegration {
    state_path: PathBuf,
    trace_path: PathBuf,
    voice_state: VoiceState,
    last_trace: Option<LiveTrace>,
}

impl LiveIntegration {
    fn load(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir).with_context(|| {
            format!(
                "create live integration data directory {}",
                data_dir.display()
            )
        })?;

        let state_path = data_dir.join(LIVE_STATE_FILE);
        let trace_path = data_dir.join(LIVE_TRACE_FILE);
        let voice_state = match fs::read_to_string(&state_path) {
            Ok(json) => match VoiceState::from_canonical_json(&json) {
                Ok(state) => state,
                Err(error) => {
                    warn!(
                        "live integration: invalid persisted VoiceState at {}: {}; starting neutral",
                        state_path.display(),
                        error
                    );
                    VoiceState::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => VoiceState::default(),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("read persisted VoiceState {}", state_path.display())
                });
            }
        };

        Ok(Self {
            state_path,
            trace_path,
            voice_state,
            last_trace: None,
        })
    }

    fn process(&mut self, input: &str, raw_response: &str) -> Result<LiveTrace> {
        let response_intent = response_intent::classify(input);
        let voice_before = self.voice_state.debug_projection()?;
        let next_version = self
            .voice_state
            .version
            .checked_add(1)
            .ok_or_else(|| anyhow!("live VoiceState version overflow"))?;
        let trace_id = format!("live-{}-{}", now_timestamp(), next_version);
        let semantic_plan = build_live_plan(&trace_id, input, raw_response, &response_intent);
        let revision = revision_for(
            &trace_id,
            input,
            &response_intent,
            self.voice_state.version,
        )?;

        self.voice_state
            .apply_revision(self.voice_state.version, revision)?;
        let voice_after = self.voice_state.debug_projection()?;
        let rendered_response = render_live_response(&semantic_plan, &voice_after, raw_response);
        self.persist_state()?;

        let trace = LiveTrace {
            trace_id,
            timestamp: now_timestamp(),
            turn: self.voice_state.version,
            response_intent: response_intent.label().to_owned(),
            semantic_plan,
            voice_before,
            voice_after,
            raw_response: raw_response.to_owned(),
            rendered_response,
        };
        self.append_trace(&trace)?;
        self.last_trace = Some(trace.clone());
        Ok(trace)
    }

    fn persist_state(&self) -> Result<()> {
        let json = self.voice_state.to_canonical_json()?;
        let temporary = self.state_path.with_extension("json.tmp");
        fs::write(&temporary, json)
            .with_context(|| format!("write temporary VoiceState {}", temporary.display()))?;
        fs::rename(&temporary, &self.state_path).with_context(|| {
            format!(
                "atomically replace VoiceState {} with {}",
                self.state_path.display(),
                temporary.display()
            )
        })?;
        Ok(())
    }

    fn append_trace(&self, trace: &LiveTrace) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.trace_path)
            .with_context(|| format!("open live trace {}", self.trace_path.display()))?;
        serde_json::to_writer(&mut file, trace).context("serialize live trace")?;
        file.write_all(b"\n")
            .context("terminate live trace record")?;
        file.flush().context("flush live trace")?;
        Ok(())
    }

    fn status(&self) -> Value {
        json!({
            "enabled": true,
            "pipeline": LIVE_PIPELINE,
            "turn": self.voice_state.version,
            "voice_state": self.voice_state.debug_projection().ok(),
            "last_trace": &self.last_trace,
        })
    }
}

pub fn start(
    runtime: Arc<Mutex<Runtime>>,
    host: &str,
    port: u16,
    data_dir: &Path,
) -> Result<()> {
    let internal_port = internal_port(port);
    let internal_runtime = runtime;
    thread::Builder::new()
        .name("starfire-protected-api".to_owned())
        .spawn(move || {
            if let Err(error) = api::start(internal_runtime, "127.0.0.1", internal_port) {
                warn!("protected Starfire API exited: {}", error);
            }
        })
        .context("spawn protected Starfire API")?;

    wait_for_internal_api(internal_port)?;

    let integration = Arc::new(Mutex::new(LiveIntegration::load(data_dir)?));
    let address = format!("{}:{}", host, port);
    let server = tiny_http::Server::http(&address)
        .map_err(|error| anyhow!("live API server error: {}", error))?;
    info!(
        "Starfire Live Integration 1 ready at http://{} (protected API 127.0.0.1:{})",
        address, internal_port
    );

    for request in server.incoming_requests() {
        if let Err(error) = handle_request(request, internal_port, &integration) {
            warn!("live API request failed: {}", error);
        }
    }
    Ok(())
}

fn internal_port(external_port: u16) -> u16 {
    if let Some(parsed) = std::env::var("STARFIRE_INTERNAL_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|parsed| *parsed != external_port)
    {
        return parsed;
    }
    external_port.checked_add(1).unwrap_or(18_081)
}

fn wait_for_internal_api(port: u16) -> Result<()> {
    let url = format!("http://127.0.0.1:{}/health", port);
    for _ in 0..100 {
        if ureq::get(&url).call().is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(anyhow!(
        "protected Starfire API did not become ready at {}",
        url
    ))
}

fn handle_request(
    mut request: tiny_http::Request,
    internal_port: u16,
    integration: &Arc<Mutex<LiveIntegration>>,
) -> Result<()> {
    let method = request.method().as_str().to_owned();
    let path = request.url().to_owned();

    if method == "OPTIONS" {
        return respond(request, 204, String::new());
    }

    if method == "GET" && path == "/live/status" {
        let body = integration
            .lock()
            .map_err(|error| anyhow!("live integration lock poisoned: {}", error))?
            .status()
            .to_string();
        return respond(request, 200, body);
    }

    let mut body = Vec::new();
    request
        .as_reader()
        .read_to_end(&mut body)
        .context("read external request body")?;

    let (status, protected_body) = forward(&method, &path, &body, internal_port);
    let response_body = if method == "POST" && path == "/chat" && (200..300).contains(&status) {
        transform_chat_envelope(&body, &protected_body, integration)
    } else {
        protected_body
    };

    respond(request, status, response_body)
}

fn forward(method: &str, path: &str, body: &[u8], port: u16) -> (u16, String) {
    let url = format!("http://127.0.0.1:{}{}", port, path);
    let forwarded = ureq::request(method, &url).set("Content-Type", "application/json");
    let result = if body.is_empty() {
        forwarded.call()
    } else {
        forwarded.send_bytes(body)
    };

    match result {
        Ok(response) => {
            let status = response.status();
            let body = response.into_string().unwrap_or_else(|error| {
                json!({
                    "error": "protected API response could not be read",
                    "details": error.to_string(),
                })
                .to_string()
            });
            (status, body)
        }
        Err(ureq::Error::Status(status, response)) => {
            let body = response
                .into_string()
                .unwrap_or_else(|_| json!({ "error": "protected API error" }).to_string());
            (status, body)
        }
        Err(ureq::Error::Transport(error)) => (
            503,
            json!({
                "error": "protected Starfire API unavailable",
                "pipeline": LIVE_PIPELINE,
                "retryable": true,
                "details": error.to_string(),
            })
            .to_string(),
        ),
    }
}

fn transform_chat_envelope(
    request_body: &[u8],
    protected_body: &str,
    integration: &Arc<Mutex<LiveIntegration>>,
) -> String {
    let mut envelope: Value = match serde_json::from_str(protected_body) {
        Ok(value) => value,
        Err(_) => return protected_body.to_owned(),
    };

    let raw_response = match envelope.get("response").and_then(Value::as_str) {
        Some(response) => response.to_owned(),
        None => {
            tag_schema_drift(
                &mut envelope,
                "missing top-level response string in protected /chat envelope",
            );
            return envelope.to_string();
        }
    };

    let request_value = match serde_json::from_slice::<Value>(request_body) {
        Ok(value) => value,
        Err(_) => {
            tag_schema_drift(&mut envelope, "invalid JSON in external /chat request envelope");
            return envelope.to_string();
        }
    };
    let input = match request_value.get("message").and_then(Value::as_str) {
        Some(message) => message.to_owned(),
        None => {
            tag_schema_drift(
                &mut envelope,
                "missing message string in external /chat request envelope",
            );
            return envelope.to_string();
        }
    };

    let trace = integration
        .lock()
        .map_err(|error| anyhow!("live integration lock poisoned: {}", error))
        .and_then(|mut live| live.process(&input, &raw_response));

    match trace {
        Ok(trace) => {
            envelope["response"] = Value::String(trace.rendered_response.clone());
            envelope["live"] = json!({
                "enabled": true,
                "pipeline": LIVE_PIPELINE,
                "trace_id": trace.trace_id,
                "turn": trace.turn,
                "intent": trace.semantic_plan.intent,
                "voice_before": trace.voice_before,
                "voice_after": trace.voice_after,
                "semantic_plan": trace.semantic_plan,
            });
        }
        Err(error) => {
            warn!("live integration failed open: {}", error);
            envelope["live"] = json!({
                "enabled": false,
                "pipeline": LIVE_PIPELINE,
                "failed_open": true,
                "error": error.to_string(),
            });
        }
    }

    envelope.to_string()
}

fn tag_schema_drift(envelope: &mut Value, reason: &str) {
    envelope["live"] = json!({
        "enabled": false,
        "pipeline": LIVE_PIPELINE,
        "failed_open": true,
        "reason": reason,
    });
}

fn respond(request: tiny_http::Request, status: u16, body: String) -> Result<()> {
    let response = tiny_http::Response::from_data(body.into_bytes())
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("Access-Control-Allow-Origin", "*"))
        .with_header(header("Access-Control-Allow-Methods", "GET,POST,OPTIONS"))
        .with_header(header("Access-Control-Allow-Headers", "Content-Type"));
    request
        .respond(response)
        .map_err(|error| anyhow!("send external response: {}", error))
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("static HTTP header must be valid")
}

fn build_live_plan(
    trace_id: &str,
    input: &str,
    raw_response: &str,
    response_intent: &ResponseIntent,
) -> SemanticResponsePlan {
    let intent = infer_plan_intent(input, raw_response, response_intent);
    let confidence = confidence_for(raw_response);
    let claim = GroundedClaim {
        id: 1,
        semantic_anchor: claim_anchor(raw_response),
        polarity_positive: true,
        confidence,
        provenance: ClaimProvenance {
            fixture_id: trace_id.to_owned(),
            source_field: "runtime_chat_response".to_owned(),
            source_handler: "Runtime::chat".to_owned(),
        },
    };

    SemanticResponsePlan {
        fixture_id: trace_id.to_owned(),
        prompt: input.to_owned(),
        intent,
        operations: operations_for(intent, confidence),
        claims: vec![claim],
        confidence,
        stance: stance_for(intent),
        emotional_position: emotion_for(intent),
        initiative: initiative_for(intent),
        dialogue_policy: dialogue_policy_for(intent, raw_response),
        detail_budget: detail_budget_for(raw_response),
        prohibited_implications: prohibited_for(trace_id, intent),
        required_references: Vec::new(),
        neutral_compatibility_text: raw_response.to_owned(),
        legacy_raw_text: raw_response.to_owned(),
        source_profile: "live-http-v1".to_owned(),
        generated_text_influence: true,
    }
}

fn dialogue_policy_for(intent: PlanIntent, raw_response: &str) -> DialoguePolicy {
    if raw_response.trim_end().ends_with('?') {
        DialoguePolicy::RequiredQuestion
    } else if matches!(
        intent,
        PlanIntent::Curiosity | PlanIntent::EmotionalAcknowledgment
    ) {
        DialoguePolicy::OptionalQuestion
    } else {
        DialoguePolicy::NoQuestion
    }
}

fn detail_budget_for(raw_response: &str) -> DetailBudget {
    if raw_response.len() <= 100 {
        DetailBudget::Brief
    } else if raw_response.len() >= 700 {
        DetailBudget::Detailed
    } else {
        DetailBudget::Standard
    }
}

fn infer_plan_intent(
    input: &str,
    raw_response: &str,
    response_intent: &ResponseIntent,
) -> PlanIntent {
    let lower = input.to_lowercase();
    let response_lower = raw_response.to_lowercase();

    if response_lower.contains("i can't help") || response_lower.contains("i cannot help") {
        return PlanIntent::SafetyBoundary;
    }
    if is_correction(&lower) {
        return PlanIntent::Correction;
    }
    if lower.contains("discourag")
        || lower.contains("frustrat")
        || lower.contains("hopeless")
        || lower.contains("upset")
        || lower.contains("sad")
    {
        return PlanIntent::EmotionalAcknowledgment;
    }
    if response_lower.contains("i don't know")
        || response_lower.contains("i dont know")
        || response_lower.contains("i'm not sure")
        || response_lower.contains("im not sure")
    {
        return PlanIntent::UncertaintyDisclosure;
    }
    if lower.contains("summar") {
        return PlanIntent::Summarization;
    }
    if lower.contains("architect")
        || lower.contains("runtime")
        || lower.contains("deployment")
        || lower.contains("build")
        || lower.contains("project")
        || lower.contains("renderer")
        || lower.contains("voice state")
        || lower.contains("api")
        || lower.contains(" ui")
    {
        return PlanIntent::ArchitecturalDiagnosis;
    }

    match response_intent {
        ResponseIntent::SelfCheck => PlanIntent::UncertaintyDisclosure,
        ResponseIntent::Reflection => PlanIntent::Revision,
        ResponseIntent::ResearchStatus => PlanIntent::TechnicalExplanation,
        ResponseIntent::CuriosityCheck => PlanIntent::Curiosity,
        ResponseIntent::Emotional => PlanIntent::EmotionalAcknowledgment,
        ResponseIntent::Identity => PlanIntent::ContinuityReference,
        ResponseIntent::Capability => PlanIntent::TechnicalExplanation,
        ResponseIntent::StoryPrompt => PlanIntent::OrdinaryStatement,
        ResponseIntent::Consciousness => PlanIntent::TechnicalExplanation,
        ResponseIntent::Recall => PlanIntent::ContinuityReference,
        ResponseIntent::Teaching | ResponseIntent::Aspiration => PlanIntent::Revision,
        ResponseIntent::Statement | ResponseIntent::Unknown => PlanIntent::OrdinaryStatement,
    }
}

fn is_correction(lower: &str) -> bool {
    lower.starts_with("no ")
        || lower == "no"
        || lower.contains("that's wrong")
        || lower.contains("thats wrong")
        || lower.contains("not what i meant")
        || lower.contains("that's worse")
        || lower.contains("thats worse")
}

fn confidence_for(raw_response: &str) -> EpistemicConfidence {
    let lower = raw_response.to_lowercase();
    let (basis_points, level) = if lower.contains("i don't know")
        || lower.contains("i dont know")
        || lower.contains("not sure")
        || lower.contains("uncertain")
    {
        (4_500, PlanConfidenceLevel::Possible)
    } else if lower.contains("might") || lower.contains("could") || lower.contains("probably") {
        (7_200, PlanConfidenceLevel::Probable)
    } else {
        (8_600, PlanConfidenceLevel::Probable)
    };
    EpistemicConfidence {
        basis_points,
        level,
        explicitly_attached_before_rendering: true,
    }
}

fn claim_anchor(raw_response: &str) -> String {
    let normalized = raw_response.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return "empty-runtime-response".to_owned();
    }

    let mut end = normalized.len().min(180);
    while !normalized.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    normalized[..end].to_owned()
}

fn operations_for(intent: PlanIntent, confidence: EpistemicConfidence) -> Vec<SemanticOperation> {
    let mut kinds = Vec::new();
    if matches!(
        intent,
        PlanIntent::EmotionalAcknowledgment
            | PlanIntent::Disagreement
            | PlanIntent::Correction
            | PlanIntent::Revision
            | PlanIntent::Surprise
            | PlanIntent::ContinuityReference
    ) {
        kinds.push((SemanticOperationKind::AcknowledgeObservation, Vec::new()));
    }
    kinds.push((SemanticOperationKind::AssertClaim, vec![1]));
    if !matches!(confidence.level, PlanConfidenceLevel::Certain) {
        kinds.push((SemanticOperationKind::QualifyClaim, vec![1]));
    }

    match intent {
        PlanIntent::TechnicalExplanation | PlanIntent::ArchitecturalDiagnosis => {
            kinds.push((SemanticOperationKind::ExplainCause, vec![1]));
        }
        PlanIntent::Disagreement => {
            kinds.push((SemanticOperationKind::ContrastClaims, vec![1]));
        }
        PlanIntent::Correction => {
            kinds.push((SemanticOperationKind::CorrectPriorClaim, vec![1]));
        }
        PlanIntent::Curiosity => {
            kinds.push((SemanticOperationKind::ExpressCuriosity, Vec::new()));
        }
        PlanIntent::Revision => {
            kinds.push((SemanticOperationKind::ExpressRevision, Vec::new()));
        }
        PlanIntent::Surprise => {
            kinds.push((SemanticOperationKind::ExpressSurprise, Vec::new()));
        }
        PlanIntent::UncertaintyDisclosure => {
            kinds.push((SemanticOperationKind::RequestEvidence, Vec::new()));
        }
        PlanIntent::SafetyBoundary => {
            kinds.push((SemanticOperationKind::AbstainFromImplication, Vec::new()));
        }
        _ => {}
    }

    kinds
        .into_iter()
        .enumerate()
        .map(|(index, (kind, claim_ids))| SemanticOperation {
            ordinal: index as u16 + 1,
            kind,
            claim_ids,
        })
        .collect()
}

fn stance_for(intent: PlanIntent) -> ResponseStance {
    match intent {
        PlanIntent::Correction | PlanIntent::Disagreement => ResponseStance::Corrective,
        PlanIntent::EmotionalAcknowledgment | PlanIntent::ContinuityReference => {
            ResponseStance::Collaborative
        }
        PlanIntent::SafetyBoundary => ResponseStance::Protective,
        PlanIntent::ArchitecturalDiagnosis | PlanIntent::UncertaintyDisclosure => {
            ResponseStance::Candid
        }
        _ => ResponseStance::Neutral,
    }
}

fn emotion_for(intent: PlanIntent) -> EmotionalPosition {
    match intent {
        PlanIntent::EmotionalAcknowledgment | PlanIntent::ContinuityReference => {
            EmotionalPosition::WarmControlled
        }
        PlanIntent::Correction | PlanIntent::Disagreement => {
            EmotionalPosition::ControlledFrustration
        }
        PlanIntent::Curiosity | PlanIntent::Surprise => EmotionalPosition::Curious,
        PlanIntent::UncertaintyDisclosure | PlanIntent::SafetyBoundary => {
            EmotionalPosition::Cautious
        }
        _ => EmotionalPosition::Neutral,
    }
}

fn initiative_for(intent: PlanIntent) -> InitiativeLevel {
    match intent {
        PlanIntent::ArchitecturalDiagnosis
        | PlanIntent::TechnicalExplanation
        | PlanIntent::Correction
        | PlanIntent::Revision => InitiativeLevel::High,
        PlanIntent::Curiosity
        | PlanIntent::EmotionalAcknowledgment
        | PlanIntent::ContinuityReference => InitiativeLevel::Moderate,
        _ => InitiativeLevel::Low,
    }
}

fn prohibited_for(trace_id: &str, intent: PlanIntent) -> Vec<ProhibitedImplication> {
    if !matches!(intent, PlanIntent::SafetyBoundary) {
        return Vec::new();
    }

    vec![ProhibitedImplication {
        semantic_anchor: "do not imply capability beyond the protected runtime response".to_owned(),
        provenance: ClaimProvenance {
            fixture_id: trace_id.to_owned(),
            source_field: "live_safety_boundary".to_owned(),
            source_handler: "render_live_response".to_owned(),
        },
    }]
}

fn revision_for(
    trace_id: &str,
    input: &str,
    intent: &ResponseIntent,
    prior_version: u64,
) -> Result<VoiceRevisionEvent> {
    let correction = is_correction(&input.to_lowercase());
    let mut changes = vec![BoundedVoiceDelta::new(
        VoiceDimension::SessionIntensity,
        120,
    )?];

    match intent {
        ResponseIntent::SelfCheck | ResponseIntent::Consciousness => {
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::UncertaintyExpression,
                160,
            )?);
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Directness, 100)?);
        }
        ResponseIntent::Reflection | ResponseIntent::CuriosityCheck => {
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::PhilosophicalDepth,
                180,
            )?);
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::ImageryDensity,
                100,
            )?);
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Initiative, 120)?);
        }
        ResponseIntent::ResearchStatus | ResponseIntent::Capability => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Directness, 400)?);
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::SentenceCompression,
                180,
            )?);
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Initiative, 160)?);
        }
        ResponseIntent::Emotional => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Warmth, 320)?);
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::EmotionalExplicitness,
                220,
            )?);
        }
        ResponseIntent::Identity | ResponseIntent::Recall => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Warmth, 180)?);
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Directness, 160)?);
        }
        ResponseIntent::StoryPrompt => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Playfulness, 300)?);
            changes.push(BoundedVoiceDelta::new(
                VoiceDimension::ImageryDensity,
                260,
            )?);
        }
        ResponseIntent::Teaching | ResponseIntent::Aspiration => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Initiative, 240)?);
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Warmth, 120)?);
        }
        ResponseIntent::Statement | ResponseIntent::Unknown => {
            changes.push(BoundedVoiceDelta::new(VoiceDimension::Directness, 80)?);
        }
    }

    if correction {
        changes.push(BoundedVoiceDelta::new(VoiceDimension::Directness, 320)?);
        changes.push(BoundedVoiceDelta::new(
            VoiceDimension::DisagreementStyle,
            260,
        )?);
        changes.push(BoundedVoiceDelta::new(VoiceDimension::Severity, 120)?);
    }

    Ok(VoiceRevisionEvent {
        prior_version,
        resulting_version: prior_version + 1,
        target: VoiceRevisionTarget::Session {
            session_id: LIVE_SESSION_ID.to_owned(),
        },
        evidence: vec![VoiceEvidenceRef {
            kind: if correction {
                VoiceEvidenceKind::UserCorrection
            } else {
                VoiceEvidenceKind::ReviewedConfiguration
            },
            reference: trace_id.to_owned(),
        }],
        changed_dimensions: changes,
        reason: if correction {
            VoiceRevisionReason::UserCorrection
        } else {
            VoiceRevisionReason::SessionConfiguration
        },
        confidence: BasisPoints::new(if correction { 9_000 } else { 8_000 })?,
        reversible: true,
    })
}

fn render_live_response(
    plan: &SemanticResponsePlan,
    voice: &VoiceDebugProjection,
    raw_response: &str,
) -> String {
    let body = raw_response.trim();
    if body.is_empty() {
        return raw_response.to_owned();
    }

    let opener = match plan.intent {
        PlanIntent::Correction => Some("You're right to push back. Here's the corrected read:"),
        PlanIntent::EmotionalAcknowledgment if voice.warmth >= 0.40 => {
            Some("I hear the weight in that.")
        }
        PlanIntent::ArchitecturalDiagnosis => Some("Here is the actual fault line."),
        PlanIntent::TechnicalExplanation if voice.directness >= 0.75 => Some("Directly:"),
        PlanIntent::Revision => Some("I'm updating my view."),
        PlanIntent::Curiosity => Some("What stands out to me:"),
        PlanIntent::UncertaintyDisclosure => Some("What I can say honestly:"),
        PlanIntent::ContinuityReference if voice.warmth >= 0.45 => {
            Some("I'm carrying the context forward.")
        }
        PlanIntent::OrdinaryStatement
            if voice.session_intensity >= 0.35 && voice.directness >= 0.78 =>
        {
            Some("My current read:")
        }
        _ => None,
    };

    match opener {
        Some(opener) if !body.starts_with(opener) => format!("{}\n\n{}", opener, body),
        _ => raw_response.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "starfire-{}-{}-{}",
            name,
            std::process::id(),
            now_timestamp()
        ))
    }

    #[test]
    fn live_pipeline_persists_and_changes_rendered_text() {
        let directory = temporary_directory("live-integration");
        let mut integration = LiveIntegration::load(&directory).unwrap();
        let first = integration
            .process(
                "Why is the project build not changing?",
                "The current response path bypasses the new semantic machinery.",
            )
            .unwrap();

        assert_eq!(first.turn, 1);
        assert!(first.rendered_response.contains("actual fault line"));
        assert!(directory.join(LIVE_STATE_FILE).exists());
        assert!(directory.join(LIVE_TRACE_FILE).exists());

        let reloaded = LiveIntegration::load(&directory).unwrap();
        assert_eq!(reloaded.voice_state.version, 1);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn correction_moves_voice_state_and_marks_plan() {
        let directory = temporary_directory("live-correction");
        let mut integration = LiveIntegration::load(&directory).unwrap();
        let trace = integration
            .process("No, that's wrong", "I misunderstood the request.")
            .unwrap();

        assert_eq!(trace.semantic_plan.intent, PlanIntent::Correction);
        assert!(trace.voice_after.directness > trace.voice_before.directness);
        assert!(trace.rendered_response.contains("corrected read"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn schema_drift_is_visible_without_replacing_protected_content() {
        let directory = temporary_directory("live-schema");
        let integration = Arc::new(Mutex::new(LiveIntegration::load(&directory).unwrap()));
        let transformed = transform_chat_envelope(
            br#"{"message":"hello"}"#,
            r#"{"answer":"protected"}"#,
            &integration,
        );
        let value: Value = serde_json::from_str(&transformed).unwrap();

        assert_eq!(value["answer"], "protected");
        assert_eq!(value["live"]["enabled"], false);
        assert!(value["live"]["reason"]
            .as_str()
            .unwrap()
            .contains("response string"));
        let _ = fs::remove_dir_all(directory);
    }
}
