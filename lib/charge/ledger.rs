use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use super::types::{Charge, Resolution};

const DEFAULT_TOLERANCE: f32 = 1e-4;

/// Accounting failure for a CHARGE resolution attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum ChargeLedgerError {
    UnknownCharge(u64),
    DuplicateCharge(u64),
    NonFiniteAccounting,
    OverAccounted {
        incoming: f32,
        accounted: f32,
        tolerance: f32,
    },
}

impl fmt::Display for ChargeLedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCharge(id) => write!(f, "unknown charge id {id}"),
            Self::DuplicateCharge(id) => write!(f, "duplicate charge id {id}"),
            Self::NonFiniteAccounting => {
                write!(f, "charge accounting values must be finite and non-negative")
            }
            Self::OverAccounted {
                incoming,
                accounted,
                tolerance,
            } => write!(
                f,
                "resolution over-accounted charge: incoming={incoming:.6}, accounted={accounted:.6}, tolerance={tolerance:.6}",
            ),
        }
    }
}

impl Error for ChargeLedgerError {}

#[derive(Debug, Clone)]
struct LedgerEntry {
    charge: Charge,
    remaining: f32,
}

/// Receipt for one accepted accounting operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionReceipt {
    pub charge_id: u64,
    pub incoming: f32,
    pub discharged: f32,
    pub emitted: f32,
    pub permitted_decay: f32,
    /// Magnitude left unresolved on the parent after this attempt.
    pub remaining: f32,
    pub compute_cost: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LedgerSummary {
    pub charges_created: u64,
    pub resolutions_recorded: u64,
    pub total_generated: f64,
    pub total_discharged: f64,
    pub total_emitted: f64,
    pub total_decay: f64,
    /// Sum of the parent's remaining magnitude after each recorded resolution attempt
    /// (i.e. repeated attempts on the same charge contribute multiple times).
    pub total_remaining_after_attempts: f64,
    pub total_compute: u64,
}

/// Provenance and conservation ledger for unresolved computational tension.
#[derive(Debug, Clone)]
pub struct ChargeLedger {
    next_id: u64,
    tolerance: f32,
    entries: HashMap<u64, LedgerEntry>,
    summary: LedgerSummary,
}

impl Default for ChargeLedger {
    fn default() -> Self {
        Self::new(DEFAULT_TOLERANCE)
    }
}

impl ChargeLedger {
    pub fn new(tolerance: f32) -> Self {
        let tolerance = if tolerance.is_finite() {
            tolerance.max(0.0)
        } else {
            DEFAULT_TOLERANCE
        };

        Self {
            next_id: 1,
            tolerance,
            entries: HashMap::new(),
            summary: LedgerSummary::default(),
        }
    }

    /// Assign an identity and register a newly generated charge.
    pub fn issue(&mut self, mut charge: Charge) -> Result<Charge, ChargeLedgerError> {
        if !charge.magnitude.is_finite() || charge.magnitude < 0.0 {
            return Err(ChargeLedgerError::NonFiniteAccounting);
        }

        charge.id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(ChargeLedgerError::DuplicateCharge(charge.id))?;

        if self.entries.contains_key(&charge.id) {
            return Err(ChargeLedgerError::DuplicateCharge(charge.id));
        }

        self.summary.charges_created = self.summary.charges_created.saturating_add(1);
        self.summary.total_generated += charge.magnitude as f64;
        self.entries.insert(
            charge.id,
            LedgerEntry {
                charge: charge.clone(),
                remaining: charge.magnitude,
            },
        );
        Ok(charge)
    }

    /// Validate and record one attempt. Emitted charges are registered with fresh IDs.
    ///
    /// CHARGE uses approximate conservation: a resolver may discharge, transform, or
    /// explicitly decay incoming magnitude, but may not create more accounted output
    /// than remains on the parent within the configured tolerance.
    pub fn record_resolution(
        &mut self,
        charge_id: u64,
        mut resolution: Resolution,
    ) -> Result<(ResolutionReceipt, Vec<Charge>), ChargeLedgerError> {
        let incoming = self
            .entries
            .get(&charge_id)
            .ok_or(ChargeLedgerError::UnknownCharge(charge_id))?
            .remaining;

        let emitted_total: f32 = resolution
            .emitted
            .iter()
            .map(|charge| charge.magnitude)
            .sum();
        let accounted = resolution.discharged + resolution.permitted_decay + emitted_total;

        if !incoming.is_finite()
            || !resolution.discharged.is_finite()
            || !resolution.permitted_decay.is_finite()
            || !emitted_total.is_finite()
            || resolution.discharged < 0.0
            || resolution.permitted_decay < 0.0
            || resolution
                .emitted
                .iter()
                .any(|charge| !charge.magnitude.is_finite() || charge.magnitude < 0.0)
        {
            return Err(ChargeLedgerError::NonFiniteAccounting);
        }

        if accounted > incoming + self.tolerance {
            return Err(ChargeLedgerError::OverAccounted {
                incoming,
                accounted,
                tolerance: self.tolerance,
            });
        }

        let remaining = (incoming - accounted).max(0.0);
        let mut issued = Vec::with_capacity(resolution.emitted.len());
        for emitted in resolution.emitted.drain(..) {
            issued.push(self.issue(emitted)?);
        }

        if let Some(entry) = self.entries.get_mut(&charge_id) {
            entry.remaining = remaining;
        }

        self.summary.resolutions_recorded = self.summary.resolutions_recorded.saturating_add(1);
        self.summary.total_discharged += resolution.discharged as f64;
        self.summary.total_emitted += emitted_total as f64;
        self.summary.total_decay += resolution.permitted_decay as f64;
        self.summary.total_remaining_after_attempts += remaining as f64;
        self.summary.total_compute = self.summary.total_compute.saturating_add(resolution.compute_cost);

        let receipt = ResolutionReceipt {
            charge_id,
            incoming,
            discharged: resolution.discharged,
            emitted: emitted_total,
            permitted_decay: resolution.permitted_decay,
            remaining,
            compute_cost: resolution.compute_cost,
        };

        Ok((receipt, issued))
    }

    pub fn get(&self, charge_id: u64) -> Option<&Charge> {
        self.entries.get(&charge_id).map(|entry| &entry.charge)
    }

    pub fn remaining(&self, charge_id: u64) -> Option<f32> {
        self.entries.get(&charge_id).map(|entry| entry.remaining)
    }

    pub fn is_resolved(&self, charge_id: u64) -> Option<bool> {
        self.remaining(charge_id)
            .map(|remaining| remaining <= self.tolerance)
    }

    pub fn summary(&self) -> &LedgerSummary {
        &self.summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

    fn charge(magnitude: f32) -> Charge {
        Charge::new(
            ChargeKind::EpistemicGap,
            vec![magnitude],
            magnitude,
            ChargeScope::Topic("test".to_string()),
        )
    }

    #[test]
    fn ledger_accepts_conserved_resolution_and_issues_children() {
        let mut ledger = ChargeLedger::default();
        let parent = ledger.issue(charge(0.8)).unwrap();
        let resolution = Resolution {
            discharged: 0.5,
            emitted: vec![charge(0.2)],
            permitted_decay: 0.1,
            compute_cost: 7,
        };

        let (receipt, children) = ledger.record_resolution(parent.id, resolution).unwrap();

        assert_eq!(children.len(), 1);
        assert!(children[0].id > parent.id);
        assert!(receipt.remaining <= 1e-4);
        assert_eq!(ledger.is_resolved(parent.id), Some(true));
    }

    #[test]
    fn ledger_rejects_charge_creation_by_accounting() {
        let mut ledger = ChargeLedger::default();
        let parent = ledger.issue(charge(0.8)).unwrap();
        let resolution = Resolution {
            discharged: 0.7,
            emitted: vec![charge(0.3)],
            permitted_decay: 0.0,
            compute_cost: 1,
        };

        let error = ledger.record_resolution(parent.id, resolution).unwrap_err();
        assert!(matches!(error, ChargeLedgerError::OverAccounted { .. }));
    }

    #[test]
    fn unresolved_magnitude_remains_visible() {
        let mut ledger = ChargeLedger::default();
        let parent = ledger.issue(charge(1.0)).unwrap();
        let resolution = Resolution {
            discharged: 0.25,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: 2,
        };

        let (receipt, _) = ledger.record_resolution(parent.id, resolution).unwrap();
        assert!((receipt.remaining - 0.75).abs() < 1e-6);
        assert_eq!(ledger.is_resolved(parent.id), Some(false));
    }

    #[test]
    fn repeated_attempts_only_consume_remaining_parent_charge() {
        let mut ledger = ChargeLedger::default();
        let parent = ledger.issue(charge(1.0)).unwrap();

        let (first, _) = ledger
            .record_resolution(
                parent.id,
                Resolution {
                    discharged: 0.25,
                    emitted: vec![],
                    permitted_decay: 0.0,
                    compute_cost: 1,
                },
            )
            .unwrap();
        let (second, _) = ledger
            .record_resolution(
                parent.id,
                Resolution {
                    discharged: 0.5,
                    emitted: vec![],
                    permitted_decay: 0.0,
                    compute_cost: 1,
                },
            )
            .unwrap();

        assert!((first.incoming - 1.0).abs() < 1e-6);
        assert!((first.remaining - 0.75).abs() < 1e-6);
        assert!((second.incoming - 0.75).abs() < 1e-6);
        assert!((second.remaining - 0.25).abs() < 1e-6);
        assert_eq!(ledger.remaining(parent.id), Some(0.25));
    }
}
