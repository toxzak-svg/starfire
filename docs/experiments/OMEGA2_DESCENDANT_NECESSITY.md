# Ω2 — Descendant Necessity

Status: preregistered before implementation and before any verdict-producing run.

## Lineage

Ω1 established one-step endogenous state-space genesis under a frozen symbolic sequence-history regime:

```text
L0
  -> witnessed representational alias D1
  -> synthesize Δ1 from raw history structure
  -> independently validate Δ1
  -> admit Δ1
  -> L1 = L0 + Δ1
  -> later representation-bound prediction becomes possible
```

Ω2 asks a narrower and harder question:

> Can an admitted endogenous refinement create a genuinely new hypothesis-language terminal that is required to express, discover, validate, and use a second descendant refinement?

The target claim is not merely that two independently available features are useful together. The second executable program must be outside the frozen one-step Ω1 raw-program language and become enumerable only because `L1` contains an admitted ancestor refinement.

## Primary descendant-necessity structure

For each root:

```text
L0
  -> D1
  -> synthesize / validate / admit Δ1
  -> L1
  -> descendant candidate language H(L1) becomes non-empty
  -> D2 behavioral partition is expressible in H(L1)
  -> synthesize / validate / admit Δ2
  -> L2
  -> later representation-bound prediction succeeds
```

The decisive ablations are:

```text
L0 has no admitted refinement
  -> H(L0) contains zero descendant programs
  -> Δ2 cannot be proposed

remove exact Δ1 and replace it with a different valid refinement
  -> candidate count and compute remain matched
  -> the previously proposed Δ2 no longer validates
  -> target descendant partition is not recoverable
```

## Frozen raw fixture

The cohort is:

```text
16 training roots
8 holdout roots
32 future roots
4 future domain vocabularies
7 total domain vocabularies
8 roots per family
```

Each root has six raw history atoms arranged as three independent ordered pairs:

```text
(A, B) -> latent order bit p
(C, D) -> latent order bit q
(E, F) -> latent order bit r
```

The eight discovery histories exhaust the complete Boolean cube:

```text
(p, q, r) ∈ {0,1}^3
```

Discovery histories use three fixed pair blocks:

```text
AB-block | CD-block | EF-block
```

Within each block, pair order realizes its Boolean bit.

Every history therefore has:

```text
same six raw atoms
same atom counts
same base order-blind multiset key
```

The eight withheld transfer histories exhaust the same Boolean cube but permute the pair blocks:

```text
EF-block | AB-block | CD-block
```

The useful within-pair relations therefore transfer while discovery-only absolute positions and cross-block coincidences do not.

The six raw atoms are renamed independently in each of seven domain vocabularies. The hidden useful pair is never supplied to synthesis, validation, candidate enumeration, downstream prediction, or objective checking.

## Stage 1: ancestor refinement Δ1

Stage-1 outcome is determined by the first pair-order bit:

```text
Y1 = p
```

The stage-1 intervention is constant within the root.

Under `L0`, all eight histories share the same executable key and the 4/4 outcome split yields exactly:

```text
16 alias defects
```

Ω1's unchanged raw synthesis grammar is used:

```text
metric := FirstIndex(atom) | LastIndex(atom) | Count(atom)
raw_program := metric_a < metric_b | metric_a == metric_b
```

For six atoms:

```text
18 metrics
459 raw programs
4 unique behavioral partitions under the frozen cube
3,672 program/history executions per complete search
28 history-pair checks per complete defect scan
```

The target stage-1 partition repairs all 16 defects. The strongest non-target partition repairs 8, yielding the frozen structural margin:

```text
winner repaired defects = 16
runner-up repaired defects = 8
winner margin = 8
partition support min = 4
```

The unchanged Ω1 certificate gates apply.

`Δ1` is whatever canonical executable representative the Ω1 search independently selects for the winning behavioral partition. The experiment does not assume a specific syntax string or polarity.

## Stage 2 target behavioral partition

Stage-2 outcome is the parity/equality relation between the first two hidden pair-order bits:

```text
Y2 = (p == q)
```

The stage-2 intervention is constant within the root and distinct from the stage-1 intervention.

Across the exhaustive cube:

```text
4 histories have Y2 = 0
4 histories have Y2 = 1
16 opposite-outcome history pairs exist
```

### L0 raw-program expressibility audit

Before any descendant search, Ω2 exhaustively evaluates every one of the same 459 Ω1 raw programs against the stage-2 behavioral partition.

Because fixed pair blocks make every non-constant raw-program partition depend on at most one of `p`, `q`, or `r`, no single Ω1 raw program can express `p == q`.

The frozen expected audit result is:

```text
raw candidate programs = 459
raw unique partitions = 4
stage-2 opposite-outcome pairs = 16
best raw repaired pairs = 8
complete raw repair exists = false
```

This audit is recomputed independently and is a mandatory success gate.

A PASS therefore cannot be explained by simply rerunning Ω1 on the stage-2 labels and finding an already available raw predicate.

## Frozen descendant hypothesis language

Ω2 introduces a bounded generic descendant grammar whose terminals are generated from the **current executable state language**:

```text
ancestor_bit := output of one admitted StateLanguage refinement
raw_bit      := output of one enumerated Ω1 raw_program

descendant_program :=
      ancestor_bit == raw_bit
    | ancestor_bit != raw_bit
```

The grammar is developer-supplied and frozen. The available `ancestor_bit` terminals are not developer-supplied: they are generated only from actually admitted refinements in the current root-bound `StateLanguage`.

Therefore:

```text
H(L0): 0 admitted ancestor bits -> 0 descendant programs
H(L1): 1 admitted ancestor bit  -> 2 × 459 = 918 descendant programs
```

No placeholder, synthetic, endpoint-computed, text-only, scalar-only, foreign, or rejected refinement may create an `ancestor_bit` terminal.

## Descendant program semantics

A descendant program binds to the exact admitted ancestor proof identity and executes:

```text
ancestor_value = execute(the admitted ancestor refinement on raw history)
raw_value      = execute(the candidate raw program on raw history)

result = ancestor_value == raw_value
      or ancestor_value != raw_value
```

The candidate program cannot inspect:

```text
hidden p/q/r labels
family label
split label
stage-2 target outcome during transfer
future outcome
verifier target
developer-supplied useful atom pair
```

The proposer receives only:

```text
current root-bound StateLanguage
stage-2 witnessed discovery histories
raw history events
stage-2 intervention/outcome witnesses
```

## Frozen descendant partition search

The proposer:

1. verifies that the current `StateLanguage` contains at least one admitted refinement;
2. derives the six-atom raw vocabulary from stage-2 discovery histories;
3. enumerates all 459 Ω1 raw programs;
4. pairs every admitted ancestor bit with every raw program under `==` and `!=`;
5. executes all descendant candidates over all eight discovery histories;
6. canonicalizes binary partitions up to Boolean complement;
7. retains the canonical executable representative for each unique partition;
8. ranks partitions by separation of all 16 independently witnessed opposite-outcome pairs;
9. breaks score ties by minimum partition support, then canonical program string.

With exactly one admitted ancestor:

```text
918 descendant candidates
7,344 descendant program/history executions
4 unique descendant partitions
16 target opposite-outcome pairs
winner repaired pairs = 16
runner-up repaired pairs = 8
winner margin = 8
partition support min = 4
```

The winning partition is the stage-2 behavioral partition up to Boolean complement. Its syntax/polarity is not supplied.

## Independent validation and certificate binding

The validator independently recomputes:

1. the current root-bound ancestor-language canonical signature;
2. admitted ancestor proof identities;
3. stage-2 problem digest;
4. raw vocabulary;
5. all 459 raw programs;
6. all 918 descendant candidates;
7. all 7,344 descendant candidate/history executions;
8. canonical partition grouping;
9. all 16 opposite-outcome pairs;
10. complete partition ranking;
11. the unique winning behavioral partition;
12. every proof field.

A validated descendant certificate is bound to:

```text
root_id
ancestor_language_signature
ancestor_proof_id
stage2_problem_digest
descendant_program
```

Only an opaque validated descendant certificate may add `Δ2` to executable state.

If the exact ancestor language is removed or replaced before validation/admission, the certificate is invalid.

## Layered executable state

Ω2 wraps the accepted Ω1 `StateLanguage` without altering Ω1 semantics.

```text
L1 key = Ω1 StateKey(base multiset + admitted Δ1 bits)
L2 key = L1 key + admitted descendant refinement bits
```

The downstream predictor receives only the current layered executable key and witnessed stage-2 discovery outcomes attached to those keys.

It cannot inspect:

```text
raw histories
Δ1 proof text
Δ2 proof text
program syntax
scores
hidden p/q/r bits
family labels
split labels
```

If a key maps to conflicting stage-2 outcomes, the predictor abstains.

Under the frozen cube:

```text
L0: conflicting outcomes -> abstain
L1 with correct Δ1 only: conflicting outcomes within each ancestor key -> abstain
L1 with wrong valid ancestor only: conflicting outcomes -> abstain
L2 with correct descendant Δ2: unique outcome per key -> 8/8 predictions
```

## Frozen controls

Every root executes ten paths from fresh state and each path is replayed exactly twice.

### 1. `stateful_descendant_chain`

```text
synthesize/validate/admit correct Δ1
synthesize/validate/admit Δ2 under exact L1
predict under L2
```

Expected: `8/8` transfer predictions.

### 2. `l0_raw_search`

Run the complete 459-program L0 raw expressibility search against stage-2 behavior. Independent validation must fail complete-repair gates. No Δ1 or Δ2 admission.

Expected: `0/8` predictions.

### 3. `l0_descendant_no_ancestor`

Attempt descendant synthesis from a fresh `L0`.

Expected:

```text
admitted ancestor terminals = 0
descendant candidates = 0
proposal rejected as NoAncestorRefinement
0/8 predictions
```

### 4. `delta1_endpoint_only`

Synthesize and validate correct Δ1 and execute its program as endpoint information, but do not admit it to `StateLanguage`.

Expected:

```text
H(current language) descendant candidates = 0
0/8 predictions
```

### 5. `wrong_valid_ancestor`

Independently synthesize, validate, and admit a same-root Ω1 refinement for the third pair-order bit `r` instead of target ancestor partition `p`.

This creates one genuine admitted ancestor terminal and therefore the full matched descendant search:

```text
918 candidates
7,344 candidate/history executions
```

Expected:

```text
best descendant repaired pairs = 8/16
no validated descendant certificate
0/8 predictions
```

### 6. `delta1_ablation_replaced_before_validation`

Synthesize the correct Δ2 proposal under correct `L1`, then remove exact `Δ1` by rebuilding fresh state and admit the valid wrong ancestor before independent Δ2 validation.

The replacement language still has one admitted ancestor and therefore receives the same 918-candidate validator budget.

Expected:

```text
full validator recomputation
ancestor-language/proof mismatch or proof mismatch
no Δ2 certificate
0/8 predictions
```

### 7. `delta2_payload_only`

Correct Δ1 is admitted. Correct Δ2 is fully synthesized and independently validated, but only proposal/proof/scalar payload is retained; Δ2 is not admitted to layered executable state.

Expected: `0/8` predictions.

### 8. `counterfeit_delta2_proof`

Correct Δ1 is admitted. A structurally plausible Δ2 proof is tampered before independent full validation.

Expected:

```text
full 918-candidate validator recomputation
validation rejection
0/8 predictions
```

### 9. `outcome_shuffled_descendant`

Correct Δ1 is admitted. Stage-2 outcomes are deterministically permuted while preserving the 4/4 outcome count and complete descendant-search budget.

Expected:

```text
no valid target descendant certificate
0/8 predictions
```

### 10. `delayed_delta2_admission`

Correct Δ1 and Δ2 are synthesized and validated. Δ2 is admitted only after the frozen transfer-prediction window.

Expected:

```text
0/8 predictions during window
successful eventual Δ2 admission after window
```

## Wrong-ancestor fixture

The valid wrong ancestor problem uses the same eight raw histories and the same Ω1 grammar, but its independently witnessed stage-1 outcomes are:

```text
Ywrong = r
```

It therefore produces a genuine root-local validated Ω1 certificate with the same raw vocabulary, candidate count, search complexity, support, and 16-defect structure as the correct Δ1.

Across the exhaustive `(p,q,r)` cube, no descendant program of the form:

```text
r == raw_bit
r != raw_bit
```

can express `p == q`; the frozen maximum is 8/16 repaired opposite-outcome pairs.

## Frozen budgets

### Ω1 ancestor search

For every complete proposer or validator search over eight histories and six atoms:

```text
vocabulary scans = 8
history-pair evaluations = 28
raw candidate programs = 459
raw program/history executions = 3,672
unique raw partitions = 4
```

Correct and wrong-valid ancestor paths use the same complete Ω1 search budget.

### L0 raw stage-2 expressibility audit

For every complete audit:

```text
vocabulary scans = 8
raw candidate programs = 459
raw program/history executions = 3,672
unique raw partitions = 4
opposite-outcome pair checks = 16
```

### L1 descendant search

With exactly one admitted ancestor, every complete proposer or validator search uses:

```text
ancestor terminals = 1
raw candidate programs = 459
descendant candidate programs = 918
descendant program/history executions = 7,344
unique descendant partitions = 4
opposite-outcome pair checks = 16
```

A path that intentionally has no admitted ancestor must report:

```text
ancestor terminals = 0
descendant candidate programs = 0
```

and must not fabricate matched descendant computation with a synthetic bit.

## Frozen success gates

A `PASS` requires all of:

1. exact `16/8/32` root split and four future families;
2. every root has exactly six raw atoms and eight discovery plus eight transfer histories;
3. every correct and wrong stage-1 Ω1 problem has exactly 16 base alias defects;
4. every correct and wrong stage-1 complete search has exact `459 / 3,672 / 4` program/evaluation/partition counts;
5. correct Δ1 validates and admits on every stateful/control path that requires it;
6. wrong-valid ancestor validates and admits on every path that requires it;
7. L0 stage-2 raw expressibility audit has exactly `459` candidates, `4` unique partitions, best score `8/16`, and no complete repair on every root;
8. L0 descendant candidate count is exactly `0` without an admitted ancestor;
9. correct L1 descendant search has exactly `918` candidates, `7,344` candidate/history executions, `4` unique partitions, winner `16/16`, runner-up `8/16`, margin `8`, support `4` on every root;
10. correct L1 has exactly `8` remaining stage-2 alias defects before Δ2 admission;
11. every stateful training root predicts `8/8` transfer outcomes;
12. every stateful holdout root predicts `8/8` transfer outcomes;
13. every stateful future root predicts `8/8` transfer outcomes;
14. every non-stateful control produces `0/8` correct individual transfer predictions per root;
15. wrong-valid ancestor path receives a full 918-candidate descendant search but validates no Δ2 certificate;
16. exact-Δ1 ablation/replacement causes the previously proposed Δ2 to fail independent full validation on every root;
17. counterfeit Δ2 proofs are rejected on every root after full recomputation;
18. outcome-shuffled descendant synthesis validates no target certificate on every root;
19. delayed Δ2 admission succeeds eventually but yields zero predictions during the frozen window;
20. all four future families independently achieve stateful success `1.0` and maximum control success `0.0`;
21. all declared budgets are exact;
22. exact replay from fresh state holds for every root/path;
23. Ω1 and layered-state provenance/invariants hold everywhere.

Terminal classification:

```text
CONTROL_FAILURE
```

if budget, certificate-binding, or invariant controls fail;

```text
REPLAY_FAILURE
```

if exact fresh-state replay fails;

```text
PASS
```

only if every frozen success gate passes;

otherwise:

```text
REJECTED
```

No grammar rule, fixture structure, hidden outcome law, family, root count, control, budget, threshold, acceptance gate, or terminal-classification rule may be changed after the first complete verdict-producing run to rescue a failure.

## Required interpretation of PASS

A PASS would support only:

> Under the frozen symbolic history regime and bounded descendant grammar, an independently validated and admitted Ω1 refinement created an executable ancestor-bit terminal that did not exist in `L0`; the stage-2 behavioral partition was not expressible by any single Ω1 raw program and the descendant candidate language was empty under `L0`, but after exact `Δ1` admission the new hypothesis language enumerated and validated a descendant `Δ2` whose timely admission was necessary for withheld prediction. Replacing or ablating exact `Δ1` removed validation of the descendant despite matched descendant-search compute under a valid wrong ancestor.

## Claims explicitly not established

A PASS would not establish:

- unrestricted recursive ontology growth;
- learned invention of the descendant grammar itself;
- arbitrary-depth self-extension;
- natural-language concept genesis;
- open-world causal abstraction;
- safe automatic production promotion;
- AGI;
- consciousness;
- human-level cognition.

## Scientific next step after PASS

The next justified experiment would be **Ω3 depth and grammar mutation**: require at least a third descendant whose necessary composition operator is not present in the frozen Ω2 descendant grammar and must itself be proposed, independently validated, and admitted under strict anti-self-authorization controls.
