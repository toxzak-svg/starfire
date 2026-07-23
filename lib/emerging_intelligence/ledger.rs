//! EI-0C append-only episode ledger and fresh-state replay.
//!
//! This module is offline-only and feature-gated. It stores canonical EI-0A
//! sealed episodes in a bounded hash chain, validates the complete ledger, and
//! reconstructs the same ordered episodes from canonical bytes. It has no live
//! persistence, `Runtime::chat()`, response, routing, tool, ontology, learning-
//! update, or autonomous-action authority.

use super::{
    AuthoritySnapshot, EpisodeContractError, SealedCognitiveEpisode, EI_0A_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const EI_0C_LEDGER_SCHEMA_VERSION: u16 = 1;
pub const EI_0C_LEDGER_ID: &str = "ei-0c-append-only-ledger-v1";
pub const EI_0C_CLAIM: &str = "ei-0c-ledger-infrastructure-only";
pub const MAX_LEDGER_ENTRIES: usize = 4_096;

const ENTRY_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0c/ledger-entry/v1";
const ROOT_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0c/ledger-root/v1";
const GENESIS_DIGEST_DOMAIN: &[u8] = b"starfire/ei-0c/ledger-genesis/v1";
const DIGEST_HEX_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LedgerDigest(String);

impl LedgerDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), LedgerError> {
        if self.0.len() != DIGEST_HEX_LEN
            || !self
                .0
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(LedgerError::InvalidDigest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    schema_version: u16,
    sequence: u64,
    previous_entry_digest: LedgerDigest,
    episode_schema_version: u16,
    episode_id: String,
    episode_digest: String,
    episode_bytes: Vec<u8>,
    entry_digest: LedgerDigest,
}

impl LedgerEntry {
    fn new(
        sequence: u64,
        previous_entry_digest: LedgerDigest,
        episode: &SealedCognitiveEpisode,
    ) -> Result<Self, LedgerError> {
        episode.validate()?;
        let episode_bytes = episode.to_canonical_bytes()?;
        let episode_id = episode.episode.episode_id.as_str().to_owned();
        let episode_digest = episode.digest.as_str().to_owned();
        let entry_digest = compute_entry_digest(
            EI_0C_LEDGER_SCHEMA_VERSION,
            sequence,
            &previous_entry_digest,
            episode.schema_version,
            &episode_id,
            &episode_digest,
            &episode_bytes,
        )?;
        Ok(Self {
            schema_version: EI_0C_LEDGER_SCHEMA_VERSION,
            sequence,
            previous_entry_digest,
            episode_schema_version: episode.schema_version,
            episode_id,
            episode_digest,
            episode_bytes,
            entry_digest,
        })
    }

    fn validate(
        &self,
        expected_sequence: u64,
        expected_previous: &LedgerDigest,
    ) -> Result<SealedCognitiveEpisode, LedgerError> {
        if self.schema_version != EI_0C_LEDGER_SCHEMA_VERSION {
            return Err(LedgerError::UnsupportedEntrySchema(self.schema_version));
        }
        if self.sequence != expected_sequence {
            return Err(LedgerError::SequenceMismatch {
                expected: expected_sequence,
                actual: self.sequence,
            });
        }
        self.previous_entry_digest.validate()?;
        if self.previous_entry_digest != *expected_previous {
            return Err(LedgerError::ChainMismatch(self.sequence));
        }
        if self.episode_schema_version != EI_0A_SCHEMA_VERSION {
            return Err(LedgerError::UnsupportedEpisodeSchema(
                self.episode_schema_version,
            ));
        }
        validate_identifier(&self.episode_id)?;
        validate_digest_text(&self.episode_digest)?;
        self.entry_digest.validate()?;

        let episode = SealedCognitiveEpisode::from_canonical_bytes(&self.episode_bytes)?;
        if episode.schema_version != self.episode_schema_version {
            return Err(LedgerError::EpisodeSchemaMismatch(self.sequence));
        }
        if episode.episode.episode_id.as_str() != self.episode_id {
            return Err(LedgerError::EpisodeIdMismatch(self.sequence));
        }
        if episode.digest.as_str() != self.episode_digest {
            return Err(LedgerError::EpisodeDigestMismatch(self.sequence));
        }

        let expected_digest = compute_entry_digest(
            self.schema_version,
            self.sequence,
            &self.previous_entry_digest,
            self.episode_schema_version,
            &self.episode_id,
            &self.episode_digest,
            &self.episode_bytes,
        )?;
        if self.entry_digest != expected_digest {
            return Err(LedgerError::EntryDigestMismatch(self.sequence));
        }
        Ok(episode)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendOnlyEpisodeLedger {
    schema_version: u16,
    ledger_id: String,
    entry_count: u64,
    terminal_entry_digest: LedgerDigest,
    root_digest: LedgerDigest,
    entries: Vec<LedgerEntry>,
    authority: AuthoritySnapshot,
}

impl Default for AppendOnlyEpisodeLedger {
    fn default() -> Self {
        Self::new().expect("the empty EI-0C ledger contract must be valid")
    }
}

impl AppendOnlyEpisodeLedger {
    pub fn new() -> Result<Self, LedgerError> {
        let terminal_entry_digest = genesis_digest();
        let authority = AuthoritySnapshot::closed();
        let mut ledger = Self {
            schema_version: EI_0C_LEDGER_SCHEMA_VERSION,
            ledger_id: EI_0C_LEDGER_ID.to_owned(),
            entry_count: 0,
            terminal_entry_digest,
            root_digest: LedgerDigest(String::new()),
            entries: Vec::new(),
            authority,
        };
        ledger.root_digest = ledger.compute_root_digest()?;
        ledger.validate()?;
        Ok(ledger)
    }

    pub fn append(
        &mut self,
        episode: &SealedCognitiveEpisode,
    ) -> Result<LedgerDigest, LedgerError> {
        self.validate()?;
        episode.validate()?;
        if self.entries.len() >= MAX_LEDGER_ENTRIES {
            return Err(LedgerError::EntryLimitExceeded);
        }
        let episode_id = episode.episode.episode_id.as_str();
        if self
            .entries
            .iter()
            .any(|entry| entry.episode_id == episode_id)
        {
            return Err(LedgerError::DuplicateEpisodeId(episode_id.to_owned()));
        }

        let sequence = self
            .entry_count
            .checked_add(1)
            .ok_or(LedgerError::EntryLimitExceeded)?;
        let entry = LedgerEntry::new(
            sequence,
            self.terminal_entry_digest.clone(),
            episode,
        )?;
        let appended_digest = entry.entry_digest.clone();

        let mut next = self.clone();
        next.entries.push(entry);
        next.entry_count = sequence;
        next.terminal_entry_digest = appended_digest.clone();
        next.root_digest = next.compute_root_digest()?;
        next.validate()?;
        *self = next;
        Ok(appended_digest)
    }

    pub fn validate(&self) -> Result<(), LedgerError> {
        if self.schema_version != EI_0C_LEDGER_SCHEMA_VERSION {
            return Err(LedgerError::UnsupportedLedgerSchema(self.schema_version));
        }
        if self.ledger_id != EI_0C_LEDGER_ID {
            return Err(LedgerError::InvalidLedgerId);
        }
        if !self.authority.is_closed() {
            return Err(LedgerError::UnauthorizedLedger);
        }
        if self.entries.len() > MAX_LEDGER_ENTRIES {
            return Err(LedgerError::EntryLimitExceeded);
        }
        let actual_count =
            u64::try_from(self.entries.len()).map_err(|_| LedgerError::EntryLimitExceeded)?;
        if self.entry_count != actual_count {
            return Err(LedgerError::EntryCountMismatch {
                declared: self.entry_count,
                actual: actual_count,
            });
        }
        self.terminal_entry_digest.validate()?;
        self.root_digest.validate()?;

        let mut expected_previous = genesis_digest();
        let mut episode_ids = BTreeSet::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let expected_sequence = u64::try_from(index)
                .map_err(|_| LedgerError::EntryLimitExceeded)?
                .checked_add(1)
                .ok_or(LedgerError::EntryLimitExceeded)?;
            let episode = entry.validate(expected_sequence, &expected_previous)?;
            let episode_id = episode.episode.episode_id.as_str().to_owned();
            if !episode_ids.insert(episode_id.clone()) {
                return Err(LedgerError::DuplicateEpisodeId(episode_id));
            }
            expected_previous = entry.entry_digest.clone();
        }

        if self.terminal_entry_digest != expected_previous {
            return Err(LedgerError::TerminalDigestMismatch);
        }
        let expected_root = self.compute_root_digest()?;
        if self.root_digest != expected_root {
            return Err(LedgerError::RootDigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, LedgerError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|error| LedgerError::Serialization(error.to_string()))
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, LedgerError> {
        let ledger: Self = serde_json::from_slice(bytes)
            .map_err(|error| LedgerError::Deserialization(error.to_string()))?;
        ledger.validate()?;
        let canonical = serde_json::to_vec(&ledger)
            .map_err(|error| LedgerError::Serialization(error.to_string()))?;
        if canonical != bytes {
            return Err(LedgerError::NonCanonicalEncoding);
        }
        Ok(ledger)
    }

    pub fn replay_fresh(&self) -> Result<FreshLedgerReplay, LedgerError> {
        let bytes = self.to_canonical_bytes()?;
        Self::replay_from_canonical_bytes(&bytes)
    }

    pub fn replay_from_canonical_bytes(bytes: &[u8]) -> Result<FreshLedgerReplay, LedgerError> {
        let ledger = Self::from_canonical_bytes(bytes)?;
        let mut episodes = Vec::with_capacity(ledger.entries.len());
        for entry in &ledger.entries {
            episodes.push(SealedCognitiveEpisode::from_canonical_bytes(
                &entry.episode_bytes,
            )?);
        }
        let summary = ledger.summary()?;
        Ok(FreshLedgerReplay {
            canonical_ledger_bytes: bytes.to_vec(),
            episodes,
            summary,
        })
    }

    pub fn summary(&self) -> Result<LedgerReplaySummary, LedgerError> {
        self.validate()?;
        Ok(LedgerReplaySummary {
            schema_version: self.schema_version,
            ledger_id: self.ledger_id.clone(),
            claim: EI_0C_CLAIM.to_owned(),
            entry_count: self.entry_count,
            root_digest: self.root_digest.clone(),
            terminal_entry_digest: self.terminal_entry_digest.clone(),
            episode_ids: self
                .entries
                .iter()
                .map(|entry| entry.episode_id.clone())
                .collect(),
            episode_digests: self
                .entries
                .iter()
                .map(|entry| entry.episode_digest.clone())
                .collect(),
            authority: self.authority.clone(),
        })
    }

    pub fn root_digest(&self) -> &LedgerDigest {
        &self.root_digest
    }

    pub fn entry_count(&self) -> u64 {
        self.entry_count
    }

    fn compute_root_digest(&self) -> Result<LedgerDigest, LedgerError> {
        let entry_digests: Vec<&str> = self
            .entries
            .iter()
            .map(|entry| entry.entry_digest.as_str())
            .collect();
        let payload = RootDigestPayload {
            schema_version: self.schema_version,
            ledger_id: &self.ledger_id,
            entry_count: self.entry_count,
            terminal_entry_digest: self.terminal_entry_digest.as_str(),
            entry_digests,
            authority: &self.authority,
        };
        digest_serialized(ROOT_DIGEST_DOMAIN, &payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshLedgerReplay {
    pub canonical_ledger_bytes: Vec<u8>,
    pub episodes: Vec<SealedCognitiveEpisode>,
    pub summary: LedgerReplaySummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerReplaySummary {
    pub schema_version: u16,
    pub ledger_id: String,
    pub claim: String,
    pub entry_count: u64,
    pub root_digest: LedgerDigest,
    pub terminal_entry_digest: LedgerDigest,
    pub episode_ids: Vec<String>,
    pub episode_digests: Vec<String>,
    pub authority: AuthoritySnapshot,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LedgerError {
    #[error("unsupported EI-0C ledger schema version {0}")]
    UnsupportedLedgerSchema(u16),
    #[error("unsupported EI-0C entry schema version {0}")]
    UnsupportedEntrySchema(u16),
    #[error("unsupported EI-0A episode schema version {0}")]
    UnsupportedEpisodeSchema(u16),
    #[error("invalid EI-0C ledger identifier")]
    InvalidLedgerId,
    #[error("invalid EI-0C digest text")]
    InvalidDigest,
    #[error("invalid EI-0C identifier text")]
    InvalidIdentifier,
    #[error("EI-0C ledger entry limit exceeded")]
    EntryLimitExceeded,
    #[error("ledger entry count mismatch: declared {declared}, actual {actual}")]
    EntryCountMismatch { declared: u64, actual: u64 },
    #[error("ledger sequence mismatch: expected {expected}, actual {actual}")]
    SequenceMismatch { expected: u64, actual: u64 },
    #[error("ledger chain mismatch at sequence {0}")]
    ChainMismatch(u64),
    #[error("duplicate episode identifier in ledger: {0}")]
    DuplicateEpisodeId(String),
    #[error("episode schema does not match ledger entry at sequence {0}")]
    EpisodeSchemaMismatch(u64),
    #[error("episode identifier does not match ledger entry at sequence {0}")]
    EpisodeIdMismatch(u64),
    #[error("episode digest does not match ledger entry at sequence {0}")]
    EpisodeDigestMismatch(u64),
    #[error("ledger entry digest mismatch at sequence {0}")]
    EntryDigestMismatch(u64),
    #[error("ledger terminal digest mismatch")]
    TerminalDigestMismatch,
    #[error("ledger root digest mismatch")]
    RootDigestMismatch,
    #[error("EI-0C ledger attempted to claim runtime authority")]
    UnauthorizedLedger,
    #[error("ledger is valid JSON but not canonical byte encoding")]
    NonCanonicalEncoding,
    #[error("ledger serialization failed: {0}")]
    Serialization(String),
    #[error("ledger deserialization failed: {0}")]
    Deserialization(String),
    #[error("sealed EI-0A episode failed validation: {0}")]
    EpisodeContract(#[from] EpisodeContractError),
}

#[derive(Serialize)]
struct EntryDigestPayload<'a> {
    schema_version: u16,
    sequence: u64,
    previous_entry_digest: &'a str,
    episode_schema_version: u16,
    episode_id: &'a str,
    episode_digest: &'a str,
    episode_bytes: &'a [u8],
}

#[derive(Serialize)]
struct RootDigestPayload<'a> {
    schema_version: u16,
    ledger_id: &'a str,
    entry_count: u64,
    terminal_entry_digest: &'a str,
    entry_digests: Vec<&'a str>,
    authority: &'a AuthoritySnapshot,
}

fn compute_entry_digest(
    schema_version: u16,
    sequence: u64,
    previous_entry_digest: &LedgerDigest,
    episode_schema_version: u16,
    episode_id: &str,
    episode_digest: &str,
    episode_bytes: &[u8],
) -> Result<LedgerDigest, LedgerError> {
    let payload = EntryDigestPayload {
        schema_version,
        sequence,
        previous_entry_digest: previous_entry_digest.as_str(),
        episode_schema_version,
        episode_id,
        episode_digest,
        episode_bytes,
    };
    digest_serialized(ENTRY_DIGEST_DOMAIN, &payload)
}

fn digest_serialized<T: Serialize>(
    domain: &[u8],
    payload: &T,
) -> Result<LedgerDigest, LedgerError> {
    let bytes =
        serde_json::to_vec(payload).map_err(|error| LedgerError::Serialization(error.to_string()))?;
    Ok(LedgerDigest(checksum128(domain, &bytes)))
}

fn genesis_digest() -> LedgerDigest {
    LedgerDigest(checksum128(GENESIS_DIGEST_DOMAIN, b"genesis"))
}

fn validate_identifier(value: &str) -> Result<(), LedgerError> {
    if value.is_empty() || value.trim() != value {
        return Err(LedgerError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_digest_text(value: &str) -> Result<(), LedgerError> {
    if value.len() != DIGEST_HEX_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(LedgerError::InvalidDigest);
    }
    Ok(())
}

fn checksum128(domain: &[u8], payload: &[u8]) -> String {
    let left = fnv1a64(0xcbf29ce484222325, 0x4c, domain, payload);
    let right = fnv1a64(0x84222325cbf29ce4, 0x52, domain, payload);
    format!("{left:016x}{right:016x}")
}

fn fnv1a64(seed: u64, lane: u8, domain: &[u8], payload: &[u8]) -> u64 {
    let mut digest = seed;
    for byte in domain
        .iter()
        .copied()
        .chain([lane])
        .chain((payload.len() as u64).to_le_bytes())
        .chain(payload.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emerging_intelligence::{
        CognitiveEpisode, EpisodeId, EpisodePhase, EpisodeProvenance, EvaluationPartition,
        Observation, ObservationId,
    };

    fn observed_episode(id: &str, seed: u64) -> SealedCognitiveEpisode {
        CognitiveEpisode {
            episode_id: EpisodeId::new(id).unwrap(),
            phase: EpisodePhase::Observed,
            partition: EvaluationPartition::Development,
            task_family: "route-choice".into(),
            observation: Observation {
                observation_id: ObservationId::new(format!("observation-{id}")).unwrap(),
                kind: "route-state".into(),
                facts: vec![format!("seed:{seed:08x}")],
                observed_at_step: 1,
            },
            evidence: Vec::new(),
            predictions: Vec::new(),
            selected_strategy: None,
            intention: None,
            action: None,
            outcome: None,
            evaluation: None,
            proposed_updates: Vec::new(),
            accepted_updates: Vec::new(),
            authority: AuthoritySnapshot::closed(),
            provenance: EpisodeProvenance {
                cohort_id: "ei-0c-test".into(),
                fixture_digest: format!("fixture:{seed:08x}"),
                seed,
                generator_version: "ei-0c-test-v1".into(),
                source_hashes: vec![format!("source:{seed:08x}")],
            },
        }
        .seal()
        .unwrap()
    }

    fn two_entry_ledger() -> AppendOnlyEpisodeLedger {
        let mut ledger = AppendOnlyEpisodeLedger::new().unwrap();
        ledger.append(&observed_episode("episode-001", 1)).unwrap();
        ledger.append(&observed_episode("episode-002", 2)).unwrap();
        ledger
    }

    #[test]
    fn canonical_ledger_replays_exactly_from_fresh_state() {
        let ledger = two_entry_ledger();
        let bytes = ledger.to_canonical_bytes().unwrap();
        let replay = AppendOnlyEpisodeLedger::replay_from_canonical_bytes(&bytes).unwrap();
        assert_eq!(replay.canonical_ledger_bytes, bytes);
        assert_eq!(replay.episodes.len(), 2);
        assert_eq!(replay.summary, ledger.summary().unwrap());
        assert_eq!(replay.summary.authority, AuthoritySnapshot::closed());
    }

    #[test]
    fn same_episode_sequence_produces_identical_bytes_and_root() {
        let first = two_entry_ledger();
        let second = two_entry_ledger();
        assert_eq!(first.to_canonical_bytes().unwrap(), second.to_canonical_bytes().unwrap());
        assert_eq!(first.root_digest(), second.root_digest());
    }

    #[test]
    fn duplicate_episode_identifier_fails_closed() {
        let episode = observed_episode("episode-001", 1);
        let mut ledger = AppendOnlyEpisodeLedger::new().unwrap();
        ledger.append(&episode).unwrap();
        assert_eq!(
            ledger.append(&episode),
            Err(LedgerError::DuplicateEpisodeId("episode-001".into()))
        );
    }

    #[test]
    fn reordered_entries_fail_closed() {
        let mut ledger = two_entry_ledger();
        ledger.entries.swap(0, 1);
        assert!(matches!(
            ledger.validate(),
            Err(LedgerError::SequenceMismatch { .. })
                | Err(LedgerError::ChainMismatch(_))
        ));
    }

    #[test]
    fn broken_chain_fails_closed() {
        let mut ledger = two_entry_ledger();
        ledger.entries[1].previous_entry_digest = genesis_digest();
        assert_eq!(ledger.validate(), Err(LedgerError::ChainMismatch(2)));
    }

    #[test]
    fn modified_episode_bytes_fail_closed() {
        let mut ledger = two_entry_ledger();
        ledger.entries[0].episode_bytes[0] ^= 1;
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn modified_entry_digest_fails_closed() {
        let mut ledger = two_entry_ledger();
        ledger.entries[0].entry_digest = genesis_digest();
        assert_eq!(
            ledger.validate(),
            Err(LedgerError::EntryDigestMismatch(1))
        );
    }

    #[test]
    fn truncated_ledger_fails_closed() {
        let mut ledger = two_entry_ledger();
        ledger.entries.pop();
        assert_eq!(
            ledger.validate(),
            Err(LedgerError::EntryCountMismatch {
                declared: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn stale_schema_fails_closed() {
        let mut ledger = two_entry_ledger();
        ledger.schema_version += 1;
        assert_eq!(
            ledger.validate(),
            Err(LedgerError::UnsupportedLedgerSchema(2))
        );
    }

    #[test]
    fn noncanonical_encoding_fails_closed() {
        let ledger = two_entry_ledger();
        let mut bytes = ledger.to_canonical_bytes().unwrap();
        bytes.push(b'\n');
        assert_eq!(
            AppendOnlyEpisodeLedger::from_canonical_bytes(&bytes),
            Err(LedgerError::NonCanonicalEncoding)
        );
    }

    #[test]
    fn unauthorized_ledger_fails_closed() {
        let mut ledger = two_entry_ledger();
        ledger.authority.persistence_authority = true;
        assert_eq!(ledger.validate(), Err(LedgerError::UnauthorizedLedger));
    }
}
