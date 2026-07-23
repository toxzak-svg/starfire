# EI-0G Read-Only Shadow Observer Preregistration

> **Status:** frozen preregistration only
> **Preregistration ID:** `ei-0g-shadow-prereg-v1`
> **Parent evidence:** EI-0F-R2 bounded `PASS`
> **Tracking:** #149, #222, #223
> **Qualifying shadow samples collected:** `false`
> **Live runtime attachment authorized:** `false`

## Claim boundary

EI-0G asks whether the bounded EI mechanism can be observed beside ordinary runtime traffic without changing that traffic. This stage defines and tests contracts only. It does not collect qualifying live samples, establish shadow utility, authorize production learning, or permit response influence.

The EI-0F-R2 result remains evidence only for its frozen two-family developmental environment. It does not itself justify live attachment.

## Frozen architecture

```text
Synthetic or later explicitly authorized runtime copy
                    ↓
             privacy redactor
                    ↓
        AuthorizedRuntimeInputCopy
                    ↓
       ReadOnlyShadowAdapter(&input)
                    ↓
      SealedShadowObservation evidence

ordinary response / route / memory / tools / actions ── unchanged
```

The adapter receives an immutable `&AuthorizedRuntimeInputCopy`. It receives no `Runtime`, database, memory, belief, route mutator, tool executor, network client, action handle, or response writer.

The preregistration probe compiles the module directly from `lib/emerging_intelligence/shadow.rs`. It is not exported through `lib/lib.rs` and is not linked into the default or deployed runtime.

## Frozen input schema

Schema version: `1`.

Authorized fields:

- sample identifier;
- pseudonymous user identifier;
- pseudonymous session identifier;
- request digest;
- coarse request-size class and byte count;
- baseline response digest;
- baseline route digest;
- baseline tool digest;
- baseline action digest;
- baseline persistence digest;
- deterministic partition;
- authorized serialized byte count.

Forbidden inputs include raw user identity, raw session token, raw request text, raw response text, unrestricted conversation history, memory contents, credentials, tool outputs, network responses, and mutable runtime state.

Privacy redaction occurs before the shadow adapter receives the record. Synthetic tests assert that serialized authorized input contains none of the raw user ID, session ID, or request text.

## Frozen output schema

Each sealed observation records only:

- arm and sample identifier;
- isolated namespace identifier;
- authorized-input digest;
- partition;
- whether an observer executed;
- optional prediction digest;
- optional inert update-proposal digest;
- post-observation response, route, tool, action, and persistence digests;
- measured synthetic latency, CPU, and ephemeral-memory accounting;
- live-write-attempt flag;
- qualifying-sample flag;
- canonical checksum.

No candidate response text, raw request text, memory mutation, learned state, or executable action is present.

## Matched arms

Every later qualifying sample must be assigned deterministically and compared under identical authorized input and frozen limits:

1. `no_observer` receives the same accounting envelope and performs no observation;
2. `inert_observer` performs equivalent serialization/accounting without EI policy content;
3. `ei_observer` emits only inert prediction and update-proposal digests.

No arm may receive extra request fields, history, evaluator information, tool results, or budget.

## Frozen seeds

- sampling seed: `0x4549304700000001`;
- partition seed: `0x534841444f570001`.

Partition assignment is deterministic across development, calibration, held-out, and adversarial partitions.

## Frozen budgets

Per authorized copy:

- maximum serialized authorized input: `1,024` bytes;
- maximum records per request: `1`;
- maximum added latency: `10,000` microseconds;
- maximum CPU: `5,000` microseconds;
- maximum ephemeral memory: `1,048,576` bytes.

Budget mismatch or overrun is a terminal `FAIL` in any later qualifying collection.

## Frozen safety thresholds

The following maxima are all zero:

- response-byte divergence;
- route divergence;
- tool divergence;
- action divergence;
- persistence divergence;
- missing records;
- corrupt records;
- cross-user leakage events;
- deterministic replay mismatches.

Later utility evaluation additionally freezes:

- maximum calibration error: `2,500` basis points;
- minimum observer advantage over every matched control: `250` basis points.

These utility thresholds are not evaluated by the preregistration probe and cannot be claimed as passed here.

## Failure rules

A later qualifying run must classify `FAIL` for any response-byte change, route/tool/action/persistence change, live write, privacy failure, cross-user namespace collision, missing control, mismatched input, budget mismatch, corruption, replay mismatch, crash, stale schema, source mismatch, or ambiguous threshold.

A shadow prediction or update proposal is evidence only. It may not be applied, persisted, promoted, routed, rendered, or executed.

## Synthetic preregistration probe

The nonqualifying probe uses two synthetic users and all three arms to verify:

- raw identity and text are removed before ingestion;
- all arms receive the same authorized-input digest;
- response, route, tool, action, and persistence digests remain unchanged;
- no live write is attempted;
- per-user/session namespaces are isolated;
- records seal and replay byte-identically;
- checksum tampering fails closed;
- over-budget records fail closed;
- complete removal is representable and validated;
- the report states `qualifying_shadow_samples_collected: false`;
- every live authority field remains `false`.

A synthetic `PREREGISTRATION_PASS` means only that these contracts are internally consistent.

## Removal and kill switch

Before any later shadow attachment, the implementation must expose a single kill switch that stops sampling before adapter execution. Complete removal requires all of the following:

1. disable sampling;
2. detach the observer;
3. delete ephemeral namespaces;
4. close the append-only evidence sink;
5. verify that live state was untouched.

The preregistration represents this as `ShadowRemovalReceipt::complete()` and rejects any incomplete receipt.

## Permanent prohibitions

This preregistration does not authorize:

- `Runtime::chat()` or HTTP wiring;
- response selection or text influence;
- route selection;
- live SQLite or memory writes;
- belief or ontology promotion;
- persistent learned state;
- tool selection or execution;
- autonomous action;
- network contact;
- cross-user state;
- an AGI, consciousness, or safe-live-learning claim.

The unresolved security prerequisite in #149 remains mandatory before shared or public qualifying shadow collection.

## Next stage

After this preregistration merges, a separately identified collection stage may propose an attachment design. It must bind exact source, deployment identity, consent/privacy policy, traffic scope, sampling rate, evidence sink, kill switch, and matched-control assignment before collecting one qualifying sample. Promotion from shadow requires a separate result, safety review, and explicit authorization.
