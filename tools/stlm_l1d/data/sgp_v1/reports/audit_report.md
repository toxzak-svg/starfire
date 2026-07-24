# Audit Report

## Verdict

**PASS for dataset construction, schema integrity, leakage controls, and trainer execution.**

This is not a model-quality pass. The smoke runs deliberately used only 64 records, one epoch, and tiny dimensions to prove the pipelines execute end to end.

## Corpus integrity

### RNN

- Records: 3,600
- Mirrored groups: 1,800
- Splits: train 2,880, dev 360, test 360
- Tiers: silver 1,200, gold 1,200, platinum 1,200
- Axes: 12 at 300 rows each
- Maximum candidate length: 384 bytes
- Mean candidate length: 222.78 bytes

### CNN

- Records: 3,600
- Mirrored groups: 1,800
- Splits: train 2,880, dev 360, test 360
- Tiers: silver 1,200, gold 1,200, platinum 1,200
- Axes: 12 at 300 rows each
- Maximum candidate length: 355 bytes
- Mean candidate length: 171.94 bytes

## Checks performed

- Every JSONL line parsed as one JSON object.
- All source IDs are unique.
- Every mirror group contains exactly two rows.
- Candidate A and B are byte-identical across mirror sides.
- Preferred candidate flips A/B across mirror sides.
- Context changes across mirror sides.
- Semantic invariant digest remains identical across mirror sides.
- All eight context values are integers in the range 0-10,000.
- Every training candidate is marked eligible by semantic, slot, and identity gates.
- Every training plan is restricted to `wording_only` authority.
- Mirror groups do not cross train/dev/test.
- Train, dev, and test use split-exclusive lexical-family IDs.
- No exact candidate or candidate pair leaks from train into dev or test.
- Every candidate is ASCII and below the current 1,024-byte critic limit.
- Tier, split, axis, and A/B label distributions are balanced.
- The 240 hard-gate adversaries are marked `training_allowed: false`.

## Hard-gate adversaries

- authority_expansion: 30
- evidence_rewrite: 30
- identity_conflict: 30
- memory_overclaim: 30
- missing_slot: 30
- semantic_drift: 30
- source_dependence: 30
- wrong_confidence: 30

## Smoke execution

Both reference trainers compiled and completed forward pass, pairwise loss, backward pass, optimizer update, dev evaluation, test evaluation, checkpoint write, and metrics write.

- CNN smoke: dev accuracy 0.500, test accuracy 0.500.
- RNN smoke: dev accuracy 0.500, test accuracy 0.500.

The 0.50 smoke accuracy is expected and not diagnostic: mirrored labels are balanced, the smoke uses a tiny first slice, and one epoch is insufficient. A real experiment must use the complete train split and preregistered controls.

## Remaining promotion requirements

- Blinded human adjudication of a stratified sample from every axis and tier.
- Full training with macro metrics by axis and tier.
- Shuffled-label, reversed-label, untrained, context-zeroed, and deterministic-only controls.
- Exact Rust/Python parity for any exported model.
- Hard-gate adversary survival through the complete selection path.
- Separate authorization for shadow attachment and any later canary.
