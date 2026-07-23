# EI-0F-R1B Frozen Terminal Remediation

> **Status:** frozen specification, no terminal result executed  
> **Preregistration ID:** `ei-0f-remediation-v1`  
> **Freeze base:** `b9b5f70d7da98088d2546c0ac5a730c9854326ab`  
> **Canonical manifest SHA-256:** `c4028c7acd080a4809a15b73556120d53f8a159df440150c5e58f6f96ee99561`  
> **Parent EI-0F result:** immutable `FAIL` at `2e74746eeb524a50d41749d964e59838b7fbd919`

## Purpose

This package freezes one separately identified terminal remediation after the original EI-0F runner failed closed before arm evaluation. It does not revise or erase the original result.

## Sole semantic change

The episode-level proposal-digest reference converts underscores in the existing update identifier to hyphens. Update identifiers, arm names, validators, tasks, partitions, seeds, budgets, evaluators, thresholds, classifier semantics, report fields, safety requirements, and authority remain unchanged.

## Evidence chain

- remediation preflight merge: `b9b5f70d7da98088d2546c0ac5a730c9854326ab`;
- remediation preflight artifact: `sha256:680509ea0932e3545fa307506ae1cd694e37cd429eb38d0058b681601f060a16`;
- repaired runner Git blob: `4df981c93b21176abacc0fec5c2a991dc26b1510`.

## Boundary

This stage is compile-only. It adds no runtime wiring, persistence, response or routing authority, ontology promotion, tools, autonomous action, or live learning. EI-0G remains unauthorized.
