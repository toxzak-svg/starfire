# S5-B — Independently Witnessed Policy Outcomes

## Purpose

S5-B resolves the unresolved S5-A interaction-policy predictions using delayed evidence supplied by a user observation, task environment, or external evaluator.

This slice collects evidence only. It does not compare policies, promote a policy, alter generated text, or wire companion state into `Runtime::chat()`.

## Frozen split assignment

Every enrollment is assigned before any outcome is visible:

1. contexts issued at or after a preregistered timestamp enter the temporal holdout;
2. earlier contexts whose opaque subject digest matches a preregistered modulus/remainder enter the subject holdout;
3. all remaining contexts enter development.

The caller cannot relabel a completed outcome into a preferred split.

## Complete-arm collection

Every non-abstaining S5-A arm must have exactly one observation. A batch is rejected if it contains:

- a missing arm;
- a duplicate arm;
- an arm outside the enrollment;
- a mismatched policy variant;
- reused evidence across arms;
- a witness outside the prediction window;
- a response-generator witness;
- a witness identifier equal to the prediction producer;
- correction, clarification, completion, or compute metrics outside configured bounds.

Candidate abstentions remain explicit S4 abstentions and require no fabricated outcome.

## Atomicity and replay

Collection runs against a cloned prediction ledger. The caller's ledger is replaced only after every observation validates and resolves successfully.

Each accepted outcome preserves:

- prediction ID and policy variant;
- deterministic evaluation split;
- witness source, identifier, time, label, and evidence digest;
- exact S4 Brier score;
- correction count;
- clarification burden;
- completion rate;
- compute cost;
- subject scope, producer ID, and context digest.

The resulting issue and resolution events must replay to the same ledger state.

## Frozen probe

The executable probe requires:

- six resolved development arms;
- six resolved subject-holdout arms;
- six resolved temporal-holdout arms;
- deterministic pre-outcome split assignment;
- complete variant coverage;
- unique arm-specific evidence;
- delayed independent witnesses only;
- rejection of response-generator self-grading;
- rejection of incomplete batches;
- unchanged ledger state after rejection;
- exact event replay;
- preserved S5-C metric inputs;
- zero runtime, response, routing, belief, or action authority.

## Claim boundary

Passing S5-B proves only that Starfire can collect bounded, independently witnessed policy outcomes without outcome leakage or partial ledger mutation.

S5-C must perform the preregistered comparative analysis across development, subject-holdout, and temporal-holdout splits. No policy may influence live response planning until held-out improvement, calibration, abstention quality, and overhead gates pass and the change remains reversible.
