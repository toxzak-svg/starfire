//! ΩG2 recursive grammar composition.
//!
//! This module is offline, deterministic, and shadow-only. It requires an
//! independently revalidated ΩG1 `AdjacentBefore` production as an executable
//! parent, composes that parent through the frozen `SharedMiddleAnd` operator,
//! independently validates the generic arity-3 child, and requires a second
//! independently validated root-local binding before a state-key bit exists.

use crate::commitment_state::Atom;
use crate::grammar_extension::{
    validate_grammar_extension, BoundExtensionProgram, ExtensionKind, GrammarExtensionBudget,
    GrammarExtensionConfig, GrammarExtensionProblem, GrammarExtensionProof, GrammarRegistry,
    GrammarRoot,
};
use crate::representation_genesis::{
    derive_vocabulary, detect_alias_defects, enumerate_programs, AliasDefect, GenesisBudget,
    RawHistory, RefinementProblem, StateLanguage,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompositionOperator {
    SharedMiddleAnd,
}

impl CompositionOperator {
    pub fn name(self) -> &'static str {
        match self {
            Self::SharedMiddleAnd => "shared_middle_and",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComposedProductionKind {
    ConsecutiveChain3,
}

impl ComposedProductionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::ConsecutiveChain3 => "consecutive_chain3",
        }
    }

    pub fn arity(self) -> usize {
        match self {
            Self::ConsecutiveChain3 => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentLineage {
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub proof_id: u64,
    pub admitted_kind: ExtensionKind,
    pub registry_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedParentHandle {
    lineage: ParentLineage,
}

impl ValidatedParentHandle {
    pub fn lineage(&self) -> &ParentLineage {
        &self.lineage
    }

    pub fn kind(&self) -> ExtensionKind {
        self.lineage.admitted_kind
    }
}

pub fn revalidate_parent(
    problem: &GrammarExtensionProblem,
    proof: &GrammarExtensionProof,
    registry: &GrammarRegistry,
    config: GrammarExtensionConfig,
    budget: &mut GrammarExtensionBudget,
) -> Result<ValidatedParentHandle, RecursiveCompositionError> {
    let certificate = validate_grammar_extension(problem, proof, config, budget)
        .map_err(|error| RecursiveCompositionError::ParentValidation(error.to_string()))?;
    if proof.winner.kind != ExtensionKind::AdjacentBefore
        || certificate.kind() != ExtensionKind::AdjacentBefore
    {
        return Err(RecursiveCompositionError::WrongParentKind);
    }
    if certificate.cohort_id() != problem.cohort_id
        || certificate.proof_id() != proof.proof_id
        || certificate.problem_digest() != proof.problem_digest
    {
        return Err(RecursiveCompositionError::ParentLineageMismatch);
    }
    if registry.cohort_id() != problem.cohort_id
        || registry.admitted_count() != 1
        || !registry.supports(ExtensionKind::AdjacentBefore)
    {
        return Err(RecursiveCompositionError::ParentRegistryMismatch);
    }
    Ok(ValidatedParentHandle {
        lineage: ParentLineage {
            cohort_id: problem.cohort_id,
            problem_digest: proof.problem_digest,
            proof_id: proof.proof_id,
            admitted_kind: ExtensionKind::AdjacentBefore,
            registry_signature: registry.canonical_signature(),
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecursiveCompositionProblem {
    pub cohort_id: u64,
    pub roots: Vec<GrammarRoot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecursiveCompositionBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub base_candidate_programs: usize,
    pub base_program_history_evaluations: usize,
    pub single_m1_candidate_programs: usize,
    pub single_m1_program_history_evaluations: usize,
    pub c1_candidate_programs: usize,
    pub c1_program_history_evaluations: usize,
    pub unique_base_partitions: usize,
    pub unique_single_m1_partitions: usize,
    pub unique_c1_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundComposedProgram {
    first: Atom,
    middle: Atom,
    last: Atom,
}

impl BoundComposedProgram {
    pub fn new(first: Atom, middle: Atom, last: Atom) -> Result<Self, RecursiveCompositionError> {
        if first == middle || first == last || middle == last {
            return Err(RecursiveCompositionError::DegenerateBinding);
        }
        Ok(Self {
            first,
            middle,
            last,
        })
    }

    pub fn first(&self) -> &Atom {
        &self.first
    }

    pub fn middle(&self) -> &Atom {
        &self.middle
    }

    pub fn last(&self) -> &Atom {
        &self.last
    }

    pub fn execute(&self, history: &RawHistory) -> bool {
        let Some(first) = history.events.iter().position(|atom| atom == &self.first) else {
            return false;
        };
        let Some(middle) = history.events.iter().position(|atom| atom == &self.middle) else {
            return false;
        };
        let Some(last) = history.events.iter().position(|atom| atom == &self.last) else {
            return false;
        };
        middle == first.saturating_add(1) && last == middle.saturating_add(1)
    }

    pub fn canonical_string(&self) -> String {
        format!(
            "consecutive_chain3({},{},{})",
            self.first.as_str(),
            self.middle.as_str(),
            self.last.as_str()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootCompositionAnalysis {
    pub root_id: u64,
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
    pub local_winner: BoundComposedProgram,
    pub local_winner_support_min: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecursiveCompositionProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub parent_lineage: ParentLineage,
    pub root_count: usize,
    pub operator: CompositionOperator,
    pub child_kind: ComposedProductionKind,
    pub child_arity: usize,
    pub exact_roots: usize,
    pub root_analyses: Vec<RootCompositionAnalysis>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedComposedProductionCertificate {
    cohort_id: u64,
    proof_id: u64,
    problem_digest: u64,
    parent_lineage: ParentLineage,
    operator: CompositionOperator,
    child_kind: ComposedProductionKind,
}

impl ValidatedComposedProductionCertificate {
    pub fn cohort_id(&self) -> u64 {
        self.cohort_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }

    pub fn parent_lineage(&self) -> &ParentLineage {
        &self.parent_lineage
    }

    pub fn operator(&self) -> CompositionOperator {
        self.operator
    }

    pub fn child_kind(&self) -> ComposedProductionKind {
        self.child_kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedGrammarRegistry {
    cohort_id: u64,
    parent_lineage: ParentLineage,
    admitted: BTreeSet<ComposedProductionKind>,
    proof_ids: BTreeSet<u64>,
}

impl ComposedGrammarRegistry {
    pub fn new(cohort_id: u64, parent: &ValidatedParentHandle) -> Self {
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

    pub fn parent_lineage(&self) -> &ParentLineage {
        &self.parent_lineage
    }

    pub fn admitted_count(&self) -> usize {
        self.admitted.len()
    }

    pub fn supports(&self, kind: ComposedProductionKind) -> bool {
        self.admitted.contains(&kind)
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![
            format!("cohort:{}", self.cohort_id),
            format!(
                "parent:{}:{}:{}:{}:{}",
                self.parent_lineage.cohort_id,
                self.parent_lineage.problem_digest,
                self.parent_lineage.proof_id,
                self.parent_lineage.admitted_kind.name(),
                self.parent_lineage.registry_signature
            ),
        ];
        parts.extend(self.admitted.iter().map(|kind| kind.name().to_string()));
        parts.join("|")
    }

    pub fn admit(
        &mut self,
        certificate: &ValidatedComposedProductionCertificate,
    ) -> Result<(), RecursiveCompositionError> {
        let before = self.clone();
        let result = self.admit_inner(certificate);
        if result.is_err() {
            *self = before;
        }
        result
    }

    fn admit_inner(
        &mut self,
        certificate: &ValidatedComposedProductionCertificate,
    ) -> Result<(), RecursiveCompositionError> {
        if certificate.cohort_id != self.cohort_id {
            return Err(RecursiveCompositionError::ForeignChildCertificate);
        }
        if certificate.parent_lineage != self.parent_lineage {
            return Err(RecursiveCompositionError::ParentRegistryMismatch);
        }
        if certificate.operator != CompositionOperator::SharedMiddleAnd
            || certificate.child_kind != ComposedProductionKind::ConsecutiveChain3
        {
            return Err(RecursiveCompositionError::ChildCertificateMismatch);
        }
        if !self.proof_ids.insert(certificate.proof_id)
            || !self.admitted.insert(certificate.child_kind)
        {
            return Err(RecursiveCompositionError::DuplicateChildAdmission);
        }
        self.verify_invariants()
    }

    pub fn reject_raw_schema_injection(
        &self,
        _kind: ExtensionKind,
    ) -> Result<(), RecursiveCompositionError> {
        Err(RecursiveCompositionError::RawSchemaInjection)
    }

    pub fn verify_invariants(&self) -> Result<(), RecursiveCompositionError> {
        if self.admitted.len() != self.proof_ids.len() || self.admitted.len() > 1 {
            return Err(RecursiveCompositionError::InvariantViolation(
                "child registry proof/production cardinality mismatch".to_string(),
            ));
        }
        if self.parent_lineage.admitted_kind != ExtensionKind::AdjacentBefore {
            return Err(RecursiveCompositionError::InvariantViolation(
                "child registry lost required ΩG1 parent".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn audit_counterfeit_child_rejected(
    registry: &ComposedGrammarRegistry,
    certificate: &ValidatedComposedProductionCertificate,
) -> bool {
    let mut counterfeit = certificate.clone();
    counterfeit.proof_id ^= 1;
    let mut trial = registry.clone();
    let before = trial.clone();
    trial.admit(&counterfeit).is_err() && trial == before
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalComposedRefinementBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
    pub unique_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalComposedRefinementProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub registry_signature: String,
    pub detected_defects: usize,
    pub candidate_programs: usize,
    pub unique_partitions: usize,
    pub complete_repair_count: usize,
    pub program: BoundComposedProgram,
    pub canonical_partition: Vec<bool>,
    pub repaired_defects: usize,
    pub unrepaired_defects: usize,
    pub support_min: usize,
    pub runner_up_repaired_defects: usize,
    pub winner_margin: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLocalComposedRefinementCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    registry_signature: String,
    program: BoundComposedProgram,
}

impl ValidatedLocalComposedRefinementCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }

    pub fn program(&self) -> &BoundComposedProgram {
        &self.program
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ComposedStateKey {
    pub intervention: Atom,
    pub base_multiset: Vec<(Atom, usize)>,
    pub refinement_bits: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedStateLanguage {
    root_id: u64,
    registry_signature: String,
    refinements: Vec<BoundComposedProgram>,
    proof_ids: BTreeSet<u64>,
}

impl ComposedStateLanguage {
    pub fn new(root_id: u64, registry: &ComposedGrammarRegistry) -> Self {
        Self {
            root_id,
            registry_signature: registry.canonical_signature(),
            refinements: Vec::new(),
            proof_ids: BTreeSet::new(),
        }
    }

    pub fn refinement_count(&self) -> usize {
        self.refinements.len()
    }

    pub fn admit_local(
        &mut self,
        certificate: &ValidatedLocalComposedRefinementCertificate,
    ) -> Result<(), RecursiveCompositionError> {
        let before = self.clone();
        let result = self.admit_local_inner(certificate);
        if result.is_err() {
            *self = before;
        }
        result
    }

    fn admit_local_inner(
        &mut self,
        certificate: &ValidatedLocalComposedRefinementCertificate,
    ) -> Result<(), RecursiveCompositionError> {
        if certificate.root_id != self.root_id {
            return Err(RecursiveCompositionError::ForeignLocalCertificate);
        }
        if certificate.registry_signature != self.registry_signature {
            return Err(RecursiveCompositionError::ChildRegistryMismatch);
        }
        if !self.proof_ids.insert(certificate.proof_id) {
            return Err(RecursiveCompositionError::DuplicateLocalRefinement);
        }
        self.refinements.push(certificate.program.clone());
        self.verify_invariants()
    }

    pub fn state_key(&self, history: &RawHistory, intervention: &Atom) -> ComposedStateKey {
        let mut counts = BTreeMap::<Atom, usize>::new();
        for atom in &history.events {
            *counts.entry(atom.clone()).or_insert(0) += 1;
        }
        ComposedStateKey {
            intervention: intervention.clone(),
            base_multiset: counts.into_iter().collect(),
            refinement_bits: self
                .refinements
                .iter()
                .map(|program| program.execute(history))
                .collect(),
        }
    }

    pub fn verify_invariants(&self) -> Result<(), RecursiveCompositionError> {
        if self.refinements.len() != self.proof_ids.len() || self.refinements.len() > 1 {
            return Err(RecursiveCompositionError::InvariantViolation(
                "local composed proof/program cardinality mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RecursiveCompositionError {
    #[error("ΩG1 parent validation failed: {0}")]
    ParentValidation(String),
    #[error("ΩG1 parent is not AdjacentBefore")]
    WrongParentKind,
    #[error("ΩG1 parent proof/certificate lineage mismatch")]
    ParentLineageMismatch,
    #[error("ΩG1 parent registry lineage mismatch")]
    ParentRegistryMismatch,
    #[error("recursive composition problem has no roots")]
    EmptyCohort,
    #[error("recursive composition root {0} has no discovery histories")]
    EmptyRoot(u64),
    #[error("duplicate recursive composition root id {0}")]
    DuplicateRoot(u64),
    #[error("root analysis failed: {0}")]
    RootAnalysis(String),
    #[error("no candidate partition")]
    NoCandidatePartition,
    #[error("composed program requires three distinct atoms")]
    DegenerateBinding,
    #[error("recursive composition proof mismatch")]
    ProofMismatch,
    #[error("recursive composition certificate gate failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign child certificate")]
    ForeignChildCertificate,
    #[error("child certificate does not match the frozen operator or production")]
    ChildCertificateMismatch,
    #[error("duplicate child admission")]
    DuplicateChildAdmission,
    #[error("raw ΩG1 schema values cannot substitute for an admitted parent handle")]
    RawSchemaInjection,
    #[error("local composed refinement proof mismatch")]
    LocalProofMismatch,
    #[error("local composed refinement certificate gate failed: {0}")]
    LocalCertificateGate(&'static str),
    #[error("foreign local composed certificate")]
    ForeignLocalCertificate,
    #[error("local composed certificate is bound to a different child registry")]
    ChildRegistryMismatch,
    #[error("duplicate local composed refinement")]
    DuplicateLocalRefinement,
    #[error("recursive composition invariant violation: {0}")]
    InvariantViolation(String),
}

#[derive(Debug, Clone)]
struct PartitionScore {
    repaired: usize,
    support_min: usize,
    bits: Vec<bool>,
    syntax: String,
}

pub fn synthesize_recursive_composition(
    problem: &RecursiveCompositionProblem,
    parent: &ValidatedParentHandle,
    budget: &mut RecursiveCompositionBudget,
) -> Result<RecursiveCompositionProof, RecursiveCompositionError> {
    search_recursive_composition(problem, parent, budget)
}

pub fn validate_recursive_composition(
    problem: &RecursiveCompositionProblem,
    parent: &ValidatedParentHandle,
    supplied: &RecursiveCompositionProof,
    budget: &mut RecursiveCompositionBudget,
) -> Result<ValidatedComposedProductionCertificate, RecursiveCompositionError> {
    let recomputed = search_recursive_composition(problem, parent, budget)?;
    if supplied != &recomputed {
        return Err(RecursiveCompositionError::ProofMismatch);
    }
    enforce_recursive_gates(&recomputed)?;
    Ok(ValidatedComposedProductionCertificate {
        cohort_id: recomputed.cohort_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        parent_lineage: recomputed.parent_lineage,
        operator: recomputed.operator,
        child_kind: recomputed.child_kind,
    })
}

pub fn synthesize_local_composed_refinement(
    root: &GrammarRoot,
    registry: &ComposedGrammarRegistry,
    budget: &mut LocalComposedRefinementBudget,
) -> Result<LocalComposedRefinementProof, RecursiveCompositionError> {
    search_local_composed_refinement(root, registry, budget)
}

pub fn validate_local_composed_refinement(
    root: &GrammarRoot,
    registry: &ComposedGrammarRegistry,
    supplied: &LocalComposedRefinementProof,
    budget: &mut LocalComposedRefinementBudget,
) -> Result<ValidatedLocalComposedRefinementCertificate, RecursiveCompositionError> {
    let recomputed = search_local_composed_refinement(root, registry, budget)?;
    if supplied != &recomputed {
        return Err(RecursiveCompositionError::LocalProofMismatch);
    }
    enforce_local_gates(&recomputed)?;
    Ok(ValidatedLocalComposedRefinementCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        registry_signature: recomputed.registry_signature,
        program: recomputed.program,
    })
}

fn search_recursive_composition(
    problem: &RecursiveCompositionProblem,
    parent: &ValidatedParentHandle,
    budget: &mut RecursiveCompositionBudget,
) -> Result<RecursiveCompositionProof, RecursiveCompositionError> {
    validate_problem(problem)?;
    if parent.kind() != ExtensionKind::AdjacentBefore {
        return Err(RecursiveCompositionError::WrongParentKind);
    }

    let mut analyses = Vec::with_capacity(problem.roots.len());
    for root in &problem.roots {
        analyses.push(analyze_root(root, budget)?);
    }
    let exact_roots = analyses
        .iter()
        .filter(|analysis| {
            analysis.c1_best_repaired_defects == analysis.detected_defects
                && analysis.c1_complete_repair_count == 1
        })
        .count();
    let problem_digest = digest_problem(problem);
    let proof_id = digest_recursive_proof(
        problem.cohort_id,
        problem_digest,
        parent.lineage(),
        &analyses,
    );
    Ok(RecursiveCompositionProof {
        proof_id,
        cohort_id: problem.cohort_id,
        problem_digest,
        parent_lineage: parent.lineage.clone(),
        root_count: problem.roots.len(),
        operator: CompositionOperator::SharedMiddleAnd,
        child_kind: ComposedProductionKind::ConsecutiveChain3,
        child_arity: 3,
        exact_roots,
        root_analyses: analyses,
    })
}

fn analyze_root(
    root: &GrammarRoot,
    budget: &mut RecursiveCompositionBudget,
) -> Result<RootCompositionAnalysis, RecursiveCompositionError> {
    if root.discovery.is_empty() {
        return Err(RecursiveCompositionError::EmptyRoot(root.root_id));
    }
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)
        .map_err(|error| RecursiveCompositionError::RootAnalysis(error.to_string()))?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)
        .map_err(|error| RecursiveCompositionError::RootAnalysis(error.to_string()))?;
    budget.vocabulary_history_scans = budget
        .vocabulary_history_scans
        .saturating_add(genesis.vocabulary_history_scans);
    budget.history_pair_evaluations = budget
        .history_pair_evaluations
        .saturating_add(genesis.history_pair_evaluations);
    let evidence_index = evidence_index(&root.discovery);

    let base_programs = enumerate_programs(&vocabulary);
    let base_candidate_count = base_programs.len();
    budget.base_candidate_programs = budget
        .base_candidate_programs
        .saturating_add(base_candidate_count);
    let mut base_partitions = BTreeMap::<Vec<bool>, String>::new();
    for program in base_programs {
        let bits = root
            .discovery
            .iter()
            .map(|episode| {
                budget.base_program_history_evaluations = budget
                    .base_program_history_evaluations
                    .saturating_add(1);
                program.execute(&episode.history)
            })
            .collect::<Vec<_>>();
        let canonical = canonical_partition(bits);
        let syntax = program.canonical_string();
        match base_partitions.get_mut(&canonical) {
            Some(existing) if syntax < *existing => *existing = syntax,
            Some(_) => {}
            None => {
                base_partitions.insert(canonical, syntax);
            }
        }
    }
    budget.unique_base_partitions = budget
        .unique_base_partitions
        .saturating_add(base_partitions.len());
    let mut base_scores = base_partitions
        .into_iter()
        .map(|(bits, syntax)| score_partition(bits, syntax, &defects, &evidence_index))
        .collect::<Vec<_>>();
    rank_scores(&mut base_scores);
    let base_best = base_scores
        .first()
        .ok_or(RecursiveCompositionError::NoCandidatePartition)?
        .repaired;
    let base_unique = base_scores.len();

    let mut m1_partitions = BTreeMap::<Vec<bool>, BoundExtensionProgram>::new();
    let mut m1_candidate_count = 0usize;
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
                m1_candidate_count = m1_candidate_count.saturating_add(1);
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
                let canonical = canonical_partition(bits);
                match m1_partitions.get_mut(&canonical) {
                    Some(existing)
                        if program.canonical_string() < existing.canonical_string() =>
                    {
                        *existing = program;
                    }
                    Some(_) => {}
                    None => {
                        m1_partitions.insert(canonical, program);
                    }
                }
            }
        }
    }
    budget.unique_single_m1_partitions = budget
        .unique_single_m1_partitions
        .saturating_add(m1_partitions.len());
    let mut m1_scores = m1_partitions
        .into_iter()
        .map(|(bits, program)| {
            score_partition(
                bits,
                program.canonical_string(),
                &defects,
                &evidence_index,
            )
        })
        .collect::<Vec<_>>();
    rank_scores(&mut m1_scores);
    let m1_best = m1_scores
        .first()
        .ok_or(RecursiveCompositionError::NoCandidatePartition)?
        .repaired;
    let m1_unique = m1_scores.len();

    let mut c1_partitions = BTreeMap::<Vec<bool>, BoundComposedProgram>::new();
    let mut c1_candidate_count = 0usize;
    for first in &vocabulary {
        for middle in &vocabulary {
            for last in &vocabulary {
                if first == middle || first == last || middle == last {
                    continue;
                }
                let program =
                    BoundComposedProgram::new(first.clone(), middle.clone(), last.clone())?;
                c1_candidate_count = c1_candidate_count.saturating_add(1);
                budget.c1_candidate_programs = budget.c1_candidate_programs.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        budget.c1_program_history_evaluations = budget
                            .c1_program_history_evaluations
                            .saturating_add(1);
                        program.execute(&episode.history)
                    })
                    .collect::<Vec<_>>();
                let canonical = canonical_partition(bits);
                match c1_partitions.get_mut(&canonical) {
                    Some(existing)
                        if program.canonical_string() < existing.canonical_string() =>
                    {
                        *existing = program;
                    }
                    Some(_) => {}
                    None => {
                        c1_partitions.insert(canonical, program);
                    }
                }
            }
        }
    }
    budget.unique_c1_partitions = budget
        .unique_c1_partitions
        .saturating_add(c1_partitions.len());
    let mut c1_scores = c1_partitions
        .into_iter()
        .map(|(bits, program)| {
            let score = score_partition(
                bits,
                program.canonical_string(),
                &defects,
                &evidence_index,
            );
            (score, program)
        })
        .collect::<Vec<_>>();
    c1_scores.sort_by(|left, right| compare_score(&left.0, &right.0));
    let complete_count = c1_scores
        .iter()
        .filter(|(score, _)| score.repaired == defects.len())
        .count();
    let (winner_score, winner_program) = c1_scores
        .first()
        .cloned()
        .ok_or(RecursiveCompositionError::NoCandidatePartition)?;
    let c1_unique = c1_scores.len();

    Ok(RootCompositionAnalysis {
        root_id: root.root_id,
        detected_defects: defects.len(),
        base_candidate_programs: base_candidate_count,
        base_unique_partitions: base_unique,
        base_best_repaired_defects: base_best,
        single_m1_candidate_programs: m1_candidate_count,
        single_m1_unique_partitions: m1_unique,
        single_m1_best_repaired_defects: m1_best,
        c1_candidate_programs: c1_candidate_count,
        c1_unique_partitions: c1_unique,
        c1_best_repaired_defects: winner_score.repaired,
        c1_complete_repair_count: complete_count,
        local_winner: winner_program,
        local_winner_support_min: winner_score.support_min,
    })
}

fn search_local_composed_refinement(
    root: &GrammarRoot,
    registry: &ComposedGrammarRegistry,
    budget: &mut LocalComposedRefinementBudget,
) -> Result<LocalComposedRefinementProof, RecursiveCompositionError> {
    if !registry.supports(ComposedProductionKind::ConsecutiveChain3) {
        return Err(RecursiveCompositionError::LocalCertificateGate(
            "composed child is not admitted",
        ));
    }
    if root.discovery.is_empty() {
        return Err(RecursiveCompositionError::EmptyRoot(root.root_id));
    }
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)
        .map_err(|error| RecursiveCompositionError::RootAnalysis(error.to_string()))?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)
        .map_err(|error| RecursiveCompositionError::RootAnalysis(error.to_string()))?;
    budget.vocabulary_history_scans = budget
        .vocabulary_history_scans
        .saturating_add(genesis.vocabulary_history_scans);
    budget.history_pair_evaluations = budget
        .history_pair_evaluations
        .saturating_add(genesis.history_pair_evaluations);
    let evidence_index = evidence_index(&root.discovery);

    let mut partitions = BTreeMap::<Vec<bool>, BoundComposedProgram>::new();
    for first in &vocabulary {
        for middle in &vocabulary {
            for last in &vocabulary {
                if first == middle || first == last || middle == last {
                    continue;
                }
                let program =
                    BoundComposedProgram::new(first.clone(), middle.clone(), last.clone())?;
                budget.candidate_programs = budget.candidate_programs.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        budget.program_history_evaluations =
                            budget.program_history_evaluations.saturating_add(1);
                        program.execute(&episode.history)
                    })
                    .collect::<Vec<_>>();
                let canonical = canonical_partition(bits);
                match partitions.get_mut(&canonical) {
                    Some(existing)
                        if program.canonical_string() < existing.canonical_string() =>
                    {
                        *existing = program;
                    }
                    Some(_) => {}
                    None => {
                        partitions.insert(canonical, program);
                    }
                }
            }
        }
    }
    budget.unique_partitions = budget
        .unique_partitions
        .saturating_add(partitions.len());
    let mut scored = partitions
        .into_iter()
        .map(|(bits, program)| {
            let score = score_partition(
                bits,
                program.canonical_string(),
                &defects,
                &evidence_index,
            );
            (score, program)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| compare_score(&left.0, &right.0));
    let complete_repair_count = scored
        .iter()
        .filter(|(score, _)| score.repaired == defects.len())
        .count();
    let (winner, program) = scored
        .first()
        .cloned()
        .ok_or(RecursiveCompositionError::NoCandidatePartition)?;
    let runner_up = scored.get(1).map(|entry| entry.0.repaired).unwrap_or(0);
    let problem_digest = digest_root(root);
    let proof_id = digest_local_proof(
        root.root_id,
        problem_digest,
        &registry.canonical_signature(),
        &program,
        &winner.bits,
    );
    Ok(LocalComposedRefinementProof {
        proof_id,
        root_id: root.root_id,
        problem_digest,
        registry_signature: registry.canonical_signature(),
        detected_defects: defects.len(),
        candidate_programs: budget.candidate_programs,
        unique_partitions: scored.len(),
        complete_repair_count,
        program,
        canonical_partition: winner.bits,
        repaired_defects: winner.repaired,
        unrepaired_defects: defects.len().saturating_sub(winner.repaired),
        support_min: winner.support_min,
        runner_up_repaired_defects: runner_up,
        winner_margin: winner.repaired.saturating_sub(runner_up),
    })
}

fn enforce_recursive_gates(
    proof: &RecursiveCompositionProof,
) -> Result<(), RecursiveCompositionError> {
    if proof.operator != CompositionOperator::SharedMiddleAnd
        || proof.child_kind != ComposedProductionKind::ConsecutiveChain3
        || proof.child_arity != 3
    {
        return Err(RecursiveCompositionError::CertificateGate(
            "wrong frozen operator or child",
        ));
    }
    if proof.parent_lineage.admitted_kind != ExtensionKind::AdjacentBefore {
        return Err(RecursiveCompositionError::CertificateGate(
            "wrong executable parent",
        ));
    }
    if proof.root_count == 0 || proof.root_count != proof.root_analyses.len() {
        return Err(RecursiveCompositionError::CertificateGate(
            "root cardinality mismatch",
        ));
    }
    if proof.exact_roots != proof.root_count {
        return Err(RecursiveCompositionError::CertificateGate(
            "not every development root has a unique complete repair",
        ));
    }
    for root in &proof.root_analyses {
        if root.detected_defects != 368
            || root.base_candidate_programs != 315
            || root.base_unique_partitions != 26
            || root.base_best_repaired_defects != 204
            || root.single_m1_candidate_programs != 60
            || root.single_m1_unique_partitions != 60
            || root.single_m1_best_repaired_defects != 328
            || root.c1_candidate_programs != 60
            || root.c1_unique_partitions != 60
            || root.c1_best_repaired_defects != 368
            || root.c1_complete_repair_count != 1
        {
            return Err(RecursiveCompositionError::CertificateGate(
                "frozen expressibility ladder mismatch",
            ));
        }
    }
    Ok(())
}

fn enforce_local_gates(
    proof: &LocalComposedRefinementProof,
) -> Result<(), RecursiveCompositionError> {
    if proof.detected_defects != 368
        || proof.candidate_programs != 60
        || proof.unique_partitions != 60
        || proof.complete_repair_count != 1
        || proof.repaired_defects != 368
        || proof.unrepaired_defects != 0
    {
        return Err(RecursiveCompositionError::LocalCertificateGate(
            "local child synthesis did not reproduce the frozen complete repair",
        ));
    }
    Ok(())
}

fn validate_problem(problem: &RecursiveCompositionProblem) -> Result<(), RecursiveCompositionError> {
    if problem.roots.is_empty() {
        return Err(RecursiveCompositionError::EmptyCohort);
    }
    let mut root_ids = BTreeSet::new();
    for root in &problem.roots {
        if root.discovery.is_empty() {
            return Err(RecursiveCompositionError::EmptyRoot(root.root_id));
        }
        if !root_ids.insert(root.root_id) {
            return Err(RecursiveCompositionError::DuplicateRoot(root.root_id));
        }
    }
    Ok(())
}

fn evidence_index(
    discovery: &[crate::representation_genesis::WitnessedHistory],
) -> BTreeMap<u64, usize> {
    discovery
        .iter()
        .enumerate()
        .map(|(index, episode)| (episode.evidence_id, index))
        .collect()
}

fn score_partition(
    bits: Vec<bool>,
    syntax: String,
    defects: &[AliasDefect],
    evidence_index: &BTreeMap<u64, usize>,
) -> PartitionScore {
    let repaired = defects
        .iter()
        .filter(|defect| {
            let left = evidence_index[&defect.left_evidence_id];
            let right = evidence_index[&defect.right_evidence_id];
            bits[left] != bits[right]
        })
        .count();
    let true_count = bits.iter().filter(|bit| **bit).count();
    let support_min = true_count.min(bits.len().saturating_sub(true_count));
    PartitionScore {
        repaired,
        support_min,
        bits,
        syntax,
    }
}

fn rank_scores(scores: &mut [PartitionScore]) {
    scores.sort_by(compare_score);
}

fn compare_score(left: &PartitionScore, right: &PartitionScore) -> std::cmp::Ordering {
    right
        .repaired
        .cmp(&left.repaired)
        .then_with(|| right.support_min.cmp(&left.support_min))
        .then_with(|| left.syntax.cmp(&right.syntax))
}

fn canonical_partition(bits: Vec<bool>) -> Vec<bool> {
    let inverted = bits.iter().map(|bit| !*bit).collect::<Vec<_>>();
    if inverted < bits {
        inverted
    } else {
        bits
    }
}

fn digest_problem(problem: &RecursiveCompositionProblem) -> u64 {
    let mut bytes = Vec::new();
    push_u64(&mut bytes, problem.cohort_id);
    for root in &problem.roots {
        append_root(&mut bytes, root);
    }
    digest_bytes(&bytes)
}

fn digest_root(root: &GrammarRoot) -> u64 {
    let mut bytes = Vec::new();
    append_root(&mut bytes, root);
    digest_bytes(&bytes)
}

fn append_root(bytes: &mut Vec<u8>, root: &GrammarRoot) {
    push_u64(bytes, root.root_id);
    for episode in &root.discovery {
        push_u64(bytes, episode.evidence_id);
        push_u64(bytes, episode.history.history_id);
        for atom in &episode.history.events {
            push_text(bytes, atom.as_str());
        }
        push_text(bytes, episode.intervention.as_str());
        push_text(bytes, episode.outcome.as_str());
    }
}

fn digest_recursive_proof(
    cohort_id: u64,
    problem_digest: u64,
    parent: &ParentLineage,
    analyses: &[RootCompositionAnalysis],
) -> u64 {
    let mut bytes = Vec::new();
    push_u64(&mut bytes, cohort_id);
    push_u64(&mut bytes, problem_digest);
    push_u64(&mut bytes, parent.cohort_id);
    push_u64(&mut bytes, parent.problem_digest);
    push_u64(&mut bytes, parent.proof_id);
    push_text(&mut bytes, parent.admitted_kind.name());
    push_text(&mut bytes, &parent.registry_signature);
    for analysis in analyses {
        push_u64(&mut bytes, analysis.root_id);
        push_u64(&mut bytes, analysis.detected_defects as u64);
        push_u64(&mut bytes, analysis.base_best_repaired_defects as u64);
        push_u64(
            &mut bytes,
            analysis.single_m1_best_repaired_defects as u64,
        );
        push_u64(&mut bytes, analysis.c1_best_repaired_defects as u64);
        push_text(&mut bytes, &analysis.local_winner.canonical_string());
    }
    digest_bytes(&bytes)
}

fn digest_local_proof(
    root_id: u64,
    problem_digest: u64,
    registry_signature: &str,
    program: &BoundComposedProgram,
    partition: &[bool],
) -> u64 {
    let mut bytes = Vec::new();
    push_u64(&mut bytes, root_id);
    push_u64(&mut bytes, problem_digest);
    push_text(&mut bytes, registry_signature);
    push_text(&mut bytes, &program.canonical_string());
    for bit in partition {
        bytes.push(u8::from(*bit));
    }
    digest_bytes(&bytes)
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_text(bytes: &mut Vec<u8>, value: &str) {
    push_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn digest_bytes(bytes: &[u8]) -> u64 {
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

    #[test]
    fn composed_program_requires_distinct_atoms() {
        let atom = Atom::new("same").expect("atom");
        assert_eq!(
            BoundComposedProgram::new(atom.clone(), atom.clone(), atom),
            Err(RecursiveCompositionError::DegenerateBinding)
        );
    }

    #[test]
    fn composed_program_executes_two_parent_links() {
        let first = Atom::new("a").expect("atom");
        let middle = Atom::new("b").expect("atom");
        let last = Atom::new("c").expect("atom");
        let spare = Atom::new("d").expect("atom");
        let program =
            BoundComposedProgram::new(first.clone(), middle.clone(), last.clone()).expect("program");
        let positive = RawHistory {
            history_id: 1,
            events: vec![spare.clone(), first.clone(), middle.clone(), last.clone()],
        };
        let negative = RawHistory {
            history_id: 2,
            events: vec![first, spare, middle, last],
        };
        assert!(program.execute(&positive));
        assert!(!program.execute(&negative));
    }
}
