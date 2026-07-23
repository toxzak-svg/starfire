# EI-0A Canonical Episode Contracts

**Program:** Emerging Intelligence EI-0  
**Stage:** EI-0A  
**Tracking:** #149 and #150  
**Status:** implementation candidate; no EI experiment verdict

## Purpose

EI-0A defines the data contract required to represent one bounded developmental episode before Starfire receives any developmental persistence or learning authority.

The contract records:

- observation and evidence provenance;
- predictions made before action;
- selected strategy and intention;
- bounded action and declared cost;
- independently witnessed outcome and evaluation;
- proposal-only learning-update references;
- accepted proposal references after evaluation;
- a closed authority snapshot;
- deterministic provenance, partition, schema version, and digest.

## Canonical lifecycle

```text
Observed -> Predicted -> Acted -> OutcomeObserved -> Evaluated
```

Each phase fails closed when fields from a later phase appear prematurely. Any acted or scored episode must contain at least one prediction created before the action. Outcomes must reference the recorded action, evaluations must reference the recorded outcome, and learning proposals must reference the recorded evaluation.

## Canonical encoding and replay

`SealedCognitiveEpisode` contains:

1. the EI-0A schema version;
2. one validated `CognitiveEpisode`;
3. a domain-separated deterministic payload checksum.

Decoding accepts only the exact canonical byte representation. Valid JSON with reordered fields, reordered identifier collections, unsupported schema versions, malformed identifiers, dangling references, or a modified digest is rejected.

The checksum is intended for deterministic replay and corruption detection inside the frozen EI experiment pipeline. It is not a cryptographic signature and does not authenticate an untrusted producer.

## Authority boundary

The feature `emerging-intelligence-contracts` is disabled by default. EI-0A does not:

- change `Runtime::chat()`;
- read or write SQLite or other persistent state;
- apply learning updates;
- alter response generation, voice, routing, beliefs, ontology, or tools;
- authorize autonomous actions;
- establish cumulative improvement or emerging intelligence.

`AuthoritySnapshot` defaults fully closed, and validation rejects any episode that claims one of these powers.

## Verification gate

The dedicated CI workflow checks:

- scoped Rust formatting;
- locked feature-gated compilation;
- Clippy with warnings denied;
- deterministic focused unit tests;
- an executable contract probe run twice with byte-identical output;
- explicit closed-authority and no-runtime-wiring assertions.

A passing EI-0A gate establishes only that the contract machinery is internally consistent and replayable. It does not count as EI-0F evidence and does not authorize EI-0C persistence, EI-0D learning updates, or EI-0G runtime observation.
