# Global Workspace Hive

**Core intuition:** Brain-inspired global workspace theory + graph of small processors compiled into a single trainable architecture (not a multi-agent system at inference time).

> 📄 Full specification: [SPEC.md](./SPEC.md)

## Architecture
- **Processors**: 64-256 tiny RNNs or micro-LLMs specialized for different tasks (syntax, math, memory, planning)
- **Workspace**: Dense vector (8192 dim) acting as global broadcast each timestep
- **Router/GraphDesigner**: Lightweight transformer outputting sparse adjacency matrix (which processor connects to which)
- **Competition**: Top-k softmax (k=4-8) selects processors that write to workspace
- **I/O layer**: Embed tokens → workspace seed; workspace state → logits

## Key Insight
Kill the powerful pretrained backbone — instead train weak specialized experts and use a meta-learner to dynamically route and blend. Intelligence emerges from coalition.

## Why It Could Matter
- If it outperforms same-param transformers on reasoning/compositionality → working "global workspace LM"
- Almost 1:1 mapping to hive-mind / Neural-OS intuitions
- Trainable monolith vs brittle multi-agent pipeline

## What Could Go Wrong
- Processors might not learn useful specializations
- Workspace becomes bottleneck
- Dynamic routing collapses to identity

## Benchmark
- Baseline: same-param dense transformer
- Ablations: collapse to single processor, remove competition, remove dynamic graph
- Tasks: compositional reasoning, tool use, multi-step planning