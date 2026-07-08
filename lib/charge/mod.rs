//! Routable Computational Tension (RCT) / CHARGE.
//!
//! CHARGE makes unresolved computational tension a first-class object. Cognitive
//! systems can emit a charge, attempt to resolve it, and account for what was
//! discharged, transformed into new charges, or explicitly decayed.
//!
//! This module intentionally contains only the primitive and accounting layer.
//! Runtime routing and component-specific emitters belong in later integration
//! changes so the primitive can be evaluated independently.

pub mod ledger;
pub mod resolver;
pub mod types;

pub use ledger::{ChargeLedger, ChargeLedgerError, LedgerSummary, ResolutionReceipt};
pub use resolver::{Resolver, ResolverStats};
pub use types::{Charge, ChargeKind, ChargeScope, ChargeSignature, ChargeTrace, Resolution};
