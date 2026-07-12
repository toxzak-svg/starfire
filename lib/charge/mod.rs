//! Routable Computational Tension (RCT) / CHARGE.
//!
//! CHARGE makes unresolved computational tension a first-class object. Cognitive
//! systems can emit a charge, attempt to resolve it, and account for what was
//! discharged, transformed into new charges, or explicitly decayed.
//!
//! The primitive, accounting layer, independent discharge judging, subsystem-
//! backed emitters, CHARGE-native feature adapters, empirical ontology induction,
//! and shadow-only promotion evaluation live here. Emitters translate unresolved
//! state into charge but never choose a resolver; routing and induced distinctions
//! remain empirical and can be falsified independently.

pub mod contrast;
pub mod diagnostics;
pub mod emitters;
pub mod features;
pub mod induction;
pub mod judge;
pub mod ledger;
pub mod ontology;
pub mod resolver;
pub mod shadow;
pub mod types;
pub mod verifier;

pub use contrast::{
    disagreement_pair_schedule, fit_contrast_from_pairs, fit_disagreement_contrast,
    valid_contrast_pairs, ContrastProbeConfig, ContrastProbeError, ContrastProbeFit,
    LearnedContrastProbe, ProbeSide, TensionContrast,
};
pub use diagnostics::{
    assess_resolver_identifiability, IdentifiabilityAssessment, IdentifiabilityCriteria,
    ResolverLeaderDistribution, ResolverMarginSummary,
};
pub use emitters::{
    knowledge_gap_charge, prediction_contradiction_charge, QuanotTrajectoryEmitter,
};
pub use features::{
    fixed_residual_feature_charge, fixed_residual_projection, ontology_feature_charge,
    residual_geometry, FixedResidualProjection, FixedResidualProjectionConfig, ResidualGeometry,
};
pub use induction::{
    ConceptRoute, EmpiricalInductionConfig, EmpiricalOntologyInducer, LearnedOntology,
    OntologyInductionError, OntologyInductionSummary, OntologyObservation, OntologyPolicyMetrics,
    OntologyRouteDecision, ResolverOutcome,
};
pub use judge::{
    DischargeJudge, ImprovementDirection, JudgedDischarge, OutcomeWitness, RelativeImprovementJudge,
};
pub use ledger::{ChargeLedger, ChargeLedgerError, LedgerSummary, ResolutionReceipt};
pub use ontology::{
    ConceptEvidence, ConceptId, ConceptPredicate, ConceptUtility, Direction, InducedConcept,
    OntologyInducer, OntologyMutation, PromotionCriteria,
};
pub use resolver::{ChargeRoutingSignature, ChargeScopeClass, Resolver, ResolverStats};
pub use shadow::{
    ShadowBudget, ShadowControlComparison, ShadowControlScore, ShadowPromotionAssessment,
    ShadowPromotionConfig, ShadowPromotionCriteria, ShadowPromotionError, ShadowPromotionMonitor,
    ShadowPromotionStatus, ShadowTransferSummary, ShadowUpdate, ShadowWindowMetrics,
};
pub use types::{Charge, ChargeKind, ChargeScope, ChargeSignature, ChargeTrace, Resolution};
pub use verifier::{
    score_resolution, surface_resolution_score, VerifierProfile, VerifierTaskClass,
};

#[cfg(test)]
mod companion_integration_tests {
    use super::ChargeKind;
    use crate::companion_state::{
        ClaimInput, ClaimSource, ClaimStatus, CompanionState, Retention, Sensitivity,
    };
    use crate::persistence::{CompanionPersistence, Store};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn claim(key: &str, value: &str, observed_at_ms: u64, retention: Retention) -> ClaimInput {
        ClaimInput {
            key: key.to_owned(),
            value: value.to_owned(),
            source: ClaimSource::UserStatement,
            confidence_bps: 10_000,
            sensitivity: Sensitivity::Personal,
            retention,
            observed_at_ms,
        }
    }

    #[test]
    fn companion_contradiction_emits_charge_without_overwriting_user_claim() {
        let mut state = CompanionState::new();
        let original = state
            .record_claim(
                0,
                claim("response style", "direct", 10, Retention::Durable),
            )
            .unwrap();
        let conflict = state
            .record_claim(
                original.version,
                ClaimInput {
                    key: "response style".to_owned(),
                    value: "verbose".to_owned(),
                    source: ClaimSource::Inference {
                        method: "shadow-style-classifier".to_owned(),
                    },
                    confidence_bps: 8_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 11,
                },
            )
            .unwrap();

        assert_eq!(conflict.emitted_charges.len(), 1);
        assert_eq!(conflict.emitted_charges[0].kind, ChargeKind::Contradiction);
        assert_eq!(
            state.active_claim("response style", 11, true).unwrap().id,
            original.claim_id.unwrap()
        );
        assert!(matches!(
            &state.claim(conflict.claim_id.unwrap()).unwrap().status,
            ClaimStatus::Contested { .. }
        ));
    }

    #[test]
    fn expired_claim_is_retired_before_fresh_claim_becomes_active() {
        let mut state = CompanionState::new();
        let original = state
            .record_claim(
                0,
                claim(
                    "temporary preference",
                    "old",
                    10,
                    Retention::Until { expires_at_ms: 20 },
                ),
            )
            .unwrap();
        let replacement = state
            .record_claim(
                original.version,
                claim("temporary preference", "new", 21, Retention::Durable),
            )
            .unwrap();

        assert!(replacement.emitted_charges.is_empty());
        assert_eq!(
            state
                .active_claim("temporary preference", 21, true)
                .unwrap()
                .id,
            replacement.claim_id.unwrap()
        );
        assert!(matches!(
            &state.claim(original.claim_id.unwrap()).unwrap().status,
            ClaimStatus::Invalidated { reason } if reason == "retention expired"
        ));
    }

    #[test]
    fn companion_journal_commits_and_compacts_through_starfire_store() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "starfire-charge-companion-journal-{}-{nonce}.sqlite",
            std::process::id()
        ));
        let store = Arc::new(Store::open(&path).unwrap());
        let persistence = CompanionPersistence::new(store);
        let mut state = CompanionState::new();
        let recorded = state
            .record_claim(
                0,
                claim("private note", "temporary secret", 10, Retention::Durable),
            )
            .unwrap();
        persistence.commit(0, &recorded, &state, 10).unwrap();
        assert_eq!(persistence.load_state().unwrap(), state);

        let deleted = state
            .delete_claim(state.version, recorded.claim_id.unwrap(), 20)
            .unwrap();
        persistence.commit(1, &deleted, &state, 20).unwrap();
        let stats = persistence.stats().unwrap();
        assert_eq!(stats.checkpoint_version, 2);
        assert_eq!(stats.current_version, 2);
        assert_eq!(stats.tail_events, 0);
        assert_eq!(persistence.load_state().unwrap(), state);

        drop(persistence);
        let _ = std::fs::remove_file(path);
    }
}
