# Closed Cognitive Cycle AGI Plan

**Status:** research implementation plan  
**Date:** 2026-07-08  
**Primary host:** Starfire  
**Falsification host:** Mind  

## Thesis

Starfire should not become a larger collection of named cognitive modules. The next research program is to close the loop between observation, world-model prediction, persistent unresolved state, cognitive routing, action, independently judged outcome, learning, and replay.

The core hypothesis is:

> General capability can emerge from a system that persistently represents structured unresolved computation, routes it to empirically useful cognitive operators, revises its world model from externally observed outcomes, and compiles recurrent successful resolution paths into transferable skills.

CHARGE is the candidate pressure and routing primitive. It is not assumed to be sufficient for intelligence. The program below is designed so CHARGE, ontology induction, operator compilation, or the entire closed-loop thesis can fail cleanly.

## Repository roles

### Starfire: the organism

Starfire is the production research host for the integrated cognitive cycle. It already contains persistent memory, symbolic reasoning, metacognition, Quanot, a world-model shell, prediction engines, curiosity, causal machinery, goals, and CHARGE.

Starfire should receive only primitives that have explicit contracts and measurable outcomes.

### Mind: the crucible

Mind is the falsification environment. It should receive small deterministic probes, controls, negative controls, ablations, fixed compute budgets, and predeclared pass/fail gates.

Mind should try to kill each Starfire idea before Starfire depends on it.

## Architectural target

```text
OBSERVE
  |
  v
UPDATE / RECONCILE WORLD MODEL
  |
  v
PREDICT
  |
  v
COMPARE PREDICTION WITH OBSERVATION
  |
  v
EMIT PERSISTENT TYPED CHARGE
  |
  v
ROUTE CHARGE TO CANDIDATE COGNITIVE OPERATORS
  |
  v
PROPOSE HYPOTHESIS / QUERY / ACTION / MODEL REVISION
  |
  v
EXECUTE OR TEST
  |
  v
OBSERVE OUTCOME
  |
  v
INDEPENDENT DISCHARGE JUDGE
  |
  +--> UPDATE WORLD MODEL
  +--> UPDATE ROUTER PROFILES
  +--> UPDATE OPERATOR STATISTICS
  +--> STORE EPISODE / COUNTEREXAMPLE
  +--> REPLAY CANDIDATE MODEL
  |
  v
REPEAT UNTIL RESOLVED, PROVEN UNREACHABLE, OR BUDGET EXHAUSTED
```

Chat is one environment. It must stop being the definition of the cognitive architecture.

## Non-negotiable invariants

### 1. A resolver never judges its own success

A component may propose a `Resolution`, but its claimed discharge is only a request. An independent `DischargeJudge` measures outcome evidence and caps accepted discharge.

A resolver saying "I solved it" is not evidence.

### 2. External or independently replayed outcome beats internal confidence

Examples:

- Prediction charge: held-out prediction error actually falls.
- Goal charge: measured environment state moves toward the target.
- Epistemic charge: uncertainty falls because new evidence was acquired.
- Contradiction charge: incompatible propositions are rejected, repaired, or explicitly scoped with evidence.
- Ontology charge: a revised representation explains prior failures and generalizes on replay or held-out episodes.

### 3. Exact provenance and reusable routing identity remain separate

Exact scope remains attached to CHARGE for accounting and diagnosis. Routing profiles use a coarser class when transfer across exact instances is required.

### 4. No component role labels are supplied to the router

The router may observe signatures, attempts, cost, and independently judged discharge. It may not be told "causal handles prediction residuals" or "memory handles epistemic gaps."

### 5. Final gates are declared before final evaluation

Exploratory runs may tune experiment design. The final probe must freeze seeds, environment families, baselines, budgets, metrics, and thresholds before its acceptance run.

### 6. No direct self-edit-to-main loop

Cognitive self-improvement must generate candidate variants, evaluate them in isolated branches or sandboxes, compare them on held-out task families and regression suites, and retain an archive. Passing unit tests alone is never sufficient evidence of cognitive improvement.

## Foundational charge classes

The current CHARGE enum already covers several of these concepts. The closed-cycle program should make the following semantic roles explicit through emitters and judges rather than immediately growing the enum without evidence.

### Prediction residual

The world model predicted one state or outcome and observed another.

```text
predicted_state != observed_state
```

The residual should identify the modeled transition or belief whose predictive behavior failed.

### Ontology gap

The current representational vocabulary cannot express a recurring structure required to explain observations.

This is model-class failure, not merely parameter error.

Example:

```text
Initial ontology:
  Door(state = open | closed)

Recurring observations:
  OPEN action sometimes has no effect.
  Failures correlate with another object class.

Candidate latent structure:
  Door.locked
  Key.compatible_with(Door)
  Unlock(Key, Door)
```

Ontology gaps should be inferred from recurring structured failure. They should not be emitted merely because a topic is novel.

### Goal gap

The current measured state differs from a desired state.

```text
current_state != desired_state
```

Goal charge persists until the objective is reached, explicitly abandoned under policy, or demonstrated unreachable under the current action/model budget.

### Contradiction

Two active propositions, transition rules, or model commitments cannot simultaneously stand under the same scope and conditions.

Contradiction charge must point to the incompatible commitments. It is not an affective conflict scalar.

## Cognitive operators

The atomic unit of Starfire cognition should move from response handlers toward operators with explicit applicability, cost, predicted effects, and outcome history.

Target contract:

```rust
pub struct CognitiveOperator {
    pub id: OperatorId,
    pub preconditions: Vec<Predicate>,
    pub applicable_charge: Vec<ChargePattern>,
    pub predicted_effects: Vec<Effect>,
    pub estimated_cost: ComputeCost,
    pub evidence: Vec<EvidenceId>,
    pub success_count: u64,
    pub failure_count: u64,
}
```

Candidate primitive operators include:

- recall analogous episode
- search persistent memory
- forward chain
- backward chain
- generate counterexample
- propose latent variable
- split entity or concept
- merge concepts
- induce transition rule
- test causal intervention
- ask an information query
- replay a hypothesis
- compare competing models
- execute an environment action

The initial library can be hand-authored. The research target is the induction and compilation of new operators from successful resolution paths.

## World-model requirement

The existing Starfire `WorldModel` is a useful entity/relation representation shell, but the closed cycle requires explicit transition hypotheses:

```text
STATE(t) + ACTION(t) -> predicted STATE(t+1)
```

Target representation:

```rust
pub struct TransitionHypothesis {
    pub preconditions: Vec<Predicate>,
    pub action_pattern: ActionPattern,
    pub effects: Vec<Effect>,
    pub invariants: Vec<Predicate>,
    pub confidence: f64,
    pub supporting_episodes: Vec<EpisodeId>,
    pub counterexamples: Vec<EpisodeId>,
}
```

A transition hypothesis is provisional until replayed against prior episodes and tested on held-out transitions.

## Concept formation through unresolved pressure

The key H4 research hypothesis is:

> Persistent, recurring, structured unresolved charge can create useful abstraction pressure: when the existing ontology and operator library repeatedly fail to discharge the same residual structure, proposing a latent concept can improve prediction and transfer.

Candidate induction loop:

```text
recurring charge family
  |
  v
existing resolvers fail
  |
  v
cluster residual/context structure
  |
  v
propose latent variable or relation
  |
  v
re-encode affected episodes
  |
  v
re-induce transition hypotheses
  |
  v
replay on historical episodes
  |
  v
held-out prediction test
  |
  +--> no improvement: reject candidate
  |
  +--> improvement: provisional concept
                         |
                         v
                 transfer / recurrence test
                         |
                         v
                    promote concept
```

A new concept is not promoted because it sounds coherent. It must reduce independently measured unresolved error and survive replay or transfer.

## Environment interface

The cognitive core needs a domain-independent interaction boundary.

Initial contract:

```rust
pub trait Environment {
    type Action;
    type Observation;

    fn reset(&mut self, seed: u64) -> Self::Observation;
    fn available_actions(&self) -> Vec<Self::Action>;
    fn act(&mut self, action: &Self::Action) -> Step<Self::Observation>;
    fn objective_feedback(&self) -> ObjectiveFeedback;
}
```

Initial environment families:

1. `HiddenRuleEnvironment` — procedural symbolic worlds with unknown transition dynamics.
2. `FileSystemEnvironment` — bounded scratch workspace with explicit tasks and objective checks.
3. `CodeEnvironment` — isolated repository/task harness with tests as partial outcome evidence.
4. `ResearchEnvironment` — evidence acquisition and claim verification tasks.
5. `ChatEnvironment` — human interaction as one observation/action channel, not the architecture root.

## Phase 0: contracts and accounting

**Purpose:** establish interfaces before integration.

Deliverables:

- `Environment`, `Step`, and `ObjectiveFeedback` contracts.
- `OutcomeWitness` for before/after independently measured metrics.
- `DischargeJudge` contract.
- A deterministic relative-improvement judge that cannot accept more than the resolver requested or the incoming charge contains.
- Unit tests for directionality, clamping, no-evidence behavior, and overclaim rejection.
- No `Runtime::chat()` integration.

Exit gate:

- Contracts compile in the Star library.
- Deterministic unit tests pass.
- A resolver cannot create accepted discharge by only increasing its requested value.

## Phase 1: closed-cycle probe

Build a tiny `HiddenRuleEnvironment` with generated worlds.

Objects:

- shapes
- gates
- tokens
- switches

Actions:

- move
- touch
- pick up
- drop

Latent rules are sampled from a grammar. Examples:

- touching triangles flips square states
- a gate opens only when an even number of switches are active
- carrying a token reverses TOUCH semantics
- one object property gates whether another action has an effect

The agent receives observations and available actions but not the latent rules.

Baselines:

- random action selection
- deterministic systematic exploration
- fixed operator schedule
- no-CHARGE ablation
- scrambled charge-routing signatures
- oracle model/operator policy for upper-bound context

Primary metrics:

- objective solve rate
- action cost to solve
- held-out transition prediction error
- unresolved charge at termination
- charge lifetime mean/P95
- resolver/operator specialization entropy
- routing stability
- charge accounting violations

## Phase 2: H4 ontology-pressure test

Mind owns the initial deterministic proof-of-mechanism.

Construct world families where the initial schema is intentionally insufficient. A hidden binary or categorical variable must be introduced to predict outcomes.

Compare:

1. persistent structured charge + latent-variable proposal
2. scalar prediction error + latent-variable proposal
3. novelty-only proposal
4. periodic random proposal
5. no ontology expansion
6. oracle ontology

Predeclare final gates after exploratory calibration.

The final experiment must include:

- held-out surface names
- held-out exact world instances
- at least one held-out latent rule family
- equal proposal/evaluation budgets
- replay against prior episodes
- a complexity penalty so the system cannot win by adding arbitrary variables

## Phase 3: resolution-path compilation

Track successful operator sequences by generalized charge signature and context structure.

Candidate skill:

```text
PROPOSE_LATENT_VARIABLE
  -> RE-ENCODE_EPISODES
  -> INDUCE_TRANSITION_RULE
  -> REPLAY_HYPOTHESIS
```

Compile a recurrent path only when it:

- recurs across independent episodes
- beats the uncompiled operator schedule at equal compute
- transfers to held-out exact scopes
- does not regress prior task families

Compiled skills are operators and therefore remain subject to independent discharge judgment.

## Phase 4: operator induction

The system proposes a new operator schema from recurring successful transformations.

Candidate operators are evaluated in an archive, not immediately promoted into the live core.

Promotion requires:

- deterministic replay where applicable
- held-out family improvement
- compute accounting
- no charge-accounting violations
- regression suite pass
- outcome evidence stronger than self-reported confidence

## Phase 5: multi-environment generality

Use one cognitive-cycle core across hidden-rule worlds, bounded filesystem tasks, code tasks, research tasks, and chat.

The goal is not benchmark maximization through per-domain handlers. The key question is whether the same world-model reconciliation, CHARGE routing, operator selection, outcome judging, replay, and skill compilation machinery transfers.

## Phase 6: Darwinian cognitive variants

Create candidate Starfire variants by mutating bounded cognitive mechanisms:

- charge router policy
- memory retrieval policy
- transition induction policy
- ontology candidate generator
- operator scheduler
- exploration policy

Do not mutate arbitrary production code in the first generation.

Maintain an archive rather than one lineage. Compare candidates on held-out environment families, regressions, compute cost, epistemic integrity, and safety checks.

A candidate's unit tests are a minimum validity condition, not a fitness score.

## Research questions and experiment queue

### H4 — Abstraction pressure

Persistent structured charge induces useful latent-variable formation more efficiently than scalar error, novelty-only, random proposal, or no ontology growth.

### H5 — Skill compilation

Recurring successful resolution paths can be compiled into operators that improve discharge per compute on held-out scopes.

### H6 — Structural transfer

Compiled operators transfer under surface renaming and to unseen instances sharing causal structure.

### H7 — Dynamic specialization

CHARGE routing changes operator preference when environment dynamics change, outperforming a fixed specialist schedule.

### H8 — Model-class failure

Ontology-gap detection identifies representational insufficiency better than scalar prediction residual alone.

### H9 — Efficient exploration

Charge persistence drives lower-cost systematic exploration than novelty-only exploration under matched action budgets.

### H10 — Spontaneous specialization

Initially undifferentiated operator variants develop stable, task-dependent functional specialization through empirical discharge profiles.

## Explicit rejection conditions

Redesign or reject the program if repeated controlled experiments show any of the following:

- CHARGE magnitude does not predict resolution progress or useful compute allocation.
- Independent judging collapses to the same answer as resolver self-report.
- Fixed scheduling matches learned routing at equal compute across held-out families.
- Ontology induction wins only by uncontrolled representational growth.
- New concepts fail surface-renaming or held-out structural transfer.
- Compiled operator paths overfit exact scopes.
- Persistent charge causes compute hoarding or unbounded loops.
- No-CHARGE ablations match the closed cycle after budget normalization.
- Multi-environment performance requires domain-specific response-handler expansion rather than shared cognitive machinery.

## Immediate implementation sequence

1. Land Phase 0 contracts in Starfire.
2. Land the H4 experimental protocol and deterministic scaffold in Mind.
3. Build `HiddenRuleEnvironment` in Starfire without wiring it into chat.
4. Implement transition hypotheses and prediction reconciliation.
5. Add charge emitters for transition residual and model-class failure candidates.
6. Route existing and initial candidate operators through CHARGE.
7. Add independently measured outcome witnesses and judges.
8. Run the closed-cycle baselines.
9. Freeze H4 gates in Mind after exploratory calibration.
10. Only after H4 survives, implement latent-concept promotion in Starfire.

## Definition of credible progress

The project should not claim AGI because Starfire has continuity, metacognition, autonomous thoughts, a consciousness proxy, or a self-improvement loop.

Credible progress is demonstrated when the same integrated core repeatedly does the following in unfamiliar held-out environments:

```text
recognizes model failure
  -> preserves unresolved structure
  -> explores selectively
  -> forms a falsifiable hypothesis
  -> changes its representation when the model class is insufficient
  -> learns a transition or operator
  -> verifies against external outcomes
  -> retains the successful mechanism
  -> transfers it to a structurally related task
```

That is the research program.
