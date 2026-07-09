use std::collections::HashMap;

use super::types::Charge;

/// Opaque identity for a machine-induced distinction.
///
/// Concept IDs deliberately carry no human semantic label. Meaning is grounded
/// in the predicate that selects observations and the measured utility earned by
/// retaining the distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConceptId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    AtLeast,
    AtMost,
}

/// Executable selector for a candidate concept.
///
/// The initial vocabulary is intentionally small and CHARGE-native. Future
/// experiments may add predicates over causal structure or other subsystem
/// state, but the first H4 probe must be able to replay every predicate against
/// historical charges deterministically.
#[derive(Debug, Clone, PartialEq)]
pub enum ConceptPredicate {
    Any,
    ResidualThreshold {
        dimension: usize,
        threshold: f32,
        direction: Direction,
    },
    PersistenceRange {
        min: u32,
        max: Option<u32>,
    },
    TraceContains {
        resolver: String,
    },
    And(Vec<ConceptPredicate>),
    Or(Vec<ConceptPredicate>),
    Not(Box<ConceptPredicate>),
}

impl ConceptPredicate {
    pub fn matches(&self, charge: &Charge) -> bool {
        match self {
            Self::Any => true,
            Self::ResidualThreshold {
                dimension,
                threshold,
                direction,
            } => charge.residual.get(*dimension).is_some_and(|value| match direction {
                Direction::AtLeast => *value >= *threshold,
                Direction::AtMost => *value <= *threshold,
            }),
            Self::PersistenceRange { min, max } => {
                charge.persistence >= *min && max.is_none_or(|upper| charge.persistence <= upper)
            }
            Self::TraceContains { resolver } => charge.trace.has_visited(resolver),
            Self::And(predicates) => predicates.iter().all(|predicate| predicate.matches(charge)),
            Self::Or(predicates) => predicates.iter().any(|predicate| predicate.matches(charge)),
            Self::Not(predicate) => !predicate.matches(charge),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConceptEvidence {
    pub observations: u64,
    pub positive_instances: Vec<u64>,
    pub negative_instances: Vec<u64>,
    /// Improvement measured only on observations withheld from proposal search.
    pub holdout_gain: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConceptUtility {
    pub routing_gain: f64,
    pub prediction_gain: f64,
    pub discharge_gain: f64,
    pub compute_gain: f64,
    pub recurrence_reduction: f64,
}

impl ConceptUtility {
    pub fn total_gain(&self) -> f64 {
        self.routing_gain
            + self.prediction_gain
            + self.discharge_gain
            + self.compute_gain
            + self.recurrence_reduction
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InducedConcept {
    pub id: ConceptId,
    pub parent: Option<ConceptId>,
    pub predicate: ConceptPredicate,
    pub evidence: ConceptEvidence,
    pub utility: ConceptUtility,
    pub generation: u64,
}

/// Structural mutations available to an ontology proposal search.
///
/// These describe candidate representation changes; they do not imply that a
/// mutation is correct or promoted. Promotion remains an empirical holdout gate.
#[derive(Debug, Clone, PartialEq)]
pub enum OntologyMutation {
    Split {
        parent: ConceptId,
        left: ConceptPredicate,
        right: ConceptPredicate,
    },
    Merge {
        members: Vec<ConceptId>,
    },
    Abstract {
        members: Vec<ConceptId>,
        predicate: ConceptPredicate,
    },
    Specialize {
        parent: ConceptId,
        predicate: ConceptPredicate,
    },
    Relate {
        source: ConceptId,
        relation: String,
        target: ConceptId,
    },
    Reify {
        relation_pattern: Vec<(ConceptId, String, ConceptId)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PromotionCriteria {
    pub min_observations: u64,
    pub min_holdout_gain: f64,
    pub min_total_utility_gain: f64,
}

impl Default for PromotionCriteria {
    fn default() -> Self {
        Self {
            min_observations: 32,
            min_holdout_gain: 0.05,
            min_total_utility_gain: 0.05,
        }
    }
}

/// Registry and empirical promotion gate for machine-induced concepts.
///
/// Proposal search intentionally lives outside this type. H4 can compare
/// multiple search strategies while sharing the same concept representation and
/// promotion contract.
#[derive(Debug, Clone)]
pub struct OntologyInducer {
    next_id: u64,
    generation: u64,
    criteria: PromotionCriteria,
    concepts: HashMap<ConceptId, InducedConcept>,
}

impl OntologyInducer {
    pub fn new(criteria: PromotionCriteria) -> Self {
        Self {
            next_id: 1,
            generation: 0,
            criteria,
            concepts: HashMap::new(),
        }
    }

    pub fn propose(
        &mut self,
        parent: Option<ConceptId>,
        predicate: ConceptPredicate,
        evidence: ConceptEvidence,
        utility: ConceptUtility,
    ) -> InducedConcept {
        let concept = InducedConcept {
            id: ConceptId(self.next_id),
            parent,
            predicate,
            evidence,
            utility,
            generation: self.generation,
        };
        self.next_id = self.next_id.saturating_add(1);
        concept
    }

    pub fn should_promote(&self, concept: &InducedConcept) -> bool {
        concept.evidence.observations >= self.criteria.min_observations
            && concept.evidence.holdout_gain >= self.criteria.min_holdout_gain
            && concept.utility.total_gain() >= self.criteria.min_total_utility_gain
    }

    pub fn promote(&mut self, concept: InducedConcept) -> bool {
        if !self.should_promote(&concept) {
            return false;
        }

        self.concepts.insert(concept.id, concept);
        true
    }

    pub fn concept(&self, id: ConceptId) -> Option<&InducedConcept> {
        self.concepts.get(&id)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &InducedConcept> {
        self.concepts.values()
    }

    pub fn advance_generation(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

    fn sample_charge() -> Charge {
        Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![0.2, 0.8],
            1.0,
            ChargeScope::Topic("held-out".into()),
        )
        .age()
        .age()
        .traced("memory")
    }

    #[test]
    fn predicates_replay_deterministically_against_charge_state() {
        let charge = sample_charge();
        let predicate = ConceptPredicate::And(vec![
            ConceptPredicate::ResidualThreshold {
                dimension: 1,
                threshold: 0.7,
                direction: Direction::AtLeast,
            },
            ConceptPredicate::PersistenceRange {
                min: 2,
                max: Some(3),
            },
            ConceptPredicate::TraceContains {
                resolver: "memory".into(),
            },
        ]);

        assert!(predicate.matches(&charge));
        assert!(predicate.matches(&charge));
    }

    #[test]
    fn promotion_requires_observations_holdout_gain_and_utility() {
        let mut inducer = OntologyInducer::new(PromotionCriteria {
            min_observations: 10,
            min_holdout_gain: 0.1,
            min_total_utility_gain: 0.2,
        });

        let weak = inducer.propose(
            None,
            ConceptPredicate::Any,
            ConceptEvidence {
                observations: 100,
                holdout_gain: 0.01,
                ..ConceptEvidence::default()
            },
            ConceptUtility {
                discharge_gain: 1.0,
                ..ConceptUtility::default()
            },
        );
        assert!(!inducer.promote(weak));

        let useful = inducer.propose(
            None,
            ConceptPredicate::ResidualThreshold {
                dimension: 1,
                threshold: 0.5,
                direction: Direction::AtLeast,
            },
            ConceptEvidence {
                observations: 100,
                holdout_gain: 0.25,
                ..ConceptEvidence::default()
            },
            ConceptUtility {
                routing_gain: 0.1,
                discharge_gain: 0.2,
                ..ConceptUtility::default()
            },
        );
        let id = useful.id;

        assert!(inducer.promote(useful));
        assert!(inducer.concept(id).is_some());
    }

    #[test]
    fn concept_ids_are_opaque_and_monotonic_across_generations() {
        let mut inducer = OntologyInducer::new(PromotionCriteria::default());
        let first = inducer.propose(
            None,
            ConceptPredicate::Any,
            ConceptEvidence::default(),
            ConceptUtility::default(),
        );
        inducer.advance_generation();
        let second = inducer.propose(
            Some(first.id),
            ConceptPredicate::Any,
            ConceptEvidence::default(),
            ConceptUtility::default(),
        );

        assert_eq!(first.id, ConceptId(1));
        assert_eq!(first.generation, 0);
        assert_eq!(second.id, ConceptId(2));
        assert_eq!(second.generation, 1);
    }
}
