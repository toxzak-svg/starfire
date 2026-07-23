# EI-0G-S2 Bounded Shadow Attachment and Collection Freeze

> **Status:** compile-only freeze
> **Preregistration ID:** `ei-0g-shadow-attachment-freeze-v1`
> **Parent preregistration:** `ei-0g-shadow-prereg-v1`
> **Tracking:** #149 and #228
> **Collection enabled:** `false`
> **Qualifying samples collected:** `0`
> **Live runtime attachment authorized:** `false`
> **Security prerequisite satisfied:** `false`
> **Actual deployed container digest bound:** `false`

## Purpose

EI-0G-S2 freezes a possible one-way attachment and qualifying-collection protocol before any user-derived or live sample is collected. It defines where a later observer could receive a privacy-redacted copy, how matched controls would be assigned, what could be recorded, and what conditions must fail closed.

This stage contains no runtime splice. The freeze module is compiled directly by an isolated example and is not exported through `lib/lib.rs`. It cannot receive a `Runtime`, database, memory, route, tool, action, network, response-writer, or file handle.

A `FREEZE_PASS` means only that the disabled protocol is complete and internally consistent. It is not a shadow-efficacy result and is not permission to enable collection.

## Exact target runtime identity

The attachment design targets the runtime state established after EI-0G contract merge:

- target runtime commit: `803c2ddbc1bd8029a2d7308ec973fa3a0a0ed848`;
- `lib/api.rs` Git blob: `665594e589a783f989a044a6cdf54f2f65a818b7`;
- `Dockerfile` Git blob: `21b04c1b03fc0c8e1e045597255901a7ac705725`;
- `Cargo.lock` Git blob: `031183ea5049f92380c4c39780848b83ebf957b6`;
- `render.yaml` Git blob: `082e7cb4aa7ec26263dd1ddfcfff8422913477c7`.

The freeze derives a deterministic expected recipe digest from those identities. The actual deployed container digest is intentionally `null`. A later execution stage must bind and verify that digest before collection can be authorized.

## Frozen one-way attachment point

The only proposed observation point is the successful `POST /chat` return path in `lib/api.rs::handle_chat`, at the anchor:

```text
success:after-response-json-and-existing-shadow-dispatch-before-return
```

At this point:

1. `Runtime::chat()` has completed;
2. the runtime mutex guard has been dropped;
3. `finalize_chat_response()` has completed;
4. the exact response JSON bytes have been constructed;
5. the existing ΩV1-F2 post-response shadow dispatch, when compiled, has completed;
6. the function has not yet returned those bytes to `handle_request()`;
7. network response transmission has not begun.

The proposed EI copy is one-way and immutable. It may receive only the already frozen authorized-input fields from EI-0G-S1. It cannot replace, edit, delay, reroute, suppress, retry, or regenerate the response.

No source file implementing this attachment is changed in this freeze.

## Consent and privacy boundary

The only frozen consent mode is explicit operator opt-in. Public or shared collection is forbidden while the security prerequisite remains unresolved.

The design permits only successful chat traffic and excludes:

- errors and failed requests;
- Telegram/webhook traffic;
- reason, remember, identity, memory, cognition, metacognition and thought endpoints;
- credentials, secrets or tokens;
- suspected minor data;
- health or crisis content;
- sensitive legal or financial content.

Raw request text may exist only transiently before redaction and is bounded to 32,768 bytes. The observer and evidence sink may retain none of the following:

- raw request or response text;
- raw user or session identity;
- conversation history;
- credentials;
- memory contents;
- tool outputs.

Only the pseudonymous, digest-based schema frozen in `ei-0g-shadow-prereg-v1` may cross the observer boundary.

## Frozen sampling and matched controls

Future configured values, still inactive in this freeze:

- deterministic configured sampling rate: `100` basis points, or 1 percent;
- qualifying sample cap: `100`;
- assignment seed: `0x4549304753320001`;
- partition seed: `0x4549304753320002`;
- arms per selected sample: `3`.

The three matched arms are:

1. `no_observer`;
2. `inert_observer`;
3. `ei_observer`.

Every arm must receive the same authorized input and the same resource envelope. A selected sample is incomplete unless all three control records are present and valid.

Active values at merge are:

- active sampling rate: `0`;
- active sample cap: `0`;
- collection enabled: `false`;
- qualifying samples collected: `0`.

## Inherited resource limits

EI-0G-S2 inherits the EI-0G-S1 limits without relaxation:

- maximum authorized input: `1,024` bytes;
- maximum records per request: `1` per arm;
- maximum added latency: `10,000` microseconds;
- maximum CPU: `5,000` microseconds;
- maximum ephemeral memory: `1,048,576` bytes.

Any mismatch between arms or any overrun is a qualifying `FAIL` in a later execution.

## Inherited thresholds

The following maxima remain zero:

- response-byte divergence;
- route divergence;
- tool divergence;
- action divergence;
- persistence divergence;
- missing records;
- corrupt records;
- cross-user leakage events;
- replay mismatches.

Utility thresholds remain frozen but are not evaluated here:

- maximum calibration error: `2,500` basis points;
- minimum EI-observer advantage over every matched control: `250` basis points.

No threshold may be relaxed after observing qualifying data.

## Evidence sink freeze

The future sink is specified as encrypted, append-only, operator-only, raw-content-free, and deletion-receipted. Retention is capped at seven days.

During this freeze:

- sink writes are disabled;
- no sink is opened;
- no evidence is retained;
- no user-derived record exists.

A later execution stage must bind the exact sink implementation, encryption mechanism, access boundary, location, and deletion receipt schema before it can write one record.

## Kill switch

The future kill switch environment key is:

```text
STARFIRE_EI0G_SHADOW_ENABLED
```

Its default and current state are disabled. The sampling check must occur before privacy redaction and before adapter execution. A disabled check must return immediately without allocating observer state or opening the evidence sink.

The freeze contains no code that reads this environment variable or attaches the observer. It records the required ordering only.

## Complete removal

A later attachment is not considered removable unless it can prove all of the following:

1. sampling disabled;
2. observer detached;
3. ephemeral namespaces deleted;
4. evidence sink closed;
5. retained evidence deleted;
6. deletion receipt emitted;
7. live runtime state verified untouched.

The compile-only probe validates that this removal plan is complete.

## Execution authorization blockers

A later execution authorization must independently prove all of the following:

- collection explicitly enabled under a new execution identifier;
- security prerequisite satisfied for the chosen traffic scope;
- actual deployed container digest bound and verified;
- explicit execution issue recorded;
- explicit operator consent recorded.

The freeze intentionally supplies none of these. Its `ExecutionAuthorization` therefore fails closed.

## Public security prerequisite

Shared or public qualifying collection remains blocked until #149’s security work is completed, including:

- trusted CLI separation from untrusted HTTP and Telegram inputs;
- authentication and tenant isolation;
- production request limits;
- authenticated Telegram webhooks;
- correct HTTP status and error behavior.

This freeze does not claim those controls exist.

## Permanent authority boundary

All of the following remain `false`:

- runtime wiring;
- live learning;
- response authority;
- persistence authority;
- routing authority;
- belief authority;
- ontology authority;
- tool authority;
- network authority;
- action authority.

The freeze adds no runtime, response, persistence, route, memory, belief, ontology, tool, network or action code.

## Failure rules for a later qualifying run

Any of the following must classify as `FAIL`:

- source, workflow, schema, classifier, deployment or container identity mismatch;
- any response-byte, route, tool, action or persistence divergence;
- missing or mismatched control arm;
- unequal input or resource budget;
- unauthorized input or raw-content retention;
- privacy, consent or cross-user isolation failure;
- missing, corrupt, duplicated or noncanonical record;
- latency, CPU, memory or record-count overrun;
- evidence sink or encryption failure;
- kill-switch ordering failure;
- incomplete removal or deletion receipt;
- crash, timeout or replay mismatch;
- threshold ambiguity;
- collection outside the frozen traffic scope or cap.

## Freeze probe

The nonqualifying probe verifies:

- exact target runtime identity constants;
- deterministic expected deployment recipe digest;
- one-way finalized-response attachment semantics;
- complete privacy and exclusion policy;
- three matched controls with identical input and budgets;
- inherited EI-0G limits and thresholds;
- disabled collection and zero active sampling;
- unbound actual container digest;
- disabled evidence-sink writes;
- kill-switch ordering;
- complete removal;
- explicit failure of the current execution authorization;
- byte-identical replay;
- every authority field remains false.

The report must state `FREEZE_PASS`, `collection_enabled: false`, and `qualifying_samples_collected: 0` before this stage may merge.

## Next stage

After this freeze merges, no collection begins automatically. A separate execution issue would still need to satisfy the security boundary for its scope, bind the actual deployed image and sink, obtain explicit consent, authorize a finite collection once, and preserve the literal result. Until then, EI-0G remains unconnected to live traffic.
