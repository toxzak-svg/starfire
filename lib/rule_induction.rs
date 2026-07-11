//! Evidence-bound deterministic rule induction for the H10 shadow experiment.
//!
//! This module deliberately separates proposal from validation. The proposer
//! scores every candidate rule from intervention evidence. The validator then
//! recomputes the complete ranking from the raw evidence before returning an
//! opaque certificate. Only a validated certificate can be admitted to the H10
//! executable-state wrapper.

use crate::commitment_state::{
    Atom, CommitmentId, CommitmentStateError, ExecutableCommitmentState, Rule, StateDelta,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceEpisode {
    pub evidence_id: u64,
    pub intervention: Atom,
    pub outcomes: BTreeSet<Atom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceProblem {
    pub antecedents: Vec<Atom>,
    pub consequents: Vec<Atom>,
    pub evidence: Vec<EvidenceEpisode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleInductionConfig {
    pub min_score: i32,
    pub min_support: usize,
    pub max_contradictions: usize,
    pub min_margin: i32,
}

impl Default for RuleInductionConfig {
    fn default() -> Self {
        Self {
            min_score: 10,
            min_support: 4,
            max_contradictions: 0,
            min_margin: 6,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoringBudget {
    pub candidate_episode_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleInferenceProof {
    pub proof_id: u64,
    pub rule: Rule,
    pub score: i32,
    pub support: usize,
    pub contradictions: usize,
    pub runner_up_score: i32,
    pub supporting_evidence_ids: Vec<u64>,
    pub contradicting_evidence_ids: Vec<u64>,
}

/// Opaque outside this module: callers can inspect a certificate but cannot
/// construct one without passing `validate_rule_inference`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedInferenceCertificate {
    proof_id: u64,
    rule: Rule,
    score: i32,
    runner_up_score: i32,
    evidence_ids: Vec<u64>,
}

impl ValidatedInferenceCertificate {
    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn runner_up_score(&self) -> i32 {
        self.runner_up_score
    }

    pub fn evidence_ids(&self) -> &[u64] {
        &self.evidence_ids
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuleInductionError {
    #[error("candidate antecedent universe is empty")]
    EmptyAntecedents,
    #[error("candidate consequent universe is empty")]
    EmptyConsequents,
    #[error("evidence table is empty")]
    EmptyEvidence,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("duplicate candidate atom in {0}")]
    DuplicateCandidate(&'static str),
    #[error("candidate universe contains a degenerate self-loop")]
    DegenerateCandidate,
    #[error("proof rule is not in the candidate universe")]
    RuleOutsideUniverse,
    #[error("candidate ranking does not have a unique winner")]
    NonUniqueWinner,
    #[error("proof does not match independent recomputation: {0}")]
    ProofMismatch(&'static str),
    #[error("winner score {actual} is below required {required}")]
    ScoreGate { actual: i32, required: i32 },
    #[error("winner support {actual} is below required {required}")]
    SupportGate { actual: usize, required: usize },
    #[error("winner contradictions {actual} exceed maximum {maximum}")]
    ContradictionGate { actual: usize, maximum: usize },
    #[error("winner margin {actual} is below required {required}")]
    MarginGate { actual: i32, required: i32 },
    #[error("commitment state rejected inferred certificate: {0}")]
    Commitment(String),
    #[error("inference provenance invariant violation: {0}")]
    InferenceInvariant(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateScore {
    rule: Rule,
    score: i32,
    support: usize,
    contradictions: usize,
    supporting_evidence_ids: Vec<u64>,
    contradicting_evidence_ids: Vec<u64>,
}

pub fn infer_rule(
    problem: &InferenceProblem,
    config: RuleInductionConfig,
    budget: &mut ScoringBudget,
) -> Result<RuleInferenceProof, RuleInductionError> {
    let ranking = score_all(problem, budget)?;
    let winner = &ranking[0];
    let runner_up = ranking.get(1).ok_or(RuleInductionError::NonUniqueWinner)?;
    enforce_gates(winner, runner_up, config)?;

    let proof_id = proof_digest(problem, winner, runner_up.score);
    Ok(RuleInferenceProof {
        proof_id,
        rule: winner.rule.clone(),
        score: winner.score,
        support: winner.support,
        contradictions: winner.contradictions,
        runner_up_score: runner_up.score,
        supporting_evidence_ids: winner.supporting_evidence_ids.clone(),
        contradicting_evidence_ids: winner.contradicting_evidence_ids.clone(),
    })
}

pub fn validate_rule_inference(
    problem: &InferenceProblem,
    proof: &RuleInferenceProof,
    config: RuleInductionConfig,
    budget: &mut ScoringBudget,
) -> Result<ValidatedInferenceCertificate, RuleInductionError> {
    // Recompute the complete ranking before examining proof fields. This keeps
    // the validation budget exact even for foreign or counterfeit proofs.
    let ranking = score_all(problem, budget)?;
    let winner = &ranking[0];
    let runner_up = ranking.get(1).ok_or(RuleInductionError::NonUniqueWinner)?;
    enforce_gates(winner, runner_up, config)?;

    if !candidate_contains(problem, &proof.rule) {
        return Err(RuleInductionError::RuleOutsideUniverse);
    }
    if proof.rule != winner.rule {
        return Err(RuleInductionError::ProofMismatch("rule"));
    }
    if proof.score != winner.score {
        return Err(RuleInductionError::ProofMismatch("score"));
    }
    if proof.support != winner.support {
        return Err(RuleInductionError::ProofMismatch("support"));
    }
    if proof.contradictions != winner.contradictions {
        return Err(RuleInductionError::ProofMismatch("contradictions"));
    }
    if proof.runner_up_score != runner_up.score {
        return Err(RuleInductionError::ProofMismatch("runner_up_score"));
    }
    if proof.supporting_evidence_ids != winner.supporting_evidence_ids {
        return Err(RuleInductionError::ProofMismatch("supporting_evidence_ids"));
    }
    if proof.contradicting_evidence_ids != winner.contradicting_evidence_ids {
        return Err(RuleInductionError::ProofMismatch("contradicting_evidence_ids"));
    }

    let expected_proof_id = proof_digest(problem, winner, runner_up.score);
    if proof.proof_id != expected_proof_id {
        return Err(RuleInductionError::ProofMismatch("proof_id"));
    }

    Ok(ValidatedInferenceCertificate {
        proof_id: proof.proof_id,
        rule: proof.rule.clone(),
        score: proof.score,
        runner_up_score: proof.runner_up_score,
        evidence_ids: proof.supporting_evidence_ids.clone(),
    })
}

pub fn expected_scoring_evaluations(problem: &InferenceProblem) -> usize {
    problem
        .antecedents
        .len()
        .saturating_mul(problem.consequents.len())
        .saturating_mul(problem.evidence.len())
}

fn validate_problem(problem: &InferenceProblem) -> Result<(), RuleInductionError> {
    if problem.antecedents.is_empty() {
        return Err(RuleInductionError::EmptyAntecedents);
    }
    if problem.consequents.is_empty() {
        return Err(RuleInductionError::EmptyConsequents);
    }
    if problem.evidence.is_empty() {
        return Err(RuleInductionError::EmptyEvidence);
    }

    let antecedents: BTreeSet<_> = problem.antecedents.iter().cloned().collect();
    if antecedents.len() != problem.antecedents.len() {
        return Err(RuleInductionError::DuplicateCandidate("antecedents"));
    }
    let consequents: BTreeSet<_> = problem.consequents.iter().cloned().collect();
    if consequents.len() != problem.consequents.len() {
        return Err(RuleInductionError::DuplicateCandidate("consequents"));
    }
    if antecedents.iter().any(|atom| consequents.contains(atom)) {
        return Err(RuleInductionError::DegenerateCandidate);
    }

    let mut evidence_ids = BTreeSet::new();
    for episode in &problem.evidence {
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(RuleInductionError::DuplicateEvidence(episode.evidence_id));
        }
    }
    Ok(())
}

fn score_all(
    problem: &InferenceProblem,
    budget: &mut ScoringBudget,
) -> Result<Vec<CandidateScore>, RuleInductionError> {
    validate_problem(problem)?;
    let mut ranking = Vec::with_capacity(problem.antecedents.len() * problem.consequents.len());

    for antecedent in &problem.antecedents {
        for consequent in &problem.consequents {
            let rule = Rule::new(antecedent.clone(), consequent.clone())
                .map_err(|_| RuleInductionError::DegenerateCandidate)?;
            let mut score = 0_i32;
            let mut support = 0_usize;
            let mut contradictions = 0_usize;
            let mut supporting_evidence_ids = Vec::new();
            let mut contradicting_evidence_ids = Vec::new();

            for episode in &problem.evidence {
                budget.candidate_episode_evaluations =
                    budget.candidate_episode_evaluations.saturating_add(1);
                if episode.intervention == *antecedent {
                    if episode.outcomes.contains(consequent) {
                        score += 3;
                        support += 1;
                        supporting_evidence_ids.push(episode.evidence_id);
                    } else {
                        score -= 4;
                        contradictions += 1;
                        contradicting_evidence_ids.push(episode.evidence_id);
                    }
                } else if episode.outcomes.contains(consequent) {
                    score -= 1;
                }
            }

            ranking.push(CandidateScore {
                rule,
                score,
                support,
                contradictions,
                supporting_evidence_ids,
                contradicting_evidence_ids,
            });
        }
    }

    ranking.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rule.cmp(&right.rule))
    });
    Ok(ranking)
}

fn enforce_gates(
    winner: &CandidateScore,
    runner_up: &CandidateScore,
    config: RuleInductionConfig,
) -> Result<(), RuleInductionError> {
    if winner.score == runner_up.score {
        return Err(RuleInductionError::NonUniqueWinner);
    }
    if winner.score < config.min_score {
        return Err(RuleInductionError::ScoreGate {
            actual: winner.score,
            required: config.min_score,
        });
    }
    if winner.support < config.min_support {
        return Err(RuleInductionError::SupportGate {
            actual: winner.support,
            required: config.min_support,
        });
    }
    if winner.contradictions > config.max_contradictions {
        return Err(RuleInductionError::ContradictionGate {
            actual: winner.contradictions,
            maximum: config.max_contradictions,
        });
    }
    let margin = winner.score - runner_up.score;
    if margin < config.min_margin {
        return Err(RuleInductionError::MarginGate {
            actual: margin,
            required: config.min_margin,
        });
    }
    Ok(())
}

fn candidate_contains(problem: &InferenceProblem, rule: &Rule) -> bool {
    problem.antecedents.contains(&rule.antecedent)
        && problem.consequents.contains(&rule.consequent)
}

fn proof_digest(problem: &InferenceProblem, winner: &CandidateScore, runner_up_score: i32) -> u64 {
    let mut bytes = Vec::new();
    for atom in &problem.antecedents {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0xff);
    }
    for atom in &problem.consequents {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0xfe);
    }
    for episode in &problem.evidence {
        bytes.extend_from_slice(&episode.evidence_id.to_le_bytes());
        bytes.extend_from_slice(episode.intervention.as_str().as_bytes());
        bytes.push(0xfd);
        for outcome in &episode.outcomes {
            bytes.extend_from_slice(outcome.as_str().as_bytes());
            bytes.push(0xfc);
        }
    }
    bytes.extend_from_slice(winner.rule.antecedent.as_str().as_bytes());
    bytes.push(0xfb);
    bytes.extend_from_slice(winner.rule.consequent.as_str().as_bytes());
    bytes.extend_from_slice(&winner.score.to_le_bytes());
    bytes.extend_from_slice(&runner_up_score.to_le_bytes());
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceProvenance {
    pub proof_id: u64,
    pub evidence_ids: Vec<u64>,
    pub score: i32,
    pub runner_up_score: i32,
}

/// H10 shadow wrapper around the accepted H9 executable state.
///
/// The inner H9 state remains unchanged. Certificate admission inserts the rule
/// into the inner executable rule index and records inference provenance in a
/// deterministic overlay. Later closure uses the ordinary H9 derivation path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBoundCommitmentState {
    inner: ExecutableCommitmentState,
    inferred_provenance: BTreeMap<Rule, InferenceProvenance>,
}

impl Default for EvidenceBoundCommitmentState {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceBoundCommitmentState {
    pub fn new() -> Self {
        Self {
            inner: ExecutableCommitmentState::new(),
            inferred_provenance: BTreeMap::new(),
        }
    }

    pub fn seed_fact(&mut self, atom: Atom) -> Result<CommitmentId, CommitmentStateError> {
        self.inner.seed_fact(atom)
    }

    pub fn seed_rule(&mut self, rule: Rule) -> Result<CommitmentId, CommitmentStateError> {
        self.inner.seed_rule(rule)
    }

    pub fn admit_certificate(
        &mut self,
        certificate: &ValidatedInferenceCertificate,
    ) -> Result<CommitmentId, RuleInductionError> {
        let rule = certificate.rule().clone();
        let id = self
            .inner
            .seed_rule(rule.clone())
            .map_err(|error| RuleInductionError::Commitment(error.to_string()))?;
        self.inferred_provenance.insert(
            rule,
            InferenceProvenance {
                proof_id: certificate.proof_id(),
                evidence_ids: certificate.evidence_ids().to_vec(),
                score: certificate.score(),
                runner_up_score: certificate.runner_up_score(),
            },
        );
        self.verify_invariants()?;
        Ok(id)
    }

    pub fn enabled_derivations(&self) -> Vec<StateDelta> {
        self.inner.enabled_derivations()
    }

    pub fn apply_delta(
        &mut self,
        delta: StateDelta,
    ) -> Result<CommitmentId, CommitmentStateError> {
        self.inner.apply_delta(delta)
    }

    pub fn contains_fact(&self, atom: &Atom) -> bool {
        self.inner.contains_fact(atom)
    }

    pub fn contains_rule(&self, rule: &Rule) -> bool {
        self.inner.contains_rule(rule)
    }

    pub fn inferred_rule_count(&self) -> usize {
        self.inferred_provenance.len()
    }

    pub fn verify_invariants(&self) -> Result<(), RuleInductionError> {
        self.inner
            .verify_invariants()
            .map_err(|error| RuleInductionError::Commitment(error.to_string()))?;
        for (rule, provenance) in &self.inferred_provenance {
            if !self.inner.contains_rule(rule) {
                return Err(RuleInductionError::InferenceInvariant(format!(
                    "provenance references absent executable rule {} -> {}",
                    rule.antecedent.as_str(),
                    rule.consequent.as_str()
                )));
            }
            if provenance.evidence_ids.is_empty() {
                return Err(RuleInductionError::InferenceInvariant(
                    "inferred rule has empty evidence provenance".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn canonical_signature(&self) -> String {
        let mut signature = self.inner.canonical_signature();
        for (rule, provenance) in &self.inferred_provenance {
            signature.push_str(&format!(
                "\ninference:{}->{}:proof:{}:score:{}:runner:{}:evidence:{}",
                rule.antecedent.as_str(),
                rule.consequent.as_str(),
                provenance.proof_id,
                provenance.score,
                provenance.runner_up_score,
                provenance
                    .evidence_ids
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn problem(prefix: &str) -> InferenceProblem {
        let middle = atom(&format!("{prefix}_middle"));
        let goal = atom(&format!("{prefix}_goal"));
        let decoy_source = atom(&format!("{prefix}_decoy_source"));
        let decoy_goal = atom(&format!("{prefix}_decoy_goal"));
        let noise_source = atom(&format!("{prefix}_noise_source"));
        let noise_goal = atom(&format!("{prefix}_noise_goal"));
        let mut evidence = Vec::new();
        for id in 1..=4 {
            evidence.push(EvidenceEpisode {
                evidence_id: id,
                intervention: middle.clone(),
                outcomes: BTreeSet::from([goal.clone()]),
            });
        }
        for id in 5..=7 {
            evidence.push(EvidenceEpisode {
                evidence_id: id,
                intervention: decoy_source.clone(),
                outcomes: BTreeSet::from([decoy_goal.clone()]),
            });
        }
        evidence.push(EvidenceEpisode {
            evidence_id: 8,
            intervention: decoy_source.clone(),
            outcomes: BTreeSet::new(),
        });
        evidence.push(EvidenceEpisode {
            evidence_id: 9,
            intervention: noise_source.clone(),
            outcomes: BTreeSet::from([noise_goal.clone()]),
        });
        evidence.push(EvidenceEpisode {
            evidence_id: 10,
            intervention: noise_source.clone(),
            outcomes: BTreeSet::new(),
        });
        InferenceProblem {
            antecedents: vec![middle, decoy_source, noise_source],
            consequents: vec![goal, decoy_goal, noise_goal],
            evidence,
        }
    }

    #[test]
    fn infers_unique_rule_and_recomputes_full_validation_budget() {
        let problem = problem("alpha");
        let config = RuleInductionConfig::default();
        let mut proposal_budget = ScoringBudget::default();
        let proof = infer_rule(&problem, config, &mut proposal_budget).unwrap();
        assert_eq!(proposal_budget.candidate_episode_evaluations, 90);
        assert_eq!(proof.score, 12);
        assert_eq!(proof.support, 4);
        assert_eq!(proof.contradictions, 0);
        assert_eq!(proof.runner_up_score, 5);

        let mut validation_budget = ScoringBudget::default();
        let certificate =
            validate_rule_inference(&problem, &proof, config, &mut validation_budget).unwrap();
        assert_eq!(validation_budget.candidate_episode_evaluations, 90);
        assert_eq!(certificate.rule(), &proof.rule);
    }

    #[test]
    fn counterfeit_score_is_rejected_after_full_recomputation() {
        let problem = problem("beta");
        let config = RuleInductionConfig::default();
        let mut proposal_budget = ScoringBudget::default();
        let mut proof = infer_rule(&problem, config, &mut proposal_budget).unwrap();
        proof.score += 1;
        let mut validation_budget = ScoringBudget::default();
        let error = validate_rule_inference(&problem, &proof, config, &mut validation_budget)
            .unwrap_err();
        assert_eq!(validation_budget.candidate_episode_evaluations, 90);
        assert_eq!(error, RuleInductionError::ProofMismatch("score"));
    }

    #[test]
    fn certificate_admission_changes_reachable_closure() {
        let problem = problem("gamma");
        let config = RuleInductionConfig::default();
        let mut proposal_budget = ScoringBudget::default();
        let proof = infer_rule(&problem, config, &mut proposal_budget).unwrap();
        let mut validation_budget = ScoringBudget::default();
        let certificate =
            validate_rule_inference(&problem, &proof, config, &mut validation_budget).unwrap();

        let source = atom("gamma_source");
        let middle = problem.antecedents[0].clone();
        let goal = problem.consequents[0].clone();
        let mut state = EvidenceBoundCommitmentState::new();
        state.seed_fact(source.clone()).unwrap();
        state
            .seed_rule(Rule::new(source, middle).unwrap())
            .unwrap();
        state.admit_certificate(&certificate).unwrap();

        for _ in 0..3 {
            if let Some(delta) = state.enabled_derivations().into_iter().next() {
                state.apply_delta(delta).unwrap();
            }
        }
        assert!(state.contains_fact(&goal));
        assert_eq!(state.inferred_rule_count(), 1);
        state.verify_invariants().unwrap();
    }
}
