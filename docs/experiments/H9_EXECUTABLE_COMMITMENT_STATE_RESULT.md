# H9 Proof-Carrying Executable Commitment State — Frozen Result

Status: **PASS**

First complete verdict-producing run:

```text
GitHub Actions workflow: CHARGE CI
run id: 29143168794
run number: 92
head commit: 1e075b47cb79f9074b359cdd9c3aa4178e6fc0ac
artifact: charge-real-component-diagnostics
artifact id: 8245844170
```

The preregistration in `H9_EXECUTABLE_COMMITMENT_STATE.md` was committed before the implementation and before this run. No cohort, path, budget, threshold, gate, or terminal-classification rule was changed after observing the result.

## Terminal classification

```text
PASS
```

## Frozen support

```text
training roots: 16
holdout roots:   8
future roots:   32
future families: 4
total roots:    56
```

## Primary objective result

| Split | Stateful | Endpoint blind | Text only | Scalar only | Rewired | Random valid | Invalid matched | Delayed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Training | 16/16 | 0/16 | 0/16 | 0/16 | 0/16 | 0/16 | 0/16 | 0/16 |
| Holdout | 8/8 | 0/8 | 0/8 | 0/8 | 0/8 | 0/8 | 0/8 | 0/8 |
| Future | 32/32 | 0/32 | 0/32 | 0/32 | 0/32 | 0/32 | 0/32 | 0/32 |

The future causal margins are therefore all exactly:

```text
stateful success rate - control success rate = 1.0
```

for endpoint-blind, text-only, scalar-only, rewired, matched random valid, invalid matched, and delayed controls.

## Same-root and validity controls

```text
rewired compile rejection:
  training 16/16
  holdout   8/8
  future   32/32

structurally matched invalid compile rejection:
  training 16/16
  holdout   8/8
  future   32/32
```

The irrelevant root-local `random_valid` path did perform a valid mutation of the same delta type and used the same successful-state-mutation count as the stateful path on every split, but reached the objective on 0 roots.

This is the strongest control in H9 against the claim that any extra valid mutation or additional state activity is sufficient.

## Future-family transfer

| Future family | Roots | Stateful | Maximum control success rate | Pass |
|---|---:|---:|---:|---|
| `cell_signaling` | 8 | 1.0 | 0.0 | yes |
| `network_routing` | 8 | 1.0 | 0.0 | yes |
| `supply_chain` | 8 | 1.0 | 0.0 | yes |
| `orbital_dynamics` | 8 | 1.0 | 0.0 | yes |

No family-specific parameter was fit. The mechanism did not receive the family label.

## Determinism and accounting

Every root/path execution was reconstructed and executed twice.

Frozen path budget:

```text
operation calls per path:             2
existing CausalEngine calls per path: 1
compile transition slots per path:    1
executor scans per path:              3
total transition opportunities:       4
independent objective checks:          1
```

Observed:

```text
budgets exact:       true
exact replay:        true
state invariants:    true
CI compilation:      success
commitment tests:    success
full CHARGE CI job:  success
```

The final canonical executable-state signature was part of exact replay equality.

## What changed computationally

The successful path did not merely preserve more context.

Before Operation A:

```text
Fact(source)
Rule(source -> middle)
raw witness(middle -> goal)
```

The raw witness was inert to Operation B.

Operation A used the existing `CausalEngine` interface to produce a typed proposal and the H9 state validator admitted:

```text
Rule(middle -> goal)
```

as an executable commitment with witness provenance.

That state mutation changed the later transition relation:

```text
source -> middle -> goal
```

became executable inside the fixed three-scan budget.

When the same relation was retained only as serialized text or scalar history, the later executor still could not use it. When a valid but irrelevant rule was committed, the later executor still could not reach the target. When the correct rule was applied after the executor, the target was not reached. When same-root witness incidence was destroyed, validation rejected the delta.

## Scientific interpretation

H9 supplies a positive answer to the narrow question that H4-H8 had not established:

> Starfire can contain a deterministic shadow substrate in which one operation creates a validated, provenance-carrying executable state transformation that is causally necessary for a later fixed-budget operation to perform a computation unavailable from the original executable state.

The result identifies a concrete architectural distinction between **context** and **computational state**.

A transcript can describe a relation. PECS can commit that relation into a typed transition system that changes which later computations are enabled.

## Important limitations

H9 is a deliberately constructed substrate-level falsification probe.

It does **not** show that:

- legacy Starfire reasoning resolvers already use PECS;
- the causal relation was autonomously discovered rather than supplied as a witnessed observation;
- Starfire can invent new operator types;
- natural-language reasoning is now generally compositional;
- latent concepts should be automatically promoted;
- live routing should be changed;
- H8's legacy resolver nondeterminism is globally repaired;
- Starfire is AGI, conscious, or human-level.

The positive result is architectural, not a claim of autonomous general intelligence.

## Consequence for the research program

The next scientifically justified step is not automatic ontology promotion.

The new question is whether a real Starfire cognitive component can **propose useful PECS deltas from non-privileged observations** and whether a later, independently useful component can exploit those commitments on held-out tasks without a hand-authored target bridge.

A future experiment should preserve H9's validated state-transition and provenance invariants while replacing the direct witnessed-rule compiler with a learned or inferred proposal mechanism under controls that distinguish:

```text
correct executable commitment
plausible but wrong commitment
text-only explanation
random valid commitment
rewired commitment
no commitment
```

That experiment must be preregistered independently. H9 itself is frozen at `PASS`.
