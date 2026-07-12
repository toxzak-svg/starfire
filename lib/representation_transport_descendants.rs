//! Proof-carrying descendants over transport-certified executable ancestors.
//!
//! ΩD1 composes the ΩR1 transport-certified ancestor state with a bounded
//! descendant grammar. A descendant terminal exists only when an ancestor
//! refinement is actually admitted to the current root-bound transport state.
//! Descendant certificates bind to the exact ancestor-state signature that made
//! their program executable.

use crate::commitment_state::Atom;
use crate::representation_genesis::{
    enumerate_programs, RawHistory, RefinementProblem, RefinementProgram, StateKey,
};
use crate::representation_transport::TransportStateLanguage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DescendantRelation {
    Equal,
    NotEqual,
}

impl DescendantRelation {
    fn all() -> [Self; 2] {
        [Self::Equal, Self::NotEqual]
    }

    fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TransportDescendantProgram {
    pub ancestor_index: usize,
    pub relation: DescendantRelation,
    pub raw_program: RefinementProgram,
}

impl TransportDescendantProgram {
    pub fn execute(
        &self,
        ancestor: &TransportStateLanguage,
        history: &RawHistory,
        intervention: &Atom,
    ) -> Result<bool, TransportDescendantError> {
        let key = ancestor.state_key(history, intervention);
        let ancestor_value = key
            .refinement_bits
            .get(self.ancestor_index)
            .copied()
            .ok_or(TransportDescendantError::MissingAncestorIndex {
                index: self.ancestor_index,
                available: key.refinement_bits.len(),
            })?;
        let raw_value = self.raw_program.execute(history);
        Ok(match self.relation {
            DescendantRelation::Equal => ancestor_value == raw_value,
            DescendantRelation::NotEqual => ancestor_value != raw_value,
        })
    }

    pub fn canonical_string(&self) -> String {
        format!(
            "bit[{}]{}{}",
            self.ancestor_index,
            self.relation.symbol(),
            self.raw_program.canonical_string()
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDescendantBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub raw_candidate_programs: usize,
    pub raw_program_history_evaluations: usize,
    pub ancestor_terminals: usize,
    pub descendant_candidate_programs: usize,
    pub descendant_program_history_evaluations: usize,
    pub unique_partitions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDescendantConfig {
    pub min_partition_support: usize,
    pub min_winner_margin: usize,
}

impl Default for TransportDescendantConfig {
    fn default() -> Self {
        Self {
            min_partition_support: 8,
            min_winner_margin: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawExpressibilityAudit {
    pub problem_digest: u64,
    pub candidate_program_count: usize,
    pub unique_partition_count: usize,
    pub opposite_outcome_pairs: usize,
    pub best_repaired_pairs: usize,
    pub complete_repair_exists: bool,
    pub canonical_best_program: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDescendantProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub ancestor_state_signature: String,
    pub ancestor_state_digest: u64,
    pub ancestor_index: usize,
    pub raw_candidate_program_count: usize,
    pub descendant_candidate_program_count: usize,
    pub unique_partition_count: usize,
    pub opposite_outcome_pairs: usize,
    pub program: TransportDescendantProgram,
    pub canonical_partition: Vec<bool>,
    pub repaired_pairs: usize,
    pub unrepaired_pairs: usize,
    pub partition_support_min: usize,
    pub runner_up_repaired_pairs: usize,
    pub winner_margin: usize,
}

/// Opaque certificate issued only after independent full descendant recomputation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedTransportDescendantCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    ancestor_state_signature: String,
    ancestor_index: usize,
    program: TransportDescendantProgram,
}

impl ValidatedTransportDescendantCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn ancestor_state_signature(&self) -> &str {
        &self.ancestor_state_signature
    }

    pub fn ancestor_index(&self) -> usize {
        self.ancestor_index
    }

    pub fn program(&self) -> &TransportDescendantProgram {
        &self.program
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedTransportDescendant {
    pub proof_id: u64,
    pub problem_digest: u64,
    pub ancestor_state_signature: String,
    pub program: TransportDescendantProgram,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TransportLayeredStateKey {
    pub ancestor_key: StateKey,
    pub descendant_bits: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDescendantStateLanguage {
    ancestor: TransportStateLanguage,
    descendants: Vec<AdmittedTransportDescendant>,
}

impl TransportDescendantStateLanguage {
    pub fn new(ancestor: TransportStateLanguage) -> Self {
        Self {
            ancestor,
            descendants: Vec::new(),
        }
    }

    pub fn root_id(&self) -> u64 {
        self.ancestor.root_id()
    }

    pub fn ancestor_language(&self) -> &TransportStateLanguage {
        &self.ancestor
    }

    pub fn descendant_count(&self) -> usize {
        self.descendants.len()
    }

    pub fn state_key(
        &self,
        history: &RawHistory,
        intervention: &Atom,
    ) -> Result<TransportLayeredStateKey, TransportDescendantError> {
        let ancestor_key = self.ancestor.state_key(history, intervention);
        let mut descendant_bits = Vec::with_capacity(self.descendants.len());
        for descendant in &self.descendants {
            descendant_bits.push(descendant.program.execute(
                &self.ancestor,
                history,
                intervention,
            )?);
        }
        Ok(TransportLayeredStateKey {
            ancestor_key,
            descendant_bits,
        })
    }

    pub fn admit_certificate(
        &mut self,
        certificate: &ValidatedTransportDescendantCertificate,
    ) -> Result<(), TransportDescendantError> {
        if certificate.root_id != self.root_id() {
            return Err(TransportDescendantError::ForeignCertificate {
                expected_root: self.root_id(),
                certificate_root: certificate.root_id,
            });
        }
        let current_signature = self.ancestor.canonical_signature();
        if certificate.ancestor_state_signature != current_signature {
            return Err(TransportDescendantError::AncestorStateMismatch);
        }
        if certificate.ancestor_index >= self.ancestor.refinement_count() {
            return Err(TransportDescendantError::MissingAncestorIndex {
                index: certificate.ancestor_index,
                available: self.ancestor.refinement_count(),
            });
        }
        if self
            .descendants
            .iter()
            .any(|existing| existing.proof_id == certificate.proof_id)
        {
            return Err(TransportDescendantError::DuplicateDescendant(
                certificate.proof_id,
            ));
        }
        self.descendants.push(AdmittedTransportDescendant {
            proof_id: certificate.proof_id,
            problem_digest: certificate.problem_digest,
            ancestor_state_signature: certificate.ancestor_state_signature.clone(),
            program: certificate.program.clone(),
        });
        self.verify_invariants()
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![format!("ancestor:{}", self.ancestor.canonical_signature())];
        for descendant in &self.descendants {
            parts.push(format!(
                "{}:{}:{}:{}",
                descendant.proof_id,
                descendant.problem_digest,
                stable_digest(descendant.ancestor_state_signature.as_bytes()),
                descendant.program.canonical_string()
            ));
        }
        parts.join("|")
    }

    pub fn verify_invariants(&self) -> Result<(), TransportDescendantError> {
        self.ancestor
            .verify_invariants()
            .map_err(|error| TransportDescendantError::InvariantViolation(error.to_string()))?;
        let current_signature = self.ancestor.canonical_signature();
        let mut proof_ids = BTreeSet::new();
        for descendant in &self.descendants {
            if !proof_ids.insert(descendant.proof_id) {
                return Err(TransportDescendantError::InvariantViolation(
                    "duplicate descendant proof id".to_string(),
                ));
            }
            if descendant.ancestor_state_signature != current_signature {
                return Err(TransportDescendantError::InvariantViolation(
                    "descendant is not bound to current ancestor state".to_string(),
                ));
            }
            if descendant.program.ancestor_index >= self.ancestor.refinement_count() {
                return Err(TransportDescendantError::InvariantViolation(
                    "descendant references a missing ancestor terminal".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportDescendantError {
    #[error("descendant problem has no discovery histories")]
    EmptyDiscovery,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("duplicate history id {0}")]
    DuplicateHistory(u64),
    #[error("problem root {problem_root} does not match ancestor root {ancestor_root}")]
    RootMismatch {
        problem_root: u64,
        ancestor_root: u64,
    },
    #[error("no admitted transport-certified ancestor exists; descendant language is empty")]
    NoAncestorRefinement,
    #[error("ancestor bit index {index} is unavailable; ancestor state has {available} bits")]
    MissingAncestorIndex { index: usize, available: usize },
    #[error("no opposite-outcome behavioral pairs exist")]
    NoBehavioralOpposition,
    #[error("descendant search produced no partitions")]
    NoCandidatePartition,
    #[error("descendant proof does not match independent recomputation")]
    ProofMismatch,
    #[error("descendant certificate gate failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign descendant certificate: expected root {expected_root}, got {certificate_root}")]
    ForeignCertificate {
        expected_root: u64,
        certificate_root: u64,
    },
    #[error("descendant certificate is bound to a different ancestor state")]
    AncestorStateMismatch,
    #[error("descendant proof id {0} is already admitted")]
    DuplicateDescendant(u64),
    #[error("transport-descendant state invariant violation: {0}")]
    InvariantViolation(String),
}

pub fn expected_descendant_candidate_programs(
    raw_programs: usize,
    ancestor_terminals: usize,
) -> usize {
    raw_programs
        .saturating_mul(ancestor_terminals)
        .saturating_mul(DescendantRelation::all().len())
}

pub fn audit_raw_expressibility(
    problem: &RefinementProblem,
    budget: &mut TransportDescendantBudget,
) -> Result<RawExpressibilityAudit, TransportDescendantError> {
    validate_problem(problem)?;
    let vocabulary = derive_vocabulary(problem, budget);
    let raw_programs = enumerate_programs(&vocabulary);
    budget.raw_candidate_programs = raw_programs.len();
    let opposite_pairs = opposite_outcome_pairs(problem, budget)?;

    let mut partitions = BTreeMap::<Vec<bool>, RefinementProgram>::new();
    for program in raw_programs {
        let mut bits = Vec::with_capacity(problem.discovery.len());
        for episode in &problem.discovery {
            budget.raw_program_history_evaluations =
                budget.raw_program_history_evaluations.saturating_add(1);
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
        return Err(TransportDescendantError::NoCandidatePartition);
    }

    let mut ranked = partitions
        .into_iter()
        .map(|(partition, program)| {
            let repaired = repaired_pairs(&partition, &opposite_pairs);
            let true_count = partition.iter().filter(|value| **value).count();
            let support_min = true_count.min(partition.len().saturating_sub(true_count));
            (repaired, support_min, program.canonical_string())
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let winner = ranked
        .first()
        .ok_or(TransportDescendantError::NoCandidatePartition)?;

    Ok(RawExpressibilityAudit {
        problem_digest: digest_problem(problem),
        candidate_program_count: budget.raw_candidate_programs,
        unique_partition_count: budget.unique_partitions,
        opposite_outcome_pairs: opposite_pairs.len(),
        best_repaired_pairs: winner.0,
        complete_repair_exists: winner.0 == opposite_pairs.len(),
        canonical_best_program: winner.2.clone(),
    })
}

pub fn synthesize_transport_descendant(
    problem: &RefinementProblem,
    ancestor: &TransportStateLanguage,
    config: TransportDescendantConfig,
    budget: &mut TransportDescendantBudget,
) -> Result<TransportDescendantProof, TransportDescendantError> {
    search(problem, ancestor, config, budget)
}

pub fn validate_transport_descendant(
    problem: &RefinementProblem,
    ancestor: &TransportStateLanguage,
    supplied: &TransportDescendantProof,
    config: TransportDescendantConfig,
    budget: &mut TransportDescendantBudget,
) -> Result<ValidatedTransportDescendantCertificate, TransportDescendantError> {
    let recomputed = search(problem, ancestor, config, budget)?;
    if supplied != &recomputed {
        return Err(TransportDescendantError::ProofMismatch);
    }
    enforce_certificate_gates(&recomputed, config)?;
    Ok(ValidatedTransportDescendantCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        ancestor_state_signature: recomputed.ancestor_state_signature,
        ancestor_index: recomputed.ancestor_index,
        program: recomputed.program,
    })
}

fn search(
    problem: &RefinementProblem,
    ancestor: &TransportStateLanguage,
    _config: TransportDescendantConfig,
    budget: &mut TransportDescendantBudget,
) -> Result<TransportDescendantProof, TransportDescendantError> {
    validate_problem(problem)?;
    if problem.root_id != ancestor.root_id() {
        return Err(TransportDescendantError::RootMismatch {
            problem_root: problem.root_id,
            ancestor_root: ancestor.root_id(),
        });
    }

    let vocabulary = derive_vocabulary(problem, budget);
    let raw_programs = enumerate_programs(&vocabulary);
    budget.raw_candidate_programs = raw_programs.len();
    budget.ancestor_terminals = ancestor.refinement_count();
    budget.descendant_candidate_programs = expected_descendant_candidate_programs(
        raw_programs.len(),
        ancestor.refinement_count(),
    );
    if ancestor.refinement_count() == 0 {
        return Err(TransportDescendantError::NoAncestorRefinement);
    }

    let opposite_pairs = opposite_outcome_pairs(problem, budget)?;
    let mut partitions = BTreeMap::<Vec<bool>, TransportDescendantProgram>::new();
    for ancestor_index in 0..ancestor.refinement_count() {
        for raw_program in &raw_programs {
            for relation in DescendantRelation::all() {
                let program = TransportDescendantProgram {
                    ancestor_index,
                    relation,
                    raw_program: raw_program.clone(),
                };
                let mut bits = Vec::with_capacity(problem.discovery.len());
                for episode in &problem.discovery {
                    budget.descendant_program_history_evaluations = budget
                        .descendant_program_history_evaluations
                        .saturating_add(1);
                    bits.push(program.execute(
                        ancestor,
                        &episode.history,
                        &episode.intervention,
                    )?);
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
        }
    }
    budget.unique_partitions = partitions.len();
    if partitions.is_empty() {
        return Err(TransportDescendantError::NoCandidatePartition);
    }

    let mut ranked = Vec::<(usize, usize, Vec<bool>, TransportDescendantProgram)>::new();
    for (partition, program) in partitions {
        let repaired = repaired_pairs(&partition, &opposite_pairs);
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
        .ok_or(TransportDescendantError::NoCandidatePartition)?;
    let runner_up = ranked.get(1).map(|entry| entry.0).unwrap_or(0);
    let winner_margin = winner.0.saturating_sub(runner_up);
    let unrepaired_pairs = opposite_pairs.len().saturating_sub(winner.0);
    let ancestor_state_signature = ancestor.canonical_signature();
    let ancestor_state_digest = stable_digest(ancestor_state_signature.as_bytes());
    let problem_digest = digest_problem(problem);
    let proof_id = digest_descendant_identity(
        problem.root_id,
        problem_digest,
        ancestor_state_digest,
        &winner.3,
        &winner.2,
    );

    Ok(TransportDescendantProof {
        proof_id,
        root_id: problem.root_id,
        problem_digest,
        ancestor_state_signature,
        ancestor_state_digest,
        ancestor_index: winner.3.ancestor_index,
        raw_candidate_program_count: budget.raw_candidate_programs,
        descendant_candidate_program_count: budget.descendant_candidate_programs,
        unique_partition_count: budget.unique_partitions,
        opposite_outcome_pairs: opposite_pairs.len(),
        program: winner.3.clone(),
        canonical_partition: winner.2.clone(),
        repaired_pairs: winner.0,
        unrepaired_pairs,
        partition_support_min: winner.1,
        runner_up_repaired_pairs: runner_up,
        winner_margin,
    })
}

fn enforce_certificate_gates(
    proof: &TransportDescendantProof,
    config: TransportDescendantConfig,
) -> Result<(), TransportDescendantError> {
    if proof.opposite_outcome_pairs == 0 {
        return Err(TransportDescendantError::CertificateGate(
            "no_behavioral_opposition",
        ));
    }
    if proof.repaired_pairs != proof.opposite_outcome_pairs || proof.unrepaired_pairs != 0 {
        return Err(TransportDescendantError::CertificateGate(
            "incomplete_behavioral_partition",
        ));
    }
    if proof.partition_support_min < config.min_partition_support {
        return Err(TransportDescendantError::CertificateGate(
            "insufficient_partition_support",
        ));
    }
    if proof.winner_margin < config.min_winner_margin {
        return Err(TransportDescendantError::CertificateGate(
            "insufficient_winner_margin",
        ));
    }
    Ok(())
}

fn validate_problem(problem: &RefinementProblem) -> Result<(), TransportDescendantError> {
    if problem.discovery.is_empty() {
        return Err(TransportDescendantError::EmptyDiscovery);
    }
    let mut evidence_ids = BTreeSet::new();
    let mut history_ids = BTreeSet::new();
    for episode in &problem.discovery {
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(TransportDescendantError::DuplicateEvidence(
                episode.evidence_id,
            ));
        }
        if !history_ids.insert(episode.history.history_id) {
            return Err(TransportDescendantError::DuplicateHistory(
                episode.history.history_id,
            ));
        }
    }
    Ok(())
}

fn derive_vocabulary(
    problem: &RefinementProblem,
    budget: &mut TransportDescendantBudget,
) -> Vec<Atom> {
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
    budget: &mut TransportDescendantBudget,
) -> Result<Vec<(usize, usize)>, TransportDescendantError> {
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
        return Err(TransportDescendantError::NoBehavioralOpposition);
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
    let complement = bits.iter().map(|value| !value).collect::<Vec<_>>();
    if complement < bits {
        complement
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
            bytes.push(0x61);
        }
        bytes.extend_from_slice(episode.intervention.as_str().as_bytes());
        bytes.push(0x62);
        bytes.extend_from_slice(episode.outcome.as_str().as_bytes());
        bytes.push(0x63);
    }
    stable_digest(&bytes)
}

fn digest_descendant_identity(
    root_id: u64,
    problem_digest: u64,
    ancestor_state_digest: u64,
    program: &TransportDescendantProgram,
    partition: &[bool],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(&ancestor_state_digest.to_le_bytes());
    bytes.extend_from_slice(program.canonical_string().as_bytes());
    for bit in partition {
        bytes.push(u8::from(*bit));
    }
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
    use crate::representation_genesis::WitnessedHistory;
    use crate::representation_transport::{
        synthesize_transport_refinement, validate_transport_refinement, BlockPermutation,
        CorrespondenceMode, TransformationSuite, TransportBudget, TransportConfig,
    };

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn history(history_id: u64, bits: [bool; 4]) -> RawHistory {
        let atoms = [
            atom("A"), atom("B"), atom("C"), atom("D"),
            atom("E"), atom("F"), atom("G"), atom("H"),
        ];
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
        RawHistory { history_id, events }
    }

    fn problems(root_id: u64) -> (RefinementProblem, RefinementProblem, RefinementProblem) {
        let mut correct = Vec::new();
        let mut wrong = Vec::new();
        let mut stage2 = Vec::new();
        let mut index = 0_u64;
        for p in [false, true] {
            for q in [false, true] {
                for r in [false, true] {
                    for s in [false, true] {
                        let history = history(100 + index, [p, q, r, s]);
                        correct.push(WitnessedHistory {
                            evidence_id: 1_000 + index,
                            history: history.clone(),
                            intervention: atom("correct"),
                            outcome: atom(if p { "p1" } else { "p0" }),
                        });
                        wrong.push(WitnessedHistory {
                            evidence_id: 2_000 + index,
                            history: history.clone(),
                            intervention: atom("wrong"),
                            outcome: atom(if s { "s1" } else { "s0" }),
                        });
                        stage2.push(WitnessedHistory {
                            evidence_id: 3_000 + index,
                            history,
                            intervention: atom("stage2"),
                            outcome: atom(if p != r { "z1" } else { "z0" }),
                        });
                        index += 1;
                    }
                }
            }
        }
        (
            RefinementProblem { root_id, discovery: correct },
            RefinementProblem { root_id, discovery: wrong },
            RefinementProblem { root_id, discovery: stage2 },
        )
    }

    fn correct_suite() -> TransformationSuite {
        TransformationSuite {
            transformations: vec![
                BlockPermutation { block_width: 2, block_order: vec![1,0,2,3] },
                BlockPermutation { block_width: 2, block_order: vec![1,2,0,3] },
            ],
            correspondence: CorrespondenceMode::SameHistory,
        }
    }

    fn wrong_suite() -> TransformationSuite {
        TransformationSuite {
            transformations: vec![
                BlockPermutation { block_width: 2, block_order: vec![0,3,1,2] },
                BlockPermutation { block_width: 2, block_order: vec![0,1,3,2] },
            ],
            correspondence: CorrespondenceMode::SameHistory,
        }
    }

    fn stationary_suite() -> TransformationSuite {
        TransformationSuite {
            transformations: vec![
                BlockPermutation { block_width: 2, block_order: vec![0,2,1,3] },
                BlockPermutation { block_width: 2, block_order: vec![0,3,2,1] },
            ],
            correspondence: CorrespondenceMode::SameHistory,
        }
    }

    fn admitted_ancestor(problem: &RefinementProblem, suite: &TransformationSuite) -> TransportStateLanguage {
        let mut proposal_budget = TransportBudget::default();
        let proof = synthesize_transport_refinement(
            problem,
            suite,
            TransportConfig::default(),
            &mut proposal_budget,
        ).unwrap();
        let mut validation_budget = TransportBudget::default();
        let certificate = validate_transport_refinement(
            problem,
            suite,
            &proof,
            TransportConfig::default(),
            &mut validation_budget,
        ).unwrap();
        let mut state = TransportStateLanguage::new(problem.root_id);
        state.admit_certificate(&certificate).unwrap();
        state
    }

    #[test]
    fn raw_l0_cannot_express_stage2_parity() {
        let (_, _, stage2) = problems(1);
        let mut budget = TransportDescendantBudget::default();
        let audit = audit_raw_expressibility(&stage2, &mut budget).unwrap();
        assert_eq!(audit.candidate_program_count, 828);
        assert_eq!(audit.unique_partition_count, 5);
        assert_eq!(audit.opposite_outcome_pairs, 64);
        assert_eq!(audit.best_repaired_pairs, 32);
        assert!(!audit.complete_repair_exists);
        assert_eq!(budget.history_pair_evaluations, 120);
        assert_eq!(budget.raw_program_history_evaluations, 13_248);
    }

    #[test]
    fn l0_has_no_descendant_frontier() {
        let (_, _, stage2) = problems(2);
        let ancestor = TransportStateLanguage::new(2);
        let mut budget = TransportDescendantBudget::default();
        let error = synthesize_transport_descendant(
            &stage2,
            &ancestor,
            TransportDescendantConfig::default(),
            &mut budget,
        ).unwrap_err();
        assert_eq!(error, TransportDescendantError::NoAncestorRefinement);
        assert_eq!(budget.raw_candidate_programs, 828);
        assert_eq!(budget.ancestor_terminals, 0);
        assert_eq!(budget.descendant_candidate_programs, 0);
        assert_eq!(budget.descendant_program_history_evaluations, 0);
    }

    #[test]
    fn stable_correct_ancestor_creates_exact_descendant_frontier() {
        let (correct, _, stage2) = problems(3);
        let ancestor = admitted_ancestor(&correct, &correct_suite());
        let mut proposal_budget = TransportDescendantBudget::default();
        let proof = synthesize_transport_descendant(
            &stage2,
            &ancestor,
            TransportDescendantConfig::default(),
            &mut proposal_budget,
        ).unwrap();
        assert_eq!(proof.raw_candidate_program_count, 828);
        assert_eq!(proof.descendant_candidate_program_count, 1_656);
        assert_eq!(proof.unique_partition_count, 5);
        assert_eq!(proof.opposite_outcome_pairs, 64);
        assert_eq!(proof.repaired_pairs, 64);
        assert_eq!(proof.runner_up_repaired_pairs, 32);
        assert_eq!(proof.winner_margin, 32);
        assert_eq!(proof.partition_support_min, 8);
        assert_eq!(proposal_budget.history_pair_evaluations, 120);
        assert_eq!(proposal_budget.descendant_program_history_evaluations, 26_496);

        let mut validation_budget = TransportDescendantBudget::default();
        let certificate = validate_transport_descendant(
            &stage2,
            &ancestor,
            &proof,
            TransportDescendantConfig::default(),
            &mut validation_budget,
        ).unwrap();
        let mut layered = TransportDescendantStateLanguage::new(ancestor);
        layered.admit_certificate(&certificate).unwrap();
        assert_eq!(layered.descendant_count(), 1);
    }

    #[test]
    fn wrong_transport_certified_ancestor_cannot_validate_descendant() {
        let (_, wrong, stage2) = problems(4);
        let ancestor = admitted_ancestor(&wrong, &wrong_suite());
        let mut proposal_budget = TransportDescendantBudget::default();
        let proof = synthesize_transport_descendant(
            &stage2,
            &ancestor,
            TransportDescendantConfig::default(),
            &mut proposal_budget,
        ).unwrap();
        assert_eq!(proof.descendant_candidate_program_count, 1_656);
        assert_eq!(proof.repaired_pairs, 32);
        let mut validation_budget = TransportDescendantBudget::default();
        let error = validate_transport_descendant(
            &stage2,
            &ancestor,
            &proof,
            TransportDescendantConfig::default(),
            &mut validation_budget,
        ).unwrap_err();
        assert_eq!(
            error,
            TransportDescendantError::CertificateGate("incomplete_behavioral_partition")
        );
        assert_eq!(validation_budget.descendant_program_history_evaluations, 26_496);
    }

    #[test]
    fn exact_ancestor_replacement_invalidates_proposed_descendant() {
        let (correct, wrong, stage2) = problems(5);
        let correct_ancestor = admitted_ancestor(&correct, &correct_suite());
        let wrong_ancestor = admitted_ancestor(&wrong, &wrong_suite());
        let mut proposal_budget = TransportDescendantBudget::default();
        let proof = synthesize_transport_descendant(
            &stage2,
            &correct_ancestor,
            TransportDescendantConfig::default(),
            &mut proposal_budget,
        ).unwrap();
        let mut validation_budget = TransportDescendantBudget::default();
        let error = validate_transport_descendant(
            &stage2,
            &wrong_ancestor,
            &proof,
            TransportDescendantConfig::default(),
            &mut validation_budget,
        ).unwrap_err();
        assert_eq!(error, TransportDescendantError::ProofMismatch);
        assert_eq!(validation_budget.descendant_candidate_programs, 1_656);
        assert_eq!(validation_budget.descendant_program_history_evaluations, 26_496);
    }

    #[test]
    fn stationary_ancestor_supports_discovery_descendant_validation() {
        let (correct, _, stage2) = problems(6);
        let ancestor = admitted_ancestor(&correct, &stationary_suite());
        let mut proposal_budget = TransportDescendantBudget::default();
        let proof = synthesize_transport_descendant(
            &stage2,
            &ancestor,
            TransportDescendantConfig::default(),
            &mut proposal_budget,
        ).unwrap();
        assert_eq!(proof.repaired_pairs, 64);
        let mut validation_budget = TransportDescendantBudget::default();
        assert!(validate_transport_descendant(
            &stage2,
            &ancestor,
            &proof,
            TransportDescendantConfig::default(),
            &mut validation_budget,
        ).is_ok());
    }
}
