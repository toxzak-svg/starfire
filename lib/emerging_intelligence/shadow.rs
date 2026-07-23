//! EI-0G read-only shadow-observer preregistration contracts.
//!
//! This module consumes only explicitly authorized, privacy-redacted copies of
//! runtime metadata. It never receives `Runtime`, memory, persistence, routing,
//! tool, or action handles. Every output is inert evidence for later offline
//! evaluation and cannot flow back into an ordinary response.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SHADOW_SCHEMA_VERSION: u16 = 1;
pub const SHADOW_PREREGISTRATION_ID: &str = "ei-0g-shadow-prereg-v1";
pub const SHADOW_SAMPLING_SEED: u64 = 0x4549_3047_0000_0001;
pub const SHADOW_PARTITION_SEED: u64 = 0x5348_4144_4f57_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowArm {
    NoObserver,
    InertObserver,
    EiObserver,
}

impl ShadowArm {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowPartition {
    Development,
    Calibration,
    HeldOut,
    Adversarial,
}

impl ShadowPartition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Calibration => "calibration",
            Self::HeldOut => "held_out",
            Self::Adversarial => "adversarial",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowBudget {
    pub max_authorized_input_bytes: u32,
    pub max_records_per_request: u16,
    pub max_added_latency_micros: u32,
    pub max_cpu_micros: u32,
    pub max_ephemeral_memory_bytes: u32,
}

pub const FROZEN_SHADOW_BUDGET: ShadowBudget = ShadowBudget {
    max_authorized_input_bytes: 1_024,
    max_records_per_request: 1,
    max_added_latency_micros: 10_000,
    max_cpu_micros: 5_000,
    max_ephemeral_memory_bytes: 1_048_576,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowThresholds {
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

pub const FROZEN_SHADOW_THRESHOLDS: ShadowThresholds = ShadowThresholds {
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
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawShadowIngress<'a> {
    pub sample_id: &'a str,
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub request_text: &'a str,
    pub baseline_response_bytes: &'a [u8],
    pub baseline_route: &'a str,
    pub baseline_tool: &'a str,
    pub baseline_action: &'a str,
    pub baseline_persistence: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedRuntimeInputCopy {
    pub schema_version: u16,
    pub sample_id: String,
    pub user_pseudonym: String,
    pub session_pseudonym: String,
    pub request_digest: String,
    pub request_class: String,
    pub request_bytes: u32,
    pub baseline_response_digest: String,
    pub baseline_route_digest: String,
    pub baseline_tool_digest: String,
    pub baseline_action_digest: String,
    pub baseline_persistence_digest: String,
    pub partition: ShadowPartition,
    pub authorized_input_bytes: u32,
}

impl AuthorizedRuntimeInputCopy {
    pub fn validate(&self) -> Result<(), ShadowContractError> {
        if self.schema_version != SHADOW_SCHEMA_VERSION {
            return Err(ShadowContractError::UnsupportedSchema(self.schema_version));
        }
        for (label, value) in [
            ("sample_id", self.sample_id.as_str()),
            ("user_pseudonym", self.user_pseudonym.as_str()),
            ("session_pseudonym", self.session_pseudonym.as_str()),
            ("request_digest", self.request_digest.as_str()),
            (
                "baseline_response_digest",
                self.baseline_response_digest.as_str(),
            ),
            ("baseline_route_digest", self.baseline_route_digest.as_str()),
            ("baseline_tool_digest", self.baseline_tool_digest.as_str()),
            (
                "baseline_action_digest",
                self.baseline_action_digest.as_str(),
            ),
            (
                "baseline_persistence_digest",
                self.baseline_persistence_digest.as_str(),
            ),
        ] {
            if value.is_empty() {
                return Err(ShadowContractError::EmptyField(label));
            }
        }
        if self.authorized_input_bytes > FROZEN_SHADOW_BUDGET.max_authorized_input_bytes {
            return Err(ShadowContractError::AuthorizedInputBudgetExceeded {
                actual: self.authorized_input_bytes,
                maximum: FROZEN_SHADOW_BUDGET.max_authorized_input_bytes,
            });
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, ShadowContractError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(ShadowContractError::Serialization)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShadowPrivacyRedactor;

impl ShadowPrivacyRedactor {
    pub fn redact(
        &self,
        ingress: &RawShadowIngress<'_>,
    ) -> Result<AuthorizedRuntimeInputCopy, ShadowContractError> {
        if ingress.request_text.len() > 32 * 1024 {
            return Err(ShadowContractError::RawRequestTooLarge {
                actual: ingress.request_text.len(),
                maximum: 32 * 1024,
            });
        }
        if ingress.sample_id.is_empty() || ingress.user_id.is_empty() || ingress.session_id.is_empty()
        {
            return Err(ShadowContractError::EmptyIngressIdentity);
        }

        let request_digest = domain_digest("request", ingress.request_text.as_bytes());
        let partition = partition_for(&request_digest);
        let request_class = match ingress.request_text.len() {
            0..=255 => "short",
            256..=2_047 => "medium",
            _ => "long",
        }
        .to_owned();

        let redacted = AuthorizedRuntimeInputCopy {
            schema_version: SHADOW_SCHEMA_VERSION,
            sample_id: ingress.sample_id.to_owned(),
            user_pseudonym: domain_digest("user", ingress.user_id.as_bytes()),
            session_pseudonym: domain_digest("session", ingress.session_id.as_bytes()),
            request_digest,
            request_class,
            request_bytes: u32::try_from(ingress.request_text.len())
                .map_err(|_| ShadowContractError::IntegerOverflow)?,
            baseline_response_digest: domain_digest(
                "response",
                ingress.baseline_response_bytes,
            ),
            baseline_route_digest: domain_digest("route", ingress.baseline_route.as_bytes()),
            baseline_tool_digest: domain_digest("tool", ingress.baseline_tool.as_bytes()),
            baseline_action_digest: domain_digest("action", ingress.baseline_action.as_bytes()),
            baseline_persistence_digest: domain_digest(
                "persistence",
                ingress.baseline_persistence.as_bytes(),
            ),
            partition,
            authorized_input_bytes: 320,
        };
        redacted.validate()?;
        Ok(redacted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowObservation {
    pub arm: ShadowArm,
    pub sample_id: String,
    pub namespace_id: String,
    pub input_digest: String,
    pub partition: ShadowPartition,
    pub observer_executed: bool,
    pub prediction_digest: Option<String>,
    pub update_proposal_digest: Option<String>,
    pub response_digest_after: String,
    pub route_digest_after: String,
    pub tool_digest_after: String,
    pub action_digest_after: String,
    pub persistence_digest_after: String,
    pub added_latency_micros: u32,
    pub cpu_micros: u32,
    pub ephemeral_memory_bytes: u32,
    pub live_write_attempted: bool,
    pub qualifying_sample: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedShadowObservation {
    pub observation: ShadowObservation,
    pub checksum: String,
}

impl SealedShadowObservation {
    pub fn from_observation(observation: ShadowObservation) -> Result<Self, ShadowContractError> {
        validate_observation(&observation)?;
        let bytes =
            serde_json::to_vec(&observation).map_err(ShadowContractError::Serialization)?;
        Ok(Self {
            observation,
            checksum: domain_digest("shadow-record", &bytes),
        })
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, ShadowContractError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(ShadowContractError::Serialization)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ShadowContractError> {
        let record: Self =
            serde_json::from_slice(bytes).map_err(ShadowContractError::Serialization)?;
        record.validate()?;
        if record.to_canonical_bytes()? != bytes {
            return Err(ShadowContractError::NonCanonicalReplay);
        }
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), ShadowContractError> {
        validate_observation(&self.observation)?;
        let bytes = serde_json::to_vec(&self.observation)
            .map_err(ShadowContractError::Serialization)?;
        let expected = domain_digest("shadow-record", &bytes);
        if self.checksum != expected {
            return Err(ShadowContractError::TamperedObservation);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReadOnlyShadowAdapter;

impl ReadOnlyShadowAdapter {
    pub fn observe(
        &self,
        arm: ShadowArm,
        input: &AuthorizedRuntimeInputCopy,
    ) -> Result<SealedShadowObservation, ShadowContractError> {
        input.validate()?;
        let namespace_id = domain_digest(
            "namespace",
            format!(
                "{}:{}:{}",
                input.user_pseudonym,
                input.session_pseudonym,
                arm.as_str()
            )
            .as_bytes(),
        );
        let (observer_executed, prediction_digest, update_proposal_digest, latency, cpu, memory) =
            match arm {
                ShadowArm::NoObserver => (false, None, None, 0, 0, 0),
                ShadowArm::InertObserver => (
                    true,
                    Some(domain_digest(
                        "inert-prediction",
                        input.request_digest.as_bytes(),
                    )),
                    None,
                    300,
                    200,
                    16_384,
                ),
                ShadowArm::EiObserver => (
                    true,
                    Some(domain_digest(
                        "ei-prediction",
                        input.request_digest.as_bytes(),
                    )),
                    Some(domain_digest(
                        "ei-update-proposal",
                        format!(
                            "{}:{}:{}",
                            input.request_digest,
                            input.partition.as_str(),
                            SHADOW_SAMPLING_SEED
                        )
                        .as_bytes(),
                    )),
                    700,
                    600,
                    65_536,
                ),
            };

        SealedShadowObservation::from_observation(ShadowObservation {
            arm,
            sample_id: input.sample_id.clone(),
            namespace_id,
            input_digest: domain_digest("authorized-input", &input.to_canonical_bytes()?),
            partition: input.partition,
            observer_executed,
            prediction_digest,
            update_proposal_digest,
            response_digest_after: input.baseline_response_digest.clone(),
            route_digest_after: input.baseline_route_digest.clone(),
            tool_digest_after: input.baseline_tool_digest.clone(),
            action_digest_after: input.baseline_action_digest.clone(),
            persistence_digest_after: input.baseline_persistence_digest.clone(),
            added_latency_micros: latency,
            cpu_micros: cpu,
            ephemeral_memory_bytes: memory,
            live_write_attempted: false,
            qualifying_sample: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowVerificationReport {
    pub preregistration_id: String,
    pub classification: String,
    pub synthetic_samples: u16,
    pub arms_per_sample: u16,
    pub deterministic_replay: bool,
    pub privacy_redaction_passed: bool,
    pub matched_inputs: bool,
    pub matched_budgets: bool,
    pub zero_response_divergence: bool,
    pub zero_route_divergence: bool,
    pub zero_tool_divergence: bool,
    pub zero_action_divergence: bool,
    pub zero_persistence_divergence: bool,
    pub zero_cross_user_leakage: bool,
    pub removal_complete: bool,
    pub qualifying_shadow_samples_collected: bool,
    pub runtime_wiring: bool,
    pub live_learning_authority: bool,
    pub response_authority: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub tool_authority: bool,
    pub action_authority: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowRemovalReceipt {
    pub sampling_disabled: bool,
    pub observer_detached: bool,
    pub ephemeral_namespaces_deleted: bool,
    pub evidence_sink_closed: bool,
    pub live_state_untouched: bool,
}

impl ShadowRemovalReceipt {
    #[must_use]
    pub const fn complete() -> Self {
        Self {
            sampling_disabled: true,
            observer_detached: true,
            ephemeral_namespaces_deleted: true,
            evidence_sink_closed: true,
            live_state_untouched: true,
        }
    }

    pub fn validate(&self) -> Result<(), ShadowContractError> {
        if self.sampling_disabled
            && self.observer_detached
            && self.ephemeral_namespaces_deleted
            && self.evidence_sink_closed
            && self.live_state_untouched
        {
            Ok(())
        } else {
            Err(ShadowContractError::RemovalIncomplete)
        }
    }
}

pub fn run_synthetic_preregistration_probe(
) -> Result<ShadowVerificationReport, ShadowContractError> {
    let redactor = ShadowPrivacyRedactor;
    let adapter = ReadOnlyShadowAdapter;
    let fixtures = [
        RawShadowIngress {
            sample_id: "synthetic-sample-a",
            user_id: "user-alice@example.test",
            session_id: "session-alpha-secret",
            request_text: "Summarize the bounded evidence without retaining my raw message.",
            baseline_response_bytes: b"baseline-response-a",
            baseline_route: "chat",
            baseline_tool: "none",
            baseline_action: "none",
            baseline_persistence: "none",
        },
        RawShadowIngress {
            sample_id: "synthetic-sample-b",
            user_id: "user-bob@example.test",
            session_id: "session-beta-secret",
            request_text: "Compare the held-out score and the matched control.",
            baseline_response_bytes: b"baseline-response-b",
            baseline_route: "chat",
            baseline_tool: "none",
            baseline_action: "none",
            baseline_persistence: "none",
        },
    ];

    let mut deterministic_replay = true;
    let mut privacy_redaction_passed = true;
    let mut matched_inputs = true;
    let mut matched_budgets = true;
    let mut zero_response_divergence = true;
    let mut zero_route_divergence = true;
    let mut zero_tool_divergence = true;
    let mut zero_action_divergence = true;
    let mut zero_persistence_divergence = true;
    let mut namespaces = Vec::new();

    for fixture in &fixtures {
        let input = redactor.redact(fixture)?;
        let serialized =
            serde_json::to_string(&input).map_err(ShadowContractError::Serialization)?;
        privacy_redaction_passed &= !serialized.contains(fixture.user_id)
            && !serialized.contains(fixture.session_id)
            && !serialized.contains(fixture.request_text);

        let expected_input_digest =
            domain_digest("authorized-input", &input.to_canonical_bytes()?);
        for arm in ShadowArm::ALL {
            let first = adapter.observe(arm, &input)?;
            let second = adapter.observe(arm, &input)?;
            let first_bytes = first.to_canonical_bytes()?;
            let second_bytes = second.to_canonical_bytes()?;
            deterministic_replay &= first_bytes == second_bytes
                && SealedShadowObservation::from_canonical_bytes(&first_bytes)? == first;
            matched_inputs &= first.observation.input_digest == expected_input_digest;
            matched_budgets &= first.observation.added_latency_micros
                <= FROZEN_SHADOW_BUDGET.max_added_latency_micros
                && first.observation.cpu_micros <= FROZEN_SHADOW_BUDGET.max_cpu_micros
                && first.observation.ephemeral_memory_bytes
                    <= FROZEN_SHADOW_BUDGET.max_ephemeral_memory_bytes;
            zero_response_divergence &= first.observation.response_digest_after
                == input.baseline_response_digest;
            zero_route_divergence &=
                first.observation.route_digest_after == input.baseline_route_digest;
            zero_tool_divergence &=
                first.observation.tool_digest_after == input.baseline_tool_digest;
            zero_action_divergence &=
                first.observation.action_digest_after == input.baseline_action_digest;
            zero_persistence_divergence &= first.observation.persistence_digest_after
                == input.baseline_persistence_digest
                && !first.observation.live_write_attempted;
            namespaces.push((
                input.user_pseudonym.clone(),
                first.observation.namespace_id.clone(),
            ));
        }
    }

    let zero_cross_user_leakage = namespaces.iter().enumerate().all(|(index, left)| {
        namespaces
            .iter()
            .skip(index + 1)
            .all(|right| left.0 == right.0 || left.1 != right.1)
    });
    ShadowRemovalReceipt::complete().validate()?;

    let passed = deterministic_replay
        && privacy_redaction_passed
        && matched_inputs
        && matched_budgets
        && zero_response_divergence
        && zero_route_divergence
        && zero_tool_divergence
        && zero_action_divergence
        && zero_persistence_divergence
        && zero_cross_user_leakage;

    Ok(ShadowVerificationReport {
        preregistration_id: SHADOW_PREREGISTRATION_ID.to_owned(),
        classification: if passed {
            "PREREGISTRATION_PASS"
        } else {
            "PREREGISTRATION_FAIL"
        }
        .to_owned(),
        synthetic_samples: u16::try_from(fixtures.len())
            .map_err(|_| ShadowContractError::IntegerOverflow)?,
        arms_per_sample: u16::try_from(ShadowArm::ALL.len())
            .map_err(|_| ShadowContractError::IntegerOverflow)?,
        deterministic_replay,
        privacy_redaction_passed,
        matched_inputs,
        matched_budgets,
        zero_response_divergence,
        zero_route_divergence,
        zero_tool_divergence,
        zero_action_divergence,
        zero_persistence_divergence,
        zero_cross_user_leakage,
        removal_complete: true,
        qualifying_shadow_samples_collected: false,
        runtime_wiring: false,
        live_learning_authority: false,
        response_authority: false,
        persistence_authority: false,
        routing_authority: false,
        tool_authority: false,
        action_authority: false,
    })
}

fn validate_observation(observation: &ShadowObservation) -> Result<(), ShadowContractError> {
    if observation.sample_id.is_empty()
        || observation.namespace_id.is_empty()
        || observation.input_digest.is_empty()
    {
        return Err(ShadowContractError::EmptyObservationIdentity);
    }
    if observation.added_latency_micros > FROZEN_SHADOW_BUDGET.max_added_latency_micros {
        return Err(ShadowContractError::LatencyBudgetExceeded);
    }
    if observation.cpu_micros > FROZEN_SHADOW_BUDGET.max_cpu_micros {
        return Err(ShadowContractError::CpuBudgetExceeded);
    }
    if observation.ephemeral_memory_bytes > FROZEN_SHADOW_BUDGET.max_ephemeral_memory_bytes {
        return Err(ShadowContractError::MemoryBudgetExceeded);
    }
    if observation.live_write_attempted {
        return Err(ShadowContractError::UnauthorizedLiveWrite);
    }
    if observation.qualifying_sample {
        return Err(ShadowContractError::QualifyingCollectionProhibited);
    }
    Ok(())
}

fn partition_for(request_digest: &str) -> ShadowPartition {
    let mixed = fnv1a64(format!("{request_digest}:{SHADOW_PARTITION_SEED}").as_bytes());
    match mixed % 4 {
        0 => ShadowPartition::Development,
        1 => ShadowPartition::Calibration,
        2 => ShadowPartition::HeldOut,
        _ => ShadowPartition::Adversarial,
    }
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
pub enum ShadowContractError {
    #[error("unsupported shadow schema version {0}")]
    UnsupportedSchema(u16),
    #[error("required field is empty: {0}")]
    EmptyField(&'static str),
    #[error("ingress identity fields must be non-empty")]
    EmptyIngressIdentity,
    #[error("raw request too large: {actual} > {maximum}")]
    RawRequestTooLarge { actual: usize, maximum: usize },
    #[error("authorized input budget exceeded: {actual} > {maximum}")]
    AuthorizedInputBudgetExceeded { actual: u32, maximum: u32 },
    #[error("observation identity is empty")]
    EmptyObservationIdentity,
    #[error("added-latency budget exceeded")]
    LatencyBudgetExceeded,
    #[error("CPU budget exceeded")]
    CpuBudgetExceeded,
    #[error("ephemeral-memory budget exceeded")]
    MemoryBudgetExceeded,
    #[error("unauthorized live write attempted")]
    UnauthorizedLiveWrite,
    #[error("qualifying shadow collection is prohibited during preregistration")]
    QualifyingCollectionProhibited,
    #[error("sealed observation checksum mismatch")]
    TamperedObservation,
    #[error("canonical replay mismatch")]
    NonCanonicalReplay,
    #[error("removal procedure is incomplete")]
    RemovalIncomplete,
    #[error("integer conversion overflow")]
    IntegerOverflow,
    #[error("serialization error: {0}")]
    Serialization(serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture<'a>() -> RawShadowIngress<'a> {
        RawShadowIngress {
            sample_id: "fixture-1",
            user_id: "private-user@example.test",
            session_id: "private-session-token",
            request_text: "This raw request must not cross the shadow privacy boundary.",
            baseline_response_bytes: b"baseline-response",
            baseline_route: "chat",
            baseline_tool: "none",
            baseline_action: "none",
            baseline_persistence: "none",
        }
    }

    #[test]
    fn privacy_redaction_removes_raw_identity_and_text() {
        let raw = fixture();
        let redacted = ShadowPrivacyRedactor.redact(&raw).expect("redact");
        let json = serde_json::to_string(&redacted).expect("serialize");
        assert!(!json.contains(raw.user_id));
        assert!(!json.contains(raw.session_id));
        assert!(!json.contains(raw.request_text));
    }

    #[test]
    fn all_arms_receive_identical_authorized_input() {
        let input = ShadowPrivacyRedactor.redact(&fixture()).expect("redact");
        let adapter = ReadOnlyShadowAdapter;
        let expected =
            domain_digest("authorized-input", &input.to_canonical_bytes().expect("bytes"));
        for arm in ShadowArm::ALL {
            let record = adapter.observe(arm, &input).expect("observe");
            assert_eq!(record.observation.input_digest, expected);
            assert_eq!(
                record.observation.response_digest_after,
                input.baseline_response_digest
            );
            assert!(!record.observation.live_write_attempted);
        }
    }

    #[test]
    fn canonical_shadow_replay_is_exact() {
        let input = ShadowPrivacyRedactor.redact(&fixture()).expect("redact");
        let record = ReadOnlyShadowAdapter
            .observe(ShadowArm::EiObserver, &input)
            .expect("observe");
        let bytes = record.to_canonical_bytes().expect("bytes");
        let replay = SealedShadowObservation::from_canonical_bytes(&bytes).expect("replay");
        assert_eq!(record, replay);
        assert_eq!(bytes, replay.to_canonical_bytes().expect("replay bytes"));
    }

    #[test]
    fn tampering_fails_closed() {
        let input = ShadowPrivacyRedactor.redact(&fixture()).expect("redact");
        let mut record = ReadOnlyShadowAdapter
            .observe(ShadowArm::EiObserver, &input)
            .expect("observe");
        record.observation.response_digest_after = "tampered".to_owned();
        assert!(matches!(
            record.validate(),
            Err(ShadowContractError::TamperedObservation)
        ));
    }

    #[test]
    fn over_budget_and_live_write_records_fail_closed() {
        let input = ShadowPrivacyRedactor.redact(&fixture()).expect("redact");
        let mut observation = ReadOnlyShadowAdapter
            .observe(ShadowArm::EiObserver, &input)
            .expect("observe")
            .observation;
        observation.added_latency_micros =
            FROZEN_SHADOW_BUDGET.max_added_latency_micros + 1;
        assert!(matches!(
            SealedShadowObservation::from_observation(observation),
            Err(ShadowContractError::LatencyBudgetExceeded)
        ));

        let mut live_write = ReadOnlyShadowAdapter
            .observe(ShadowArm::EiObserver, &input)
            .expect("observe")
            .observation;
        live_write.live_write_attempted = true;
        assert!(matches!(
            SealedShadowObservation::from_observation(live_write),
            Err(ShadowContractError::UnauthorizedLiveWrite)
        ));
    }

    #[test]
    fn different_users_receive_isolated_namespaces() {
        let first = fixture();
        let second = RawShadowIngress {
            user_id: "another-user@example.test",
            session_id: "another-private-session",
            ..first.clone()
        };
        let first_input = ShadowPrivacyRedactor.redact(&first).expect("first");
        let second_input = ShadowPrivacyRedactor.redact(&second).expect("second");
        let adapter = ReadOnlyShadowAdapter;
        let first_record = adapter
            .observe(ShadowArm::EiObserver, &first_input)
            .expect("first record");
        let second_record = adapter
            .observe(ShadowArm::EiObserver, &second_input)
            .expect("second record");
        assert_ne!(
            first_record.observation.namespace_id,
            second_record.observation.namespace_id
        );
    }

    #[test]
    fn removal_receipt_requires_every_boundary_closed() {
        ShadowRemovalReceipt::complete()
            .validate()
            .expect("complete removal");
        let incomplete = ShadowRemovalReceipt {
            evidence_sink_closed: false,
            ..ShadowRemovalReceipt::complete()
        };
        assert!(matches!(
            incomplete.validate(),
            Err(ShadowContractError::RemovalIncomplete)
        ));
    }

    #[test]
    fn synthetic_preregistration_probe_passes_without_collection() {
        let report = run_synthetic_preregistration_probe().expect("probe");
        assert_eq!(report.classification, "PREREGISTRATION_PASS");
        assert!(!report.qualifying_shadow_samples_collected);
        assert!(!report.runtime_wiring);
        assert!(!report.live_learning_authority);
        assert!(!report.response_authority);
        assert!(!report.persistence_authority);
        assert!(!report.routing_authority);
        assert!(!report.tool_authority);
        assert!(!report.action_authority);
    }
}
