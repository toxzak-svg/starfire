# EI-0C Append-Only Episode Ledger

> **Stage:** EI-0C  
> **Status:** implementation contract  
> **Authority:** offline ledger infrastructure only  
> **Parent:** issue #185 and EI tracker #149  
> **Implementation:** PR #187  
> **Feature:** `emerging-intelligence-ledger`

## Purpose

EI-0C preserves validated EI-0A sealed cognitive episodes in an append-only, hash-chained ledger and proves that the same ordered history can be reconstructed from canonical bytes in a fresh state.

This stage creates no live memory or learning authority. It does not write to Starfire's SQLite database, connect to `Runtime::chat()`, accept or apply learning updates, or influence responses, routing, tools, beliefs, ontology, or autonomous actions.

## Ledger contract

The ledger must provide:

- an explicit versioned schema and domain-separated digests;
- strictly increasing sequence numbers beginning at one;
- an explicit genesis link and previous-entry digest chaining;
- canonical EI-0A sealed episode bytes in every entry;
- duplicate episode-identifier rejection;
- canonical whole-ledger serialization and exact decoding;
- root, terminal, entry, episode, order, chain, count, and schema validation;
- corruption, truncation, reordering, stale-version, and non-canonical encoding rejection;
- deterministic fresh-state replay and a bounded replay summary;
- a closed authority snapshot.

## Claim boundary

A passing EI-0C probe supports only this claim:

> Starfire contains offline append-only episode-ledger infrastructure that can detect bounded corruption and replay canonical EI-0A episodes exactly from fresh state.

It does not support cumulative improvement, transfer learning, safe live learning, EI-0 PASS, emerging intelligence, consciousness, or runtime persistence authority.

## Verification target

The permanent EI-0C gate must prove scoped formatting and lint cleanliness, full feature compilation, focused corruption and replay tests, and two byte-identical executions of the bounded probe.
