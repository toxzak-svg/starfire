//! Durable companion-state journal backed by Starfire's existing SQLite store.
//!
//! The complete journal is stored as one reserved identity value. Updating the
//! checkpoint and event tail therefore uses one SQLite statement under the
//! existing `Store` mutex instead of introducing a second database or runtime.
//! Deletion commits compact the journal into a fresh checkpoint so previously
//! stored claim values are physically removed from the active journal payload.

use crate::companion_state::{
    CompanionError, CompanionEvent, CompanionState, CompanionTransition,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::Store;

const JOURNAL_KEY: &str = "__starfire_companion_journal_v1";
const JOURNAL_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CompanionJournal {
    format_version: u32,
    checkpoint: CompanionState,
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

/// Process-local serialization boundary for durable companion mutations.
///
/// Callers should share one `Arc<CompanionPersistence>` per Starfire `Store`.
/// The adapter enforces optimistic versions before replacing the single atomic
/// journal value.
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
        let journal = self.load_journal()?;
        replay_journal(&journal)
    }

    pub fn stats(&self) -> Result<CompanionJournalStats, CompanionPersistenceError> {
        let journal = self.load_journal()?;
        let state = replay_journal(&journal)?;
        Ok(CompanionJournalStats {
            checkpoint_version: journal.checkpoint.version,
            current_version: state.version,
            tail_events: journal.tail.len(),
            last_compacted_at_ms: journal.last_compacted_at_ms,
        })
    }

    /// Atomically append one validated transition to the durable journal.
    ///
    /// The transition is independently replayed against the persisted state.
    /// Persistence is rejected unless that replay exactly equals the caller's
    /// resulting state. A deletion replaces the journal checkpoint and clears
    /// the prior event tail, removing historical raw claim values from the JSON
    /// payload used by the live store.
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
        let mut journal = self.load_journal()?;
        let current = replay_journal(&journal)?;

        if current.version != expected_version {
            return Err(CompanionPersistenceError::VersionConflict {
                expected: expected_version,
                actual: current.version,
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

        let mut replayed = current;
        replayed.apply_event(expected_version, transition.event.clone())?;
        if replayed != *resulting_state {
            return Err(CompanionPersistenceError::StateMismatch);
        }

        if matches!(&transition.event, CompanionEvent::ClaimDeleted { .. }) {
            journal.checkpoint = resulting_state.clone();
            journal.tail.clear();
            journal.last_compacted_at_ms = Some(committed_at_ms);
        } else {
            journal.tail.push(transition.event.clone());
        }

        let encoded = serde_json::to_string(&journal)?;
        self.store
            .put_identity(JOURNAL_KEY, &encoded, committed_at_ms)
            .map_err(CompanionPersistenceError::Store)
    }

    fn load_journal(&self) -> Result<CompanionJournal, CompanionPersistenceError> {
        let encoded = self
            .store
            .get_identity(JOURNAL_KEY)
            .map_err(CompanionPersistenceError::Store)?;
        let journal = match encoded {
            Some(encoded) => serde_json::from_str(&encoded)?,
            None => CompanionJournal::default(),
        };
        if journal.format_version != JOURNAL_FORMAT_VERSION {
            return Err(CompanionPersistenceError::UnsupportedFormat(
                journal.format_version,
            ));
        }
        Ok(journal)
    }
}

fn replay_journal(
    journal: &CompanionJournal,
) -> Result<CompanionState, CompanionPersistenceError> {
    let mut state = journal.checkpoint.clone();
    for event in &journal.tail {
        let expected_version = state.version;
        state.apply_event(expected_version, event.clone())?;
    }
    Ok(state)
}

#[derive(Debug, Error)]
pub enum CompanionPersistenceError {
    #[error("Starfire store operation failed: {0}")]
    Store(#[source] anyhow::Error),
    #[error("companion journal serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("companion event replay failed: {0}")]
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
    #[error("replayed companion transition did not equal the supplied resulting state")]
    StateMismatch,
    #[error("unsupported companion journal format version {0}")]
    UnsupportedFormat(u32),
    #[error("companion journal version overflow")]
    VersionOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_state::{ClaimInput, ClaimSource, Retention, Sensitivity};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_path() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "starfire-companion-persistence-{}-{nonce}.sqlite",
            std::process::id()
        ))
    }

    fn user_claim(key: &str, value: &str, at: u64) -> ClaimInput {
        ClaimInput {
            key: key.to_owned(),
            value: value.to_owned(),
            source: ClaimSource::UserStatement,
            confidence_bps: 10_000,
            sensitivity: Sensitivity::Personal,
            retention: Retention::Durable,
            observed_at_ms: at,
        }
    }

    #[test]
    fn journal_recovers_state_after_store_reopen() {
        let path = temporary_path();
        let mut state = CompanionState::new();
        let transition = state
            .record_claim(0, user_claim("editor", "helix", 10))
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
    fn stale_commit_is_rejected_without_replacing_state() {
        let path = temporary_path();
        let store = Arc::new(Store::open(&path).unwrap());
        let persistence = CompanionPersistence::new(store);
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, user_claim("language", "rust", 10))
            .unwrap();
        persistence.commit(0, &first, &state, 10).unwrap();

        let stale_state = CompanionState::new();
        let error = persistence
            .commit(0, &first, &stale_state, 11)
            .unwrap_err();
        assert!(matches!(
            error,
            CompanionPersistenceError::VersionConflict {
                expected: 0,
                actual: 1
            }
        ));
        assert_eq!(persistence.load_state().unwrap(), state);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn deletion_compacts_raw_claim_value_out_of_journal() {
        let path = temporary_path();
        let store = Arc::new(Store::open(&path).unwrap());
        let persistence = CompanionPersistence::new(store.clone());
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, user_claim("private note", "secret-value", 10))
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
