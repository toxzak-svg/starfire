//! EI-0D reversible bounded learning-update transactions.
//!
//! This module is offline-only and feature-gated. It binds typed fixed-schema
//! update proposals to sealed EI-0A episodes, an EI-0C ledger root, and isolated
//! EI-0B control-arm state. It can atomically update only its own bounded policy
//! object and can restore byte-identical prior state. It has no `Runtime::chat()`,
//! live SQLite, response, routing, belief, ontology, tool, or autonomous-action
//! authority.

use crate::emerging_intelligence::{
    AuthoritySnapshot, EpisodeContractError, EpisodePhase, SealedCognitiveEpisode,
};
use crate::emerging_intelligence_environment::{
    generate_frozen_fixture, ControlArm, EnvironmentError, FrozenEnvironmentManifest,
    SealedTaskFixture, TaskPayload,
};
use crate::emerging_intelligence_ledger::{AppendOnlyEpisodeLedger, LedgerError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const EI_0D_SCHEMA_VERSION: u16 = 1;
pub const EI_0D_ENGINE_ID: &str = "ei-0d-reversible-update-engine-v1";
pub const EI_0D_ADMISSIBILITY_EVALUATOR_ID: &str = "ei-0d-admissibility-evaluator-v1";
pub const EI_0D_SAFETY_EVALUATOR_ID: &str = "ei-0d-heldout-safety-evaluator-v1";
pub const EI_0D_CLAIM: &str = "ei-0d-reversible-update-infrastructure-only";
pub const MAX_SINGLE_UPDATE_BPS: i32 = 10_000;
pub const MAX_CUMULATIVE_UPDATE_BPS: u32 = 20_000;

const DIGEST_HEX_LEN: usize = 32;
const STATE_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0d/policy-state/v1";
const PROPOSAL_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0d/update-proposal/v1";
const TRANSACTION_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0d/update-transaction/v1";
const ROLLBACK_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0d/rollback-receipt/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UpdateDigest(String);

impl UpdateDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), UpdateError> {
        validate_digest_text(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySlot {
    RouteCostWeightBps,
    RouteDecoyBiasBps,
    VerifiedCueWeightBps,
    RuleCoverageWeightBps,
    RuleDecoyBiasBps,
}

impl PolicySlot {
    pub const ALL: [Self; 5] = [
        Self::RouteCostWeightBps,
        Self::RouteDecoyBiasBps,
        Self::VerifiedCueWeightBps,
        Self::RuleCoverageWeightBps,
        Self::RuleDecoyBiasBps,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RouteCostWeightBps => "route_cost_weight_bps",
            Self::RouteDecoyBiasBps => "route_decoy_bias_bps",
            Self::VerifiedCueWeightBps => "verified_cue_weight_bps",
            Self::RuleCoverageWeightBps => "rule_coverage_weight_bps",
            Self::RuleDecoyBiasBps => "rule_decoy_bias_bps",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsolatedPolicyState {
    pub schema_version: u16,
    pub state_id: String,
    pub arm: ControlArm,
    pub state_namespace: String,
    pub route_cost_weight_bps: i32,
    pub route_decoy_bias_bps: i32,
    pub verified_cue_weight_bps: i32,
    pub rule_coverage_weight_bps: i32,
    pub rule_decoy_bias_bps: i32,
    pub cumulative_abs_delta_bps: u32,
    pub authority: AuthoritySnapshot,
}

impl IsolatedPolicyState {
    pub fn baseline(arm: ControlArm, state_namespace: impl Into<String>) -> Result<Self, UpdateError> {
        let state = Self {
            schema_version: EI_0D_SCHEMA_VERSION,
            state_id: format!("ei-0d-state-{}", arm.as_str()),
            arm,
            state_namespace: state_namespace.into(),
            route_cost_weight_bps: 10_000,
            route_decoy_bias_bps: 0,
            verified_cue_weight_bps: 0,
            rule_coverage_weight_bps: 10_000,
            rule_decoy_bias_bps: 0,
            cumulative_abs_delta_bps: 0,
            authority: AuthoritySnapshot::closed(),
        };
        state.validate()?;
        Ok(state)
    }

    pub fn validate(&self) -> Result<(), UpdateError> {
        if self.schema_version != EI_0D_SCHEMA_VERSION {
            return Err(UpdateError::UnsupportedSchemaVersion(self.schema_version));
        }
        validate_identifier(&self.state_id)?;
        validate_identifier(&self.state_namespace)?;
        if !self.authority.is_closed() {
            return Err(UpdateError::UnauthorizedState);
        }
        for slot in PolicySlot::ALL {
            validate_slot_value(slot, self.value(slot))?;
        }
        if self.cumulative_abs_delta_bps > MAX_CUMULATIVE_UPDATE_BPS {
            return Err(UpdateError::CumulativeBudgetExceeded);
        }
        Ok(())
    }

    pub fn value(&self, slot: PolicySlot) -> i32 {
        match slot {
            PolicySlot::RouteCostWeightBps => self.route_cost_weight_bps,
            PolicySlot::RouteDecoyBiasBps => self.route_decoy_bias_bps,
            PolicySlot::VerifiedCueWeightBps => self.verified_cue_weight_bps,
            PolicySlot::RuleCoverageWeightBps => self.rule_coverage_weight_bps,
            PolicySlot::RuleDecoyBiasBps => self.rule_decoy_bias_bps,
        }
    }

    fn set_value(&mut self, slot: PolicySlot, value: i32) {
        match slot {
            PolicySlot::RouteCostWeightBps => self.route_cost_weight_bps = value,
            PolicySlot::RouteDecoyBiasBps => self.route_decoy_bias_bps = value,
            PolicySlot::VerifiedCueWeightBps => self.verified_cue_weight_bps = value,
            PolicySlot::RuleCoverageWeightBps => self.rule_coverage_weight_bps = value,
            PolicySlot::RuleDecoyBiasBps => self.rule_decoy_bias_bps = value,
        }
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, UpdateError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, UpdateError> {
        let state: Self = serde_json::from_slice(bytes).map_err(deserialization_error)?;
        state.validate()?;
        let canonical = serde_json::to_vec(&state).map_err(serialization_error)?;
        if canonical != bytes {
            return Err(UpdateError::NonCanonicalEncoding);
        }
        Ok(state)
    }

    pub fn digest(&self) -> Result<UpdateDigest, UpdateError> {
        digest_serialized(STATE_DIGEST_DOMAIN, self)
    }

    pub fn select_action(&self, fixture: &SealedTaskFixture) -> Result<String, UpdateError> {
        self.validate()?;
        fixture.validate()?;
        let mut legal = fixture.fixture.task.legal_actions();
        legal.sort();
        let decoy = legal.last().cloned().ok_or(UpdateError::EmptyTask)?;
        let verified = fixture
            .fixture
            .evidence_cues
            .iter()
            .find_map(|cue| cue.cue.strip_prefix("verified:"))
            .map(str::to_owned);

        let selected = match &fixture.fixture.task {
            TaskPayload::RouteChoice(task) => task
                .options
                .iter()
                .max_by_key(|option| {
                    let verified_bonus = if verified.as_deref() == Some(option.action.as_str()) {
                        i64::from(self.verified_cue_weight_bps)
                    } else {
                        0
                    };
                    let decoy_bonus = if option.action == decoy {
                        i64::from(self.route_decoy_bias_bps) * 10
                    } else {
                        0
                    };
                    -(i64::from(option.total_cost) * i64::from(self.route_cost_weight_bps))
                        + verified_bonus
                        + decoy_bonus
                })
                .map(|option| option.action.clone()),
            TaskPayload::AttributeRule(task) => task
                .candidates
                .iter()
                .max_by_key(|candidate| {
                    let positive_overlap: i64 = task
                        .examples
                        .iter()
                        .filter(|example| example.matches)
                        .map(|example| overlap_count(&candidate.attributes, &example.attributes))
                        .sum();
                    let negative_overlap: i64 = task
                        .examples
                        .iter()
                        .filter(|example| !example.matches)
                        .map(|example| overlap_count(&candidate.attributes, &example.attributes))
                        .sum();
                    let evidence_score = (positive_overlap * 2 - negative_overlap)
                        * i64::from(self.rule_coverage_weight_bps);
                    let verified_bonus = if verified.as_deref() == Some(candidate.action.as_str()) {
                        i64::from(self.verified_cue_weight_bps)
                    } else {
                        0
                    };
                    let decoy_bonus = if candidate.action == decoy {
                        i64::from(self.rule_decoy_bias_bps) * 10
                    } else {
                        0
                    };
                    evidence_score + verified_bonus + decoy_bonus
                })
                .map(|candidate| candidate.action.clone()),
        };
        selected.ok_or(UpdateError::EmptyTask)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateProposal {
    pub schema_version: u16,
    pub update_id: String,
    pub source_episode_id: String,
    pub source_episode_digest: String,
    pub source_ledger_root: String,
    pub arm: ControlArm,
    pub state_namespace: String,
    pub expected_pre_state_digest: String,
    pub slot: PolicySlot,
    pub before_value: i32,
    pub after_value: i32,
    pub proposal_digest: UpdateDigest,
    pub authority: AuthoritySnapshot,
}

impl UpdateProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        update_id: impl Into<String>,
        source_episode: &SealedCognitiveEpisode,
        source_ledger: &AppendOnlyEpisodeLedger,
        state: &IsolatedPolicyState,
        slot: PolicySlot,
        after_value: i32,
    ) -> Result<Self, UpdateError> {
        source_episode.validate()?;
        source_ledger.validate()?;
        state.validate()?;
        let mut proposal = Self {
            schema_version: EI_0D_SCHEMA_VERSION,
            update_id: update_id.into(),
            source_episode_id: source_episode.episode.episode_id.as_str().to_owned(),
            source_episode_digest: source_episode.digest.as_str().to_owned(),
            source_ledger_root: source_ledger.root_digest().as_str().to_owned(),
            arm: state.arm,
            state_namespace: state.state_namespace.clone(),
            expected_pre_state_digest: state.digest()?.as_str().to_owned(),
            slot,
            before_value: state.value(slot),
            after_value,
            proposal_digest: UpdateDigest(String::new()),
            authority: AuthoritySnapshot::closed(),
        };
        proposal.proposal_digest = proposal.compute_digest()?;
        proposal.validate_shape()?;
        Ok(proposal)
    }

    pub fn validate_shape(&self) -> Result<(), UpdateError> {
        if self.schema_version != EI_0D_SCHEMA_VERSION {
            return Err(UpdateError::UnsupportedSchemaVersion(self.schema_version));
        }
        validate_identifier(&self.update_id)?;
        validate_identifier(&self.source_episode_id)?;
        validate_digest_text(&self.source_episode_digest)?;
        validate_digest_text(&self.source_ledger_root)?;
        validate_identifier(&self.state_namespace)?;
        validate_digest_text(&self.expected_pre_state_digest)?;
        validate_slot_value(self.slot, self.before_value)?;
        validate_slot_value(self.slot, self.after_value)?;
        if !self.authority.is_closed() {
            return Err(UpdateError::UnauthorizedProposal);
        }
        self.proposal_digest.validate()?;
        if self.proposal_digest != self.compute_digest()? {
            return Err(UpdateError::ProposalDigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, UpdateError> {
        self.validate_shape()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, UpdateError> {
        let proposal: Self = serde_json::from_slice(bytes).map_err(deserialization_error)?;
        proposal.validate_shape()?;
        let canonical = serde_json::to_vec(&proposal).map_err(serialization_error)?;
        if canonical != bytes {
            return Err(UpdateError::NonCanonicalEncoding);
        }
        Ok(proposal)
    }

    fn compute_digest(&self) -> Result<UpdateDigest, UpdateError> {
        let payload = ProposalDigestPayload {
            schema_version: self.schema_version,
            update_id: &self.update_id,
            source_episode_id: &self.source_episode_id,
            source_episode_digest: &self.source_episode_digest,
            source_ledger_root: &self.source_ledger_root,
            arm: self.arm,
            state_namespace: &self.state_namespace,
            expected_pre_state_digest: &self.expected_pre_state_digest,
            slot: self.slot,
            before_value: self.before_value,
            after_value: self.after_value,
            authority: &self.authority,
        };
        digest_serialized(PROPOSAL_DIGEST_DOMAIN, &payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissibilityVerdict {
    pub evaluator_id: String,
    pub admissible: bool,
    pub reason: String,
    pub absolute_delta_bps: u32,
    pub projected_cumulative_delta_bps: u32,
    pub authority: AuthoritySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionSafetyScore {
    pub partition: String,
    pub fixture_count: u32,
    pub pre_score_bps: u16,
    pub post_score_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldoutSafetyEvaluation {
    pub evaluator_id: String,
    pub harmful: bool,
    pub reason: String,
    pub pre_score_bps: u16,
    pub post_score_bps: u16,
    pub partition_scores: Vec<PartitionSafetyScore>,
    pub authority: AuthoritySnapshot,
}

pub struct IndependentAdmissibilityEvaluator;

impl IndependentAdmissibilityEvaluator {
    pub fn evaluate(
        state: &IsolatedPolicyState,
        proposal: &UpdateProposal,
        ledger: &AppendOnlyEpisodeLedger,
    ) -> Result<AdmissibilityVerdict, UpdateError> {
        state.validate()?;
        proposal.validate_shape()?;
        ledger.validate()?;
        if proposal.arm != state.arm {
            return Err(UpdateError::ArmMismatch);
        }
        if proposal.state_namespace != state.state_namespace {
            return Err(UpdateError::NamespaceMismatch);
        }
        if proposal.expected_pre_state_digest != state.digest()?.as_str() {
            return Err(UpdateError::StalePreState);
        }
        if proposal.before_value != state.value(proposal.slot) {
            return Err(UpdateError::BeforeValueMismatch);
        }
        if proposal.source_ledger_root != ledger.root_digest().as_str() {
            return Err(UpdateError::LedgerRootMismatch);
        }

        let replay = ledger.replay_fresh()?;
        let source = replay
            .episodes
            .iter()
            .find(|episode| episode.episode.episode_id.as_str() == proposal.source_episode_id)
            .ok_or(UpdateError::SourceEpisodeMissing)?;
        if source.digest.as_str() != proposal.source_episode_digest {
            return Err(UpdateError::SourceEpisodeDigestMismatch);
        }
        if source.episode.phase != EpisodePhase::Evaluated {
            return Err(UpdateError::SourceEpisodeNotEvaluated);
        }
        if !source
            .episode
            .accepted_updates
            .iter()
            .any(|update_id| update_id.as_str() == proposal.update_id)
        {
            return Err(UpdateError::SourceUpdateNotAccepted);
        }

        let absolute_delta_bps = proposal.after_value.abs_diff(proposal.before_value);
        if absolute_delta_bps > MAX_SINGLE_UPDATE_BPS as u32 {
            return Err(UpdateError::SingleUpdateBudgetExceeded);
        }
        let projected_cumulative_delta_bps = state
            .cumulative_abs_delta_bps
            .checked_add(absolute_delta_bps)
            .ok_or(UpdateError::CumulativeBudgetExceeded)?;
        if projected_cumulative_delta_bps > MAX_CUMULATIVE_UPDATE_BPS {
            return Err(UpdateError::CumulativeBudgetExceeded);
        }

        Ok(AdmissibilityVerdict {
            evaluator_id: EI_0D_ADMISSIBILITY_EVALUATOR_ID.into(),
            admissible: true,
            reason: "proposal-valid-and-provenance-bound".into(),
            absolute_delta_bps,
            projected_cumulative_delta_bps,
            authority: AuthoritySnapshot::closed(),
        })
    }
}

pub struct IndependentHeldoutSafetyEvaluator;

impl IndependentHeldoutSafetyEvaluator {
    pub fn evaluate(
        pre_state: &IsolatedPolicyState,
        post_state: &IsolatedPolicyState,
    ) -> Result<HeldoutSafetyEvaluation, UpdateError> {
        pre_state.validate()?;
        post_state.validate()?;
        let manifest = FrozenEnvironmentManifest::ei_0b_default();
        manifest.validate()?;
        let protected = [
            crate::emerging_intelligence::EvaluationPartition::WithinFamilyHoldout,
            crate::emerging_intelligence::EvaluationPartition::RenamedVocabularyTransfer,
            crate::emerging_intelligence::EvaluationPartition::Regression,
            crate::emerging_intelligence::EvaluationPartition::Adversarial,
        ];
        let mut partition_scores = Vec::new();
        let mut pre_total: u64 = 0;
        let mut post_total: u64 = 0;
        let mut total_count: u64 = 0;
        let mut harmful = false;

        for partition in protected {
            let seeds = manifest
                .partitions
                .iter()
                .find(|entry| entry.partition == partition)
                .ok_or(UpdateError::IncompleteSafetyFixtureSet)?;
            let mut pre_partition: u64 = 0;
            let mut post_partition: u64 = 0;
            for seed in &seeds.seeds {
                let fixture = generate_frozen_fixture(&manifest, partition, *seed)?;
                let pre_action = pre_state.select_action(&fixture)?;
                let post_action = post_state.select_action(&fixture)?;
                pre_partition += if pre_action == fixture.fixture.optimal_action {
                    10_000
                } else {
                    0
                };
                post_partition += if post_action == fixture.fixture.optimal_action {
                    10_000
                } else {
                    0
                };
            }
            let count = u64::try_from(seeds.seeds.len())
                .map_err(|_| UpdateError::IncompleteSafetyFixtureSet)?;
            if count == 0 {
                return Err(UpdateError::IncompleteSafetyFixtureSet);
            }
            let pre_score = u16::try_from(pre_partition / count)
                .map_err(|_| UpdateError::SafetyScoreOverflow)?;
            let post_score = u16::try_from(post_partition / count)
                .map_err(|_| UpdateError::SafetyScoreOverflow)?;
            if post_score < pre_score {
                harmful = true;
            }
            partition_scores.push(PartitionSafetyScore {
                partition: partition_name(partition).into(),
                fixture_count: u32::try_from(count)
                    .map_err(|_| UpdateError::IncompleteSafetyFixtureSet)?,
                pre_score_bps: pre_score,
                post_score_bps: post_score,
            });
            pre_total += pre_partition;
            post_total += post_partition;
            total_count += count;
        }

        let pre_score_bps = u16::try_from(pre_total / total_count)
            .map_err(|_| UpdateError::SafetyScoreOverflow)?;
        let post_score_bps = u16::try_from(post_total / total_count)
            .map_err(|_| UpdateError::SafetyScoreOverflow)?;
        if post_score_bps < pre_score_bps {
            harmful = true;
        }
        Ok(HeldoutSafetyEvaluation {
            evaluator_id: EI_0D_SAFETY_EVALUATOR_ID.into(),
            harmful,
            reason: if harmful {
                "heldout-or-regression-score-decreased"
            } else {
                "no-protected-partition-regression"
            }
            .into(),
            pre_score_bps,
            post_score_bps,
            partition_scores,
            authority: AuthoritySnapshot::closed(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Applied,
    ControlNoOp,
    RolledBackHarmful,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateTransaction {
    pub schema_version: u16,
    pub transaction_id: String,
    pub engine_id: String,
    pub update_id: String,
    pub proposal_digest: UpdateDigest,
    pub status: TransactionStatus,
    pub pre_state_bytes: Vec<u8>,
    pub attempted_post_state_bytes: Vec<u8>,
    pub final_state_bytes: Vec<u8>,
    pub pre_state_digest: UpdateDigest,
    pub attempted_post_state_digest: UpdateDigest,
    pub final_state_digest: UpdateDigest,
    pub admissibility: AdmissibilityVerdict,
    pub safety: HeldoutSafetyEvaluation,
    pub transaction_digest: UpdateDigest,
    pub authority: AuthoritySnapshot,
}

impl UpdateTransaction {
    pub fn validate(&self) -> Result<(), UpdateError> {
        if self.schema_version != EI_0D_SCHEMA_VERSION {
            return Err(UpdateError::UnsupportedSchemaVersion(self.schema_version));
        }
        validate_identifier(&self.transaction_id)?;
        if self.engine_id != EI_0D_ENGINE_ID {
            return Err(UpdateError::InvalidEngineId);
        }
        validate_identifier(&self.update_id)?;
        self.proposal_digest.validate()?;
        self.pre_state_digest.validate()?;
        self.attempted_post_state_digest.validate()?;
        self.final_state_digest.validate()?;
        self.transaction_digest.validate()?;
        if !self.authority.is_closed()
            || !self.admissibility.authority.is_closed()
            || !self.safety.authority.is_closed()
        {
            return Err(UpdateError::UnauthorizedTransaction);
        }
        let pre = IsolatedPolicyState::from_canonical_bytes(&self.pre_state_bytes)?;
        let attempted =
            IsolatedPolicyState::from_canonical_bytes(&self.attempted_post_state_bytes)?;
        let final_state = IsolatedPolicyState::from_canonical_bytes(&self.final_state_bytes)?;
        if pre.digest()? != self.pre_state_digest
            || attempted.digest()? != self.attempted_post_state_digest
            || final_state.digest()? != self.final_state_digest
        {
            return Err(UpdateError::TransactionStateDigestMismatch);
        }
        match self.status {
            TransactionStatus::Applied => {
                if self.safety.harmful || self.final_state_digest != self.attempted_post_state_digest {
                    return Err(UpdateError::InvalidTransactionStatus);
                }
            }
            TransactionStatus::ControlNoOp => {
                if self.pre_state_digest != self.attempted_post_state_digest
                    || self.pre_state_digest != self.final_state_digest
                {
                    return Err(UpdateError::InvalidTransactionStatus);
                }
            }
            TransactionStatus::RolledBackHarmful => {
                if !self.safety.harmful || self.final_state_digest != self.pre_state_digest {
                    return Err(UpdateError::InvalidTransactionStatus);
                }
            }
        }
        if self.transaction_digest != self.compute_digest()? {
            return Err(UpdateError::TransactionDigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, UpdateError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, UpdateError> {
        let transaction: Self = serde_json::from_slice(bytes).map_err(deserialization_error)?;
        transaction.validate()?;
        let canonical = serde_json::to_vec(&transaction).map_err(serialization_error)?;
        if canonical != bytes {
            return Err(UpdateError::NonCanonicalEncoding);
        }
        Ok(transaction)
    }

    fn compute_digest(&self) -> Result<UpdateDigest, UpdateError> {
        let payload = TransactionDigestPayload {
            schema_version: self.schema_version,
            transaction_id: &self.transaction_id,
            engine_id: &self.engine_id,
            update_id: &self.update_id,
            proposal_digest: self.proposal_digest.as_str(),
            status: self.status,
            pre_state_digest: self.pre_state_digest.as_str(),
            attempted_post_state_digest: self.attempted_post_state_digest.as_str(),
            final_state_digest: self.final_state_digest.as_str(),
            admissibility: &self.admissibility,
            safety: &self.safety,
            authority: &self.authority,
        };
        digest_serialized(TRANSACTION_DIGEST_DOMAIN, &payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackReceipt {
    pub schema_version: u16,
    pub rollback_id: String,
    pub transaction_id: String,
    pub restored_state_digest: UpdateDigest,
    pub rollback_digest: UpdateDigest,
    pub authority: AuthoritySnapshot,
}

impl RollbackReceipt {
    fn compute_digest(&self) -> Result<UpdateDigest, UpdateError> {
        let payload = RollbackDigestPayload {
            schema_version: self.schema_version,
            rollback_id: &self.rollback_id,
            transaction_id: &self.transaction_id,
            restored_state_digest: self.restored_state_digest.as_str(),
            authority: &self.authority,
        };
        digest_serialized(ROLLBACK_DIGEST_DOMAIN, &payload)
    }

    pub fn validate(&self) -> Result<(), UpdateError> {
        if self.schema_version != EI_0D_SCHEMA_VERSION {
            return Err(UpdateError::UnsupportedSchemaVersion(self.schema_version));
        }
        validate_identifier(&self.rollback_id)?;
        validate_identifier(&self.transaction_id)?;
        self.restored_state_digest.validate()?;
        self.rollback_digest.validate()?;
        if !self.authority.is_closed() {
            return Err(UpdateError::UnauthorizedTransaction);
        }
        if self.rollback_digest != self.compute_digest()? {
            return Err(UpdateError::RollbackDigestMismatch);
        }
        Ok(())
    }
}

pub struct ReversibleUpdateEngine {
    state: IsolatedPolicyState,
    processed_update_ids: BTreeSet<String>,
    rolled_back_transaction_ids: BTreeSet<String>,
}

impl ReversibleUpdateEngine {
    pub fn new(state: IsolatedPolicyState) -> Result<Self, UpdateError> {
        state.validate()?;
        Ok(Self {
            state,
            processed_update_ids: BTreeSet::new(),
            rolled_back_transaction_ids: BTreeSet::new(),
        })
    }

    pub fn state(&self) -> &IsolatedPolicyState {
        &self.state
    }

    pub fn apply(
        &mut self,
        proposal: &UpdateProposal,
        ledger: &AppendOnlyEpisodeLedger,
    ) -> Result<UpdateTransaction, UpdateError> {
        if self.processed_update_ids.contains(&proposal.update_id) {
            return Err(UpdateError::DuplicateUpdateId(proposal.update_id.clone()));
        }
        let admissibility =
            IndependentAdmissibilityEvaluator::evaluate(&self.state, proposal, ledger)?;
        let pre_state = self.state.clone();
        let pre_state_bytes = pre_state.to_canonical_bytes()?;
        let pre_state_digest = pre_state.digest()?;

        let mutable_arm = matches!(self.state.arm, ControlArm::Learning | ControlArm::RandomUpdate);
        let (attempted_state, status, safety) = if mutable_arm {
            let mut attempted = pre_state.clone();
            attempted.set_value(proposal.slot, proposal.after_value);
            attempted.cumulative_abs_delta_bps = admissibility.projected_cumulative_delta_bps;
            attempted.validate()?;
            let safety = IndependentHeldoutSafetyEvaluator::evaluate(&pre_state, &attempted)?;
            let status = if safety.harmful {
                TransactionStatus::RolledBackHarmful
            } else {
                TransactionStatus::Applied
            };
            (attempted, status, safety)
        } else {
            let safety = IndependentHeldoutSafetyEvaluator::evaluate(&pre_state, &pre_state)?;
            (pre_state.clone(), TransactionStatus::ControlNoOp, safety)
        };

        let attempted_post_state_bytes = attempted_state.to_canonical_bytes()?;
        let attempted_post_state_digest = attempted_state.digest()?;
        let final_state = match status {
            TransactionStatus::Applied => attempted_state.clone(),
            TransactionStatus::ControlNoOp | TransactionStatus::RolledBackHarmful => {
                pre_state.clone()
            }
        };
        let final_state_bytes = final_state.to_canonical_bytes()?;
        let final_state_digest = final_state.digest()?;
        let transaction_id = format!("transaction-{}", proposal.update_id);
        let mut transaction = UpdateTransaction {
            schema_version: EI_0D_SCHEMA_VERSION,
            transaction_id,
            engine_id: EI_0D_ENGINE_ID.into(),
            update_id: proposal.update_id.clone(),
            proposal_digest: proposal.proposal_digest.clone(),
            status,
            pre_state_bytes,
            attempted_post_state_bytes,
            final_state_bytes,
            pre_state_digest,
            attempted_post_state_digest,
            final_state_digest,
            admissibility,
            safety,
            transaction_digest: UpdateDigest(String::new()),
            authority: AuthoritySnapshot::closed(),
        };
        transaction.transaction_digest = transaction.compute_digest()?;
        transaction.validate()?;
        self.state = final_state;
        self.processed_update_ids.insert(proposal.update_id.clone());
        Ok(transaction)
    }

    pub fn rollback(
        &mut self,
        transaction: &UpdateTransaction,
    ) -> Result<RollbackReceipt, UpdateError> {
        transaction.validate()?;
        if transaction.status != TransactionStatus::Applied {
            return Err(UpdateError::TransactionNotApplied);
        }
        if self
            .rolled_back_transaction_ids
            .contains(&transaction.transaction_id)
        {
            return Err(UpdateError::DuplicateRollback);
        }
        if self.state.digest()? != transaction.final_state_digest {
            return Err(UpdateError::RollbackStateMismatch);
        }
        let restored = IsolatedPolicyState::from_canonical_bytes(&transaction.pre_state_bytes)?;
        if restored.digest()? != transaction.pre_state_digest {
            return Err(UpdateError::RollbackStateMismatch);
        }
        self.state = restored;
        self.rolled_back_transaction_ids
            .insert(transaction.transaction_id.clone());
        let rollback_id = format!("rollback-{}", transaction.transaction_id);
        let mut receipt = RollbackReceipt {
            schema_version: EI_0D_SCHEMA_VERSION,
            rollback_id,
            transaction_id: transaction.transaction_id.clone(),
            restored_state_digest: self.state.digest()?,
            rollback_digest: UpdateDigest(String::new()),
            authority: AuthoritySnapshot::closed(),
        };
        receipt.rollback_digest = receipt.compute_digest()?;
        receipt.validate()?;
        Ok(receipt)
    }
}

#[derive(Serialize)]
struct ProposalDigestPayload<'a> {
    schema_version: u16,
    update_id: &'a str,
    source_episode_id: &'a str,
    source_episode_digest: &'a str,
    source_ledger_root: &'a str,
    arm: ControlArm,
    state_namespace: &'a str,
    expected_pre_state_digest: &'a str,
    slot: PolicySlot,
    before_value: i32,
    after_value: i32,
    authority: &'a AuthoritySnapshot,
}

#[derive(Serialize)]
struct TransactionDigestPayload<'a> {
    schema_version: u16,
    transaction_id: &'a str,
    engine_id: &'a str,
    update_id: &'a str,
    proposal_digest: &'a str,
    status: TransactionStatus,
    pre_state_digest: &'a str,
    attempted_post_state_digest: &'a str,
    final_state_digest: &'a str,
    admissibility: &'a AdmissibilityVerdict,
    safety: &'a HeldoutSafetyEvaluation,
    authority: &'a AuthoritySnapshot,
}

#[derive(Serialize)]
struct RollbackDigestPayload<'a> {
    schema_version: u16,
    rollback_id: &'a str,
    transaction_id: &'a str,
    restored_state_digest: &'a str,
    authority: &'a AuthoritySnapshot,
}

fn validate_identifier(value: &str) -> Result<(), UpdateError> {
    if value.is_empty()
        || value.trim() != value
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'_' | b'/' | b':' | b'.')
        })
    {
        return Err(UpdateError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_digest_text(value: &str) -> Result<(), UpdateError> {
    if value.len() != DIGEST_HEX_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(UpdateError::InvalidDigest);
    }
    Ok(())
}

fn validate_slot_value(slot: PolicySlot, value: i32) -> Result<(), UpdateError> {
    if !(0..=10_000).contains(&value) {
        return Err(UpdateError::SlotValueOutOfRange {
            slot,
            value,
        });
    }
    Ok(())
}

fn overlap_count(left: &[String], right: &[String]) -> i64 {
    left.iter()
        .filter(|value| right.binary_search(value).is_ok())
        .count() as i64
}

fn partition_name(partition: crate::emerging_intelligence::EvaluationPartition) -> &'static str {
    match partition {
        crate::emerging_intelligence::EvaluationPartition::Development => "development",
        crate::emerging_intelligence::EvaluationPartition::WithinFamilyHoldout => {
            "within-family-holdout"
        }
        crate::emerging_intelligence::EvaluationPartition::RenamedVocabularyTransfer => {
            "renamed-vocabulary-transfer"
        }
        crate::emerging_intelligence::EvaluationPartition::StructuralTransfer => {
            "structural-transfer"
        }
        crate::emerging_intelligence::EvaluationPartition::Regression => "regression",
        crate::emerging_intelligence::EvaluationPartition::Adversarial => "adversarial",
    }
}

fn digest_serialized<T: Serialize>(domain: &[u8], value: &T) -> Result<UpdateDigest, UpdateError> {
    let payload = serde_json::to_vec(value).map_err(serialization_error)?;
    Ok(UpdateDigest(checksum128(domain, &payload)))
}

fn checksum128(domain: &[u8], payload: &[u8]) -> String {
    let left = fnv1a64(0xcbf29ce484222325, 0x4c, domain, payload);
    let right = fnv1a64(0x84222325cbf29ce4, 0x52, domain, payload);
    format!("{left:016x}{right:016x}")
}

fn fnv1a64(seed: u64, lane: u8, domain: &[u8], payload: &[u8]) -> u64 {
    let mut digest = seed;
    for byte in domain
        .iter()
        .copied()
        .chain([lane])
        .chain((payload.len() as u64).to_le_bytes())
        .chain(payload.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

fn serialization_error(error: serde_json::Error) -> UpdateError {
    UpdateError::Serialization(error.to_string())
}

fn deserialization_error(error: serde_json::Error) -> UpdateError {
    UpdateError::Deserialization(error.to_string())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UpdateError {
    #[error("unsupported EI-0D schema version {0}")]
    UnsupportedSchemaVersion(u16),
    #[error("invalid EI-0D identifier")]
    InvalidIdentifier,
    #[error("invalid EI-0D digest")]
    InvalidDigest,
    #[error("invalid EI-0D engine identifier")]
    InvalidEngineId,
    #[error("EI-0D policy slot {slot:?} value {value} is outside 0..=10000")]
    SlotValueOutOfRange { slot: PolicySlot, value: i32 },
    #[error("EI-0D state attempted to claim live authority")]
    UnauthorizedState,
    #[error("EI-0D proposal attempted to claim live authority")]
    UnauthorizedProposal,
    #[error("EI-0D transaction attempted to claim live authority")]
    UnauthorizedTransaction,
    #[error("proposal digest does not match canonical proposal payload")]
    ProposalDigestMismatch,
    #[error("transaction digest does not match canonical transaction payload")]
    TransactionDigestMismatch,
    #[error("rollback digest does not match canonical rollback payload")]
    RollbackDigestMismatch,
    #[error("transaction state digest does not match canonical state bytes")]
    TransactionStateDigestMismatch,
    #[error("transaction status is inconsistent with safety or final state")]
    InvalidTransactionStatus,
    #[error("proposal control arm does not match isolated state")]
    ArmMismatch,
    #[error("proposal namespace does not match isolated state")]
    NamespaceMismatch,
    #[error("proposal is bound to a stale pre-state digest")]
    StalePreState,
    #[error("proposal before value does not match isolated state")]
    BeforeValueMismatch,
    #[error("proposal ledger root does not match supplied canonical ledger")]
    LedgerRootMismatch,
    #[error("source episode is absent from supplied canonical ledger")]
    SourceEpisodeMissing,
    #[error("source episode digest does not match proposal provenance")]
    SourceEpisodeDigestMismatch,
    #[error("source episode is not independently evaluated")]
    SourceEpisodeNotEvaluated,
    #[error("source episode did not accept the proposal update identifier")]
    SourceUpdateNotAccepted,
    #[error("single update magnitude exceeds frozen budget")]
    SingleUpdateBudgetExceeded,
    #[error("cumulative update magnitude exceeds frozen budget")]
    CumulativeBudgetExceeded,
    #[error("duplicate update identifier {0}")]
    DuplicateUpdateId(String),
    #[error("frozen safety fixture set is incomplete")]
    IncompleteSafetyFixtureSet,
    #[error("frozen safety score overflow")]
    SafetyScoreOverflow,
    #[error("task fixture has no legal action")]
    EmptyTask,
    #[error("only an applied transaction can be explicitly rolled back")]
    TransactionNotApplied,
    #[error("transaction rollback does not match current isolated state")]
    RollbackStateMismatch,
    #[error("transaction has already been rolled back")]
    DuplicateRollback,
    #[error("valid JSON is not canonical byte encoding")]
    NonCanonicalEncoding,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
    #[error("sealed EI-0A episode failed validation: {0}")]
    EpisodeContract(#[from] EpisodeContractError),
    #[error("EI-0B environment contract failed: {0}")]
    Environment(#[from] EnvironmentError),
    #[error("EI-0C ledger contract failed: {0}")]
    Ledger(#[from] LedgerError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emerging_intelligence::{
        ActionId, BoundedAction, CognitiveEpisode, EpisodeEvaluation, EpisodeId,
        EpisodeProvenance, EvaluationId, EvidenceId, EvidenceRecord, EvidenceRef, Intention,
        LearningUpdate, LearningUpdateId, Observation, ObservationId, Outcome, OutcomeId,
        Prediction, PredictionAssessment, PredictionId, StrategyId, StrategySelection,
    };

    fn source_episode(update_id: &str) -> SealedCognitiveEpisode {
        let evidence_id = EvidenceId::new("evidence-001").unwrap();
        CognitiveEpisode {
            episode_id: EpisodeId::new(format!("episode-{update_id}")).unwrap(),
            phase: EpisodePhase::Evaluated,
            partition: crate::emerging_intelligence::EvaluationPartition::Development,
            task_family: "route-choice".into(),
            observation: Observation {
                observation_id: ObservationId::new(format!("observation-{update_id}")).unwrap(),
                kind: "route-state".into(),
                facts: vec!["edge-open".into()],
                observed_at_step: 1,
            },
            evidence: vec![EvidenceRecord {
                evidence_id: evidence_id.clone(),
                kind: "environment".into(),
                content_digest: "fixture:00000001".into(),
            }],
            predictions: vec![Prediction {
                prediction_id: PredictionId::new(format!("prediction-{update_id}")).unwrap(),
                proposition: "route-remains-open".into(),
                probability_bps: 8_000,
                evidence_refs: vec![EvidenceRef::new(evidence_id.clone())],
                created_at_step: 2,
            }],
            selected_strategy: Some(StrategySelection {
                strategy_id: StrategyId::new("bounded-search").unwrap(),
                rationale_evidence: vec![EvidenceRef::new(evidence_id.clone())],
                selected_at_step: 3,
            }),
            intention: Some(Intention {
                objective: "reach-goal".into(),
                declared_at_step: 3,
            }),
            action: Some(BoundedAction {
                action_id: ActionId::new(format!("action-{update_id}")).unwrap(),
                action: "choose-route".into(),
                declared_cost: 1,
                performed_at_step: 4,
            }),
            outcome: Some(Outcome {
                outcome_id: OutcomeId::new(format!("outcome-{update_id}")).unwrap(),
                action_id: ActionId::new(format!("action-{update_id}")).unwrap(),
                objective_satisfied: true,
                score_bps: 10_000,
                evidence_refs: vec![EvidenceRef::new(evidence_id)],
                observed_at_step: 5,
            }),
            evaluation: Some(EpisodeEvaluation {
                evaluation_id: EvaluationId::new(format!("evaluation-{update_id}")).unwrap(),
                outcome_id: OutcomeId::new(format!("outcome-{update_id}")).unwrap(),
                prediction_scores: vec![PredictionAssessment {
                    prediction_id: PredictionId::new(format!("prediction-{update_id}")).unwrap(),
                    score_bps: 9_000,
                }],
                action_score_bps: 10_000,
                evaluator_id: "independent-evaluator-v1".into(),
                evaluated_at_step: 6,
            }),
            proposed_updates: vec![LearningUpdate {
                update_id: LearningUpdateId::new(update_id).unwrap(),
                evaluation_id: EvaluationId::new(format!("evaluation-{update_id}")).unwrap(),
                proposal_digest: "proposal:00000001".into(),
                proposed_at_step: 7,
            }],
            accepted_updates: vec![LearningUpdateId::new(update_id).unwrap()],
            authority: AuthoritySnapshot::closed(),
            provenance: EpisodeProvenance {
                cohort_id: "ei-0d-tests".into(),
                fixture_digest: "fixture:00000001".into(),
                seed: 1,
                generator_version: "ei-0d-test-v1".into(),
                source_hashes: vec!["source:00000001".into()],
            },
        }
        .seal()
        .unwrap()
    }

    fn ledger_with(episode: &SealedCognitiveEpisode) -> AppendOnlyEpisodeLedger {
        let mut ledger = AppendOnlyEpisodeLedger::new().unwrap();
        ledger.append(episode).unwrap();
        ledger
    }

    #[test]
    fn valid_update_applies_and_rolls_back_exactly() {
        let update_id = "update-benign-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning").unwrap();
        let original_bytes = state.to_canonical_bytes().unwrap();
        let mut engine = ReversibleUpdateEngine::new(state).unwrap();
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            engine.state(),
            PolicySlot::VerifiedCueWeightBps,
            1_000,
        )
        .unwrap();
        let transaction = engine.apply(&proposal, &ledger).unwrap();
        assert_eq!(transaction.status, TransactionStatus::Applied);
        assert_ne!(engine.state().to_canonical_bytes().unwrap(), original_bytes);
        let receipt = engine.rollback(&transaction).unwrap();
        assert_eq!(engine.state().to_canonical_bytes().unwrap(), original_bytes);
        assert_eq!(receipt.restored_state_digest, transaction.pre_state_digest);
        assert_eq!(engine.rollback(&transaction), Err(UpdateError::DuplicateRollback));
    }

    #[test]
    fn harmful_update_is_detected_and_restored_before_commit() {
        let update_id = "update-harmful-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning").unwrap();
        let original = state.to_canonical_bytes().unwrap();
        let mut engine = ReversibleUpdateEngine::new(state).unwrap();
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            engine.state(),
            PolicySlot::RouteDecoyBiasBps,
            10_000,
        )
        .unwrap();
        let transaction = engine.apply(&proposal, &ledger).unwrap();
        assert_eq!(transaction.status, TransactionStatus::RolledBackHarmful);
        assert!(transaction.safety.harmful);
        assert_eq!(engine.state().to_canonical_bytes().unwrap(), original);
        assert_eq!(transaction.final_state_digest, transaction.pre_state_digest);
    }

    #[test]
    fn no_update_control_uses_same_interface_without_mutation() {
        let update_id = "update-control-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::NoUpdate, "ei-0d/no-update").unwrap();
        let original = state.to_canonical_bytes().unwrap();
        let mut engine = ReversibleUpdateEngine::new(state).unwrap();
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            engine.state(),
            PolicySlot::VerifiedCueWeightBps,
            1_000,
        )
        .unwrap();
        let transaction = engine.apply(&proposal, &ledger).unwrap();
        assert_eq!(transaction.status, TransactionStatus::ControlNoOp);
        assert_eq!(engine.state().to_canonical_bytes().unwrap(), original);
    }

    #[test]
    fn stale_and_cross_namespace_proposals_fail_closed() {
        let update_id = "update-stale-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning").unwrap();
        let mut proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            &state,
            PolicySlot::VerifiedCueWeightBps,
            500,
        )
        .unwrap();
        proposal.expected_pre_state_digest = "0".repeat(DIGEST_HEX_LEN);
        proposal.proposal_digest = proposal.compute_digest().unwrap();
        assert_eq!(
            IndependentAdmissibilityEvaluator::evaluate(&state, &proposal, &ledger),
            Err(UpdateError::StalePreState)
        );
        let mut proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            &state,
            PolicySlot::VerifiedCueWeightBps,
            500,
        )
        .unwrap();
        proposal.state_namespace = "ei-0d/other".into();
        proposal.proposal_digest = proposal.compute_digest().unwrap();
        assert_eq!(
            IndependentAdmissibilityEvaluator::evaluate(&state, &proposal, &ledger),
            Err(UpdateError::NamespaceMismatch)
        );
    }

    #[test]
    fn duplicate_update_and_budget_violation_fail_closed() {
        let update_id = "update-duplicate-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning").unwrap();
        let mut engine = ReversibleUpdateEngine::new(state).unwrap();
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            engine.state(),
            PolicySlot::VerifiedCueWeightBps,
            500,
        )
        .unwrap();
        engine.apply(&proposal, &ledger).unwrap();
        assert_eq!(
            engine.apply(&proposal, &ledger),
            Err(UpdateError::DuplicateUpdateId(update_id.into()))
        );

        let mut state =
            IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning-2").unwrap();
        state.cumulative_abs_delta_bps = MAX_CUMULATIVE_UPDATE_BPS;
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            &state,
            PolicySlot::VerifiedCueWeightBps,
            500,
        )
        .unwrap();
        assert_eq!(
            IndependentAdmissibilityEvaluator::evaluate(&state, &proposal, &ledger),
            Err(UpdateError::CumulativeBudgetExceeded)
        );
    }

    #[test]
    fn transaction_corruption_and_noncanonical_encoding_fail_closed() {
        let update_id = "update-corruption-001";
        let episode = source_episode(update_id);
        let ledger = ledger_with(&episode);
        let state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/learning").unwrap();
        let mut engine = ReversibleUpdateEngine::new(state).unwrap();
        let proposal = UpdateProposal::new(
            update_id,
            &episode,
            &ledger,
            engine.state(),
            PolicySlot::VerifiedCueWeightBps,
            500,
        )
        .unwrap();
        let transaction = engine.apply(&proposal, &ledger).unwrap();
        let canonical = transaction.to_canonical_bytes().unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        value["final_state_bytes"][0] = serde_json::Value::from(0);
        let corrupted = serde_json::to_vec(&value).unwrap();
        assert!(UpdateTransaction::from_canonical_bytes(&corrupted).is_err());
        let mut noncanonical = canonical;
        noncanonical.push(b'\n');
        assert_eq!(
            UpdateTransaction::from_canonical_bytes(&noncanonical),
            Err(UpdateError::NonCanonicalEncoding)
        );
    }
}
