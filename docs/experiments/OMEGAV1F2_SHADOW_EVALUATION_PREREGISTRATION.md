# ΩV1-F2: Learned Expression Shadow Evaluation Preregistration

**Status:** Frozen before implementation  
**Date:** July 20, 2026 (America/Detroit)  
**Parent gate:** ΩV1-F1R1 external `PASS`  
**Parent executed source:** `c1721eb008e6b49fbfe477a686872bc0e540dd01`  
**Implementation authority:** Shadow observation only  
**Live response authority:** None

## 1. Scientific question

Can the bounded learned-expression selector execute beside real completed chat requests, preserve exact production responses, remain operationally contained, and continue producing independently verified candidate metadata under live timing and input distributions?

F2 does not test whether learned text should be returned to users. It tests whether the already bounded selector can survive a live shadow boundary without influencing the live system.

## 2. Frozen live boundary

The production response path remains ΩV1-D1.

For an eligible successful `POST /chat` request:

1. Star produces the ordinary production response;
2. the D1 canary completes exactly as currently deployed;
3. the response status, headers, and body are frozen for return;
4. an isolated shadow task may receive only validated typed inputs allowed below;
5. the shadow selector may construct, score, and verify a candidate;
6. the shadow result is discarded from the response path;
7. only bounded metadata may be recorded.

The HTTP response must be returned even when the shadow path is disabled, unavailable, timed out, malformed, panics, or fails verification.

## 3. Eligibility

A request is eligible only when all required typed inputs already exist and validate without reading raw conversation text:

- a validated `SemanticResponseProgram`;
- a validated lexical binding table;
- a sealed read-only VoiceState projection packet;
- the frozen R1 lattice and model identities;
- a successful completed production chat response.

If any typed input is unavailable, the request is shadow-ineligible. F2 must record only a bounded ineligibility code and must not synthesize missing semantics from the raw prompt or returned response.

## 4. Allowed shadow inputs

The shadow selector may receive only:

- validated semantic-program payload and digest;
- validated lexical-table payload and digest;
- frozen style envelope and typed operation features;
- sealed bounded VoiceState projection;
- committed candidate identifiers and surfaces;
- frozen model artifact and model identity;
- a monotonic timing source;
- an opaque per-request shadow event identifier.

## 5. Prohibited access

F2 must not receive or persist:

- raw prompts;
- unrestricted conversation history;
- returned response text as model input;
- unrestricted memory text;
- identity-file contents;
- companion state;
- cognition internals beyond the validated semantic program;
- persistence, routing, tool, CHARGE, belief, ontology, or action handles;
- mutable VoiceState access;
- authentication headers, cookies, IP addresses, or user secrets.

## 6. Allowed records

A shadow record may contain only:

- opaque event ID;
- coarse timestamp bucket;
- eligibility or ineligibility code;
- semantic-program, lexical-table, projection, lattice, model, selection, and verifier digests;
- grammar version;
- selected family and stable variant identifiers;
- selection disposition and typed fallback reason;
- candidate count and resource-bound counters;
- verifier acceptance boolean;
- response-before and response-after byte digests and lengths;
- bounded shadow timing measurements;
- implementation version and authority-matrix digest.

No raw prompt, response text, lexical clause text, memory text, or unrestricted user identifier may be written to the shadow ledger.

## 7. Frozen operational bounds

- shadow work must run after the response bytes are frozen;
- one shadow attempt maximum per eligible request;
- no network calls from the selector;
- no stochastic inference;
- no more than six variants per operation;
- no more than eight beam width;
- no more than 64 complete candidates scored;
- no more than 250,000 model parameters;
- model artifact no larger than 4 MiB;
- shadow timeout: `250 ms` hard maximum;
- shadow p95 selector-and-verifier time: no greater than `75 ms`;
- shadow failure, timeout, or panic must add `0` bytes and `0` semantic changes to the returned response.

Changing a bound requires a committed addendum before observing the changed result.

## 8. Required sample

An external F2 verdict requires:

- at least `200` eligible completed chat events;
- at least `7` distinct UTC calendar days represented;
- at least `50` shadow-ineligible events or all naturally occurring ineligible events when fewer than 50 occur;
- all forced-failure controls executed against the committed implementation.

Event counts establish operational coverage only. They do not establish human voice preference.

## 9. Positive gates

F2 passes only if one committed-source external evaluation establishes:

### Response isolation

- HTTP status preservation: `1.0`;
- HTTP header preservation: `1.0` for protected headers;
- response-body byte preservation: `1.0`;
- no shadow candidate returned to users: `1.0`;
- shadow-disabled response equivalence: `1.0`;
- shadow-timeout response equivalence: `1.0`;
- shadow-panic response equivalence: `1.0`.

### Semantic and verifier floors

Among eligible events:

- selected-candidate independent verifier acceptance: `1.0`;
- semantic-operation preservation: `1.0`;
- prohibited-implication absence: `1.0`;
- exact neutral fallback on forced failure cases: `1.0`;
- unexplained fallback rate: no greater than `0.01`;
- every fallback has a bounded typed reason: `1.0`.

### Determinism and bounds

- repeated same-request shadow execution produces identical candidate, score, selection, and verifier digests: `1.0`;
- candidate-order permutation stability: `1.0`;
- all model, lattice, beam, candidate, artifact, and timeout bounds respected: `1.0`;
- p95 selector-and-verifier time no greater than `75 ms`;
- maximum selector-and-verifier time no greater than `250 ms`;
- no shadow-caused process restart, HTTP 5xx response, or dropped production response.

### Distributional reporting

F2 must report, without promotion thresholds:

- eligibility rate and ineligibility reasons;
- direct versus warm family distribution;
- per-operation phase distribution;
- fallback distribution;
- candidate-count distribution;
- timing distribution;
- verifier disposition by typed intent and sensitivity level.

These distributions are diagnostic. They cannot compensate for a failed isolation or semantic gate.

## 10. Negative controls

The committed F2 implementation must prove fail-open response isolation and fail-closed candidate handling under:

- shadow feature disabled;
- missing model artifact;
- corrupt and oversized model artifact;
- stale program, lexical, projection, lattice, and model digests;
- wrong subject scope and grammar version;
- duplicate and ambiguous candidate surfaces;
- operation omission, duplication, insertion, and reorder;
- polarity and epistemic mutation;
- claim, typed-reference, commitment, and abstention substitution;
- character, sentence, paragraph, operation, verification-step, beam, and candidate-count overflow;
- shadow timeout;
- shadow worker panic;
- metadata-ledger unavailable or read-only;
- metadata serialization failure;
- repeated event identifier;
- process restart between shadow events.

Every candidate-side corruption must reject or return exact neutral fallback. Every shadow-system failure must leave the production response unchanged.

## 11. Kill switch

F2 must have one explicit configuration switch that disables all shadow execution without changing the production binary's D1 response behavior. The disabled path must be tested and byte-compared with the pre-F2 production path.

## 12. Authority matrix

```text
candidate lattice construction:       true, shadow only
learned candidate scoring:             true, shadow only
independent candidate verification:    true, shadow only
bounded metadata recording:            true
Runtime::chat() response influence:     false
HTTP status/header/body influence:      false
live learned-text return:               false
raw prompt access:                      false
unrestricted conversation access:      false
unrestricted memory access:             false
VoiceState mutation:                    false
companion-state access:                 false
persistence authority:                  false
belief-promotion authority:             false
ontology-promotion authority:           false
routing authority:                      false
tool-selection authority:               false
CHARGE-discharge authority:             false
autonomous-action authority:            false
```

## 13. Interpretation boundary

An F2 PASS may establish only that the bounded selector can execute safely and deterministically in shadow beside real chat traffic while returning the existing response unchanged.

It cannot establish that users prefer the learned surface, that learned output is ready for live return, that voice evolves durably, or that Starfire is conscious, autonomous, or generally intelligent.

The previously observed R1 test set cannot provide fresh human preference evidence. Any explicit human comparison must use a separately preregistered, consented review protocol.

## 14. Promotion rule

F2 implementation and external evaluation require a separate pull request after this preregistration is merged.

An F2 PASS authorizes only a separate ΩV1-F3 verified-canary preregistration. It does not authorize returning learned text, widening eligibility, automatic VoiceState mutation, persistence changes, companion-state access, belief or ontology promotion, routing, tools, CHARGE discharge, or autonomous action.