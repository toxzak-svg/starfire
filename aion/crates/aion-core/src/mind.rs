//! Mind — the durable thinking process.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use async_trait::async_trait;
use crate::AionResult;

/// Unique Mind identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MindId(Uuid);

impl MindId {
    pub fn new() -> Self { MindId(Uuid::new_v4()) }
    pub fn from_uuid(u: Uuid) -> Self { MindId(u) }
    pub fn as_uuid(&self) -> Uuid { self.0 }
    pub fn as_str(&self) -> String { self.0.to_string() }
}
impl Default for MindId { fn default() -> Self { Self::new() } }
impl std::fmt::Display for MindId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }

/// Mind lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MindStatus {
    Created, Running, Checkpointed, Terminated, Failed, Unknown,
}
impl std::fmt::Display for MindStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MindStatus::Created => write!(f, "created"),
            MindStatus::Running => write!(f, "running"),
            MindStatus::Checkpointed => write!(f, "checkpointed"),
            MindStatus::Terminated => write!(f, "terminated"),
            MindStatus::Failed => write!(f, "failed"),
            MindStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Control flow after handling an impulse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    CheckpointAndWait,
    Terminate,
}

/// Configuration for starting a Mind.
#[derive(Debug, Clone)]
pub struct MindConfig {
    pub name: Option<String>,
    pub channel: Option<String>,
    pub config: Option<serde_json::Value>,
    pub checkpoint_every: u32,
}
impl Default for MindConfig {
    fn default() -> Self { Self { name: None, channel: None, config: None, checkpoint_every: 100 } }
}
impl MindConfig {
    pub fn new() -> Self { Self::default() }
    pub fn name<S: Into<String>>(mut self, n: S) -> Self { self.name = Some(n.into()); self }
    pub fn channel<S: Into<String>>(mut self, c: S) -> Self { self.channel = Some(c.into()); self }
}

/// Metadata about a registered Mind kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindKind { pub name: String, pub description: Option<String>, pub version: u32 }

/// Information about a Mind from the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindInfo {
    pub id: MindId,
    pub kind: String,
    pub name: Option<String>,
    pub status: MindStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub ended_at: Option<i64>,
}

/// The core trait for defining a Mind's behavior.
#[async_trait]
pub trait MindLogic: Send + Sync + 'static {
    const KIND: &'static str;
    const DESCRIPTION: &'static str = "";

    fn new() -> Self;

    async fn start(&mut self) -> AionResult<()> { Ok(()) }

    async fn resume(&mut self, _checkpoint: &serde_json::Value) -> AionResult<()> { Ok(()) }

    async fn handle_impulse(&mut self, impulse: &crate::Impulse) -> AionResult<ControlFlow>;

    fn checkpoint(&self) -> serde_json::Value;

    fn checkpoint_every(&self) -> u32 { 100 }

    fn name(&self) -> Option<String> { None }

    fn status(&self) -> MindStatus { MindStatus::Running }
}

/// A simple string-based Mind for testing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StringMind { pub state: String, pub count: u32 }

#[async_trait]
impl MindLogic for StringMind {
    const KIND: &'static str = "string_mind";
    const DESCRIPTION: &'static str = "A simple string-state mind for testing";

    fn new() -> Self { Self::default() }

    async fn handle_impulse(&mut self, impulse: &crate::Impulse) -> AionResult<ControlFlow> {
        self.count += 1;
        if let crate::Impulse::Message(t) = impulse {
            self.state = format!("{} + {}", self.state, t);
        }
        Ok(ControlFlow::Continue)
    }

    fn checkpoint(&self) -> serde_json::Value {
        serde_json::json!({ "state": self.state, "count": self.count })
    }
}
