# S6-C Limited Runtime Canary — Result

Status: **PASS**

## Scientific provenance

The S6-C mechanics contract was frozen before implementation at:

```text
ad71b18d7bcaca6aef48db2042a625e1a3586aaf
```

The first complete verdict-producing committed-source execution was:

```text
workflow: Companion Runtime Canary S6-C CI
run id:   29271528107
run no:   5
head:     1d4fdfcef162089a7d8a6c90afc625aa60415f36
artifact: 8287683041
artifact digest:
sha256:d941f035f5f02347df2475c4d45fcbdb56608147733e6d0bc4e7c2a9219174bd
```

The source was synchronized with current `main` through explicit two-parent merge
commit `1bf5e62b48f280f38753a590480ec576161d3a2a`, preserving both the STLM program
and the preregistered S6-C history.

## Terminal result

```text
S6-C gate: PASS
unauthorized applied turns: 0
committed turns:             9
companion-applied turns:     2
neutral fallbacks:           7
```

Every frozen canary control passed:

- preregistration preceded implementation;
- production activation rejected frozen-simulation evidence;
- held-out-conversation evidence activated only with complete canary metadata;
- preparation left live state unchanged;
- pending-turn debug output exposed no response body;
- applied turns required the companion-derived S5-B arm;
- neutral fallbacks required the neutral-default S5-B arm;
- wrong arm, subject, session, context, companion version, policy digest, and
  trial timing were rejected atomically;
- sensitive and disallowed contexts produced exact neutral fallback;
- companion-version drift produced exact neutral fallback;
- duplicate turns produced exact neutral fallback;
- the applied-turn budget was enforced;
- session expiry produced exact neutral fallback;
- revocation was immediate;
- audit replay was deterministic.

## Inherited regressions

The same workflow reran the frozen parent probes:

```text
S6-A bounded live-policy gate: PASS
S6-B adversarial stress gate:  PASS
```

S6-A retained:

- exact neutral fallback;
- metadata-only applied planning;
- version-bound activation;
- deterministic replay;
- zero routing, persistence, belief-promotion, or action authority.

S6-B retained:

- forged-authorization and forged-policy rejection;
- event-order, chain, deletion, and semantic-tamper rejection;
- exact trusted replay;
- zero unauthorized applied turns;
- zero routing, persistence, belief, ontology, or action authority.

## Build and review gates

The authoritative run passed:

- scoped Rust formatting;
- library compilation with `companion-runtime-canary`;
- S6-A, S6-B, and S6-C example compilation;
- scoped S6-C Clippy with no S6-C findings;
- deterministic S6-C unit contracts;
- frozen S6-A regression;
- frozen S6-B regression;
- frozen S6-C mechanics probe.

Repository-wide pre-existing Clippy warnings outside the S6-C files remained
non-blocking and were preserved in the artifact log. No S6-C source warning was
reported.

## Authority boundary

The PASS does **not** attach the canary to default `Runtime::chat()`.

The following remained false:

```text
default Runtime::chat() wiring: false
routing authority:               false
persistence authority:           false
belief-promotion authority:      false
ontology-promotion authority:    false
tool-selection authority:        false
autonomous-action authority:     false
```

The canary can influence only a committed response plan's bounded style and
reranker metadata after a matching S5-B trial is registered. The baseline body
and semantic intent remain Starfire-owned.

## Claim boundary

This PASS establishes only the mechanics of:

- session and subject isolation;
- prepare/register/commit response delivery;
- atomic rejected commits;
- delivered-arm binding;
- exact neutral fallback;
- bounded revocation and replay.

It does **not** establish that companion-derived policy improves real
conversations. No conversational-benefit claim is supported until real,
independently witnessed interaction outcomes are frozen and evaluated through a
separate S5-C held-out study. It does not support AGI, consciousness, unrestricted
personalization, persistent live authority, or autonomous action claims.
