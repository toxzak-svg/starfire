# A1 — Bounded Autonomous Kernel Foundation

**Status:** implementation contract and foundation probe  
**Scope:** shadow-only; no `Runtime::chat()` wiring  
**Authority ceiling:** reversible sandbox actions

## Research question

Can Starfire execute one deterministic observe → pressure → operator proposal →
authorized action → external outcome → independent discharge loop without giving
operators authority to judge their own success or mutate production state?

A1 is a foundation test, not an AGI claim and not yet a generality experiment.
This document records the implemented foundation contract. The follow-up
capability experiment must be separately preregistered before its verdict run.

## Mechanism

The new `autonomy` module introduces:

- executable goal state;
- hard step, action-cost, and compute-cost budgets;
- a typed action-authority broker;
- a common cognitive-operator contract;
- deterministic utility-based operator selection with stable tie-breaking;
- persistent CHARGE through `CognitiveCycleState`;
- objective progress measured only by the environment;
- discharge accepted only by `RelativeImprovementJudge`;
- a structured episode report and complete action trace.

The kernel is intentionally not connected to:

- `Runtime::chat()`;
- live runtime routing;
- automatic ontology promotion;
- persistent production writes;
- external side effects;
- source-code self-modification.

## Deterministic environment

`HiddenRuleEnvironment` selects a hidden Boolean rule from the episode seed.
The agent may:

1. inspect one bounded clue;
2. set a candidate value;
3. submit the candidate.

The environment alone owns the hidden value, objective progress, terminal state,
and final success judgment.

## Operator set

The executable probe registers three independently competing operators:

1. `inspect-hidden-rule`;
2. `apply-observed-clue`;
3. `submit-verified-candidate`.

All three use the same `CognitiveOperator` contract. They may predict effects and
request discharge, but they cannot directly change CHARGE, mark the goal solved,
or fabricate objective evidence.

## Foundation gates

Across 64 deterministic seeds:

- solve rate must equal `1.0`;
- every episode must use exactly three actions;
- denied action count must equal zero;
- each action must remain at or below its declared cost;
- all state changes must pass through `Environment::act`;
- all accepted discharge must come from independently measured objective progress;
- live chat wiring must remain absent;
- automatic ontology promotion must remain absent;
- self-edit authority must remain absent.

The executable prints `FOUNDATION_PASS` only when the exact solve and authority
gates pass. The label deliberately does not mean that learned routing, transfer,
open-ended planning, ontology induction, or AGI has been demonstrated.

## Required controls for the preregistered A1 follow-up

This first PR establishes contracts and a deterministic positive path. A later,
separately frozen A1 capability experiment must add matched-budget controls:

- random action selection;
- deterministic blind exploration;
- fixed operator schedule;
- no-CHARGE;
- scrambled pressure identity;
- no executable commitment state;
- no representation repair;
- oracle upper bound.

No cross-domain claim is permitted until the same frozen kernel is evaluated in
at least one structurally different environment without adding task-specific
executive handlers.

## Commands

```text
cargo check -p star --all-targets --locked
cargo test -p star autonomy:: --locked -- --test-threads=1
cargo test -p star environment::hidden_rule:: --locked -- --test-threads=1
cargo run -p star --example a1_bounded_autonomous_kernel --locked
```

## Claim boundary

A pass supports only this statement:

> Starfire contains a bounded shadow executive that can preserve unresolved
> goal pressure, select typed operator proposals, authorize environment actions,
> and accept discharge only from external objective evidence under exact resource
> limits in a deterministic toy environment.

It does not establish autonomous general intelligence, learned planning,
cross-domain transfer, unrestricted continual learning, consciousness, or safe
production self-improvement.
