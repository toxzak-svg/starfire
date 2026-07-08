use std::collections::HashMap;

use super::types::{Charge, ChargeKind, ChargeScope, ChargeSignature, Resolution};

/// A cognitive component capable of attempting to reduce or transform charge.
pub trait Resolver {
    fn name(&self) -> &str;

    fn resolve(&mut self, charge: &Charge) -> Resolution;
}

/// Coarse scope class used for routing generalization.
///
/// Exact [`ChargeScope`] remains on the charge for provenance. Resolver profiles
/// use this class when they need to generalize learned utility across instances,
/// such as from `Topic("dns")` to `Topic("mutex")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChargeScopeClass {
    Global,
    Reservoir,
    Topic,
    Belief,
    Goal,
    Component,
    Custom,
}

impl From<&ChargeScope> for ChargeScopeClass {
    fn from(scope: &ChargeScope) -> Self {
        match scope {
            ChargeScope::Global => Self::Global,
            ChargeScope::Reservoir => Self::Reservoir,
            ChargeScope::Topic(_) => Self::Topic,
            ChargeScope::Belief(_) => Self::Belief,
            ChargeScope::Goal(_) => Self::Goal,
            ChargeScope::Component(_) => Self::Component,
            ChargeScope::Custom(_) => Self::Custom,
        }
    }
}

/// Routing identity for a class of unresolved computation.
///
/// This deliberately omits the exact topic, belief ID, or component name. Those
/// belong to provenance. Routing needs a reusable key that can transfer utility
/// estimates to unseen charge instances.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChargeRoutingSignature {
    pub kind: ChargeKind,
    pub scope_class: ChargeScopeClass,
}

impl ChargeRoutingSignature {
    pub fn from_charge(charge: &Charge) -> Self {
        Self {
            kind: charge.kind.clone(),
            scope_class: ChargeScopeClass::from(&charge.scope),
        }
    }
}

/// Empirical resolver profile used to measure specialization and discharge
/// efficiency without assigning semantic roles in advance.
#[derive(Debug, Clone, Default)]
pub struct ResolverStats {
    attempts: u64,
    total_input: f64,
    total_discharged: f64,
    total_compute: u64,
    by_signature: HashMap<ChargeSignature, SignatureStats>,
    by_routing_signature: HashMap<ChargeRoutingSignature, SignatureStats>,
}

#[derive(Debug, Clone, Default)]
struct SignatureStats {
    attempts: u64,
    total_input: f64,
    total_discharged: f64,
    total_compute: u64,
}

impl SignatureStats {
    fn observe(&mut self, input: f32, discharged: f32, compute_cost: u64) {
        self.attempts = self.attempts.saturating_add(1);
        self.total_input += input as f64;
        self.total_discharged += discharged as f64;
        self.total_compute = self.total_compute.saturating_add(compute_cost);
    }

    fn discharge_rate(&self) -> f64 {
        if self.total_input <= f64::EPSILON {
            0.0
        } else {
            self.total_discharged / self.total_input
        }
    }

    fn efficiency(&self) -> f64 {
        if self.total_compute == 0 {
            0.0
        } else {
            self.total_discharged / self.total_compute as f64
        }
    }
}

impl ResolverStats {
    pub fn observe(&mut self, charge: &Charge, resolution: &Resolution) {
        self.attempts = self.attempts.saturating_add(1);

        let input = charge.magnitude.max(0.0);
        let discharged = resolution.discharged.clamp(0.0, input);

        self.total_input += input as f64;
        self.total_discharged += discharged as f64;
        self.total_compute = self.total_compute.saturating_add(resolution.compute_cost);

        self.by_signature
            .entry(charge.signature())
            .or_default()
            .observe(input, discharged, resolution.compute_cost);
        self.by_routing_signature
            .entry(ChargeRoutingSignature::from_charge(charge))
            .or_default()
            .observe(input, discharged, resolution.compute_cost);
    }

    pub fn attempts(&self) -> u64 {
        self.attempts
    }

    /// Fraction of observed incoming magnitude discharged.
    pub fn discharge_rate(&self) -> f64 {
        if self.total_input <= f64::EPSILON {
            0.0
        } else {
            self.total_discharged / self.total_input
        }
    }

    /// Discharged magnitude per unit of declared compute cost.
    pub fn discharge_efficiency(&self) -> f64 {
        if self.total_compute == 0 {
            0.0
        } else {
            self.total_discharged / self.total_compute as f64
        }
    }

    /// Exact-instance discharge rate, including precise scope identity.
    pub fn discharge_rate_for(&self, signature: &ChargeSignature) -> Option<f64> {
        self.by_signature
            .get(signature)
            .map(SignatureStats::discharge_rate)
    }

    /// Exact-instance discharge efficiency, including precise scope identity.
    pub fn efficiency_for(&self, signature: &ChargeSignature) -> Option<f64> {
        self.by_signature
            .get(signature)
            .map(SignatureStats::efficiency)
    }

    /// Generalized discharge rate for a reusable routing class.
    pub fn routing_discharge_rate_for(
        &self,
        signature: &ChargeRoutingSignature,
    ) -> Option<f64> {
        self.by_routing_signature
            .get(signature)
            .map(SignatureStats::discharge_rate)
    }

    /// Generalized discharge efficiency for a reusable routing class.
    pub fn routing_efficiency_for(
        &self,
        signature: &ChargeRoutingSignature,
    ) -> Option<f64> {
        self.by_routing_signature
            .get(signature)
            .map(SignatureStats::efficiency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_learn_signature_specific_discharge() {
        let charge = Charge::new(
            ChargeKind::Contradiction,
            vec![1.0],
            0.8,
            ChargeScope::Global,
        );
        let resolution = Resolution {
            discharged: 0.6,
            emitted: vec![],
            permitted_decay: 0.2,
            compute_cost: 3,
        };

        let mut stats = ResolverStats::default();
        stats.observe(&charge, &resolution);

        assert_eq!(stats.attempts(), 1);
        assert!((stats.discharge_rate() - 0.75).abs() < 1e-6);
        assert!((stats.discharge_rate_for(&charge.signature()).unwrap() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn routing_signature_generalizes_across_exact_topic_scopes() {
        let first = Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0],
            1.0,
            ChargeScope::Topic("dns".into()),
        );
        let second = Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0],
            1.0,
            ChargeScope::Topic("mutex".into()),
        );
        let mut stats = ResolverStats::default();
        stats.observe(
            &first,
            &Resolution {
                discharged: 0.8,
                emitted: vec![],
                permitted_decay: 0.0,
                compute_cost: 1,
            },
        );
        stats.observe(
            &second,
            &Resolution {
                discharged: 0.6,
                emitted: vec![],
                permitted_decay: 0.0,
                compute_cost: 1,
            },
        );

        let route = ChargeRoutingSignature::from_charge(&first);
        assert_eq!(route, ChargeRoutingSignature::from_charge(&second));
        assert!((stats.routing_discharge_rate_for(&route).unwrap() - 0.7).abs() < 1e-6);
        assert!(stats.efficiency_for(&first.signature()).is_some());
        assert!(stats.efficiency_for(&second.signature()).is_some());
    }
}
