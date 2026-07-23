# EI-0E Terminal Preregistration

> **Stage:** EI-0E  
> **Status:** draft contract, not yet frozen  
> **Authority:** experiment specification only  
> **Parent:** EI-0 tracker #149

## Purpose

EI-0E will freeze the exact terminal EI-0 experiment before any qualifying EI-0F run is inspected. It must bind the hypotheses, source commit, fixture manifest, seeds, control arms, budgets, update lattice, evaluators, thresholds, report schema, failure rules, and claim boundary into a canonical digest.

No result may count as the terminal EI-0 experiment unless it uses the exact preregistered source and manifest or is explicitly recorded as a failed or superseded run under a new identifier.

## Required frozen fields

- exact source commit and Cargo lock digest;
- EI-0A/B/C/D schema and implementation digests;
- complete EI-0B fixture manifest and partition seeds;
- five matched arms: learning, no-update, memory-disabled, random-update, fixed-policy;
- identical action, evidence, episode, and update-opportunity budgets;
- fixed EI-0D update slots, per-update bounds, cumulative budget, admissibility evaluator, and safety evaluator;
- primary and secondary hypotheses;
- arm-level and task-family-level scoring rules;
- transfer, regression, rollback, causal-attribution, and replay thresholds;
- deterministic run command and report schema;
- terminal PASS/FAIL classifier;
- explicit missing-data, crash, corruption, and nondeterminism rules;
- authority and claim boundaries.

## Claim boundary

EI-0E freezes an experiment. It produces no evidence that Starfire improves from experience and grants no runtime, persistence, learning, response, routing, ontology, tool, or autonomous-action authority.
