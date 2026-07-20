//! ΩG2-S0 real-trace shadow integration.
//!
//! This module passively observes typed `PredictionCenter` batches and their
//! independently supplied outcomes. It stores no raw user text, prediction
//! descriptions, reasoning strings, topics, memories, or model state. Its
//! outputs are inert diagnostics only: it cannot construct certificates, admit
//! grammar productions, mutate state keys, influence predictions, persist data,
//! select tools, or perform actions.

use crate::commitment_state::Atom;
use crate::grammar_extension::{BoundExtensionProgram, ExtensionKind, GrammarRoot};
use crate::prediction::{Evidence, Prediction, PredictionOutcome};
use crate::representation_genesis::{
    derive_vocabulary, detect_alias_defects, enumerate_programs, AliasDefect, GenesisBudget,
    RawHistory, RefinementProblem, StateLanguage, WitnessedHistory,
};
use crate::recursive_grammar_composition::BoundComposedProgram;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

const DEFAULT_RANK_WIDTH: usize = 5;
const DEFAULT_MIN_WITNESSES_FOR_AUDIT: usize = 16;
const DEFAULT_MAX_PENDING_TRACES: usize = 512;
const DEFAULT_MAX_WITNESSES_PER_ROOT: usize = 256;
const DEFAULT_MAX_ROOTS: usize = 32;
const DEFAULT_MAX_VOCABULARY: usize = 8;
const DEFAULT_MAX_SETTLED_IDS: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmegaG2ShadowConfig {
    pub rank_width: usize,
    pub min_witnesses_for_audit: usize,
    pub max_pending_traces: usize,
    pub max_witnesses_per_root: usize,
    pub max_roots: usize,
    pub max_vocabulary: usize,
}

impl Default for OmegaG2ShadowConfig {
    fn default() -> Self {
        Self {
            rank_width: DEFAULT_RANK_WIDTH,
            min_witnesses_for_audit: DEFAULT_MIN_WITNESSES_FOR_AUDIT,
            max_pending_traces: DEFAULT_MAX_PENDING_TRACES,
            max_witnesses_per_root: DEFAULT_MAX_WITNESSES_PER_ROOT,
            max_roots: DEFAULT_MAX_ROOTS,
            max_vocabulary: DEFAULT_MAX_VOCABULARY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowAuditClassification {
    NoDefects,
    NoCompositionalGain,
    PartialCompositionalGain,
    CompleteCompositionalCandidate,
    SkippedVocabularyLimit,
}

impl ShadowAuditClassification {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoDefects => "no_defects",
            Self::NoCompositionalGain => "no_compositional_gain",
            Self::PartialCompositionalGain => "partial_compositional_gain",
            Self::CompleteCompositionalCandidate => "complete_compositional_candidate",
            Self::SkippedVocabularyLimit => "skipped_vocabulary_limit",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowAuditBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub base_candidate_programs: usize,
    pub base_program_history_evaluations: usize,
    pub single_m1_candidate_programs: usize,
    pub single_m1_program_history_evaluations: usize,
    pub c1_candidate_programs: usize,
    pub c1_program_history_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowCompositionAudit {
    pub root_id: u64,
    pub witness_count: usize,
    pub vocabulary_size: usize,
    pub detected_defects: usize,
    pub base_candidate_programs: usize,
    pub base_unique_partitions: usize,
    pub base_best_repaired_defects: usize,
    pub single_m1_candidate_programs: usize,
    pub single_m1_unique_partitions: usize,
    pub single_m1_best_repaired_defects: usize,
    pub c1_candidate_programs: usize,
    pub c1_unique_partitions: usize,
    pub c1_best_repaired_defects: usize,
    pub c1_complete_repair_count: usize,
    pub best_c1_program: Option<String>,
    pub classification: ShadowAuditClassification,
    pub budget: ShadowAuditBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowAuthorityBoundary {
    pub runtime_chat_response_influence: bool,
    pub prediction_generation_influence: bool,
    pub prediction_ranking_influence: bool,
    pub prediction_outcome_influence: bool,
    pub routing_authority: bool,
    pub persistence_authority: bool,
    pub belief_or_ontology_promotion: bool,
    pub grammar_registry_admission: bool,
    pub state_key_mutation: bool,
    pub tool_or_capability_selection: bool,
    pub external_side_effects: bool,
    pub autonomous_action: bool,
    pub automatic_source_modification: bool,
}

impl Default for ShadowAuthorityBoundary {
    fn default() -> Self {
        Self {
            runtime_chat_response_influence: false,
            prediction_generation_influence: false,
            prediction_ranking_influence: false,
            prediction_outcome_influence: false,
            routing_authority: false,
            persistence_authority: false,
            belief_or_ontology_promotion: false,
            grammar_registry_admission: false,
            state_key_mutation: false,
            tool_or_capability_selection: false,
            external_side_effects: false,
            autonomous_action: false,
            automatic_source_modification: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmegaG2ShadowSnapshot {
    pub config: OmegaG2ShadowConfig,
    pub observed_batches: usize,
    pub pending_traces: usize,
    pub settled_witnesses: usize,
    pub roots: usize,
    pub duplicate_evidence: usize,
    pub dropped_pending_traces: usize,
    pub evicted_witnesses: usize,
    pub audits: Vec<ShadowCompositionAudit>,
    pub authority: ShadowAuthorityBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowEvidenceResolution {
    Settled,
    Duplicate,
    MissingPendingTrace,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OmegaG2ShadowError {
    #[error("rank width must be at least three")]
    RankWidthTooSmall,
    #[error("minimum witnesses for audit must be at least two")]
    AuditThresholdTooSmall,
    #[error("all ΩG2-S0 capacity limits must be nonzero")]
    ZeroCapacity,
    #[error("maximum vocabulary must be at least three")]
    VocabularyLimitTooSmall,
    #[error("failed to construct typed shadow atom: {0}")]
    InvalidAtom(String),
    #[error("shadow audit failed: {0}")]
    Audit(String),
}

#[derive(Debug, Clone)]
struct PendingTrace {
    history_id: u64,
    root_id: u64,
    events: Vec<Atom>,
    intervention: Atom,
}

#[derive(Debug, Clone)]
struct RootBuffer {
    intervention: Atom,
    discovery: VecDeque<WitnessedHistory>,
    latest_audit: Option<ShadowCompositionAudit>,
}

#[derive(Debug)]
pub struct OmegaG2ShadowObserver {
    config: OmegaG2ShadowConfig,
    observed_batches: usize,
    next_evidence_id: u64,
    pending: BTreeMap<u64, PendingTrace>,
    pending_order: VecDeque<u64>,
    roots: BTreeMap<u64, RootBuffer>,
    root_order: VecDeque<u64>,
    settled_prediction_ids: BTreeSet<u64>,
    settled_order: VecDeque<u64>,
    duplicate_evidence: usize,
    dropped_pending_traces: usize,
    evicted_witnesses: usize,
}

impl Default for OmegaG2ShadowObserver {
    fn default() -> Self {
        Self::new(OmegaG2ShadowConfig::default())
            .expect("the frozen ΩG2-S0 default configuration is valid")
    }
}

impl OmegaG2ShadowObserver {
    pub fn new(config: OmegaG2ShadowConfig) -> Result<Self, OmegaG2ShadowError> {
        if config.rank_width < 3 {
            return Err(OmegaG2ShadowError::RankWidthTooSmall);
        }
        if config.min_witnesses_for_audit < 2 {
            return Err(OmegaG2ShadowError::AuditThresholdTooSmall);
        }
        if config.max_pending_traces == 0
            || config.max_witnesses_per_root == 0
            || config.max_roots == 0
        {
            return Err(OmegaG2ShadowError::ZeroCapacity);
        }
        if config.max_vocabulary < 3 {
            return Err(OmegaG2ShadowError::VocabularyLimitTooSmall);
        }
        Ok(Self {
            config,
            observed_batches: 0,
            next_evidence_id: 1,
            pending: BTreeMap::new(),
            pending_order: VecDeque::new(),
            roots: BTreeMap::new(),
            root_order: VecDeque::new(),
            settled_prediction_ids: BTreeSet::new(),
            settled_order: VecDeque::new(),
            duplicate_evidence: 0,
            dropped_pending_traces: 0,
            evicted_witnesses: 0,
        })
    }

    #[must_use]
    pub fn config(&self) -> &OmegaG2ShadowConfig {
        &self.config
    }

    /// Copy one already-ranked prediction batch into the inert pending ledger.
    ///
    /// Only engine/kind category tokens and prediction identifiers are retained.
    /// No raw topic, text, description, reasoning, memory, or model state enters
    /// the observer.
    pub fn observe_prediction_batch(&mut self, predictions: &[Prediction]) -> usize {
        self.observed_batches = self.observed_batches.saturating_add(1);
        if predictions.is_empty() {
            return 0;
        }

        let events = match ranking_events(predictions, self.config.rank_width) {
            Ok(events) => events,
            Err(_) => return 0,
        };

        let mut accepted = 0usize;
        for prediction in predictions {
            let intervention_label = format!("selected:{}:{}", prediction.engine, prediction.kind);
            let intervention = match Atom::new(intervention_label.clone()) {
                Ok(atom) => atom,
                Err(_) => continue,
            };
            let root_id = fnv1a64(intervention_label.as_bytes());
            let prediction_id = prediction.id.0;
            if self.pending.contains_key(&prediction_id)
                || self.settled_prediction_ids.contains(&prediction_id)
            {
                continue;
            }
            self.pending.insert(
                prediction_id,
                PendingTrace {
                    history_id: prediction_id,
                    root_id,
                    events: events.clone(),
                    intervention,
                },
            );
            self.pending_order.push_back(prediction_id);
            accepted = accepted.saturating_add(1);
        }
        self.enforce_pending_bound();
        accepted
    }

    /// Settle one pending typed trace from the same evidence already consumed by
    /// the live prediction center. The observer cannot change that evidence or
    /// the prediction status transition.
    pub fn resolve_prediction_evidence(
        &mut self,
        prediction: &Prediction,
        evidence: &Evidence,
    ) -> ShadowEvidenceResolution {
        let prediction_id = evidence.prediction_id.0;
        if self.settled_prediction_ids.contains(&prediction_id) {
            self.duplicate_evidence = self.duplicate_evidence.saturating_add(1);
            return ShadowEvidenceResolution::Duplicate;
        }
        let Some(pending) = self.pending.remove(&prediction_id) else {
            return ShadowEvidenceResolution::MissingPendingTrace;
        };
        self.pending_order.retain(|candidate| *candidate != prediction_id);

        let outcome = match outcome_atom(evidence.outcome) {
            Ok(outcome) => outcome,
            Err(_) => return ShadowEvidenceResolution::MissingPendingTrace,
        };
        let witness = WitnessedHistory {
            evidence_id: self.next_evidence_id,
            history: RawHistory {
                history_id: pending.history_id,
                events: pending.events,
            },
            intervention: pending.intervention.clone(),
            outcome,
        };
        self.next_evidence_id = self.next_evidence_id.saturating_add(1);

        self.ensure_root_capacity(pending.root_id, pending.intervention);
        if let Some(root) = self.roots.get_mut(&pending.root_id) {
            if root.discovery.len() >= self.config.max_witnesses_per_root {
                root.discovery.pop_front();
                self.evicted_witnesses = self.evicted_witnesses.saturating_add(1);
            }
            root.discovery.push_back(witness);
        }

        self.settled_prediction_ids.insert(prediction_id);
        self.settled_order.push_back(prediction_id);
        self.enforce_settled_id_bound();
        self.refresh_root_audit(pending.root_id);

        // Keep the parameter semantically tied to the live prediction resolved
        // by the caller without retaining any free-form fields from it.
        debug_assert_eq!(prediction.id, evidence.prediction_id);
        ShadowEvidenceResolution::Settled
    }

    #[must_use]
    pub fn snapshot(&self) -> OmegaG2ShadowSnapshot {
        let settled_witnesses = self
            .roots
            .values()
            .map(|root| root.discovery.len())
            .sum();
        let audits = self
            .roots
            .values()
            .filter_map(|root| root.latest_audit.clone())
            .collect();
        OmegaG2ShadowSnapshot {
            config: self.config.clone(),
            observed_batches: self.observed_batches,
            pending_traces: self.pending.len(),
            settled_witnesses,
            roots: self.roots.len(),
            duplicate_evidence: self.duplicate_evidence,
            dropped_pending_traces: self.dropped_pending_traces,
            evicted_witnesses: self.evicted_witnesses,
            audits,
            authority: ShadowAuthorityBoundary::default(),
        }
    }

    fn enforce_pending_bound(&mut self) {
        while self.pending.len() > self.config.max_pending_traces {
            let Some(oldest) = self.pending_order.pop_front() else {
                break;
            };
            if self.pending.remove(&oldest).is_some() {
                self.dropped_pending_traces = self.dropped_pending_traces.saturating_add(1);
            }
        }
    }

    fn ensure_root_capacity(&mut self, root_id: u64, intervention: Atom) {
        if self.roots.contains_key(&root_id) {
            return;
        }
        while self.roots.len() >= self.config.max_roots {
            let Some(oldest) = self.root_order.pop_front() else {
                break;
            };
            if let Some(removed) = self.roots.remove(&oldest) {
                self.evicted_witnesses = self
                    .evicted_witnesses
                    .saturating_add(removed.discovery.len());
            }
        }
        self.roots.insert(
            root_id,
            RootBuffer {
                intervention,
                discovery: VecDeque::new(),
                latest_audit: None,
            },
        );
        self.root_order.push_back(root_id);
    }

    fn enforce_settled_id_bound(&mut self) {
        while self.settled_order.len() > DEFAULT_MAX_SETTLED_IDS {
            let Some(oldest) = self.settled_order.pop_front() else {
                break;
            };
            self.settled_prediction_ids.remove(&oldest);
        }
    }

    fn refresh_root_audit(&mut self, root_id: u64) {
        let Some(root) = self.roots.get(&root_id) else {
            return;
        };
        if root.discovery.len() < self.config.min_witnesses_for_audit {
            return;
        }
        let grammar_root = GrammarRoot {
            root_id,
            discovery: root.discovery.iter().cloned().collect(),
        };
        let audit = audit_shadow_root(&grammar_root, self.config.max_vocabulary).ok();
        if let Some(root) = self.roots.get_mut(&root_id) {
            root.latest_audit = audit;
        }
    }
}

fn ranking_events(
    predictions: &[Prediction],
    rank_width: usize,
) -> Result<Vec<Atom>, OmegaG2ShadowError> {
    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut events = Vec::with_capacity(rank_width);
    for prediction in predictions.iter().take(rank_width) {
        let category = format!("prediction:{}:{}", prediction.engine, prediction.kind);
        let occurrence = occurrences.entry(category.clone()).or_insert(0);
        *occurrence = occurrence.saturating_add(1);
        events.push(
            Atom::new(format!("{}:occurrence:{}", category, occurrence))
                .map_err(|error| OmegaG2ShadowError::InvalidAtom(error.to_string()))?,
        );
    }
    while events.len() < rank_width {
        events.push(
            Atom::new(format!("prediction:padding:{}", events.len()))
                .map_err(|error| OmegaG2ShadowError::InvalidAtom(error.to_string()))?,
        );
    }
    Ok(events)
}

fn outcome_atom(outcome: PredictionOutcome) -> Result<Atom, OmegaG2ShadowError> {
    let label = match outcome {
        PredictionOutcome::Confirmed => "confirmed",
        PredictionOutcome::Refuted => "refuted",
        PredictionOutcome::Surprised => "surprised",
        PredictionOutcome::Uncertain => "uncertain",
    };
    Atom::new(label).map_err(|error| OmegaG2ShadowError::InvalidAtom(error.to_string()))
}

#[derive(Debug, Clone)]
struct PartitionScore {
    repaired: usize,
    support_min: usize,
    syntax: String,
}

fn audit_shadow_root(
    root: &GrammarRoot,
    max_vocabulary: usize,
) -> Result<ShadowCompositionAudit, OmegaG2ShadowError> {
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)
        .map_err(|error| OmegaG2ShadowError::Audit(error.to_string()))?;
    if vocabulary.len() > max_vocabulary {
        return Ok(ShadowCompositionAudit {
            root_id: root.root_id,
            witness_count: root.discovery.len(),
            vocabulary_size: vocabulary.len(),
            detected_defects: 0,
            base_candidate_programs: 0,
            base_unique_partitions: 0,
            base_best_repaired_defects: 0,
            single_m1_candidate_programs: 0,
            single_m1_unique_partitions: 0,
            single_m1_best_repaired_defects: 0,
            c1_candidate_programs: 0,
            c1_unique_partitions: 0,
            c1_best_repaired_defects: 0,
            c1_complete_repair_count: 0,
            best_c1_program: None,
            classification: ShadowAuditClassification::SkippedVocabularyLimit,
            budget: ShadowAuditBudget {
                vocabulary_history_scans: genesis.vocabulary_history_scans,
                history_pair_evaluations: genesis.history_pair_evaluations,
                ..ShadowAuditBudget::default()
            },
        });
    }

    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)
        .map_err(|error| OmegaG2ShadowError::Audit(error.to_string()))?;
    let evidence_index = evidence_index(&root.discovery);
    let mut budget = ShadowAuditBudget {
        vocabulary_history_scans: genesis.vocabulary_history_scans,
        history_pair_evaluations: genesis.history_pair_evaluations,
        ..ShadowAuditBudget::default()
    };

    let base_programs = enumerate_programs(&vocabulary);
    budget.base_candidate_programs = base_programs.len();
    let mut base_partitions = BTreeMap::<Vec<bool>, String>::new();
    for program in base_programs {
        let bits = root
            .discovery
            .iter()
            .map(|episode| {
                budget.base_program_history_evaluations =
                    budget.base_program_history_evaluations.saturating_add(1);
                program.execute(&episode.history)
            })
            .collect::<Vec<_>>();
        insert_canonical_partition(&mut base_partitions, bits, program.canonical_string());
    }
    let base_scores = score_partitions(base_partitions, &defects, &evidence_index);
    let base_best = base_scores.first().map_or(0, |score| score.repaired);

    let mut m1_partitions = BTreeMap::<Vec<bool>, String>::new();
    for kind in ExtensionKind::all() {
        for left in &vocabulary {
            for right in &vocabulary {
                if left == right {
                    continue;
                }
                let program = BoundExtensionProgram {
                    kind,
                    left: left.clone(),
                    right: right.clone(),
                };
                budget.single_m1_candidate_programs =
                    budget.single_m1_candidate_programs.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        budget.single_m1_program_history_evaluations = budget
                            .single_m1_program_history_evaluations
                            .saturating_add(1);
                        program.execute(&episode.history)
                    })
                    .collect::<Vec<_>>();
                insert_canonical_partition(&mut m1_partitions, bits, program.canonical_string());
            }
        }
    }
    let m1_scores = score_partitions(m1_partitions, &defects, &evidence_index);
    let m1_best = m1_scores.first().map_or(0, |score| score.repaired);

    let mut c1_partitions = BTreeMap::<Vec<bool>, String>::new();
    for first in &vocabulary {
        for middle in &vocabulary {
            for last in &vocabulary {
                if first == middle || first == last || middle == last {
                    continue;
                }
                let program = BoundComposedProgram::new(
                    first.clone(),
                    middle.clone(),
                    last.clone(),
                )
                .map_err(|error| OmegaG2ShadowError::Audit(error.to_string()))?;
                budget.c1_candidate_programs = budget.c1_candidate_programs.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        budget.c1_program_history_evaluations =
                            budget.c1_program_history_evaluations.saturating_add(1);
                        program.execute(&episode.history)
                    })
                    .collect::<Vec<_>>();
                insert_canonical_partition(&mut c1_partitions, bits, program.canonical_string());
            }
        }
    }
    let c1_scores = score_partitions(c1_partitions, &defects, &evidence_index);
    let c1_best = c1_scores.first().map_or(0, |score| score.repaired);
    let c1_complete = c1_scores
        .iter()
        .filter(|score| score.repaired == defects.len() && !defects.is_empty())
        .count();
    let classification = if defects.is_empty() {
        ShadowAuditClassification::NoDefects
    } else if c1_best == defects.len() && c1_best > m1_best {
        ShadowAuditClassification::CompleteCompositionalCandidate
    } else if c1_best > m1_best {
        ShadowAuditClassification::PartialCompositionalGain
    } else {
        ShadowAuditClassification::NoCompositionalGain
    };

    Ok(ShadowCompositionAudit {
        root_id: root.root_id,
        witness_count: root.discovery.len(),
        vocabulary_size: vocabulary.len(),
        detected_defects: defects.len(),
        base_candidate_programs: budget.base_candidate_programs,
        base_unique_partitions: base_scores.len(),
        base_best_repaired_defects: base_best,
        single_m1_candidate_programs: budget.single_m1_candidate_programs,
        single_m1_unique_partitions: m1_scores.len(),
        single_m1_best_repaired_defects: m1_best,
        c1_candidate_programs: budget.c1_candidate_programs,
        c1_unique_partitions: c1_scores.len(),
        c1_best_repaired_defects: c1_best,
        c1_complete_repair_count: c1_complete,
        best_c1_program: c1_scores.first().map(|score| score.syntax.clone()),
        classification,
        budget,
    })
}

fn insert_canonical_partition(
    partitions: &mut BTreeMap<Vec<bool>, String>,
    bits: Vec<bool>,
    syntax: String,
) {
    let canonical = canonical_partition(bits);
    match partitions.get_mut(&canonical) {
        Some(existing) if syntax < *existing => *existing = syntax,
        Some(_) => {}
        None => {
            partitions.insert(canonical, syntax);
        }
    }
}

fn score_partitions(
    partitions: BTreeMap<Vec<bool>, String>,
    defects: &[AliasDefect],
    evidence_index: &BTreeMap<u64, usize>,
) -> Vec<PartitionScore> {
    let mut scores = partitions
        .into_iter()
        .map(|(bits, syntax)| {
            let repaired = defects
                .iter()
                .filter(|defect| {
                    let left = evidence_index[&defect.left_evidence_id];
                    let right = evidence_index[&defect.right_evidence_id];
                    bits[left] != bits[right]
                })
                .count();
            let true_count = bits.iter().filter(|bit| **bit).count();
            PartitionScore {
                repaired,
                support_min: true_count.min(bits.len().saturating_sub(true_count)),
                syntax,
            }
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| {
        right
            .repaired
            .cmp(&left.repaired)
            .then_with(|| right.support_min.cmp(&left.support_min))
            .then_with(|| left.syntax.cmp(&right.syntax))
    });
    scores
}

fn evidence_index(discovery: &[WitnessedHistory]) -> BTreeMap<u64, usize> {
    discovery
        .iter()
        .enumerate()
        .map(|(index, episode)| (episode.evidence_id, index))
        .collect()
}

fn canonical_partition(bits: Vec<bool>) -> Vec<bool> {
    let inverted = bits.iter().map(|bit| !*bit).collect::<Vec<_>>();
    if inverted < bits {
        inverted
    } else {
        bits
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction::{
        PredictedCore, PredictionEngine, PredictionId, PredictionKind, PredictionStatus,
    };

    fn prediction(
        id: u64,
        engine: PredictionEngine,
        kind: PredictionKind,
        confidence: f64,
    ) -> Prediction {
        Prediction {
            id: PredictionId(id),
            engine,
            kind,
            core: PredictedCore::Conclusion {
                topic: "redacted".to_string(),
                predicate: "redacted".to_string(),
                confidence,
            },
            description: "must not enter shadow trace".to_string(),
            confidence,
            horizon: 1,
            reasoning: vec!["must not enter shadow trace".to_string()],
            generated_at: 0,
            expires_at: None,
            status: PredictionStatus::Pending,
        }
    }

    #[test]
    fn captures_only_typed_ranking_and_settles_once() {
        let mut observer = OmegaG2ShadowObserver::default();
        let predictions = vec![
            prediction(
                1,
                PredictionEngine::QuestionGravity,
                PredictionKind::Question,
                0.9,
            ),
            prediction(
                2,
                PredictionEngine::BeliefRevision,
                PredictionKind::Conclusion,
                0.8,
            ),
            prediction(
                3,
                PredictionEngine::Basin,
                PredictionKind::NecessaryTruth,
                0.7,
            ),
        ];
        assert_eq!(observer.observe_prediction_batch(&predictions), 3);
        let evidence = Evidence {
            outcome: PredictionOutcome::Confirmed,
            prediction_id: PredictionId(1),
        };
        assert_eq!(
            observer.resolve_prediction_evidence(&predictions[0], &evidence),
            ShadowEvidenceResolution::Settled
        );
        assert_eq!(
            observer.resolve_prediction_evidence(&predictions[0], &evidence),
            ShadowEvidenceResolution::Duplicate
        );
        let snapshot = observer.snapshot();
        assert_eq!(snapshot.settled_witnesses, 1);
        assert_eq!(snapshot.duplicate_evidence, 1);
        assert_eq!(snapshot.pending_traces, 2);
        assert_eq!(snapshot.authority, ShadowAuthorityBoundary::default());
    }

    #[test]
    fn pending_capacity_is_bounded() {
        let mut observer = OmegaG2ShadowObserver::new(OmegaG2ShadowConfig {
            max_pending_traces: 2,
            ..OmegaG2ShadowConfig::default()
        })
        .expect("config");
        let predictions = vec![
            prediction(
                1,
                PredictionEngine::QuestionGravity,
                PredictionKind::Question,
                0.9,
            ),
            prediction(
                2,
                PredictionEngine::BeliefRevision,
                PredictionKind::Conclusion,
                0.8,
            ),
            prediction(
                3,
                PredictionEngine::Basin,
                PredictionKind::NecessaryTruth,
                0.7,
            ),
        ];
        observer.observe_prediction_batch(&predictions);
        let snapshot = observer.snapshot();
        assert_eq!(snapshot.pending_traces, 2);
        assert_eq!(snapshot.dropped_pending_traces, 1);
    }

    #[test]
    fn identical_replay_produces_identical_snapshot() {
        let predictions = vec![
            prediction(
                11,
                PredictionEngine::QuestionGravity,
                PredictionKind::Question,
                0.9,
            ),
            prediction(
                12,
                PredictionEngine::BeliefRevision,
                PredictionKind::Conclusion,
                0.8,
            ),
            prediction(
                13,
                PredictionEngine::Basin,
                PredictionKind::NecessaryTruth,
                0.7,
            ),
        ];
        let evidence = Evidence {
            outcome: PredictionOutcome::Refuted,
            prediction_id: PredictionId(12),
        };
        let mut first = OmegaG2ShadowObserver::default();
        let mut second = OmegaG2ShadowObserver::default();
        first.observe_prediction_batch(&predictions);
        second.observe_prediction_batch(&predictions);
        first.resolve_prediction_evidence(&predictions[1], &evidence);
        second.resolve_prediction_evidence(&predictions[1], &evidence);
        assert_eq!(first.snapshot(), second.snapshot());
    }

    #[test]
    fn no_defect_audit_remains_inert() {
        let intervention = Atom::new("selected:test").expect("atom");
        let outcome = Atom::new("confirmed").expect("atom");
        let events = vec![
            Atom::new("a").expect("atom"),
            Atom::new("b").expect("atom"),
            Atom::new("c").expect("atom"),
        ];
        let root = GrammarRoot {
            root_id: 7,
            discovery: (0..4)
                .map(|index| WitnessedHistory {
                    evidence_id: index + 1,
                    history: RawHistory {
                        history_id: index + 1,
                        events: events.clone(),
                    },
                    intervention: intervention.clone(),
                    outcome: outcome.clone(),
                })
                .collect(),
        };
        let audit = audit_shadow_root(&root, 8).expect("audit");
        assert_eq!(audit.classification, ShadowAuditClassification::NoDefects);
        assert_eq!(audit.c1_complete_repair_count, 0);
    }
}
