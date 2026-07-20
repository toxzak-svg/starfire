//! ΩG4 bounded intervention-guided abstraction selection.
//!
//! This module is deterministic, offline, and shadow-only. It preserves an
//! observational tie between two compact reusable abstractions, selects one
//! bounded intervention from candidate disagreement without outcome access,
//! and requires an independently constructed digest-bound witness before a
//! final abstraction can be admitted.

use crate::commitment_state::Atom;
use crate::multistep_abstraction_reuse::{
    AbstractionRegistry, AbstractionSchemaKind,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionHistory {
    pub history_id: u64,
    pub events: Vec<Atom>,
    pub positive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRoot {
    pub root_id: u64,
    pub family: String,
    pub chain: Vec<Atom>,
    pub proxy: Atom,
    pub distractors: Vec<Atom>,
    pub passive: Vec<SelectionHistory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractionSelectionProblem {
    pub cohort_id: u64,
    pub roots: Vec<SelectionRoot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedOmegaG3SelectionParent {
    registry_signature: String,
}

impl ValidatedOmegaG3SelectionParent {
    pub fn registry_signature(&self) -> &str {
        &self.registry_signature
    }
}

pub fn revalidate_omega_g3_selection_parent(
    registry: &AbstractionRegistry,
) -> Result<ValidatedOmegaG3SelectionParent, SelectionError> {
    registry
        .verify_invariants()
        .map_err(|error| SelectionError::ParentValidation(error.to_string()))?;
    if registry.admitted_count() != 1
        || !registry.supports(AbstractionSchemaKind::RecursiveAppendAdjacent)
    {
        return Err(SelectionError::OmegaG3ParentNotAdmitted);
    }
    Ok(ValidatedOmegaG3SelectionParent {
        registry_signature: registry.canonical_signature(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SelectionSchemaKind {
    ProxyAnchorAdjacent,
    RecursiveAppendAdjacent,
    FixedArityFourMemorizer,
    SurfaceAtomLookup,
}

impl SelectionSchemaKind {
    pub fn all() -> [Self; 4] {
        [
            Self::ProxyAnchorAdjacent,
            Self::RecursiveAppendAdjacent,
            Self::FixedArityFourMemorizer,
            Self::SurfaceAtomLookup,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ProxyAnchorAdjacent => "proxy_anchor_adjacent",
            Self::RecursiveAppendAdjacent => "recursive_append_adjacent",
            Self::FixedArityFourMemorizer => "fixed_arity_four_memorizer",
            Self::SurfaceAtomLookup => "surface_atom_lookup",
        }
    }

    pub fn node_cost(self) -> usize {
        match self {
            Self::ProxyAnchorAdjacent | Self::RecursiveAppendAdjacent => 5,
            Self::FixedArityFourMemorizer => 14,
            Self::SurfaceAtomLookup => 24,
        }
    }

    fn reusable_support(self) -> bool {
        matches!(
            self,
            Self::ProxyAnchorAdjacent | Self::RecursiveAppendAdjacent
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveCandidateAnalysis {
    pub kind: SelectionSchemaKind,
    pub exact_histories: usize,
    pub reusable_support: bool,
    pub node_cost: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveSearchBudget {
    pub schema_candidates: usize,
    pub schema_history_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveAmbiguityProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub parent_registry_signature: String,
    pub analyses: Vec<PassiveCandidateAnalysis>,
    pub winners: Vec<SelectionSchemaKind>,
    pub winner_node_costs: Vec<usize>,
    pub passive_histories: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPassiveAmbiguityCertificate {
    cohort_id: u64,
    proof_id: u64,
    problem_digest: u64,
    parent_registry_signature: String,
    winners: Vec<SelectionSchemaKind>,
}

impl ValidatedPassiveAmbiguityCertificate {
    pub fn winners(&self) -> &[SelectionSchemaKind] {
        &self.winners
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }
}

pub fn synthesize_passive_ambiguity(
    problem: &AbstractionSelectionProblem,
    parent: &ValidatedOmegaG3SelectionParent,
    budget: &mut PassiveSearchBudget,
) -> Result<PassiveAmbiguityProof, SelectionError> {
    search_passive_ambiguity(problem, parent, budget)
}

pub fn validate_passive_ambiguity(
    problem: &AbstractionSelectionProblem,
    parent: &ValidatedOmegaG3SelectionParent,
    supplied: &PassiveAmbiguityProof,
    budget: &mut PassiveSearchBudget,
) -> Result<ValidatedPassiveAmbiguityCertificate, SelectionError> {
    let recomputed = search_passive_ambiguity(problem, parent, budget)?;
    if supplied != &recomputed {
        return Err(SelectionError::PassiveProofMismatch);
    }
    enforce_passive_gates(&recomputed)?;
    Ok(ValidatedPassiveAmbiguityCertificate {
        cohort_id: recomputed.cohort_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        parent_registry_signature: recomputed.parent_registry_signature,
        winners: recomputed.winners,
    })
}

fn search_passive_ambiguity(
    problem: &AbstractionSelectionProblem,
    parent: &ValidatedOmegaG3SelectionParent,
    budget: &mut PassiveSearchBudget,
) -> Result<PassiveAmbiguityProof, SelectionError> {
    validate_problem(problem)?;
    let passive_histories = problem.roots.iter().map(|root| root.passive.len()).sum();
    let mut analyses = Vec::new();
    for kind in SelectionSchemaKind::all() {
        budget.schema_candidates = budget.schema_candidates.saturating_add(1);
        let mut exact_histories = 0usize;
        for root in &problem.roots {
            for history in &root.passive {
                budget.schema_history_evaluations =
                    budget.schema_history_evaluations.saturating_add(1);
                if schema_predict(kind, root, &history.events)? == history.positive {
                    exact_histories = exact_histories.saturating_add(1);
                }
            }
        }
        analyses.push(PassiveCandidateAnalysis {
            kind,
            exact_histories,
            reusable_support: kind.reusable_support(),
            node_cost: kind.node_cost(),
        });
    }
    let winners = analyses
        .iter()
        .filter(|analysis| {
            analysis.exact_histories == passive_histories && analysis.reusable_support
        })
        .map(|analysis| analysis.kind)
        .collect::<Vec<_>>();
    let winner_node_costs = winners.iter().map(|kind| kind.node_cost()).collect::<Vec<_>>();
    let problem_digest = digest_problem(problem);
    let proof_id = mix64(
        problem_digest
            ^ hash_text(parent.registry_signature())
            ^ winners
                .iter()
                .fold(0u64, |acc, kind| acc ^ hash_text(kind.name())),
    );
    Ok(PassiveAmbiguityProof {
        proof_id,
        cohort_id: problem.cohort_id,
        problem_digest,
        parent_registry_signature: parent.registry_signature.clone(),
        analyses,
        winners,
        winner_node_costs,
        passive_histories,
    })
}

fn enforce_passive_gates(proof: &PassiveAmbiguityProof) -> Result<(), SelectionError> {
    let expected = vec![
        SelectionSchemaKind::ProxyAnchorAdjacent,
        SelectionSchemaKind::RecursiveAppendAdjacent,
    ];
    if proof.passive_histories != 192
        || proof.winners != expected
        || proof.winner_node_costs != vec![5, 5]
    {
        return Err(SelectionError::PassiveGate(
            "passive tie, support, or compression mismatch",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InterventionKind {
    MoveProxyAfterX0,
    SwapInternalChainEdge,
    MoveDistractorLeft,
    MoveDistractorRight,
    RotateDistractors,
    IdentityControl,
}

impl InterventionKind {
    pub fn all() -> [Self; 6] {
        [
            Self::MoveProxyAfterX0,
            Self::SwapInternalChainEdge,
            Self::MoveDistractorLeft,
            Self::MoveDistractorRight,
            Self::RotateDistractors,
            Self::IdentityControl,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MoveProxyAfterX0 => "move_proxy_after_x0",
            Self::SwapInternalChainEdge => "swap_internal_chain_edge",
            Self::MoveDistractorLeft => "move_distractor_left",
            Self::MoveDistractorRight => "move_distractor_right",
            Self::RotateDistractors => "rotate_distractors",
            Self::IdentityControl => "identity_control",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionAnalysis {
    pub intervention: InterventionKind,
    pub predictions: Vec<(SelectionSchemaKind, bool)>,
    pub disagreement_score: usize,
    pub cost: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionPlanBudget {
    pub intervention_candidates: usize,
    pub candidate_intervention_predictions: usize,
    pub pairwise_disagreement_comparisons: usize,
    pub selected_interventions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionPlanProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub passive_proof_id: u64,
    pub root_id: u64,
    pub history_id: u64,
    pub before_digest: u64,
    pub analyses: Vec<InterventionAnalysis>,
    pub selected: InterventionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInterventionPlanCertificate {
    proof_id: u64,
    cohort_id: u64,
    passive_proof_id: u64,
    root_id: u64,
    history_id: u64,
    before_digest: u64,
    selected: InterventionKind,
}

impl ValidatedInterventionPlanCertificate {
    pub fn selected(&self) -> InterventionKind {
        self.selected
    }

    pub fn canonical_trace(&self) -> String {
        format!(
            "cohort:{}|passive:{}|root:{}|history:{}|before:{}|selected:{}",
            self.cohort_id,
            self.passive_proof_id,
            self.root_id,
            self.history_id,
            self.before_digest,
            self.selected.name()
        )
    }
}

pub fn synthesize_intervention_plan(
    problem: &AbstractionSelectionProblem,
    passive: &ValidatedPassiveAmbiguityCertificate,
    budget: &mut InterventionPlanBudget,
) -> Result<InterventionPlanProof, SelectionError> {
    search_intervention_plan(problem, passive, budget)
}

pub fn validate_intervention_plan(
    problem: &AbstractionSelectionProblem,
    passive: &ValidatedPassiveAmbiguityCertificate,
    supplied: &InterventionPlanProof,
    budget: &mut InterventionPlanBudget,
) -> Result<ValidatedInterventionPlanCertificate, SelectionError> {
    let recomputed = search_intervention_plan(problem, passive, budget)?;
    if supplied != &recomputed {
        return Err(SelectionError::InterventionPlanMismatch);
    }
    if recomputed.selected != InterventionKind::MoveProxyAfterX0 {
        return Err(SelectionError::InterventionGate("wrong intervention"));
    }
    Ok(ValidatedInterventionPlanCertificate {
        proof_id: recomputed.proof_id,
        cohort_id: recomputed.cohort_id,
        passive_proof_id: recomputed.passive_proof_id,
        root_id: recomputed.root_id,
        history_id: recomputed.history_id,
        before_digest: recomputed.before_digest,
        selected: recomputed.selected,
    })
}

fn search_intervention_plan(
    problem: &AbstractionSelectionProblem,
    passive: &ValidatedPassiveAmbiguityCertificate,
    budget: &mut InterventionPlanBudget,
) -> Result<InterventionPlanProof, SelectionError> {
    if passive.cohort_id != problem.cohort_id
        || passive.problem_digest != digest_problem(problem)
        || passive.winners.len() != 2
    {
        return Err(SelectionError::PassiveCertificateMismatch);
    }
    let root = problem.roots.first().ok_or(SelectionError::EmptyProblem)?;
    let base = root
        .passive
        .iter()
        .filter(|history| history.positive)
        .min_by_key(|history| history.history_id)
        .ok_or(SelectionError::MissingCalibrationHistory)?;
    let mut analyses = Vec::new();
    for intervention in InterventionKind::all() {
        budget.intervention_candidates = budget.intervention_candidates.saturating_add(1);
        let transformed = apply_intervention(root, &base.events, intervention)?;
        let mut predictions = Vec::new();
        for kind in &passive.winners {
            budget.candidate_intervention_predictions =
                budget.candidate_intervention_predictions.saturating_add(1);
            predictions.push((*kind, schema_predict(*kind, root, &transformed)?));
        }
        let mut disagreement_score = 0usize;
        for left in 0..predictions.len() {
            for right in left + 1..predictions.len() {
                budget.pairwise_disagreement_comparisons =
                    budget.pairwise_disagreement_comparisons.saturating_add(1);
                if predictions[left].1 != predictions[right].1 {
                    disagreement_score = disagreement_score.saturating_add(1);
                }
            }
        }
        analyses.push(InterventionAnalysis {
            intervention,
            predictions,
            disagreement_score,
            cost: 1,
        });
    }
    let selected = analyses
        .iter()
        .max_by_key(|analysis| {
            (
                analysis.disagreement_score,
                usize::MAX - analysis.cost,
                std::cmp::Reverse(analysis.intervention),
            )
        })
        .ok_or(SelectionError::NoIntervention)?
        .intervention;
    budget.selected_interventions = budget.selected_interventions.saturating_add(1);
    let before_digest = digest_events(&base.events);
    let proof_id = mix64(
        passive.proof_id
            ^ before_digest
            ^ hash_text(selected.name())
            ^ root.root_id.rotate_left(11),
    );
    Ok(InterventionPlanProof {
        proof_id,
        cohort_id: problem.cohort_id,
        passive_proof_id: passive.proof_id,
        root_id: root.root_id,
        history_id: base.history_id,
        before_digest,
        analyses,
        selected,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionWitness {
    root_id: u64,
    history_id: u64,
    before_digest: u64,
    intervention: InterventionKind,
    after_digest: u64,
    observed_outcome: bool,
    witness_id: u64,
}

impl InterventionWitness {
    pub fn observed_outcome(&self) -> bool {
        self.observed_outcome
    }

    pub fn intervention(&self) -> InterventionKind {
        self.intervention
    }
}

pub fn execute_intervention(
    problem: &AbstractionSelectionProblem,
    plan: &ValidatedInterventionPlanCertificate,
) -> Result<InterventionWitness, SelectionError> {
    let root = problem
        .roots
        .iter()
        .find(|root| root.root_id == plan.root_id)
        .ok_or(SelectionError::RootMismatch)?;
    let base = root
        .passive
        .iter()
        .find(|history| history.history_id == plan.history_id)
        .ok_or(SelectionError::MissingCalibrationHistory)?;
    if digest_events(&base.events) != plan.before_digest {
        return Err(SelectionError::WitnessDigestMismatch);
    }
    let after = apply_intervention(root, &base.events, plan.selected)?;
    let after_digest = digest_events(&after);
    let observed_outcome = recursive_chain_predict(root, &after);
    let witness_id = mix64(
        plan.proof_id
            ^ plan.before_digest
            ^ after_digest.rotate_left(13)
            ^ u64::from(observed_outcome),
    );
    Ok(InterventionWitness {
        root_id: root.root_id,
        history_id: base.history_id,
        before_digest: plan.before_digest,
        intervention: plan.selected,
        after_digest,
        observed_outcome,
        witness_id,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateVerdict {
    pub kind: SelectionSchemaKind,
    pub predicted_outcome: bool,
    pub observed_outcome: bool,
    pub retained: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalSelectionProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub passive_proof_id: u64,
    pub intervention_plan_id: u64,
    pub witness_id: u64,
    pub parent_registry_signature: String,
    pub verdicts: Vec<CandidateVerdict>,
    pub winner: SelectionSchemaKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFinalAbstractionCertificate {
    cohort_id: u64,
    proof_id: u64,
    parent_registry_signature: String,
    kind: SelectionSchemaKind,
}

impl ValidatedFinalAbstractionCertificate {
    pub fn kind(&self) -> SelectionSchemaKind {
        self.kind
    }
}

pub fn validate_witness_and_select(
    problem: &AbstractionSelectionProblem,
    parent: &ValidatedOmegaG3SelectionParent,
    passive: &ValidatedPassiveAmbiguityCertificate,
    plan: &ValidatedInterventionPlanCertificate,
    witness: &InterventionWitness,
) -> Result<(FinalSelectionProof, ValidatedFinalAbstractionCertificate), SelectionError> {
    validate_witness(problem, plan, witness)?;
    if passive.parent_registry_signature != parent.registry_signature
        || passive.cohort_id != problem.cohort_id
        || plan.passive_proof_id != passive.proof_id
    {
        return Err(SelectionError::ParentRegistryMismatch);
    }
    let root = problem
        .roots
        .iter()
        .find(|root| root.root_id == witness.root_id)
        .ok_or(SelectionError::RootMismatch)?;
    let base = root
        .passive
        .iter()
        .find(|history| history.history_id == witness.history_id)
        .ok_or(SelectionError::MissingCalibrationHistory)?;
    let after = apply_intervention(root, &base.events, witness.intervention)?;
    let verdicts = passive
        .winners
        .iter()
        .map(|kind| {
            let predicted_outcome = schema_predict(*kind, root, &after)?;
            Ok(CandidateVerdict {
                kind: *kind,
                predicted_outcome,
                observed_outcome: witness.observed_outcome,
                retained: predicted_outcome == witness.observed_outcome,
            })
        })
        .collect::<Result<Vec<_>, SelectionError>>()?;
    let retained = verdicts
        .iter()
        .filter(|verdict| verdict.retained)
        .map(|verdict| verdict.kind)
        .collect::<Vec<_>>();
    if retained.len() != 1 {
        return Err(SelectionError::TieUnresolved);
    }
    let winner = retained[0];
    if winner != SelectionSchemaKind::RecursiveAppendAdjacent {
        return Err(SelectionError::FinalSelectionGate("wrong final abstraction"));
    }
    let proof_id = mix64(
        passive.proof_id
            ^ plan.proof_id.rotate_left(7)
            ^ witness.witness_id.rotate_left(17)
            ^ hash_text(winner.name()),
    );
    let proof = FinalSelectionProof {
        proof_id,
        cohort_id: problem.cohort_id,
        passive_proof_id: passive.proof_id,
        intervention_plan_id: plan.proof_id,
        witness_id: witness.witness_id,
        parent_registry_signature: parent.registry_signature.clone(),
        verdicts,
        winner,
    };
    let certificate = ValidatedFinalAbstractionCertificate {
        cohort_id: problem.cohort_id,
        proof_id,
        parent_registry_signature: parent.registry_signature.clone(),
        kind: winner,
    };
    Ok((proof, certificate))
}

fn validate_witness(
    problem: &AbstractionSelectionProblem,
    plan: &ValidatedInterventionPlanCertificate,
    witness: &InterventionWitness,
) -> Result<(), SelectionError> {
    let expected = execute_intervention(problem, plan)?;
    if witness != &expected {
        return Err(SelectionError::CounterfeitWitness);
    }
    Ok(())
}

pub fn counterfeit_witness_control(
    problem: &AbstractionSelectionProblem,
    plan: &ValidatedInterventionPlanCertificate,
    witness: &InterventionWitness,
) -> bool {
    let mut counterfeit = witness.clone();
    counterfeit.observed_outcome = !counterfeit.observed_outcome;
    validate_witness(problem, plan, &counterfeit).is_err()
}

pub fn forced_intervention_remaining_candidates(
    problem: &AbstractionSelectionProblem,
    passive: &ValidatedPassiveAmbiguityCertificate,
    intervention: InterventionKind,
) -> Result<usize, SelectionError> {
    let root = problem.roots.first().ok_or(SelectionError::EmptyProblem)?;
    let base = root
        .passive
        .iter()
        .filter(|history| history.positive)
        .min_by_key(|history| history.history_id)
        .ok_or(SelectionError::MissingCalibrationHistory)?;
    let after = apply_intervention(root, &base.events, intervention)?;
    let observed = recursive_chain_predict(root, &after);
    passive.winners.iter().try_fold(0usize, |count, kind| {
        Ok(count + usize::from(schema_predict(*kind, root, &after)? == observed))
    })
}

pub fn observational_promotion_rejected(
    passive: &ValidatedPassiveAmbiguityCertificate,
) -> bool {
    passive.winners.len() != 1
}

pub fn enum_order_control_kind(
    passive: &ValidatedPassiveAmbiguityCertificate,
) -> Result<SelectionSchemaKind, SelectionError> {
    passive
        .winners
        .first()
        .copied()
        .ok_or(SelectionError::NoAbstraction)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferCase {
    pub events: Vec<Atom>,
    pub expected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionTransferTask {
    pub root_id: u64,
    pub family: String,
    pub chain: Vec<Atom>,
    pub proxy: Atom,
    pub distractors: Vec<Atom>,
    pub cases: Vec<TransferCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalAbstractionRegistry {
    cohort_id: u64,
    parent_registry_signature: String,
    admitted: BTreeSet<SelectionSchemaKind>,
    proof_ids: BTreeSet<u64>,
}

impl FinalAbstractionRegistry {
    pub fn new(
        cohort_id: u64,
        parent: &ValidatedOmegaG3SelectionParent,
    ) -> Self {
        Self {
            cohort_id,
            parent_registry_signature: parent.registry_signature.clone(),
            admitted: BTreeSet::new(),
            proof_ids: BTreeSet::new(),
        }
    }

    pub fn admitted_count(&self) -> usize {
        self.admitted.len()
    }

    pub fn supports(&self, kind: SelectionSchemaKind) -> bool {
        self.admitted.contains(&kind)
    }

    pub fn canonical_signature(&self) -> String {
        let mut values = vec![
            format!("cohort:{}", self.cohort_id),
            format!("parent:{}", self.parent_registry_signature),
        ];
        values.extend(self.admitted.iter().map(|kind| kind.name().to_string()));
        values.join("|")
    }

    pub fn admit(
        &mut self,
        certificate: &ValidatedFinalAbstractionCertificate,
    ) -> Result<(), SelectionError> {
        let before = self.clone();
        let result = self.admit_inner(certificate);
        if result.is_err() {
            *self = before;
        }
        result
    }

    fn admit_inner(
        &mut self,
        certificate: &ValidatedFinalAbstractionCertificate,
    ) -> Result<(), SelectionError> {
        if certificate.cohort_id != self.cohort_id
            || certificate.parent_registry_signature != self.parent_registry_signature
        {
            return Err(SelectionError::ForeignFinalCertificate);
        }
        if certificate.kind != SelectionSchemaKind::RecursiveAppendAdjacent {
            return Err(SelectionError::WrongFinalAbstraction);
        }
        if !self.proof_ids.insert(certificate.proof_id)
            || !self.admitted.insert(certificate.kind)
        {
            return Err(SelectionError::DuplicateFinalAdmission);
        }
        self.verify_invariants()
    }

    pub fn reject_raw_schema_injection(
        &self,
        _kind: SelectionSchemaKind,
    ) -> Result<(), SelectionError> {
        Err(SelectionError::RawSchemaInjection)
    }

    pub fn verify_invariants(&self) -> Result<(), SelectionError> {
        if self.admitted.len() != self.proof_ids.len() || self.admitted.len() > 1 {
            return Err(SelectionError::InvariantViolation(
                "final proof/schema cardinality mismatch".to_string(),
            ));
        }
        Ok(())
    }

    pub fn predict(
        &self,
        task: &SelectionTransferTask,
        events: &[Atom],
    ) -> Result<bool, SelectionError> {
        if !self.supports(SelectionSchemaKind::RecursiveAppendAdjacent)
            || self.admitted_count() != 1
        {
            return Err(SelectionError::FinalAbstractionNotAdmitted);
        }
        validate_transfer_vocabulary(task, events)?;
        Ok(chain_is_consecutive(&task.chain, events))
    }
}

pub fn diagnostic_schema_prediction(
    kind: SelectionSchemaKind,
    task: &SelectionTransferTask,
    events: &[Atom],
) -> Result<bool, SelectionError> {
    validate_transfer_vocabulary(task, events)?;
    match kind {
        SelectionSchemaKind::RecursiveAppendAdjacent => Ok(chain_is_consecutive(&task.chain, events)),
        SelectionSchemaKind::ProxyAnchorAdjacent => {
            Ok(adjacent_before(&task.proxy, &task.chain[0], events))
        }
        SelectionSchemaKind::FixedArityFourMemorizer
        | SelectionSchemaKind::SurfaceAtomLookup => Ok(false),
    }
}

fn validate_problem(problem: &AbstractionSelectionProblem) -> Result<(), SelectionError> {
    if problem.roots.len() != 8 {
        return Err(SelectionError::WrongRootCount);
    }
    let mut root_ids = BTreeSet::new();
    for root in &problem.roots {
        if !root_ids.insert(root.root_id) {
            return Err(SelectionError::DuplicateRoot);
        }
        if root.chain.len() != 4 || root.distractors.len() != 2 || root.passive.len() != 24 {
            return Err(SelectionError::FrozenFixtureMismatch);
        }
        let expected_vocab = root_vocabulary(root);
        for history in &root.passive {
            validate_exact_vocabulary(&expected_vocab, &history.events)?;
            let causal = recursive_chain_predict(root, &history.events);
            let proxy = proxy_predict(root, &history.events);
            if causal != proxy || causal != history.positive {
                return Err(SelectionError::PassiveCorrelationMismatch);
            }
        }
    }
    Ok(())
}

fn schema_predict(
    kind: SelectionSchemaKind,
    root: &SelectionRoot,
    events: &[Atom],
) -> Result<bool, SelectionError> {
    validate_exact_vocabulary(&root_vocabulary(root), events)?;
    match kind {
        SelectionSchemaKind::RecursiveAppendAdjacent => Ok(recursive_chain_predict(root, events)),
        SelectionSchemaKind::ProxyAnchorAdjacent => Ok(proxy_predict(root, events)),
        SelectionSchemaKind::FixedArityFourMemorizer
        | SelectionSchemaKind::SurfaceAtomLookup => root
            .passive
            .iter()
            .find(|history| history.events == events)
            .map(|history| history.positive)
            .ok_or(SelectionError::MemorizerMiss),
    }
}

fn recursive_chain_predict(root: &SelectionRoot, events: &[Atom]) -> bool {
    chain_is_consecutive(&root.chain, events)
}

fn proxy_predict(root: &SelectionRoot, events: &[Atom]) -> bool {
    adjacent_before(&root.proxy, &root.chain[0], events)
}

fn chain_is_consecutive(chain: &[Atom], events: &[Atom]) -> bool {
    events
        .windows(chain.len())
        .any(|window| window == chain)
}

fn adjacent_before(left: &Atom, right: &Atom, events: &[Atom]) -> bool {
    events
        .windows(2)
        .any(|pair| &pair[0] == left && &pair[1] == right)
}

fn apply_intervention(
    root: &SelectionRoot,
    events: &[Atom],
    intervention: InterventionKind,
) -> Result<Vec<Atom>, SelectionError> {
    validate_exact_vocabulary(&root_vocabulary(root), events)?;
    let mut transformed = events.to_vec();
    match intervention {
        InterventionKind::MoveProxyAfterX0 => {
            let proxy_index = position(&root.proxy, &transformed)?;
            let proxy = transformed.remove(proxy_index);
            transformed.push(proxy);
        }
        InterventionKind::SwapInternalChainEdge => {
            let left = position(&root.chain[1], &transformed)?;
            let right = position(&root.chain[2], &transformed)?;
            transformed.swap(left, right);
        }
        InterventionKind::MoveDistractorLeft => {
            let index = position(&root.distractors[0], &transformed)?;
            let value = transformed.remove(index);
            transformed.insert(0, value);
        }
        InterventionKind::MoveDistractorRight => {
            let index = position(&root.distractors[0], &transformed)?;
            let value = transformed.remove(index);
            transformed.push(value);
        }
        InterventionKind::RotateDistractors => {
            let left = position(&root.distractors[0], &transformed)?;
            let right = position(&root.distractors[1], &transformed)?;
            transformed.swap(left, right);
        }
        InterventionKind::IdentityControl => {}
    }
    validate_exact_vocabulary(&root_vocabulary(root), &transformed)?;
    Ok(transformed)
}

fn position(atom: &Atom, events: &[Atom]) -> Result<usize, SelectionError> {
    events
        .iter()
        .position(|candidate| candidate == atom)
        .ok_or(SelectionError::VocabularyMismatch)
}

fn root_vocabulary(root: &SelectionRoot) -> Vec<Atom> {
    let mut values = root.chain.clone();
    values.push(root.proxy.clone());
    values.extend(root.distractors.clone());
    values.sort();
    values
}

fn validate_transfer_vocabulary(
    task: &SelectionTransferTask,
    events: &[Atom],
) -> Result<(), SelectionError> {
    let mut expected = task.chain.clone();
    expected.push(task.proxy.clone());
    expected.extend(task.distractors.clone());
    expected.sort();
    validate_exact_vocabulary(&expected, events)
}

fn validate_exact_vocabulary(expected: &[Atom], events: &[Atom]) -> Result<(), SelectionError> {
    let mut actual = events.to_vec();
    actual.sort();
    if actual != expected {
        return Err(SelectionError::VocabularyMismatch);
    }
    let mut dedup = actual.clone();
    dedup.dedup();
    if dedup.len() != actual.len() {
        return Err(SelectionError::VocabularyMismatch);
    }
    Ok(())
}

fn digest_problem(problem: &AbstractionSelectionProblem) -> u64 {
    let mut value = mix64(problem.cohort_id ^ problem.roots.len() as u64);
    for root in &problem.roots {
        value = mix64(value ^ root.root_id.rotate_left(5) ^ hash_text(&root.family));
        value = mix64(value ^ digest_events(&root.chain));
        value = mix64(value ^ hash_text(root.proxy.as_str()));
        value = mix64(value ^ digest_events(&root.distractors));
        for history in &root.passive {
            value = mix64(
                value
                    ^ history.history_id.rotate_left(9)
                    ^ digest_events(&history.events)
                    ^ u64::from(history.positive),
            );
        }
    }
    value
}

fn digest_events(events: &[Atom]) -> u64 {
    events.iter().enumerate().fold(0x9e37_79b9_7f4a_7c15, |acc, (index, atom)| {
        mix64(acc ^ hash_text(atom.as_str()).rotate_left((index % 63) as u32 + 1))
    })
}

fn hash_text(value: &str) -> u64 {
    value.as_bytes().iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        hash.wrapping_mul(0x1000_0000_01b3) ^ u64::from(*byte)
    })
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SelectionError {
    #[error("ΩG3 parent validation failed: {0}")]
    ParentValidation(String),
    #[error("ΩG3 recursive abstraction is not admitted")]
    OmegaG3ParentNotAdmitted,
    #[error("selection problem has no roots")]
    EmptyProblem,
    #[error("selection problem must contain exactly eight roots")]
    WrongRootCount,
    #[error("duplicate root id")]
    DuplicateRoot,
    #[error("frozen fixture shape mismatch")]
    FrozenFixtureMismatch,
    #[error("passive causal/proxy correlation mismatch")]
    PassiveCorrelationMismatch,
    #[error("history vocabulary mismatch")]
    VocabularyMismatch,
    #[error("memorizer cannot evaluate an unseen history")]
    MemorizerMiss,
    #[error("passive ambiguity proof mismatch")]
    PassiveProofMismatch,
    #[error("passive ambiguity gate failed: {0}")]
    PassiveGate(&'static str),
    #[error("passive ambiguity certificate mismatch")]
    PassiveCertificateMismatch,
    #[error("missing positive calibration history")]
    MissingCalibrationHistory,
    #[error("no abstraction candidate")]
    NoAbstraction,
    #[error("no intervention candidate")]
    NoIntervention,
    #[error("intervention plan proof mismatch")]
    InterventionPlanMismatch,
    #[error("intervention gate failed: {0}")]
    InterventionGate(&'static str),
    #[error("intervention root mismatch")]
    RootMismatch,
    #[error("witness digest mismatch")]
    WitnessDigestMismatch,
    #[error("counterfeit intervention witness")]
    CounterfeitWitness,
    #[error("ΩG3 parent registry mismatch")]
    ParentRegistryMismatch,
    #[error("intervention did not resolve abstraction tie")]
    TieUnresolved,
    #[error("final selection gate failed: {0}")]
    FinalSelectionGate(&'static str),
    #[error("foreign final abstraction certificate")]
    ForeignFinalCertificate,
    #[error("wrong final abstraction")]
    WrongFinalAbstraction,
    #[error("duplicate final abstraction admission")]
    DuplicateFinalAdmission,
    #[error("raw schema injection is forbidden")]
    RawSchemaInjection,
    #[error("final abstraction is not admitted")]
    FinalAbstractionNotAdmitted,
    #[error("ΩG4 invariant violation: {0}")]
    InvariantViolation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(value: &str) -> Atom {
        Atom::new(value.to_string()).expect("atom")
    }

    fn root() -> SelectionRoot {
        let chain = vec![atom("a"), atom("b"), atom("c"), atom("d")];
        let proxy = atom("p");
        let distractors = vec![atom("u"), atom("v")];
        let mut passive = Vec::new();
        let positive = vec![
            proxy.clone(),
            chain[0].clone(),
            chain[1].clone(),
            chain[2].clone(),
            chain[3].clone(),
            distractors[0].clone(),
            distractors[1].clone(),
        ];
        passive.push(SelectionHistory {
            history_id: 1,
            events: positive,
            positive: true,
        });
        let values = {
            let mut values = chain.clone();
            values.push(proxy.clone());
            values.extend(distractors.clone());
            values
        };
        for index in 1..24 {
            let mut events = values.clone();
            events.rotate_left(index % events.len());
            if chain_is_consecutive(&chain, &events)
                == adjacent_before(&proxy, &chain[0], &events)
            {
                passive.push(SelectionHistory {
                    history_id: 1 + index as u64,
                    positive: chain_is_consecutive(&chain, &events),
                    events,
                });
            }
        }
        while passive.len() < 24 {
            let mut events = values.clone();
            let offset = passive.len() % events.len();
            events.swap(0, offset);
            if chain_is_consecutive(&chain, &events)
                == adjacent_before(&proxy, &chain[0], &events)
                && !passive.iter().any(|history| history.events == events)
            {
                passive.push(SelectionHistory {
                    history_id: 100 + passive.len() as u64,
                    positive: chain_is_consecutive(&chain, &events),
                    events,
                });
            } else {
                events.reverse();
                if chain_is_consecutive(&chain, &events)
                    == adjacent_before(&proxy, &chain[0], &events)
                    && !passive.iter().any(|history| history.events == events)
                {
                    passive.push(SelectionHistory {
                        history_id: 100 + passive.len() as u64,
                        positive: chain_is_consecutive(&chain, &events),
                        events,
                    });
                }
            }
        }
        SelectionRoot {
            root_id: 1,
            family: "test".to_string(),
            chain,
            proxy,
            distractors,
            passive,
        }
    }

    #[test]
    fn discriminating_interventions_split_predictions() {
        let root = root();
        let base = &root.passive[0].events;
        let moved = apply_intervention(&root, base, InterventionKind::MoveProxyAfterX0)
            .expect("move proxy");
        assert!(recursive_chain_predict(&root, &moved));
        assert!(!proxy_predict(&root, &moved));
        let swapped = apply_intervention(&root, base, InterventionKind::SwapInternalChainEdge)
            .expect("swap edge");
        assert!(!recursive_chain_predict(&root, &swapped));
        assert!(proxy_predict(&root, &swapped));
    }

    #[test]
    fn non_discriminating_interventions_preserve_tie() {
        let root = root();
        let base = &root.passive[0].events;
        for intervention in [
            InterventionKind::MoveDistractorLeft,
            InterventionKind::MoveDistractorRight,
            InterventionKind::RotateDistractors,
            InterventionKind::IdentityControl,
        ] {
            let transformed = apply_intervention(&root, base, intervention).expect("transform");
            assert_eq!(
                recursive_chain_predict(&root, &transformed),
                proxy_predict(&root, &transformed)
            );
        }
    }
}
