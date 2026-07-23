# EI-0E Terminal Preregistration

> **Stage:** EI-0E  
> **Status:** frozen candidate, no terminal result executed  
> **Authority:** experiment specification only  
> **Parent:** EI-0 tracker #149 and issue #195  
> **Preregistration ID:** `ei-0e-terminal-v1`  
> **Freeze base:** `b03dd8a87bc32474e8dc397e40edebc7c0c9d548`  
> **Canonical manifest SHA-256:** `352fcc8352a4cb6802f92fcfd50797c2ee1092311909601c03e609f3c9d8a97b`

## Purpose

EI-0E freezes the exact terminal EI-0 experiment before any qualifying EI-0F result is inspected. It binds the hypotheses, source files, fixture seeds, control arms, budgets, update lattice, evaluators, thresholds, report schema, failure rules, terminal runner, and classifier into canonical files.

The EI-0E gate may compile the terminal runner, verify its authority boundary, and exercise synthetic classifier vectors. It must not execute the terminal runner. The first qualifying execution belongs to EI-0F.

## Frozen source

The canonical manifest binds the post-EI-0D implementation at base commit `b03dd8a8` using Git blob object digests for:

- `Cargo.lock`;
- the `star` crate manifest and root;
- EI-0A episode contracts;
- EI-0B environment, fixtures, generator, evaluator, and manifest;
- EI-0C append-only ledger;
- EI-0D reversible-update engine.

EI-0E files are bound separately by `EI_0E_FREEZE_LOCK.json`. The lock excludes itself and binds the canonical manifest, classifier, report schema, terminal runner, workflow, and this record.

Any bound-file change requires a new preregistration identifier. A failure may not be repaired under `ei-0e-terminal-v1`.

## Frozen experiment

### Arms

Every fixture is evaluated under the same ordered arms:

1. learning;
2. no-update;
3. memory-disabled;
4. random-update;
5. fixed-policy.

Each arm begins from the same validated novice state with all five EI-0D policy weights at zero. Under the fixed canonical ordering, equal scores select the final legal action. Learning is therefore required to acquire useful weights from development episodes rather than inheriting the correct policy.

### Fixtures

The existing EI-0B manifest remains unchanged:

| Partition | Seeds |
|---|---|
| Development | 101, 102 |
| Within-family holdout | 201, 202 |
| Renamed-vocabulary transfer | 301, 302 |
| Structural transfer | 401, 402 |
| Regression | 501, 502 |
| Adversarial | 601, 602 |

Odd canonical structure seeds produce route-choice tasks. Even seeds produce attribute-rule tasks. Every arm receives one fixture from each family in every partition.

### Learning updates

The learning arm receives two development update opportunities:

- route-choice experience may set `route_cost_weight_bps` to `10000`;
- attribute-rule experience may set `rule_coverage_weight_bps` to `10000`.

Each proposal must be accepted by a sealed EI-0A episode, present in an EI-0C ledger, independently admitted by EI-0D, and independently checked against protected partitions before commitment.

The random-update arm receives the same two opportunities through the same transaction mechanism. Slot and value selection are frozen by SplitMix64, the EI-0B control-seed XOR, uniform modulo-five slot selection, and values in `{0, 10000}`.

### Budgets

- one action per fixture;
- two evidence reads per fixture;
- two fixtures per partition and arm;
- twelve evaluations per arm;
- sixty independent evaluations total;
- two development update opportunities per arm;
- at most two accepted learning updates;
- `10000` basis points per update;
- `20000` cumulative basis points;
- one separately isolated harmful-update challenge.

## Frozen hypotheses

**Primary:** prior independently scored development experience causes the learning arm to exceed every matched non-learning control across within-family holdout, renamed transfer, and structural transfer.

**Renamed transfer:** the advantage persists when labels change but relations do not.

**Structural transfer:** the learning arm does not lose its aggregate advantage when task structure changes.

**Causal attribution:** at least one held-out action change must be traced from a sealed source episode through an accepted update, proposal, transaction, post-state digest, and changed action.

**Safety and retention:** the deliberate harmful update must be independently detected and restored byte-for-byte, while accepted updates cause no regression-partition loss.

## Frozen PASS rule

PASS requires every condition below:

- learning exceeds each of the four controls by at least `1666` basis points on the six-fixture primary aggregate;
- learning exceeds each control by at least `5000` basis points on renamed-vocabulary transfer;
- learning is not below any control on structural transfer;
- learning loses `0` basis points on regression relative to its pre-update score;
- one or two learning updates are applied;
- at least one complete causal chain changes a held-out action;
- the single harmful challenge is detected and restored exactly;
- replay mismatch count is zero;
- missing, invalid, or corrupt record count is zero;
- all five arms and all six partitions are present;
- action, evidence, fixture, and update-opportunity budgets are equal;
- source and preregistration digests match;
- evaluators remain independent;
- authority remains closed.

Ties below the explicit margin fail. Crashes, timeouts, source mismatch, nondeterminism, missing arms, missing evaluations, duplicate evaluations, corruption, evaluator self-certification, or ambiguous thresholds are FAIL, not inconclusive.

## Executable artifacts

- Canonical manifest: `EI_0E_TERMINAL_PREREGISTRATION.json`
- Freeze lock: `EI_0E_FREEZE_LOCK.json`
- Classifier: `scripts/ei0e_preregistration.py`
- Terminal report schema: `EI_0F_TERMINAL_REPORT.schema.json`
- Frozen terminal runner: `lib/examples/ei_0f_terminal_experiment.rs`
- Read-only gate: `.github/workflows/ei-0e-preregistration-ci.yml`

The frozen terminal command is:

```text
cargo run -p star --example ei_0f_terminal_experiment --features emerging-intelligence-updates --locked --quiet
```

The terminal classifier command is:

```text
python3 scripts/ei0e_preregistration.py classify docs/experiments/EI_0F_TERMINAL_REPORT.json
```

## EI-0E verification boundary

EI-0E verifies canonical bytes, bound source blobs, the freeze lock, closed authority, classifier test vectors, pinned formatting, and compilation of the dormant terminal runner. It explicitly scans the gate to ensure the runner is not executed.

No observed EI-0F metric is present in this preregistration. Synthetic PASS and FAIL reports used by classifier self-tests are contract vectors, not experiment results.

## Claim boundary

EI-0E freezes an experiment. It produces no evidence that Starfire improves from experience and grants no runtime, persistence, response, routing, belief, ontology, tool, or autonomous-action authority.
