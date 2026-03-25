# Metacognitive Confidence Monitoring During Evidence Accumulation

## Objective

Build a computational architecture where confidence is computed online while
evidence is being accumulated, then fed back into the decision process before a
final choice is made. This replaces the standard sequential pattern:

1. accumulate evidence
2. commit to a decision
3. compute confidence after the fact

The working hypothesis is that a parallel confidence signal can improve
calibration, compute allocation, and error detection by changing the dynamics of
the decision itself.

## Core Architecture

The system is organized around three modules:

1. `EvidenceAccumulator`
   Integrates incoming evidence with drift, stochasticity, and a decision
   threshold.
2. `ConfidenceMonitor`
   Estimates confidence continuously from the current accumulator state,
   uncertainty estimate, and elapsed time.
3. `MetacognitiveController`
   Uses the current confidence estimate to modulate threshold and attention so
   low-confidence states gather more evidence and high-confidence states can
   commit earlier.

## Initial Computational Model

The first implementation in this repository intentionally stays simple:

- scalar evidence state instead of a full neural population
- time-stepped drift-diffusion style updates
- confidence derived from evidence magnitude relative to tracked uncertainty
- bounded threshold and attention modulation to keep the closed loop stable

The update order per step is:

1. estimate current confidence from the present state
2. derive a threshold and attention policy from that confidence
3. integrate the next evidence increment
4. update uncertainty with an exponential moving average
5. recompute confidence after the update
6. test whether the updated evidence crosses the current threshold

This gives a minimal executable model of "confidence during accumulation" rather
than "confidence after accumulation."

## Roadmap

## Phase 1: Foundation

- Formalize the confidence objective and stability constraints.
- Establish deterministic unit tests around the feedback loop.
- Benchmark against a no-feedback baseline inside the same simulation harness.

## Phase 2: Expanded Decision Dynamics

- Add alternative confidence estimators, including calibrated posterior
  approximations.
- Track meta-uncertainty separately from aleatoric noise.
- Support task-conditioned control policies instead of a fixed heuristic
  controller.

## Phase 3: Transformer Integration

- Replace scalar evidence with token-level hidden-state or logit summaries.
- Compute confidence during autoregressive decoding, not only after generation.
- Let confidence modulate sampling, early exit, or cache retention policies.

## Phase 4: Experimental Validation

- Compare online confidence to post-hoc confidence baselines.
- Measure calibration error, compute cost, and error-recovery behavior.
- Test whether the controller improves accuracy-compute tradeoffs on curated
  language tasks.

## Current Scaffold

The code now includes:

1. An online metacognitive loop and a sequential post-hoc confidence baseline
   for direct comparison.
2. A controller sweep harness that logs calibration and compute tradeoffs
   against that baseline for both constant signals and time-varying traces.
3. A transformer-facing adapter that maps logits or hidden states into
   confidence and controller signals, plus a simple adaptive decode scaffold
   that can stop early once confidence stabilizes.
