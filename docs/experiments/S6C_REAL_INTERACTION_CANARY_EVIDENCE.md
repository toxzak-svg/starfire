# S6-C Real-Interaction Canary Evidence — Preregistration

Status: **FROZEN BEFORE IMPLEMENTATION — one post-freeze channel erratum recorded below**

## Post-freeze erratum

The original frozen text mistakenly listed `ExternalEvaluator` as a valid direct witness.
That conflicts with the already-frozen S5-B contract, which reserves external evaluators
for offline blind pairwise comparison and rejects them on direct observed evidence as
`WrongEvidenceChannel`.

The correction is restrictive rather than permissive:

- direct intake accepts only `UserObservation` or channel-compatible `Environment` evidence;
- `ExternalEvaluator` on direct intake must be rejected atomically;
- external evaluators remain required for blind pairwise intake.

The original preregistration commit remains preserved at
`469654d1ce2008888e6804d407d17d37379a88f1`. This erratum does not loosen any gate,
change any outcome threshold, or grant new authority.

## Question

Can Starfire accept consented, independently witnessed interaction outcomes from a bounded real-conversation canary, convert them into the existing S5-B outcome contract, and evaluate them through the frozen S5-C held-out gates without allowing the response generator to grade itself, retaining raw conversation content, leaking holdout assignment, or granting runtime authority?

## Scope

S6-C is an evidence-intake and provenance boundary. It is not a new response generator, policy learner, router, memory system, or autonomous controller.

The implementation may:

- seal an existing S5-B trial before any outcome is observed;
- assign and record its S5-C split before outcome intake;
- accept only typed, digest-addressed witness attestations;
- atomically import valid direct or blind pairwise evidence into S5-B;
- run the existing S5-C evaluator over the resulting frozen ledger;
- report whether evidence is real or synthetic.

The implementation may not:

- retain prompts, responses, names, email addresses, free-form feedback, or other raw conversation content;
- allow `WitnessSource::ResponseGenerator`;
- allow `WitnessSource::ExternalEvaluator` on direct observed evidence;
- permit the producer and witness identities to share the same non-zero digest;
- change a trial's split after sealing;
- resolve an unshown arm from direct user evidence;
- treat a synthetic fixture as real interaction evidence;
- wire itself into `Runtime::chat()` by default;
- alter routing, generated text, beliefs, ontology, persistence, tools, capabilities, or actions;
- grant promotion from an intake-mechanism probe alone.

## Evidence origins

Two origins are distinguished in the type system:

1. `RealInteraction`
2. `SyntheticFixture`

Production-default intake must reject `SyntheticFixture`. A test-only configuration may admit synthetic fixtures to validate the mechanism, but any report containing synthetic evidence is promotion-ineligible.

## Trial sealing

Before outcome intake, each canary trial must be sealed against an existing S5-B `InteractionTrial` with:

- non-zero study, protocol, consent, operator, subject-scope, and context digests;
- a valid S5-C split computed from the frozen split policy;
- the original issued, not-before, and expiry times;
- the delivered arm, if any;
- the complete arm set and policy digests;
- a canonical seal digest.

A trial cannot be sealed twice or resealed into a different split.

## Direct witness intake

A direct witness attestation must:

- reference a sealed trial with a delivered arm;
- use `UserObservation` or channel-compatible `Environment` evidence as its witness source;
- reject `ExternalEvaluator` as the wrong direct-evidence channel;
- carry non-zero witness, producer, consent, and evidence digests;
- have a witness digest different from the producer digest;
- match the sealed consent digest;
- fall within the trial's observation window;
- be unique for that trial;
- resolve only the delivered arm through the existing S5-B method.

Neutral follow-up remains inconclusive and must not be converted into success or failure.

## Pairwise intake

A pairwise attestation must:

- reference two distinct arms present in the sealed trial;
- use an independent external evaluator;
- carry distinct, non-zero render digests and a non-zero blinded-order digest;
- carry non-zero evaluator, producer, consent, and evidence digests;
- have an evaluator digest different from the producer digest;
- match the sealed consent digest;
- fall within the trial's observation window;
- be unique for the unordered arm pair;
- import through the existing S5-B pairwise method.

## Atomicity and replay

Every operation uses optimistic concurrency. Rejected intake must leave both the canary ledger and the S5-B/S4 ledgers unchanged.

The canary event log must replay deterministically. Replay validates:

- event order;
- seal canonicality;
- split immutability;
- witness independence;
- evidence origin;
- duplicate prevention;
- typed export shape.

## Evaluation gate

The canary report may be promotion-eligible only when all conditions hold:

- the underlying S5-C verdict is `Pass`;
- every imported attestation has origin `RealInteraction`;
- no independence, consent, timing, split, duplicate, or atomicity violation occurred;
- both opaque-subject and temporal holdouts are present;
- development evidence remains excluded from the verdict;
- live response, routing, belief-promotion, persistence, and action authority remain false.

A synthetic mechanism probe can pass the engineering gate while remaining promotion-ineligible.

## Frozen engineering probe

The committed probe must demonstrate:

- production-default rejection of synthetic intake;
- successful synthetic-fixture intake only under explicit test configuration;
- rejection of response-generator witnesses;
- atomic rejection of external evaluators on the direct-evidence channel;
- rejection of same-identity producer/witness pairs;
- rejection of consent mismatch, early evidence, expired evidence, duplicate direct evidence, duplicate pairwise evidence, unknown trials, and unsealed trials;
- exact preservation of S5-B and S4 state after rejected operations;
- correct direct-arm and pairwise imports;
- deterministic canary replay;
- deterministic S5-B replay;
- an S5-C report generated from imported evidence;
- promotion ineligibility because the probe uses synthetic fixtures;
- zero runtime, routing, belief, persistence, tool, or action authority.

## Claim boundary

A PASS establishes only that the S6-C intake mechanism conforms to this synthetic preregistered contract as restrictively corrected above. It does not establish that real users prefer the companion policy, that the canary is statistically powered, that broader deployment is safe, or that Starfire has autonomous intelligence or AGI.