//! Proof-carrying descendant representation refinement for the Ω2 shadow experiment.
//!
//! Ω1 creates executable raw-history refinements. Ω2 adds a bounded descendant
//! grammar whose terminals are generated only from refinements that are actually
//! admitted to the current root-bound `StateLanguage`. A descendant certificate
//! is bound to the exact ancestor-language signature that made its program
//! expressible.

use crate::commitment_state::Atom;
use crate::representation_genesis::{
    enumerate_programs, RawHistory, RefinementProblem, RefinementProgram, StateKey, StateLanguage,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DescendantRelation {
    Equal,
    NotEqual,
}

impl DescendantRelation {
    fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
        }
    }

    fn all() -> [Self; 2] {
        [Self::Equal, Self::NotEqual]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DescendantProgram {
    pub ancestor_index: usize,
    pub relation: DescendantRelation,
    pub raw_program: RefinementProgram,
}

impl DescendantProgram {
    pub fn execute(
        &self,
        language: &StateLanguage,
        history: &RawHistory,
        intervention: &Atom,
    ) -> Result<bool, DescendantGenesisError> {
        let key = language.state_key(history, intervention);
        let ancestor = key
            .refinement_bits
            .get(self.ancestor_index)
            .copied()
            .ok_or(DescendantGenesisError::MissingAncestorIndex {
                index: self.ancestor_index,
                available: key.refinement_bits.len(),
            })?;
        let raw = self.raw_program.execute(history);
        Ok(match self.relation {
            DescendantRelation::Equal => ancestor == raw,
            DescendantRelation::NotEqual => ancestor != raw,
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
pub struct DescendantBudget {
    pub vocabulary_history_scans: usize,
    pub opposite_outcome_pair_checks: usize,
    pub raw_candidate_programs: usize,
    pub raw_program_history_evaluations: usize,
    pub ancestor_terminals: usize,
    pub descendant_candidate_programs: usize,
    pub descendant_program_history_evaluations: usize,
    pub unique_partitions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescendantConfig {
    pub min_partition_support: usize,
    pub min_winner_margin: usize,
}

impl Default for DescendantConfig {
    fn default() -> Self {
        Self {
            min_partition_support: 4,
            min_winner_margin: 4,
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
pub struct DescendantProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub ancestor_language_signature: String,
    pub ancestor_language_digest: u64,
    pub ancestor_index: usize,
    pub raw_candidate_program_count: usize,
    pub descendant_candidate_program_count: usize,
    pub unique_partition_count: usize,
    pub opposite_outcome_pairs: usize,
    pub program: DescendantProgram,
    pub canonical_partition: Vec<bool>,
    pub repaired_pairs: usize,
    pub unrepaired_pairs: usize,
    pub partition_support_min: usize,
    pub runner_up_repaired_pairs: usize,
    pub winner_margin: usize,
}

/// Opaque certificate produced only by independent full recomputation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedDescendantCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    ancestor_language_signature: String,
    ancestor_index: usize,
    program: DescendantProgram,
}

impl ValidatedDescendantCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn ancestor_language_signature(&self) -> &str {
        &self.ancestor_language_signature
    }

    pub fn ancestor_index(&self) -> usize {
        self.ancestor_index
    }

    pub fn program(&self) -> &DescendantProgram {
        &self.program
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedDescendantRefinement {
    pub proof_id: u64,
    pub problem_digest: u64,
    pub ancestor_language_signature: String,
    pub program: DescendantProgram,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayeredStateKey {
    pub ancestor_key: StateKey,
    pub descendant_bits: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescendantStateLanguage {
    base: StateLanguage,
    descendants: Vec<AdmittedDescendantRefinement>,
}

impl DescendantStateLanguage {
    pub fn new(base: StateLanguage) -> Self {
        Self {
            base,
            descendants: Vec::new(),
        }
    }

    pub fn root_id(&self) -> u64 {
        self.base.root_id()
    }

    pub fn base_language(&self) -> &StateLanguage {
        &self.base
    }

    pub fn descendant_count(&self) -> usize {
        self.descendants.len()
    }

    pub fn state_key(
        &self,
        history: &RawHistory,
        intervention: &Atom,
    ) -> Result<LayeredStateKey, DescendantGenesisError> {
        let ancestor_key = self.base.state_key(history, intervention);
        let mut descendant_bits = Vec::with_capacity(self.descendants.len());
        for descendant in &self.descendants {
            descendant_bits.push(descendant.program.execute(
                &self.base,
                history,
                intervention,
            )?);
        }
        Ok(LayeredStateKey {
            ancestor_key,
            descendant_bits,
        })
    }

    pub fn admit_certificate(
        &mut self,
        certificate: &ValidatedDescendantCertificate,
    ) -> Result<(), DescendantGenesisError> {
        if certificate.root_id != self.root_id() {
            return Err(DescendantGenesisError::ForeignCertificate {
                expected_root: self.root_id(),
                certificate_root: certificate.root_id,
            });
        }
        let current_signature = self.base.canonical_signature();
        if certificate.ancestor_language_signature != current_signature {
            return Err(DescendantGenesisError::AncestorLanguageMismatch);
        }
        if certificate.ancestor_index >= self.base.refinement_count() {
            return Err(DescendantGenesisError::MissingAncestorIndex {
                index: certificate.ancestor_index,
                available: self.base.refinement_count(),
            });
        }
        if self
            .descendants
            .iter()
            .any(|existing| existing.proof_id == certificate.proof_id)
        {
            return Err(DescendantGenesisError::DuplicateDescendant(
                certificate.proof_id,
            ));
        }
        self.descendants.push(AdmittedDescendantRefinement {
            proof_id: certificate.proof_id,
            problem_digest: certificate.problem_digest,
            ancestor_language_signature: certificate.ancestor_language_signature.clone(),
            program: certificate.program.clone(),
        });
        self.verify_invariants()
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![format!("base:{}", self.base.canonical_signature())];
        for descendant in &self.descendants {
            parts.push(format!(
                "{}:{}:{}:{}",
                descendant.proof_id,
                descendant.problem_digest,
                stable_digest(descendant.ancestor_language_signature.as_bytes()),
                descendant.program.canonical_string()
            ));
        }
        parts.join("|")
    }

    pub fn verify_invariants(&self) -> Result<(), DescendantGenesisError> {
        self.base
            .verify_invariants()
            .map_err(|error| DescendantGenesisError::InvariantViolation(error.to_string()))?;
        let current_signature = self.base.canonical_signature();
        let mut proof_ids = BTreeSet::new();
        for descendant in &self.descendants {
            if !proof_ids.insert(descendant.proof_id) {
                return Err(DescendantGenesisError::InvariantViolation(
                    "duplicate descendant proof id".to_string(),
                ));
            }
            if descendant.ancestor_language_signature != current_signature {
                return Err(DescendantGenesisError::InvariantViolation(
                    "admitted descendant is not bound to current ancestor language".to_string(),
                ));
            }
            if descendant.program.ancestor_index >= self.base.refinement_count() {
                return Err(DescendantGenesisError::InvariantViolation(
                    "admitted descendant references a missing ancestor bit".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DescendantGenesisError {
    #[error("descendant problem has no discovery histories")]
    EmptyDiscovery,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("duplicate history id {0}")]
    DuplicateHistory(u64),
    #[error("problem root {problem_root} does not match language root {language_root}")]
    RootMismatch {
        problem_root: u64,
        language_root: u64,
    },
    #[error("no admitted ancestor refinement exists; descendant hypothesis language is empty")]
    NoAncestorRefinement,
    #[error("ancestor bit index {index} is unavailable; language has {available} bits")]
    MissingAncestorIndex { index: usize, available: usize },
    #[error("no opposite-outcome behavioral pairs exist")]
    NoBehavioralOpposition,
    #[error("descendant candidate search produced no partitions")]
    NoCandidatePartition,
    #[error("descendant proof does not match independent recomputation")]
    ProofMismatch,
    #[error("descendant certificate gates failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign descendant certificate: expected root {expected_root}, got {certificate_root}")]
    ForeignCertificate {
        expected_root: u64,
        certificate_root: u64,
    },
    #[error("descendant certificate is bound to a different ancestor language")]
    AncestorLanguageMismatch,
    #[error("descendant proof id {0} is already admitted")]
    DuplicateDescendant(u64),
    #[error("descendant-state invariant violation: {0}")]
    InvariantViolation(String),
}

pub fn expected_descendant_candidate_programs(
    raw_program_count: usize,
    ancestor_terminals: usize,
) -> usize {
    raw_program_count
        .saturating_mul(ancestor_terminals)
        .saturating_mul(DescendantRelation::all().len())
}

pub fn audit_raw_expressibility(
    problem: &RefinementProblem,
    budget: &mut DescendantBudget,
) -> Result<RawExpressibilityAudit, DescendantGenesisError> {
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
        return Err(DescendantGenesisError::NoCandidatePartition);
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
        .ok_or(DescendantGenesisError::NoCandidatePartition)?;

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

pub fn synthesize_descendant(
    problem: &RefinementProblem,
    language: &StateLanguage,
    config: DescendantConfig,
    budget: &mut DescendantBudget,
) -> Result<DescendantProof, DescendantGenesisError> {
    search(problem, language, config, budget)
}

pub fn validate_descendant(
    problem: &RefinementProblem,
    language: &StateLanguage,
    supplied: &DescendantProof,
    config: DescendantConfig,
    budget: &mut DescendantBudget,
) -> Result<ValidatedDescendantCertificate, DescendantGenesisError> {
    let recomputed = search(problem, language, config, budget)?;
    if supplied != &recomputed {
        return Err(DescendantGenesisError::ProofMismatch);
    }
    enforce_certificate_gates(&recomputed, config)?;
    Ok(ValidatedDescendantCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        ancestor_language_signature: recomputed.ancestor_language_signature,
        ancestor_index: recomputed.ancestor_index,
        program: recomputed.program,
    })
}

fn search(
    problem: &RefinementProblem,
    language: &StateLanguage,
    _config: DescendantConfig,
    budget: &mut DescendantBudget,
) -> Result<DescendantProof, DescendantGenesisError> {
    validate_problem(problem)?;
    if problem.root_id != language.root_id() {
        return Err(DescendantGenesisError::RootMismatch {
            problem_root: problem.root_id,
            language_root: language.root_id(),
        });
    }

    let vocabulary = derive_vocabulary(problem, budget);
    let raw_programs = enumerate_programs(&vocabulary);
    budget.raw_candidate_programs = raw_programs.len();
    budget.ancestor_terminals = language.refinement_count();
    budget.descendant_candidate_programs = expected_descendant_candidate_programs(
        raw_programs.len(),
        language.refinement_count(),
    );
    if language.refinement_count() == 0 {
        return Err(DescendantGenesisError::NoAncestorRefinement);
    }

    let opposite_pairs = opposite_outcome_pairs(problem, budget)?;
    let mut partitions = BTreeMap::<Vec<bool>, DescendantProgram>::new();
    for ancestor_index in 0..language.refinement_count() {
        for raw_program in &raw_programs {
            for relation in DescendantRelation::all() {
                let program = DescendantProgram {
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
                        language,
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
        return Err(DescendantGenesisError::NoCandidatePartition);
    }

    let mut ranked = Vec::<(usize, usize, Vec<bool>, DescendantProgram)>::new();
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
        .ok_or(DescendantGenesisError::NoCandidatePartition)?;
    let runner_up = ranked.get(1).map(|entry| entry.0).unwrap_or(0);
    let winner_margin = winner.0.saturating_sub(runner_up);
    let unrepaired_pairs = opposite_pairs.len().saturating_sub(winner.0);
    let ancestor_language_signature = language.canonical_signature();
    let ancestor_language_digest = stable_digest(ancestor_language_signature.as_bytes());
    let problem_digest = digest_problem(problem);
    let proof_id = digest_descendant_identity(
        problem.root_id,
        problem_digest,
        ancestor_language_digest,
        &winner.3,
        &winner.2,
    );

    Ok(DescendantProof {
        proof_id,
        root_id: problem.root_id,
        problem_digest,
        ancestor_language_signature,
        ancestor_language_digest,
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
    proof: &DescendantProof,
    config: DescendantConfig,
) -> Result<(), DescendantGenesisError> {
    if proof.opposite_outcome_pairs == 0 {
        return Err(DescendantGenesisError::CertificateGate(
            "no_behavioral_opposition",
        ));
    }
    if proof.repaired_pairs != proof.opposite_outcome_pairs || proof.unrepaired_pairs != 0 {
        return Err(DescendantGenesisError::CertificateGate(
            "incomplete_behavioral_partition",
        ));
    }
    if proof.partition_support_min < config.min_partition_support {
        return Err(DescendantGenesisError::CertificateGate(
            "insufficient_partition_support",
        ));
    }
    if proof.winner_margin < config.min_winner_margin {
        return Err(DescendantGenesisError::CertificateGate(
            "insufficient_winner_margin",
        ));
    }
    Ok(())
}

fn validate_problem(problem: &RefinementProblem) -> Result<(), DescendantGenesisError> {
    if problem.discovery.is_empty() {
        return Err(DescendantGenesisError::EmptyDiscovery);
    }
    let mut evidence_ids = BTreeSet::new();
    let mut history_ids = BTreeSet::new();
    for episode in &problem.discovery {
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(DescendantGenesisError::DuplicateEvidence(
                episode.evidence_id,
            ));
        }
        if !history_ids.insert(episode.history.history_id) {
            return Err(DescendantGenesisError::DuplicateHistory(
                episode.history.history_id,
            ));
        }
    }
    Ok(())
}

fn derive_vocabulary(problem: &RefinementProblem, budget: &mut DescendantBudget) -> Vec<Atom> {
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
    budget: &mut DescendantBudget,
) -> Result<Vec<(usize, usize)>, DescendantGenesisError> {
    let mut pairs = Vec::new();
    for left in 0..problem.discovery.len() {
        for right in (left + 1)..problem.discovery.len() {
            let left_episode = &problem.discovery[left];
            let right_episode = &problem.discovery[right];
            if left_episode.intervention == right_episode.intervention
                && left_episode.outcome != right_episode.outcome
            {
                budget.opposite_outcome_pair_checks =
                    budget.opposite_outcome_pair_checks.saturating_add(1);
                pairs.push((left, right));
            }
        }
    }
    if pairs.is_empty() {
        return Err(DescendantGenesisError::NoBehavioralOpposition);
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
            bytes.push(0xff);
        }
        bytes.extend_from_slice(episode.intervention.as_str().as_bytes());
        bytes.push(0xfe);
        bytes.extend_from_slice(episode.outcome.as_str().as_bytes());
        bytes.push(0xfd);
    }
    stable_digest(&bytes)
}

fn digest_descendant_identity(
    root_id: u64,
    problem_digest: u64,
    ancestor_language_digest: u64,
    program: &DescendantProgram,
    partition: &[bool],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(&ancestor_language_digest.to_le_bytes());
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
    use crate::representation_genesis::{
        synthesize_refinement, validate_refinement, GenesisBudget, RefinementConfig,
        WitnessedHistory,
    };

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn cube_history(history_id: u64, p: bool, q: bool, r: bool) -> RawHistory {
        let pairs = [
            (atom("A"), atom("B"), p),
            (atom("C"), atom("D"), q),
            (atom("E"), atom("F"), r),
        ];
        let mut events = Vec::new();
        for (left, right, bit) in pairs {
            if bit {
                events.push(left);
                events.push(right);
            } else {
                events.push(right);
                events.push(left);
            }
        }
        RawHistory { history_id, events }
    }

    fn problems(root_id: u64) -> (RefinementProblem, RefinementProblem, RefinementProblem) {
        let mut ancestor = Vec::new();
        let mut wrong = Vec::new();
        let mut descendant = Vec::new();
        let mut index = 0_u64;
        for p in [false, true] {
            for q in [false, true] {
                for r in [false, true] {
                    let history = cube_history(100 + index, p, q, r);
                    ancestor.push(WitnessedHistory {
                        evidence_id: 1_000 + index,
                        history: history.clone(),
                        intervention: atom("stage1"),
                        outcome: atom(if p { "p1" } else { "p0" }),
                    });
                    wrong.push(WitnessedHistory {
                        evidence_id: 2_000 + index,
                        history: history.clone(),
                        intervention: atom("wrong"),
                        outcome: atom(if r { "r1" } else { "r0" }),
                    });
                    descendant.push(WitnessedHistory {
                        evidence_id: 3_000 + index,
                        history,
                        intervention: atom("stage2"),
                        outcome: atom(if p == q { "z1" } else { "z0" }),
                    });
                    index += 1;
                }
            }
        }
        (
            RefinementProblem {
                root_id,
                discovery: ancestor,
            },
            RefinementProblem {
                root_id,
                discovery: wrong,
            },
            RefinementProblem {
                root_id,
                discovery: descendant,
            },
        )
    }

    fn admitted_language(problem: &RefinementProblem) -> StateLanguage {
        let config = RefinementConfig::default();
        let mut proposal_budget = GenesisBudget::default();
        let proof = synthesize_refinement(problem, config, &mut proposal_budget).unwrap();
        let mut validation_budget = GenesisBudget::default();
        let certificate =
            validate_refinement(problem, &proof, config, &mut validation_budget).unwrap();
        let mut language = StateLanguage::new(problem.root_id);
        language.admit_certificate(&certificate).unwrap();
        language
    }

    #[test]
    fn l0_has_no_descendant_candidates() {
        let (_, _, descendant) = problems(7);
        let language = StateLanguage::new(7);
        let mut budget = DescendantBudget::default();
        let error = synthesize_descendant(
            &descendant,
            &language,
            DescendantConfig::default(),
            &mut budget,
        )
        .unwrap_err();
        assert_eq!(error, DescendantGenesisError::NoAncestorRefinement);
        assert_eq!(budget.raw_candidate_programs, 459);
        assert_eq!(budget.ancestor_terminals, 0);
        assert_eq!(budget.descendant_candidate_programs, 0);
    }

    #[test]
    fn raw_l0_language_cannot_express_stage2_partition() {
        let (_, _, descendant) = problems(8);
        let mut budget = DescendantBudget::default();
        let audit = audit_raw_expressibility(&descendant, &mut budget).unwrap();
        assert_eq!(audit.candidate_program_count, 459);
        assert_eq!(audit.unique_partition_count, 4);
        assert_eq!(audit.opposite_outcome_pairs, 16);
        assert_eq!(audit.best_repaired_pairs, 8);
        assert!(!audit.complete_repair_exists);
        assert_eq!(budget.raw_program_history_evaluations, 3_672);
    }

    #[test]
    fn correct_ancestor_creates_exact_descendant_frontier() {
        let (ancestor, _, descendant) = problems(9);
        let language = admitted_language(&ancestor);
        let mut proposal_budget = DescendantBudget::default();
        let proof = synthesize_descendant(
            &descendant,
            &language,
            DescendantConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        assert_eq!(proof.raw_candidate_program_count, 459);
        assert_eq!(proof.descendant_candidate_program_count, 918);
        assert_eq!(proof.unique_partition_count, 4);
        assert_eq!(proof.opposite_outcome_pairs, 16);
        assert_eq!(proof.repaired_pairs, 16);
        assert_eq!(proof.runner_up_repaired_pairs, 8);
        assert_eq!(proof.winner_margin, 8);
        assert_eq!(proof.partition_support_min, 4);
        assert_eq!(proposal_budget.descendant_program_history_evaluations, 7_344);

        let mut validation_budget = DescendantBudget::default();
        let certificate = validate_descendant(
            &descendant,
            &language,
            &proof,
            DescendantConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        let mut layered = DescendantStateLanguage::new(language);
        layered.admit_certificate(&certificate).unwrap();
        assert_eq!(layered.descendant_count(), 1);
        assert!(layered.verify_invariants().is_ok());
    }

    #[test]
    fn wrong_valid_ancestor_cannot_validate_descendant() {
        let (_, wrong, descendant) = problems(10);
        let language = admitted_language(&wrong);
        let mut proposal_budget = DescendantBudget::default();
        let proof = synthesize_descendant(
            &descendant,
            &language,
            DescendantConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        assert_eq!(proof.descendant_candidate_program_count, 918);
        assert_eq!(proof.repaired_pairs, 8);
        assert_eq!(proposal_budget.descendant_program_history_evaluations, 7_344);
        let mut validation_budget = DescendantBudget::default();
        let error = validate_descendant(
            &descendant,
            &language,
            &proof,
            DescendantConfig::default(),
            &mut validation_budget,
        )
        .unwrap_err();
        assert_eq!(
            error,
            DescendantGenesisError::CertificateGate("incomplete_behavioral_partition")
        );
        assert_eq!(validation_budget.descendant_program_history_evaluations, 7_344);
    }

    #[test]
    fn replacing_exact_ancestor_invalidates_previously_proposed_descendant() {
        let (ancestor, wrong, descendant) = problems(11);
        let correct_language = admitted_language(&ancestor);
        let wrong_language = admitted_language(&wrong);
        let mut proposal_budget = DescendantBudget::default();
        let proof = synthesize_descendant(
            &descendant,
            &correct_language,
            DescendantConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        let mut validation_budget = DescendantBudget::default();
        let error = validate_descendant(
            &descendant,
            &wrong_language,
            &proof,
            DescendantConfig::default(),
            &mut validation_budget,
        )
        .unwrap_err();
        assert_eq!(error, DescendantGenesisError::ProofMismatch);
        assert_eq!(validation_budget.descendant_candidate_programs, 918);
        assert_eq!(validation_budget.descendant_program_history_evaluations, 7_344);
    }
}
