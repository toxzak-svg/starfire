# EI-0F-R1 Proposal-Digest Remediation Preregistration

> **Status:** frozen remediation preflight
> **Preregistration ID:** `ei-0f-remediation-v1`
> **Parent result:** EI-0F / `ei-0e-terminal-v1` / preserved `FAIL`
> **Tracking issue:** #204
> **Preflight implementation:** PR #206
> **Formatting authority:** repository Rustfmt only
> **Authority:** offline experiment preparation only

## Decision

EI-0F remains permanently preserved as the first qualifying terminal execution and remains classified `FAIL`. It must not be edited, relabeled, or rerun under its original preregistration identifier.

EI-0F-R1 addresses one construction defect observed before arm evaluation:

```rust
format!("preregistered:{update_id}")
```

The frozen terminal runner used control-arm text inside `update_id`. Arms such as `random_update`, `no_update`, `memory_disabled`, and `fixed_policy` contain `_`, while the EI-0A digest-text alphabet permits lowercase letters, digits, `:`, and `-` only.

## Sole permitted remediation

For the episode-level `LearningUpdate.proposal_digest` reference only:

```rust
format!("preregistered:{}", update_id.replace('_', "-"))
```

The update identifier itself is unchanged. The EI-0A validator is unchanged. EI-0D proposal and transaction digesting are unchanged.

## Frozen invariants

The later terminal remediation must retain the EI-0E scientific design without alteration:

- the same two task families;
- the same six frozen partitions and seeds;
- the same five arms;
- the same action and evidence budgets;
- the same independent EI-0B evaluator boundary;
- the same EI-0D admissibility and safety evaluators;
- the same harmful-update challenge and exact rollback requirement;
- the same thresholds, classifier semantics, report fields, causal-chain requirement, regression bound, and claim boundary.

No threshold may be relaxed after observing output.

## Required preflight

`ei_0f_r1_digest_remediation_probe` must prove all of the following before a new terminal runner is frozen:

1. the original `random_update` proposal-digest text fails closed with `InvalidDigestText("learning proposal digest")`;
2. underscore-to-hyphen normalization seals all five arm records;
3. every sealed record replays from canonical bytes exactly;
4. no runtime, live-learning, ontology, unrestricted-tool, or autonomous-action authority is introduced;
5. the original EI-0F source and evidence remain unmodified.

## Mechanical formatting boundary

The repository-owned formatter may modify only `lib/examples/ei_0f_r1_digest_remediation_probe.rs` and only by applying the repository Rustfmt toolchain. Any semantic edit, additional changed path, terminal execution, or modification to the original EI-0F result invalidates this preflight.

Quiet Cargo output is diagnostic-only. It does not relax compilation, Clippy `-D warnings`, exact replay, or any scientific gate.

## Terminal execution sequence

After this preflight passes:

1. create a new terminal runner under a new path and the same remediation ID;
2. copy the frozen EI-0F design and change only the preregistration identity plus the permitted proposal-digest construction;
3. bind exact runner source, lockfile, manifest, thresholds, and classifier hashes before execution;
4. execute exactly once;
5. preserve the resulting `PASS` or `FAIL` literally;
6. do not authorize EI-0G unless that terminal run passes and a separate shadow preregistration is approved.

## Live boundary

Production deployment may serve the existing `starfire-live` chat runtime. EI-0F-R1 is offline-only and cannot affect live responses, persistence, routing, tools, ontology, or autonomous actions.
