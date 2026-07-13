# H13-C Structural Role Transfer Stress — Frozen Preregistration

Status: **frozen before implementation and before any verdict-producing run**.

This experiment follows the merged H12 latent-role substrate and the merged strict root-seeded random-partition conformance result. It asks whether an unnamed role learned only from small development graphs can transfer, unchanged, to larger and partially altered graphs when local degree and two-hop statistics are no longer reliable.

The experiment remains shadow-only. It adds no production routing, autonomous action authority, automatic ontology promotion, `Live` registry state, or persistent production ontology mutation.

## Hypothesis

A proof-carrying role based on a bounded global cut-and-context signature can be induced from small opaque development graphs, independently reconstructed, frozen, and reused to identify the functionally equivalent bridge entity in larger unseen graphs despite:

- graph-size growth;
- longer paths;
- denser distractors;
- edge subdivision;
- harmless node duplication;
- partial observation of noncritical edges;
- vocabulary permutation and isomorphic relabeling;
- local degree changes;
- local two-hop motif matches that do not preserve global function.

The validated role must expose evidence that unchanged H11/H10 validation can turn into a timely executable certificate. Superficial controls must not recover the target effect.

## Claim boundary

A `PASS` supports only bounded structural-role transfer under the frozen synthetic graph transformations. It does not establish natural-language concept learning, unrestricted semantic equivalence, open-world ontology induction, AGI, consciousness, or safe production self-modification.

## Frozen representation

Every graph contains only:

```text
StructuralGraph {
    graph_id,
    opaque Atom nodes,
    directed edges
}
```

Node text is unique per graph and semantically meaningless to the mechanism.

## Frozen transfer-invariant signature

For each node, independently compute:

```text
TransportRoleSignature {
    source_reachable,
    sink_reachable,
    upstream_sources_ge_2,
    downstream_sinks_ge_2,
    complete_reachable_pair_cut,
    lost_reachable_pairs_ge_4,
    has_convergent_ancestor,
    has_divergent_ancestor,
    has_convergent_descendant,
    has_divergent_descendant,
    upstream_depth_ge_2,
    downstream_depth_ge_2,
}
```

Definitions:

- source: in-degree zero;
- sink: out-degree zero;
- convergent: in-degree at least two;
- divergent: out-degree at least two;
- reachable source/sink sets are computed by deterministic graph traversal;
- `complete_reachable_pair_cut` is true only when removing the candidate node eliminates every previously reachable pair between that node's upstream source set and downstream sink set;
- `lost_reachable_pairs_ge_4` is true when at least four such source/sink pairs are destroyed;
- depth predicates use shortest directed path distance and are thresholded, not stored as exact counts.

The signature contains no task outcome, evidence score, objective atom, family label, split label, human role name, or future identity.

## Frozen target role

The development corpus contains one recurring unnamed signature with:

```text
source_reachable = true
sink_reachable = true
upstream_sources_ge_2 = true
downstream_sinks_ge_2 = true
complete_reachable_pair_cut = true
lost_reachable_pairs_ge_4 = true
has_convergent_ancestor = true
has_divergent_ancestor = false
has_convergent_descendant = false
has_divergent_descendant = true
upstream_depth_ge_2 = true
downstream_depth_ge_2 = true
```

No role label or target member is supplied to induction.

A second recurring, independently valid but target-irrelevant role is a complete cut with the opposite context polarity: divergent ancestry and convergent descendants.

## Development corpus

The role is induced once from exactly 12 small development graphs:

```text
3 topology families × 4 opaque graph instances
12–20 nodes per graph
limited distractor density
complete observation
unique node names and graph ids
```

A candidate signature is proposed only when it occurs in all 12 development graphs with exactly one member per graph.

The proposer enumerates every node signature and every recurring exact signature group. It receives no requested role count.

## Independent validation

The validator receives only the raw development corpus and a proposed proof. It independently recomputes:

- graph validity;
- canonical node and edge ordering;
- every node signature;
- every recurring signature group;
- exact member instances;
- distinct supporting graph ids;
- role id;
- development-scope digest;
- proof digest;
- budget counters.

It must reject inserted, removed, substituted, reordered, foreign-scope, or identity-permuted membership.

Only the validator may return an opaque `ValidatedTransportRoleCertificate`.

## Frozen evaluation cohort

The validated role certificate is frozen before any evaluation graph is generated or scored.

Exactly 56 evaluation roots are used:

```text
training sanity: 2 families × 8 roots = 16
holdout:         1 family  × 8 roots = 8
future:          4 families × 8 roots = 32
total:                                      56
```

The seven families are opaque to induction and validation. Family and split labels are used only by the external report aggregator.

## Frozen future transformations

Each evaluation root applies a deterministic combination selected by family index and root index:

1. unseen graph sizes from 40 through 180 nodes;
2. upstream and downstream path subdivision from one through six inserted nodes;
3. four through forty irrelevant branches;
4. distractor density from 0.10 through 0.45 relative to node count;
5. harmless duplication of non-cut local motifs;
6. partial observation that removes exactly 10% of lexicographically selected noncritical distractor edges while preserving every edge on a target source-to-sink path;
7. surface vocabulary permutation;
8. isomorphic insertion-order reversal;
9. target local in/out-degree variation between one and three while preserving the global cut role;
10. extra bypasses around decoys but never around the target role.

## Frozen causal construction

Each root begins with executable state:

```text
Fact(source)
Rule(source -> middle)
```

The target role's recognized transfer member is `middle`.

The mixed evidence table contains:

- four target-member intervention episodes supporting `middle -> goal`;
- five valid-irrelevant-role episodes supporting an irrelevant rule;
- four local-degree-decoy episodes;
- four two-hop-motif-decoy episodes;
- four rewired-decoy episodes;
- four vocabulary-decoy episodes;
- deterministic outcome noise that preserves the target rule only after correct role projection.

Role-conditioned evidence remains inert. It must pass unchanged H11 frontier discovery, unchanged H10 ranking and independent validation, and ordinary certificate admission before three H9 closure scans.

## Frozen paths and controls

Every root evaluates twice from fresh state:

1. `validated_transport_role`
   - frozen independently validated certificate;
   - transfer signature recomputed on the evaluation graph;
   - exact recognized members only;
   - unchanged H11/H10 validation and timely admission.

2. `exact_h12_fingerprint_baseline`
   - diagnostic exact-local H12 fingerprint transfer;
   - no H13 pass gate depends on it.

3. `local_degree_matched_decoy`
   - same direct in/out degree as the target;
   - not a complete source/sink cut.

4. `two_hop_motif_matched_decoy`
   - same frozen two-hop local pattern;
   - an alternate bypass destroys global cut equivalence.

5. `degree_preserving_rewire`
   - preserves direct degree and node count;
   - rewiring destroys the complete-pair cut.

6. `vocabulary_only_similarity`
   - shares a deterministic textual prefix with the target;
   - no structural certificate.

7. `foreign_family_certificate`
   - valid certificate from a disjoint development scope;
   - must be rejected before projection.

8. `valid_irrelevant_role`
   - independently induced and validated;
   - may validate and admit its own H11 rule;
   - target objective must remain false.

9. `role_identity_permutation`
   - correct proof payload bound to the wrong opaque role id;
   - must be rejected by independent recomputation.

10. `delayed_role_admission`
    - correct H11 certificate is admitted only after all three closure scans.

11. `random_same_cardinality_grouping`
    - deterministic root-seeded evidence grouping;
    - same episode count as the target projection;
    - at least one target and one non-target episode;
    - no resampling after scoring.

12. `payload_only`
    - serialized proof and certificate fields are available;
    - no validated certificate is admitted.

13. `oracle_role`
    - external exact target member;
    - same H11/H10/admission/closure path;
    - ceiling control only.

14. `unpartitioned_full_evidence`
    - unchanged H11/H10 over all mixed evidence.

## Frozen computation budgets

Per root and path, record:

- graph validation node and edge scans;
- signature node evaluations;
- reachability traversals;
- removal-counterfactual traversals;
- candidate groups considered;
- proof-validation signature evaluations;
- transfer signature evaluations;
- evidence episodes scanned;
- H11 frontier scans;
- H10 candidate/episode evaluations;
- admission opportunities;
- successful admissions;
- closure scans;
- objective checks.

All executable paths receive:

```text
one H11 proposal opportunity
one independent H11 validation opportunity
one pre-closure admission slot
three closure scans
one objective check
```

The delayed path receives its admission slot after closure. Payload-only and invalid-certificate paths must report an empty trusted frontier rather than fabricate candidates.

## Frozen replay requirement

Every root/path pair executes twice from fresh state. Exact equality is required for:

- candidate ordering;
- proof and certificate digests;
- recognized members;
- projected evidence ids;
- H11 proof and certificate identity;
- state signature;
- all budget counters;
- objective result;
- rejection reason;
- terminal classification inputs.

## PASS gate

`PASS` requires all of the following:

```text
validated transport role objective success:
  training 16/16
  holdout 8/8
  future 32/32
  every future family 8/8

oracle objective success:
  56/56

all target-negative controls:
  0/56 target successes

local-degree, two-hop, rewired, vocabulary, random, payload-only,
unpartitioned, delayed, foreign, identity-permuted, and valid-irrelevant paths:
  target success 0/56

valid irrelevant role:
  independent certificate validation 56/56
  timely admission of its own rule 56/56

foreign-family certificate:
  rejected 56/56

role identity permutation:
  rejected 56/56

primary transfer recognition:
  exactly one target member 56/56

superficial decoy rejection:
  local-degree decoy excluded 56/56
  two-hop decoy excluded 56/56
  rewired decoy excluded 56/56

future structural transformations:
  all declared transformations exercised

replay:
  exact for every root/path pair

budget accounting:
  exact for every root/path pair

state/provenance invariants:
  true for every executable path
```

## Terminal classification

Allowed terminal classifications are exactly:

```text
PASS
REJECTED
CONTROL_FAILURE
INFRASTRUCTURE_FAILURE
INCONCLUSIVE
```

Classification order:

1. `INFRASTRUCTURE_FAILURE` for compilation failure, malformed fixture, incomplete report, replay mismatch, budget-accounting mismatch, or provenance invariant failure;
2. `REJECTED` if the primary mechanism fails any required split or future family;
3. `CONTROL_FAILURE` if any target-negative control succeeds, any foreign/permuted proof is accepted, or any superficial decoy is recognized as the target role;
4. `INCONCLUSIVE` if the oracle fails or a declared transformation is not exercised;
5. `PASS` only when every frozen gate is true.

The first complete verdict-producing committed-code run is terminal. No fixture, signature field, threshold, control, budget, split, or gate may be changed to rescue that result.
