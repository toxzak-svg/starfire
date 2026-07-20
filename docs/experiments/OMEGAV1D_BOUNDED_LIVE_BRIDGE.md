# ΩV1-D: Bounded Deterministic Live Bridge

**Status:** Preregistered implementation gate  
**Parent gate:** ΩV1-C PASS on externally executed Render build, July 20, 2026  
**Feature flag:** `omega-v1-live-bridge`

## Scientific question

Can Starfire make one detectable, deterministic expression change in the live response path while preserving the cognition-produced response body exactly and retaining an unconditional neutral fallback?

ΩV1-D is deliberately smaller than a general renderer. It is the first live canary for the cognitive-to-voice bridge, not permission for free-form rewriting.

## Pre-execution boundary amendments

Before any external D0 execution:

1. The selector input was narrowed from `prompt + NUL + protected_body` to `protected_body` alone. Passing the raw user prompt would contradict the frozen `raw_conversation_access: false` authority boundary.
2. Lexical replacement phrases were removed because alternatives such as “I’m with you” could imply agreement or comprehension not present in the semantic plan. The canary now changes only the separator after the exact same opener words.

No gate result was observed before either amendment.

## Frozen canary transformation

The neutral input is the completed post-reranker, post-`VoiceEngine` response.

A response is eligible only when it begins with the exact byte sequence `Here for it. `, including the trailing space.

The remainder of the response is the protected body. The bridge may replace only the eligible opener with one member of this closed separator-only table:

```text
Here for it.\n
Here for it.\n\n
```

The backslash notation above denotes newline bytes. The words and punctuation of the opener remain identical; only the separator between opener and protected body changes.

Selection is deterministic: FNV-1a over the protected body, modulo the table length.

No claim word, punctuation mark, hedge, question, reference, or protected-body byte may change.

## Mandatory neutral fallback

The original response must be returned byte-for-byte when any of these conditions holds:

- the exact eligible opener is absent;
- the protected body is empty or whitespace-only;
- the protected body exceeds 4,096 bytes;
- the selected opener is not in the frozen table;
- the selected opener changes the words or punctuation `Here for it.`;
- the rendered text does not end with the exact protected body;
- any internal invariant fails.

Fallback is a successful bounded outcome, not an error.

## Frozen gates

The implementation passes only if all of the following hold:

1. The same neutral response produces exactly the same output and decision metadata.
2. Every applied transformation preserves the protected body byte-for-byte.
3. Ineligible responses pass through byte-for-byte.
4. Empty, whitespace-only, and oversized bodies pass through byte-for-byte.
5. Applied output uses only the frozen separator table.
6. Every replacement preserves the exact opener words and punctuation.
7. Maximum output growth is one byte.
8. The bridge cannot read the raw prompt or mutate `VoiceState`, memory, beliefs, ontology, routing, tools, CHARGE, companion state, or autonomous actions.
9. The probe emits `gate_passed: true` before the production binary is built.

## Authority boundary

ΩV1-D grants two narrowly scoped authorities only after D1 wiring:

- the HTTP chat path may pass the completed neutral response to the bounded bridge;
- the bridge may alter only the eligible opener separator in returned live text.

It grants no authority to:

- read the raw user prompt or unrestricted conversation history;
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
