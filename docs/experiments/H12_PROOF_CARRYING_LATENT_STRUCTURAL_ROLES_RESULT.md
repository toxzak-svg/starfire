# H12 Proof-Carrying Latent Structural Roles — Result

## Status

**Executable acceptance classification: `PASS`**

**PR status: draft pending one strict preregistration-conformance hardening item described below.**

H12 was proposed in `H12_PROOF_CARRYING_LATENT_STRUCTURAL_ROLES.md` as the first deterministic, auditable representation-invention experiment after H9–H11.

The accepted executable path is:

```text
raw renamed structural graphs
    -> target-blind deterministic fingerprints
    -> recurring opaque latent role proofs
    -> independent exact membership recomputation
    -> validated shadow role certificates
    -> role-conditioned inert evidence projection
    -> unchanged H11 frontier discovery
    -> unchanged H10 full-ranking validation
    -> H11 certificate admission
    -> H9 executable closure
```

No automatic ontology promotion, production routing, persistent ontology mutation, or `Live` abstraction state was enabled.

---

## Frozen implementation lineage

Preregistration commit:

```text
49654e75b514df453db942c22eaaf63b4c216f8a
```

Core latent-role substrate:

```text
8cfd27857bc3a09e5b1abecfd177d0a507066be9
```

First verdict-producing head:

```text
7818f1819ee24475256cd52a585282265e6ea383
```

Corrected acceptance head:

```text
152c419aa7a565cb84b76d2246f05ed67a8cf88c
```

---

## First verdict-producing run

CHARGE CI:

```text
run number: 100
run id: 29144787280
head: 7818f1819ee24475256cd52a585282265e6ea383
terminal classification: INFRASTRUCTURE_FAILURE
```

The failure was isolated to the H12 experiment budget assertion. Compilation, deterministic contract tests, H9, H10, and H11 all passed in the same run.

The emitted H12 report showed:

```text
stateful train:   16/16
stateful holdout:  8/8
stateful future:  32/32
all controls:       0 objective successes
foreign rejection:  true
membership tamper rejection: true
role membership exact: true
future-family transfer: true
replay exact: true
invariants hold: true
budgets exact: false
```

The infrastructure defect was specific and auditable:

```text
A role could match a transfer-graph structure,
scan the complete evidence table,
and then return EmptyProjection because no evidence episode
used that role member as its intervention.

The observed budget correctly counted that evidence scan.
The expected-budget formula counted only non-empty projections.
```

Therefore the first verdict was preserved as `INFRASTRUCTURE_FAILURE`. It was not relabeled as a scientific pass.

First-run artifact:

```text
artifact id: 8246402783
digest: sha256:00473816057d77c4f566e931a7cd54b2f4084f90a35a7ce8fe93a55de40ccd22
```

---

## Correction scope

The acceptance correction changed only budget accounting.

It counts every role projection that actually scans the evidence table, including a structurally matched role whose resulting evidence projection is empty.

The correction did **not** change:

```text
graphs
node identities
structural fingerprints
role recurrence gates
role membership
proof format
validation logic
evidence episodes
H10 scoring law
H10 thresholds
H11 frontier law
matched-control evidence
certificate admission rules
closure budget
objective definition
train / holdout / future cohort
```

The corrected acceptance executable also verifies H11 scoring budgets from the discovered frontier dimensions and evidence-table size.

---

## Accepted run

CHARGE CI:

```text
run number: 102
run id: 29144994239
head: 152c419aa7a565cb84b76d2246f05ed67a8cf88c
workflow conclusion: success
H12 terminal classification: PASS
```

Accepted artifact:

```text
artifact id: 8246470458
digest: sha256:6d5f9f2e7c42ad8a8527b819baf67d6c308d47fe91afd3318d961d28a6ea6690
```

The same workflow run passed:

```text
cargo check -p star --all-targets --locked
CHARGE / closed-cycle deterministic contract tests
latent_roles unit tests
H9 executable commitment probe
H10 evidence-bound rule induction probe
H11 graph-discovered relation induction probe
H12 corrected acceptance probe
```

---

## Objective result

### Training

```text
stateful_validated_role: 16/16
all controls:              0/16 objective successes each
```

### Holdout

```text
stateful_validated_role: 8/8
all controls:             0/8 objective successes each
```

### Future

```text
stateful_validated_role: 32/32
all controls:              0/32 objective successes each
```

### Future-family transfer

```text
cellular:       stateful 8/8, maximum control 0/8
manufacturing:  stateful 8/8, maximum control 0/8
software:       stateful 8/8, maximum control 0/8
watershed:      stateful 8/8, maximum control 0/8
```

---

## Control result

Across all 56 roots:

```text
no_abstraction_full_graph:          0/56 objective successes
role_proof_text_only:               0/56 objective successes
scalar_role_id_only:                0/56 objective successes
random_grouping:                    0/56 objective successes
size_matched_grouping:              0/56 objective successes
valid_irrelevant_role:              0/56 objective successes
foreign_abstraction:                0/56 objective successes
membership_tampered_abstraction:    0/56 objective successes
delayed_abstraction_admission:      0/56 objective successes
```

The stronger executable controls behaved as required:

```text
valid_irrelevant_role:
    valid H11 certificate available 56/56
    executable admission 56/56
    objective success 0/56

size_matched_grouping:
    valid irrelevant H11 certificate available 56/56
    executable admission 56/56
    objective success 0/56

foreign_abstraction:
    rejected 56/56

membership_tampered_abstraction:
    rejected 56/56

delayed_abstraction_admission:
    correct target certificate available 56/56
    admitted only after the three-scan closure window
    objective success 0/56
```

The stateful path admitted two independently validated role-derived executable relations per root: the useful target-role relation and a causally irrelevant role relation. The irrelevant-role-only control demonstrates that the irrelevant admitted relation does not satisfy the objective.

---

## Gate result

The accepted JSON report recorded every implemented acceptance gate as true:

```text
cohort_exact: true
stateful_train_holdout_future: true
all_controls_zero: true
valid_irrelevant_accepted: true
foreign_rejected: true
membership_tamper_rejected: true
role_membership_exact: true
future_family_transfer: true
budgets_exact: true
replay_exact: true
invariants_hold: true
```

Terminal classification:

```text
PASS
```

---

## What the result supports

Under the frozen deterministic synthetic graph regime, the implementation demonstrates the following executable chain:

1. Starfire receives graph structure with opaque, root-specific node names.
2. It computes target-blind structural fingerprints.
3. It proposes recurring unnamed structural equivalence classes without receiving a role count or role label.
4. An independent validator recomputes exact role membership and proof identity from the raw discovery corpus.
5. A validated shadow role recognizes a structurally equivalent node in unseen topology variants.
6. The role changes computation by partitioning mixed evidence.
7. The partition exposes a relation that must still survive unchanged H11 discovery and H10 independent validation.
8. Only the resulting H11 certificate may alter executable PECS state.
9. The resulting executable relation enables the bounded objective that the full-evidence, inert, matched, irrelevant, foreign, tampered, and delayed controls do not achieve.

This is a meaningful transition from:

```text
reasoning only with developer-supplied executable relations
```

toward:

```text
reasoning with an internally induced structural representation
that is independently validated before it can contribute to executable state
```

---

## Strict conformance note

The executable acceptance report is `PASS`, but one stronger wording in the preregistration should be hardened before treating the PR as a final, maximally strict H12 closure.

The preregistration described `random_grouping` as:

```text
a deterministic root-seeded grouping
with the same target-group cardinality as the real role partition
```

The implemented control instead uses a deterministic mixed target/adversarial **evidence grouping** with the same four-episode size as the useful target evidence projection. It is a meaningful matched adversarial grouping and it fails 0/56, but it is not literally a root-seeded same-member-cardinality random partition.

This discrepancy does not change the accepted executable outcome, and the original representation-invention program required a random grouping and a size-matched grouping without prescribing this stronger exact construction. Nevertheless, the discrepancy is recorded rather than silently ignored.

Recommended closure action:

```text
add a separate preregistration-conformance control probe
for root-seeded random member partitions,
without changing the accepted H12 worlds or mechanism
```

Until that hardening is complete, keep the pull request in draft status.

---

## Claim boundary

A `PASS` supports only this narrow claim:

> Under a deterministic synthetic graph regime, Starfire can induce recurring unnamed structural equivalence classes from raw topology, independently validate exact role membership, recognize those roles in held-out structural variants, and use a validated role as a causally necessary evidence partition that enables the unchanged H11/H10 proof path to certify executable reasoning unavailable to the implemented matched controls.

It does **not** establish:

```text
natural-language concept discovery
continuous latent representation learning
open-world ontology induction
unbounded representation invention
automatic ontology promotion
live routing readiness
persistent self-modifying ontology
cross-domain semantic concepts
AGI
consciousness
human-level cognition
```
