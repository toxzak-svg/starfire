//! ΩG3 bounded multi-step abstraction and reuse.
//!
//! This module is deterministic, offline, and shadow-only. It independently
//! revalidates the admitted ΩG2 child, synthesizes concrete lower-level chain
//! expressions, factors validated examples through a frozen four-schema
//! meta-grammar, and requires proof-bound admission before reuse can occur.

use crate::commitment_state::Atom;
use crate::recursive_grammar_composition::{
    validate_recursive_composition, ComposedGrammarRegistry, ComposedProductionKind,
    ParentLineage, RecursiveCompositionBudget, RecursiveCompositionProblem,
    RecursiveCompositionProof, ValidatedParentHandle,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabeledChainHistory {
    pub history_id: u64,
    pub events: Vec<Atom>,
    pub positive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainTask {
    pub root_id: u64,
    pub discovery: Vec<LabeledChainHistory>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BoundChainProgram {
    binding: Vec<Atom>,
}

impl BoundChainProgram {
    pub fn binding(&self) -> &[Atom] {
        &self.binding
    }

    pub fn arity(&self) -> usize {
        self.binding.len()
    }

    pub fn execute(&self, events: &[Atom]) -> bool {
        events == self.binding.as_slice()
    }

    pub fn canonical_string(&self) -> String {
        let args = self
            .binding
            .iter()
            .map(|atom| atom.as_str())
            .collect::<Vec<_>>()
            .join(",");
        format!("chain({args})")
    }

    fn edge_signature(&self) -> Vec<(Atom, Atom)> {
        self.binding
            .windows(2)
            .map(|pair| (pair[0].clone(), pair[1].clone()))
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcreteSynthesisBudget {
    pub tasks: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcreteChainProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub arity: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
    pub exact_candidates: usize,
    pub tree_shape_rank: usize,
    pub node_cost: usize,
    pub program: BoundChainProgram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedConcreteSolutionCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    arity: usize,
    node_cost: usize,
    program: BoundChainProgram,
}

impl ValidatedConcreteSolutionCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn node_cost(&self) -> usize {
        self.node_cost
    }

    pub fn program(&self) -> &BoundChainProgram {
        &self.program
    }

    fn lineage(&self) -> ConcreteSolutionLineage {
        ConcreteSolutionLineage {
            root_id: self.root_id,
            proof_id: self.proof_id,
            problem_digest: self.problem_digest,
            arity: self.arity,
            canonical_expression: self.program.canonical_string(),
        }
    }
}

pub fn synthesize_concrete_chain(
    task: &ChainTask,
    budget: &mut ConcreteSynthesisBudget,
) -> Result<ConcreteChainProof, AbstractionReuseError> {
    search_concrete_chain(task, budget)
}

pub fn validate_concrete_chain(
    task: &ChainTask,
    supplied: &ConcreteChainProof,
    budget: &mut ConcreteSynthesisBudget,
) -> Result<ValidatedConcreteSolutionCertificate, AbstractionReuseError> {
    let recomputed = search_concrete_chain(task, budget)?;
    if supplied != &recomputed {
        return Err(AbstractionReuseError::ConcreteProofMismatch);
    }
    Ok(ValidatedConcreteSolutionCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        arity: recomputed.arity,
        node_cost: recomputed.node_cost,
        program: recomputed.program,
    })
}

fn search_concrete_chain(
    task: &ChainTask,
    budget: &mut ConcreteSynthesisBudget,
) -> Result<ConcreteChainProof, AbstractionReuseError> {
    let vocabulary = validate_task(task)?;
    let arity = vocabulary.len();
    let shapes = catalan(arity - 3);
    let expected_candidates = factorial(arity).saturating_mul(shapes);
    let expected_evaluations = expected_candidates.saturating_mul(task.discovery.len());
    let mut exact_candidates = 0usize;
    let mut winner: Option<(BoundChainProgram, usize)> = None;

    budget.tasks = budget.tasks.saturating_add(1);
    for binding in permutations(&vocabulary) {
        let program = BoundChainProgram { binding };
        for shape_rank in 0..shapes {
            budget.candidate_programs = budget.candidate_programs.saturating_add(1);
            let mut exact = true;
            for history in &task.discovery {
                budget.program_history_evaluations =
                    budget.program_history_evaluations.saturating_add(1);
                if program.execute(&history.events) != history.positive {
                    exact = false;
                }
            }
            if exact {
                exact_candidates = exact_candidates.saturating_add(1);
                let candidate_key = (program.canonical_string(), shape_rank);
                let replace = winner
                    .as_ref()
                    .map(|(current, rank)| {
                        candidate_key < (current.canonical_string(), *rank)
                    })
                    .unwrap_or(true);
                if replace {
                    winner = Some((program.clone(), shape_rank));
                }
            }
        }
    }

    if budget.candidate_programs < expected_candidates
        || budget.program_history_evaluations < expected_evaluations
    {
        return Err(AbstractionReuseError::BudgetInvariant);
    }
    if exact_candidates != shapes {
        return Err(AbstractionReuseError::ConcreteExactCount {
            expected: shapes,
            actual: exact_candidates,
        });
    }
    let (program, tree_shape_rank) =
        winner.ok_or(AbstractionReuseError::NoConcreteSolution)?;
    let problem_digest = digest_task(task);
    let node_cost = arity.saturating_mul(2).saturating_sub(5);
    let proof_id = mix64(
        problem_digest
            ^ task.root_id.rotate_left(7)
            ^ (arity as u64).rotate_left(19)
            ^ hash_text(&program.canonical_string()),
    );
    Ok(ConcreteChainProof {
        proof_id,
        root_id: task.root_id,
        problem_digest,
        arity,
        candidate_programs: expected_candidates,
        program_history_evaluations: expected_evaluations,
        exact_candidates,
        tree_shape_rank,
        node_cost,
        program,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmegaG2AbstractionLineage {
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub proof_id: u64,
    pub admitted_kind: ComposedProductionKind,
    pub registry_signature: String,
    pub nested_parent: ParentLineage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedOmegaG2AbstractionParent {
    lineage: OmegaG2AbstractionLineage,
}

impl ValidatedOmegaG2AbstractionParent {
    pub fn lineage(&self) -> &OmegaG2AbstractionLineage {
        &self.lineage
    }
}

pub fn revalidate_omega_g2_abstraction_parent(
    problem: &RecursiveCompositionProblem,
    omega_g1_parent: &ValidatedParentHandle,
    proof: &RecursiveCompositionProof,
    registry: &ComposedGrammarRegistry,
    budget: &mut RecursiveCompositionBudget,
) -> Result<ValidatedOmegaG2AbstractionParent, AbstractionReuseError> {
    let certificate = validate_recursive_composition(problem, omega_g1_parent, proof, budget)
        .map_err(|error| AbstractionReuseError::ParentValidation(error.to_string()))?;
    if certificate.child_kind() != ComposedProductionKind::ConsecutiveChain3
        || proof.child_kind != ComposedProductionKind::ConsecutiveChain3
        || proof.child_arity != 3
    {
        return Err(AbstractionReuseError::WrongParentKind);
    }
    if registry.cohort_id() != problem.cohort_id
        || registry.admitted_count() != 1
        || !registry.supports(ComposedProductionKind::ConsecutiveChain3)
    {
        return Err(AbstractionReuseError::ParentRegistryMismatch);
    }
    if certificate.cohort_id() != problem.cohort_id
        || certificate.problem_digest() != proof.problem_digest
        || certificate.proof_id() != proof.proof_id
    {
        return Err(AbstractionReuseError::ParentLineageMismatch);
    }
    Ok(ValidatedOmegaG2AbstractionParent {
        lineage: OmegaG2AbstractionLineage {
            cohort_id: problem.cohort_id,
            problem_digest: proof.problem_digest,
            proof_id: proof.proof_id,
            admitted_kind: proof.child_kind,
            registry_signature: registry.canonical_signature(),
            nested_parent: registry.parent_lineage().clone(),
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AbstractionSchemaKind {
    RecursiveAppendAdjacent,
    EndpointBridge,
    DisjointAdjacentPairs,
    FixedArityThree,
}

impl AbstractionSchemaKind {
    pub fn all() -> [Self; 4] {
        [
            Self::RecursiveAppendAdjacent,
            Self::EndpointBridge,
            Self::DisjointAdjacentPairs,
            Self::FixedArityThree,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::RecursiveAppendAdjacent => "recursive_append_adjacent",
            Self::EndpointBridge => "endpoint_bridge",
            Self::DisjointAdjacentPairs => "disjoint_adjacent_pairs",
            Self::FixedArityThree => "fixed_arity_three",
        }
    }

    pub fn node_cost(self) -> usize {
        match self {
            Self::RecursiveAppendAdjacent => 5,
            Self::EndpointBridge => 3,
            Self::DisjointAdjacentPairs => 4,
            Self::FixedArityThree => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcreteSolutionLineage {
    pub root_id: u64,
    pub proof_id: u64,
    pub problem_digest: u64,
    pub arity: usize,
    pub canonical_expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAnalysis {
    pub kind: AbstractionSchemaKind,
    pub exact_examples: usize,
    pub node_cost: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractionSearchBudget {
    pub schema_candidates: usize,
    pub schema_example_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractionProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub parent_lineage: OmegaG2AbstractionLineage,
    pub examples: Vec<ConcreteSolutionLineage>,
    pub analyses: Vec<SchemaAnalysis>,
    pub winner: AbstractionSchemaKind,
    pub exact_schema_count: usize,
    pub concrete_node_cost: usize,
    pub schema_node_cost: usize,
    pub compression_advantage: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAbstractionCertificate {
    cohort_id: u64,
    proof_id: u64,
    problem_digest: u64,
    parent_lineage: OmegaG2AbstractionLineage,
    kind: AbstractionSchemaKind,
}

impl ValidatedAbstractionCertificate {
    pub fn cohort_id(&self) -> u64 {
        self.cohort_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn kind(&self) -> AbstractionSchemaKind {
        self.kind
    }
}

pub fn synthesize_abstraction(
    cohort_id: u64,
    parent: &ValidatedOmegaG2AbstractionParent,
    examples: &[ValidatedConcreteSolutionCertificate],
    budget: &mut AbstractionSearchBudget,
) -> Result<AbstractionProof, AbstractionReuseError> {
    search_abstraction(cohort_id, parent, examples, budget)
}

pub fn validate_abstraction(
    cohort_id: u64,
    parent: &ValidatedOmegaG2AbstractionParent,
    examples: &[ValidatedConcreteSolutionCertificate],
    supplied: &AbstractionProof,
    budget: &mut AbstractionSearchBudget,
) -> Result<ValidatedAbstractionCertificate, AbstractionReuseError> {
    let recomputed = search_abstraction(cohort_id, parent, examples, budget)?;
    if supplied != &recomputed {
        return Err(AbstractionReuseError::AbstractionProofMismatch);
    }
    enforce_abstraction_gates(&recomputed)?;
    Ok(ValidatedAbstractionCertificate {
        cohort_id: recomputed.cohort_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        parent_lineage: recomputed.parent_lineage,
        kind: recomputed.winner,
    })
}

fn search_abstraction(
    cohort_id: u64,
    parent: &ValidatedOmegaG2AbstractionParent,
    examples: &[ValidatedConcreteSolutionCertificate],
    budget: &mut AbstractionSearchBudget,
) -> Result<AbstractionProof, AbstractionReuseError> {
    validate_example_support(examples)?;
    let mut lineages = examples
        .iter()
        .map(ValidatedConcreteSolutionCertificate::lineage)
        .collect::<Vec<_>>();
    lineages.sort_by_key(|lineage| (lineage.arity, lineage.root_id));
    let concrete_node_cost = examples.iter().map(|example| example.node_cost()).sum();
    let mut analyses = Vec::new();
    for kind in AbstractionSchemaKind::all() {
        budget.schema_candidates = budget.schema_candidates.saturating_add(1);
        let mut exact_examples = 0usize;
        for example in examples {
            budget.schema_example_evaluations =
                budget.schema_example_evaluations.saturating_add(1);
            if schema_edges(kind, example.program()) == example.program().edge_signature() {
                exact_examples = exact_examples.saturating_add(1);
            }
        }
        analyses.push(SchemaAnalysis {
            kind,
            exact_examples,
            node_cost: kind.node_cost(),
        });
    }
    let exact = analyses
        .iter()
        .filter(|analysis| analysis.exact_examples == examples.len())
        .collect::<Vec<_>>();
    let winner = exact
        .iter()
        .min_by_key(|analysis| (analysis.node_cost, analysis.kind))
        .ok_or(AbstractionReuseError::NoAbstraction)?
        .kind;
    let exact_schema_count = exact.len();
    let schema_node_cost = winner.node_cost();
    let compression_advantage = concrete_node_cost.saturating_sub(schema_node_cost);
    let problem_digest = digest_abstraction(cohort_id, parent.lineage(), &lineages);
    let proof_id = mix64(
        problem_digest
            ^ hash_text(winner.name())
            ^ (compression_advantage as u64).rotate_left(23),
    );
    Ok(AbstractionProof {
        proof_id,
        cohort_id,
        problem_digest,
        parent_lineage: parent.lineage.clone(),
        examples: lineages,
        analyses,
        winner,
        exact_schema_count,
        concrete_node_cost,
        schema_node_cost,
        compression_advantage,
    })
}

fn enforce_abstraction_gates(proof: &AbstractionProof) -> Result<(), AbstractionReuseError> {
    if proof.winner != AbstractionSchemaKind::RecursiveAppendAdjacent {
        return Err(AbstractionReuseError::AbstractionGate("wrong winner"));
    }
    if proof.exact_schema_count != 1 {
        return Err(AbstractionReuseError::AbstractionGate(
            "winner is not unique",
        ));
    }
    if proof.examples.len() != 12
        || proof.concrete_node_cost != 36
        || proof.schema_node_cost != 5
        || proof.compression_advantage != 31
    {
        return Err(AbstractionReuseError::AbstractionGate(
            "frozen support or compression mismatch",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractionRegistry {
    cohort_id: u64,
    parent_lineage: OmegaG2AbstractionLineage,
    admitted: BTreeSet<AbstractionSchemaKind>,
    proof_ids: BTreeSet<u64>,
}

impl AbstractionRegistry {
    pub fn new(cohort_id: u64, parent: &ValidatedOmegaG2AbstractionParent) -> Self {
        Self {
            cohort_id,
            parent_lineage: parent.lineage.clone(),
            admitted: BTreeSet::new(),
            proof_ids: BTreeSet::new(),
        }
    }

    pub fn cohort_id(&self) -> u64 {
        self.cohort_id
    }

    pub fn admitted_count(&self) -> usize {
        self.admitted.len()
    }

    pub fn supports(&self, kind: AbstractionSchemaKind) -> bool {
        self.admitted.contains(&kind)
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![
            format!("cohort:{}", self.cohort_id),
            format!(
                "parent:{}:{}:{}:{}",
                self.parent_lineage.cohort_id,
                self.parent_lineage.problem_digest,
                self.parent_lineage.proof_id,
                self.parent_lineage.registry_signature
            ),
        ];
        parts.extend(self.admitted.iter().map(|kind| kind.name().to_string()));
        parts.join("|")
    }

    pub fn admit(
        &mut self,
        certificate: &ValidatedAbstractionCertificate,
    ) -> Result<(), AbstractionReuseError> {
        let before = self.clone();
        let result = self.admit_inner(certificate);
        if result.is_err() {
            *self = before;
        }
        result
    }

    fn admit_inner(
        &mut self,
        certificate: &ValidatedAbstractionCertificate,
    ) -> Result<(), AbstractionReuseError> {
        if certificate.cohort_id != self.cohort_id {
            return Err(AbstractionReuseError::ForeignAbstractionCertificate);
        }
        if certificate.parent_lineage != self.parent_lineage {
            return Err(AbstractionReuseError::ParentRegistryMismatch);
        }
        if certificate.kind != AbstractionSchemaKind::RecursiveAppendAdjacent {
            return Err(AbstractionReuseError::WrongAbstractionKind);
        }
        if !self.proof_ids.insert(certificate.proof_id)
            || !self.admitted.insert(certificate.kind)
        {
            return Err(AbstractionReuseError::DuplicateAbstractionAdmission);
        }
        self.verify_invariants()
    }

    pub fn reject_raw_schema_injection(
        &self,
        _kind: AbstractionSchemaKind,
    ) -> Result<(), AbstractionReuseError> {
        Err(AbstractionReuseError::RawSchemaInjection)
    }

    pub fn verify_invariants(&self) -> Result<(), AbstractionReuseError> {
        if self.admitted.len() != self.proof_ids.len() || self.admitted.len() > 1 {
            return Err(AbstractionReuseError::InvariantViolation(
                "abstraction proof/schema cardinality mismatch".to_string(),
            ));
        }
        if self.parent_lineage.admitted_kind
            != ComposedProductionKind::ConsecutiveChain3
        {
            return Err(AbstractionReuseError::InvariantViolation(
                "abstraction registry lost ΩG2 parent".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseSynthesisBudget {
    pub tasks: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub registry_signature: String,
    pub arity: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
    pub exact_candidates: usize,
    pub program: BoundChainProgram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedReuseCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    registry_signature: String,
    program: BoundChainProgram,
}

impl ValidatedReuseCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn program(&self) -> &BoundChainProgram {
        &self.program
    }
}

pub fn synthesize_abstraction_reuse(
    task: &ChainTask,
    registry: &AbstractionRegistry,
    budget: &mut ReuseSynthesisBudget,
) -> Result<ReuseProof, AbstractionReuseError> {
    search_reuse(task, registry, budget)
}

pub fn validate_abstraction_reuse(
    task: &ChainTask,
    registry: &AbstractionRegistry,
    supplied: &ReuseProof,
    budget: &mut ReuseSynthesisBudget,
) -> Result<ValidatedReuseCertificate, AbstractionReuseError> {
    let recomputed = search_reuse(task, registry, budget)?;
    if supplied != &recomputed {
        return Err(AbstractionReuseError::ReuseProofMismatch);
    }
    if recomputed.exact_candidates != 1 {
        return Err(AbstractionReuseError::ReuseGate(
            "reuse winner is not unique",
        ));
    }
    Ok(ValidatedReuseCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        registry_signature: recomputed.registry_signature,
        program: recomputed.program,
    })
}

fn search_reuse(
    task: &ChainTask,
    registry: &AbstractionRegistry,
    budget: &mut ReuseSynthesisBudget,
) -> Result<ReuseProof, AbstractionReuseError> {
    if !registry.supports(AbstractionSchemaKind::RecursiveAppendAdjacent)
        || registry.admitted_count() != 1
    {
        return Err(AbstractionReuseError::AbstractionNotAdmitted);
    }
    let vocabulary = validate_task(task)?;
    let expected_candidates = factorial(vocabulary.len());
    let expected_evaluations = expected_candidates.saturating_mul(task.discovery.len());
    let mut exact_candidates = 0usize;
    let mut winner: Option<BoundChainProgram> = None;
    budget.tasks = budget.tasks.saturating_add(1);
    for binding in permutations(&vocabulary) {
        budget.candidate_programs = budget.candidate_programs.saturating_add(1);
        let program = BoundChainProgram { binding };
        let mut exact = true;
        for history in &task.discovery {
            budget.program_history_evaluations =
                budget.program_history_evaluations.saturating_add(1);
            if program.execute(&history.events) != history.positive {
                exact = false;
            }
        }
        if exact {
            exact_candidates = exact_candidates.saturating_add(1);
            let replace = winner
                .as_ref()
                .map(|current| program.canonical_string() < current.canonical_string())
                .unwrap_or(true);
            if replace {
                winner = Some(program);
            }
        }
    }
    if budget.candidate_programs < expected_candidates
        || budget.program_history_evaluations < expected_evaluations
    {
        return Err(AbstractionReuseError::BudgetInvariant);
    }
    let program = winner.ok_or(AbstractionReuseError::NoReuseSolution)?;
    let problem_digest = digest_task(task);
    let registry_signature = registry.canonical_signature();
    let proof_id = mix64(
        problem_digest
            ^ hash_text(&registry_signature)
            ^ hash_text(&program.canonical_string()),
    );
    Ok(ReuseProof {
        proof_id,
        root_id: task.root_id,
        problem_digest,
        registry_signature,
        arity: vocabulary.len(),
        candidate_programs: expected_candidates,
        program_history_evaluations: expected_evaluations,
        exact_candidates,
        program,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstractionStateLanguage {
    root_id: u64,
    registry_signature: String,
    program: Option<BoundChainProgram>,
    proof_ids: BTreeSet<u64>,
}

impl AbstractionStateLanguage {
    pub fn new(root_id: u64, registry: &AbstractionRegistry) -> Self {
        Self {
            root_id,
            registry_signature: registry.canonical_signature(),
            program: None,
            proof_ids: BTreeSet::new(),
        }
    }

    pub fn admit_local(
        &mut self,
        certificate: &ValidatedReuseCertificate,
    ) -> Result<(), AbstractionReuseError> {
        let before = self.clone();
        let result = self.admit_local_inner(certificate);
        if result.is_err() {
            *self = before;
        }
        result
    }

    fn admit_local_inner(
        &mut self,
        certificate: &ValidatedReuseCertificate,
    ) -> Result<(), AbstractionReuseError> {
        if certificate.root_id != self.root_id {
            return Err(AbstractionReuseError::ForeignReuseCertificate);
        }
        if certificate.registry_signature != self.registry_signature {
            return Err(AbstractionReuseError::AbstractionRegistryMismatch);
        }
        if !self.proof_ids.insert(certificate.proof_id) || self.program.is_some() {
            return Err(AbstractionReuseError::DuplicateReuseAdmission);
        }
        self.program = Some(certificate.program.clone());
        self.verify_invariants()
    }

    pub fn predict(&self, events: &[Atom]) -> Option<bool> {
        self.program.as_ref().map(|program| program.execute(events))
    }

    pub fn refinement_count(&self) -> usize {
        usize::from(self.program.is_some())
    }

    pub fn verify_invariants(&self) -> Result<(), AbstractionReuseError> {
        if self.proof_ids.len() != usize::from(self.program.is_some())
            || self.proof_ids.len() > 1
        {
            return Err(AbstractionReuseError::InvariantViolation(
                "reuse proof/program cardinality mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AbstractionReuseError {
    #[error("chain task has no discovery evidence")]
    EmptyTask,
    #[error("chain task arity must be at least three")]
    ArityTooSmall,
    #[error("chain task histories do not share one exact vocabulary")]
    VocabularyMismatch,
    #[error("chain task must contain exactly one positive discovery history")]
    PositiveCountMismatch,
    #[error("concrete lower-level proof mismatch")]
    ConcreteProofMismatch,
    #[error("expected {expected} exact concrete candidates but observed {actual}")]
    ConcreteExactCount { expected: usize, actual: usize },
    #[error("no exact lower-level concrete solution")]
    NoConcreteSolution,
    #[error("ΩG2 parent validation failed: {0}")]
    ParentValidation(String),
    #[error("ΩG2 parent is not ConsecutiveChain3")]
    WrongParentKind,
    #[error("ΩG2 parent proof lineage mismatch")]
    ParentLineageMismatch,
    #[error("ΩG2 parent registry mismatch")]
    ParentRegistryMismatch,
    #[error("abstraction requires twelve independently validated examples")]
    WrongExampleCount,
    #[error("abstraction support must contain four examples at each arity 3, 4, and 5")]
    WrongExampleSupport,
    #[error("duplicate concrete example root")]
    DuplicateExampleRoot,
    #[error("no abstraction fits all validated examples")]
    NoAbstraction,
    #[error("abstraction proof mismatch")]
    AbstractionProofMismatch,
    #[error("abstraction certificate gate failed: {0}")]
    AbstractionGate(&'static str),
    #[error("foreign abstraction certificate")]
    ForeignAbstractionCertificate,
    #[error("wrong abstraction kind")]
    WrongAbstractionKind,
    #[error("duplicate abstraction admission")]
    DuplicateAbstractionAdmission,
    #[error("raw schema injection is forbidden")]
    RawSchemaInjection,
    #[error("abstraction is not admitted")]
    AbstractionNotAdmitted,
    #[error("reuse proof mismatch")]
    ReuseProofMismatch,
    #[error("reuse certificate gate failed: {0}")]
    ReuseGate(&'static str),
    #[error("no exact reuse solution")]
    NoReuseSolution,
    #[error("foreign reuse certificate")]
    ForeignReuseCertificate,
    #[error("reuse certificate is bound to another abstraction registry")]
    AbstractionRegistryMismatch,
    #[error("duplicate reuse admission")]
    DuplicateReuseAdmission,
    #[error("frozen candidate or execution budget invariant failed")]
    BudgetInvariant,
    #[error("ΩG3 invariant violation: {0}")]
    InvariantViolation(String),
}

fn validate_task(task: &ChainTask) -> Result<Vec<Atom>, AbstractionReuseError> {
    let first = task.discovery.first().ok_or(AbstractionReuseError::EmptyTask)?;
    if first.events.len() < 3 {
        return Err(AbstractionReuseError::ArityTooSmall);
    }
    let mut vocabulary = first.events.clone();
    vocabulary.sort();
    vocabulary.dedup();
    if vocabulary.len() != first.events.len() {
        return Err(AbstractionReuseError::VocabularyMismatch);
    }
    let positives = task
        .discovery
        .iter()
        .filter(|history| history.positive)
        .count();
    if positives != 1 {
        return Err(AbstractionReuseError::PositiveCountMismatch);
    }
    for history in &task.discovery {
        let mut current = history.events.clone();
        current.sort();
        current.dedup();
        if current != vocabulary || history.events.len() != vocabulary.len() {
            return Err(AbstractionReuseError::VocabularyMismatch);
        }
    }
    Ok(vocabulary)
}

fn validate_example_support(
    examples: &[ValidatedConcreteSolutionCertificate],
) -> Result<(), AbstractionReuseError> {
    if examples.len() != 12 {
        return Err(AbstractionReuseError::WrongExampleCount);
    }
    let roots = examples
        .iter()
        .map(|example| example.root_id())
        .collect::<BTreeSet<_>>();
    if roots.len() != examples.len() {
        return Err(AbstractionReuseError::DuplicateExampleRoot);
    }
    let mut counts = BTreeMap::<usize, usize>::new();
    for example in examples {
        *counts.entry(example.arity()).or_insert(0) += 1;
    }
    if counts.get(&3) != Some(&4)
        || counts.get(&4) != Some(&4)
        || counts.get(&5) != Some(&4)
        || counts.len() != 3
    {
        return Err(AbstractionReuseError::WrongExampleSupport);
    }
    Ok(())
}

fn schema_edges(
    kind: AbstractionSchemaKind,
    program: &BoundChainProgram,
) -> Vec<(Atom, Atom)> {
    let binding = program.binding();
    match kind {
        AbstractionSchemaKind::RecursiveAppendAdjacent => program.edge_signature(),
        AbstractionSchemaKind::EndpointBridge => vec![(
            binding[0].clone(),
            binding[binding.len() - 1].clone(),
        )],
        AbstractionSchemaKind::DisjointAdjacentPairs => binding
            .chunks_exact(2)
            .map(|pair| (pair[0].clone(), pair[1].clone()))
            .collect(),
        AbstractionSchemaKind::FixedArityThree if binding.len() == 3 => {
            program.edge_signature()
        }
        AbstractionSchemaKind::FixedArityThree => Vec::new(),
    }
}

fn digest_task(task: &ChainTask) -> u64 {
    let mut hash = mix64(task.root_id ^ 0x0a63_1001);
    for history in &task.discovery {
        hash = mix64(
            hash ^ history.history_id ^ if history.positive { 1 } else { 0 },
        );
        for atom in &history.events {
            hash = mix64(hash ^ hash_text(atom.as_str()));
        }
    }
    hash
}

fn digest_abstraction(
    cohort_id: u64,
    parent: &OmegaG2AbstractionLineage,
    examples: &[ConcreteSolutionLineage],
) -> u64 {
    let mut hash = mix64(
        cohort_id
            ^ parent.cohort_id
            ^ parent.problem_digest
            ^ parent.proof_id
            ^ hash_text(&parent.registry_signature),
    );
    for example in examples {
        hash = mix64(
            hash
                ^ example.root_id
                ^ example.proof_id
                ^ example.problem_digest
                ^ (example.arity as u64)
                ^ hash_text(&example.canonical_expression),
        );
    }
    hash
}

fn permutations(values: &[Atom]) -> Vec<Vec<Atom>> {
    fn visit(prefix: &mut Vec<Atom>, remaining: &mut Vec<Atom>, out: &mut Vec<Vec<Atom>>) {
        if remaining.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..remaining.len() {
            let value = remaining.remove(index);
            prefix.push(value.clone());
            visit(prefix, remaining, out);
            prefix.pop();
            remaining.insert(index, value);
        }
    }
    let mut out = Vec::new();
    visit(&mut Vec::new(), &mut values.to_vec(), &mut out);
    out
}

fn factorial(value: usize) -> usize {
    (1..=value).product()
}

fn catalan(index: usize) -> usize {
    let mut values = vec![0usize; index + 1];
    values[0] = 1;
    for n in 1..=index {
        values[n] = (0..n)
            .map(|left| values[left].saturating_mul(values[n - 1 - left]))
            .sum();
    }
    values[index]
}

fn hash_text(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod multistep_abstraction_reuse_tests {
    use super::*;

    fn atom(value: &str) -> Atom {
        Atom::new(value.to_string()).expect("valid atom")
    }

    fn task(arity: usize) -> ChainTask {
        let canonical = (0..arity)
            .map(|index| atom(&format!("a{index}")))
            .collect::<Vec<_>>();
        let mut discovery = vec![LabeledChainHistory {
            history_id: 1,
            events: canonical.clone(),
            positive: true,
        }];
        for index in 0..arity - 1 {
            let mut events = canonical.clone();
            events.swap(index, index + 1);
            discovery.push(LabeledChainHistory {
                history_id: 2 + index as u64,
                events,
                positive: false,
            });
        }
        let mut reversed = canonical;
        reversed.reverse();
        discovery.push(LabeledChainHistory {
            history_id: 100,
            events: reversed,
            positive: false,
        });
        ChainTask {
            root_id: arity as u64,
            discovery,
        }
    }

    #[test]
    fn frozen_concrete_counts_match() {
        let expected = [
            (3, 6, 24, 1),
            (4, 24, 120, 1),
            (5, 240, 1_440, 2),
            (6, 3_600, 25_200, 5),
            (7, 70_560, 564_480, 14),
        ];
        for (arity, candidates, executions, exact) in expected {
            let mut budget = ConcreteSynthesisBudget::default();
            let proof =
                synthesize_concrete_chain(&task(arity), &mut budget).expect("synthesis");
            assert_eq!(proof.candidate_programs, candidates);
            assert_eq!(proof.program_history_evaluations, executions);
            assert_eq!(proof.exact_candidates, exact);
        }
    }

    #[test]
    fn catalan_sequence_is_frozen() {
        assert_eq!((0..5).map(catalan).collect::<Vec<_>>(), vec![1, 1, 2, 5, 14]);
    }
}
