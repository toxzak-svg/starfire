# SPEC: Fact-Locked Circuit — Anti-Hallucination Hard Constraint

## What It Is

A Starfire module that trains a small FSM circuit on Zach's verified facts (project names, file paths, decision dates, conversation outcomes), then uses that circuit as a hard constraint on Bonsai's output — bumping any generated text that contradicts a known fact.

Not RLHF. Not a separate safety layer. A **state machine that is literally incapable of certain wrong answers**, making a small model (Bonsai-8B Q4) punch above its weight on factual correctness.

## Why It Matters

Bonsai hallucinates. Zach already said "FUCK BONSAI" tonight. Small models are lossy — they compress knowledge and fill gaps with plausible fiction. The circuit fixes this structurally: if a fact is in the circuit, the output gets checked against it. The circuit doesn't reason about facts — it *enforces* them the way a chess program enforces legal moves.

## Architecture

```
Zach's verified facts (JSON)
        ↓
   FactEncoder
   (converts facts → token sequences)
        ↓
   circuit_lm.train_fsm()  ← CPU-only, no GPU needed
        ↓
   FactCircuit (FSM, ~16-64 states, char-level vocab)
        ↓
   [Bonsai generates text]
        ↓
   FactChecker
   (for each generated token, check circuit state)
        ↓
   If circuit rejects → reroute to safe fallback ("I don't know" or circuit-constrained generation)
        ↓
   Verified output
```

## Fact Schema

```json
{
  "type": "project_name",
  "value": "circuit_lm",
  "tokens": [99, 104, 105, ...]  // BPE encoded
},
{
  "type": "file_path",
  "value": "projects/starfire/lib/llm",
  "tokens": [...]
},
{
  "type": "decision",
  "value": "use two-phase PDA trainer",
  "context": "vocab > 1000 causes joint solver to never finish",
  "date": "2026-04-12"
}
```

## Training (CPU-only)

1. Collect facts from Zach's workspace (project names, recent commits, decisions logged in daily notes)
2. Encode each fact as a token sequence (char-level or BPE at low vocab)
3. Train FSM circuit with `circuit_lm.train_fsm()` — vocab_size ≤ 64, state_bits ≤ 4
4. Store the circuit as a JSON file in `~/.star/facts/`

## FactChecker Interface

```rust
pub struct FactCircuit {
    transitions: HashMap<(u32, u32), u32>,  // (state, token) → next_state
    valid_states: HashSet<u32>,
    fact_tokens: Vec<u32>,
}

impl FactCircuit {
    /// Returns true if the given token sequence is consistent with known facts
    pub fn validate(&self, tokens: &[u32]) -> ValidationResult {
        // Walk the token sequence through the circuit state machine
        // If we reach a state that's inconsistent with fact boundaries, return Violation
    }
    
    /// Given a partial generation and the circuit state, return allowed next tokens
    pub fn allowed_next_tokens(&self, partial: &[u32]) -> Vec<u32> {
        // Instead of returning ALL tokens, return only tokens that keep us in valid states
        // Bonsai samples from this constrained set
    }
}
```

## Integration with Starfire Runtime

1. Load `FactCircuit` at startup (lazy, ~10KB JSON)
2. After Bonsai generates each token, run through `FactChecker.validate()`
3. If validation fails: mark the generation as untrusted, log the violation, fall back to circuit-consistent generation
4. Log a "fact_violation" event to TCMW-A's CEF so it learns Zach's corrections

## Files

- `projects/starfire/lib/fact_lock/` — new module
  - `mod.rs` — FactCircuit struct + ValidationResult
  - `encoder.rs` — converts JSON facts → token sequences
  - `checker.rs` — FactChecker, validates token sequences against circuit
  - `loader.rs` — loads fact circuit from JSON at startup

## Status

Not started. Waiting on: circuit_lm training pipeline to complete (GPU), fact collection from Zach's workspace.