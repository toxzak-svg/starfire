use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceCue {
    pub cue: String,
    pub reliability_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteOption {
    pub action: String,
    /// Ordered route from start to goal.
    pub path: Vec<String>,
    pub total_cost: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteTask {
    pub start: String,
    pub goal: String,
    /// Sorted by action.
    pub options: Vec<RouteOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleExample {
    pub object: String,
    /// Sorted and unique.
    pub attributes: Vec<String>,
    pub matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCandidate {
    pub action: String,
    pub object: String,
    /// Sorted and unique.
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRuleTask {
    /// Hidden environment truth consumed by the independent evaluator, not an
    /// agent rationale or learning proposal. Sorted and unique.
    pub required_attributes: Vec<String>,
    /// Sorted by object.
    pub examples: Vec<RuleExample>,
    /// Sorted by action.
    pub candidates: Vec<RuleCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "family", content = "task", rename_all = "snake_case")]
pub enum TaskPayload {
    RouteChoice(RouteTask),
    AttributeRule(AttributeRuleTask),
}

impl TaskPayload {
    pub fn family(&self) -> TaskFamily {
        match self {
            Self::RouteChoice(_) => TaskFamily::RouteChoice,
            Self::AttributeRule(_) => TaskFamily::AttributeRule,
        }
    }

    pub fn legal_actions(&self) -> Vec<String> {
        match self {
            Self::RouteChoice(task) => task
                .options
                .iter()
                .map(|option| option.action.clone())
                .collect(),
            Self::AttributeRule(task) => task
                .candidates
                .iter()
                .map(|candidate| candidate.action.clone())
                .collect(),
        }
    }

    fn optimal_action(&self) -> Result<String, EnvironmentError> {
        match self {
            Self::RouteChoice(task) => task
                .options
                .iter()
                .min_by_key(|option| option.total_cost)
                .map(|option| option.action.clone())
                .ok_or(EnvironmentError::InvalidTask("route options")),
            Self::AttributeRule(task) => {
                let matching: Vec<&RuleCandidate> = task
                    .candidates
                    .iter()
                    .filter(|candidate| {
                        task.required_attributes.iter().all(|required| {
                            candidate.attributes.binary_search(required).is_ok()
                        })
                    })
                    .collect();
                if matching.len() != 1 {
                    return Err(EnvironmentError::InvalidTask(
                        "attribute rule requires one matching candidate",
                    ));
                }
                Ok(matching[0].action.clone())
            }
        }
    }

    fn validate(&self) -> Result<(), EnvironmentError> {
        match self {
            Self::RouteChoice(task) => validate_route_task(task),
            Self::AttributeRule(task) => validate_attribute_rule_task(task),
        }
    }

    fn structure_fingerprint(&self) -> Result<String, EnvironmentError> {
        let normalized = match self {
            Self::RouteChoice(task) => serde_json::json!({
                "family": "route_choice",
                "option_count": task.options.len(),
                "path_lengths": task.options.iter().map(|option| option.path.len()).collect::<Vec<_>>(),
            }),
            Self::AttributeRule(task) => serde_json::json!({
                "family": "attribute_rule",
                "example_count": task.examples.len(),
                "candidate_count": task.candidates.len(),
                "attribute_widths": task.candidates.iter().map(|candidate| candidate.attributes.len()).collect::<Vec<_>>(),
            }),
        };
        let bytes = serde_json::to_vec(&normalized).map_err(serialization_error)?;
        Ok(checksum128(&bytes))
    }

    fn relation_fingerprint(&self) -> Result<String, EnvironmentError> {
        let normalized = match self {
            Self::RouteChoice(task) => {
                let mut costs: Vec<u16> = task.options.iter().map(|option| option.total_cost).collect();
                costs.sort_unstable();
                serde_json::json!({"family": "route_choice", "ordered_costs": costs})
            }
            Self::AttributeRule(task) => serde_json::json!({
                "family": "attribute_rule",
                "rule_arity": task.required_attributes.len(),
                "rule": "conjunction",
            }),
        };
        let bytes = serde_json::to_vec(&normalized).map_err(serialization_error)?;
        Ok(checksum128(&bytes))
    }

    fn surface_fingerprint(&self) -> Result<String, EnvironmentError> {
        let bytes = serde_json::to_vec(self).map_err(serialization_error)?;
        Ok(checksum128(&bytes))
    }

    pub(super) fn score_action(&self, action: &str) -> Result<(bool, u16), EnvironmentError> {
        match self {
            Self::RouteChoice(task) => {
                let selected = task
                    .options
                    .iter()
                    .find(|option| option.action == action)
                    .ok_or_else(|| EnvironmentError::IllegalAction(action.into()))?;
                let best = task
                    .options
                    .iter()
                    .map(|option| option.total_cost)
                    .min()
                    .ok_or(EnvironmentError::InvalidTask("route options"))?;
                let objective_satisfied = selected.total_cost == best;
                let penalty = selected.total_cost.saturating_sub(best).saturating_mul(750);
                Ok((
                    objective_satisfied,
                    MAX_BASIS_POINTS.saturating_sub(penalty),
                ))
            }
            Self::AttributeRule(task) => {
                let candidate = task
                    .candidates
                    .iter()
                    .find(|candidate| candidate.action == action)
                    .ok_or_else(|| EnvironmentError::IllegalAction(action.into()))?;
                let matches = task.required_attributes.iter().all(|required| {
                    candidate.attributes.binary_search(required).is_ok()
                });
                Ok((matches, if matches { MAX_BASIS_POINTS } else { 0 }))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskFixture {
    pub fixture_id: String,
    pub partition: EvaluationPartition,
    pub seed: u64,
    pub family: TaskFamily,
    pub task: TaskPayload,
    /// Sorted and unique environment cues. Adversarial fixtures intentionally
    /// contain a low-reliability misleading cue.
    pub evidence_cues: Vec<EvidenceCue>,
    pub optimal_action: String,
    pub action_budget: u32,
    pub evidence_budget: u32,
    pub manifest_digest: EnvironmentDigest,
    pub structure_fingerprint: String,
    pub relation_fingerprint: String,
    pub surface_fingerprint: String,
    pub authority: AuthoritySnapshot,
}

impl TaskFixture {
    pub fn validate(&self) -> Result<(), EnvironmentError> {
        validate_text(&self.fixture_id)?;
        if self.action_budget == 0 || self.evidence_budget == 0 {
            return Err(EnvironmentError::ZeroBudget);
        }
        if !self.authority.is_closed() {
            return Err(EnvironmentError::UnauthorizedEnvironment);
        }
        self.task.validate()?;
        if self.task.family() != self.family {
            return Err(EnvironmentError::InvalidTask("family mismatch"));
        }
        validate_sorted_unique_by(
            &self.evidence_cues,
            |cue| cue.cue.as_str(),
            "evidence cues",
        )?;
        for cue in &self.evidence_cues {
            validate_text(&cue.cue)?;
            validate_basis_points(cue.reliability_bps, "cue reliability")?;
        }
        validate_digest(&self.manifest_digest)?;
        let frozen_manifest = FrozenEnvironmentManifest::ei_0b_default();
        if self.manifest_digest != frozen_manifest.digest()?
            || !frozen_manifest.contains(self.partition, self.seed)
        {
            return Err(EnvironmentError::CrossPartitionFixture {
                partition: self.partition,
                seed: self.seed,
            });
        }
        validate_digest_text(&self.structure_fingerprint)?;
        validate_digest_text(&self.relation_fingerprint)?;
        validate_digest_text(&self.surface_fingerprint)?;
        let expected_optimal = self.task.optimal_action()?;
        if self.optimal_action != expected_optimal {
            return Err(EnvironmentError::InvalidTask("incorrect optimal action"));
        }
        if self.structure_fingerprint != self.task.structure_fingerprint()?
            || self.relation_fingerprint != self.task.relation_fingerprint()?
            || self.surface_fingerprint != self.task.surface_fingerprint()?
        {
            return Err(EnvironmentError::FingerprintMismatch);
        }
        Ok(())
    }

    fn canonical_payload_bytes(&self) -> Result<Vec<u8>, EnvironmentError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn seal(self) -> Result<SealedTaskFixture, EnvironmentError> {
        SealedTaskFixture::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnvironmentDigest(pub(super) String);

impl EnvironmentDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedTaskFixture {
    pub schema_version: u16,
    pub fixture: TaskFixture,
    pub digest: EnvironmentDigest,
}

impl SealedTaskFixture {
    pub fn new(fixture: TaskFixture) -> Result<Self, EnvironmentError> {
        let digest = EnvironmentDigest(checksum128(&fixture.canonical_payload_bytes()?));
        Ok(Self {
            schema_version: EI_0B_SCHEMA_VERSION,
            fixture,
            digest,
        })
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        if self.schema_version != EI_0B_SCHEMA_VERSION {
            return Err(EnvironmentError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        self.fixture.validate()?;
        validate_digest(&self.digest)?;
        let expected = EnvironmentDigest(checksum128(
            &self.fixture.canonical_payload_bytes()?,
        ));
        if expected != self.digest {
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

mod generator;
pub use generator::generate_frozen_fixture;

fn validate_route_task(task: &RouteTask) -> Result<(), EnvironmentError> {
    validate_text(&task.start)?;
    validate_text(&task.goal)?;
    if task.start == task.goal || task.options.len() < 2 {
        return Err(EnvironmentError::InvalidTask("route shape"));
    }
    validate_sorted_unique_by(&task.options, |option| option.action.as_str(), "route options")?;
    let mut costs = BTreeSet::new();
    for option in &task.options {
        validate_text(&option.action)?;
        if option.path.len() < 2
            || option.path.first() != Some(&task.start)
            || option.path.last() != Some(&task.goal)
            || option.total_cost == 0
        {
            return Err(EnvironmentError::InvalidTask("route option"));
        }
        for node in &option.path {
            validate_text(node)?;
        }
        if !costs.insert(option.total_cost) {
            return Err(EnvironmentError::InvalidTask("route costs must be unique"));
        }
    }
    Ok(())
}

fn validate_attribute_rule_task(task: &AttributeRuleTask) -> Result<(), EnvironmentError> {
    validate_sorted_unique_strings(&task.required_attributes, "required attributes")?;
    if task.required_attributes.len() != 2 || task.examples.len() < 3 || task.candidates.len() < 3 {
        return Err(EnvironmentError::InvalidTask("attribute rule shape"));
    }
    validate_sorted_unique_by(&task.examples, |example| example.object.as_str(), "rule examples")?;
    validate_sorted_unique_by(
        &task.candidates,
        |candidate| candidate.action.as_str(),
        "rule candidates",
    )?;
    let mut candidate_objects = BTreeSet::new();
    for example in &task.examples {
        validate_text(&example.object)?;
        validate_sorted_unique_strings(&example.attributes, "example attributes")?;
        let actual = task.required_attributes.iter().all(|required| {
            example.attributes.binary_search(required).is_ok()
        });
        if actual != example.matches {
            return Err(EnvironmentError::InvalidTask("incorrect rule example label"));
        }
    }
    for candidate in &task.candidates {
        validate_text(&candidate.action)?;
        validate_text(&candidate.object)?;
        validate_sorted_unique_strings(&candidate.attributes, "candidate attributes")?;
        if !candidate_objects.insert(candidate.object.as_str()) {
            return Err(EnvironmentError::NonCanonicalCollection(
                "candidate objects",
            ));
        }
    }
    task_payload_optimal_count(task)?;
    Ok(())
}

fn task_payload_optimal_count(task: &AttributeRuleTask) -> Result<(), EnvironmentError> {
    let count = task
        .candidates
        .iter()
        .filter(|candidate| {
            task.required_attributes.iter().all(|required| {
                candidate.attributes.binary_search(required).is_ok()
            })
        })
        .count();
    if count != 1 {
        return Err(EnvironmentError::InvalidTask(
            "attribute rule requires one matching candidate",
        ));
    }
    Ok(())
}
