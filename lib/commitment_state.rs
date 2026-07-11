//! Proof-carrying executable commitment state.
//!
//! This module provides the minimal state-transition substrate used by the H9
//! experiment. Raw observations are inert until an operation proposes a typed
//! delta that validates against an observation. Derived facts must carry an
//! auditable proof that references an already committed rule and supporting
//! fact. Later operations consume committed state rather than a transcript.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// A canonical symbolic atom used by executable commitments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Atom(String);

impl Atom {
    pub fn new(value: impl Into<String>) -> Result<Self, CommitmentStateError> {
        let value = value.into();
        let canonical = value.trim();
        if canonical.is_empty() {
            return Err(CommitmentStateError::EmptyAtom);
        }
        Ok(Self(canonical.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A one-premise executable implication.
///
/// H9 deliberately starts with the smallest closed transition language that can
/// demonstrate causal state dependence. Richer operators can be added only
/// after this base substrate survives falsification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Rule {
    pub antecedent: Atom,
    pub consequent: Atom,
}

impl Rule {
    pub fn new(antecedent: Atom, consequent: Atom) -> Result<Self, CommitmentStateError> {
        if antecedent == consequent {
            return Err(CommitmentStateError::DegenerateRule);
        }
        Ok(Self {
            antecedent,
            consequent,
        })
    }
}

/// Immutable raw evidence that can authorize one exact executable rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessedRule {
    pub witness_id: u64,
    pub rule: Rule,
}

/// Stable identifier for one committed fact or rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitmentId(pub u64);

/// Auditable origin of a commitment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    Seed,
    Witness {
        witness_id: u64,
    },
    RuleApplication {
        rule_id: CommitmentId,
        support_fact_id: CommitmentId,
    },
}

/// An executable state item. Text is not an executable commitment unless it
/// has been converted into one of these typed variants through a valid delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Commitment {
    Fact {
        atom: Atom,
        provenance: Provenance,
    },
    Rule {
        rule: Rule,
        provenance: Provenance,
    },
}

/// The only mutations permitted by the H9 substrate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateDelta {
    /// Compile one exact raw witness into an executable rule.
    CompileWitnessedRule { witness_id: u64, rule: Rule },
    /// Derive a new fact from an already committed rule and support fact.
    DeriveFact {
        rule_id: CommitmentId,
        support_fact_id: CommitmentId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CommitmentStateError {
    #[error("atoms must not be empty")]
    EmptyAtom,
    #[error("self-loop rules are not admitted by the H9 primitive")]
    DegenerateRule,
    #[error("witness id {0} already exists")]
    DuplicateWitness(u64),
    #[error("fact is already committed: {0}")]
    DuplicateFact(String),
    #[error("rule is already committed: {0} -> {1}")]
    DuplicateRule(String, String),
    #[error("unknown witness id {0}")]
    UnknownWitness(u64),
    #[error("the proposed rule does not match witness id {0}")]
    WitnessMismatch(u64),
    #[error("unknown commitment id {0:?}")]
    UnknownCommitment(CommitmentId),
    #[error("commitment {0:?} is not a rule")]
    ExpectedRule(CommitmentId),
    #[error("commitment {0:?} is not a fact")]
    ExpectedFact(CommitmentId),
    #[error("support fact does not satisfy the selected rule antecedent")]
    AntecedentMismatch,
    #[error("state invariant violation: {0}")]
    InvariantViolation(String),
}

/// A deterministic, proof-carrying executable state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableCommitmentState {
    observations: BTreeMap<u64, WitnessedRule>,
    commitments: BTreeMap<CommitmentId, Commitment>,
    fact_index: BTreeMap<Atom, CommitmentId>,
    rule_index: BTreeMap<Rule, CommitmentId>,
    next_id: u64,
}

impl Default for ExecutableCommitmentState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutableCommitmentState {
    pub fn new() -> Self {
        Self {
            observations: BTreeMap::new(),
            commitments: BTreeMap::new(),
            fact_index: BTreeMap::new(),
            rule_index: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Add immutable raw evidence. This does not alter executable facts or
    /// rules; another operation must explicitly compile it through a valid delta.
    pub fn add_observation(
        &mut self,
        observation: WitnessedRule,
    ) -> Result<(), CommitmentStateError> {
        if self.observations.contains_key(&observation.witness_id) {
            return Err(CommitmentStateError::DuplicateWitness(
                observation.witness_id,
            ));
        }
        self.observations
            .insert(observation.witness_id, observation);
        self.verify_invariants()
    }

    pub fn seed_fact(&mut self, atom: Atom) -> Result<CommitmentId, CommitmentStateError> {
        self.insert_fact(atom, Provenance::Seed)
    }

    pub fn seed_rule(&mut self, rule: Rule) -> Result<CommitmentId, CommitmentStateError> {
        self.insert_rule(rule, Provenance::Seed)
    }

    pub fn observation(&self, witness_id: u64) -> Option<&WitnessedRule> {
        self.observations.get(&witness_id)
    }

    pub fn commitment(&self, id: CommitmentId) -> Option<&Commitment> {
        self.commitments.get(&id)
    }

    pub fn contains_fact(&self, atom: &Atom) -> bool {
        self.fact_index.contains_key(atom)
    }

    pub fn contains_rule(&self, rule: &Rule) -> bool {
        self.rule_index.contains_key(rule)
    }

    pub fn fact_id(&self, atom: &Atom) -> Option<CommitmentId> {
        self.fact_index.get(atom).copied()
    }

    pub fn rule_id(&self, rule: &Rule) -> Option<CommitmentId> {
        self.rule_index.get(rule).copied()
    }

    pub fn commitment_count(&self) -> usize {
        self.commitments.len()
    }

    /// Propose every currently enabled derivation in canonical order.
    ///
    /// This is the read boundary for the H9 executor: it can read executable
    /// commitments, but it cannot read raw observations or transcript text.
    pub fn enabled_derivations(&self) -> Vec<StateDelta> {
        let mut enabled = Vec::<(Atom, CommitmentId, CommitmentId, StateDelta)>::new();

        for (rule_id, commitment) in &self.commitments {
            let Commitment::Rule { rule, .. } = commitment else {
                continue;
            };
            let Some(support_fact_id) = self.fact_index.get(&rule.antecedent).copied() else {
                continue;
            };
            if self.fact_index.contains_key(&rule.consequent) {
                continue;
            }
            enabled.push((
                rule.consequent.clone(),
                *rule_id,
                support_fact_id,
                StateDelta::DeriveFact {
                    rule_id: *rule_id,
                    support_fact_id,
                },
            ));
        }

        enabled.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        enabled
            .into_iter()
            .map(|(_, _, _, delta)| delta)
            .collect()
    }

    /// Validate and atomically apply one typed delta.
    pub fn apply_delta(
        &mut self,
        delta: StateDelta,
    ) -> Result<CommitmentId, CommitmentStateError> {
        let id = match delta {
            StateDelta::CompileWitnessedRule { witness_id, rule } => {
                let witness = self
                    .observations
                    .get(&witness_id)
                    .ok_or(CommitmentStateError::UnknownWitness(witness_id))?;
                if witness.rule != rule {
                    return Err(CommitmentStateError::WitnessMismatch(witness_id));
                }
                self.insert_rule(rule, Provenance::Witness { witness_id })?
            }
            StateDelta::DeriveFact {
                rule_id,
                support_fact_id,
            } => {
                let rule = match self
                    .commitments
                    .get(&rule_id)
                    .ok_or(CommitmentStateError::UnknownCommitment(rule_id))?
                {
                    Commitment::Rule { rule, .. } => rule.clone(),
                    Commitment::Fact { .. } => {
                        return Err(CommitmentStateError::ExpectedRule(rule_id))
                    }
                };
                let support = match self
                    .commitments
                    .get(&support_fact_id)
                    .ok_or(CommitmentStateError::UnknownCommitment(support_fact_id))?
                {
                    Commitment::Fact { atom, .. } => atom.clone(),
                    Commitment::Rule { .. } => {
                        return Err(CommitmentStateError::ExpectedFact(support_fact_id))
                    }
                };
                if support != rule.antecedent {
                    return Err(CommitmentStateError::AntecedentMismatch);
                }
                self.insert_fact(
                    rule.consequent,
                    Provenance::RuleApplication {
                        rule_id,
                        support_fact_id,
                    },
                )?
            }
        };

        self.verify_invariants()?;
        Ok(id)
    }

    /// Stable, human-auditable state serialization used for replay equality.
    pub fn canonical_signature(&self) -> String {
        let mut lines = Vec::new();
        for (id, commitment) in &self.commitments {
            match commitment {
                Commitment::Fact { atom, provenance } => lines.push(format!(
                    "{}:fact:{}:{}",
                    id.0,
                    atom.as_str(),
                    provenance_signature(provenance)
                )),
                Commitment::Rule { rule, provenance } => lines.push(format!(
                    "{}:rule:{}->{}:{}",
                    id.0,
                    rule.antecedent.as_str(),
                    rule.consequent.as_str(),
                    provenance_signature(provenance)
                )),
            }
        }
        lines.join("\n")
    }

    pub fn verify_invariants(&self) -> Result<(), CommitmentStateError> {
        if self.fact_index.len() + self.rule_index.len() != self.commitments.len() {
            return Err(CommitmentStateError::InvariantViolation(
                "indexes do not cover every commitment exactly once".into(),
            ));
        }

        for (atom, id) in &self.fact_index {
            match self.commitments.get(id) {
                Some(Commitment::Fact {
                    atom: committed, ..
                }) if committed == atom => {}
                _ => {
                    return Err(CommitmentStateError::InvariantViolation(format!(
                        "fact index points to invalid commitment {:?}",
                        id
                    )))
                }
            }
        }
        for (rule, id) in &self.rule_index {
            match self.commitments.get(id) {
                Some(Commitment::Rule {
                    rule: committed, ..
                }) if committed == rule => {}
                _ => {
                    return Err(CommitmentStateError::InvariantViolation(format!(
                        "rule index points to invalid commitment {:?}",
                        id
                    )))
                }
            }
        }

        for (id, commitment) in &self.commitments {
            let provenance = match commitment {
                Commitment::Fact { provenance, .. } | Commitment::Rule { provenance, .. } => {
                    provenance
                }
            };
            match provenance {
                Provenance::Seed => {}
                Provenance::Witness { witness_id } => {
                    if !self.observations.contains_key(witness_id) {
                        return Err(CommitmentStateError::InvariantViolation(format!(
                            "commitment {:?} references missing witness {}",
                            id, witness_id
                        )));
                    }
                }
                Provenance::RuleApplication {
                    rule_id,
                    support_fact_id,
                } => {
                    if !matches!(self.commitments.get(rule_id), Some(Commitment::Rule { .. })) {
                        return Err(CommitmentStateError::InvariantViolation(format!(
                            "commitment {:?} references missing rule {:?}",
                            id, rule_id
                        )));
                    }
                    if !matches!(
                        self.commitments.get(support_fact_id),
                        Some(Commitment::Fact { .. })
                    ) {
                        return Err(CommitmentStateError::InvariantViolation(format!(
                            "commitment {:?} references missing support fact {:?}",
                            id, support_fact_id
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn insert_fact(
        &mut self,
        atom: Atom,
        provenance: Provenance,
    ) -> Result<CommitmentId, CommitmentStateError> {
        if self.fact_index.contains_key(&atom) {
            return Err(CommitmentStateError::DuplicateFact(
                atom.as_str().to_string(),
            ));
        }
        let id = self.allocate_id();
        self.fact_index.insert(atom.clone(), id);
        self.commitments
            .insert(id, Commitment::Fact { atom, provenance });
        Ok(id)
    }

    fn insert_rule(
        &mut self,
        rule: Rule,
        provenance: Provenance,
    ) -> Result<CommitmentId, CommitmentStateError> {
        if self.rule_index.contains_key(&rule) {
            return Err(CommitmentStateError::DuplicateRule(
                rule.antecedent.as_str().to_string(),
                rule.consequent.as_str().to_string(),
            ));
        }
        let id = self.allocate_id();
        self.rule_index.insert(rule.clone(), id);
        self.commitments
            .insert(id, Commitment::Rule { rule, provenance });
        Ok(id)
    }

    fn allocate_id(&mut self) -> CommitmentId {
        let id = CommitmentId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }
}

fn provenance_signature(provenance: &Provenance) -> String {
    match provenance {
        Provenance::Seed => "seed".into(),
        Provenance::Witness { witness_id } => format!("witness:{witness_id}"),
        Provenance::RuleApplication {
            rule_id,
            support_fact_id,
        } => format!("derive:{}:{}", rule_id.0, support_fact_id.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn rule(left: &str, right: &str) -> Rule {
        Rule::new(atom(left), atom(right)).unwrap()
    }

    #[test]
    fn witnessed_rule_changes_reachable_closure() {
        let mut state = ExecutableCommitmentState::new();
        state.seed_fact(atom("source")).unwrap();
        state.seed_rule(rule("source", "middle")).unwrap();
        state
            .add_observation(WitnessedRule {
                witness_id: 7,
                rule: rule("middle", "goal"),
            })
            .unwrap();

        let first = state.enabled_derivations().remove(0);
        state.apply_delta(first).unwrap();
        assert!(state.contains_fact(&atom("middle")));
        assert!(!state.contains_fact(&atom("goal")));

        state
            .apply_delta(StateDelta::CompileWitnessedRule {
                witness_id: 7,
                rule: rule("middle", "goal"),
            })
            .unwrap();
        let second = state.enabled_derivations().remove(0);
        state.apply_delta(second).unwrap();
        assert!(state.contains_fact(&atom("goal")));
        state.verify_invariants().unwrap();
    }

    #[test]
    fn mismatched_witness_is_rejected_without_state_mutation() {
        let mut state = ExecutableCommitmentState::new();
        state
            .add_observation(WitnessedRule {
                witness_id: 11,
                rule: rule("a", "b"),
            })
            .unwrap();
        let before = state.canonical_signature();
        let error = state
            .apply_delta(StateDelta::CompileWitnessedRule {
                witness_id: 11,
                rule: rule("a", "c"),
            })
            .unwrap_err();
        assert_eq!(error, CommitmentStateError::WitnessMismatch(11));
        assert_eq!(before, state.canonical_signature());
    }

    #[test]
    fn derived_fact_carries_rule_and_support_provenance() {
        let mut state = ExecutableCommitmentState::new();
        let support_id = state.seed_fact(atom("a")).unwrap();
        let rule_id = state.seed_rule(rule("a", "b")).unwrap();
        let derived_id = state
            .apply_delta(StateDelta::DeriveFact {
                rule_id,
                support_fact_id: support_id,
            })
            .unwrap();

        assert_eq!(
            state.commitment(derived_id),
            Some(&Commitment::Fact {
                atom: atom("b"),
                provenance: Provenance::RuleApplication {
                    rule_id,
                    support_fact_id: support_id,
                },
            })
        );
    }

    #[test]
    fn enabled_derivations_are_canonical() {
        let mut state = ExecutableCommitmentState::new();
        state.seed_fact(atom("root")).unwrap();
        state.seed_rule(rule("root", "zeta")).unwrap();
        state.seed_rule(rule("root", "alpha")).unwrap();

        let enabled = state.enabled_derivations();
        assert_eq!(enabled.len(), 2);
        let first_rule = match &enabled[0] {
            StateDelta::DeriveFact { rule_id, .. } => state.commitment(*rule_id).unwrap(),
            _ => unreachable!(),
        };
        assert!(matches!(
            first_rule,
            Commitment::Rule { rule, .. } if rule.consequent == atom("alpha")
        ));
    }
}
