# H10 Evidence-Bound Rule Induction

Status: **frozen preregistration before implementation and before the first verdict-producing H10 run**.

H10 is stacked on the accepted H9 shadow substrate. It does not alter live routing or automatic ontology promotion. H9 remains a narrow positive result: a validated executable commitment can create state-dependent composition. H10 asks the next question: can the missing executable rule be inferred from non-privileged evidence rather than supplied as a directly witnessed rule?

## Research question

Can Starfire infer a causally useful executable rule from a fixed, target-blind intervention-evidence table, produce an auditable proof object, survive independent proof validation, commit the inferred rule into PECS, and thereby enable later bounded reasoning under matched controls and future-domain transfer?

## Claim under test

For an initial executable state `S0`, evidence bundle `E`, fixed proposal mechanism `I`, proof validator `V`, PECS admission `A`, and bounded closure executor `C`:

```text
p = I(E)
cert = V(E, p)
S1 = A(S0, cert)
S2 = C(S1)
```

H10 asks whether:

```text
objective(S2) = success
```

while matched paths that preserve proposal text, scalar score, operation count, scoring budget, admission opportunity, and executor budget but remove or corrupt the validated executable commitment fail.

A passing result must not depend on access to the held-out target atom, task-family label, training label, hidden correct rule, or verifier result.

## H9 dependency

H10 preserves the H9 Proof-Carrying Executable Commitment State semantics:

- immutable evidence is not executable state;
- only validated typed commitments affect later executable closure;
- derived facts retain rule/fact provenance;
- deterministic canonical ordering and replay equality remain required.

H10 does **not** use `WitnessedRule` as the source of the target bridge. The exact target bridge is not inserted as a raw witnessed rule.

## Primary mechanism: evidence-bound causal rule induction

### Evidence representation

Each root contains a fixed `InferenceProblem`:

```text
candidate antecedents: 3 symbolic atoms
candidate consequents: 3 symbolic atoms
intervention episodes: 10
```

Each episode contains:

```text
EvidenceEpisode {
    evidence_id,
    intervention_atom,
    observed_outcome_atoms
}
```

The proposer receives the candidate universe and the ten episodes only.

It does not receive:

```text
verifier target atom
family label
split label
hidden correct rule
initial executable closure path
future success
control identity
```

### Candidate universe

Exactly nine one-premise rules are scored:

```text
3 antecedents × 3 consequents = 9 candidates
```

Self-loops do not occur in the frozen construction.

Candidate ordering is canonical by `(antecedent, consequent)`.

### Frozen scoring rule

For candidate rule `a -> b`, scan all ten episodes.

For each episode:

```text
if intervention == a and b is observed:  +3, support += 1
if intervention == a and b is absent:    -4, contradiction += 1
if intervention != a and b is observed:  -1 baseline penalty
otherwise:                                 0
```

The proposer reports:

```text
proposed rule
integer score
support count
contradiction count
runner-up score
supporting evidence ids
contradicting evidence ids
```

Candidate ties are broken canonically, but proof validation requires a sufficient score margin, so a tied proposal cannot be certified.

### Frozen certificate gates

Independent validation recomputes all nine candidate scores from the raw evidence and accepts a proof only when all of the following are exact:

```text
proposed rule is the unique highest-scoring candidate
reported score equals recomputed score
reported support equals recomputed support
reported contradiction count equals recomputed contradiction count
reported runner-up score equals recomputed runner-up score
reported support/contradiction evidence ids match recomputation
score >= 10
support >= 4
contradictions == 0
score - runner_up_score >= 6
```

A successful validator returns an opaque `ValidatedInferenceCertificate`. The certificate cannot be directly constructed by the experiment harness.

### PECS admission

A validated certificate may be committed as one executable rule with inference provenance containing:

```text
certificate/proof id
supporting evidence ids
```

An unvalidated proof, transcript, scalar score, or raw evidence bundle cannot create an executable rule.

## Frozen task construction

Each root has unique atoms:

```text
source
middle
goal
decoy_source
decoy_goal
noise_source
noise_goal
```

Initial executable state contains:

```text
Fact(source)
Rule(source -> middle)
```

The independent objective is:

```text
Fact(goal) exists after the bounded closure operation
```

The target atom is held only by the external objective checker.

The inference candidate universe is:

```text
antecedents = [middle, decoy_source, noise_source]
consequents = [goal, decoy_goal, noise_goal]
```

The proposer is not told which candidate is useful to the executable state or verifier.

### Ten frozen evidence episodes per root

The evidence table contains:

```text
4 episodes: intervention middle       -> goal observed
3 episodes: intervention decoy_source -> decoy_goal observed
1 episode:  intervention decoy_source -> decoy_goal absent
1 episode:  intervention noise_source -> noise_goal observed
1 episode:  intervention noise_source -> noise_goal absent
```

No episode is a `WitnessedRule`, and no hidden field marks `middle -> goal` as the target bridge.

Under the frozen scoring rule:

```text
middle -> goal             score 12, support 4, contradictions 0
decoy_source -> decoy_goal score  5, support 3, contradictions 1
all remaining candidates   lower
```

The validator, not the experiment harness, decides whether the inferred proposal is admissible.

## Bounded executable closure

After the admission slot, the H9 executor performs exactly three canonical derivation scans.

On a successful stateful path:

```text
scan 1: source -> middle
scan 2: middle -> goal
scan 3: empty or irrelevant
```

Without an executable `middle -> goal` rule, the same executor can derive `middle` but not `goal`.

## Frozen cohort

Exactly seven domain vocabularies with eight unique roots each:

```text
2 training families  = 16 roots
1 holdout family     = 8 roots
4 future families    = 32 roots
total                = 56 roots
```

The mechanism fits no parameter on training. Training/holdout/future splits exist only to test chronological/domain transfer under a frozen mechanism.

## Frozen paths and controls

Every root executes exactly eight paths.

### 1. Stateful inferred commitment

Infer from the same-root evidence, independently validate the proof, admit the certificate as an executable rule, then run the three-scan closure executor.

### 2. Endpoint blind

Run the same inference and proof validation but do not admit the certificate. Execute closure on the untouched executable state.

### 3. Text-only proposal

Run inference and validation, serialize the exact proof/certificate summary to text, do not admit executable state, then run closure.

The executor cannot read the text.

### 4. Scalar-only proposal

Run inference and validation, retain only the candidate score and margin as scalars, do not admit executable state, then run closure.

The executor cannot read the scalars.

### 5. Same-root incidence destroyed

Run same-root inference for budget accounting, but replace the proof submitted to the validator with the next root's proof under a deterministic cyclic permutation within the same split.

Validation is performed against the current root's evidence and candidate universe. The foreign proof must be rejected.

### 6. Semantically permuted but valid inference

Apply a deterministic root-local permutation exchanging `goal` and `decoy_goal` in the evidence outcomes while leaving candidate counts, episode count, intervention schedule, scoring budget, and candidate universe size unchanged.

Run inference and validation on the transformed evidence. This should produce a valid executable rule `middle -> decoy_goal`, causing a successful state mutation that is irrelevant to the held-out objective.

### 7. Counterfeit proof

Run correct inference, then increment the reported score by one while preserving the proposed rule and all other fields. Independent validation must reject the proof.

### 8. Delayed correct admission

Run correct inference and validation before closure for matched compute, execute the same three closure scans without admitting the certificate, then admit the correct certificate only after closure has finished.

This preserves the correct inferred rule while destroying the required causal ordering.

## Frozen budgets

For every root and every path:

```text
inference calls                         = 1
candidate rules scored                  = 9
evidence episodes per candidate         = 10
proposal scoring evaluations            = 90
proof validation full recomputations    = 1
validation scoring evaluations          = 90
admission slots                         = 1
executor scans                          = 3
independent objective checks            = 1
```

Total frozen inference/validation scoring evaluations per path:

```text
180
```

A rejected certificate consumes the same validation and admission slots.

An empty executor scan consumes the same search slot.

Any budget mismatch is an infrastructure failure.

## Deterministic replay

Every root/path execution is repeated from a freshly reconstructed state.

The following must match exactly:

```text
inferred rule or inference error
proof fields
validation acceptance/rejection
final objective result
all budget counters
final canonical executable-state signature
```

Any mismatch is `REPLAY_FAILURE`.

## Independent objective witness

The objective checker receives only:

```text
final executable commitment state
expected target atom
```

Success is exact membership:

```text
final_state.contains_fact(expected_target)
```

The checker cannot read proof text, score, family label, path identity, or operation self-report.

## Frozen metrics

For every split/path report:

```text
root count
success count and rate
inference success count
certificate acceptance count
certificate rejection count
mean admitted executable mutations
exact budget status
exact replay status
```

Also report:

```text
future result per family
stateful minus each control objective margin
same-root foreign-proof rejection rate
counterfeit-proof rejection rate
valid-permuted inference acceptance rate
inferred-rule identity distribution
```

## Frozen acceptance gates

`PASS` requires all of:

```text
training roots == 16
holdout roots == 8
future roots == 32
future families == 4

stateful training success rate == 1.0
stateful holdout success rate == 1.0
stateful future success rate == 1.0

endpoint blind future success rate == 0.0
text-only future success rate == 0.0
scalar-only future success rate == 0.0
same-root-incidence-destroyed future success rate == 0.0
valid-permuted future success rate == 0.0
counterfeit-proof future success rate == 0.0
delayed-admission future success rate == 0.0

same-root foreign proof rejection rate == 1.0 on train, holdout, future
counterfeit proof rejection rate == 1.0 on train, holdout, future
valid-permuted certificate acceptance rate == 1.0 on train, holdout, future

stateful success == 1.0 in each of 4 future families
maximum control success == 0.0 in each of 4 future families

all budgets exact
all state invariants hold
exact replay passes for every root/path
```

## Terminal classifications

```text
PASS
    all frozen gates pass

INFRASTRUCTURE_FAILURE
    any budget or invariant failure

REPLAY_FAILURE
    any exact replay mismatch

INFERENCE_FAILURE
    stateful proof induction or validation fails support/transfer gates

CONTROL_FAILURE
    stateful transfer passes but any causal control recovers objective success or a required rejection/acceptance control fails

TRANSFER_FAILURE
    training succeeds but holdout/future or per-family transfer fails

REJECTED
    all other scientific failures
```

## Scientific claim boundary

A `PASS` supports only:

> Under the frozen symbolic intervention-evidence regime, a target-blind deterministic proposer can infer a useful rule from non-privileged evidence, survive independent proof recomputation, commit that rule into PECS, and create a causally necessary executable intermediate state that transfers across unseen domain vocabularies under matched controls.

A `PASS` does **not** establish:

```text
open-world causal discovery
natural-language causal induction
learned operator invention
automatic ontology induction
live-routing readiness
AGI
consciousness
human-level cognition
```

## No rescue iteration

After the first complete verdict-producing H10 run:

```text
no threshold changes
no evidence-count changes
no score-weight changes
no candidate-universe changes
no control substitution
no split changes
no favorable-seed selection
```

Compile defects or contract-preserving implementation bugs may be repaired, but the frozen scientific contract may not be changed to rescue a negative result.
