# ARISE-A1 Typed Semantic-Program Shadow

## Status

Experimental, default-off, stacked on ARISE-A0, and shadow-only.

Feature: `arise-typed-plan`

The feature depends on:

- `arise-edge`
- `independent-language-verifier`

It does not replace Starfire's semantic response-program types, renderer, or verifier.

## Purpose

ARISE-A0 proved bounded reverse-obligation planning, independent lexical reconstruction, monotonic residual reduction, and local repair. A1 connects that mechanism to Starfire's existing typed cognition boundary without granting it control over live text.

A1 consumes:

1. a validated `SemanticResponseProgram`,
2. its validated scope-bound `LexicalBindingTable`, and
3. an already-constructed final response body.

It then:

1. converts canonical discourse operations into bounded ARISE obligations,
2. reverse-plans from the terminal operation,
3. independently reconstructs operations from final text using grammar-v2 verification,
4. compares reconstructed operation order with the ARISE plan,
5. records semantic residual and rejection reason, and
6. returns or stores telemetry only.

Renderer alignments are never accepted as evidence.

## Runtime integration

`ResponseSemanticShadowExt` is an opt-in extension on the existing runtime `Response` type:

```rust
let response = Response::with_body(intent, body)
    .observe_semantic_shadow(&program, &lexical_table);
```

The extension consumes and returns the same `Response`. It does not mutate the body or response metadata.

No existing handler is migrated automatically in A1. Handler adoption requires a validated semantic program and lexical table supplied by an authorized cognition path.

## Frozen authority boundary

A1 may read:

- validated semantic-program fields,
- validated lexical bindings,
- the final response body,
- independently reconstructed operation identifiers, and
- bounded digests and counts.

A1 may not:

- alter returned text,
- read raw prompts,
- read unrestricted memory,
- persist observations,
- route requests,
- select tools,
- promote beliefs or ontology,
- discharge CHARGE,
- authorize autonomous action, or
- treat a verifier PASS as new cognitive authority.

## Stored snapshot

The in-memory snapshot contains only:

- program and lexical-table digests,
- subject-scope identifier,
- opaque final-body digest,
- operation and required-claim counts,
- reconstructed-operation count,
- initial and final semantic residual,
- terminal classification,
- typed rejection reason, and
- explicit authority flags.

Raw text, prompts, memories, claim contents, and renderer alignments are not stored.

## A1 falsification gates

A1 must remain draft unless all gates pass:

1. Default Starfire library compilation remains unchanged.
2. A0 feature compilation, nine unit contracts, and deterministic probe remain green.
3. `arise-typed-plan` compiles with the independent verifier stack.
4. A valid two-operation program reconstructs with residual `2 -> 0`.
5. A missing operation is rejected.
6. A forbidden surface is rejected.
7. The response extension preserves the body byte-for-byte.
8. Repeated probes produce byte-identical snapshots.
9. Scoped Clippy reports no A0 or A1 findings.
10. Every text-influence, persistence, routing, tool, CHARGE, belief, ontology, memory, and action authority flag remains false.

## Non-claims

A1 does not demonstrate:

- open-ended language generation,
- a learned recurrent or state-space proposer,
- improved user-visible answer quality,
- production latency suitability,
- broad semantic equivalence beyond the verifier-ready grammar, or
- authority to replace Starfire's existing response path.

## Next gate

A1.1 may migrate one deterministic, low-risk handler into matched shadow observation. The legacy response remains authoritative and byte-exact. Promotion beyond shadow requires held-out evidence and a separately preregistered canary.
