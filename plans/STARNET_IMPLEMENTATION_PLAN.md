# StarNet Implementation Plan

**Status:** ✅ IMPLEMENTED — All neurons, training, serialization complete

---

## 1. Extend Neurons

Wrap additional Starfire modules as neurons:
- `CausalNeuron` - Wraps CausalEngine for causal reasoning
- `GoalsNeuron` - Wraps Goals for hierarchical goal management
- `FewShotNeuron` - Wraps FewShotLearning for rapid hypothesis formation
- `CurriculumNeuron` - Wraps CurriculumEngine for self-directed learning
- `WorldModelNeuron` - Wraps WorldModel for entity/relation perception

### Files to create/modify
- `lib/neural/neurons/causal_neuron.rs` (new)
- `lib/neural/neurons/goals_neuron.rs` (new)
- `lib/neural/neurons/fewshot_neuron.rs` (new)
- `lib/neural/neurons/curriculum_neuron.rs` (new)
- `lib/neural/neurons/worldmodel_neuron.rs` (new)
- `lib/neural/neurons/mod.rs` (update exports)

---

## 2. Training Implementation

Implement working gradient computation and training algorithms:

### Hebbian Learning
- Connection weight update: `Δw = η * pre_activity * post_activity`
- Track pre/post traces per neuron
- Apply weight changes respecting connection types

### BPTT (Backpropagation Through Time)
- Implement `backward()` pass on NeuralNet
- Compute gradients through topological order (reverse)
- Update weights using `Trainer::update_weight()`

### Files to modify
- `lib/neural/train.rs` - Implement actual gradient computation
- `lib/neural/network.rs` - Add backward pass method

---

## 3. Serialization

Add save/load functionality for NeuralNet:

### Requirements
- Serialize NetworkConfig, Topology, Connection weights
- Serialize individual neuron configurations
- Implement deserialization to rebuild network state
- Use serde for JSON or binary formats

### Files to modify
- `lib/neural/network.rs` - Add `serialize()` and `deserialize()` methods
- `lib/neural/neuron.rs` - Add serialization to NeuronState, NeuralSignal
- `lib/neural/train.rs` - Save/load training state (momentum, velocities)

---

## 4. Plastic Connections

Implement adaptive connection weights based on signal correlations:

### Hebbian Plasticity
- For each connection, compute correlation between pre/post signals
- Update weight: `w = w + λ * correlation * (1 - |w|)` (soft bounds)
- Apply only during forward pass, store traces

### Three mechanisms
1. **Excitatory plasticity** - Strengthen when pre and post both active
2. **Inhibitory plasticity** - Strengthen when pre active, post suppressed
3. **Modulatory plasticity** - Depend on global reward/error signals

### Files to modify
- `lib/neural/network.rs` - Add `apply_plasticity()` method
- `lib/neural/neuron.rs` - Add trace fields to NeuronState
- `lib/neural/train.rs` - Implement plastic update rule

---

## Implementation Order

1. Extend neurons (5 new neurons)
2. Implement training (Hebbian + BPTT)
3. Add serialization
4. Add plastic connections

Each step should compile and pass tests before moving to the next.
