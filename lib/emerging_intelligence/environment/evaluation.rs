use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ArmStatePolicy {
    ProposedLearningState,
    FreshNoUpdate,
    FreshMemoryDisabled,
    FreshRandomUpdate { control_seed: u64 },
    FreshFixedPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialArmSpec {
    pub arm: ControlArm,
    pub fixture_digest: EnvironmentDigest,
    pub state_namespace: String,
    pub state_policy: ArmStatePolicy,
    pub action_budget: u32,
    pub evidence_budget: u32,
    pub evidence_exposure_digest: EnvironmentDigest,
    pub learning_application_authority: bool,
    pub authority: AuthoritySnapshot,
}

impl TrialArmSpec {
    fn validate(&self) -> Result<(), EnvironmentError> {
        validate_digest(&self.fixture_digest)?;
        validate_text(&self.state_namespace)?;
        validate_digest(&self.evidence_exposure_digest)?;
        if self.action_budget == 0 || self.evidence_budget == 0 {
            return Err(EnvironmentError::ZeroBudget);
        }
        if self.learning_application_authority || !self.authority.is_closed() {
            return Err(EnvironmentError::UnauthorizedEnvironment);
        }
        let policy_matches_arm = matches!(
            (self.arm, &self.state_policy),
            (ControlArm::Learning, ArmStatePolicy::ProposedLearningState)
                | (ControlArm::NoUpdate, ArmStatePolicy::FreshNoUpdate)
                | (
                    ControlArm::MemoryDisabled,
                    ArmStatePolicy::FreshMemoryDisabled
                )
                | (
                    ControlArm::RandomUpdate,
                    ArmStatePolicy::FreshRandomUpdate { .. }
                )
                | (ControlArm::FixedPolicy, ArmStatePolicy::FreshFixedPolicy)
        );
        if !policy_matches_arm {
            return Err(EnvironmentError::ControlStateLeak(self.arm));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchedTrialSet {
    pub fixture_digest: EnvironmentDigest,
    /// Canonical ControlArm order.
    pub arms: Vec<TrialArmSpec>,
}

impl MatchedTrialSet {
    pub fn for_fixture(fixture: &SealedTaskFixture) -> Result<Self, EnvironmentError> {
        fixture.validate()?;
        let exposure = EnvironmentDigest(checksum128(
            &serde_json::to_vec(&fixture.fixture.evidence_cues).map_err(serialization_error)?,
        ));
        let arms = ControlArm::ALL
            .into_iter()
            .map(|arm| TrialArmSpec {
                arm,
                fixture_digest: fixture.digest.clone(),
                state_namespace: format!("ei-0b:{}:{}", fixture.fixture.fixture_id, arm.as_str()),
                state_policy: match arm {
                    ControlArm::Learning => ArmStatePolicy::ProposedLearningState,
                    ControlArm::NoUpdate => ArmStatePolicy::FreshNoUpdate,
                    ControlArm::MemoryDisabled => ArmStatePolicy::FreshMemoryDisabled,
                    ControlArm::RandomUpdate => ArmStatePolicy::FreshRandomUpdate {
                        control_seed: fixture.fixture.seed ^ 0xa5a5_5a5a_d3c1_b7e9,
                    },
                    ControlArm::FixedPolicy => ArmStatePolicy::FreshFixedPolicy,
                },
                action_budget: fixture.fixture.action_budget,
                evidence_budget: fixture.fixture.evidence_budget,
                evidence_exposure_digest: exposure.clone(),
                learning_application_authority: false,
                authority: AuthoritySnapshot::closed(),
            })
            .collect();
        let set = Self {
            fixture_digest: fixture.digest.clone(),
            arms,
        };
        set.validate()?;
        Ok(set)
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        validate_digest(&self.fixture_digest)?;
        if self.arms.len() != ControlArm::ALL.len() {
            return Err(EnvironmentError::IncompleteControlArms);
        }
        let mut namespaces = BTreeSet::new();
        let first = self
            .arms
            .first()
            .ok_or(EnvironmentError::IncompleteControlArms)?;
        for (spec, expected_arm) in self.arms.iter().zip(ControlArm::ALL) {
            spec.validate()?;
            if spec.arm != expected_arm {
                return Err(EnvironmentError::NonCanonicalCollection("control arms"));
            }
            if spec.fixture_digest != self.fixture_digest {
                return Err(EnvironmentError::FixtureMismatch);
            }
            if spec.action_budget != first.action_budget
                || spec.evidence_budget != first.evidence_budget
                || spec.evidence_exposure_digest != first.evidence_exposure_digest
            {
                return Err(EnvironmentError::UnmatchedControlBudget);
            }
            if !namespaces.insert(spec.state_namespace.as_str()) {
                return Err(EnvironmentError::ControlStateLeak(spec.arm));
            }
        }
        Ok(())
    }

    pub fn arm(&self, arm: ControlArm) -> Option<&TrialArmSpec> {
        self.arms.iter().find(|spec| spec.arm == arm)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedAction {
    pub step: u32,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionTrace {
    pub fixture_digest: EnvironmentDigest,
    pub arm: ControlArm,
    /// Strictly increasing steps.
    pub actions: Vec<RecordedAction>,
    pub evidence_reads: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentEvaluation {
    pub evaluator_id: String,
    pub partition: EvaluationPartition,
    pub family: TaskFamily,
    pub seed: u64,
    pub arm: ControlArm,
    pub fixture_digest: EnvironmentDigest,
    pub trace_digest: EnvironmentDigest,
    pub selected_action: Option<String>,
    pub objective_satisfied: bool,
    pub score_bps: u16,
    pub action_count: u32,
    pub evidence_reads: u32,
}

impl IndependentEvaluation {
    fn key(&self) -> String {
        format!(
            "{:02}:{}:{:020}:{}",
            partition_rank(self.partition),
            self.family.as_str(),
            self.seed,
            self.arm.as_str()
        )
    }

    fn validate(&self) -> Result<(), EnvironmentError> {
        if self.evaluator_id != EI_0B_EVALUATOR_ID {
            return Err(EnvironmentError::InvalidEvaluator);
        }
        validate_digest(&self.fixture_digest)?;
        validate_digest(&self.trace_digest)?;
        validate_basis_points(self.score_bps, "evaluation score")?;
        if let Some(action) = &self.selected_action {
            validate_text(action)?;
        }
        Ok(())
    }
}

pub struct IndependentEvaluator;

impl IndependentEvaluator {
    /// Scores only the sealed environment fixture and recorded action trace.
    /// No rationale, memory contents, proposed update, or internal model state is
    /// accepted by this API.
    pub fn evaluate(
        fixture: &SealedTaskFixture,
        arm_spec: &TrialArmSpec,
        trace: &ActionTrace,
    ) -> Result<IndependentEvaluation, EnvironmentError> {
        fixture.validate()?;
        arm_spec.validate()?;
        if arm_spec.fixture_digest != fixture.digest
            || trace.fixture_digest != fixture.digest
            || trace.arm != arm_spec.arm
        {
            return Err(EnvironmentError::FixtureMismatch);
        }
        if trace.actions.len() as u32 > arm_spec.action_budget
            || trace.evidence_reads > arm_spec.evidence_budget
        {
            return Err(EnvironmentError::BudgetExceeded);
        }
        if trace
            .actions
            .windows(2)
            .any(|pair| pair[0].step >= pair[1].step)
        {
            return Err(EnvironmentError::NonCanonicalCollection("action trace"));
        }
        for action in &trace.actions {
            validate_text(&action.action)?;
            if !fixture
                .fixture
                .task
                .legal_actions()
                .iter()
                .any(|legal| legal == &action.action)
            {
                return Err(EnvironmentError::IllegalAction(action.action.clone()));
            }
        }
        let trace_bytes = serde_json::to_vec(trace).map_err(serialization_error)?;
        let trace_digest = EnvironmentDigest(checksum128(&trace_bytes));
        let selected_action = trace.actions.first().map(|action| action.action.clone());
        let (objective_satisfied, score_bps) = match selected_action.as_deref() {
            Some(action) => fixture.fixture.task.score_action(action)?,
            None => (false, 0),
        };
        let evaluation = IndependentEvaluation {
            evaluator_id: EI_0B_EVALUATOR_ID.into(),
            partition: fixture.fixture.partition,
            family: fixture.fixture.family,
            seed: fixture.fixture.seed,
            arm: arm_spec.arm,
            fixture_digest: fixture.digest.clone(),
            trace_digest,
            selected_action,
            objective_satisfied,
            score_bps,
            action_count: trace.actions.len() as u32,
            evidence_reads: trace.evidence_reads,
        };
        evaluation.validate()?;
        Ok(evaluation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentReport {
    pub manifest_digest: EnvironmentDigest,
    /// Sorted by partition, family, seed, and arm.
    pub evaluations: Vec<IndependentEvaluation>,
    pub exact_replay_required: bool,
    pub runtime_wiring: bool,
    pub learning_application_authority: bool,
    pub claim: String,
}

impl EnvironmentReport {
    pub fn new(
        manifest: &FrozenEnvironmentManifest,
        mut evaluations: Vec<IndependentEvaluation>,
    ) -> Result<Self, EnvironmentError> {
        manifest.validate()?;
        evaluations.sort_by_key(IndependentEvaluation::key);
        let report = Self {
            manifest_digest: manifest.digest()?,
            evaluations,
            exact_replay_required: true,
            runtime_wiring: false,
            learning_application_authority: false,
            claim: "ei-0b-experiment-infrastructure-only".into(),
        };
        report.validate()?;
        Ok(report)
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        validate_digest(&self.manifest_digest)?;
        if self.manifest_digest != FrozenEnvironmentManifest::ei_0b_default().digest()? {
            return Err(EnvironmentError::FixtureMismatch);
        }
        if !self.exact_replay_required
            || self.runtime_wiring
            || self.learning_application_authority
            || self.claim != "ei-0b-experiment-infrastructure-only"
        {
            return Err(EnvironmentError::UnauthorizedEnvironment);
        }
        if self.evaluations.is_empty() {
            return Err(EnvironmentError::EmptyReport);
        }
        for evaluation in &self.evaluations {
            evaluation.validate()?;
        }
        if self
            .evaluations
            .windows(2)
            .any(|pair| pair[0].key() >= pair[1].key())
        {
            return Err(EnvironmentError::NonCanonicalCollection(
                "report evaluations",
            ));
        }
        Ok(())
    }

    fn canonical_payload_bytes(&self) -> Result<Vec<u8>, EnvironmentError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn seal(self) -> Result<SealedEnvironmentReport, EnvironmentError> {
        SealedEnvironmentReport::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedEnvironmentReport {
    pub schema_version: u16,
    pub report: EnvironmentReport,
    pub digest: EnvironmentDigest,
}

impl SealedEnvironmentReport {
    pub fn new(report: EnvironmentReport) -> Result<Self, EnvironmentError> {
        let digest = EnvironmentDigest(checksum128(&report.canonical_payload_bytes()?));
        Ok(Self {
            schema_version: EI_0B_SCHEMA_VERSION,
            report,
            digest,
        })
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        if self.schema_version != EI_0B_SCHEMA_VERSION {
            return Err(EnvironmentError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        self.report.validate()?;
        validate_digest(&self.digest)?;
        let expected = EnvironmentDigest(checksum128(&self.report.canonical_payload_bytes()?));
        if self.digest != expected {
            return Err(EnvironmentError::DigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, EnvironmentError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, EnvironmentError> {
        let sealed: Self = serde_json::from_slice(bytes).map_err(deserialization_error)?;
        sealed.validate()?;
        let canonical = serde_json::to_vec(&sealed).map_err(serialization_error)?;
        if canonical != bytes {
            return Err(EnvironmentError::NonCanonicalEncoding);
        }
        Ok(sealed)
    }
}
