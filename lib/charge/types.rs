use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Broad class of unresolved computation represented by a charge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChargeKind {
    EpistemicGap,
    PredictionResidual,
    Contradiction,
    ConstraintViolation,
    CausalAmbiguity,
    TemporalMismatch,
    GoalTension,
    Custom(String),
}

/// Cognitive region or object that a charge concerns.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChargeScope {
    Global,
    Reservoir,
    Topic(String),
    Belief(String),
    Goal(String),
    Component(String),
    Custom(String),
}

/// Compact routing key. The residual vector is intentionally excluded so
/// signatures remain stable across nearby instances of the same problem class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChargeSignature {
    pub kind: ChargeKind,
    pub scope: ChargeScope,
}

/// Provenance for the path already attempted by a charge.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChargeTrace {
    pub resolvers: Vec<String>,
}

impl ChargeTrace {
    pub fn with_attempt(&self, resolver: impl Into<String>) -> Self {
        let mut next = self.clone();
        next.resolvers.push(resolver.into());
        next
    }

    pub fn has_visited(&self, resolver: &str) -> bool {
        self.resolvers.iter().any(|name| name == resolver)
    }
}

/// A persistent, routable unit of unresolved computational tension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charge {
    pub id: u64,
    pub kind: ChargeKind,
    pub residual: Vec<f32>,
    pub magnitude: f32,
    pub persistence: u32,
    pub scope: ChargeScope,
    pub trace: ChargeTrace,
}

impl Charge {
    pub fn new(
        kind: ChargeKind,
        residual: Vec<f32>,
        magnitude: f32,
        scope: ChargeScope,
    ) -> Self {
        Self {
            id: 0,
            kind,
            residual,
            magnitude: magnitude.max(0.0),
            persistence: 0,
            scope,
            trace: ChargeTrace::default(),
        }
    }

    pub fn signature(&self) -> ChargeSignature {
        ChargeSignature {
            kind: self.kind.clone(),
            scope: self.scope.clone(),
        }
    }

    pub fn age(mut self) -> Self {
        self.persistence = self.persistence.saturating_add(1);
        self
    }

    pub fn traced(mut self, resolver: impl Into<String>) -> Self {
        self.trace = self.trace.with_attempt(resolver);
        self
    }
}

impl PartialEq for Charge {
    fn eq(&self, other: &Self) -> bool {
        if self.id == 0 && other.id == 0 {
            return self.kind == other.kind
                && self.scope == other.scope
                && self.persistence == other.persistence
                && self.trace == other.trace
                && self.magnitude.to_bits() == other.magnitude.to_bits()
                && self.residual.len() == other.residual.len()
                && self
                    .residual
                    .iter()
                    .zip(other.residual.iter())
                    .all(|(left, right)| left.to_bits() == right.to_bits());
        }

        self.id == other.id
    }
}

impl Eq for Charge {}

impl Hash for Charge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.id == 0 {
            0u8.hash(state);
            self.kind.hash(state);
            self.scope.hash(state);
            self.persistence.hash(state);
            self.trace.hash(state);
            self.magnitude.to_bits().hash(state);
            self.residual.len().hash(state);
            for value in &self.residual {
                value.to_bits().hash(state);
            }
            return;
        }

        self.id.hash(state);
    }
}

/// Accounted result of one resolver attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub discharged: f32,
    pub emitted: Vec<Charge>,
    pub permitted_decay: f32,
    pub compute_cost: u64,
}

impl Resolution {
    pub fn empty(compute_cost: u64) -> Self {
        Self {
            discharged: 0.0,
            emitted: Vec::new(),
            permitted_decay: 0.0,
            compute_cost,
        }
    }

    /// Returns the total accounted output of this resolution.
    ///
    /// This method is intentionally **more lenient** than the `ChargeLedger`
    /// validation logic: it clamps negative values to zero instead of
    /// rejecting them. It is intended for diagnostic/metrics use and should
    /// **not** be used as a validity check against `ChargeLedger`.
    pub fn accounted_output(&self) -> f32 {
        self.discharged.max(0.0)
            + self.permitted_decay.max(0.0)
            + self
                .emitted
                .iter()
                .map(|charge| charge.magnitude.max(0.0))
                .sum::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{Charge, ChargeKind, ChargeScope};

    #[test]
    fn unissued_charges_with_different_payloads_do_not_collide() {
        let first = Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0],
            1.0,
            ChargeScope::Topic("a".into()),
        );
        let second = Charge::new(
            ChargeKind::Contradiction,
            vec![2.0],
            2.0,
            ChargeScope::Topic("b".into()),
        );

        assert_ne!(first, second);

        let mut set = HashSet::new();
        set.insert(first);
        set.insert(second);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn issued_charges_remain_id_comparable() {
        let mut first = Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0],
            1.0,
            ChargeScope::Global,
        );
        first.id = 42;
        let mut second = Charge::new(
            ChargeKind::Contradiction,
            vec![2.0],
            2.0,
            ChargeScope::Topic("topic".into()),
        );
        second.id = 42;

        assert_eq!(first, second);

        let mut set = HashSet::new();
        set.insert(first);
        set.insert(second);
        assert_eq!(set.len(), 1);
    }
}
