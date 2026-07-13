# STLM L1 Identifiability Addendum

**Status:** frozen before L1 implementation  
**Date:** 2026-07-13  
**Applies to:** `STLM_L1_INDEPENDENT_LANGUAGE_VERIFIER.md`

## Finding discovered during preregistration review

The accepted L0-C grammar is deterministic but not fully invertible.

`Assert(claim)` and `Qualify { claim, status }` both render as the same
status marker plus the same polarity-selected lexical clause whenever the
qualifying status equals the authorized claim status. L0-B validation requires
that equality, so the two operation kinds are observably identical in L0-C
text.

Therefore, an L1 implementation that claims to reconstruct all nine operation
kinds from L0-C text would have to consult the expected operation labels or the
renderer alignments. Either shortcut would violate the frozen independence
boundary.

A second possible ambiguity exists when two lexical bindings produce the same
selected surface clause. L0-C validates coverage and canonical ordering but
does not require inverse-parse uniqueness.

## Frozen remediation

L1 will not reinterpret the existing L0-C PASS. The v1 renderer and its result
remain frozen.

Instead, L1 adds an explicitly separate verifier-ready deterministic grammar:

```text
renderer identity: DeterministicVerifierReadyV2
grammar version:   2
```

The only semantic-format change required for operation identifiability is:

```text
Assert:   <epistemic marker> <polarity-selected clause>.
Qualify:  Qualification: <epistemic marker> <polarity-selected clause>.
```

All other operation forms retain their L0-C wording unless an implementation
finding proves another collision before the verdict-producing run. Any such
finding requires another committed preregistration addendum before code changes.

The verifier-ready renderer must remain deterministic, bounded, proof-carrying,
and authority-free. It may share validated data structures with L0-C, but L1
semantic acceptance must not call the forward renderer or consume its
alignments.

## Lexical ambiguity rule

L1 must construct its inverse lexicon from the validated lexical table and the
authorized program. If more than one claim/polarity/status tuple or more than
one typed reference can explain the same candidate surface at a parse
position, verification returns `AmbiguousSurfaceBinding`.

Ambiguity is never resolved by operation order, expected claim IDs, renderer
alignments, or best-effort ranking.

## Updated positive gate

The all-nine-operation reconstruction gate applies to verifier-ready grammar
v2. L0-C grammar v1 remains valid evidence for deterministic forward
realization only and is not relabeled as independently invertible.

## Updated regression requirement

The L1 workflow must run both:

1. the frozen L0-C v1 renderer regression unchanged; and
2. verifier-ready v2 renderer and inverse-verifier gates.

A PASS requires the v1 regression to remain green and the v2 inverse verifier
to pass every frozen control.