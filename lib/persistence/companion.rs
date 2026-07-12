//! Durable companion-state journal backed by Starfire's existing SQLite store.
//!
//! The journal is stored as one reserved identity value. Every commit replaces
//! that value through a SQLite compare-and-swap, so the expected version check
//! and write remain atomic across threads, Store instances, and processes.
//! Session-retained claims are removed from the durable checkpoint before the
//! write. Deletion commits clear the audit tail so removed raw values disappear
//! from the live journal payload.

use crate::companion_state::{
    ClaimStatus, CompanionError, CompanionEvent, CompanionState, CompanionTransition, Retention,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::Store;

const JOURNAL_KEY: &str = "__starfire_companion_journal_v1";
const JOURNAL_FORMAT_VERSION: u32 = 1;
const SESSION_CONTEST_REASON: &str = "contested session claim not durable";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CompanionJournal {
    format_version: u32,
    /// Latest durable projection. Its version tracks the full source state even
    /// when the corresponding transition contained only session-retained data.
    checkpoint: CompanionState,
    /// Durable mutation audit since the last deletion compaction. Recovery uses
    /// the checkpoint; the tail exists for audit/export and is never trusted as
    /// a second state authority.
    tail: Vec<CompanionEvent>,
    last_compacted_at_ms: Option<i64>,
}

impl Default for CompanionJournal {
    fn default() -> Self {
        Self {
            format_version: JOURNAL_FORMAT_VERSION,
            checkpoint: CompanionState::new(),
            tail: Vec::new(),
            last_compacted_at_ms: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionJournalStats {
    pub checkpoint_version: u64,
    pub current_version: u64,
    pub tail_events: usize,
    pub last_compacted_at_ms: Option<i64>,
}

/// Durable companion-state adapter over Starfire's existing `Store`.
///
/// The process-local lock avoids duplicate work within one adapter. Correctness
/// does not depend on it: `Store::compare_and_swap_identity` provides the real
/// cross-adapter and cross-process serialization boundary.
pub struct CompanionPersistence {
    store: Arc<Store>,
    commit_lock: Mutex<()>,
}

impl CompanionPersistence {
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            commit_lock: Mutex::new(()),
        }
    }

    #[must_use]
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    pub fn load_state(&self) -> Result<CompanionState, CompanionPersistenceError> {
        Ok(self.load_journal()?.checkpoint)
    }

    pub fn stats(&self) -> Result<CompanionJournalStats, CompanionPersistenceError> {
        let journal = self.load_journal()?;
        Ok(CompanionJournalStats {
            checkpoint_version: journal.checkpoint.version,
            current_version: journal.checkpoint.version,
            tail_events: journal.tail.len(),
            last_compacted_at_ms: journal.last_compacted_at_ms,
        })
    }

    /// Atomically persist one validated source-state transition.
    ///
    /// The resulting state is checked for the transition's observable effect,
    /// projected to durable-only state, encoded with its audit tail, and then
    /// installed only if the raw journal value still equals the value loaded at
    /// the start of this commit.
    pub fn commit(
        &self,
        expected_version: u64,
        transition: &CompanionTransition,
        resulting_state: &CompanionState,
        committed_at_ms: i64,
    ) -> Result<(), CompanionPersistenceError> {
        let _guard = self
            .commit_lock
            .lock()
            .map_err(|_| CompanionPersistenceError::LockPoisoned)?;
        let (mut journal, expected_raw) = self.load_journal_with_raw()?;

        if journal.checkpoint.version != expected_version {
            return Err(CompanionPersistenceError::VersionConflict {
                expected: expected_version,
                actual: journal.checkpoint.version,
            });
        }
        let required_version = expected_version
            .checked_add(1)
            .ok_or(CompanionPersistenceError::VersionOverflow)?;
        if transition.version != required_version || resulting_state.version != required_version {
            return Err(CompanionPersistenceError::TransitionVersionMismatch {
                expected: required_version,
                transition: transition.version,
                state: resulting_state.version,
            });
        }
        if !transition_is_reflected(&transition.event, resulting_state) {
            return Err(CompanionPersistenceError::StateMismatch);
        }

        let durable_state = durable_projection(resulting_state)?;
        if durable_state.version != required_version {
            return Err(CompanionPersistenceError::ProjectionVersionMismatch {
                expected: required_version,
                actual: durable_state.version,
            });
        }

        journal.checkpoint = durable_state;
        if matches!(&transition.event, CompanionEvent::ClaimDeleted { .. }) {
            journal.tail.clear();
            journal.last_compacted_at_ms = Some(committed_at_ms);
        } else if event_is_durable(&transition.event, resulting_state) {
            journal.tail.push(transition.event.clone());
        }

        let encoded = serde_json::to_string(&journal)?;
        let swapped = self
            .store
            .compare_and_swap_identity(
                JOURNAL_KEY,
                expected_raw.as_deref(),
                &encoded,
                committed_at_ms,
            )
            .map_err(CompanionPersistenceError::Store)?;
        if swapped {
            return Ok(());
        }

        let actual = self.load_journal()?.checkpoint.version;
        Err(CompanionPersistenceError::VersionConflict {
            expected: expected_version,
            actual,
        })
    }

    fn load_journal(&self) -> Result<CompanionJournal, CompanionPersistenceError> {
        self.load_journal_with_raw().map(|(journal, _)| journal)
    }

    fn load_journal_with_raw(
        &self,
    ) -> Result<(CompanionJournal, Option<String>), CompanionPersistenceError> {
        let encoded = self
            .store
            .get_identity(JOURNAL_KEY)
            .map_err(CompanionPersistenceError::Store)?;
        let journal = match encoded.as_deref() {
            Some(raw) => serde_json::from_str(raw)?,
            None => CompanionJournal::default(),
        };
        if journal.format_version != JOURNAL_FORMAT_VERSION {
            return Err(CompanionPersistenceError::UnsupportedFormat(
                journal.format_version,
            ));
        }
        Ok((journal, encoded))
    }
}

fn transition_is_reflected(event: &CompanionEvent, state: &CompanionState) -> bool {
    match event {
        CompanionEvent::ClaimRecorded {
            claim, observation, ..
        } => {
            state.claim(claim.id) == Some(claim)
                && state.observations().get(&observation.id) == Some(observation)
        }
        CompanionEvent::ObservationAttached {
            claim_id,
            observation,
            confidence_bps,
            updated_at_ms,
        } => state.claim(*claim_id).is_some_and(|claim| {
            state.observations().get(&observation.id) == Some(observation)
                && claim
                    .supporting_observation_ids
                    .contains(&observation.id)
                && claim.confidence_bps >= *confidence_bps
                && claim.updated_at_ms >= *updated_at_ms
        }),
        CompanionEvent::ClaimCorrected {
            previous_claim_id,
            replacement,
            observation,
        } => {
            state.claim(replacement.id) == Some(replacement)
                && state.observations().get(&observation.id) == Some(observation)
                && matches!(
                    state.claim(*previous_claim_id).map(|claim| &claim.status),
                    Some(ClaimStatus::Superseded { by }) if *by == replacement.id
                )
        }
        CompanionEvent::ClaimInvalidated {
            claim_id,
            reason,
            occurred_at_ms,
        } => state.claim(*claim_id).is_some_and(|claim| {
            matches!(&claim.status, ClaimStatus::Invalidated { reason: actual } if actual == reason)
                && claim.updated_at_ms >= *occurred_at_ms
        }),
        CompanionEvent::ClaimDeleted { claim_id, .. } => {
            state.claim(*claim_id).is_none()
                && state
                    .observations()
                    .values()
                    .all(|observation| observation.claim_id != *claim_id)
        }
    }
}

fn event_is_durable(event: &CompanionEvent, state: &CompanionState) -> bool {
    match event {
        CompanionEvent::ClaimRecorded { claim, .. } => claim.retention != Retention::Session,
        CompanionEvent::ObservationAttached { claim_id, .. }
        | CompanionEvent::ClaimInvalidated { claim_id, .. } => state
            .claim(*claim_id)
            .is_some_and(|claim| claim.retention != Retention::Session),
        CompanionEvent::ClaimCorrected { replacement, .. } => {
            replacement.retention != Retention::Session
        }
        CompanionEvent::ClaimDeleted { .. } => true,
    }
}

/// Produce a durable checkpoint while preserving the source state's version and
/// identifier counters. Serde is already the journal's compatibility boundary;
/// projecting the serialized shape lets this adapter exclude private fields
/// without expanding the public mutation API of `CompanionState`.
fn durable_projection(
    state: &CompanionState,
) -> Result<CompanionState, CompanionPersistenceError> {
    let mut encoded = serde_json::to_value(state)?;
    let root = encoded
        .as_object_mut()
        .ok_or(CompanionPersistenceError::InvalidStateShape("root"))?;

    let session_claim_ids = root
        .get("claims")
        .and_then(Value::as_object)
        .ok_or(CompanionPersistenceError::InvalidStateShape("claims"))?
        .iter()
        .filter_map(|(id, claim)| {
            (claim.get("retention").and_then(Value::as_str) == Some("Session"))
                .then(|| id.parse::<u64>().ok())
                .flatten()
        })
        .collect::<BTreeSet<_>>();

    if session_claim_ids.is_empty() {
        return serde_json::from_value(encoded).map_err(CompanionPersistenceError::Serialization);
    }

    let mut restored_active = Vec::<(String, u64)>::new();
    {
        let claims = root
            .get_mut("claims")
            .and_then(Value::as_object_mut)
            .ok_or(CompanionPersistenceError::InvalidStateShape("claims"))?;

        for (id, claim) in claims.iter_mut() {
            let Ok(claim_id) = id.parse::<u64>() else {
                return Err(CompanionPersistenceError::InvalidStateShape("claim id"));
            };
            if session_claim_ids.contains(&claim_id) {
                continue;
            }

            let status = claim.get("status");
            let superseded_by_session = status
                .and_then(Value::as_object)
                .and_then(|status| status.get("Superseded"))
                .and_then(Value::as_object)
                .and_then(|payload| payload.get("by"))
                .and_then(Value::as_u64)
                .is_some_and(|by| session_claim_ids.contains(&by));
            if superseded_by_session {
                claim["status"] = Value::String("Active".to_owned());
                let key = claim
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or(CompanionPersistenceError::InvalidStateShape("claim key"))?;
                restored_active.push((key.to_owned(), claim_id));
                continue;
            }

            let contested_with_session = status
                .and_then(Value::as_object)
                .and_then(|status| status.get("Contested"))
                .and_then(Value::as_object)
                .and_then(|payload| payload.get("with"))
                .and_then(Value::as_u64)
                .is_some_and(|with| session_claim_ids.contains(&with));
            if contested_with_session {
                claim["status"] = json!({
                    "Invalidated": { "reason": SESSION_CONTEST_REASON }
                });
            }
        }

        claims.retain(|id, _| {
            id.parse::<u64>()
                .ok()
                .is_none_or(|claim_id| !session_claim_ids.contains(&claim_id))
        });
    }

    root.get_mut("observations")
        .and_then(Value::as_object_mut)
        .ok_or(CompanionPersistenceError::InvalidStateShape("observations"))?
        .retain(|_, observation| {
            observation
                .get("claim_id")
                .and_then(Value::as_u64)
                .is_none_or(|claim_id| !session_claim_ids.contains(&claim_id))
        });

    let active_by_key = root
        .get_mut("active_by_key")
        .and_then(Value::as_object_mut)
        .ok_or(CompanionPersistenceError::InvalidStateShape("active_by_key"))?;
    active_by_key.retain(|_, claim_id| {
        claim_id
            .as_u64()
            .is_none_or(|claim_id| !session_claim_ids.contains(&claim_id))
    });
    for (key, claim_id) in restored_active {
        active_by_key.insert(key, Value::from(claim_id));
    }

    serde_json::from_value(encoded).map_err(CompanionPersistenceError::Serialization)
}

#[derive(Debug, Error)]
pub enum CompanionPersistenceError {
    #[error("Starfire store operation failed: {0}")]
    Store(#[source] anyhow::Error),
    #[error("companion journal serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("companion event validation failed: {0}")]
    Replay(#[from] CompanionError),
    #[error("companion journal lock was poisoned")]
    LockPoisoned,
    #[error("companion journal version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error(
        "transition version mismatch: expected {expected}, transition {transition}, state {state}"
    )]
    TransitionVersionMismatch {
        expected: u64,
        transition: u64,
        state: u64,
    },
    #[error("companion transition is not reflected in the supplied resulting state")]
    StateMismatch,
    #[error("durable projection version mismatch: expected {expected}, actual {actual}")]
    ProjectionVersionMismatch { expected: u64, actual: u64 },
    #[error("unsupported companion journal format version {0}")]
    UnsupportedFormat(u32),
    #[error("invalid serialized companion-state shape at {0}")]
    InvalidStateShape(&'static str),
    #[error("companion journal version overflow")]
    VersionOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_state::{ClaimInput, ClaimSource, Retention, Sensitivity};
    use std::sync::{Arc, Barrier};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "starfire-companion-{label}-{}-{nonce}.sqlite",
            std::process::id()
        ))
    }

    fn claim(key: &str, value: &str, at: u64, retention: Retention) -> ClaimInput {
        ClaimInput {
            key: key.to_owned(),
            value: value.to_owned(),
            source: ClaimSource::UserStatement,
            confidence_bps: 10_000,
            sensitivity: Sensitivity::Personal,
            retention,
            observed_at_ms: at,
        }
    }

    #[test]
    fn journal_recovers_state_after_store_reopen() {
        let path = temporary_path("reopen");
        let mut state = CompanionState::new();
        let transition = state
            .record_claim(0, claim("editor", "helix", 10, Retention::Durable))
            .unwrap();

        {
            let store = Arc::new(Store::open(&path).unwrap());
            let persistence = CompanionPersistence::new(store);
            persistence.commit(0, &transition, &state, 10).unwrap();
            assert_eq!(persistence.load_state().unwrap(), state);
        }

        {
            let store = Arc::new(Store::open(&path).unwrap());
            let persistence = CompanionPersistence::new(store);
            assert_eq!(persistence.load_state().unwrap(), state);
            assert_eq!(persistence.stats().unwrap().tail_events, 1);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn two_store_instances_cannot_overwrite_the_same_version() {
        let path = temporary_path("cas");
        let persistence_a = Arc::new(CompanionPersistence::new(Arc::new(
            Store::open(&path).unwrap(),
        )));
        let persistence_b = Arc::new(CompanionPersistence::new(Arc::new(
            Store::open(&path).unwrap(),
        )));

        let mut state_a = CompanionState::new();
        let transition_a = state_a
            .record_claim(0, claim("winner", "a", 10, Retention::Durable))
            .unwrap();
        let mut state_b = CompanionState::new();
        let transition_b = state_b
            .record_claim(0, claim("winner", "b", 10, Retention::Durable))
            .unwrap();

        let barrier = Arc::new(Barrier::new(3));
        let result_a = {
            let persistence = persistence_a.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                persistence.commit(0, &transition_a, &state_a, 10)
            })
        };
        let result_b = {
            let persistence = persistence_b.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                persistence.commit(0, &transition_b, &state_b, 10)
            })
        };
        barrier.wait();

        let outcomes = [result_a.join().unwrap(), result_b.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|result| matches!(result, Err(CompanionPersistenceError::VersionConflict { expected: 0, actual: 1 })))
                .count(),
            1
        );
        assert_eq!(persistence_a.load_state().unwrap().version, 1);

        drop(persistence_a);
        drop(persistence_b);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn session_claims_do_not_survive_restart_or_enter_audit_tail() {
        let path = temporary_path("session-filter");
        let mut state = CompanionState::new();
        let durable = state
            .record_claim(0, claim("editor", "helix", 10, Retention::Durable))
            .unwrap();
        let session = state
            .record_claim(
                durable.version,
                claim("session secret", "do-not-persist", 11, Retention::Session),
            )
            .unwrap();

        {
            let persistence = CompanionPersistence::new(Arc::new(Store::open(&path).unwrap()));
            let mut durable_only = CompanionState::new();
            let durable_transition = durable_only
                .record_claim(0, claim("editor", "helix", 10, Retention::Durable))
                .unwrap();
            persistence
                .commit(0, &durable_transition, &durable_only, 10)
                .unwrap();
            persistence
                .commit(durable.version, &session, &state, 11)
                .unwrap();
            assert_eq!(persistence.stats().unwrap().tail_events, 1);
            let raw = persistence.store().get_identity(JOURNAL_KEY).unwrap().unwrap();
            assert!(!raw.contains("do-not-persist"));
        }

        {
            let persistence = CompanionPersistence::new(Arc::new(Store::open(&path).unwrap()));
            let recovered = persistence.load_state().unwrap();
            assert_eq!(recovered.version, 2);
            assert_eq!(
                recovered.active_claim("editor", 12, true).unwrap().value,
                "helix"
            );
            assert!(recovered.active_claim("session secret", 12, true).is_none());
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn deletion_compacts_raw_claim_value_out_of_journal() {
        let path = temporary_path("delete");
        let store = Arc::new(Store::open(&path).unwrap());
        let persistence = CompanionPersistence::new(store.clone());
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, claim("private note", "secret-value", 10, Retention::Durable))
            .unwrap();
        persistence.commit(0, &first, &state, 10).unwrap();

        let deleted = state
            .delete_claim(state.version, first.claim_id.unwrap(), 20)
            .unwrap();
        persistence.commit(1, &deleted, &state, 20).unwrap();

        let raw = store.get_identity(JOURNAL_KEY).unwrap().unwrap();
        assert!(!raw.contains("secret-value"));
        assert_eq!(persistence.load_state().unwrap(), state);
        let stats = persistence.stats().unwrap();
        assert_eq!(stats.checkpoint_version, 2);
        assert_eq!(stats.current_version, 2);
        assert_eq!(stats.tail_events, 0);
        assert_eq!(stats.last_compacted_at_ms, Some(20));

        let _ = std::fs::remove_file(path);
    }
}
