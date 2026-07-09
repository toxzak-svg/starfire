//! Routable Computational Tension (RCT) / CHARGE.
//!
//! CHARGE makes unresolved computational tension a first-class object. Cognitive
//! systems can emit a charge, attempt to resolve it, and account for what was
//! discharged, transformed into new charges, or explicitly decayed.
//!
//! The primitive, accounting layer, subsystem-backed emitters, and empirical
//! ontology-induction machinery live here. Emitters translate unresolved state
//! into charge but never choose a resolver; routing and induced distinctions
//! remain empirical and can be falsified independently.

pub mod emitters;
pub mod induction;
pub mod ledger;
pub mod ontology;
pub mod resolver;
pub mod types;

pub use emitters::{
    knowledge_gap_charge, prediction_contradiction_charge, QuanotTrajectoryEmitter,
};
pub use induction::{
    ConceptRoute, EmpiricalInductionConfig, EmpiricalOntologyInducer, LearnedOntology,
    OntologyInductionError, OntologyInductionSummary, OntologyObservation, OntologyPolicyMetrics,
    OntologyRouteDecision, ResolverOutcome,
};
pub use ledger::{ChargeLedger, ChargeLedgerError, LedgerSummary, ResolutionReceipt};
pub use ontology::{
    ConceptEvidence, ConceptId, ConceptPredicate, ConceptUtility, Direction, InducedConcept,
    OntologyInducer, OntologyMutation, PromotionCriteria,
};
pub use resolver::{
    ChargeRoutingSignature, ChargeScopeClass, Resolver, ResolverStats,
};
pub use types::{Charge, ChargeKind, ChargeScope, ChargeSignature, ChargeTrace, Resolution};
