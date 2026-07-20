//! ΩV1-F1 projection packet integrity and fail-closed selector wrapper.
//!
//! The learned selector consumes a bounded projection derived from VoiceState.
//! This wrapper seals the exact projection bytes and verifies them immediately
//! before scoring. A stale or corrupted packet forces the existing grammar-v2
//! neutral fallback through the selector's ordinary fail-closed path.

use crate::language_realization::LexicalBindingTable;
use crate::learned_expression::{
    LearnedExpressionError, LearnedExpressionModel, LearnedSelectionResult,
    LearnedVoiceProjection, OfflineLearnedExpressionSelector,
};
use crate::semantic_response::SemanticResponseProgram;
use crate::voice_state::VoiceDebugProjection;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const PROJECTION_PACKET_DOMAIN: &[u8] = b"starfire-omega-v1f1-projection-packet-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVoiceProjection {
    pub projection: LearnedVoiceProjection,
    pub packet_digest: u64,
}

impl VerifiedVoiceProjection {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u64,
        directness_bps: u16,
        warmth_bps: u16,
        compression_bps: u16,
        initiative_bps: u16,
        disagreement_bps: u16,
        uncertainty_bps: u16,
        intensity_bps: u16,
        source_digest: impl Into<String>,
    ) -> Result<Self, LearnedExpressionError> {
        Self::seal(LearnedVoiceProjection::new(
            version,
            directness_bps,
            warmth_bps,
            compression_bps,
            initiative_bps,
            disagreement_bps,
            uncertainty_bps,
            intensity_bps,
            source_digest,
        )?)
    }

    pub fn from_debug_projection(
        projection: &VoiceDebugProjection,
    ) -> Result<Self, LearnedExpressionError> {
        Self::seal(LearnedVoiceProjection::from_debug_projection(projection)?)
    }

    pub fn seal(projection: LearnedVoiceProjection) -> Result<Self, LearnedExpressionError> {
        let packet_digest = digest_projection(&projection)?;
        if packet_digest == 0 {
            return Err(LearnedExpressionError::EmptyDigest);
        }
        Ok(Self {
            projection,
            packet_digest,
        })
    }

    pub fn verify_integrity(&self) -> Result<(), ProjectionPacketError> {
        if self.packet_digest == 0 {
            return Err(ProjectionPacketError::EmptyDigest);
        }
        let expected = digest_projection(&self.projection)
            .map_err(|error| ProjectionPacketError::Serialization(error.to_string()))?;
        if expected != self.packet_digest {
            return Err(ProjectionPacketError::DigestMismatch);
        }
        Ok(())
    }
}

impl Deref for VerifiedVoiceProjection {
    type Target = LearnedVoiceProjection;

    fn deref(&self) -> &Self::Target {
        &self.projection
    }
}

impl DerefMut for VerifiedVoiceProjection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.projection
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProjectionPacketError {
    #[error("projection packet digest is zero")]
    EmptyDigest,
    #[error("projection packet digest does not match the projection bytes")]
    DigestMismatch,
    #[error("projection packet serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone)]
pub struct GuardedOfflineLearnedExpressionSelector {
    model: LearnedExpressionModel,
}

impl GuardedOfflineLearnedExpressionSelector {
    #[must_use]
    pub fn new(model: LearnedExpressionModel) -> Self {
        Self { model }
    }

    pub fn select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &VerifiedVoiceProjection,
    ) -> Result<LearnedSelectionResult, LearnedExpressionError> {
        if projection.verify_integrity().is_ok() {
            return OfflineLearnedExpressionSelector::new(self.model.clone()).select(
                program,
                lexical_table,
                &projection.projection,
            );
        }

        let mut fail_closed_model = self.model.clone();
        fail_closed_model.digest.0 ^= 1;
        OfflineLearnedExpressionSelector::new(fail_closed_model).select(
            program,
            lexical_table,
            &projection.projection,
        )
    }
}

fn digest_projection(projection: &LearnedVoiceProjection) -> Result<u64, LearnedExpressionError> {
    let bytes = serde_json::to_vec(projection)
        .map_err(|error| LearnedExpressionError::CanonicalSerialization(error.to_string()))?;
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in PROJECTION_PACKET_DOMAIN.iter().chain(bytes.iter()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_packet_detects_mutation() {
        let mut packet = VerifiedVoiceProjection::new(
            1,
            9_000,
            2_000,
            9_000,
            8_000,
            9_000,
            8_500,
            5_000,
            "voice-state-digest-v1",
        )
        .expect("packet");
        assert!(packet.verify_integrity().is_ok());
        packet.source_digest.push_str(":stale");
        assert_eq!(
            packet.verify_integrity(),
            Err(ProjectionPacketError::DigestMismatch)
        );
    }
}
