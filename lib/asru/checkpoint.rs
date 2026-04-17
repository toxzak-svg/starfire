//! ASRU Checkpoint
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use super::regime_classifier::ReasoningRegime;
use super::regime_memory::RegimeStats;
use super::{ASRUEngine, ColumnRole, EvalMetrics, InterfaceShape, PlasticityMask, RoutingConfig};

const CURRENT_VERSION: u32 = 1;

/// Serialization-friendly representation of regime transitions.
/// JSON object keys must be strings, so we use an array of objects
/// instead of a HashMap with tuple keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionEntry {
    pub from: String,
    pub to: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASRUCheckpoint {
    pub version: u32,
    pub timestamp: i64,
    pub session_id: String,
    pub current_regime: ReasoningRegime,
    pub current_dwell: u64,
    pub regime_stats: HashMap<ReasoningRegime, RegimeStats>,
    pub transitions: Vec<TransitionEntry>,
    pub total_transitions: u64,
    pub history: Vec<ReasoningRegime>,
    pub routing: RoutingConfig,
    pub plasticity: PlasticityMask,
    pub evaluation: EvalMetrics,
    pub interface: InterfaceShape,
    pub columns: Vec<ColumnData>,
    pub viscosity: f32,
    pub fragility_threshold: f32,
    pub global_fragility: f32,
    pub total_turns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnData {
    pub id: u32,
    pub role: String,
    pub plasticity: f32,
    pub stress: f32,
}

fn regime_to_string(r: ReasoningRegime) -> String { format!("{:?}", r) }
fn regime_from_string(s: &str) -> ReasoningRegime {
    match s {
        "SymbolicManipulation" => ReasoningRegime::SymbolicManipulation,
        "EmotionalResonance" => ReasoningRegime::EmotionalResonance,
        "CausalReasoning" => ReasoningRegime::CausalReasoning,
        "AssociativeRecall" => ReasoningRegime::AssociativeRecall,
        "Exploratory" => ReasoningRegime::Exploratory,
        "SteadyState" => ReasoningRegime::SteadyState,
        _ => ReasoningRegime::SteadyState,
    }
}
fn column_role_from_string(s: &str) -> ColumnRole {
    match s {
        "Calculator" => ColumnRole::Calculator,
        "SafetyMonitor" => ColumnRole::SafetyMonitor,
        "Explorer" => ColumnRole::Explorer,
        "Compressor" => ColumnRole::Compressor,
        "Sentinel" => ColumnRole::Sentinel,
        "Stem" => ColumnRole::Stem,
        _ => ColumnRole::Stem,
    }
}

impl ASRUCheckpoint {
    pub fn from_engine(engine: &ASRUEngine, session_id: &str, total_turns: u64) -> Self {
        let tracker = engine.tracker();
        let m_t = engine.meta_state();
        let mem = tracker.memory();
        let mut regime_stats: HashMap<ReasoningRegime, RegimeStats> = HashMap::new();
        for (regime, stats) in mem.all_stats() {
            regime_stats.insert(*regime, (*stats).clone());
        }
        let transitions: Vec<TransitionEntry> = mem.transitions()
            .iter()
            .map(|((from, to), count)| TransitionEntry {
                from: regime_to_string(*from),
                to: regime_to_string(*to),
                count: *count,
            })
            .collect();
        let columns: Vec<ColumnData> = engine.columns().iter().map(|col| ColumnData {
            id: col.id, role: format!("{:?}", col.role), plasticity: col.plasticity, stress: col.stress,
        }).collect();
        Self {
            version: CURRENT_VERSION,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0),
            session_id: session_id.to_string(),
            current_regime: tracker.current_regime(),
            current_dwell: tracker.current_dwell(),
            regime_stats,
            transitions,
            total_transitions: mem.total_transitions(),
            history: mem.history().to_vec(),
            routing: m_t.routing.clone(),
            plasticity: m_t.plasticity.clone(),
            evaluation: m_t.evaluation.clone(),
            interface: m_t.interface.clone(),
            columns,
            viscosity: engine.viscosity(),
            fragility_threshold: engine.fragility_threshold(),
            global_fragility: mem.global_fragility() as f32,
            total_turns,
        }
    }
}

impl ASRUEngine {
    pub fn save_checkpoint(&self, path: &Path, session_id: &str, total_turns: u64) -> std::io::Result<()> {
        let checkpoint = ASRUCheckpoint::from_engine(self, session_id, total_turns);
        let json = serde_json::to_string_pretty(&checkpoint).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if path.exists() {
            let backup = path.with_extension("json.bak");
            std::fs::copy(path, &backup).ok();
            let backup2 = path.with_extension("json.bak2");
            if backup2.exists() { std::fs::remove_file(&backup2).ok(); }
            std::fs::copy(&backup, &backup2).ok();
        }
        std::fs::write(path, json)?;
        Ok(())
    }
    pub fn load_checkpoint(&mut self, path: &Path) -> std::io::Result<bool> {
        if !path.exists() { return Ok(false); }
        let json = std::fs::read_to_string(path)?;
        let cp: ASRUCheckpoint = serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if cp.version != CURRENT_VERSION {
            let backup = path.with_extension("json.old_version");
            std::fs::copy(path, &backup).ok();
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "version mismatch"));
        }
        self.restore_from_checkpoint(&cp);
        Ok(true)
    }
    fn restore_from_checkpoint(&mut self, cp: &ASRUCheckpoint) {
        let mut transitions: HashMap<(ReasoningRegime, ReasoningRegime), u64> = HashMap::new();
        for entry in &cp.transitions {
            transitions.insert((regime_from_string(&entry.from), regime_from_string(&entry.to)), entry.count);
        }
        self.tracker_mut().restore_state(cp.current_regime, cp.current_dwell, cp.regime_stats.clone(), transitions, cp.total_transitions, cp.history.clone(), cp.global_fragility as f64);
        self.set_meta_state(cp.routing.clone(), cp.plasticity.clone(), cp.evaluation.clone(), cp.interface.clone());
        for col_data in &cp.columns {
            self.set_column(col_data.id as usize, column_role_from_string(&col_data.role), col_data.plasticity, col_data.stress);
        }
        self.set_viscosity(cp.viscosity);
        self.set_fragility_threshold(cp.fragility_threshold);
    }
    pub fn default_checkpoint_path(data_dir: &Path) -> std::path::PathBuf { data_dir.join("asru_checkpoint.json") }
}

#[derive(Debug, Clone)]
pub struct AutoCheckpoint { interval_secs: u64, last_checkpoint: i64 }
impl AutoCheckpoint {
    pub fn new(interval_secs: u64) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        Self { interval_secs, last_checkpoint: now.saturating_sub(interval_secs as i64) }
    }
    pub fn should_checkpoint(&self) -> bool {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        now - self.last_checkpoint >= self.interval_secs as i64
    }
    pub fn mark_checkpoint(&mut self) {
        self.last_checkpoint = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    }
    pub fn interval_secs(&self) -> u64 { self.interval_secs }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_checkpoint_roundtrip() {
        let mut engine = ASRUEngine::new(4);
        for i in 0..10 {
            let text = if i % 3 == 0 { "I feel really frustrated today" } else { "The meeting is at 3pm" };
            engine.step(text, &[0.1, 0.5, 0.2, 0.1, 0.2, 0.5, 0.2, 0.4]);
        }
        let path = std::env::temp_dir().join("asru_test_checkpoint.json");
        engine.save_checkpoint(&path, "test-session", 10).unwrap();
        let mut engine2 = ASRUEngine::new(4);
        let loaded = engine2.load_checkpoint(&path).unwrap();
        assert!(loaded);
        assert_eq!(engine2.current_regime(), engine.current_regime());
        std::fs::remove_file(path).ok();
    }
    #[test] fn test_auto_checkpoint_timer() {
        let mut timer = AutoCheckpoint::new(300);
        assert!(timer.should_checkpoint());
        timer.mark_checkpoint();
        assert!(!timer.should_checkpoint());
    }
}
