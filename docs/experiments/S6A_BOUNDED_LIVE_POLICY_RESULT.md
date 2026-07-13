# S6-A Bounded Live Policy Result

Status: **PENDING FIRST VERDICT-PRODUCING CI RUN**

The implementation and frozen probe are committed on `experiment/s6a-bounded-live-policy`.

No S6-A conformance claim is asserted until a committed-source workflow run successfully compiles the feature, executes the deterministic unit contracts, and runs `s6a_bounded_live_policy_probe`.

## Frozen gate

The probe must demonstrate:

- validated S5-C promotion evidence shape;
- default rejection of simulated evidence;
- explicit opt-in for the frozen simulation;
- body-preserving response planning;
- bounded metadata influence reaching the reranker;
- exact neutral fallback for sensitive and disallowed contexts;
- applied-turn budget enforcement;
- immediate revocation;
- exact replay;
- all routing, persistence, belief-promotion, and action authority flags false.

A synthetic `PASS` establishes controller and pipeline conformance only. It does not establish that companion-derived policy improves real conversations and does not authorize default `Runtime::chat()` wiring.
