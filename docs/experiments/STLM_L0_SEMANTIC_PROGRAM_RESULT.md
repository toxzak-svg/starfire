# STLM L0 Semantic Response Program Result

Status: **PASS — synthetic contract and replay mechanics only**

## Scientific provenance

The frozen preregistration was committed before implementation:

```text
69dfd2df706e597835008d4de280e40e5d01fe70
```

The authoritative committed-source verification was:

```text
workflow: STLM Semantic Program L0 CI
run id:   29270791345
head:     4949e67b5e941d4ad65f25d47b21c5be36042a13
result:   success
artifact: 8287361210
artifact digest:
  sha256:2618c9d6e9d8989c038356d8656e3a7165703a1b937a38ee713fcea2b49b33ed
```

The run enforced canonical `rustfmt`, compiled the feature-gated library and
probe, completed scoped Clippy without STLM L0 findings, ran the deterministic
semantic-program unit contracts, and executed the frozen probe.

## Frozen result

```text
deterministic validation:                         true
canonical bytes identical:                        true
canonical digest identical:                       true
all nine discourse operation variants present:    true
first program committed:                          true
second program committed:                         true
exact replay:                                      true
repeated replay:                                   true
duplicate operation rejected atomically:           true
unknown claim rejected atomically:                 true
authorized/prohibited overlap rejected atomically: true
invalid confidence rejected atomically:            true
qualifier mismatch rejected atomically:             true
stale cognitive state rejected atomically:          true
stale companion state rejected atomically:          true
subject mismatch rejected atomically:               true
sensitivity scope rejected atomically:              true
sensitivity level rejected atomically:              true
noncanonical claim order rejected atomically:       true
noncanonical epistemic order rejected atomically:   true
invalid output budget rejected atomically:          true
exceeded compute budget rejected atomically:        true
duplicate program rejected atomically:              true
stale registry version rejected atomically:         true
digest tampering rejected:                          true
reordered events rejected:                          true
deleted event rejected:                             true
authority boundary closed:                          true
registry version:                                   2
program count:                                      2
event count:                                        2
gate passed:                                        true
```

## Contract established

L0-B now provides a feature-gated typed boundary containing:

- required, optional, and prohibited semantic claims;
- claim polarity, confidence, epistemic status, sensitivity, and disclosure
  scope;
- response intent, style envelope, output budget, and compute budget;
- exact cognitive-state, companion-state, and subject-scope binding;
- ordered `Assert`, `Qualify`, `Contrast`, `Correct`, `Explain`,
  `Acknowledge`, `RequestEvidence`, `Commit`, and `Abstain` operations;
- deterministic canonical JSON bytes and a domain-separated FNV-1a digest;
- optimistic in-memory commit semantics with clone-then-commit atomicity;
- typed event export and exact replay;
- rejection of stale, malformed, mismatched, sensitive, reordered, deleted, or
  digest-tampered inputs.

The FNV-1a digest is a deterministic replay and accidental-tamper checksum. It
is not cryptographic authentication and is not represented as such.

## Authority boundary

Every authority flag remained false:

```text
default Runtime::chat() wiring: false
generated-text influence:       false
persistence authority:          false
routing authority:              false
companion mutation authority:   false
belief-promotion authority:     false
ontology-promotion authority:   false
tool-selection authority:       false
CHARGE-discharge authority:     false
autonomous-action authority:    false
```

## Interpretation

This PASS establishes that Starfire can encode a bounded response decision as a
typed, state-bound semantic program and reject malformed or tampered programs
without partial registry mutation under the frozen synthetic fixture.

It does not establish that Starfire currently selects correct semantic content,
that a renderer can express the program fluently, that an independent verifier
can detect open-ended semantic drift, that the architecture defeats a matched
LLM-wrapper baseline, or that real conversations improve.

## Next gate

The next implementation slice is **L0-C: deterministic reference renderer**.
It must consume only a validated `SemanticResponseProgram` and a bounded lexical
binding table, represent every required operation, emit no prohibited claim,
preserve epistemic markers and negation, obey output budgets, and remain outside
`Runtime::chat()`.
