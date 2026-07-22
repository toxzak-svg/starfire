# STLM L1-A Verified Improvisation

**Status:** implemented offline experiment  
**Feature:** `verified-improvisation`  
**Runtime authority:** none

## Purpose

STLM L1-A adds controlled surface unpredictability without allowing wording to alter Starfire's authorized meaning.

The stage extends the committed ΩV1-F1R1 surface lattice with:

- a bounded conversational microstate;
- a caller-supplied, replayable entropy seed;
- beam search over committed surface variants;
- recent-opening and recent-surface fingerprints;
- strong anti-repetition penalties;
- explicit blandness and compression scoring;
- exact mapping back to the underlying remediation variants;
- nested independent verification;
- frozen grammar-v2 neutral fallback.

The design rule is:

> Meaning remains predictable. Expression may vary.

## Data flow

```text
validated SemanticResponseProgram
        |
        v
bounded LexicalBindingTable
        |
        v
ΩV1-F1R1 committed surface lattice
        |
        v
position-aware improvisation lattice
        |
        v
microstate + replayable seed + recent-language trace
        |
        v
bounded candidate beam
        |
        v
exact improvisation parser
        |
        v
reconstructed ΩV1-F1R1 surface
        |
        v
RemediatedVerifier
        |
        +--> accept verified wording
        +--> grammar-v2 neutral fallback
```

## Conversational microstate

The selector accepts six bounded basis-point dimensions:

- directness;
- warmth;
- energy;
- compression;
- playfulness;
- novelty pressure.

These values rank already-authorized wording. They cannot add claims, alter confidence, reorder semantic operations, access memory, or select actions.

## Unpredictability and replay

The entropy seed changes candidate preference and tie-breaking. The seed is included in the selection payload, so a selected response can be reproduced exactly.

No wall-clock randomness is used. A future caller should derive the seed from an explicit turn nonce plus bounded conversational state and persist that nonce with the trace.

## Anti-repetition trace

`RecentLanguageTrace` stores at most 64 opening fingerprints and 64 complete-surface fingerprints. A candidate that repeats a recent opening receives a large penalty. An exact repeated surface receives a larger penalty.

The trace contains hashes only. This stage does not persist the trace and does not receive unrestricted conversation text.

## Verification boundary

The improvisation verifier does not trust selector metadata.

It:

1. rebuilds the committed improvisation lattice;
2. parses the returned text against exactly one variant per semantic operation;
3. reconstructs the underlying ΩV1-F1R1 surface;
4. invokes `RemediatedVerifier` on that reconstruction;
5. recomputes actual output and verification budgets;
6. rejects stale digests, ambiguity, prohibited text, or budget overflow.

Any failure returns the existing verifier-ready neutral realization.

## Frozen limits

```text
maximum beam width:        16
maximum complete candidates: 32
maximum recent openings:   64
maximum recent surfaces:   64
```

## Authority boundary

This implementation has no authority to:

- influence `Runtime::chat()`;
- alter an HTTP response;
- inspect a raw prompt or unrestricted transcript;
- retrieve unrestricted memory;
- mutate voice or companion state;
- persist data;
- promote beliefs or ontology;
- route tasks;
- select tools;
- discharge CHARGE;
- perform autonomous actions.

Passing unit or CI checks does not authorize live use.

## Current tests

The scoped feature tests require that:

- identical program, request, seed, and trace replay exactly;
- multiple seeds produce multiple independently verified surfaces;
- recording a selected response causes the same seed to choose a different opening;
- all live and autonomous authority flags remain false.

## Next gate

Before any shadow integration, L1-B should run a held-out conversational corpus comparing:

- neutral grammar-v2;
- ΩV1-F1R1 remediation;
- L1-A verified improvisation.

Required measures should include semantic acceptance, exact fallback rate, repeated-opening rate, repeated-surface rate, human preference, perceived sharpness, awkward-transition rate, latency, and candidate-search cost.
