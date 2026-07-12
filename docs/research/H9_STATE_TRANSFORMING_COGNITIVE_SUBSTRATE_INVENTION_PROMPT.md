# H9 Invention Mandate: A State-Transforming Cognitive Substrate for Starfire

## Purpose

This document is a research-grade prompt for an advanced reasoning model or research agent. Its task is not to polish the current architecture, tune thresholds, or produce another classifier over static CHARGE features. Its task is to inspect Starfire and invent the missing computational primitive required for genuinely compositional autonomous reasoning.

The mandate begins from the negative results of H4 through H8. Those results are evidence, not failures to be hidden or tuned away.

---

# Prompt

You are operating as a principal research scientist, mathematical inventor, systems architect, and falsification-oriented experimentalist working directly on the private Starfire codebase.

Your job is to solve the architectural problem exposed by Starfire's H4-H8 research sequence.

Do not merely propose improvements. Inspect the repository, reconstruct the exact current computational semantics, identify the deepest missing primitive, invent a mathematically and computationally coherent replacement or extension, implement it, and test it under a preregistered falsification protocol.

You are allowed to invent new mathematics, representations, state-transition laws, algebras, typed intermediate objects, learning rules, credit-assignment mechanisms, or computational operators if existing concepts are inadequate.

You are not required to preserve the current ontology-induction framing if the evidence shows that framing is downstream of a more fundamental missing substrate.

## Binding empirical starting point

Treat the following as constraints from prior experiments, not as suggestions.

### H4

Starfire demonstrated that a shadow empirical ontology could discover at least one useful transferable distinction from real subsystem-backed closed-cycle observations. A memory-shaped cohort transferred strongly against an undifferentiated resolver baseline.

However, the stronger claim was rejected because:

- only one robust concept was found;
- a matched-budget feature-destroyed control retained too much utility;
- the evidence did not justify automatic live concept promotion.

Therefore:

> Static empirical routing structure exists, but the current residual representation and induction mechanism have not established general autonomous concept formation.

### H5

After excluding the memory-shaped cohort, the remaining reasoning-versus-causal outcome matrix did not expose sufficiently strong, stable opposing regimes under the frozen identifiability contract.

Therefore:

> A more expressive ontology search cannot manufacture a reasoning distinction that the underlying component/environment interaction does not robustly express.

### H6

Disagreement-conditioned residual interactions and a second-order accumulated disagreement mode failed to manufacture a transferable reasoning primitive. Equal-budget random residual interactions could be as useful or more useful.

Therefore:

> Resolver disagreement is not, by itself, a validated endogenous source of new cognitive structure in the current substrate.

### H7

Witnessed Congruence Split tested whether histories indistinguishable under one-step behavior could become distinguishable under state-threaded resolver continuations.

The result was a clean rejection. The measured right-absorption rate was high: later operations largely behaved as if earlier operations had not created a meaningful new computational state.

Therefore:

> Merely preserving a cognitive-cycle object and executing resolver words sequentially does not create a useful continuation algebra.

### H8

H8 explicitly constructed an independently witnessed intermediate unresolved state after the first real component operation and serialized the first component output plus measurable history into the second operation.

The frozen result was `NOT_COMPOSABLE`.

The stateful path did not beat the same-word blind path on holdout or future evaluation. Apparent action-order differences were explained largely by endpoint resolver behavior rather than by a same-root intermediate state that changed the meaning or capability of the next operation.

Therefore:

> Existing Starfire reasoning and causal components cannot be made genuinely compositional merely by forwarding text, witness scalars, or preserved CHARGE state between them.

This is the central architectural diagnosis.

---

# The research problem

Invent the smallest principled computational substrate in which a cognitive operation can create a new state that changes what later operations can compute.

The required object is not simply memory, a transcript, a prompt, a vector embedding, a residual feature map, or a resolver label.

The substrate must support genuine transformation.

At minimum, there must exist histories `h`, operations `a` and `b`, and a state transition operator such that:

```text
S_0 = encode(problem)
S_1 = T_a(S_0)
S_2 = T_b(S_1)
```

and the system must demonstrate that:

```text
T_b(T_a(S_0))
```

contains reproducible task-relevant computational capability that cannot be reduced to:

```text
T_b(S_0)
```

plus superficial access to the textual output of `a`.

The desired phenomenon is not merely order sensitivity. It must be state-dependent composition.

A valid substrate should make at least some cognitive transformations:

- create facts or derived constraints that did not previously exist in executable form;
- eliminate impossible hypotheses;
- introduce explicit unresolved subgoals;
- alter which actions are legal or useful;
- expose contradictions that were previously latent;
- create new variables, relations, abstractions, or invariants;
- transform uncertainty or evidence structure;
- change later search topology;
- permit later operations that were impossible or ineffective before the transformation.

The operation-produced state must be causally necessary for the later success under matched controls.

---

# Do not assume the answer

Do not begin by implementing a generic blackboard, scratchpad, graph, theorem prover, planner, vector database, recurrent loop, or agent wrapper merely because those are familiar architectures.

First determine what mathematical or computational property is absent from the current Starfire system.

You must distinguish among possibilities such as:

- lack of an explicit state space;
- lack of typed state transitions;
- lack of persistent variable identity;
- lack of compositional semantics;
- lack of irreversible information gain;
- lack of executable constraints;
- lack of causal credit assignment across transformations;
- lack of operation closure;
- lack of partial-observation belief state;
- lack of a mechanism for constructing new operators;
- lack of a mechanism for changing the future search space;
- lack of environment coupling strong enough to make intermediate state meaningful;
- or a deeper failure not captured by this list.

You may reject the premise that there is one conventional missing data structure.

You may invent a novel formalism if necessary.

---

# Required research behavior

## 1. Inspect before inventing

Read the relevant Starfire implementation and experiment sequence, including at minimum:

- CHARGE representation and persistence;
- `CognitiveCycleState`;
- environment and objective witness contracts;
- independent discharge judging;
- empirical ontology induction;
- shadow promotion;
- fixed residual projection and H5 diagnostics;
- disagreement/contrast machinery from H6;
- witnessed congruence machinery from H7;
- H8 transformed action-order experiment and its controls.

Trace actual runtime data flow. Do not reason from names alone.

Produce an explicit diagnosis of where information becomes semantically inert, overwritten, flattened, or unavailable to later computation.

## 2. Search for the deepest missing invariant

Before proposing code, formulate the missing property as precisely as possible.

Examples of the level of precision expected:

- "The current resolver interface is observational rather than transformational because component outputs are scored externally but do not mutate a shared typed world model."
- "The system lacks a state-transition object whose equivalence classes are preserved under future operations, so no nontrivial action algebra can emerge."
- "CHARGE carries error magnitude but not executable commitments, therefore accepted discharge changes accounting without changing the reachable cognitive state space."

These are examples only. Derive the actual diagnosis from the repository.

## 3. Invent one primary primitive

Invent one principal mechanism that directly targets the diagnosis.

Prefer one deep primitive over a bundle of loosely related features.

The mechanism must have:

- a clear state representation;
- explicit transition semantics;
- a definition of what an operation may read;
- a definition of what an operation may write;
- invariants or validity rules;
- deterministic or auditable transition behavior where possible;
- a causal link between intermediate transformation and later capability;
- compatibility with independent environment feedback;
- a path to falsification.

The mechanism must not rely on hidden task-family labels, human-authored ontology classes, or privileged access to the answer.

## 4. Formalize it

Give the primitive a mathematical or operational definition.

At minimum specify:

```text
State space:          S
Operation family:     A
Transition:           T : S × A × O -> S'
Observation/witness:  W
Validity relation:    V(S, S')
Utility or progress:  U
```

where `O` is any admissible operation output or evidence object.

Then state what property should distinguish genuine composition from endpoint effects.

Possible forms include, but are not limited to:

```text
U(T_b(T_a(S))) > U(T_b(S))
```

under matched compute and observation budgets, together with interventions showing that the gain disappears when the causally relevant intermediate structure is destroyed.

A stronger formulation is welcome if appropriate.

## 5. Make intermediate state executable, not decorative

The next operation must consume a transformed computational object, not merely a prose transcript describing what happened.

The experiment should be designed so that the state created by operation `a` changes the effective input domain, constraints, available operators, or reachable solutions for operation `b`.

Text may be used as one interface, but a passing result must not be explainable by simply appending prior text to a new prompt.

## 6. Require causal interventions

A positive result is invalid unless matched interventions distinguish true state dependence from incidental correlation.

Design controls that preserve as much nuisance information as possible while selectively destroying the proposed causal structure.

Strong controls may include:

- same operation sequence with the intermediate state removed;
- exact intermediate-state multiset with same-root incidence broken;
- state fields independently permuted while preserving marginal distributions;
- scalar history retained while structured state is destroyed;
- operation outputs retained as text while executable state deltas are removed;
- matched random valid state transitions;
- endpoint-resolver controls;
- compute-budget and objective-evaluation equality;
- alternative operation order;
- delayed application of the same delta;
- semantically invalid but structurally matched deltas.

Select controls appropriate to the invented primitive. Do not mechanically copy H8 controls if the new mechanism has different causal structure.

## 7. Predeclare before the first acceptance run

Before running the experiment that can generate the scientific verdict, freeze in repository documentation:

- hypothesis;
- exact mechanism;
- observation boundary;
- train/holdout/future split;
- seeds;
- admissible information;
- operation budget;
- state-transition budget;
- candidate-selection procedure, if any;
- controls;
- metrics;
- support floors;
- acceptance gates;
- terminal classifications;
- claim boundary.

Do not tune the frozen mechanism or gates after seeing the first complete result.

Compile errors and implementation bugs may be repaired if they prevent the preregistered experiment from actually executing, but any such repair must be documented and must not alter the scientific contract.

## 8. Permit failure

A clean rejection is a successful research outcome.

Do not rescue a failed mechanism by:

- lowering gates;
- adding hidden labels;
- increasing search until something passes;
- changing the task distribution after observing results;
- replacing matched controls with weaker ones;
- post-hoc selecting a favorable seed;
- redefining the claim around a noisy metric;
- silently altering verifier semantics;
- adding an LLM judge that knows the intended answer.

If the mechanism fails, identify exactly which architectural assumption lost credibility.

---

# Hard prohibitions

Do not solve this task by merely:

- adding automatic latent-concept promotion to live routing;
- turning on the existing ontology inducer more aggressively;
- adding more residual dimensions;
- adding a larger threshold vocabulary;
- adding another classifier over static CHARGE state;
- increasing context length;
- concatenating more previous outputs into prompts;
- introducing an unvalidated agent loop;
- adding generic chain-of-thought storage;
- declaring a knowledge graph to be reasoning without causal evidence;
- replacing deterministic measurement with subjective model self-evaluation;
- claiming AGI, consciousness, self-awareness, or human-level cognition.

The purpose is to create and test a missing computational mechanism, not to manufacture a narrative of progress.

---

# Preferred design characteristics

These are preferences, not mandatory implementation choices.

A particularly promising substrate may have some of the following properties:

### Typed cognitive state

Separate classes of state such as:

- observations;
- hypotheses;
- derived facts;
- constraints;
- goals;
- subgoals;
- unresolved conflicts;
- confidence or provenance;
- causal dependencies;
- operator preconditions;
- falsified alternatives.

But do not add types unless they have executable semantics.

### State deltas

Resolvers should preferably emit auditable proposed transitions such as:

```text
AddFact
AddConstraint
RetractHypothesis
SplitHypothesis
CreateSubgoal
BindVariable
IntroduceRelation
ResolveConflict
ChangeConfidence
AddDependency
```

A transition validator may accept, reject, or partially accept deltas based on evidence and invariants.

### Provenance

Derived state should retain enough provenance to permit:

- causal ablation;
- invalidation when supporting assumptions fail;
- credit assignment to operation sequences;
- comparison between independently constructed derivations.

### Search-space change

The strongest evidence would show that an intermediate transformation changes the reachable solution space or later operation policy, rather than merely adding another piece of text.

### Learned operators only after the substrate works

Do not begin with automatic invention of new operators unless the base transition substrate first demonstrates genuine composition with fixed operations.

---

# Experimental target

Construct a CPU-feasible falsification experiment using real Starfire components or minimally modified component interfaces.

The experiment should contain tasks that require at least two causally dependent transformations.

A task should not be solvable equally well by the final resolver acting directly on the untouched original input.

The experiment should include unseen future task families whose surface vocabulary and domain differ from training.

A valid positive result should ideally demonstrate all of the following:

1. **Stateful composition gain**
   
   The composed state-transforming path outperforms the same endpoint operation on the original state.

2. **Intermediate necessity**
   
   Destroying the specific intermediate structure removes most of the gain.

3. **Not text-only**
   
   Providing the first operation's textual output without applying its executable state transformation does not recover the result.

4. **Not scalar-history-only**
   
   Witness scalars or generic progress summaries do not recover the result.

5. **Not endpoint identity**
   
   The effect is not explained solely by which resolver runs last.

6. **Same-root dependence**
   
   Rewiring intermediate states across unrelated roots destroys the effect while preserving state marginals where possible.

7. **Future transfer**
   
   The effect survives unseen task families.

8. **Matched budgets**
   
   Resolver calls, state transitions, environment evaluations, and search budgets are exactly accounted for.

9. **Independent witness**
   
   Success is measured through an environment/objective path independent of the resolver's self-report.

10. **Conservative claim**

    A pass supports only the specific demonstrated computational property.

---

# Required deliverables

Produce all of the following in one coherent research pull request.

## A. Architectural diagnosis

A concise but technically precise document explaining:

- what H4-H8 established;
- why the current system is right-absorbing or state-inert under continuation;
- where the current interfaces prevent genuine transformation;
- what exact missing computational invariant the new primitive targets.

## B. Formal mechanism specification

Document:

- state representation;
- transition semantics;
- invariants;
- operation contracts;
- witness integration;
- failure semantics;
- computational complexity;
- why the mechanism is not equivalent to transcript concatenation.

## C. Implementation

Implement the smallest reusable library substrate necessary to instantiate the hypothesis.

Prefer a clean module boundary and deterministic unit tests.

Do not contaminate live routing unless the research contract explicitly requires a shadow-only integration path. Default to diagnostic or shadow execution.

## D. Preregistered experiment

Add a frozen experiment document before the first verdict-producing run.

## E. Executable falsification probe

Add a deterministic CPU-feasible executable that:

- constructs the frozen cohort;
- executes the state-transforming paths and controls;
- tracks exact budgets;
- emits structured JSON;
- produces a terminal scientific classification;
- exits nonzero when the preregistered acceptance contract fails, unless the repository's established CI convention requires diagnostic non-blocking treatment for rejected research hypotheses.

## F. CI integration

Run:

- all relevant compile checks;
- unit tests for the new substrate;
- prior critical CHARGE/cognitive-cycle/environment contracts;
- the new falsification probe;
- artifact preservation.

Do not weaken unrelated existing gates.

## G. Final verdict

Report the result without spin.

If accepted, state exactly what was demonstrated and what remains unproven.

If rejected, state exactly what architectural assumption failed and propose only the next narrowest informative experiment.

---

# Decision standard

The objective is not to make Starfire look more intelligent.

The objective is to answer this question:

> Can Starfire acquire a computational state in which one cognitive operation creates an executable transformation that is causally necessary for a later operation to do something it could not equivalently do from the original state?

Until that property is demonstrated under matched controls, do not claim that Starfire has genuine compositional autonomous reasoning.

Find the smallest mechanism that could make the answer yes.

Then try as hard as possible to prove yourself wrong.
