# H11 Graph-Discovered Relation Induction

Status: **frozen preregistration before implementation and before the first verdict-producing run**.

H11 is shadow-only. It does not alter live routing, automatic ontology promotion, H9/H10 accepted substrate semantics, or prior frozen verdicts.

## Research question

Can Starfire discover the candidate relation vocabulary needed for a useful executable rule from a larger mixed intervention-evidence graph, rather than receiving a developer-supplied antecedent/consequent candidate universe, while preserving independent proof recomputation, exact matched budgets, deterministic replay, and causal necessity of executable admission?

## Motivation

H10 passed after removing the directly witnessed target bridge, but the proposer still received a privileged `3 x 3` candidate universe. That left the strongest remaining shortcut:

```text
the useful antecedent and consequent atoms were already named as candidates
```

H11 removes that privilege.

The proposer receives only a mixed graph corpus:

```text
- intervention episodes
- observed outcome sets
- seed executable facts/rules used only by the later PECS executor
- no antecedent candidate list
- no consequent candidate list
- no verifier target
- no family or split label
- no hidden correct relation
```

The H11 discovery mechanism must first construct a deterministic relation vocabulary from graph incidence, then pass that discovered universe into the frozen H10-style evidence-bound scoring and independent proof validation path.

## Frozen mechanism: Graph-Discovered Candidate Frontier (GDCF)

### Raw graph

Each H11 root contains exactly 24 distinct atoms and 16 intervention episodes.

The raw evidence graph is the directed bipartite incidence graph:

```text
intervention atom -> observed outcome atom
```

An edge exists when an outcome occurs in an intervention episode.

The graph contains:

```text
1 target bridge pair
1 valid but irrelevant bridge pair
4 structured distractor pairs
12 low-support/noise atoms
```

The useful pair is not marked and is not supplied in a candidate list.

### Candidate-frontier discovery

The discovery operation scans the raw evidence graph and derives candidate atoms by frozen structural eligibility rules.

An intervention atom enters the antecedent frontier iff:

```text
intervention_episode_count >= 2
```

An outcome atom enters the consequent frontier iff:

```text
outcome_incidence_count >= 2
```

No score, target information, path identity, family label, or later PECS reachability information is used during frontier construction.

The frontier is canonically sorted by atom identity.

The Cartesian product of discovered antecedents and discovered consequents is the discovered candidate-rule universe.

### Frozen graph shape

Every root is constructed so that the canonical discovered frontier contains exactly:

```text
6 antecedent atoms
6 consequent atoms
36 candidate rules
16 evidence episodes
```

The target rule is exactly one of those 36 rules, but neither endpoint is privileged to the discovery mechanism.

The valid-but-irrelevant control rule is also inside the same discovered universe.

### Frozen scoring law

H11 reuses the H10 deterministic candidate score without modification:

```text
for each candidate rule A -> B and each episode E:

if E.intervention == A and B is observed:
    +3 score, +1 support
else if E.intervention == A and B is absent:
    -4 score, +1 contradiction
else if B is observed under another intervention:
    -1 score
else:
     0
```

The winning rule must satisfy the unchanged H10 certificate gates:

```text
score >= 10
support >= 4
contradictions == 0
winner margin >= 6
```

### Proof object

The H11 proof contains:

```text
proof_id
frontier_digest
discovered_antecedents
discovered_consequents
proposed_rule
score
support
contradictions
runner_up_score
supporting_evidence_ids
contradicting_evidence_ids
```

### Independent validation

The validator receives the raw graph and proof.

It must independently recompute, from scratch:

```text
1. intervention counts
2. outcome incidence counts
3. discovered antecedent frontier
4. discovered consequent frontier
5. canonical frontier digest
6. all 36 candidate rules
7. all 36 x 16 = 576 candidate/episode scores
8. unique winner
9. H10 score/support/contradiction/margin gates
10. every proof field
```

Only then may it issue an opaque validated certificate.

A foreign, counterfeit, frontier-tampered, or structurally invalid proof must be rejected after the same full recomputation budget.

## PECS integration

H11 does not modify H9 PECS transition semantics or H10 certificate admission semantics.

A valid H11 certificate admits one inferred executable rule into the evidence-bound commitment state.

The later executor:

```text
- cannot read raw evidence
- cannot read the discovered frontier
- cannot read proof text
- cannot read scalar scores
- cannot read the verifier target
- receives exactly 3 canonical derivation scans
```

The only path by which discovery can affect later reasoning is successful certificate admission into executable state.

## Root construction

Each root contains:

```text
source
middle
goal
irrelevant_source
irrelevant_goal
4 structured distractor antecedents
4 structured distractor consequents
12 low-support/noise atoms
```

Initial executable state contains:

```text
Fact(source)
Fact(irrelevant_source)
Rule(source -> middle)
```

The target objective is `goal`.

The raw evidence graph contains enough intervention evidence for the discovery mechanism to include `middle` and `goal` in the frontier and for the scoring law to select:

```text
middle -> goal
```

The exact target rule is not separately supplied anywhere to the proposer or validator.

The valid irrelevant path uses a separately constructed same-root evidence graph in which the strongest relation is:

```text
irrelevant_source -> irrelevant_goal
```

It must pass discovery, proof validation, and admission while remaining irrelevant to the target objective.

## Frozen cohort

Exactly seven domain-vocabulary families with eight roots each:

```text
2 training families = 16 roots
1 holdout family    = 8 roots
4 future families   = 32 roots
--------------------------------
total               = 56 roots
```

The mechanism receives no family or split label and fits no parameter from training.

Frozen family order:

```text
training:
  thermal_systems
  transport_networks

holdout:
  ecological_flows

future:
  cellular_regulation
  manufacturing_processes
  software_dependency
  watershed_dynamics
```

## Frozen paths and controls

Every root executes exactly nine paths.

### 1. Stateful graph-discovered inference

Discover the candidate frontier from the same-root mixed evidence graph, infer the winning rule, independently validate the complete discovery-and-ranking proof, admit the resulting certificate, then execute the three-scan PECS closure.

### 2. Endpoint blind

Run full frontier discovery, inference, and validation, but do not admit the certificate. Execute closure on the original executable state.

### 3. Frontier text only

Run full discovery/inference/validation and serialize the exact discovered frontier and proof to text, but do not admit executable state.

### 4. Scalar only

Run full discovery/inference/validation and preserve only frontier size, winning score, and winner margin. Do not admit executable state.

### 5. Foreign proof

Run the full same-root discovery/inference budget, but validate the next root's proof against the current root's raw graph. The validator must reject it after full frontier and ranking recomputation.

### 6. Frontier tamper

Run the full same-root discovery/inference budget, then mutate one atom in the proof's discovered frontier while leaving the proposed winning rule and scalar scores unchanged. The validator must reject it after full recomputation.

### 7. Valid irrelevant discovery

Run a same-root mixed evidence graph whose strongest discovered relation is the valid irrelevant rule. It must discover a frontier, infer a unique winner, pass full independent validation, and admit one executable rule, but must not reach the target objective.

### 8. Counterfeit proof

Run full discovery/inference, then increment the claimed winning score by one. The validator must reject it after full recomputation.

### 9. Delayed correct admission

Run full correct discovery/inference/validation before execution for matched compute accounting, execute the three PECS closure scans without admission, then admit the valid certificate after the search window.

## Frozen budgets

For every root and every path:

```text
frontier discovery passes                = 1 proposer + 1 validator
raw evidence episodes scanned/pass       = 16
proposer graph-incidence scans            = 16
validator graph-incidence scans           = 16
discovered antecedents                    = 6
discovered consequents                    = 6
discovered candidate rules                = 36
proposal candidate/episode evaluations    = 576
validation candidate/episode evaluations  = 576
validation recomputations                 = 1
admission slots                            = 1
executor scans                             = 3
independent objective checks               = 1
```

A rejected proof still consumes the full validator discovery and 576-evaluation recomputation budget.

An unadmitted or delayed certificate still consumes one admission slot.

An empty executor scan still consumes one search slot.

Any budget mismatch is an infrastructure failure and cannot yield a positive scientific verdict.

## Deterministic replay

Every root/path execution is repeated from a freshly reconstructed state with identical inputs.

Exact equality is required for:

```text
frontier atoms
frontier digest
proof
validation result
certificate acceptance/rejection
budget counters
objective result
canonical executable-state signature
```

Any mismatch is `REPLAY_FAILURE`.

## Independent objective witness

The verifier is outside discovery, inference, validation, and execution.

It receives only:

```text
final executable state
expected target atom
```

Success is exact target-fact membership.

It cannot inspect:

```text
raw graph
frontier
proof
scores
family label
split label
path identity
operation self-report
```

## Frozen metrics

For each split and path report:

```text
root count
objective success count/rate
frontier discovery success count
certificate acceptance count
certificate rejection count
mean admitted executable mutations
exact budget status
exact replay status
state/provenance invariant status
```

Additionally report:

```text
frontier exactness rate
foreign-proof rejection rate
frontier-tamper rejection rate
counterfeit rejection rate
valid-irrelevant certificate acceptance rate
```

For each future family report:

```text
stateful success rate
maximum control success rate
```

## Frozen terminal classification

`PASS` requires all of the following:

```text
training roots == 16
holdout roots == 8
future roots == 32
future families == 4

frontier size == 6 x 6 on every root/path

stateful training success rate == 1.0
stateful holdout success rate == 1.0
stateful future success rate == 1.0

all eight non-stateful future controls success rate == 0.0

foreign proof rejected on every root in every split
frontier-tampered proof rejected on every root in every split
counterfeit proof rejected on every root in every split
valid irrelevant certificate accepted on every root in every split

all four future families:
  stateful success rate == 1.0
  maximum control success rate == 0.0

all budgets exact
all replays exact
all state/provenance invariants valid
```

If the executable runs correctly but any scientific gate fails, terminal classification is `REJECTED`.

If the infrastructure or budget contract fails, terminal classification is `CONTROL_FAILURE`.

If exact replay fails, terminal classification is `REPLAY_FAILURE`.

No threshold, evidence shape, frontier rule, cohort, control, budget, gate, or classification rule may be changed after the first complete verdict-producing run to rescue a failure.

## Claim boundary

A `PASS` would support only the following narrow claim:

> Under the frozen symbolic mixed-graph regime, Starfire can discover a candidate relation frontier from raw graph incidence without being given the target relation endpoints as a candidate universe, infer a useful rule from that discovered frontier, survive independent recomputation of both discovery and evidence ranking, and use certificate admission as a causally necessary intermediate executable state across unseen vocabularies.

It would not establish:

```text
open-world causal discovery
natural-language evidence extraction
learned frontier criteria
learned scoring law
unbounded relation invention
automatic ontology induction
live routing readiness
AGI
consciousness
human-level cognition
```

The remaining privilege after a possible H11 `PASS` would be the fixed evidence schema, fixed frontier eligibility rule, fixed scoring law, and finite graph construction.
