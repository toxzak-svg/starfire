use std::collections::HashMap;

use super::types::{Charge, ChargeSignature, Resolution};

/// A cognitive component capable of attempting to reduce or transform charge.
pub trait Resolver {
    fn name(&self) -> &str;

    fn resolve(&mut self, charge: &Charge) -> Resolution;
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
}

#[derive(Debug, Clone, Default)]
struct SignatureStats {
    attempts: u64,
    total_input: f64,
    total_discharged: f64,
    total_compute: u64,
}

impl ResolverStats {
    pub fn observe(&mut self, charge: &Charge, resolution: &Resolution) {
        self.attempts = self.attempts.saturating_add(1);
        self.total_input += charge.magnitude.max(0.0) as f64;
        self.total_discharged += resolution.discharged.max(0.0) as f64;
        self.total_compute = self.total_compute.saturating_add(resolution.compute_cost);

        let stats = self.by_signature.entry(charge.signature()).or_default();
        stats.attempts = stats.attempts.saturating_add(1);
        stats.total_input += charge.magnitude.max(0.0) as f64;
        stats.total_discharged += resolution.discharged.max(0.0) as f64;
        stats.total_compute = stats.total_compute.saturating_add(resolution.compute_cost);
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

    pub fn discharge_rate_for(&self, signature: &ChargeSignature) -> Option<f64> {
        let stats = self.by_signature.get(signature)?;
        if stats.total_input <= f64::EPSILON {
            Some(0.0)
        } else {
            Some(stats.total_discharged / stats.total_input)
        }
    }

    pub fn efficiency_for(&self, signature: &ChargeSignature) -> Option<f64> {
        let stats = self.by_signature.get(signature)?;
        if stats.total_compute == 0 {
            Some(0.0)
        } else {
            Some(stats.total_discharged / stats.total_compute as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

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
}
