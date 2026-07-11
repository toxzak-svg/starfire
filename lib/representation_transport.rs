//! Proof-carrying representative transportability for the ΩR1 shadow experiment.
//!
//! Ω1 identifies a behavioral partition over observed histories but chooses one
//! executable representative by syntax. ΩR1 keeps the same discovery search,
//! then distinguishes programs inside the winning discovery-equivalence class by
//! their same-history behavior under a frozen structure-preserving transformation
//! suite. Transformed outcome labels are never consumed by transport scoring.

use crate::commitment_state::Atom;
use crate::representation_genesis::{
    enumerate_programs, RawHistory, RefinementProblem, RefinementProgram, StateKey,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockPermutation {
    pub block_width: usize,
    pub block_order: Vec<usize>,
}

impl BlockPermutation {
    pub fn apply(&self, history: &RawHistory) -> Result<RawHistory, TransportError> {
        if self.block_width == 0 || history.events.len() % self.block_width != 0 {
            return Err(TransportError::InvalidTransformation(
                "block width does not divide history length".to_string(),
            ));
        }
        let block_count = history.events.len() / self.block_width;
        if self.block_order.len() != block_count {
            return Err(TransportError::InvalidTransformation(
                "block order length does not match history block count".to_string(),
            ));
        }
        let unique = self.block_order.iter().copied().collect::<BTreeSet<_>>();
        if unique.len() != block_count || unique.iter().any(|index| *index >= block_count) {
            return Err(TransportError::InvalidTransformation(
                "block order is not a permutation".to_string(),
            ));
        }
        let mut events = Vec::with_capacity(history.events.len());
        for block in &self.block_order {
            let start = block.saturating_mul(self.block_width);
            let end = start.saturating_add(self.block_width);
            events.extend_from_slice(&history.events[start..end]);
        }
        Ok(RawHistory {
            history_id: history.history_id,
            events,
        })
    }

    pub fn canonical_string(&self) -> String {
        format!(
            "w{}:[{}]",
            self.block_width,
            self.block_order
                .iter()
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrespondenceMode {
    SameHistory,
    CyclicNext,
}

impl CorrespondenceMode {
    fn name(self) -> &'static str {
        match self {
            Self::SameHistory => "same_history",
            Self::CyclicNext => "cyclic_next",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformationSuite {
    pub transformations: Vec<BlockPermutation>,
    pub correspondence: CorrespondenceMode,
}

impl TransformationSuite {
    pub fn digest(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.correspondence.name().as_bytes());
        bytes.push(0xff);
        for transformation in &self.transformations {
            bytes.extend_from_slice(transformation.canonical_string().as_bytes());
            bytes.push(0xfe);
        }
        stable_digest(&bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportConfig {
    pub min_partition_support: usize,
    pub min_winner_margin: usize,
    pub max_selected_transport_violations: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            min_partition_support: 8,
            min_winner_margin: 32,
            max_selected_transport_violations: 0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub candidate_programs: usize,
    pub discovery_program_history_evaluations: usize,
    pub unique_partitions: usize,
    pub winning_class_representatives: usize,
    pub calibration_transformations: usize,
    pub transport_program_history_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub vocabulary_digest: u64,
    pub transformation_suite_digest: u64,
    pub candidate_program_count: usize,
    pub unique_partition_count: usize,
    pub opposite_outcome_pairs: usize,
    pub canonical_partition: Vec<bool>,
    pub repaired_pairs: usize,
    pub runner_up_repaired_pairs: usize,
    pub winner_margin: usize,
    pub partition_support_min: usize,
    pub winning_class_representatives: usize,
    pub zero_violation_representatives: usize,
    pub minimum_transport_violations: usize,
    pub selected_transport_violations: usize,
    pub program: RefinementProgram,
}

/// Opaque certificate issued only after independent full recomputation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedTransportCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    transformation_suite_digest: u64,
    program: RefinementProgram,
}

impl ValidatedTransportCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn program(&self) -> &RefinementProgram {
        &self.program
    }

    pub fn transformation_suite_digest(&self) -> u64 {
        self.transformation_suite_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedTransportRefinement {
    pub proof_id: u64,
    pub problem_digest: u64,
    pub transformation_suite_digest: u64,
    pub program: RefinementProgram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportStateLanguage {
    root_id: u64,
    refinement: Option<AdmittedTransportRefinement>,
}

impl TransportStateLanguage {
    pub fn new(root_id: u64) -> Self {
        Self {
            root_id,
            refinement: None,
        }
    }

    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn refinement_count(&self) -> usize {
        usize::from(self.refinement.is_some())
    }

    pub fn admitted_program(&self) -> Option<&RefinementProgram> {
        self.refinement.as_ref().map(|refinement| &refinement.program)
    }

    pub fn admit_certificate(
        &mut self,
        certificate: &ValidatedTransportCertificate,
    ) -> Result<(), TransportError> {
        if certificate.root_id != self.root_id {
            return Err(TransportError::ForeignCertificate {
                expected_root: self.root_id,
                certificate_root: certificate.root_id,
            });
        }
        if self.refinement.is_some() {
            return Err(TransportError::DuplicateAdmission);
        }
        self.refinement = Some(AdmittedTransportRefinement {
            proof_id: certificate.proof_id,
            problem_digest: certificate.problem_digest,
            transformation_suite_digest: certificate.transformation_suite_digest,
            program: certificate.program.clone(),
        });
        self.verify_invariants()
    }

    pub fn state_key(&self, history: &RawHistory, intervention: &Atom) -> StateKey {
        let mut counts = BTreeMap::<Atom, usize>::new();
        for atom in &history.events {
            *counts.entry(atom.clone()).or_default() += 1;
        }
        let refinement_bits = self
            .refinement
            .iter()
            .map(|refinement| refinement.program.execute(history))
            .collect();
        StateKey {
            intervention: intervention.clone(),
            base_multiset: counts.into_iter().collect(),
            refinement_bits,
        }
    }

    pub fn canonical_signature(&self) -> String {
        match &self.refinement {
            Some(refinement) => format!(
                "root:{}|{}:{}:{}:{}",
                self.root_id,
                refinement.proof_id,
                refinement.problem_digest,
                refinement.transformation_suite_digest,
                refinement.program.canonical_string()
            ),
            None => format!("root:{}|L0", self.root_id),
        }
    }

    pub fn verify_invariants(&self) -> Result<(), TransportError> {
        if let Some(refinement) = &self.refinement {
            if refinement.program.left == refinement.program.right {
                return Err(TransportError::InvariantViolation(
                    "transport refinement compares one metric with itself".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportError {
    #[error("transport problem has no discovery histories")]
    EmptyDiscovery,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("duplicate history id {0}")]
    DuplicateHistory(u64),
    #[error("no opposite-outcome behavioral pairs exist")]
    NoBehavioralOpposition,
    #[error("transport search produced no behavioral partitions")]
    NoCandidatePartition,
    #[error("winning behavioral partition has no executable representatives")]
    EmptyWinningClass,
    #[error("invalid transformation: {0}")]
    InvalidTransformation(String),
    #[error("transport proof does not match independent recomputation")]
    ProofMismatch,
    #[error("transport certificate gate failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign transport certificate: expected root {expected_root}, got {certificate_root}")]
    ForeignCertificate {
        expected_root: u64,
        certificate_root: u64,
    },
    #[error("transport refinement already admitted")]
    DuplicateAdmission,
    #[error("transport-state invariant violation: {0}")]
    InvariantViolation(String),
}

pub fn synthesize_transport_refinement(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    config: TransportConfig,
    budget: &mut TransportBudget,
) -> Result<TransportProof, TransportError> {
    search(problem, suite, config, budget)
}

pub fn validate_transport_refinement(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    supplied: &TransportProof,
    config: TransportConfig,
    budget: &mut TransportBudget,
) -> Result<ValidatedTransportCertificate, TransportError> {
    let recomputed = search(problem, suite, config, budget)?;
    if supplied != &recomputed {
        return Err(TransportError::ProofMismatch);
    }
    enforce_certificate_gates(&recomputed, config)?;
    Ok(ValidatedTransportCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        transformation_suite_digest: recomputed.transformation_suite_digest,
        program: recomputed.program,
    })
}

fn search(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    _config: TransportConfig,
    budget: &mut TransportBudget,
) -> Result<TransportProof, TransportError> {
    validate_problem(problem)?;
    validate_suite(problem, suite)?;
    let vocabulary = derive_vocabulary(problem, budget);
    let vocabulary_digest = digest_atoms(&vocabulary);
    let programs = enumerate_programs(&vocabulary);
    budget.candidate_programs = programs.len();
    let opposite_pairs = opposite_outcome_pairs(problem, budget)?;

    let mut partitions = BTreeMap::<Vec<bool>, Vec<RefinementProgram>>::new();
    for program in programs {
        let mut bits = Vec::with_capacity(problem.discovery.len());
        for episode in &problem.discovery {
            budget.discovery_program_history_evaluations = budget
                .discovery_program_history_evaluations
                .saturating_add(1);
            bits.push(program.execute(&episode.history));
        }
        partitions
            .entry(canonical_partition(bits))
            .or_default()
            .push(program);
    }
    budget.unique_partitions = partitions.len();
    if partitions.is_empty() {
        return Err(TransportError::NoCandidatePartition);
    }

    let mut ranked_partitions = partitions
        .into_iter()
        .map(|(partition, mut representatives)| {
            representatives.sort_by_key(RefinementProgram::canonical_string);
            let repaired = repaired_pairs(&partition, &opposite_pairs);
            let true_count = partition.iter().filter(|value| **value).count();
            let support_min = true_count.min(partition.len().saturating_sub(true_count));
            let canonical = representatives
                .first()
                .map(RefinementProgram::canonical_string)
                .unwrap_or_default();
            (repaired, support_min, canonical, partition, representatives)
        })
        .collect::<Vec<_>>();
    ranked_partitions.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let winner = ranked_partitions
        .first()
        .ok_or(TransportError::NoCandidatePartition)?;
    let runner_up = ranked_partitions.get(1).map(|entry| entry.0).unwrap_or(0);
    if winner.4.is_empty() {
        return Err(TransportError::EmptyWinningClass);
    }
    budget.winning_class_representatives = winner.4.len();
    budget.calibration_transformations = suite.transformations.len();

    let original_outputs = winner
        .4
        .iter()
        .map(|program| {
            problem
                .discovery
                .iter()
                .map(|episode| program.execute(&episode.history))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut transport_rank = Vec::<(usize, String, RefinementProgram)>::new();
    for (program_index, program) in winner.4.iter().enumerate() {
        let mut violations = 0_usize;
        for transformation in &suite.transformations {
            for (history_index, episode) in problem.discovery.iter().enumerate() {
                let transformed = transformation.apply(&episode.history)?;
                budget.transport_program_history_evaluations = budget
                    .transport_program_history_evaluations
                    .saturating_add(1);
                let transformed_output = program.execute(&transformed);
                let reference_index = match suite.correspondence {
                    CorrespondenceMode::SameHistory => history_index,
                    CorrespondenceMode::CyclicNext => {
                        (history_index + 1) % problem.discovery.len()
                    }
                };
                if transformed_output != original_outputs[program_index][reference_index] {
                    violations = violations.saturating_add(1);
                }
            }
        }
        transport_rank.push((violations, program.canonical_string(), program.clone()));
    }
    transport_rank.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let selected = transport_rank
        .first()
        .ok_or(TransportError::EmptyWinningClass)?;
    let minimum_transport_violations = selected.0;
    let zero_violation_representatives = transport_rank
        .iter()
        .filter(|entry| entry.0 == 0)
        .count();

    let problem_digest = digest_problem(problem);
    let transformation_suite_digest = suite.digest();
    let winner_margin = winner.0.saturating_sub(runner_up);
    let proof_id = digest_proof_identity(
        problem.root_id,
        problem_digest,
        vocabulary_digest,
        transformation_suite_digest,
        &winner.3,
        &selected.2,
        selected.0,
    );

    Ok(TransportProof {
        proof_id,
        root_id: problem.root_id,
        problem_digest,
        vocabulary_digest,
        transformation_suite_digest,
        candidate_program_count: budget.candidate_programs,
        unique_partition_count: budget.unique_partitions,
        opposite_outcome_pairs: opposite_pairs.len(),
        canonical_partition: winner.3.clone(),
        repaired_pairs: winner.0,
        runner_up_repaired_pairs: runner_up,
        winner_margin,
        partition_support_min: winner.1,
        winning_class_representatives: winner.4.len(),
        zero_violation_representatives,
        minimum_transport_violations,
        selected_transport_violations: selected.0,
        program: selected.2.clone(),
    })
}

fn enforce_certificate_gates(
    proof: &TransportProof,
    config: TransportConfig,
) -> Result<(), TransportError> {
    if proof.opposite_outcome_pairs == 0 {
        return Err(TransportError::CertificateGate("no_behavioral_opposition"));
    }
    if proof.repaired_pairs != proof.opposite_outcome_pairs {
        return Err(TransportError::CertificateGate(
            "incomplete_behavioral_partition",
        ));
    }
    if proof.partition_support_min < config.min_partition_support {
        return Err(TransportError::CertificateGate(
            "insufficient_partition_support",
        ));
    }
    if proof.winner_margin < config.min_winner_margin {
        return Err(TransportError::CertificateGate(
            "insufficient_winner_margin",
        ));
    }
    if proof.selected_transport_violations > config.max_selected_transport_violations {
        return Err(TransportError::CertificateGate(
            "transport_violations_exceed_limit",
        ));
    }
    if proof.zero_violation_representatives == 0 {
        return Err(TransportError::CertificateGate(
            "no_zero_violation_representative",
        ));
    }
    Ok(())
}

fn validate_problem(problem: &RefinementProblem) -> Result<(), TransportError> {
    if problem.discovery.is_empty() {
        return Err(TransportError::EmptyDiscovery);
    }
    let mut evidence_ids = BTreeSet::new();
    let mut history_ids = BTreeSet::new();
    for episode in &problem.discovery {
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(TransportError::DuplicateEvidence(episode.evidence_id));
        }
        if !history_ids.insert(episode.history.history_id) {
            return Err(TransportError::DuplicateHistory(episode.history.history_id));
        }
    }
    Ok(())
}

fn validate_suite(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
) -> Result<(), TransportError> {
    if suite.transformations.is_empty() {
        return Err(TransportError::InvalidTransformation(
            "transformation suite is empty".to_string(),
        ));
    }
    let exemplar = &problem
        .discovery
        .first()
        .ok_or(TransportError::EmptyDiscovery)?
        .history;
    for transformation in &suite.transformations {
        let _ = transformation.apply(exemplar)?;
    }
    Ok(())
}

fn derive_vocabulary(problem: &RefinementProblem, budget: &mut TransportBudget) -> Vec<Atom> {
    let mut vocabulary = BTreeSet::new();
    for episode in &problem.discovery {
        budget.vocabulary_history_scans = budget.vocabulary_history_scans.saturating_add(1);
        for atom in &episode.history.events {
            vocabulary.insert(atom.clone());
        }
    }
    vocabulary.into_iter().collect()
}

fn opposite_outcome_pairs(
    problem: &RefinementProblem,
    budget: &mut TransportBudget,
) -> Result<Vec<(usize, usize)>, TransportError> {
    let mut pairs = Vec::new();
    for left in 0..problem.discovery.len() {
        for right in (left + 1)..problem.discovery.len() {
            budget.history_pair_evaluations = budget.history_pair_evaluations.saturating_add(1);
            let left_episode = &problem.discovery[left];
            let right_episode = &problem.discovery[right];
            if left_episode.intervention == right_episode.intervention
                && left_episode.outcome != right_episode.outcome
            {
                pairs.push((left, right));
            }
        }
    }
    if pairs.is_empty() {
        return Err(TransportError::NoBehavioralOpposition);
    }
    Ok(pairs)
}

fn repaired_pairs(partition: &[bool], pairs: &[(usize, usize)]) -> usize {
    pairs
        .iter()
        .filter(|(left, right)| partition[*left] != partition[*right])
        .count()
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
    stable_digest(&bytes)
}

fn digest_atoms(atoms: &[Atom]) -> u64 {
    let mut bytes = Vec::new();
    for atom in atoms {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0x51);
    }
    stable_digest(&bytes)
}

fn digest_proof_identity(
    root_id: u64,
    problem_digest: u64,
    vocabulary_digest: u64,
    transformation_suite_digest: u64,
    partition: &[bool],
    program: &RefinementProgram,
    violations: usize,
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(&vocabulary_digest.to_le_bytes());
    bytes.extend_from_slice(&transformation_suite_digest.to_le_bytes());
    for bit in partition {
        bytes.push(u8::from(*bit));
    }
    bytes.extend_from_slice(program.canonical_string().as_bytes());
    bytes.extend_from_slice(&(violations as u64).to_le_bytes());
    stable_digest(&bytes)
}

fn stable_digest(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::representation_genesis::{RefinementProblem, WitnessedHistory};

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn problem(root_id: u64) -> RefinementProblem {
        let atoms = [
            atom("A"), atom("B"), atom("C"), atom("D"),
            atom("E"), atom("F"), atom("G"), atom("H"),
        ];
        let intervention = atom("probe");
        let mut discovery = Vec::new();
        let mut index = 0_u64;
        for p in [false, true] {
            for q in [false, true] {
                for r in [false, true] {
                    for s in [false, true] {
                        let bits = [p, q, r, s];
                        let pairs = [
                            (&atoms[0], &atoms[1], bits[0]),
                            (&atoms[2], &atoms[3], bits[1]),
                            (&atoms[4], &atoms[5], bits[2]),
                            (&atoms[6], &atoms[7], bits[3]),
                        ];
                        let mut events = Vec::new();
                        for (left, right, bit) in pairs {
                            if bit {
                                events.push(left.clone());
                                events.push(right.clone());
                            } else {
                                events.push(right.clone());
                                events.push(left.clone());
                            }
                        }
                        discovery.push(WitnessedHistory {
                            evidence_id: 1_000 + index,
                            history: RawHistory {
                                history_id: 100 + index,
                                events,
                            },
                            intervention: intervention.clone(),
                            outcome: atom(if p { "one" } else { "zero" }),
                        });
                        index += 1;
                    }
                }
            }
        }
        RefinementProblem { root_id, discovery }
    }

    fn moving_suite() -> TransformationSuite {
        TransformationSuite {
            transformations: vec![
                BlockPermutation { block_width: 2, block_order: vec![1, 0, 2, 3] },
                BlockPermutation { block_width: 2, block_order: vec![1, 2, 0, 3] },
            ],
            correspondence: CorrespondenceMode::SameHistory,
        }
    }

    fn stationary_suite() -> TransformationSuite {
        TransformationSuite {
            transformations: vec![
                BlockPermutation { block_width: 2, block_order: vec![0, 2, 1, 3] },
                BlockPermutation { block_width: 2, block_order: vec![0, 3, 2, 1] },
            ],
            correspondence: CorrespondenceMode::SameHistory,
        }
    }

    #[test]
    fn moving_orbit_selects_exact_stable_frontier() {
        let problem = problem(1);
        let mut budget = TransportBudget::default();
        let proof = synthesize_transport_refinement(
            &problem,
            &moving_suite(),
            TransportConfig::default(),
            &mut budget,
        )
        .unwrap();
        assert_eq!(proof.candidate_program_count, 828);
        assert_eq!(proof.unique_partition_count, 5);
        assert_eq!(proof.opposite_outcome_pairs, 64);
        assert_eq!(proof.repaired_pairs, 64);
        assert_eq!(proof.runner_up_repaired_pairs, 32);
        assert_eq!(proof.winner_margin, 32);
        assert_eq!(proof.partition_support_min, 8);
        assert_eq!(proof.winning_class_representatives, 72);
        assert_eq!(proof.zero_violation_representatives, 8);
        assert_eq!(proof.minimum_transport_violations, 0);
        assert_eq!(proof.selected_transport_violations, 0);
        assert_eq!(proof.program.canonical_string(), "first(A)<first(B)");
        assert_eq!(budget.discovery_program_history_evaluations, 13_248);
        assert_eq!(budget.transport_program_history_evaluations, 2_304);
    }

    #[test]
    fn stationary_suite_leaves_all_discovery_representatives_tied() {
        let problem = problem(2);
        let mut budget = TransportBudget::default();
        let proof = synthesize_transport_refinement(
            &problem,
            &stationary_suite(),
            TransportConfig::default(),
            &mut budget,
        )
        .unwrap();
        assert_eq!(proof.winning_class_representatives, 72);
        assert_eq!(proof.zero_violation_representatives, 72);
        assert_eq!(proof.program.canonical_string(), "first(A)<count(A)");
        assert_eq!(budget.transport_program_history_evaluations, 2_304);
    }

    #[test]
    fn rewired_correspondence_has_no_zero_violation_certificate() {
        let problem = problem(3);
        let mut suite = moving_suite();
        suite.correspondence = CorrespondenceMode::CyclicNext;
        let mut proposal_budget = TransportBudget::default();
        let proof = synthesize_transport_refinement(
            &problem,
            &suite,
            TransportConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        assert_eq!(proof.zero_violation_representatives, 0);
        assert_eq!(proof.minimum_transport_violations, 4);
        let mut validation_budget = TransportBudget::default();
        let error = validate_transport_refinement(
            &problem,
            &suite,
            &proof,
            TransportConfig::default(),
            &mut validation_budget,
        )
        .unwrap_err();
        assert_eq!(
            error,
            TransportError::CertificateGate("transport_violations_exceed_limit")
        );
        assert_eq!(validation_budget.transport_program_history_evaluations, 2_304);
    }

    #[test]
    fn validated_certificate_is_root_bound() {
        let problem = problem(4);
        let suite = moving_suite();
        let mut proposal_budget = TransportBudget::default();
        let proof = synthesize_transport_refinement(
            &problem,
            &suite,
            TransportConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        let mut validation_budget = TransportBudget::default();
        let certificate = validate_transport_refinement(
            &problem,
            &suite,
            &proof,
            TransportConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        let mut foreign = TransportStateLanguage::new(5);
        assert!(matches!(
            foreign.admit_certificate(&certificate),
            Err(TransportError::ForeignCertificate { .. })
        ));
    }
}
