//! STLM L1-C post-response shadow observation for verified improvisation.
//!
//! The production HTTP response is finalized before this module is called. The
//! shadow receives only the typed ΩV1-F2 bundle and a frozen response-byte
//! fingerprint. It executes the L1-A selector, independently verifies the
//! selected surface, compares it with the grammar-v2 neutral control, and
//! records bounded metadata only. Candidate text is never returned or written
//! to the ledger. Failures, timeouts, trace updates, and ledger errors remain
//! isolated from the live response path.

include!("stlm_l1c_shadow/part1.rs");
include!("stlm_l1c_shadow/part2.rs");
include!("stlm_l1c_shadow/part3.rs");
include!("stlm_l1c_shadow/part4.rs");
include!("stlm_l1c_shadow/tests.rs");
