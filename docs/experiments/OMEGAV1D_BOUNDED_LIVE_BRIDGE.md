# ΩV1-D: Bounded Deterministic Live Bridge

**Status:** Preregistered implementation gate  
**Parent gate:** ΩV1-C PASS on externally executed Render build, July 20, 2026  
**Feature flag:** `omega-v1-live-bridge`

## Scientific question

Can Starfire make one detectable, deterministic expression change in the live response path while preserving the cognition-produced response body exactly and retaining an unconditional neutral fallback?

ΩV1-D is deliberately smaller than a general renderer. It is the first live canary for the cognitive-to-voice bridge, not permission for free-form rewriting.

## Frozen canary transformation

The neutral input is the completed post-reranker, post-`VoiceEngine` response.

A response is eligible only when it begins with the exact byte sequence:

```text
Here for it. 
```

The remainder of the response is the protected body. The bridge may replace only that opener with one member of this closed table:

```text
Got it. 
I'm following. 
I'm with you. 
```

Selection is deterministic: FNV-1a over `prompt + NUL + protected_body`, modulo the table length.

No other substring, punctuation mark, claim, hedge, question, reference, or body byte may change.

## Mandatory neutral fallback

The original response must be returned byte-for-byte when any of these conditions holds:

- the exact eligible opener is absent;
- the protected body is empty or whitespace-only;
- the protected body exceeds 4,096 bytes;
- the selected opener is not in the frozen table;
- the rendered text does not end with the exact protected body;
- any internal invariant fails.

Fallback is a successful bounded outcome, not an error.

## Frozen gates

The implementation passes only if all of the following hold:

1. Same prompt and neutral response produce exactly the same output and decision metadata.
2. Every applied transformation preserves the protected body byte-for-byte.
3. Ineligible responses pass through byte-for-byte.
4. Empty and oversized bodies pass through byte-for-byte.
5. Applied output uses only the frozen replacement table.
6. Maximum output growth is three bytes.
7. The bridge cannot mutate `VoiceState`, memory, beliefs, ontology, routing, tools, CHARGE, companion state, or autonomous actions.
8. The probe emits `gate_passed: true` before the production binary is built.

## Authority boundary

ΩV1-D grants two narrowly scoped authorities:

- the HTTP chat path may call the bounded bridge;
- the bridge may alter the eligible opener in returned live text.

It grants no authority to:

- invent, remove, negate, qualify, or reorder claims;
- change confidence, polarity, commitments, abstentions, or prohibited implications;
- inspect unrestricted memory or raw persistence state;
- mutate any durable or session voice dimension;
- select routes, tools, CHARGE discharge, or autonomous actions;
- learn new replacement strings;
- expand the eligible surface without a new preregistered gate.

## Implementation sequence

1. Commit this preregistration before the canary kernel.
2. Implement the deterministic kernel and focused tests.
3. Add a machine-readable ΩV1-D probe to the Render Docker build.
4. Wire the kernel at the `/chat` response boundary behind `omega-v1-live-bridge`.
5. Require external Render execution before calling ΩV1-D PASS.

A PASS authorizes ΩV1-E, the independent language verifier. It does not authorize a learned renderer, automatic voice evolution, or broader live rewriting.
