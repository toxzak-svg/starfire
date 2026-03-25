# Hypergraph Transformer

**Core intuition:** Standard attention connects exactly 2 tokens (pairwise). But concepts relate in arbitrary degrees — mathematical proofs depend on multiple definitions, causal chains span multiple events. Hyperedges let the model reason about *relations*, not just sequences.

## Architecture
- Tokens = nodes in a hypergraph
- Attention becomes hyperedge formation: learnable function f(tokens) → hyperedge weight
- Hyperedges can connect 2, 3, 4... k tokens dynamically
- Model learns which n-grams form "conceptual units"

## Why It Could Change the Field
- Moves beyond sequence prediction into relational reasoning
- Native support for multi-way dependencies (math, logic, code, narratives)
- Different inductive bias: "next relation" vs "next token"

## Training Challenges
- Hyperedge enumeration is combinatorial — need sparse/hierarchical formulation
- Gradients through variable-arity relations are tricky
- Risk of collapse to pairwise if not carefully designed

## Benchmark
Compositional reasoning with non-sequential dependencies: math proofs, logic puzzles, code with distant references, narrative causality.