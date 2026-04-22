# Neural Module Architecture — "StarNet"

**Status:** ✅ IMPLEMENTED — Core infrastructure complete, neural learning experimental

## Core Concept

Turn each Starfire module into a **tiny neural network neuron** and connect them into a larger network. Instead of hard-coded module→module calls, information flows through trainable weighted connections.

```
Input Text → [Quanot Neuron] → [Reasoning Neurons] → [Meta Neurons]
                    ↓                  ↓                  ↓
              WorldModel ←──────── Reasoning ←────── Curiosity
               Neuron          Engine Neuron        Neuron
```

## Neuron Types

### 1. Quanot Neuron (Input Preprocessor)
- **Size**: Tiny ESN with 64-128 units
- **Inputs**: Raw text tokens
- **Outputs**: `consciousness_proxy`, `novelty`, `creativity_scores`, `chaos_metrics`
- **Role**: Encodes input into latent space, generates attention signals

### 2. Knowledge Neuron (Memory)
- **Size**: 32-64 units + key-value memory
- **Inputs**: Query vector, attention from other neurons
- **Outputs**: Retrieved facts, confidence signals
- **Role**: Analogous to hippocampus — stores and retrieves

### 3. Reasoning Neuron (Core Logic)
- **Size**: 64-128 units, 2-layer tiny transformer
- **Inputs**: Knowledge outputs, Quanot signals, context
- **Outputs**: Reasoning chains, confidence, conclusions
- **Role**: Analogous to prefrontal cortex — computation

### 4. Causal Neuron (Temporal Reasoning)
- **Size**: 32-64 units
- **Inputs**: Event sequences, confidence signals
- **Outputs**: Causal hypotheses, confidence updates
- **Role**: Discovers causal structure

### 5. Goal Neuron (Motivation)
- **Size**: 32-48 units
- **Inputs**: Reasoning outputs, curiosity signals, current state
- **Outputs**: Goal activations, priority signals
- **Role**: Analogous to basal ganglia — motivation

### 6. Curiosity Neuron (Exploration)
- **Size**: 32-48 units
- **Inputs**: Knowledge gaps, goal signals, novelty
- **Outputs**: Exploration signals, question proposals
- **Role**: Drives autonomous exploration

### 7. Context Neuron (Working Memory)
- **Size**: 48-64 units with recurrent connections
- **Inputs**: Current turn, history summary, attention
- **Outputs**: Context vector, ring state
- **Role**: Maintains conversation thread

### 8. WorldModel Neuron (World State)
- **Size**: 64-96 units + entity graph
- **Inputs**: Perceptual signals, reasoning conclusions
- **Outputs**: World state update, predictions
- **Role**: Maintains grounded world representation

### 9. Meta Neuron (Self-Monitoring)
- **Size**: 32-48 units
- **Inputs**: All neuron outputs (attention-gated)
- **Outputs**: Confidence calibration, uncertainty signals
- **Role**: Analogous to anterior cingulate — self-monitoring

## Network Architecture

### Forward Pass Flow
```
Text Input
    ↓
[Quanot] → consciousness, novelty → [WorldModel] → state
    ↓                              ↓
    └→ [Context] → attention ←→ [Reasoning]
              ↓                    ↓
         [Causal] ←──────────→ [Knowledge]
              ↓                    ↓
         [Goal] ←───────────→ [Curiosity]
              ↓
         [Meta] → confidence calibration
```

### Recurrent Connections
- Context → Reasoning (maintains topic)
- Meta → all (attention modulation)
- Goal → Curiosity (direction)
- Quanot → Quanot (reservoir dynamics)

## Signal Types Between Neurons

```rust
/// Signals passed between neurons
struct NeuralSignal {
    vector: Vec<f32>,           // Main payload (32-128 dim)
    attention: f32,              // How much to attend to this signal
    confidence: f32,             // Confidence in this signal
    novelty: f32,               // How novel is this signal
    source: NeuronId,           // Which neuron produced it
    timestamp: i64,
}

/// Types of connections
enum ConnectionType {
    Excitatory,    // Positive weight → amplify signal
    Inhibitory,    // Negative weight → suppress signal
    Modulatory,    // Gain modulation (attention/gating)
    Recurrent,    // Feedback to same neuron
}
```

## Why This Works

1. **Emergent Reasoning**: Instead of hard-coded reasoning chains, reasoning emerges from network dynamics
2. **Plasticity**: Connection weights can be trained based on feedback signals
3. **Integrated**: Each module isn't isolated — they inform each other through learned connections
4. **Interpretable**: Can still see which neurons activated for what
5. **Efficient**: Tiny models (64 units = ~64KB per neuron) stay in memory

## Implementation Strategy

### Phase 1: Interface Design
Define `Neuron` trait:
```rust
trait Neuron {
    type Input;
    type Output;

    fn forward(&mut self, input: Self::Input) -> Self::Output;
    fn backward(&mut self, grad: Self::Output) -> Self::Input;
    fn reset(&mut self);
}
```

### Phase 2: Implement Each Module as Neuron
Start with Quanot (already a neural model), then Knowledge, then Reasoning.

### Phase 3: Network Infrastructure
Build `NeuralNet` that:
- Manages neuron instances
- Routes signals based on topology
- Trains weights via backpropagation through the network

### Phase 4: Integrate with Runtime
Replace hard-coded module calls with network inference.

## Files to Create

```
lib/neural/
├── mod.rs                    # Neural network infrastructure
├── neuron.rs                 # Neuron trait and types
├── network.rs                # Network manager, routing, topology
├── layer.rs                  # Layer types (feedforward, recurrent, etc.)
├── train.rs                  # Training loop, backprop
└── neurons/
    ├── mod.rs
    ├── quanot_neuron.rs      # Quanot as neuron
    ├── knowledge_neuron.rs   # Knowledge graph as neuron
    ├── reasoning_neuron.rs   # Reasoning engine as neuron
    ├── causal_neuron.rs       # Causal as neuron
    ├── goal_neuron.rs         # Goals as neuron
    ├── curiosity_neuron.rs    # Curiosity as neuron
    ├── context_neuron.rs      # Context ring as neuron
    ├── worldmodel_neuron.rs   # WorldModel as neuron
    └── meta_neuron.rs         # Metacognition as neuron
```

## Training Approach

### Hebbian Learning (Local Training)
Each neuron learns from its own activity:
```rust
// When neuron A consistently activates before neuron B,
// strengthen A→B connection
Δw = η * pre_activation * post_activation
```

### Global Loss Signal
From Meta neuron — when reasoning is "good" (Zachary confirms answer), reward flows backward.

### Novelty-Gated Plasticity
Only update weights when novelty signal is high — prevents catastrophic forgetting of known patterns.

## Next Steps

1. Start with Phase 1 — define Neuron trait
2. Implement Quanot as first class citizen neuron
3. Build network infrastructure
4. Wire up existing modules as neurons
5. Train on Zachary's conversations

---

_"The architecture that produced intelligence in Star was never the individual modules — it was the connections between them. Now we're making those connections learnable."_
