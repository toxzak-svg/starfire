//! S6-C consented, independently witnessed real-interaction canary evidence.
//!
//! This module seals existing S5-B trials before outcomes, accepts only typed and
//! digest-addressed attestations, and atomically imports valid evidence through
//! the existing S5-B ledger. It stores no raw conversation content and grants no
//! response, routing, memory, belief, persistence, tool, or action authority.

use crate::companion_interaction_outcomes::{
    InteractionOutcomeError, InteractionOutcomeLedger, InteractionTrial, InteractionTrialId,
    ObservedOutcomeEvidence, ObservedSignal, PairedEvaluationEvidence, PairwisePreference,
};
use crate::companion_interaction_policy::PolicyVariant;
use crate::companion_policy_evaluation::{
    evaluate_shadow_policies, ArmComputeObservation, EvaluationSplit, EvaluationSplitPolicy,
    PolicyEvaluationConfig, PolicyEvaluationError, PolicyEvaluationReport,
};
use crate::companion_prediction_ledger::{PredictionLedger, WitnessSource};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryEvidenceOrigin {
    RealInteraction,
    SyntheticFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryStudyConfig {
    pub study_digest: u64,
    pub protocol_digest: u64,
    pub split_policy: EvaluationSplitPolicy,
    pub allow_synthetic_fixture: bool,
}

impl CanaryStudyConfig {
    pub fn validate(self) -> Result<Self, CanaryEvidenceError> {
        if self.study_digest == 0 || self.protocol_digest == 0 {
            return Err(CanaryEvidenceError::InvalidStudyConfig);
        }
        self.split_policy.validate()?;
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryArmSeal {
    pub variant: PolicyVariant,
    pub policy_digest_fnv1a64: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryTrialSeal {
    pub study_digest: u64,
    pub protocol_digest: u64,
    pub trial_id: InteractionTrialId,
    pub subject_scope_digest: u64,
    pub context_digest: u64,
    pub issued_at_ms: u64,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub split: EvaluationSplit,
    pub delivered_variant: Option<PolicyVariant>,
    pub arms: Vec<CanaryArmSeal>,
    pub consent_digest: u64,
    pub operator_digest: u64,
    pub seal_digest_fnv1a64: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectCanaryAttestation {
    pub origin: CanaryEvidenceOrigin,
    pub trial_id: InteractionTrialId,
    pub signal: ObservedSignal,
    pub source: WitnessSource,
    pub witness_digest: u64,
    pub producer_digest: u64,
    pub consent_digest: u64,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairwiseCanaryAttestation {
    pub origin: CanaryEvidenceOrigin,
    pub trial_id: InteractionTrialId,
    pub left_variant: PolicyVariant,
    pub right_variant: PolicyVariant,
    pub preference: PairwisePreference,
    pub left_render_digest: u64,
    pub right_render_digest: u64,
    pub blinded_order_digest: u64,
    pub evaluator_digest: u64,
    pub producer_digest: u64,
    pub consent_digest: u64,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryEvidenceEvent {
    TrialSealed {
        seal: CanaryTrialSeal,
    },
    DirectImported {
        attestation: DirectCanaryAttestation,
        exported_evidence: ObservedOutcomeEvidence,
    },
    PairwiseImported {
        attestation: PairwiseCanaryAttestation,
        exported_evidence: PairedEvaluationEvidence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryEvidenceTransition {
    pub version: u64,
    pub event: CanaryEvidenceEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryEvidenceSummary {
    pub sealed_trials: u64,
    pub direct_imports: u64,
    pub pairwise_imports: u64,
    pub real_evidence: u64,
    pub synthetic_evidence: u64,
    pub independent_witnesses: u64,
    pub raw_content_retained: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryEvaluationReport {
    pub evidence: CanaryEvidenceSummary,
    pub s5c_report: PolicyEvaluationReport,
    pub all_trials_sealed: bool,
    pub opaque_subject_holdout_present: bool,
    pub temporal_holdout_present: bool,
    pub all_evidence_real: bool,
    pub promotion_eligible: bool,
    pub live_response_influence: bool,
    pub routing_authority: bool,
    pub belief_promotion_authority: bool,
    pub persistence_authority: bool,
    pub tool_authority: bool,
    pub action_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanaryEvidenceLedger {
    pub version: u64,
    config: CanaryStudyConfig,
    seals: BTreeMap<InteractionTrialId, CanaryTrialSeal>,
    direct_trials: BTreeSet<InteractionTrialId>,
    pairwise_pairs: BTreeSet<(InteractionTrialId, PolicyVariant, PolicyVariant)>,
    events: Vec<CanaryEvidenceEvent>,
}

impl CanaryEvidenceLedger {
    pub fn new(config: CanaryStudyConfig) -> Result<Self, CanaryEvidenceError> {
        Ok(Self {
            version: 0,
            config: config.validate()?,
            seals: BTreeMap::new(),
            direct_trials: BTreeSet::new(),
            pairwise_pairs: BTreeSet::new(),
            events: Vec::new(),
        })
    }

    #[must_use]
    pub fn config(&self) -> CanaryStudyConfig {
        self.config
    }

    #[must_use]
    pub fn seals(&self) -> &BTreeMap<InteractionTrialId, CanaryTrialSeal> {
        &self.seals
    }

    #[must_use]
    pub fn events(&self) -> &[CanaryEvidenceEvent] {
        &self.events
    }

    pub fn seal_trial(
        &mut self,
        expected_version: u64,
        outcomes: &InteractionOutcomeLedger,
        trial_id: InteractionTrialId,
        consent_digest: u64,
        operator_digest: u64,
    ) -> Result<CanaryEvidenceTransition, CanaryEvidenceError> {
        self.require_version(expected_version)?;
        let trial = outcomes
            .trials()
            .get(&trial_id)
            .ok_or(CanaryEvidenceError::UnknownTrial(trial_id))?;
        let seal = build_seal(self.config, trial, consent_digest, operator_digest)?;
        self.apply_event(CanaryEvidenceEvent::TrialSealed { seal })
    }

    pub fn import_direct(
        &mut self,
        expected_version: u64,
        outcomes: &mut InteractionOutcomeLedger,
        predictions: &mut PredictionLedger,
        attestation: DirectCanaryAttestation,
    ) -> Result<CanaryEvidenceTransition, CanaryEvidenceError> {
        self.require_version(expected_version)?;
        let seal = self
            .seals
            .get(&attestation.trial_id)
            .ok_or(CanaryEvidenceError::UnsealedTrial(attestation.trial_id))?;
        validate_seal_against_outcomes(seal, outcomes)?;
        validate_direct_attestation(self.config, seal, &attestation)?;

        let exported_evidence = ObservedOutcomeEvidence {
            signal: attestation.signal,
            source: attestation.source,
            observed_at_ms: attestation.observed_at_ms,
            evidence_digest: attestation.evidence_digest,
        };
        let event = CanaryEvidenceEvent::DirectImported {
            attestation,
            exported_evidence: exported_evidence.clone(),
        };

        let mut working_canary = self.clone();
        let transition = working_canary.apply_event(event)?;
        let mut working_outcomes = outcomes.clone();
        let mut working_predictions = predictions.clone();
        working_outcomes.record_observed_signal(
            working_outcomes.version,
            &mut working_predictions,
            exported_evidence_trial_id(&transition.event),
            exported_evidence,
        )?;

        *self = working_canary;
        *outcomes = working_outcomes;
        *predictions = working_predictions;
        Ok(transition)
    }

    pub fn import_pairwise(
        &mut self,
        expected_version: u64,
        outcomes: &mut InteractionOutcomeLedger,
        predictions: &mut PredictionLedger,
        attestation: PairwiseCanaryAttestation,
    ) -> Result<CanaryEvidenceTransition, CanaryEvidenceError> {
        self.require_version(expected_version)?;
        let seal = self
            .seals
            .get(&attestation.trial_id)
            .ok_or(CanaryEvidenceError::UnsealedTrial(attestation.trial_id))?;
        validate_seal_against_outcomes(seal, outcomes)?;
        validate_pairwise_attestation(self.config, seal, &attestation)?;

        let exported_evidence = PairedEvaluationEvidence {
            left_variant: attestation.left_variant,
            right_variant: attestation.right_variant,
            preference: attestation.preference,
            left_render_digest: attestation.left_render_digest,
            right_render_digest: attestation.right_render_digest,
            evaluator_digest: attestation.evaluator_digest,
            observed_at_ms: attestation.observed_at_ms,
            evidence_digest: attestation.evidence_digest,
        };
        let event = CanaryEvidenceEvent::PairwiseImported {
            attestation,
            exported_evidence: exported_evidence.clone(),
        };

        let mut working_canary = self.clone();
        let transition = working_canary.apply_event(event)?;
        let mut working_outcomes = outcomes.clone();
        let mut working_predictions = predictions.clone();
        working_outcomes.record_paired_evaluation(
            working_outcomes.version,
            &mut working_predictions,
            exported_evidence_trial_id(&transition.event),
            exported_evidence,
        )?;

        *self = working_canary;
        *outcomes = working_outcomes;
        *predictions = working_predictions;
        Ok(transition)
    }

    pub fn replay(
        config: CanaryStudyConfig,
        events: &[CanaryEvidenceEvent],
    ) -> Result<Self, CanaryEvidenceError> {
        let mut ledger = Self::new(config)?;
        for event in events {
            ledger.apply_event(event.clone())?;
        }
        Ok(ledger)
    }

    #[must_use]
    pub fn summary(&self) -> CanaryEvidenceSummary {
        let mut direct_imports = 0_u64;
        let mut pairwise_imports = 0_u64;
        let mut real_evidence = 0_u64;
        let mut synthetic_evidence = 0_u64;
        let mut witnesses = BTreeSet::new();

        for event in &self.events {
            match event {
                CanaryEvidenceEvent::TrialSealed { .. } => {}
                CanaryEvidenceEvent::DirectImported { attestation, .. } => {
                    direct_imports += 1;
                    witnesses.insert(attestation.witness_digest);
                    count_origin(
                        attestation.origin,
                        &mut real_evidence,
                        &mut synthetic_evidence,
                    );
                }
                CanaryEvidenceEvent::PairwiseImported { attestation, .. } => {
                    pairwise_imports += 1;
                    witnesses.insert(attestation.evaluator_digest);
                    count_origin(
                        attestation.origin,
                        &mut real_evidence,
                        &mut synthetic_evidence,
                    );
                }
            }
        }

        CanaryEvidenceSummary {
            sealed_trials: self.seals.len() as u64,
            direct_imports,
            pairwise_imports,
            real_evidence,
            synthetic_evidence,
            independent_witnesses: witnesses.len() as u64,
            raw_content_retained: false,
        }
    }

    pub fn evaluate(
        &self,
        outcomes: &InteractionOutcomeLedger,
        compute: &[ArmComputeObservation],
        config: PolicyEvaluationConfig,
    ) -> Result<CanaryEvaluationReport, CanaryEvidenceError> {
        let all_trials_sealed = outcomes.trials().len() == self.seals.len()
            && outcomes
                .trials()
                .keys()
                .all(|trial_id| self.seals.contains_key(trial_id));
        if !all_trials_sealed {
            return Err(CanaryEvidenceError::UnsealedEvaluationTrial);
        }
        for seal in self.seals.values() {
            validate_seal_against_outcomes(seal, outcomes)?;
        }

        let s5c_report = evaluate_shadow_policies(outcomes, compute, config)?;
        let opaque_subject_holdout_present = split_has_trials(
            &s5c_report,
            EvaluationSplit::OpaqueSubjectHoldout,
        );
        let temporal_holdout_present =
            split_has_trials(&s5c_report, EvaluationSplit::TemporalHoldout);
        let evidence = self.summary();
        let all_evidence_real = evidence.real_evidence > 0 && evidence.synthetic_evidence == 0;
        let promotion_eligible = s5c_report.promotion_eligible
            && all_evidence_real
            && all_trials_sealed
            && opaque_subject_holdout_present
            && temporal_holdout_present;

        Ok(CanaryEvaluationReport {
            evidence,
            s5c_report,
            all_trials_sealed,
            opaque_subject_holdout_present,
            temporal_holdout_present,
            all_evidence_real,
            promotion_eligible,
            live_response_influence: false,
            routing_authority: false,
            belief_promotion_authority: false,
            persistence_authority: false,
            tool_authority: false,
            action_authority: false,
        })
    }

    fn require_version(&self, expected_version: u64) -> Result<(), CanaryEvidenceError> {
        if self.version != expected_version {
            return Err(CanaryEvidenceError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }

    fn apply_event(
        &mut self,
        event: CanaryEvidenceEvent,
    ) -> Result<CanaryEvidenceTransition, CanaryEvidenceError> {
        match &event {
            CanaryEvidenceEvent::TrialSealed { seal } => {
                validate_seal(self.config, seal)?;
                if self.seals.contains_key(&seal.trial_id) {
                    return Err(CanaryEvidenceError::DuplicateSeal(seal.trial_id));
                }
                self.seals.insert(seal.trial_id, seal.clone());
            }
            CanaryEvidenceEvent::DirectImported {
                attestation,
                exported_evidence,
            } => {
                let seal = self
                    .seals
                    .get(&attestation.trial_id)
                    .ok_or(CanaryEvidenceError::UnsealedTrial(attestation.trial_id))?;
                validate_direct_attestation(self.config, seal, attestation)?;
                if exported_evidence.signal != attestation.signal
                    || exported_evidence.source != attestation.source
                    || exported_evidence.observed_at_ms != attestation.observed_at_ms
                    || exported_evidence.evidence_digest != attestation.evidence_digest
                {
                    return Err(CanaryEvidenceError::ExportMismatch);
                }
                if !self.direct_trials.insert(attestation.trial_id) {
                    return Err(CanaryEvidenceError::DuplicateDirectEvidence(
                        attestation.trial_id,
                    ));
                }
            }
            CanaryEvidenceEvent::PairwiseImported {
                attestation,
                exported_evidence,
            } => {
                let seal = self
                    .seals
                    .get(&attestation.trial_id)
                    .ok_or(CanaryEvidenceError::UnsealedTrial(attestation.trial_id))?;
                validate_pairwise_attestation(self.config, seal, attestation)?;
                if exported_evidence.left_variant != attestation.left_variant
                    || exported_evidence.right_variant != attestation.right_variant
                    || exported_evidence.preference != attestation.preference
                    || exported_evidence.left_render_digest != attestation.left_render_digest
                    || exported_evidence.right_render_digest != attestation.right_render_digest
                    || exported_evidence.evaluator_digest != attestation.evaluator_digest
                    || exported_evidence.observed_at_ms != attestation.observed_at_ms
                    || exported_evidence.evidence_digest != attestation.evidence_digest
                {
                    return Err(CanaryEvidenceError::ExportMismatch);
                }
                let pair = canonical_pair(
                    attestation.trial_id,
                    attestation.left_variant,
                    attestation.right_variant,
                );
                if !self.pairwise_pairs.insert(pair) {
                    return Err(CanaryEvidenceError::DuplicatePairwiseEvidence {
                        trial_id: attestation.trial_id,
                        left: pair.1,
                        right: pair.2,
                    });
                }
            }
        }

        self.version = self
            .version
            .checked_add(1)
            .ok_or(CanaryEvidenceError::VersionOverflow)?;
        self.events.push(event.clone());
        Ok(CanaryEvidenceTransition {
            version: self.version,
            event,
        })
    }
}

fn build_seal(
    config: CanaryStudyConfig,
    trial: &InteractionTrial,
    consent_digest: u64,
    operator_digest: u64,
) -> Result<CanaryTrialSeal, CanaryEvidenceError> {
    if consent_digest == 0 || operator_digest == 0 {
        return Err(CanaryEvidenceError::EmptySealEvidence);
    }
    if trial.subject_scope_digest == 0
        || trial.context_digest == 0
        || trial.issued_at_ms >= trial.not_before_ms
        || trial.not_before_ms >= trial.expires_at_ms
        || trial.arms.is_empty()
    {
        return Err(CanaryEvidenceError::MalformedTrial(trial.id));
    }

    let mut arms: Vec<_> = trial
        .arms
        .iter()
        .map(|arm| CanaryArmSeal {
            variant: arm.variant,
            policy_digest_fnv1a64: arm.policy_digest_fnv1a64,
        })
        .collect();
    arms.sort_by_key(|arm| arm.variant);
    if arms.iter().any(|arm| arm.policy_digest_fnv1a64 == 0)
        || arms.windows(2).any(|pair| pair[0].variant == pair[1].variant)
    {
        return Err(CanaryEvidenceError::MalformedTrial(trial.id));
    }

    let split = config.split_policy.classify(trial)?;
    let mut seal = CanaryTrialSeal {
        study_digest: config.study_digest,
        protocol_digest: config.protocol_digest,
        trial_id: trial.id,
        subject_scope_digest: trial.subject_scope_digest,
        context_digest: trial.context_digest,
        issued_at_ms: trial.issued_at_ms,
        not_before_ms: trial.not_before_ms,
        expires_at_ms: trial.expires_at_ms,
        split,
        delivered_variant: trial.delivered_variant,
        arms,
        consent_digest,
        operator_digest,
        seal_digest_fnv1a64: 0,
    };
    seal.seal_digest_fnv1a64 = canonical_seal_digest(&seal);
    Ok(seal)
}

fn validate_seal(
    config: CanaryStudyConfig,
    seal: &CanaryTrialSeal,
) -> Result<(), CanaryEvidenceError> {
    if seal.study_digest != config.study_digest || seal.protocol_digest != config.protocol_digest {
        return Err(CanaryEvidenceError::StudyBindingMismatch);
    }
    if seal.trial_id == 0
        || seal.subject_scope_digest == 0
        || seal.context_digest == 0
        || seal.consent_digest == 0
        || seal.operator_digest == 0
        || seal.issued_at_ms >= seal.not_before_ms
        || seal.not_before_ms >= seal.expires_at_ms
        || seal.arms.is_empty()
        || seal.arms.iter().any(|arm| arm.policy_digest_fnv1a64 == 0)
        || seal
            .arms
            .windows(2)
            .any(|pair| pair[0].variant >= pair[1].variant)
    {
        return Err(CanaryEvidenceError::MalformedSeal(seal.trial_id));
    }
    if seal.seal_digest_fnv1a64 != canonical_seal_digest(seal) {
        return Err(CanaryEvidenceError::SealDigestMismatch(seal.trial_id));
    }
    Ok(())
}

fn validate_seal_against_outcomes(
    seal: &CanaryTrialSeal,
    outcomes: &InteractionOutcomeLedger,
) -> Result<(), CanaryEvidenceError> {
    let trial = outcomes
        .trials()
        .get(&seal.trial_id)
        .ok_or(CanaryEvidenceError::UnknownTrial(seal.trial_id))?;
    let rebuilt = build_seal(
        CanaryStudyConfig {
            study_digest: seal.study_digest,
            protocol_digest: seal.protocol_digest,
            split_policy: split_policy_for_seal(seal),
            allow_synthetic_fixture: true,
        },
        trial,
        seal.consent_digest,
        seal.operator_digest,
    )?;
    if rebuilt.trial_id != seal.trial_id
        || rebuilt.subject_scope_digest != seal.subject_scope_digest
        || rebuilt.context_digest != seal.context_digest
        || rebuilt.issued_at_ms != seal.issued_at_ms
        || rebuilt.not_before_ms != seal.not_before_ms
        || rebuilt.expires_at_ms != seal.expires_at_ms
        || rebuilt.delivered_variant != seal.delivered_variant
        || rebuilt.arms != seal.arms
        || rebuilt.split != seal.split
    {
        return Err(CanaryEvidenceError::TrialChangedAfterSeal(seal.trial_id));
    }
    Ok(())
}

fn split_policy_for_seal(seal: &CanaryTrialSeal) -> EvaluationSplitPolicy {
    match seal.split {
        EvaluationSplit::TemporalHoldout => EvaluationSplitPolicy {
            temporal_holdout_start_ms: seal.issued_at_ms,
            opaque_subject_modulus: 2,
            opaque_subject_remainder: (seal.subject_scope_digest + 1) % 2,
        },
        EvaluationSplit::OpaqueSubjectHoldout => EvaluationSplitPolicy {
            temporal_holdout_start_ms: seal.expires_at_ms.saturating_add(1),
            opaque_subject_modulus: 2,
            opaque_subject_remainder: seal.subject_scope_digest % 2,
        },
        EvaluationSplit::Development => EvaluationSplitPolicy {
            temporal_holdout_start_ms: seal.expires_at_ms.saturating_add(1),
            opaque_subject_modulus: 2,
            opaque_subject_remainder: (seal.subject_scope_digest + 1) % 2,
        },
    }
}

fn validate_direct_attestation(
    config: CanaryStudyConfig,
    seal: &CanaryTrialSeal,
    attestation: &DirectCanaryAttestation,
) -> Result<(), CanaryEvidenceError> {
    validate_origin(config, attestation.origin)?;
    if attestation.trial_id != seal.trial_id {
        return Err(CanaryEvidenceError::TrialBindingMismatch);
    }
    if attestation.source == WitnessSource::ResponseGenerator {
        return Err(CanaryEvidenceError::ResponseGeneratorWitnessRejected);
    }
    if attestation.witness_digest == 0
        || attestation.producer_digest == 0
        || attestation.consent_digest == 0
        || attestation.evidence_digest == 0
    {
        return Err(CanaryEvidenceError::EmptyWitnessEvidence);
    }
    if attestation.witness_digest == attestation.producer_digest {
        return Err(CanaryEvidenceError::WitnessNotIndependent);
    }
    if attestation.consent_digest != seal.consent_digest {
        return Err(CanaryEvidenceError::ConsentMismatch);
    }
    validate_observation_time(seal, attestation.observed_at_ms)?;
    if seal.delivered_variant.is_none() {
        return Err(CanaryEvidenceError::MissingDeliveredArm(seal.trial_id));
    }
    Ok(())
}

fn validate_pairwise_attestation(
    config: CanaryStudyConfig,
    seal: &CanaryTrialSeal,
    attestation: &PairwiseCanaryAttestation,
) -> Result<(), CanaryEvidenceError> {
    validate_origin(config, attestation.origin)?;
    if attestation.trial_id != seal.trial_id {
        return Err(CanaryEvidenceError::TrialBindingMismatch);
    }
    if attestation.left_variant == attestation.right_variant
        || !seal
            .arms
            .iter()
            .any(|arm| arm.variant == attestation.left_variant)
        || !seal
            .arms
            .iter()
            .any(|arm| arm.variant == attestation.right_variant)
    {
        return Err(CanaryEvidenceError::InvalidPairwiseArms);
    }
    if attestation.left_render_digest == 0
        || attestation.right_render_digest == 0
        || attestation.left_render_digest == attestation.right_render_digest
        || attestation.blinded_order_digest == 0
        || attestation.evaluator_digest == 0
        || attestation.producer_digest == 0
        || attestation.consent_digest == 0
        || attestation.evidence_digest == 0
    {
        return Err(CanaryEvidenceError::InvalidPairwiseEvidence);
    }
    if attestation.evaluator_digest == attestation.producer_digest {
        return Err(CanaryEvidenceError::WitnessNotIndependent);
    }
    if attestation.consent_digest != seal.consent_digest {
        return Err(CanaryEvidenceError::ConsentMismatch);
    }
    validate_observation_time(seal, attestation.observed_at_ms)
}

fn validate_origin(
    config: CanaryStudyConfig,
    origin: CanaryEvidenceOrigin,
) -> Result<(), CanaryEvidenceError> {
    if origin == CanaryEvidenceOrigin::SyntheticFixture && !config.allow_synthetic_fixture {
        return Err(CanaryEvidenceError::SyntheticFixtureRejected);
    }
    Ok(())
}

fn validate_observation_time(
    seal: &CanaryTrialSeal,
    observed_at_ms: u64,
) -> Result<(), CanaryEvidenceError> {
    if observed_at_ms < seal.not_before_ms {
        return Err(CanaryEvidenceError::EvidenceBeforeWindow);
    }
    if observed_at_ms > seal.expires_at_ms {
        return Err(CanaryEvidenceError::EvidenceAfterWindow);
    }
    Ok(())
}

fn exported_evidence_trial_id(event: &CanaryEvidenceEvent) -> InteractionTrialId {
    match event {
        CanaryEvidenceEvent::DirectImported { attestation, .. } => attestation.trial_id,
        CanaryEvidenceEvent::PairwiseImported { attestation, .. } => attestation.trial_id,
        CanaryEvidenceEvent::TrialSealed { seal } => seal.trial_id,
    }
}

fn canonical_pair(
    trial_id: InteractionTrialId,
    left: PolicyVariant,
    right: PolicyVariant,
) -> (InteractionTrialId, PolicyVariant, PolicyVariant) {
    if left < right {
        (trial_id, left, right)
    } else {
        (trial_id, right, left)
    }
}

fn canonical_seal_digest(seal: &CanaryTrialSeal) -> u64 {
    let mut digest = fnv1a64(b"s6c-canary-trial-seal-v1");
    for value in [
        seal.study_digest,
        seal.protocol_digest,
        seal.trial_id,
        seal.subject_scope_digest,
        seal.context_digest,
        seal.issued_at_ms,
        seal.not_before_ms,
        seal.expires_at_ms,
        split_code(seal.split),
        seal.delivered_variant.map(variant_code).unwrap_or(u64::MAX),
        seal.consent_digest,
        seal.operator_digest,
    ] {
        digest = mix_u64(digest, value);
    }
    for arm in &seal.arms {
        digest = mix_u64(digest, variant_code(arm.variant));
        digest = mix_u64(digest, arm.policy_digest_fnv1a64);
    }
    digest
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = FNV_OFFSET_BASIS;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    digest
}

fn mix_u64(mut digest: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    digest
}

const fn split_code(split: EvaluationSplit) -> u64 {
    match split {
        EvaluationSplit::Development => 1,
        EvaluationSplit::OpaqueSubjectHoldout => 2,
        EvaluationSplit::TemporalHoldout => 3,
    }
}

const fn variant_code(variant: PolicyVariant) -> u64 {
    match variant {
        PolicyVariant::CompanionDerived => 1,
        PolicyVariant::NeutralDefault => 2,
        PolicyVariant::RecencyOnly => 3,
        PolicyVariant::MajorityPrior => 4,
        PolicyVariant::ContextOnly => 5,
        PolicyVariant::ScrambledScope => 6,
    }
}

fn count_origin(origin: CanaryEvidenceOrigin, real: &mut u64, synthetic: &mut u64) {
    match origin {
        CanaryEvidenceOrigin::RealInteraction => *real += 1,
        CanaryEvidenceOrigin::SyntheticFixture => *synthetic += 1,
    }
}

fn split_has_trials(report: &PolicyEvaluationReport, split: EvaluationSplit) -> bool {
    report.splits.iter().any(|entry| {
        entry.split == split && entry.arms.values().any(|metrics| metrics.trials > 0)
    })
}

#[derive(Debug, Error)]
pub enum CanaryEvidenceError {
    #[error("invalid S6-C study configuration")]
    InvalidStudyConfig,
    #[error("canary ledger version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("unknown S5-B interaction trial {0}")]
    UnknownTrial(InteractionTrialId),
    #[error("trial {0} has not been sealed for S6-C intake")]
    UnsealedTrial(InteractionTrialId),
    #[error("trial {0} was sealed more than once")]
    DuplicateSeal(InteractionTrialId),
    #[error("seal consent and operator evidence must be non-zero")]
    EmptySealEvidence,
    #[error("trial {0} is malformed")]
    MalformedTrial(InteractionTrialId),
    #[error("trial seal {0} is malformed")]
    MalformedSeal(InteractionTrialId),
    #[error("trial seal {0} has a non-canonical digest")]
    SealDigestMismatch(InteractionTrialId),
    #[error("trial seal does not match the configured study and protocol")]
    StudyBindingMismatch,
    #[error("trial {0} changed after S6-C sealing")]
    TrialChangedAfterSeal(InteractionTrialId),
    #[error("synthetic canary evidence is rejected by production-default intake")]
    SyntheticFixtureRejected,
    #[error("the response generator cannot witness its own outcome")]
    ResponseGeneratorWitnessRejected,
    #[error("witness or evaluator identity is not independent from the producer")]
    WitnessNotIndependent,
    #[error("witness evidence digests must be non-zero")]
    EmptyWitnessEvidence,
    #[error("attestation consent does not match the sealed trial")]
    ConsentMismatch,
    #[error("attestation references a different trial")]
    TrialBindingMismatch,
    #[error("evidence arrived before the sealed observation window")]
    EvidenceBeforeWindow,
    #[error("evidence arrived after the sealed observation window")]
    EvidenceAfterWindow,
    #[error("trial {0} has no delivered arm for direct evidence")]
    MissingDeliveredArm(InteractionTrialId),
    #[error("trial {0} already has direct evidence")]
    DuplicateDirectEvidence(InteractionTrialId),
    #[error("pairwise attestation references invalid or identical arms")]
    InvalidPairwiseArms,
    #[error("pairwise evidence digests are incomplete or non-distinct")]
    InvalidPairwiseEvidence,
    #[error("trial {trial_id} already has evidence for pair {left:?} vs {right:?}")]
    DuplicatePairwiseEvidence {
        trial_id: InteractionTrialId,
        left: PolicyVariant,
        right: PolicyVariant,
    },
    #[error("typed S5-B export does not match its source attestation")]
    ExportMismatch,
    #[error("S5-C evaluation contains a trial that was not sealed by this canary")]
    UnsealedEvaluationTrial,
    #[error("canary version overflow")]
    VersionOverflow,
    #[error(transparent)]
    Outcome(#[from] InteractionOutcomeError),
    #[error(transparent)]
    Evaluation(#[from] PolicyEvaluationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(allow_synthetic_fixture: bool) -> CanaryStudyConfig {
        CanaryStudyConfig {
            study_digest: 1,
            protocol_digest: 2,
            split_policy: EvaluationSplitPolicy {
                temporal_holdout_start_ms: 10_000,
                opaque_subject_modulus: 2,
                opaque_subject_remainder: 1,
            },
            allow_synthetic_fixture,
        }
    }

    fn seal() -> CanaryTrialSeal {
        let mut seal = CanaryTrialSeal {
            study_digest: 1,
            protocol_digest: 2,
            trial_id: 1,
            subject_scope_digest: 2,
            context_digest: 3,
            issued_at_ms: 100,
            not_before_ms: 110,
            expires_at_ms: 200,
            split: EvaluationSplit::Development,
            delivered_variant: Some(PolicyVariant::CompanionDerived),
            arms: vec![CanaryArmSeal {
                variant: PolicyVariant::CompanionDerived,
                policy_digest_fnv1a64: 4,
            }],
            consent_digest: 5,
            operator_digest: 6,
            seal_digest_fnv1a64: 0,
        };
        seal.seal_digest_fnv1a64 = canonical_seal_digest(&seal);
        seal
    }

    #[test]
    fn replay_rejects_a_tampered_seal() {
        let mut tampered = seal();
        tampered.consent_digest += 1;
        assert!(matches!(
            CanaryEvidenceLedger::replay(
                config(true),
                &[CanaryEvidenceEvent::TrialSealed { seal: tampered }]
            ),
            Err(CanaryEvidenceError::SealDigestMismatch(1))
        ));
    }

    #[test]
    fn production_default_rejects_synthetic_attestation() {
        let mut ledger = CanaryEvidenceLedger::new(config(false)).unwrap();
        ledger
            .apply_event(CanaryEvidenceEvent::TrialSealed { seal: seal() })
            .unwrap();
        let attestation = DirectCanaryAttestation {
            origin: CanaryEvidenceOrigin::SyntheticFixture,
            trial_id: 1,
            signal: ObservedSignal::TaskCompleted,
            source: WitnessSource::Environment,
            witness_digest: 7,
            producer_digest: 8,
            consent_digest: 5,
            observed_at_ms: 120,
            evidence_digest: 9,
        };
        let exported_evidence = ObservedOutcomeEvidence {
            signal: attestation.signal,
            source: attestation.source,
            observed_at_ms: attestation.observed_at_ms,
            evidence_digest: attestation.evidence_digest,
        };
        assert!(matches!(
            ledger.apply_event(CanaryEvidenceEvent::DirectImported {
                attestation,
                exported_evidence,
            }),
            Err(CanaryEvidenceError::SyntheticFixtureRejected)
        ));
        assert_eq!(ledger.version, 1);
    }

    #[test]
    fn replay_is_deterministic_for_valid_typed_events() {
        let mut ledger = CanaryEvidenceLedger::new(config(true)).unwrap();
        ledger
            .apply_event(CanaryEvidenceEvent::TrialSealed { seal: seal() })
            .unwrap();
        let attestation = DirectCanaryAttestation {
            origin: CanaryEvidenceOrigin::SyntheticFixture,
            trial_id: 1,
            signal: ObservedSignal::NeutralFollowUp,
            source: WitnessSource::UserObservation,
            witness_digest: 7,
            producer_digest: 8,
            consent_digest: 5,
            observed_at_ms: 120,
            evidence_digest: 9,
        };
        ledger
            .apply_event(CanaryEvidenceEvent::DirectImported {
                exported_evidence: ObservedOutcomeEvidence {
                    signal: attestation.signal,
                    source: attestation.source,
                    observed_at_ms: attestation.observed_at_ms,
                    evidence_digest: attestation.evidence_digest,
                },
                attestation,
            })
            .unwrap();
        let replayed = CanaryEvidenceLedger::replay(config(true), ledger.events()).unwrap();
        assert_eq!(replayed, ledger);
    }
}
