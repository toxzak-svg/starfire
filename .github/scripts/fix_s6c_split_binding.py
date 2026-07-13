from pathlib import Path
import re

path = Path("lib/companion_real_interaction_canary.rs")
src = path.read_text()

old_call = "validate_seal_against_outcomes(seal, outcomes)?;"
count = src.count(old_call)
if count != 3:
    raise SystemExit(f"expected 3 seal/outcome validation calls, found {count}")
src = src.replace(
    old_call,
    "validate_seal_against_outcomes(self.config, seal, outcomes)?;",
)

pattern = re.compile(
    r"fn validate_seal_against_outcomes\([\s\S]*?\nfn validate_direct_attestation\("
)
replacement = '''fn validate_seal_against_outcomes(
    config: CanaryStudyConfig,
    seal: &CanaryTrialSeal,
    outcomes: &InteractionOutcomeLedger,
) -> Result<(), CanaryEvidenceError> {
    validate_seal(config, seal)?;
    let trial = outcomes
        .trials()
        .get(&seal.trial_id)
        .ok_or(CanaryEvidenceError::UnknownTrial(seal.trial_id))?;
    let rebuilt = build_seal(
        config,
        trial,
        seal.consent_digest,
        seal.operator_digest,
    )?;
    if rebuilt != *seal {
        return Err(CanaryEvidenceError::TrialChangedAfterSeal(seal.trial_id));
    }
    Ok(())
}

fn validate_direct_attestation('''
src, replaced = pattern.subn(replacement, src, count=1)
if replaced != 1:
    raise SystemExit(f"expected one seal validation block, replaced {replaced}")

anchor = '''    if seal.seal_digest_fnv1a64 != canonical_seal_digest(seal) {
        return Err(CanaryEvidenceError::SealDigestMismatch(seal.trial_id));
    }
'''
insert = '''    let expected_split = if seal.issued_at_ms >= config.split_policy.temporal_holdout_start_ms {
        EvaluationSplit::TemporalHoldout
    } else if seal.subject_scope_digest % config.split_policy.opaque_subject_modulus
        == config.split_policy.opaque_subject_remainder
    {
        EvaluationSplit::OpaqueSubjectHoldout
    } else {
        EvaluationSplit::Development
    };
    if seal.split != expected_split {
        return Err(CanaryEvidenceError::SplitAssignmentMismatch(seal.trial_id));
    }
    if seal.seal_digest_fnv1a64 != canonical_seal_digest(seal) {
        return Err(CanaryEvidenceError::SealDigestMismatch(seal.trial_id));
    }
'''
if anchor not in src:
    raise SystemExit("seal digest validation anchor not found")
src = src.replace(anchor, insert, 1)

error_anchor = '''    #[error("trial {0} changed after S6-C sealing")]
    TrialChangedAfterSeal(InteractionTrialId),
'''
error_insert = '''    #[error("trial {0} has a split inconsistent with the frozen split policy")]
    SplitAssignmentMismatch(InteractionTrialId),
    #[error("trial {0} changed after S6-C sealing")]
    TrialChangedAfterSeal(InteractionTrialId),
'''
if error_anchor not in src:
    raise SystemExit("error anchor not found")
src = src.replace(error_anchor, error_insert, 1)

path.write_text(src)
