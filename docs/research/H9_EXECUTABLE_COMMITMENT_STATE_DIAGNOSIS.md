# H9 architectural diagnosis: executable commitments, not preserved context

Status: design diagnosis for H9. This document precedes the H9 implementation and verdict-producing executable.

## Binding evidence

The H4-H8 sequence constrains the next mechanism.

- H4 found one useful transferable routing distinction, but a feature-destroyed control retained too much utility for automatic ontology promotion.
- H5 could not identify a stable non-memory reasoning-versus-causal regime under the frozen contract.
- H6 showed that disagreement-conditioned residual structure did not create a transferable reasoning primitive beyond matched random interactions.
- H7 found no useful continuation-generated quotient refinement and measured strong right absorption.
- H8 attempted to preserve the first operation's output and objective history for the second operation, but its scientific verdict was invalidated by resolver-internal nondeterminism. Independently of that control failure, H8 still exposed the key interface problem: the carried object was descriptive context, not an executable mutation of a shared computational state.

H9 therefore does not add another classifier over CHARGE, another transcript format, a larger prompt, or automatic latent-concept promotion.

## Exact missing invariant

The current system can preserve observations, text, scores, CHARGE magnitude, and history without requiring those objects to change the domain of legal future computation.

The missing invariant is:

> A successful cognitive operation must be able to create a validated executable commitment whose existence changes the set of later enabled state transitions, while retaining enough provenance to causally remove that capability again.

In H9, a state is not considered transformed merely because more information is serialized. It is transformed only when a typed delta passes validation and changes the executable transition relation.

Formally, for state `S`, operation `a`, and later operation `b`, H9 seeks a case where:

```text
Delta_a = a(read_a(S))
S'      = apply_validated(S, Delta_a)
Enabled_b(S') != Enabled_b(S)
```

and an independent objective witness shows:

```text
U(b(S')) > U(b(S))
```

while controls that preserve text, scalar history, operation count, search budget, or delta shape but destroy the validated same-root commitment do not recover the gain.

## Primary primitive: Proof-Carrying Executable Commitment State

H9 introduces one primary mechanism: **Proof-Carrying Executable Commitment State** (PECS).

PECS has four object classes:

1. **Raw witnessed observations** — immutable evidence records. They are inert to later execution until compiled.
2. **Executable facts** — typed atoms that can satisfy operator preconditions.
3. **Executable rules** — typed implications that can enable deterministic derivations.
4. **Provenance** — every non-seed commitment records the witness or prior commitments that authorize it.

The initial H9 transition language is intentionally minimal:

```text
CompileWitnessedRule(witness_id, exact_rule)
DeriveFact(rule_commitment_id, support_fact_id)
```

This is not intended as a complete reasoning language. It is the smallest closed substrate that can falsify the architectural claim that Starfire cannot create a causally necessary executable intermediate state.

## State and transition semantics

Let:

```text
O = immutable raw witnessed observations
F = executable committed facts
R = executable committed rules
P = provenance relation over commitments
S = (O, F, R, P)
```

The operation family is:

```text
A = { CompileWitnessedRule, DeriveFact }
```

The transition function is:

```text
T : S x Delta -> S' or Reject
```

Validation is part of the transition, not an after-the-fact score.

### CompileWitnessedRule

A compile delta is valid only when the referenced raw witness exists and the proposed rule exactly matches the rule authorized by that witness.

A valid transition adds the executable rule to `R` with witness provenance.

A mismatched, rewired, or invented rule is rejected and does not mutate state.

### DeriveFact

A derivation delta is valid only when:

- the referenced rule commitment exists;
- the referenced support commitment is a fact;
- the support fact exactly satisfies the rule antecedent;
- the consequent is not already committed.

A valid transition adds the consequent fact to `F` with provenance pointing to both the rule and support fact.

## Determinism contract

PECS uses canonical symbolic atoms, ordered maps, monotonic commitment IDs, canonical derivation ordering, and a stable state signature.

The H9 probe does not invoke the H8 reasoning path whose `HashSet` iteration escaped the declared seed boundary. H9 therefore defines deterministic operation semantics at the new substrate boundary instead of treating an environment reset as sufficient to control arbitrary resolver internals.

This is not a claim that every legacy Starfire resolver is now deterministic. It is a narrower foundation contract: the operation semantics tested by H9 are deterministic and auditable.

## Why this is not a blackboard or transcript

A transcript can contain the sentence `B causes C` while leaving every later operator exactly as capable as before.

PECS distinguishes:

```text
text describing B -> C
```

from:

```text
validated executable rule commitment B -> C
```

The later executor in H9 cannot read transcript text or raw observations. It reads only executable commitments. Therefore a text-only control can carry the same semantic sentence and still fail to change the enabled transition set.

## Why this is not automatic ontology induction

H9 does not learn categories, promote latent concepts, or alter live routing.

The experiment tests a prior architectural prerequisite: whether Starfire can possess a state in which an earlier operation creates an executable transformation that is causally necessary for a later operation.

Only after such a substrate survives controls would it make sense to ask whether learned operators or ontology induction can safely create new commitments.

## Claim boundary

A successful H9 result would support only:

> Starfire contains a deterministic shadow substrate in which a validated typed operation can create an executable, provenance-carrying state change that is causally necessary for a later bounded operation to reach an independently verified objective under matched controls.

It would not establish:

- general autonomous reasoning;
- learned operator invention;
- automatic ontology induction;
- production-ready live routing;
- AGI, consciousness, or human-level cognition.
