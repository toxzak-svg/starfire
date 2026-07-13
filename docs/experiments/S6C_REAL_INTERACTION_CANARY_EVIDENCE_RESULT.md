# S6-C Real-Interaction Canary Evidence Result

Status: **PENDING — no verdict-producing committed-source run yet**

The frozen preregistration is in
`docs/experiments/S6C_REAL_INTERACTION_CANARY_EVIDENCE.md` and was committed before
implementation at:

```text
469654d1ce2008888e6804d407d17d37379a88f1
```

A mechanism PASS may be recorded only after the committed implementation compiles,
passes scoped lint and deterministic tests, executes the frozen S6-C probe, preserves
S5-B/S4 atomicity, and keeps synthetic evidence promotion-ineligible.

Real-interaction promotion eligibility requires a later canary dataset containing only
consented `RealInteraction` attestations and a passing frozen S5-C held-out evaluation.