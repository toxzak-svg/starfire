# STLM L0-C — Deterministic Reference Renderer

Status: **PREREGISTERED — no result yet**

## Purpose

L0-C tests whether a renderer can convert a validated
`SemanticResponseProgram` into deterministic surface text and proof alignments
without receiving the raw conversation, unrestricted memory, tools, persistence,
or cognitive mutation authority.

This is a reference renderer and information-firewall experiment. It does not
test neural generation, human-level fluency, open-ended semantic equivalence, or
real conversation benefit.

## Frozen hypothesis

A deterministic renderer supplied only with a validated semantic program and a
scope-bound lexical binding table can:

1. realize every discourse operation in order;
2. represent every required claim;
3. preserve claim polarity and epistemic status;
4. produce exact operation-to-span and claim-to-span alignments;
5. obey output and compute budgets without truncating required semantics;
6. reject missing, extra, prohibited, mismatched, malformed, or forbidden
   lexical bindings atomically;
7. replay byte-identically;
8. remain independent of `Runtime::chat()` and all side-effect authority.

## Renderer input firewall

The renderer may receive only:

- a validated `SemanticResponseProgram`;
- the exact program digest;
- the exact subject scope;
- claim clauses selected by the planner;
- bounded labels for referenced observations, missing variables, and
  predictions;
- a canonical list of forbidden surface forms.

The renderer may not receive:

- the raw user message or unrestricted conversation transcript;
- arbitrary memory text;
- retrieval or tool handles;
- companion-state mutation handles;
- belief, ontology, routing, persistence, CHARGE, or action handles;
- prose instructions asking it to reason beyond the semantic program.

## Canonical lexical contract

The lexical table must be bound to the program digest and subject scope.
Collections must be strictly ordered by typed identifier. Every string must be
nonempty, bounded UTF-8 without control characters, leading or trailing
whitespace, or repeated whitespace.

The table contains:

- one positive and one negative clause for every authorized claim referenced by
  an operation;
- labels only for referenced observations, missing variables, and predictions;
- no entry for a prohibited claim;
- no unused claim or reference entry;
- a strictly sorted, duplicate-free list of forbidden surface forms.

## Frozen realization grammar

The reference renderer uses a fixed grammar. Wording may be plain; semantic
coverage is mandatory.

- `Assert(c)` renders the bound clause with the claim's frozen epistemic marker.
- `Qualify(c, s)` renders the same clause with exactly status `s`.
- `Contrast(a, b)` renders both clauses and an explicit contrast relation.
- `Correct(a, b)` renders both clauses and an explicit replacement relation.
- `Explain([c...])` renders all listed clauses in their frozen order as jointly
  relevant support without inventing a causal direction absent from the
  program.
- `Acknowledge(o)` renders only the bound observation label.
- `RequestEvidence(v)` renders an explicit request for the bound variable.
- `Commit(p)` renders an explicit bounded commitment to the bound prediction.
- `Abstain(r)` renders a fixed reason-specific abstention.

Claim polarity selects the positive or negative bound clause. Epistemic markers
are fixed and testable:

```text
Certain   -> "I know that"
Probable  -> "It is probable that"
Possible  -> "It is possible that"
Uncertain -> "I am uncertain whether"
Unknown   -> "I do not know whether"
```

## Proof-carrying output

The renderer emits:

- final UTF-8 text;
- exact byte spans for every operation;
- claim IDs represented in every span;
- the source program digest;
- lexical-table digest;
- renderer identity and grammar version;
- deterministic realization digest;
- operation and character cost.

Alignments must be nonoverlapping, ordered, in bounds, and collectively cover
all rendered operation segments. The renderer does not judge semantic fidelity;
these alignments are witnesses for later independent verification.

## Frozen positive fixture

The primary fixture must include all nine discourse operation variants and both
positive and negative claim polarity. It must pass under at least two style
profiles without changing claim IDs, operation order, polarity, or epistemic
status.

A repeated render with identical inputs must produce byte-identical text,
alignments, costs, and digests.

## Frozen negative controls

The probe must reject all of the following without partial output or state:

- invalid or digest-tampered semantic program;
- lexical table bound to another program digest or subject scope;
- zero, duplicate, unsorted, or unused lexical IDs;
- missing lexical binding for a referenced claim or typed reference;
- lexical binding for a prohibited claim;
- positive or negative clause that is empty, oversized, malformed, or contains
  a forbidden surface form;
- forbidden forms that are empty, duplicated, unsorted, or malformed;
- output exceeding character, sentence, paragraph, operation, claim, or
  verification-step budgets;
- style that requires questions while the program forbids questions;
- generated text containing a forbidden form;
- malformed or overlapping alignment span;
- an operation omitted, reordered, or represented twice;
- claim polarity or epistemic marker mismatch.

## Frozen verdict

`PASS` requires every positive gate and negative control to pass, exact repeated
rendering, complete ordered alignments, zero prohibited claim emission, zero
forbidden surface forms, and every authority flag remaining false.

`FAIL` is mandatory if a malformed or mismatched table renders, any required
operation or claim is omitted, polarity or epistemic status changes, output is
silently truncated, a prohibited or forbidden form appears, replay differs, or
any runtime authority opens.

`INFRASTRUCTURE_FAILURE` is reserved for failure to compile or execute the
frozen probe. It may not be converted into PASS.

## Authority boundary

L0-C may add only renderer types, lexical validation, deterministic reference
realization, proof alignments, tests, a frozen probe, and scoped CI. It adds no
`Runtime::chat()` wiring, learned model, independent semantic verifier,
persistence authority, routing, companion mutation, belief or ontology
promotion, tool selection, CHARGE discharge, or autonomous action.

## Claim boundary

A PASS would establish only deterministic realization and information-firewall
mechanics under the frozen synthetic fixture. It would not establish natural
language fluency, semantic correctness of the planner, open-ended hallucination
resistance, neural renderer safety, human preference, non-wrapper attribution,
autonomous agency, or AGI.
