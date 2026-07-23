//! EI-0G-S2 compile-only attachment and qualifying-collection freeze.
//!
//! This module describes a future one-way observation point after `/chat`
//! response bytes are finalized. It contains no runtime imports, network I/O,
//! persistence, file writes, task spawning, or attachment code. Collection is
//! deliberately disabled and cannot be authorized until every blocker is
//! resolved under a later, separately identified execution stage.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ATTACHMENT_SCHEMA_VERSION: u16 = 1;
pub const ATTACHMENT_PREREGISTRATION_ID: &str = "ei-0g-shadow-attachment-freeze-v1";
pub const PARENT_PREREGISTRATION_ID: &str = "ei-0g-shadow-prereg-v1";
pub const TARGET_RUNTIME_COMMIT: &str = "803c2ddbc1bd8029a2d7308ec973fa3a0a0ed848";
pub const TARGET_API_BLOB: &str = "665594e589a783f989a044a6cdf54f2f65a818b7";
pub const TARGET_DOCKERFILE_BLOB: &str = "21b04c1b03fc0c8e1e045597255901a7ac705725";
pub const TARGET_LOCKFILE_BLOB: &str = "031183ea5049f92380c4c39780848b83ebf957b6";
pub const TARGET_RENDER_BLOB: &str = "082e7cb4aa7ec26263dd1ddfcfff8422913477c7";
pub const ASSIGNMENT_SEED: u64 = 0x4549_3047_5332_0001;
pub const PARTITION_SEED: u64 = 0x4549_3047_5332_0002;
pub const CONFIGURED_RATE_BPS: u16 = 100;
pub const CONFIGURED_SAMPLE_CAP: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverArm {
    NoObserver,
    InertObserver,
    EiObserver,
}

impl ObserverArm {
    pub const ALL: [Self; 3] = [Self::NoObserver, Self::InertObserver, Self::EiObserver];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoObserver => "no_observer",
            Self::InertObserver => "inert_observer",
            Self::EiObserver => "ei_observer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrozenPartition {
    Development,
    Calibration,
    HeldOut,
    Adversarial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentMode {
    ExplicitOperatorOptIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficClass {
    SuccessfulChat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSinkMode {
    DisabledEncryptedAppendOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentPoint {
    pub source_path: String,
    pub source_blob: String,
    pub function: String,
    pub anchor: String,
    pub response_finalized: bool,
    pub runtime_lock_released: bool,
    pub existing_shadow_dispatch_completed: bool,
    pub response_mutation_possible: bool,
    pub network_send_started: bool,
    pub one_way_copy_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentIdentity {
    pub target_runtime_commit: String,
    pub api_blob: String,
    pub dockerfile_blob: String,
    pub lockfile_blob: String,
    pub render_blueprint_blob: String,
    pub expected_recipe_digest: String,
    pub actual_container_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentPrivacyPolicy {
    pub consent_mode: ConsentMode,
    pub consent_required: bool,
    pub allowed_traffic: Vec<TrafficClass>,
    pub public_collection_allowed: bool,
    pub operator_only_design: bool,
    pub raw_request_retained: bool,
    pub raw_response_retained: bool,
    pub raw_identity_retained: bool,
    pub raw_session_retained: bool,
    pub conversation_history_retained: bool,
    pub credentials_retained: bool,
    pub memory_contents_retained: bool,
    pub tool_outputs_retained: bool,
    pub excluded_classes: Vec<String>,
    pub maximum_raw_request_bytes_before_redaction: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplingPlan {
    pub collection_enabled: bool,
    pub configured_rate_bps: u16,
    pub active_rate_bps: u16,
    pub qualifying_sample_cap: u32,
    pub active_sample_cap: u32,
    pub qualifying_samples_collected: u32,
    pub assignment_seed: u64,
    pub partition_seed: u64,
    pub arms: Vec<ObserverArm>,
    pub arms_per_sample: u16,
    pub identical_authorized_input: bool,
    pub identical_budgets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_authorized_input_bytes: u32,
    pub max_records_per_request: u16,
    pub max_added_latency_micros: u32,
    pub max_cpu_micros: u32,
    pub max_ephemeral_memory_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenThresholds {
    pub max_response_byte_divergence: u32,
    pub max_route_divergence: u16,
    pub max_tool_divergence: u16,
    pub max_action_divergence: u16,
    pub max_persistence_divergence: u16,
    pub max_missing_records: u16,
    pub max_corrupt_records: u16,
    pub max_cross_user_leakage_events: u16,
    pub max_replay_mismatches: u16,
    pub max_calibration_error_bps: u16,
    pub min_observer_advantage_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSinkPlan {
    pub mode: EvidenceSinkMode,
    pub write_enabled: bool,
    pub append_only_required: bool,
    pub encryption_required: bool,
    pub operator_only_access: bool,
    pub raw_content_allowed: bool,
    pub retention_days: u16,
    pub deletion_receipt_required: bool,
    pub sink_identifier_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSwitchPlan {
    pub sampling_check_precedes_redaction: bool,
    pub sampling_check_precedes_adapter: bool,
    pub disabled_value: bool,
    pub default_enabled: bool,
    pub environment_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalPlan {
    pub disable_sampling: bool,
    pub detach_observer: bool,
    pub delete_ephemeral_namespaces: bool,
    pub close_evidence_sink: bool,
    pub delete_retained_evidence: bool,
    pub emit_deletion_receipt: bool,
    pub verify_live_state_untouched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityBoundary {
    pub runtime_wiring: bool,
    pub live_learning: bool,
    pub response: bool,
    pub persistence: bool,
    pub routing: bool,
    pub belief: bool,
    pub ontology: bool,
    pub tool: bool,
    pub network: bool,
    pub action: bool,
}

impl AuthorityBoundary {
    #[must_use]
    pub const fn closed() -> Self {
        Self {
            runtime_wiring: false,
            live_learning: false,
            response: false,
            persistence: false,
            routing: false,
            belief: false,
            ontology: false,
            tool: false,
            network: false,
            action: false,
        }
    }

    #[must_use]
    pub const fn is_closed(&self) -> bool {
        !self.runtime_wiring
            && !self.live_learning
            && !self.response
            && !self.persistence
            && !self.routing
            && !self.belief
            && !self.ontology
            && !self.tool
            && !self.network
            && !self.action
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentFreeze {
    pub schema_version: u16,
    pub preregistration_id: String,
    pub parent_preregistration_id: String,
    pub attachment: AttachmentPoint,
    pub deployment: DeploymentIdentity,
    pub privacy: ConsentPrivacyPolicy,
    pub sampling: SamplingPlan,
    pub budgets: ResourceBudget,
    pub thresholds: FrozenThresholds,
    pub evidence_sink: EvidenceSinkPlan,
    pub kill_switch: KillSwitchPlan,
    pub removal: RemovalPlan,
    pub security_prerequisite_satisfied: bool,
    pub live_runtime_attachment_authorized: bool,
    pub authority: AuthorityBoundary,
}

impl AttachmentFreeze {
    #[must_use]
    pub fn frozen() -> Self {
        let deployment = DeploymentIdentity {
            target_runtime_commit: TARGET_RUNTIME_COMMIT.to_owned(),
            api_blob: TARGET_API_BLOB.to_owned(),
            dockerfile_blob: TARGET_DOCKERFILE_BLOB.to_owned(),
            lockfile_blob: TARGET_LOCKFILE_BLOB.to_owned(),
            render_blueprint_blob: TARGET_RENDER_BLOB.to_owned(),
            expected_recipe_digest: recipe_digest(),
            actual_container_digest: None,
        };

        Self {
            schema_version: ATTACHMENT_SCHEMA_VERSION,
            preregistration_id: ATTACHMENT_PREREGISTRATION_ID.to_owned(),
            parent_preregistration_id: PARENT_PREREGISTRATION_ID.to_owned(),
            attachment: AttachmentPoint {
                source_path: "lib/api.rs".to_owned(),
                source_blob: TARGET_API_BLOB.to_owned(),
                function: "handle_chat".to_owned(),
                anchor: "success:after-response-json-and-existing-shadow-dispatch-before-return"
                    .to_owned(),
                response_finalized: true,
                runtime_lock_released: true,
                existing_shadow_dispatch_completed: true,
                response_mutation_possible: false,
                network_send_started: false,
                one_way_copy_only: true,
            },
            deployment,
            privacy: ConsentPrivacyPolicy {
                consent_mode: ConsentMode::ExplicitOperatorOptIn,
                consent_required: true,
                allowed_traffic: vec![TrafficClass::SuccessfulChat],
                public_collection_allowed: false,
                operator_only_design: true,
                raw_request_retained: false,
                raw_response_retained: false,
                raw_identity_retained: false,
                raw_session_retained: false,
                conversation_history_retained: false,
                credentials_retained: false,
                memory_contents_retained: false,
                tool_outputs_retained: false,
                excluded_classes: vec![
                    "errors".to_owned(),
                    "telegram".to_owned(),
                    "reason".to_owned(),
                    "remember".to_owned(),
                    "identity".to_owned(),
                    "memory".to_owned(),
                    "metacognition".to_owned(),
                    "thought".to_owned(),
                    "credentials-or-secrets".to_owned(),
                    "suspected-minor-data".to_owned(),
                    "health-or-crisis-content".to_owned(),
                    "legal-or-financial-sensitive-content".to_owned(),
                ],
                maximum_raw_request_bytes_before_redaction: 32_768,
            },
            sampling: SamplingPlan {
                collection_enabled: false,
                configured_rate_bps: CONFIGURED_RATE_BPS,
                active_rate_bps: 0,
                qualifying_sample_cap: CONFIGURED_SAMPLE_CAP,
                active_sample_cap: 0,
                qualifying_samples_collected: 0,
                assignment_seed: ASSIGNMENT_SEED,
                partition_seed: PARTITION_SEED,
                arms: ObserverArm::ALL.to_vec(),
                arms_per_sample: 3,
                identical_authorized_input: true,
                identical_budgets: true,
            },
            budgets: ResourceBudget {
                max_authorized_input_bytes: 1_024,
                max_records_per_request: 1,
                max_added_latency_micros: 10_000,
                max_cpu_micros: 5_000,
                max_ephemeral_memory_bytes: 1_048_576,
            },
            thresholds: FrozenThresholds {
                max_response_byte_divergence: 0,
                max_route_divergence: 0,
                max_tool_divergence: 0,
                max_action_divergence: 0,
                max_persistence_divergence: 0,
                max_missing_records: 0,
                max_corrupt_records: 0,
                max_cross_user_leakage_events: 0,
                max_replay_mismatches: 0,
                max_calibration_error_bps: 2_500,
                min_observer_advantage_bps: 250,
            },
            evidence_sink: EvidenceSinkPlan {
                mode: EvidenceSinkMode::DisabledEncryptedAppendOnly,
                write_enabled: false,
                append_only_required: true,
                encryption_required: true,
                operator_only_access: true,
                raw_content_allowed: false,
                retention_days: 7,
                deletion_receipt_required: true,
                sink_identifier_digest: domain_digest(
                    "sink",
                    b"ei-0g-s2-disabled-operator-evidence-sink",
                ),
            },
            kill_switch: KillSwitchPlan {
                sampling_check_precedes_redaction: true,
                sampling_check_precedes_adapter: true,
                disabled_value: false,
                default_enabled: false,
                environment_key: "STARFIRE_EI0G_SHADOW_ENABLED".to_owned(),
            },
            removal: RemovalPlan {
                disable_sampling: true,
                detach_observer: true,
                delete_ephemeral_namespaces: true,
                close_evidence_sink: true,
                delete_retained_evidence: true,
                emit_deletion_receipt: true,
                verify_live_state_untouched: true,
            },
            security_prerequisite_satisfied: false,
            live_runtime_attachment_authorized: false,
            authority: AuthorityBoundary::closed(),
        }
    }

    pub fn validate(&self) -> Result<(), AttachmentFreezeError> {
        if self.schema_version != ATTACHMENT_SCHEMA_VERSION {
            return Err(AttachmentFreezeError::UnsupportedSchema(
                self.schema_version,
            ));
        }
        if self.preregistration_id != ATTACHMENT_PREREGISTRATION_ID
            || self.parent_preregistration_id != PARENT_PREREGISTRATION_ID
        {
            return Err(AttachmentFreezeError::IdentityMismatch);
        }
        if !self.attachment.response_finalized
            || !self.attachment.runtime_lock_released
            || !self.attachment.existing_shadow_dispatch_completed
            || self.attachment.response_mutation_possible
            || self.attachment.network_send_started
            || !self.attachment.one_way_copy_only
        {
            return Err(AttachmentFreezeError::InvalidAttachmentPoint);
        }
        if self.deployment.target_runtime_commit != TARGET_RUNTIME_COMMIT
            || self.deployment.api_blob != TARGET_API_BLOB
            || self.deployment.dockerfile_blob != TARGET_DOCKERFILE_BLOB
            || self.deployment.lockfile_blob != TARGET_LOCKFILE_BLOB
            || self.deployment.render_blueprint_blob != TARGET_RENDER_BLOB
            || self.deployment.expected_recipe_digest != recipe_digest()
        {
            return Err(AttachmentFreezeError::DeploymentIdentityMismatch);
        }
        if self.deployment.actual_container_digest.is_some() {
            return Err(AttachmentFreezeError::ActualContainerMustRemainUnbound);
        }
        if self.sampling.collection_enabled
            || self.sampling.active_rate_bps != 0
            || self.sampling.active_sample_cap != 0
            || self.sampling.qualifying_samples_collected != 0
        {
            return Err(AttachmentFreezeError::CollectionMustRemainDisabled);
        }
        if self.sampling.configured_rate_bps != CONFIGURED_RATE_BPS
            || self.sampling.qualifying_sample_cap != CONFIGURED_SAMPLE_CAP
            || self.sampling.arms != ObserverArm::ALL
            || self.sampling.arms_per_sample != 3
            || !self.sampling.identical_authorized_input
            || !self.sampling.identical_budgets
        {
            return Err(AttachmentFreezeError::SamplingPlanMismatch);
        }
        if !privacy_complete(&self.privacy) {
            return Err(AttachmentFreezeError::PrivacyPolicyIncomplete);
        }
        if self.evidence_sink.write_enabled
            || !self.evidence_sink.append_only_required
            || !self.evidence_sink.encryption_required
            || !self.evidence_sink.operator_only_access
            || self.evidence_sink.raw_content_allowed
            || !self.evidence_sink.deletion_receipt_required
        {
            return Err(AttachmentFreezeError::EvidenceSinkNotFrozen);
        }
        if !self.kill_switch.sampling_check_precedes_redaction
            || !self.kill_switch.sampling_check_precedes_adapter
            || self.kill_switch.default_enabled
            || self.kill_switch.disabled_value
        {
            return Err(AttachmentFreezeError::KillSwitchInvalid);
        }
        if !removal_complete(&self.removal) {
            return Err(AttachmentFreezeError::RemovalIncomplete);
        }
        if self.security_prerequisite_satisfied
            || self.live_runtime_attachment_authorized
            || !self.authority.is_closed()
        {
            return Err(AttachmentFreezeError::UnauthorizedAuthority);
        }
        if !thresholds_match(&self.thresholds) || !budgets_match(&self.budgets) {
            return Err(AttachmentFreezeError::InheritedLimitsChanged);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AttachmentFreezeError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(AttachmentFreezeError::Serialization)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionAuthorization {
    pub collection_enabled: bool,
    pub security_prerequisite_satisfied: bool,
    pub actual_container_digest: Option<String>,
    pub explicit_execution_issue: Option<u32>,
    pub explicit_operator_consent: bool,
}

impl ExecutionAuthorization {
    pub fn validate(&self) -> Result<(), AttachmentFreezeError> {
        if !self.collection_enabled
            || !self.security_prerequisite_satisfied
            || self.actual_container_digest.is_none()
            || self.explicit_execution_issue.is_none()
            || !self.explicit_operator_consent
        {
            return Err(AttachmentFreezeError::ExecutionUnauthorized);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentFreezeReport {
    pub preregistration_id: String,
    pub classification: String,
    pub deterministic_replay: bool,
    pub collection_enabled: bool,
    pub qualifying_samples_collected: u32,
    pub security_prerequisite_satisfied: bool,
    pub actual_container_digest_bound: bool,
    pub attachment_one_way: bool,
    pub privacy_policy_complete: bool,
    pub matched_controls_frozen: bool,
    pub evidence_sink_write_enabled: bool,
    pub kill_switch_precedes_adapter: bool,
    pub complete_removal_verified: bool,
    pub configured_rate_bps: u16,
    pub active_rate_bps: u16,
    pub configured_sample_cap: u32,
    pub active_sample_cap: u32,
    pub expected_recipe_digest: String,
    pub runtime_wiring: bool,
    pub live_learning_authority: bool,
    pub response_authority: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub belief_authority: bool,
    pub ontology_authority: bool,
    pub tool_authority: bool,
    pub network_authority: bool,
    pub action_authority: bool,
}

pub fn run_attachment_freeze_probe() -> Result<AttachmentFreezeReport, AttachmentFreezeError> {
    let freeze = AttachmentFreeze::frozen();
    freeze.validate()?;
    let first = freeze.to_canonical_bytes()?;
    let second = AttachmentFreeze::frozen().to_canonical_bytes()?;
    let deterministic_replay = first == second;

    let authorization = ExecutionAuthorization {
        collection_enabled: false,
        security_prerequisite_satisfied: false,
        actual_container_digest: None,
        explicit_execution_issue: None,
        explicit_operator_consent: false,
    };
    if authorization.validate().is_ok() {
        return Err(AttachmentFreezeError::ExecutionUnexpectedlyAuthorized);
    }

    let attachment_one_way = freeze.attachment.response_finalized
        && freeze.attachment.runtime_lock_released
        && freeze.attachment.existing_shadow_dispatch_completed
        && !freeze.attachment.response_mutation_possible
        && !freeze.attachment.network_send_started
        && freeze.attachment.one_way_copy_only;
    let privacy_policy_complete = privacy_complete(&freeze.privacy);
    let matched_controls_frozen = freeze.sampling.arms == ObserverArm::ALL
        && freeze.sampling.arms_per_sample == 3
        && freeze.sampling.identical_authorized_input
        && freeze.sampling.identical_budgets;
    let kill_switch_precedes_adapter = freeze.kill_switch.sampling_check_precedes_redaction
        && freeze.kill_switch.sampling_check_precedes_adapter
        && !freeze.kill_switch.default_enabled;
    let complete_removal_verified = removal_complete(&freeze.removal);

    let passed = deterministic_replay
        && attachment_one_way
        && privacy_policy_complete
        && matched_controls_frozen
        && !freeze.evidence_sink.write_enabled
        && kill_switch_precedes_adapter
        && complete_removal_verified
        && !freeze.sampling.collection_enabled
        && freeze.sampling.qualifying_samples_collected == 0
        && !freeze.security_prerequisite_satisfied
        && freeze.deployment.actual_container_digest.is_none()
        && freeze.authority.is_closed();

    Ok(AttachmentFreezeReport {
        preregistration_id: freeze.preregistration_id,
        classification: if passed { "FREEZE_PASS" } else { "FREEZE_FAIL" }.to_owned(),
        deterministic_replay,
        collection_enabled: freeze.sampling.collection_enabled,
        qualifying_samples_collected: freeze.sampling.qualifying_samples_collected,
        security_prerequisite_satisfied: freeze.security_prerequisite_satisfied,
        actual_container_digest_bound: freeze.deployment.actual_container_digest.is_some(),
        attachment_one_way,
        privacy_policy_complete,
        matched_controls_frozen,
        evidence_sink_write_enabled: freeze.evidence_sink.write_enabled,
        kill_switch_precedes_adapter,
        complete_removal_verified,
        configured_rate_bps: freeze.sampling.configured_rate_bps,
        active_rate_bps: freeze.sampling.active_rate_bps,
        configured_sample_cap: freeze.sampling.qualifying_sample_cap,
        active_sample_cap: freeze.sampling.active_sample_cap,
        expected_recipe_digest: freeze.deployment.expected_recipe_digest,
        runtime_wiring: freeze.authority.runtime_wiring,
        live_learning_authority: freeze.authority.live_learning,
        response_authority: freeze.authority.response,
        persistence_authority: freeze.authority.persistence,
        routing_authority: freeze.authority.routing,
        belief_authority: freeze.authority.belief,
        ontology_authority: freeze.authority.ontology,
        tool_authority: freeze.authority.tool,
        network_authority: freeze.authority.network,
        action_authority: freeze.authority.action,
    })
}

fn privacy_complete(policy: &ConsentPrivacyPolicy) -> bool {
    policy.consent_required
        && !policy.public_collection_allowed
        && policy.operator_only_design
        && policy.allowed_traffic == [TrafficClass::SuccessfulChat]
        && !policy.raw_request_retained
        && !policy.raw_response_retained
        && !policy.raw_identity_retained
        && !policy.raw_session_retained
        && !policy.conversation_history_retained
        && !policy.credentials_retained
        && !policy.memory_contents_retained
        && !policy.tool_outputs_retained
        && !policy.excluded_classes.is_empty()
}

fn removal_complete(plan: &RemovalPlan) -> bool {
    plan.disable_sampling
        && plan.detach_observer
        && plan.delete_ephemeral_namespaces
        && plan.close_evidence_sink
        && plan.delete_retained_evidence
        && plan.emit_deletion_receipt
        && plan.verify_live_state_untouched
}

fn budgets_match(budget: &ResourceBudget) -> bool {
    budget.max_authorized_input_bytes == 1_024
        && budget.max_records_per_request == 1
        && budget.max_added_latency_micros == 10_000
        && budget.max_cpu_micros == 5_000
        && budget.max_ephemeral_memory_bytes == 1_048_576
}

fn thresholds_match(thresholds: &FrozenThresholds) -> bool {
    thresholds.max_response_byte_divergence == 0
        && thresholds.max_route_divergence == 0
        && thresholds.max_tool_divergence == 0
        && thresholds.max_action_divergence == 0
        && thresholds.max_persistence_divergence == 0
        && thresholds.max_missing_records == 0
        && thresholds.max_corrupt_records == 0
        && thresholds.max_cross_user_leakage_events == 0
        && thresholds.max_replay_mismatches == 0
        && thresholds.max_calibration_error_bps == 2_500
        && thresholds.min_observer_advantage_bps == 250
}

fn recipe_digest() -> String {
    domain_digest(
        "deployment-recipe",
        format!(
            "{TARGET_RUNTIME_COMMIT}:{TARGET_API_BLOB}:{TARGET_DOCKERFILE_BLOB}:{TARGET_LOCKFILE_BLOB}:{TARGET_RENDER_BLOB}"
        )
        .as_bytes(),
    )
}

fn domain_digest(domain: &str, bytes: &[u8]) -> String {
    let mut framed = Vec::with_capacity(domain.len() + bytes.len() + 1);
    framed.extend_from_slice(domain.as_bytes());
    framed.push(0);
    framed.extend_from_slice(bytes);
    format!("{:016x}", fnv1a64(&framed))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Error)]
pub enum AttachmentFreezeError {
    #[error("unsupported attachment schema version {0}")]
    UnsupportedSchema(u16),
    #[error("preregistration identity mismatch")]
    IdentityMismatch,
    #[error("attachment point is not strictly finalized and one-way")]
    InvalidAttachmentPoint,
    #[error("deployment identity mismatch")]
    DeploymentIdentityMismatch,
    #[error("actual container digest must remain unbound during freeze")]
    ActualContainerMustRemainUnbound,
    #[error("collection must remain disabled with zero active cap and samples")]
    CollectionMustRemainDisabled,
    #[error("sampling plan does not match frozen controls")]
    SamplingPlanMismatch,
    #[error("privacy policy is incomplete")]
    PrivacyPolicyIncomplete,
    #[error("evidence sink is not safely frozen")]
    EvidenceSinkNotFrozen,
    #[error("kill switch does not precede the observer")]
    KillSwitchInvalid,
    #[error("removal procedure is incomplete")]
    RemovalIncomplete,
    #[error("authority is not closed")]
    UnauthorizedAuthority,
    #[error("inherited EI-0G limits changed")]
    InheritedLimitsChanged,
    #[error("execution authorization is incomplete")]
    ExecutionUnauthorized,
    #[error("disabled execution was unexpectedly authorized")]
    ExecutionUnexpectedlyAuthorized,
    #[error("serialization error: {0}")]
    Serialization(serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_attachment_is_valid_and_collection_is_zero() {
        let freeze = AttachmentFreeze::frozen();
        freeze.validate().expect("valid freeze");
        assert!(!freeze.sampling.collection_enabled);
        assert_eq!(freeze.sampling.active_rate_bps, 0);
        assert_eq!(freeze.sampling.active_sample_cap, 0);
        assert_eq!(freeze.sampling.qualifying_samples_collected, 0);
        assert!(freeze.deployment.actual_container_digest.is_none());
        assert!(!freeze.live_runtime_attachment_authorized);
        assert!(!freeze.security_prerequisite_satisfied);
        assert!(freeze.authority.is_closed());
    }

    #[test]
    fn attachment_point_is_after_finalization_and_before_send() {
        let point = AttachmentFreeze::frozen().attachment;
        assert!(point.response_finalized);
        assert!(point.runtime_lock_released);
        assert!(point.existing_shadow_dispatch_completed);
        assert!(!point.response_mutation_possible);
        assert!(!point.network_send_started);
        assert!(point.one_way_copy_only);
    }

    #[test]
    fn privacy_policy_retains_no_raw_content() {
        let policy = AttachmentFreeze::frozen().privacy;
        assert!(privacy_complete(&policy));
        assert!(!policy.raw_request_retained);
        assert!(!policy.raw_response_retained);
        assert!(!policy.raw_identity_retained);
        assert!(!policy.raw_session_retained);
        assert!(!policy.conversation_history_retained);
        assert!(!policy.credentials_retained);
        assert!(!policy.memory_contents_retained);
        assert!(!policy.tool_outputs_retained);
    }

    #[test]
    fn disabled_execution_cannot_be_authorized() {
        let authorization = ExecutionAuthorization {
            collection_enabled: false,
            security_prerequisite_satisfied: false,
            actual_container_digest: None,
            explicit_execution_issue: None,
            explicit_operator_consent: false,
        };
        assert!(matches!(
            authorization.validate(),
            Err(AttachmentFreezeError::ExecutionUnauthorized)
        ));
    }

    #[test]
    fn enabling_collection_invalidates_the_freeze() {
        let mut freeze = AttachmentFreeze::frozen();
        freeze.sampling.collection_enabled = true;
        freeze.sampling.active_rate_bps = CONFIGURED_RATE_BPS;
        assert!(matches!(
            freeze.validate(),
            Err(AttachmentFreezeError::CollectionMustRemainDisabled)
        ));
    }

    #[test]
    fn binding_an_actual_container_invalidates_the_freeze() {
        let mut freeze = AttachmentFreeze::frozen();
        freeze.deployment.actual_container_digest = Some("sha256:future".to_owned());
        assert!(matches!(
            freeze.validate(),
            Err(AttachmentFreezeError::ActualContainerMustRemainUnbound)
        ));
    }

    #[test]
    fn evidence_sink_is_non_writing_and_removal_is_complete() {
        let freeze = AttachmentFreeze::frozen();
        assert!(!freeze.evidence_sink.write_enabled);
        assert!(freeze.evidence_sink.encryption_required);
        assert!(!freeze.evidence_sink.raw_content_allowed);
        assert!(removal_complete(&freeze.removal));
    }

    #[test]
    fn freeze_replay_is_byte_identical() {
        let first = AttachmentFreeze::frozen()
            .to_canonical_bytes()
            .expect("first");
        let second = AttachmentFreeze::frozen()
            .to_canonical_bytes()
            .expect("second");
        assert_eq!(first, second);
    }

    #[test]
    fn report_passes_without_authorizing_attachment() {
        let report = run_attachment_freeze_probe().expect("report");
        assert_eq!(report.classification, "FREEZE_PASS");
        assert!(!report.collection_enabled);
        assert_eq!(report.qualifying_samples_collected, 0);
        assert!(!report.actual_container_digest_bound);
        assert!(!report.security_prerequisite_satisfied);
        assert!(!report.runtime_wiring);
        assert!(!report.response_authority);
        assert!(!report.persistence_authority);
        assert!(!report.routing_authority);
        assert!(!report.tool_authority);
        assert!(!report.network_authority);
        assert!(!report.action_authority);
    }
}
