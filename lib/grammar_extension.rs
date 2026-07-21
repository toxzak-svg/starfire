//! ΩG1 bounded grammar extension.
//!
//! This module is offline and shadow-only. It exhausts the inherited Ω1
//! single-comparison grammar, ranks a frozen three-production meta-grammar,
//! independently validates one generic production, and requires a second
//! independently validated local binding before a new state-key bit exists.

use crate::commitment_state::Atom;
use crate::representation_genesis::{
    derive_vocabulary, detect_alias_defects, enumerate_programs, AliasDefect, GenesisBudget,
    RawHistory, RefinementProblem, StateLanguage, WitnessedHistory,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExtensionKind {
    AdjacentBefore,
    ExactlyOneBetween,
    WithinTwoBefore,
}

impl ExtensionKind {
    pub fn all() -> [Self; 3] {
        [
            Self::AdjacentBefore,
            Self::ExactlyOneBetween,
            Self::WithinTwoBefore,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::AdjacentBefore => "adjacent_before",
            Self::ExactlyOneBetween => "exactly_one_between",
            Self::WithinTwoBefore => "within_two_before",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BoundExtensionProgram {
    pub kind: ExtensionKind,
    pub left: Atom,
    pub right: Atom,
}

impl BoundExtensionProgram {
    pub fn execute(&self, history: &RawHistory) -> bool {
        let Some(left) = history.events.iter().position(|atom| atom == &self.left) else {
            return false;
        };
        let Some(right) = history.events.iter().position(|atom| atom == &self.right) else {
            return false;
        };
        match self.kind {
            ExtensionKind::AdjacentBefore => right == left.saturating_add(1),
            ExtensionKind::ExactlyOneBetween => right == left.saturating_add(2),
            ExtensionKind::WithinTwoBefore => {
                left < right && right.saturating_sub(left) <= 2
            }
        }
    }

    pub fn canonical_string(&self) -> String {
        format!(
            "{}({},{})",
            self.kind.name(),
            self.left.as_str(),
            self.right.as_str()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarRoot {
    pub root_id: u64,
    pub discovery: Vec<WitnessedHistory>,
}

impl GrammarRoot {
    fn as_refinement_problem(&self) -> RefinementProblem {
        RefinementProblem {
            root_id: self.root_id,
            discovery: self.discovery.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarExtensionProblem {
    pub cohort_id: u64,
    pub roots: Vec<GrammarRoot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarExtensionConfig {
    pub min_partition_support: usize,
    pub min_exact_root_margin: usize,
}

impl Default for GrammarExtensionConfig {
    fn default() -> Self {
        Self {
            min_partition_support: 8,
            min_exact_root_margin: 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarExtensionBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub base_candidate_programs: usize,
    pub base_program_history_evaluations: usize,
    pub extension_bound_candidates: usize,
    pub extension_program_history_evaluations: usize,
    pub unique_base_partitions: usize,
    pub unique_extension_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootInsufficiency {
    pub root_id: u64,
    pub detected_defects: usize,
    pub base_candidate_programs: usize,
    pub base_unique_partitions: usize,
    pub base_best_repaired_defects: usize,
    pub base_best_program: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSchemaWinner {
    pub root_id: u64,
    pub program: BoundExtensionProgram,
    pub repaired_defects: usize,
    pub support_min: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaScore {
    pub kind: ExtensionKind,
    pub exact_roots: usize,
    pub total_repaired_defects: usize,
    pub total_detected_defects: usize,
    pub aggregate_support_min: usize,
    pub local_winners: Vec<LocalSchemaWinner>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarExtensionProof {
    pub proof_id: u64,
    pub cohort_id: u64,
    pub problem_digest: u64,
    pub root_count: usize,
    pub root_insufficiency: Vec<RootInsufficiency>,
    pub winner: SchemaScore,
    pub runner_up_exact_roots: usize,
    pub exact_root_margin: usize,
    pub schema_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedGrammarExtensionCertificate {
    cohort_id: u64,
    proof_id: u64,
    problem_digest: u64,
    kind: ExtensionKind,
}

impl ValidatedGrammarExtensionCertificate {
    pub fn cohort_id(&self) -> u64 {
        self.cohort_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }

    pub fn kind(&self) -> ExtensionKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarRegistry {
    cohort_id: u64,
    admitted: BTreeSet<ExtensionKind>,
}

impl GrammarRegistry {
    pub fn new(cohort_id: u64) -> Self {
        Self {
            cohort_id,
            admitted: BTreeSet::new(),
        }
    }

    pub fn cohort_id(&self) -> u64 {
        self.cohort_id
    }

    pub fn admitted_count(&self) -> usize {
        self.admitted.len()
    }

    pub fn supports(&self, kind: ExtensionKind) -> bool {
        self.admitted.contains(&kind)
    }

    pub fn admitted_kinds(&self) -> Vec<ExtensionKind> {
        self.admitted.iter().copied().collect()
    }

    pub fn admit(
        &mut self,
        certificate: &ValidatedGrammarExtensionCertificate,
    ) -> Result<(), GrammarExtensionError> {
        if certificate.cohort_id != self.cohort_id {
            return Err(GrammarExtensionError::ForeignCertificate {
                expected_cohort: self.cohort_id,
                certificate_cohort: certificate.cohort_id,
            });
        }
        if !self.admitted.insert(certificate.kind) {
            return Err(GrammarExtensionError::DuplicateProduction(
                certificate.kind,
            ));
        }
        self.verify_invariants()
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![format!("cohort:{}", self.cohort_id)];
        parts.extend(self.admitted.iter().map(|kind| kind.name().to_string()));
        parts.join("|")
    }

    pub fn verify_invariants(&self) -> Result<(), GrammarExtensionError> {
        if self.admitted.len() > ExtensionKind::all().len() {
            return Err(GrammarExtensionError::InvariantViolation(
                "registry exceeds frozen meta-grammar".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRefinementBudget {
    pub vocabulary_history_scans: usize,
    pub history_pair_evaluations: usize,
    pub candidate_programs: usize,
    pub program_history_evaluations: usize,
    pub unique_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRefinementProof {
    pub proof_id: u64,
    pub root_id: u64,
    pub problem_digest: u64,
    pub registry_signature: String,
    pub detected_defects: usize,
    pub candidate_programs: usize,
    pub unique_partitions: usize,
    pub program: BoundExtensionProgram,
    pub canonical_partition: Vec<bool>,
    pub repaired_defects: usize,
    pub unrepaired_defects: usize,
    pub support_min: usize,
    pub runner_up_repaired_defects: usize,
    pub winner_margin: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLocalRefinementCertificate {
    root_id: u64,
    proof_id: u64,
    problem_digest: u64,
    registry_signature: String,
    program: BoundExtensionProgram,
}

impl ValidatedLocalRefinementCertificate {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }

    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn problem_digest(&self) -> u64 {
        self.problem_digest
    }

    pub fn program(&self) -> &BoundExtensionProgram {
        &self.program
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExtendedStateKey {
    pub intervention: Atom,
    pub base_multiset: Vec<(Atom, usize)>,
    pub refinement_bits: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarExtendedStateLanguage {
    root_id: u64,
    registry_signature: String,
    refinements: Vec<BoundExtensionProgram>,
    proof_ids: BTreeSet<u64>,
}

impl GrammarExtendedStateLanguage {
    pub fn new(root_id: u64, registry: &GrammarRegistry) -> Self {
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
        certificate: &ValidatedLocalRefinementCertificate,
    ) -> Result<(), GrammarExtensionError> {
        if certificate.root_id != self.root_id {
            return Err(GrammarExtensionError::ForeignLocalCertificate {
                expected_root: self.root_id,
                certificate_root: certificate.root_id,
            });
        }
        if certificate.registry_signature != self.registry_signature {
            return Err(GrammarExtensionError::RegistryMismatch);
        }
        if !self.proof_ids.insert(certificate.proof_id) {
            return Err(GrammarExtensionError::DuplicateLocalRefinement(
                certificate.proof_id,
            ));
        }
        self.refinements.push(certificate.program.clone());
        self.verify_invariants()
    }

    pub fn state_key(&self, history: &RawHistory, intervention: &Atom) -> ExtendedStateKey {
        let mut counts = BTreeMap::<Atom, usize>::new();
        for atom in &history.events {
            *counts.entry(atom.clone()).or_insert(0) += 1;
        }
        ExtendedStateKey {
            intervention: intervention.clone(),
            base_multiset: counts.into_iter().collect(),
            refinement_bits: self
                .refinements
                .iter()
                .map(|program| program.execute(history))
                .collect(),
        }
    }

    pub fn canonical_signature(&self) -> String {
        let mut parts = vec![
            format!("root:{}", self.root_id),
            self.registry_signature.clone(),
        ];
        parts.extend(
            self.refinements
                .iter()
                .map(BoundExtensionProgram::canonical_string),
        );
        parts.join("|")
    }

    pub fn verify_invariants(&self) -> Result<(), GrammarExtensionError> {
        if self.refinements.len() != self.proof_ids.len() {
            return Err(GrammarExtensionError::InvariantViolation(
                "local proof/program cardinality mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GrammarExtensionError {
    #[error("grammar-extension problem has no roots")]
    EmptyCohort,
    #[error("grammar root {0} has no discovery histories")]
    EmptyRoot(u64),
    #[error("duplicate grammar root id {0}")]
    DuplicateRoot(u64),
    #[error("root analysis failed: {0}")]
    RootAnalysis(String),
    #[error("no candidate partitions")]
    NoCandidatePartition,
    #[error("grammar-extension proof mismatch: {0}")]
    ProofMismatch(&'static str),
    #[error("grammar-extension certificate gate failed: {0}")]
    CertificateGate(&'static str),
    #[error("foreign certificate: expected cohort {expected_cohort}, got {certificate_cohort}")]
    ForeignCertificate {
        expected_cohort: u64,
        certificate_cohort: u64,
    },
    #[error("production {0:?} is already admitted")]
    DuplicateProduction(ExtensionKind),
    #[error("local refinement proof mismatch: {0}")]
    LocalProofMismatch(&'static str),
    #[error("local refinement gate failed: {0}")]
    LocalCertificateGate(&'static str),
    #[error("foreign local certificate: expected root {expected_root}, got {certificate_root}")]
    ForeignLocalCertificate {
        expected_root: u64,
        certificate_root: u64,
    },
    #[error("local refinement certificate is bound to a different registry")]
    RegistryMismatch,
    #[error("local refinement proof id {0} is already admitted")]
    DuplicateLocalRefinement(u64),
    #[error("grammar-extension invariant violation: {0}")]
    InvariantViolation(String),
}

#[derive(Debug, Clone)]
struct PartitionScore {
    repaired: usize,
    support_min: usize,
    bits: Vec<bool>,
    syntax: String,
}

#[derive(Debug, Clone)]
struct RootAnalysis {
    insufficiency: RootInsufficiency,
    schema_winners: BTreeMap<ExtensionKind, LocalSchemaWinner>,
}

pub fn synthesize_grammar_extension(
    problem: &GrammarExtensionProblem,
    config: GrammarExtensionConfig,
    budget: &mut GrammarExtensionBudget,
) -> Result<GrammarExtensionProof, GrammarExtensionError> {
    search_grammar_extension(problem, config, budget)
}

pub fn validate_grammar_extension(
    problem: &GrammarExtensionProblem,
    supplied: &GrammarExtensionProof,
    config: GrammarExtensionConfig,
    budget: &mut GrammarExtensionBudget,
) -> Result<ValidatedGrammarExtensionCertificate, GrammarExtensionError> {
    let recomputed = search_grammar_extension(problem, config, budget)?;
    compare_grammar_proof(supplied, &recomputed)?;
    enforce_grammar_gates(&recomputed, config)?;
    Ok(ValidatedGrammarExtensionCertificate {
        cohort_id: recomputed.cohort_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        kind: recomputed.winner.kind,
    })
}

pub fn synthesize_local_refinement(
    root: &GrammarRoot,
    registry: &GrammarRegistry,
    budget: &mut LocalRefinementBudget,
) -> Result<LocalRefinementProof, GrammarExtensionError> {
    search_local_refinement(root, registry, budget)
}

pub fn validate_local_refinement(
    root: &GrammarRoot,
    registry: &GrammarRegistry,
    supplied: &LocalRefinementProof,
    config: GrammarExtensionConfig,
    budget: &mut LocalRefinementBudget,
) -> Result<ValidatedLocalRefinementCertificate, GrammarExtensionError> {
    let recomputed = search_local_refinement(root, registry, budget)?;
    compare_local_proof(supplied, &recomputed)?;
    enforce_local_gates(&recomputed, config)?;
    Ok(ValidatedLocalRefinementCertificate {
        root_id: recomputed.root_id,
        proof_id: recomputed.proof_id,
        problem_digest: recomputed.problem_digest,
        registry_signature: recomputed.registry_signature,
        program: recomputed.program,
    })
}

pub fn exhaustive_base_ceiling(
    root: &GrammarRoot,
    budget: &mut GrammarExtensionBudget,
) -> Result<RootInsufficiency, GrammarExtensionError> {
    analyze_root(root, budget).map(|analysis| analysis.insufficiency)
}

fn search_grammar_extension(
    problem: &GrammarExtensionProblem,
    _config: GrammarExtensionConfig,
    budget: &mut GrammarExtensionBudget,
) -> Result<GrammarExtensionProof, GrammarExtensionError> {
    validate_problem(problem)?;
    let mut analyses = Vec::with_capacity(problem.roots.len());
    for root in &problem.roots {
        analyses.push(analyze_root(root, budget)?);
    }

    let mut schema_scores = Vec::new();
    for kind in ExtensionKind::all() {
        let mut local_winners = Vec::with_capacity(analyses.len());
        let mut exact_roots = 0usize;
        let mut total_repaired = 0usize;
        let mut total_defects = 0usize;
        let mut aggregate_support = 0usize;
        for analysis in &analyses {
            let winner = analysis
                .schema_winners
                .get(&kind)
                .ok_or(GrammarExtensionError::NoCandidatePartition)?
                .clone();
            let defects = analysis.insufficiency.detected_defects;
            if winner.repaired_defects == defects {
                exact_roots = exact_roots.saturating_add(1);
            }
            total_repaired = total_repaired.saturating_add(winner.repaired_defects);
            total_defects = total_defects.saturating_add(defects);
            aggregate_support = aggregate_support.saturating_add(winner.support_min);
            local_winners.push(winner);
        }
        schema_scores.push(SchemaScore {
            kind,
            exact_roots,
            total_repaired_defects: total_repaired,
            total_detected_defects: total_defects,
            aggregate_support_min: aggregate_support,
            local_winners,
        });
    }
    schema_scores.sort_by(|left, right| {
        right
            .exact_roots
            .cmp(&left.exact_roots)
            .then_with(|| {
                right
                    .total_repaired_defects
                    .cmp(&left.total_repaired_defects)
            })
            .then_with(|| {
                right
                    .aggregate_support_min
                    .cmp(&left.aggregate_support_min)
            })
            .then_with(|| left.kind.name().cmp(right.kind.name()))
    });
    let winner = schema_scores
        .first()
        .cloned()
        .ok_or(GrammarExtensionError::NoCandidatePartition)?;
    let runner_up_exact_roots = schema_scores.get(1).map(|score| score.exact_roots).unwrap_or(0);
    let exact_root_margin = winner
        .exact_roots
        .saturating_sub(runner_up_exact_roots);
    let problem_digest = digest_grammar_problem(problem);
    let root_insufficiency = analyses
        .into_iter()
        .map(|analysis| analysis.insufficiency)
        .collect::<Vec<_>>();
    let proof_id = digest_grammar_proof(
        problem.cohort_id,
        problem_digest,
        &root_insufficiency,
        &winner,
    );
    Ok(GrammarExtensionProof {
        proof_id,
        cohort_id: problem.cohort_id,
        problem_digest,
        root_count: problem.roots.len(),
        root_insufficiency,
        winner,
        runner_up_exact_roots,
        exact_root_margin,
        schema_count: ExtensionKind::all().len(),
    })
}

fn analyze_root(
    root: &GrammarRoot,
    budget: &mut GrammarExtensionBudget,
) -> Result<RootAnalysis, GrammarExtensionError> {
    if root.discovery.is_empty() {
        return Err(GrammarExtensionError::EmptyRoot(root.root_id));
    }
    let problem = root.as_refinement_problem();
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)
        .map_err(|error| GrammarExtensionError::RootAnalysis(error.to_string()))?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)
        .map_err(|error| GrammarExtensionError::RootAnalysis(error.to_string()))?;
    budget.vocabulary_history_scans = budget
        .vocabulary_history_scans
        .saturating_add(genesis.vocabulary_history_scans);
    budget.history_pair_evaluations = budget
        .history_pair_evaluations
        .saturating_add(genesis.history_pair_evaluations);

    let evidence_index = evidence_index(&root.discovery);
    let base_programs = enumerate_programs(&vocabulary);
    budget.base_candidate_programs = budget
        .base_candidate_programs
        .saturating_add(base_programs.len());
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
    rank_partition_scores(&mut base_scores);
    let base_best = base_scores
        .first()
        .ok_or(GrammarExtensionError::NoCandidatePartition)?;

    let mut schema_winners = BTreeMap::new();
    for kind in ExtensionKind::all() {
        let mut partitions = BTreeMap::<Vec<bool>, BoundExtensionProgram>::new();
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
                budget.extension_bound_candidates =
                    budget.extension_bound_candidates.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        budget.extension_program_history_evaluations = budget
                            .extension_program_history_evaluations
                            .saturating_add(1);
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
        budget.unique_extension_partitions = budget
            .unique_extension_partitions
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
        scored.sort_by(|left, right| compare_partition_score(&left.0, &right.0));
        let (score, program) = scored
            .first()
            .cloned()
            .ok_or(GrammarExtensionError::NoCandidatePartition)?;
        schema_winners.insert(
            kind,
            LocalSchemaWinner {
                root_id: root.root_id,
                program,
                repaired_defects: score.repaired,
                support_min: score.support_min,
            },
        );
    }

    Ok(RootAnalysis {
        insufficiency: RootInsufficiency {
            root_id: root.root_id,
            detected_defects: defects.len(),
            base_candidate_programs: budget_count_for_vocabulary(vocabulary.len()),
            base_unique_partitions: base_scores.len(),
            base_best_repaired_defects: base_best.repaired,
            base_best_program: base_best.syntax.clone(),
        },
        schema_winners,
    })
}

fn search_local_refinement(
    root: &GrammarRoot,
    registry: &GrammarRegistry,
    budget: &mut LocalRefinementBudget,
) -> Result<LocalRefinementProof, GrammarExtensionError> {
    if root.discovery.is_empty() {
        return Err(GrammarExtensionError::EmptyRoot(root.root_id));
    }
    let problem = root.as_refinement_problem();
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)
        .map_err(|error| GrammarExtensionError::RootAnalysis(error.to_string()))?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)
        .map_err(|error| GrammarExtensionError::RootAnalysis(error.to_string()))?;
    budget.vocabulary_history_scans = budget
        .vocabulary_history_scans
        .saturating_add(genesis.vocabulary_history_scans);
    budget.history_pair_evaluations = budget
        .history_pair_evaluations
        .saturating_add(genesis.history_pair_evaluations);

    let evidence_index = evidence_index(&root.discovery);
    let mut partitions = BTreeMap::<Vec<bool>, BoundExtensionProgram>::new();
    for kind in registry.admitted_kinds() {
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
    budget.unique_partitions = partitions.len();
    if partitions.is_empty() {
        return Err(GrammarExtensionError::NoCandidatePartition);
    }

    let mut ranked = partitions
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
    ranked.sort_by(|left, right| compare_partition_score(&left.0, &right.0));
    let (winner_score, winner_program) = ranked
        .first()
        .cloned()
        .ok_or(GrammarExtensionError::NoCandidatePartition)?;
    let runner_up = ranked.get(1).map(|entry| entry.0.repaired).unwrap_or(0);
    let unrepaired = defects.len().saturating_sub(winner_score.repaired);
    let problem_digest = digest_root(root);
    let registry_signature = registry.canonical_signature();
    let proof_id = digest_local_proof(
        root.root_id,
        problem_digest,
        &registry_signature,
        &winner_program,
        &winner_score.bits,
    );
    Ok(LocalRefinementProof {
        proof_id,
        root_id: root.root_id,
        problem_digest,
        registry_signature,
        detected_defects: defects.len(),
        candidate_programs: budget.candidate_programs,
        unique_partitions: budget.unique_partitions,
        program: winner_program,
        canonical_partition: winner_score.bits,
        repaired_defects: winner_score.repaired,
        unrepaired_defects: unrepaired,
        support_min: winner_score.support_min,
        runner_up_repaired_defects: runner_up,
        winner_margin: winner_score.repaired.saturating_sub(runner_up),
    })
}

fn enforce_grammar_gates(
    proof: &GrammarExtensionProof,
    config: GrammarExtensionConfig,
) -> Result<(), GrammarExtensionError> {
    if proof.root_count == 0 {
        return Err(GrammarExtensionError::CertificateGate("empty_cohort"));
    }
    if proof.root_insufficiency.len() != proof.root_count {
        return Err(GrammarExtensionError::CertificateGate(
            "root_cardinality_mismatch",
        ));
    }
    if proof
        .root_insufficiency
        .iter()
        .any(|root| root.base_best_repaired_defects >= root.detected_defects)
    {
        return Err(GrammarExtensionError::CertificateGate(
            "base_grammar_not_exhausted_or_sufficient",
        ));
    }
    if proof.winner.exact_roots != proof.root_count {
        return Err(GrammarExtensionError::CertificateGate(
            "winner_not_exact_everywhere",
        ));
    }
    if proof.exact_root_margin < config.min_exact_root_margin {
        return Err(GrammarExtensionError::CertificateGate(
            "insufficient_schema_margin",
        ));
    }
    if proof
        .winner
        .local_winners
        .iter()
        .any(|winner| winner.support_min < config.min_partition_support)
    {
        return Err(GrammarExtensionError::CertificateGate(
            "insufficient_partition_support",
        ));
    }
    Ok(())
}

fn enforce_local_gates(
    proof: &LocalRefinementProof,
    config: GrammarExtensionConfig,
) -> Result<(), GrammarExtensionError> {
    if proof.detected_defects == 0 {
        return Err(GrammarExtensionError::LocalCertificateGate("no_defects"));
    }
    if proof.repaired_defects != proof.detected_defects || proof.unrepaired_defects != 0 {
        return Err(GrammarExtensionError::LocalCertificateGate(
            "incomplete_defect_repair",
        ));
    }
    if proof.support_min < config.min_partition_support {
        return Err(GrammarExtensionError::LocalCertificateGate(
            "insufficient_partition_support",
        ));
    }
    Ok(())
}

fn compare_grammar_proof(
    supplied: &GrammarExtensionProof,
    recomputed: &GrammarExtensionProof,
) -> Result<(), GrammarExtensionError> {
    macro_rules! check {
        ($field:ident) => {
            if supplied.$field != recomputed.$field {
                return Err(GrammarExtensionError::ProofMismatch(stringify!($field)));
            }
        };
    }
    check!(proof_id);
    check!(cohort_id);
    check!(problem_digest);
    check!(root_count);
    check!(root_insufficiency);
    check!(winner);
    check!(runner_up_exact_roots);
    check!(exact_root_margin);
    check!(schema_count);
    Ok(())
}

fn compare_local_proof(
    supplied: &LocalRefinementProof,
    recomputed: &LocalRefinementProof,
) -> Result<(), GrammarExtensionError> {
    macro_rules! check {
        ($field:ident) => {
            if supplied.$field != recomputed.$field {
                return Err(GrammarExtensionError::LocalProofMismatch(stringify!($field)));
            }
        };
    }
    check!(proof_id);
    check!(root_id);
    check!(problem_digest);
    check!(registry_signature);
    check!(detected_defects);
    check!(candidate_programs);
    check!(unique_partitions);
    check!(program);
    check!(canonical_partition);
    check!(repaired_defects);
    check!(unrepaired_defects);
    check!(support_min);
    check!(runner_up_repaired_defects);
    check!(winner_margin);
    Ok(())
}

fn validate_problem(problem: &GrammarExtensionProblem) -> Result<(), GrammarExtensionError> {
    if problem.roots.is_empty() {
        return Err(GrammarExtensionError::EmptyCohort);
    }
    let mut root_ids = BTreeSet::new();
    for root in &problem.roots {
        if root.discovery.is_empty() {
            return Err(GrammarExtensionError::EmptyRoot(root.root_id));
        }
        if !root_ids.insert(root.root_id) {
            return Err(GrammarExtensionError::DuplicateRoot(root.root_id));
        }
    }
    Ok(())
}

fn evidence_index(discovery: &[WitnessedHistory]) -> BTreeMap<u64, usize> {
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

fn rank_partition_scores(scores: &mut [PartitionScore]) {
    scores.sort_by(compare_partition_score);
}

fn compare_partition_score(left: &PartitionScore, right: &PartitionScore) -> std::cmp::Ordering {
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

fn budget_count_for_vocabulary(vocabulary_size: usize) -> usize {
    let metrics = vocabulary_size.saturating_mul(3);
    metrics
        .saturating_mul(metrics.saturating_sub(1))
        .saturating_add(metrics.saturating_mul(metrics.saturating_sub(1)) / 2)
}

fn digest_grammar_problem(problem: &GrammarExtensionProblem) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&problem.cohort_id.to_le_bytes());
    for root in &problem.roots {
        bytes.extend_from_slice(&digest_root(root).to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn digest_root(root: &GrammarRoot) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root.root_id.to_le_bytes());
    for episode in &root.discovery {
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

fn digest_grammar_proof(
    cohort_id: u64,
    problem_digest: u64,
    roots: &[RootInsufficiency],
    winner: &SchemaScore,
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&cohort_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(winner.kind.name().as_bytes());
    for root in roots {
        bytes.extend_from_slice(&root.root_id.to_le_bytes());
        bytes.extend_from_slice(&root.detected_defects.to_le_bytes());
        bytes.extend_from_slice(&root.base_best_repaired_defects.to_le_bytes());
        bytes.extend_from_slice(root.base_best_program.as_bytes());
        bytes.push(0x55);
    }
    for local in &winner.local_winners {
        bytes.extend_from_slice(&local.root_id.to_le_bytes());
        bytes.extend_from_slice(local.program.canonical_string().as_bytes());
        bytes.extend_from_slice(&local.repaired_defects.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn digest_local_proof(
    root_id: u64,
    problem_digest: u64,
    registry_signature: &str,
    program: &BoundExtensionProgram,
    partition: &[bool],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&root_id.to_le_bytes());
    bytes.extend_from_slice(&problem_digest.to_le_bytes());
    bytes.extend_from_slice(registry_signature.as_bytes());
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

    fn root(root_id: u64, kind: ExtensionKind) -> GrammarRoot {
        let vocabulary = ["a", "b", "c", "d", "e"]
            .into_iter()
            .map(atom)
            .collect::<Vec<_>>();
        let discovery = permutations(&vocabulary)
            .into_iter()
            .enumerate()
            .filter(|(index, _)| index % 5 != 0)
            .map(|(index, events)| {
                let program = BoundExtensionProgram {
                    kind,
                    left: vocabulary[0].clone(),
                    right: vocabulary[1].clone(),
                };
                WitnessedHistory {
                    evidence_id: root_id * 1000 + index as u64,
                    history: RawHistory {
                        history_id: root_id * 10_000 + index as u64,
                        events: events.clone(),
                    },
                    intervention: atom("probe"),
                    outcome: atom(if program.execute(&RawHistory {
                        history_id: 0,
                        events,
                    }) {
                        "positive"
                    } else {
                        "negative"
                    }),
                }
            })
            .collect();
        GrammarRoot { root_id, discovery }
    }

    #[test]
    fn discovers_and_admits_adjacent_before() {
        let problem = GrammarExtensionProblem {
            cohort_id: 7,
            roots: (1..=2)
                .map(|root_id| root(root_id, ExtensionKind::AdjacentBefore))
                .collect(),
        };
        let mut proposal_budget = GrammarExtensionBudget::default();
        let proof = synthesize_grammar_extension(
            &problem,
            GrammarExtensionConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        assert_eq!(proof.winner.kind, ExtensionKind::AdjacentBefore);
        assert_eq!(proof.winner.exact_roots, 2);
        assert!(proof
            .root_insufficiency
            .iter()
            .all(|root| root.base_best_repaired_defects < root.detected_defects));

        let mut validation_budget = GrammarExtensionBudget::default();
        let certificate = validate_grammar_extension(
            &problem,
            &proof,
            GrammarExtensionConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        assert_eq!(proposal_budget, validation_budget);

        let mut registry = GrammarRegistry::new(7);
        registry.admit(&certificate).unwrap();
        assert!(registry.supports(ExtensionKind::AdjacentBefore));
        assert!(registry.admit(&certificate).is_err());
    }

    #[test]
    fn local_refinement_requires_the_admitted_schema() {
        let development = GrammarExtensionProblem {
            cohort_id: 9,
            roots: vec![root(1, ExtensionKind::AdjacentBefore)],
        };
        let mut budget = GrammarExtensionBudget::default();
        let proof = synthesize_grammar_extension(
            &development,
            GrammarExtensionConfig::default(),
            &mut budget,
        )
        .unwrap();
        let mut validation_budget = GrammarExtensionBudget::default();
        let certificate = validate_grammar_extension(
            &development,
            &proof,
            GrammarExtensionConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        let mut registry = GrammarRegistry::new(9);
        registry.admit(&certificate).unwrap();

        let target = root(99, ExtensionKind::AdjacentBefore);
        let mut local_budget = LocalRefinementBudget::default();
        let local = synthesize_local_refinement(&target, &registry, &mut local_budget).unwrap();
        assert_eq!(local.repaired_defects, local.detected_defects);
        let mut local_validation_budget = LocalRefinementBudget::default();
        let local_certificate = validate_local_refinement(
            &target,
            &registry,
            &local,
            GrammarExtensionConfig::default(),
            &mut local_validation_budget,
        )
        .unwrap();
        assert_eq!(local_budget, local_validation_budget);

        let mut language = GrammarExtendedStateLanguage::new(target.root_id, &registry);
        language.admit_local(&local_certificate).unwrap();
        assert_eq!(language.refinement_count(), 1);
    }

    #[test]
    fn tampered_and_foreign_proofs_fail_closed() {
        let problem = GrammarExtensionProblem {
            cohort_id: 11,
            roots: vec![root(1, ExtensionKind::AdjacentBefore)],
        };
        let mut budget = GrammarExtensionBudget::default();
        let proof = synthesize_grammar_extension(
            &problem,
            GrammarExtensionConfig::default(),
            &mut budget,
        )
        .unwrap();
        let mut tampered = proof.clone();
        tampered.winner.total_repaired_defects =
            tampered.winner.total_repaired_defects.saturating_sub(1);
        let mut validation_budget = GrammarExtensionBudget::default();
        assert!(validate_grammar_extension(
            &problem,
            &tampered,
            GrammarExtensionConfig::default(),
            &mut validation_budget,
        )
        .is_err());

        let mut clean_validation_budget = GrammarExtensionBudget::default();
        let certificate = validate_grammar_extension(
            &problem,
            &proof,
            GrammarExtensionConfig::default(),
            &mut clean_validation_budget,
        )
        .unwrap();
        let mut foreign_registry = GrammarRegistry::new(12);
        assert!(foreign_registry.admit(&certificate).is_err());
        assert_eq!(foreign_registry.admitted_count(), 0);
    }
}
