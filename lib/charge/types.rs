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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
        self.id == other.id
    }
}

impl Eq for Charge {}

impl Hash for Charge {
    fn hash<H: Hasher>(&self, state: &mut H) {
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
