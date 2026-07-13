# STLM L0-C Deterministic Renderer Result

Status: **PASS — deterministic realization and lexical-firewall mechanics only**

## Scientific provenance

The frozen preregistration was committed before implementation:

```text
5b2a87fa9e4054de272ccdd6971fc3416723c4b0
```

The authoritative committed-source verification was:

```text
workflow: STLM Deterministic Renderer L0-C CI
run id:   29272412070
head:     0bb4c438319cea0502f82c6ba6a74ae5a36b7535
result:   success
artifact: 8288075892
artifact digest:
  sha256:5b09396bcf09f1087d7c4ff9b15f60e1c063626c7d74b475cbcfc5f1b2857ad2
```

The run used the restored read-only, check-only workflow. It enforced canonical
`rustfmt`, compiled the feature-gated renderer and probe, completed scoped
Clippy without L0-C findings, ran deterministic renderer unit contracts, and
executed the frozen probe with shell `pipefail` enabled.

## Invalid preliminary run

An earlier workflow run (`29271712402`) was initially displayed as successful,
but its artifact JSON contained:

```text
terminal_classification: FAIL
gate_passed: false
polarity_tampering_rejected: false
```

That workflow piped the probe through `tee` without `set -o pipefail`, so the
shell reported the successful `tee` exit code instead of the failing probe exit
code. The preliminary run is explicitly invalid and is not used as evidence.

The defect was corrected by:

- enabling `pipefail` for the frozen probe;
- changing the polarity-tampering fixture to preserve byte length so the
  semantic-marker control is isolated from alignment-length failure;
- rejecting every noncanonical gap between aligned operation spans;
- rejecting trailing unaligned text;
- recomputing operation, claim, verification-step, character, sentence, and
  paragraph costs from the untrusted realization payload;
- requiring serialized cost fields to equal the independently recomputed costs;
- adding frozen controls for unaligned injected text and under-reported costs.

## Frozen result

```text
deterministic text:                              true
deterministic alignments:                        true
deterministic costs:                             true
deterministic digest:                            true
integrity verified:                              true
all nine operations aligned:                     true
all typed reference kinds present:               true
negative polarity preserved:                     true
epistemic markers preserved:                     true
forbidden form absent:                           true
style changes layout:                            true
style preserves semantic alignment:              true
question policy preserved:                       true
tampered program rejected:                       true
wrong program digest rejected:                   true
wrong subject scope rejected:                    true
unsorted lexical claims rejected:                true
missing lexical binding rejected:                true
unused lexical binding rejected:                 true
prohibited claim binding rejected:               true
malformed lexical text rejected:                 true
forbidden lexical binding rejected:              true
malformed forbidden-form list rejected:          true
forbidden generated text rejected:               true
budget overflow rejected without truncation:     true
lexical digest tampering rejected:               true
realization digest tampering rejected:           true
alignment overlap rejected:                      true
operation reorder rejected:                      true
operation omission rejected:                     true
unaligned gap rejected:                          true
under-reported costs rejected:                   true
polarity tampering rejected:                     true
authority boundary closed:                       true
detailed character cost:                         836
detailed sentence count:                         10
detailed paragraph count:                        9
alignment count:                                 9
gate passed:                                     true
```

## Contract established

L0-C now provides a feature-gated deterministic reference renderer containing:

- a lexical binding table bound to the exact semantic-program digest and subject
  scope;
- canonical, exact-coverage bindings for claim clauses, observations, missing
  variables, and predictions;
- prohibited-claim and forbidden-surface-form exclusion;
- fixed realization grammar for `Assert`, `Qualify`, `Contrast`, `Correct`,
  `Explain`, `Acknowledge`, `RequestEvidence`, `Commit`, and `Abstain`;
- polarity-selected positive or negative clauses;
- fixed epistemic markers;
- deterministic style-dependent layout without semantic-operation changes;
- exact byte-span proof alignments for every operation;
- exact separator validation and rejection of all unaligned text;
- independently recomputed budget accounting;
- deterministic lexical-table and realization digests;
- tamper-detecting integrity verification.

The digests are deterministic replay and accidental-tamper checksums. They are
not cryptographic authentication and are not represented as such.

## Authority boundary

Every renderer authority flag remained false:

```text
Runtime::chat() wiring:           false
live generated-text influence:    false
raw conversation access:          false
unrestricted memory access:       false
persistence authority:            false
routing authority:                false
companion mutation authority:     false
belief-promotion authority:       false
ontology-promotion authority:     false
tool-selection authority:         false
CHARGE-discharge authority:       false
autonomous-action authority:      false
```

## Interpretation

This PASS establishes that a validated semantic program plus an exact bounded
lexical table can be realized deterministically into text with ordered proof
alignments, preserved polarity and epistemic markers, complete budget
accounting, and fail-closed rejection of the frozen malformed controls.

It does not establish that Starfire chooses correct semantic programs, that the
reference wording is naturally fluent, that renderer-supplied alignments are
independent evidence, that an open-ended renderer cannot hallucinate, that a
learned verifier is reliable, that users prefer the output, or that the system
is not an LLM wrapper under matched attribution tests.

## Next gate

The next implementation slice is **L1: Independent Language Verifier**.

L1 must not trust renderer-provided alignments as its semantic judgment. It
must independently reconstruct a normalized projection from rendered text and
check required-operation coverage, unsupported claims, claim polarity,
epistemic status, prohibited implications, reference bindings, sensitive-form
leakage, and budget compliance. It remains offline and outside
`Runtime::chat()` until separately preregistered gates pass.
