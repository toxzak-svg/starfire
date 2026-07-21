# ARISE-A0 Edge Bridge

## Status

ARISE-A0 is an opt-in, deterministic proof-of-mechanism for terminal-first language execution on bounded hardware. It is disabled by default behind the `arise-edge` Cargo feature.

This stage does not claim a trained state-space model, improved conversational quality, or production superiority. It freezes the execution contract that a later learned proposer, reverse-obligation model, and span renderer must satisfy.

## Architectural contract

ARISE-A0 performs four steps:

1. Begin from one or more typed terminal obligations.
2. Walk explicit dependencies backward to produce a canonical forward execution order.
3. Render bounded local spans from adjacent ready obligations.
4. Independently reconstruct obligation identifiers from text before reducing the residual ledger.

An accepted span must strictly reduce unresolved semantic obligations. A failed multi-obligation span is split recursively and retried locally. A failed singleton leaves residual pressure unresolved and terminates the trace as `Rejected`.

## Edge bounds

The default reference engine freezes these limits:

- maximum obligations: 32
- maximum obligations per span: 4
- maximum span bytes: 512
- maximum repair depth: 5
- deterministic ordered maps and sets only
- no growing attention or KV-cache state
- no unbounded candidate search

The runtime-shadow bridge is additionally capped at 16 final-text segments and stores only an in-memory typed snapshot plus an opaque FNV-1a body digest.

## Integration boundary

The `star::arise_edge` module is registered only when `arise-edge` is enabled. It exports:

- typed terminal obligations and dependency edges
- reverse planning
- pluggable `SpanRenderer` and `TransitionVerifier` traits
- a deterministic reference renderer
- an independent lexical transition verifier
- recursive local repair
- residual and authority traces
- `observe_runtime_response`, an inert bridge for auditing a completed response without changing it

ARISE-A0 is not attached to `Runtime::chat()` and cannot alter returned text. The bridge exists so a later shadow-only runtime splice can be introduced as a separate reviewed gate rather than smuggling generation authority into this foundation.

## Authority boundary

Only `runtime_shadow_observation` is true. All of the following remain false:

- generated-text influence
- raw-prompt access
- memory access
- persistence authority
- routing authority
- belief or ontology promotion
- tool selection
- CHARGE discharge
- autonomous action

## Falsification gates

The stage is eligible to merge only when the scoped CI proves:

1. the default library still compiles without `arise-edge`;
2. the feature-gated library and probe compile under `--locked`;
3. scoped Clippy is clean;
4. reverse dependency ordering is canonical;
5. every accepted span strictly reduces residual;
6. a deliberately damaged grouped span repairs by local splitting;
7. cyclic dependency graphs are rejected;
8. the bridge probe reaches `Pass` with zero final residual;
9. every non-observation authority flag remains closed.

## Next gate

ARISE-A1 may replace only the reference span renderer with a tiny quantization-aware recurrent proposer. It must retain the A0 request type, independent verifier, residual monotonicity, local repair, deterministic neutral fallback, and closed authority boundary. Runtime text influence requires a separate canary preregistration and matched control.
