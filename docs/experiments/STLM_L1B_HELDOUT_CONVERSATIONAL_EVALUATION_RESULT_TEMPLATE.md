# STLM L1-B Result

## Frozen identifiers

- Experiment: `STLM_L1B_HELDOUT_CONVERSATIONAL_EVALUATION`
- Corpus: `stlm-l1b-heldout-v1`
- Evidence workflow run: `29982861230`
- Evaluated head: `c18c75f6e90307b9745d922b6983facf1cbecc28`
- Runtime authority: none
- Live text influence: none

## CI result

Recorded from the preserved `stlm-l1b-heldout-report.json` artifact produced by the frozen workflow.

- Gate passed: **true**
- Scenario count: **10**
- Total selections: **160**
- Independently verified selections: **160**
- Verification pass rate: **10,000 bps (100%)**
- Exact replay matches: **160**
- Replay pass rate: **10,000 bps (100%)**
- Neutral divergences: **160**
- Neutral divergence rate: **10,000 bps (100%)**
- Trace opening changes: **10 of 10 scenarios**
- Trace opening-change rate: **10,000 bps (100%)**
- Microstate surface changes: **10 of 10 scenarios**
- Microstate response rate: **10,000 bps (100%)**
- Fallback count: **0**
- Fallback rate: **0 bps**
- Legacy remediation lead hits: **0**
- Minimum unique complete surfaces per scenario: **3**
- Minimum unique openings per scenario: **3**
- Authority boundary closed: **true**
- Runtime influence: **none**
- Held-out corpus separate from L1-A unit fixture: **true**

## Interpretation

The selector passed every preregistered L1-B gate. Across the heterogeneous ten-scenario corpus, every selected response survived independent semantic verification and exact replay. The selector also responded to both recent-language pressure and conversational microstate changes in every scenario without using the neutral fallback or reverting to legacy remediation leads.

This result supports a separately preregistered L1-C shadow-observation proposal. It does **not** authorize returning improvised text from `Runtime::chat()`, HTTP responses, or any other live surface.

## Decision rule

A passing result permits a separately preregistered shadow-observation proposal. It does not permit runtime or HTTP response influence. A failing result freezes progression and must be recorded without relaxing the preregistered thresholds.
