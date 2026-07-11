//! Endogenous executable state-space refinement for the Ω1 shadow experiment.
//!
//! The H9-H11 lineage changes executable knowledge inside a fixed symbolic
//! language. Ω1 instead asks whether a witnessed behavioral alias can force a
//! change to the executable state key itself. A bounded synthesis grammar
//! generates candidate history predicates from the raw atom vocabulary; an
//! independent validator recomputes the complete defect set and search before
//! issuing an opaque certificate. Only certificate admission can add the new
//! state-key dimension.

use crate::commitment_state::Atom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawHistory {
    pub history_id: u64,
    pub events: Vec<Atom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessedHistory {
    pub evidence_id: u64,
    pub history: RawHistory,
    pub intervention: Atom,
    pub outcome: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementProblem {
    pub root_id: u64,
    pub discovery: Vec<WitnessedHistory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MetricKind {
    FirstIndex,
    LastIndex,
    Count,
}

impl MetricKind {
    fn name(self) -> &'static str {
        match self {
            Self::FirstIndex => "first",
            Self::LastIndex => "last",
            Self::Count => "count",
        }
    }

    fn all() -> [Self; 3] {
        [Self::FirstIndex, Self::LastIndex, Self::Count]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HistoryMetric {
    pub kind: MetricKind,
    pub atom: Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Comparator {
    LessThan,
    Equal,
}

impl Comparator {
    fn symbol(self) -> &'static str {
        match self {
            Self::LessThan => "<",
            Self::Equal => "==",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RefinementProgram {
    pub left: HistoryMetric,
    pub comparator: Comparator,
    pub right: HistoryMetric,
}

impl RefinementProgram {
    pub fn execute(&self, history: &RawHistory) -> bool {
        let left = metric_value(&self.left, history);
        let right = metric_value(&self.right, history);
        match self.comparator {
            Comparator::LessThan => left < right,
            Comparator::Equal => left == right,
        }
    }

    pub fn canonical_string(&self) -> String {
        format!(
            "{}({}){}{}({})",
            self.left.kind.name(),
            self.left.atom.as_str(),
            self.comparator.symbol(),
            self.right.kind.name(),
            self.right.atom.as_str()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateKey {
    pub intervention: Atom,
    pub base_multiset: Vec<(Atom, usize)>,
    pub refinement_bits: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasDefect {
    pub left_evidence_id: u64,
    pub right_evidence_id: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
    pub unique_partitions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementConfig {
    pub min_partition_support: usize,
    pub min_winner_margin: usize,
}

impl Default for RefinementConfig {
    fn default() -> Self {
        Self {
            min_partition_support: 4,
            min_winner_margin: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub vocabulary_digest: u64,
    pub defect_digest: u64,
    pub candidate_program_count: usize,
    pub unique_partition_count: usize,
    pub detected_defects: usize,
    pub program: RefinementProgram,
    pub canonical_partition: Vec<bool>,
    pub repaired_defects: usize,
    pub unrepaired_defects: usize,
    pub partition_support_min: usize,
    pub runner_up_repaired_defects: usize,
    pub winner_margin: usize,
}

/// Opaque certificate: only independent full recomputation can construct one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedRefinementCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    program: RefinementProgram,
}

impl ValidatedRefinementCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn program(&self) -> &RefinementProgram {
        &self.program
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedRefinement {
    pub proof_id: u64,
    pub problem_digest: u64,
    pub program: RefinementProgram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateLanguage {
    root_id: u64,
    refinements: Vec<AdmittedRefinement>,
}

impl StateLanguage {
    pub fn new(root_id: u64) -> Self {
        Self {
            root_id,
            refinements: Vec::new(),
        }
    }

    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn refinement_count(&self) -> usize {
        self.refinements.len()
    }

    pub fn state_key(&self, history: &RawHistory, intervention: &Atom) -> StateKey {
        let mut counts = BTreeMap::<Atom, usize>::new();
        for atom in &history.events {
            *counts.entry(atom.clone()).or_insert(0) += 1;
        }
        let refinement_bits = self
            .refinements
            .iter()
            .map(|refinement| refinement.program.execute(history))
            .collect::<Vec<_>>();
        StateKey {
            intervention: intervention.clone(),
            base_multiset: counts.into_iter().collect(),
            refinement_bits,
        }
    }

    pub fn admit_certificate(
        &mut self,
        certificate: &ValidatedRefinementCertificate,
    ) -> Result<(), RepresentationGenesisError> {
        if certificate.root_id != self.root_id {
            return Err(RepresentationGenesisError::ForeignCertificate {
                expected_root: self.root_id,
                certificate_root: certificate.root_id,
            });
        }
        if self
            .refinements
            .iter()
            .any(|existing| existing.proof_id == certificate.proof_id)
        {
            return Err(RepresentationGenesisError::DuplicateRefinement(
                certificate.proof_id,
            ));
        }
        self.refinements.push(AdmittedRefinement {
            proof_id: certificate.proof_id,
            problem_digest: certificate.problem_digest,
            program: certificate.program.clone(),
        });
        self.verify_invariants()
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![format!("root:{}", self.root_id)];
        for refinement in &self.refinements {
            parts.push(format!(
                "{}:{}:{}",
                refinement.proof_id,
                refinement.problem_digest,
                refinement.program.canonical_string()
            ));
        }
        parts.join("|")
    }

    pub fn verify_invariants(&self) -> Result<(), RepresentationGenesisError> {
        let mut proof_ids = BTreeSet::new();
        for refinement in &self.refinements {
            if !proof_ids.insert(refinement.proof_id) {
                return Err(RepresentationGenesisError::InvariantViolation(
                    "duplicate admitted proof id".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RepresentationGenesisError {
    #[error("refinement problem has no discovery histories")]
    EmptyDiscovery,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("duplicate history id {0}")]
    DuplicateHistory(u64),
    #[error("no representational alias defects were detected")]
    NoAliasDefect,
    #[error("candidate program search produced no partitions")]
    NoCandidatePartition,
    #[error("refinement proof does not match independent recomputation: {0}")]
    ProofMismatch(&'static str),
    #[error("refinement certificate gates failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign certificate: expected root {expected_root}, got {certificate_root}")]
    ForeignCertificate {
        expected_root: u64,
        certificate_root: u64,
    },
    #[error("refinement proof id {0} is already admitted")]
    DuplicateRefinement(u64),
    #[error("state-language invariant violation: {0}")]
    InvariantViolation(String),
}

#[derive(Debug, Clone)]
struct SearchResult {
    proof: RefinementProof,
}

pub fn derive_vocabulary(
    problem: &RefinementProblem,
    budget: &mut GenesisBudget,
) -> Result<Vec<Atom>, RepresentationGenesisError> {
    validate_problem(problem)?;
    let mut vocabulary = BTreeSet::new();
    for episode in &problem.discovery {
        budget.vocabulary_history_scans = budget.vocabulary_history_scans.saturating_add(1);
        for atom in &episode.history.events {
            vocabulary.insert(atom.clone());
        }
    }
    Ok(vocabulary.into_iter().collect())
}

pub fn enumerate_programs(vocabulary: &[Atom]) -> Vec<RefinementProgram> {
    let mut metrics = Vec::new();
    for atom in vocabulary {
        for kind in MetricKind::all() {
            metrics.push(HistoryMetric {
                kind,
                atom: atom.clone(),
            });
        }
    }
    metrics.sort();

    let mut programs = Vec::new();
    for left_index in 0..metrics.len() {
        for right_index in 0..metrics.len() {
            if left_index == right_index {
                continue;
            }
            programs.push(RefinementProgram {
                left: metrics[left_index].clone(),
                comparator: Comparator::LessThan,
                right: metrics[right_index].clone(),
            });
            if left_index < right_index {
                programs.push(RefinementProgram {
                    left: metrics[left_index].clone(),
                    comparator: Comparator::Equal,
                    right: metrics[right_index].clone(),
                });
            }
        }
    }
    programs.sort();
    programs
}

pub fn detect_alias_defects(
    problem: &RefinementProblem,
    language: &StateLanguage,
    budget: &mut GenesisBudget,
) -> Result<Vec<AliasDefect>, RepresentationGenesisError> {
    validate_problem(problem)?;
    let mut defects = Vec::new();
    for left_index in 0..problem.discovery.len() {
        for right_index in (left_index + 1)..problem.discovery.len() {
            budget.history_pair_evaluations = budget.history_pair_evaluations.saturating_add(1);
            let left = &problem.discovery[left_index];
            let right = &problem.discovery[right_index];
            if left.intervention == right.intervention
                && left.outcome != right.outcome
                && language.state_key(&left.history, &left.intervention)
                    == language.state_key(&right.history, &right.intervention)
            {
                defects.push(AliasDefect {
                    left_evidence_id: left.evidence_id,
                    right_evidence_id: right.evidence_id,
                });
            }
        }
    }
    if defects.is_empty() {
        return Err(RepresentationGenesisError::NoAliasDefect);
    }
    defects.sort_by_key(|defect| (defect.left_evidence_id, defect.right_evidence_id));
    Ok(defects)
}

pub fn synthesize_refinement(
    problem: &RefinementProblem,
    config: RefinementConfig,
    budget: &mut GenesisBudget,
) -> Result<RefinementProof, RepresentationGenesisError> {
    search(problem, config, budget).map(|result| result.proof)
}

pub fn validate_refinement(
    problem: &RefinementProblem,
    proof: &RefinementProof,
    config: RefinementConfig,
    budget: &mut GenesisBudget,
) -> Result<ValidatedRefinementCertificate, RepresentationGenesisError> {
    let recomputed = search(problem, config, budget)?.proof;
    compare_proof(proof, &recomputed)?;
    enforce_certificate_gates(&recomputed, config)?;
    Ok(ValidatedRefinementCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        program: recomputed.program,
    })
}

pub fn expected_candidate_programs(vocabulary_size: usize) -> usize {
    let metrics = vocabulary_size.saturating_mul(3);
    metrics
        .saturating_mul(metrics.saturating_sub(1))
        .saturating_add(metrics.saturating_mul(metrics.saturating_sub(1)) / 2)
}

fn search(
    problem: &RefinementProblem,
    _config: RefinementConfig,
    budget: &mut GenesisBudget,
) -> Result<SearchResult, RepresentationGenesisError> {
    validate_problem(problem)?;
    let base_language = StateLanguage::new(problem.root_id);
    let vocabulary = derive_vocabulary(problem, budget)?;
    let defects = detect_alias_defects(problem, &base_language, budget)?;
    let programs = enumerate_programs(&vocabulary);
    budget.candidate_programs = programs.len();

    let evidence_index = problem
        .discovery
        .iter()
        .enumerate()
        .map(|(index, episode)| (episode.evidence_id, index))
        .collect::<BTreeMap<_, _>>();

    let mut partitions = BTreeMap::<Vec<bool>, RefinementProgram>::new();
    for program in programs {
        let mut bits = Vec::with_capacity(problem.discovery.len());
        for episode in &problem.discovery {
            budget.program_history_evaluations =
                budget.program_history_evaluations.saturating_add(1);
            bits.push(program.execute(&episode.history));
        }
        let canonical = canonical_partition(bits);
        match partitions.get_mut(&canonical) {
            Some(existing) => {
                if program.canonical_string() < existing.canonical_string() {
                    *existing = program;
                }
            }
            None => {
                partitions.insert(canonical, program);
            }
        }
    }
    budget.unique_partitions = partitions.len();
    if partitions.is_empty() {
        return Err(RepresentationGenesisError::NoCandidatePartition);
    }

    let mut ranked = Vec::<(usize, usize, Vec<bool>, RefinementProgram)>::new();
    for (partition, program) in partitions {
        let repaired = defects
            .iter()
            .filter(|defect| {
                let left = evidence_index[&defect.left_evidence_id];
                let right = evidence_index[&defect.right_evidence_id];
                partition[left] != partition[right]
            })
            .count();
        let true_count = partition.iter().filter(|value| **value).count();
        let support_min = true_count.min(partition.len().saturating_sub(true_count));
        ranked.push((repaired, support_min, partition, program));
    }

    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.3.canonical_string().cmp(&right.3.canonical_string()))
    });

    let winner = ranked
        .first()
        .ok_or(RepresentationGenesisError::NoCandidatePartition)?;
    let runner_up = ranked.get(1).map(|entry| entry.0).unwrap_or(0);
    let winner_margin = winner.0.saturating_sub(runner_up);
    let unrepaired = defects.len().saturating_sub(winner.0);
    let problem_digest = digest_problem(problem);
    let vocabulary_digest = digest_atoms(&vocabulary);
    let defect_digest = digest_defects(&defects);
    let proof_id = digest_proof_identity(problem.root_id, problem_digest, &winner.3, &winner.2);

    Ok(SearchResult {
        proof: RefinementProof {
            proof_id,
            root_id: problem.root_id,
            problem_digest,
            vocabulary_digest,
            defect_digest,
            candidate_program_count: budget.candidate_programs,
            unique_partition_count: budget.unique_partitions,
            detected_defects: defects.len(),
            program: winner.3.clone(),
            canonical_partition: winner.2.clone(),
            repaired_defects: winner.0,
            unrepaired_defects: unrepaired,
            partition_support_min: winner.1,
            runner_up_repaired_defects: runner_up,
            winner_margin,
        },
    })
}

fn enforce_certificate_gates(
    proof: &RefinementProof,
    config: RefinementConfig,
) -> Result<(), RepresentationGenesisError> {
    if proof.detected_defects == 0 {
        return Err(RepresentationGenesisError::CertificateGate("no_defects"));
    }
    if proof.repaired_defects != proof.detected_defects || proof.unrepaired_defects != 0 {
        return Err(RepresentationGenesisError::CertificateGate(
            "incomplete_defect_repair",
        ));
    }
    if proof.partition_support_min < config.min_partition_support {
        return Err(RepresentationGenesisError::CertificateGate(
            "insufficient_partition_support",
        ));
    }
    if proof.winner_margin < config.min_winner_margin {
        return Err(RepresentationGenesisError::CertificateGate(
            "insufficient_winner_margin",
        ));
    }
    Ok(())
}

fn compare_proof(
    supplied: &RefinementProof,
    recomputed: &RefinementProof,
) -> Result<(), RepresentationGenesisError> {
    macro_rules! check {
        ($field:ident) => {
            if supplied.$field != recomputed.$field {
                return Err(RepresentationGenesisError::ProofMismatch(stringify!($field)));
            }
        };
    }
    check!(proof_id);
    check!(root_id);
    check!(problem_digest);
    check!(vocabulary_digest);
    check!(defect_digest);
    check!(candidate_program_count);
    check!(unique_partition_count);
    check!(detected_defects);
    check!(program);
    check!(canonical_partition);
    check!(repaired_defects);
    check!(unrepaired_defects);
    check!(partition_support_min);
    check!(runner_up_repaired_defects);
    check!(winner_margin);
    Ok(())
}

fn validate_problem(problem: &RefinementProblem) -> Result<(), RepresentationGenesisError> {
    if problem.discovery.is_empty() {
        return Err(RepresentationGenesisError::EmptyDiscovery);
    }
    let mut evidence_ids = BTreeSet::new();
    let mut history_ids = BTreeSet::new();
    for episode in &problem.discovery {
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(RepresentationGenesisError::DuplicateEvidence(
                episode.evidence_id,
            ));
        }
        if !history_ids.insert(episode.history.history_id) {
            return Err(RepresentationGenesisError::DuplicateHistory(
                episode.history.history_id,
            ));
        }
    }
    Ok(())
}

fn metric_value(metric: &HistoryMetric, history: &RawHistory) -> i64 {
    match metric.kind {
        MetricKind::FirstIndex => history
            .events
            .iter()
            .position(|atom| atom == &metric.atom)
            .map(|index| index as i64)
            .unwrap_or(i64::MAX / 4),
        MetricKind::LastIndex => history
            .events
            .iter()
            .rposition(|atom| atom == &metric.atom)
            .map(|index| index as i64)
            .unwrap_or(i64::MAX / 4),
        MetricKind::Count => history
            .events
            .iter()
            .filter(|atom| *atom == &metric.atom)
            .count() as i64,
    }
}

fn canonical_partition(bits: Vec<bool>) -> Vec<bool> {
    let inverted = bits.iter().map(|value| !*value).collect::<Vec<_>>();
    if inverted < bits {
        inverted
    } else {
        bits
    }
}

fn digest_problem(problem: &RefinementProblem) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&problem.root_id.to_le_bytes());
    for episode in &problem.discovery {
        bytes.extend_from_slice(&episode.evidence_id.to_le_bytes());
        bytes.extend_from_slice(&episode.history.history_id.to_le_bytes());
        for atom in &episode.history.events {
            bytes.extend_from_slice(atom.as_str().as_bytes());
            bytes.push(0x11);
        }
        bytes.push(0x22);
        bytes.extend_from_slice(episode.intervention.as_str().as_bytes());
        bytes.push(0x33);
        bytes.extend_from_slice(episode.outcome.as_str().as_bytes());
        bytes.push(0x44);
    }
    fnv1a64(&bytes)
}

fn digest_atoms(atoms: &[Atom]) -> u64 {
    let mut bytes = Vec::new();
    for atom in atoms {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0x51);
    }
    fnv1a64(&bytes)
}

fn digest_defects(defects: &[AliasDefect]) -> u64 {
    let mut bytes = Vec::new();
    for defect in defects {
        bytes.extend_from_slice(&defect.left_evidence_id.to_le_bytes());
        bytes.extend_from_slice(&defect.right_evidence_id.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn digest_proof_identity(
    root_id: u64,
    problem_digest: u64,
    program: &RefinementProgram,
    partition: &[bool],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(program.canonical_string().as_bytes());
    for bit in partition {
        bytes.push(u8::from(*bit));
    }
    fnv1a64(&bytes)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn episode(id: u64, order: &str, positive: bool) -> WitnessedHistory {
        let atoms = order
            .chars()
            .map(|ch| atom(&ch.to_string()))
            .collect::<Vec<_>>();
        WitnessedHistory {
            evidence_id: id,
            history: RawHistory {
                history_id: 1000 + id,
                events: atoms,
            },
            intervention: atom("probe"),
            outcome: atom(if positive { "up" } else { "down" }),
        }
    }

    fn problem(root_id: u64) -> RefinementProblem {
        let rows = [
            ("BACDXY", true),
            ("XDBYAC", true),
            ("DCXYBA", true),
            ("CDAYBX", false),
            ("YBDXAC", false),
            ("BYXACD", false),
            ("YDCBXA", false),
            ("CAYBXD", false),
            ("DXACYB", true),
            ("ADXYCB", true),
            ("XACDBY", true),
            ("YDCABX", false),
        ];
        RefinementProblem {
            root_id,
            discovery: rows
                .iter()
                .enumerate()
                .map(|(index, (order, positive))| episode(index as u64 + 1, order, *positive))
                .collect(),
        }
    }

    #[test]
    fn detects_alias_and_synthesizes_a_complete_refinement() {
        let problem = problem(7);
        let language = StateLanguage::new(7);
        let mut defect_budget = GenesisBudget::default();
        let defects = detect_alias_defects(&problem, &language, &mut defect_budget).unwrap();
        assert_eq!(defects.len(), 36);
        assert_eq!(defect_budget.history_pair_evaluations, 66);

        let mut proposal_budget = GenesisBudget::default();
        let proof = synthesize_refinement(
            &problem,
            RefinementConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        assert_eq!(proposal_budget.vocabulary_history_scans, 12);
        assert_eq!(proposal_budget.candidate_programs, 459);
        assert_eq!(proposal_budget.program_history_evaluations, 5508);
        assert_eq!(proof.detected_defects, 36);
        assert_eq!(proof.repaired_defects, 36);
        assert_eq!(proof.unrepaired_defects, 0);
        assert!(proof.winner_margin >= 2);

        let mut validation_budget = GenesisBudget::default();
        let certificate = validate_refinement(
            &problem,
            &proof,
            RefinementConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        assert_eq!(validation_budget.candidate_programs, 459);
        assert_eq!(validation_budget.program_history_evaluations, 5508);

        let mut language = StateLanguage::new(7);
        language.admit_certificate(&certificate).unwrap();
        assert_eq!(language.refinement_count(), 1);
        language.verify_invariants().unwrap();
    }

    #[test]
    fn foreign_certificate_is_rejected() {
        let problem = problem(7);
        let mut proposal_budget = GenesisBudget::default();
        let proof = synthesize_refinement(
            &problem,
            RefinementConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        let mut validation_budget = GenesisBudget::default();
        let certificate = validate_refinement(
            &problem,
            &proof,
            RefinementConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        let mut foreign = StateLanguage::new(8);
        assert!(matches!(
            foreign.admit_certificate(&certificate),
            Err(RepresentationGenesisError::ForeignCertificate { .. })
        ));
    }

    #[test]
    fn tampered_proof_fails_independent_recomputation() {
        let problem = problem(7);
        let mut proposal_budget = GenesisBudget::default();
        let mut proof = synthesize_refinement(
            &problem,
            RefinementConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        proof.repaired_defects = proof.repaired_defects.saturating_sub(1);
        let mut validation_budget = GenesisBudget::default();
        assert!(matches!(
            validate_refinement(
                &problem,
                &proof,
                RefinementConfig::default(),
                &mut validation_budget,
            ),
            Err(RepresentationGenesisError::ProofMismatch("repaired_defects"))
        ));
        assert_eq!(validation_budget.candidate_programs, 459);
        assert_eq!(validation_budget.program_history_evaluations, 5508);
    }
}
