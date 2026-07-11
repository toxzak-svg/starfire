//! Graph-discovered candidate-frontier induction for the H11 shadow experiment.
//!
//! H10 receives an explicit candidate antecedent/consequent universe. H11 removes
//! that input. A deterministic frontier is first discovered from raw intervention
//! graph incidence, then the unchanged H10 scoring and independent proof
//! recomputation path is applied over that discovered universe.

use crate::commitment_state::{Atom, CommitmentId, Rule};
use crate::rule_induction::{
    infer_rule, validate_rule_inference, EvidenceBoundCommitmentState, EvidenceEpisode,
    InferenceProblem, RuleInductionConfig, RuleInferenceProof, ScoringBudget,
    ValidatedInferenceCertificate,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixedEvidenceGraph {
    pub evidence: Vec<EvidenceEpisode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontierDiscoveryBudget {
    pub evidence_episode_scans: usize,
    pub discovered_antecedents: usize,
    pub discovered_consequents: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredFrontier {
    pub antecedents: Vec<Atom>,
    pub consequents: Vec<Atom>,
    pub digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphInferenceProof {
    pub proof_id: u64,
    pub frontier_digest: u64,
    pub discovered_antecedents: Vec<Atom>,
    pub discovered_consequents: Vec<Atom>,
    pub rule: Rule,
    pub score: i32,
    pub support: usize,
    pub contradictions: usize,
    pub runner_up_score: i32,
    pub supporting_evidence_ids: Vec<u64>,
    pub contradicting_evidence_ids: Vec<u64>,
}

/// Opaque H11 certificate. Callers cannot construct the embedded H10
/// certificate directly; admission is only exposed through
/// `admit_graph_certificate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedGraphInferenceCertificate {
    inner: ValidatedInferenceCertificate,
    frontier_digest: u64,
}

impl ValidatedGraphInferenceCertificate {
    pub fn proof_id(&self) -> u64 {
        self.inner.proof_id()
    }

    pub fn rule(&self) -> &Rule {
        self.inner.rule()
    }

    pub fn frontier_digest(&self) -> u64 {
        self.frontier_digest
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GraphDiscoveryError {
    #[error("mixed evidence graph is empty")]
    EmptyEvidence,
    #[error("duplicate evidence id {0}")]
    DuplicateEvidence(u64),
    #[error("discovered {0} frontier is empty")]
    EmptyFrontier(&'static str),
    #[error("rule induction failed: {0}")]
    RuleInduction(String),
    #[error("graph-discovery proof does not match independent recomputation: {0}")]
    ProofMismatch(&'static str),
    #[error("certificate admission failed: {0}")]
    Commitment(String),
}

/// Discover the candidate relation vocabulary from graph incidence alone.
///
/// Frozen H11 eligibility:
/// - intervention atom appears in at least two episodes;
/// - outcome atom appears in at least two episode outcome sets.
pub fn discover_frontier(
    graph: &MixedEvidenceGraph,
    budget: &mut FrontierDiscoveryBudget,
) -> Result<DiscoveredFrontier, GraphDiscoveryError> {
    if graph.evidence.is_empty() {
        return Err(GraphDiscoveryError::EmptyEvidence);
    }

    let mut evidence_ids = BTreeSet::new();
    let mut intervention_counts = BTreeMap::<Atom, usize>::new();
    let mut outcome_counts = BTreeMap::<Atom, usize>::new();

    for episode in &graph.evidence {
        budget.evidence_episode_scans = budget.evidence_episode_scans.saturating_add(1);
        if !evidence_ids.insert(episode.evidence_id) {
            return Err(GraphDiscoveryError::DuplicateEvidence(episode.evidence_id));
        }
        *intervention_counts
            .entry(episode.intervention.clone())
            .or_insert(0) += 1;
        for outcome in &episode.outcomes {
            *outcome_counts.entry(outcome.clone()).or_insert(0) += 1;
        }
    }

    let antecedents = intervention_counts
        .into_iter()
        .filter_map(|(atom, count)| (count >= 2).then_some(atom))
        .collect::<Vec<_>>();
    let consequents = outcome_counts
        .into_iter()
        .filter_map(|(atom, count)| (count >= 2).then_some(atom))
        .collect::<Vec<_>>();

    if antecedents.is_empty() {
        return Err(GraphDiscoveryError::EmptyFrontier("antecedent"));
    }
    if consequents.is_empty() {
        return Err(GraphDiscoveryError::EmptyFrontier("consequent"));
    }

    budget.discovered_antecedents = antecedents.len();
    budget.discovered_consequents = consequents.len();
    let digest = frontier_digest(&antecedents, &consequents);

    Ok(DiscoveredFrontier {
        antecedents,
        consequents,
        digest,
    })
}

pub fn infer_graph_rule(
    graph: &MixedEvidenceGraph,
    config: RuleInductionConfig,
    frontier_budget: &mut FrontierDiscoveryBudget,
    scoring_budget: &mut ScoringBudget,
) -> Result<GraphInferenceProof, GraphDiscoveryError> {
    let frontier = discover_frontier(graph, frontier_budget)?;
    let problem = frontier_problem(graph, &frontier);
    let proof = infer_rule(&problem, config, scoring_budget)
        .map_err(|error| GraphDiscoveryError::RuleInduction(error.to_string()))?;

    Ok(GraphInferenceProof {
        proof_id: proof.proof_id,
        frontier_digest: frontier.digest,
        discovered_antecedents: frontier.antecedents,
        discovered_consequents: frontier.consequents,
        rule: proof.rule,
        score: proof.score,
        support: proof.support,
        contradictions: proof.contradictions,
        runner_up_score: proof.runner_up_score,
        supporting_evidence_ids: proof.supporting_evidence_ids,
        contradicting_evidence_ids: proof.contradicting_evidence_ids,
    })
}

/// Independently rediscover the frontier and then invoke the unchanged H10
/// full-ranking validator. H10 validation is deliberately performed before H11
/// frontier-field comparison so a frontier-tampered proof still consumes the
/// complete 36 x 16 scoring recomputation budget.
pub fn validate_graph_rule(
    graph: &MixedEvidenceGraph,
    proof: &GraphInferenceProof,
    config: RuleInductionConfig,
    frontier_budget: &mut FrontierDiscoveryBudget,
    scoring_budget: &mut ScoringBudget,
) -> Result<ValidatedGraphInferenceCertificate, GraphDiscoveryError> {
    let frontier = discover_frontier(graph, frontier_budget)?;
    let problem = frontier_problem(graph, &frontier);
    let inner_proof = RuleInferenceProof {
        proof_id: proof.proof_id,
        rule: proof.rule.clone(),
        score: proof.score,
        support: proof.support,
        contradictions: proof.contradictions,
        runner_up_score: proof.runner_up_score,
        supporting_evidence_ids: proof.supporting_evidence_ids.clone(),
        contradicting_evidence_ids: proof.contradicting_evidence_ids.clone(),
    };

    let inner = validate_rule_inference(&problem, &inner_proof, config, scoring_budget)
        .map_err(|error| GraphDiscoveryError::RuleInduction(error.to_string()))?;

    if proof.frontier_digest != frontier.digest {
        return Err(GraphDiscoveryError::ProofMismatch("frontier_digest"));
    }
    if proof.discovered_antecedents != frontier.antecedents {
        return Err(GraphDiscoveryError::ProofMismatch(
            "discovered_antecedents",
        ));
    }
    if proof.discovered_consequents != frontier.consequents {
        return Err(GraphDiscoveryError::ProofMismatch(
            "discovered_consequents",
        ));
    }

    Ok(ValidatedGraphInferenceCertificate {
        inner,
        frontier_digest: frontier.digest,
    })
}

pub fn admit_graph_certificate(
    state: &mut EvidenceBoundCommitmentState,
    certificate: &ValidatedGraphInferenceCertificate,
) -> Result<CommitmentId, GraphDiscoveryError> {
    state
        .admit_certificate(&certificate.inner)
        .map_err(|error| GraphDiscoveryError::Commitment(error.to_string()))
}

pub fn discovered_candidate_count(frontier: &DiscoveredFrontier) -> usize {
    frontier
        .antecedents
        .len()
        .saturating_mul(frontier.consequents.len())
}

pub fn expected_graph_scoring_evaluations(
    graph: &MixedEvidenceGraph,
    frontier: &DiscoveredFrontier,
) -> usize {
    discovered_candidate_count(frontier).saturating_mul(graph.evidence.len())
}

fn frontier_problem(graph: &MixedEvidenceGraph, frontier: &DiscoveredFrontier) -> InferenceProblem {
    InferenceProblem {
        antecedents: frontier.antecedents.clone(),
        consequents: frontier.consequents.clone(),
        evidence: graph.evidence.clone(),
    }
}

fn frontier_digest(antecedents: &[Atom], consequents: &[Atom]) -> u64 {
    let mut bytes = Vec::new();
    for atom in antecedents {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0xa1);
    }
    bytes.push(0xff);
    for atom in consequents {
        bytes.extend_from_slice(atom.as_str().as_bytes());
        bytes.push(0xb2);
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
    use crate::commitment_state::Rule;

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn graph(prefix: &str) -> MixedEvidenceGraph {
        let antecedents = (0..6)
            .map(|index| atom(&format!("{prefix}_a{index}")))
            .collect::<Vec<_>>();
        let consequents = (0..6)
            .map(|index| atom(&format!("{prefix}_b{index}")))
            .collect::<Vec<_>>();
        let noise_a0 = atom(&format!("{prefix}_noise_a0"));
        let noise_a1 = atom(&format!("{prefix}_noise_a1"));
        let mut evidence = Vec::new();
        let mut id = 1_u64;

        for _ in 0..4 {
            evidence.push(EvidenceEpisode {
                evidence_id: id,
                intervention: antecedents[0].clone(),
                outcomes: BTreeSet::from([consequents[0].clone()]),
            });
            id += 1;
        }
        for pair in 1..6 {
            for _ in 0..2 {
                evidence.push(EvidenceEpisode {
                    evidence_id: id,
                    intervention: antecedents[pair].clone(),
                    outcomes: BTreeSet::from([consequents[pair].clone()]),
                });
                id += 1;
            }
        }
        evidence.push(EvidenceEpisode {
            evidence_id: id,
            intervention: noise_a0,
            outcomes: BTreeSet::new(),
        });
        evidence.push(EvidenceEpisode {
            evidence_id: id + 1,
            intervention: noise_a1,
            outcomes: BTreeSet::new(),
        });

        MixedEvidenceGraph { evidence }
    }

    #[test]
    fn discovers_six_by_six_frontier_and_reuses_h10_scoring() {
        let graph = graph("alpha");
        let config = RuleInductionConfig::default();
        let mut proposal_frontier_budget = FrontierDiscoveryBudget::default();
        let mut proposal_scoring_budget = ScoringBudget::default();
        let proof = infer_graph_rule(
            &graph,
            config,
            &mut proposal_frontier_budget,
            &mut proposal_scoring_budget,
        )
        .unwrap();
        assert_eq!(proposal_frontier_budget.evidence_episode_scans, 16);
        assert_eq!(proposal_frontier_budget.discovered_antecedents, 6);
        assert_eq!(proposal_frontier_budget.discovered_consequents, 6);
        assert_eq!(proposal_scoring_budget.candidate_episode_evaluations, 576);
        assert_eq!(proof.score, 12);
        assert_eq!(proof.support, 4);
        assert_eq!(proof.runner_up_score, 6);

        let mut validation_frontier_budget = FrontierDiscoveryBudget::default();
        let mut validation_scoring_budget = ScoringBudget::default();
        let certificate = validate_graph_rule(
            &graph,
            &proof,
            config,
            &mut validation_frontier_budget,
            &mut validation_scoring_budget,
        )
        .unwrap();
        assert_eq!(validation_frontier_budget.evidence_episode_scans, 16);
        assert_eq!(validation_scoring_budget.candidate_episode_evaluations, 576);
        assert_eq!(certificate.rule(), &proof.rule);
    }

    #[test]
    fn frontier_tamper_is_rejected_after_full_recomputation() {
        let graph = graph("beta");
        let config = RuleInductionConfig::default();
        let mut proposal_frontier_budget = FrontierDiscoveryBudget::default();
        let mut proposal_scoring_budget = ScoringBudget::default();
        let mut proof = infer_graph_rule(
            &graph,
            config,
            &mut proposal_frontier_budget,
            &mut proposal_scoring_budget,
        )
        .unwrap();
        proof.discovered_antecedents[0] = atom("tampered_frontier_atom");

        let mut validation_frontier_budget = FrontierDiscoveryBudget::default();
        let mut validation_scoring_budget = ScoringBudget::default();
        let error = validate_graph_rule(
            &graph,
            &proof,
            config,
            &mut validation_frontier_budget,
            &mut validation_scoring_budget,
        )
        .unwrap_err();
        assert_eq!(validation_frontier_budget.evidence_episode_scans, 16);
        assert_eq!(validation_scoring_budget.candidate_episode_evaluations, 576);
        assert_eq!(
            error,
            GraphDiscoveryError::ProofMismatch("discovered_antecedents")
        );
    }

    #[test]
    fn admitted_graph_certificate_changes_reachable_closure() {
        let graph = graph("gamma");
        let config = RuleInductionConfig::default();
        let mut proposal_frontier_budget = FrontierDiscoveryBudget::default();
        let mut proposal_scoring_budget = ScoringBudget::default();
        let proof = infer_graph_rule(
            &graph,
            config,
            &mut proposal_frontier_budget,
            &mut proposal_scoring_budget,
        )
        .unwrap();
        let mut validation_frontier_budget = FrontierDiscoveryBudget::default();
        let mut validation_scoring_budget = ScoringBudget::default();
        let certificate = validate_graph_rule(
            &graph,
            &proof,
            config,
            &mut validation_frontier_budget,
            &mut validation_scoring_budget,
        )
        .unwrap();

        let source = atom("gamma_source");
        let middle = proof.rule.antecedent.clone();
        let goal = proof.rule.consequent.clone();
        let mut state = EvidenceBoundCommitmentState::new();
        state.seed_fact(source.clone()).unwrap();
        state
            .seed_rule(Rule::new(source, middle).unwrap())
            .unwrap();
        admit_graph_certificate(&mut state, &certificate).unwrap();

        for _ in 0..3 {
            if let Some(delta) = state.enabled_derivations().into_iter().next() {
                state.apply_delta(delta).unwrap();
            }
        }
        assert!(state.contains_fact(&goal));
        state.verify_invariants().unwrap();
    }
}
