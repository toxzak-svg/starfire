# ΩV1-F2 Shadow Implementation Status

**Status:** Implementation candidate, external build pending  
**Date:** July 20, 2026 (America/Detroit)  
**Parent preregistration:** `208deae52d5830168376bf3ae67e79cd569a6f01`  
**Live learned-text authority:** None  
**Default runtime switch:** Disabled

## Implemented boundary

The F2 implementation compiles the externally passed F1R1 selector and nested verifier into the production binary behind:

```text
STARFIRE_OMEGA_V1F2_SHADOW=1
```

The entrypoint defaults that switch to `0`. With the switch disabled, `/chat` follows the existing D1 path and performs no F2 bundle construction, model loading, worker dispatch, or metadata recording.

When enabled for a successful `POST /chat` request:

1. the existing runtime produces its response;
2. D1 finalizes the response;
3. the final JSON body is serialized and fingerprinted;
4. an optional typed shadow event is dispatched to a detached worker;
5. the worker constructs, scores, and independently verifies a bounded candidate;
6. candidate text is discarded;
7. only bounded metadata is appended to the shadow ledger.

The worker never receives the raw prompt, returned response text, conversation history, memory text, identity text, user identifier, cookies, authentication headers, IP address, companion state, or mutable VoiceState access.

## Narrow initial eligibility

Initial eligibility uses Starfire's existing typed `ResponseIntent` classifier. The API passes only that enum value into the F2 module. The F2 module maps supported intents to committed semantic seeds, then validates a complete `SemanticResponseProgram`, lexical table, and sealed read-only default VoiceState projection.

Supported initial intent classes are:

- self-check;
- reflection;
- research status;
- curiosity;
- emotional acknowledgment;
- identity;
- capability;
- story interaction;
- consciousness uncertainty;
- recall;
- teaching;
- aspiration.

`Statement` and `Unknown` are shadow-ineligible. Failed typed-program construction is also ineligible. No missing semantic field is reconstructed from returned prose.

This narrow bridge tests containment and operational execution. It does not establish that the shadow semantic seed is a complete semantic explanation of the live response, and it does not establish human preference.

## Model identity

A dedicated exporter retrains the deterministic F1R1 model from the frozen 74-example training split using the same manifest, projections, preferences, epoch count, and learning rate as the passed evaluator.

The Docker builder:

- reruns the complete F1R1 gate;
- exports the model artifact;
- verifies exact artifact replay;
- compares the exported model digest with the model digest in the F1R1 report;
- packages the artifact read-only in the runtime image;
- seeds it into the persistent data directory without replacing an existing nonempty artifact.

## Response isolation

The shadow worker receives a `ResponseFingerprint`, not response bytes. The fingerprint contains only before/after digests and byte lengths. Both pairs are fixed before dispatch.

Model-load failure, selector failure, verifier rejection, worker creation failure, timeout, panic, ledger failure, or a disabled switch cannot return candidate text or alter the frozen response.

## Metadata ledger

Default path:

```text
/data/logs/omega_v1f2_shadow.jsonl
```

Allowed fields include:

- opaque event identifier;
- coarse UTC day and hour buckets;
- typed eligibility code;
- intent and sensitivity labels;
- program, lexical, projection, model, lattice, selection, verifier, implementation, and authority digests;
- selected family and stable variant identifiers;
- selection disposition and bounded reason;
- candidate count;
- verifier acceptance;
- response fingerprints and lengths;
- bounded timing and timeout/panic flags.

There is no field for prompt text, response text, candidate text, lexical clause text, memory text, or a user identifier.

## Builder gate

Before the production binary is built, Render must prove:

- frozen F1R1 regression PASS;
- exact deterministic model export and digest agreement;
- typed bundle validation;
- learned-candidate independent verification;
- deterministic replay;
- byte-preservation fingerprint invariant;
- stale projection fail-closed behavior;
- missing, corrupt, and oversized model rejection;
- timeout, panic, and unavailable-ledger isolation;
- a closed F2 authority matrix.

The production binary is built with `omega-v1-f2-shadow`, but the runtime switch remains disabled unless explicitly enabled in the deployment environment.

## Collection and verdict tools

The controlled client harness is:

```text
python scripts/omega_v1f2_daily_traffic.py
```

Its default batch sends 30 typed-eligible and 8 expected-ineligible requests. It prevents an accidental second batch on the same UTC day unless `--force` is supplied. It prints only status, response-byte length, and a response digest prefix.

The ledger evaluator is:

```text
OMEGA_V1F2_LEDGER_PATH=/data/logs/omega_v1f2_shadow.jsonl \
  cargo run -p star --example omega_v1f2_ledger_report \
  --features omega-v1-f2-shadow --locked
```

It reports one of:

- `COLLECTING`: all observed hard gates pass, but the frozen sample is incomplete;
- `FAIL`: an isolation, verifier, identity, timing, uniqueness, schema, or bound gate failed;
- `PASS`: every hard gate passes with at least 200 eligible completed events, 50 ineligible events, and seven UTC days.

## Scientific status

This implementation is not an F2 result. F2 remains unclassified until:

1. the merged source passes the external Render builder and deployment gate;
2. the switch is explicitly enabled;
3. the frozen sample is collected;
4. the committed ledger evaluator returns `PASS`;
5. required forced-failure controls are recorded against the deployed source.

An F2 PASS could authorize only a separately preregistered F3 verified canary. It would not itself authorize learned text return, state mutation, persistence changes, routing, tools, CHARGE, belief or ontology promotion, companion access, or autonomous action.
