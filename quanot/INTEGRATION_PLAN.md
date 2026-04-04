# QuaNot + Starfire Integration Plan

> Connecting quantum-inspired perception to symbolic reasoning

**Created:** 2026-04-03  
**Status:** Planning  
**Related:** quanot (Python), starfire (Rust)

---

## Executive Summary

QuaNot and Starfire are complementary architectures that together form a full AGI pipeline:

| Layer | quanot (Python) | starfire (Rust) |
|-------|-----------------|-----------------|
| **Focus** | Perception & Creativity | Reasoning & Identity |
| **Paradigm** | Neural dynamical systems | Symbolic AI |
| **Memory** | ESN reservoir state | SQLite with decay |
| **Consciousness** | IIT proxies (Φ), GWT | Functional meta-cognition |

**Unified Pipeline:**

```
Input → QuaNot (perception) → Starfire (reasoning) → Output
         ↑                    ↓
         ←←←←feedback←←←←←
```

---

## 1. Architecture Overview

### 1.1 The Integration

```
┌─────────────────────────────────────────────┐
│           UNIFIED AGI SYSTEM                │
│                                                   │
│  ┌───────────────┐   ┌───────────────┐     │
│  │  QuaNot       │   │  Starfire      │     │
│  │  (Python)     │◄──►│  (Rust)       │     │
│  │               │    │               │     │
│  │  ESN Reservoir│    │  Memory      │     │
│  │  Creative Osc │    │  Reasoning   │     │
│  │  Φ-Metrics    │    │  Identity    │     │
│  │  Novelty      │    │  Meta-Cog    │     │
│  └──────┬───────┘    └──────┬───────┘     │
│         │                   │              │
│         ▼                   ▼              │
│  ┌──────────────────────────────────┐    │
│  │      Integration Layer            │    │
│  │  - State encoder/decoder         │    │
│  │  - Message protocol               │    │
│  │  - Bidirectional feedback        │    │
│  └──────────────────────────────────┘    │
└────────────────────────────────────────────┘
```

### 1.2 Data Flow

```
User Input (text)
    ↓
[QuaNot: Encode] → Reservoir State (N=1000 vector)
    ↓
[QuaNot: Consciousness] → ψ proxy (0-1)
    ↓
[Integration: Encode] → Symbol form (text/memory)
    ↓
[Starfire: Reason] → Symbolic output
    ↓
[Starfire: Identity] → Coherent response
    ↓
[Integration: Decode] → User Output (text)
```

---

## 2. Integration Components

### 2.1 QuaNot Side (Python)

```python
# New file: quanot/src/starfire_interface.py

class StarfireBridge:
    """
    Bridge to Starfire for symbolic reasoning.
    
    Encodes reservoir states for Starfire input,
    decodes Starfire responses for QuaNot feedback.
    """
    
    def __init__(self, reservoir: ChaoticReservoir, starfire_path: str):
        self.reservoir = reservoir
        self.starfire_path = starfire_path
        self.state_history = []
        
    def encode_for_starfire(self, input_text: str) -> dict:
        """
        Encode input and reservoir state for Starfire.
        
        Returns structured memory object for Starfire.
        """
        # Get current reservoir state
        state = self.reservoir.get_state()
        
        # Compute consciousness proxy
        from consciousness import compute_phi_proxy
        psi = compute_phi_proxy(state)
        
        # Compute novelty
        from creativity import compute_novelty
        novelty = compute_novelty(state, self.state_history)
        
        return {
            'input': input_text,
            'reservoir_state': state.tolist(),  # For Starfire to store
            'consciousness': psi,
            'novelty': novelty,
            'domain': 'perceptual',  # Mark as perceptual input
            'importance': 0.5 + novelty * 0.5,  # Novel = more important
        }
    
    def decode_from_starfire(self, starfire_response: dict) -> dict:
        """
        Interpret Starfire response for QuaNot feedback.
        
        Returns modulation signals for reservoir.
        """
        # Extract reasoning result
        reasoning = starfire_response.get('result', '')
        
        # Extract confidence to modulate oscillation
        confidence = starfire_response.get('confidence', 0.5)
        
        # Curiosity drives exploration
        curiosity = starfire_response.get('curiosity', 0.0)
        
        return {
            'reasoning': reasoning,
            'confidence': confidence,
            'curiosity': curiosity,
            'oscillation_modulation': self._compute_modulation(
                confidence, curiosity
            ),
        }
    
    def _compute_modulation(self, confidence: float, curiosity: float) -> float:
        """
        Compute creative oscillation modulation from Starfire.
        
        High curiosity → increase exploration (chaos)
        High confidence → increase exploitation (order)
        """
        # Curiosity drives chaos, confidence drives order
        modulation = curiosity - confidence
        
        return np.clip(modulation, -1.0, 1.0)
```

### 2.2 Starfire Side (Rust)

Add perceptual input capability to Starfire:

```rust
// In starfire/src/

pub struct PerceptualInput {
    pub reservoir_state: Vec<f64>,      // Raw reservoir activation
    pub consciousness: f64,            // ψ proxy (0-1)
    pub novelty: f64,                   // Novelty score (0-1)
    pub domain: MemoryDomain,            // perceptual
}

impl PerceptualInput {
    /// Convert perceptual input to Starfire memory
    pub fn to_memory(&self) -> Memory {
        Memory {
            content: format!(
                "Perception: consciousness={:.2}, novelty={:.2}",
                self.consciousness, self.novelty
            ),
            domain: MemoryDomain::Perceptual,
            confidence: self.consciousness as f32,
            importance: (0.5 + self.novelty * 0.5) as f32,
            provenance: Some("quanot".to_string()),
            ..Default::default()
        }
    }
}
```

### 2.3 Message Protocol

```python
# quanot/src/bridge_protocol.py

from dataclasses import dataclass
from typing import Optional
import json

@dataclass
class BridgeMessage:
    """Message passed between QuaNot and Starfire."""
    
    # Header
    message_id: str
    timestamp: float
    
    # QuaNot → Starfire
    input_text: Optional[str] = None
    reservoir_state: Optional[list] = None  # 1000-dim vector
    consciousness_proxy: Optional[float] = None  # ψ
    novelty_score: Optional[float] = None
    creativity_scores: Optional[dict] = None
    
    # Starfire → QuaNot
    reasoning_result: Optional[str] = None
    confidence: Optional[float] = None
    curiosity: Optional[float] = None
    identity_consistent: Optional[bool] = None
    
    # Control
    response_requested: bool = False
    
    def to_json(self) -> str:
        """Serialize for IPC."""
        return json.dumps(self.__dict__)
    
    @classmethod
    def from_json(cls, data: str) -> 'BridgeMessage':
        """Deserialize from IPC."""
        return cls(**json.loads(data))
```

---

## 3. Interaction Patterns

### 3.1 Mode A: Sequential Processing

```
QuaNot processes input → Starfire reasons → QuaNot receives feedback
```

Best for: Complex reasoning tasks

```python
# Example flow
bridge = StarfireBridge(reservoir, starfire_path)

# 1. Process input in QuaNot
input_state = encoder(input_text)
states = reservoir.forward(input_state)
psi = compute_psi(states)

# 2. Send to Starfire
message = BridgeMessage(
    message_id=str(uuid.uuid4()),
    timestamp=time.time(),
    input_text=input_text,
    reservoir_state=reservoir.get_state().tolist(),
    consciousness_proxy=psi,
    novelty_score=novelty,
    response_requested=True,
)
response = await starfire.process(message.to_json())

# 3. Receive feedback
feedback = bridge.decode_from_starfire(response)
reservoir.apply_modulation(feedback['oscillation_modulation'])
```

### 3.2 Mode B: Parallel Processing

```
QuaNet runs continuously ↔ Starfire operates on demand
```

Best for: Continuous learning

```python
# Background mode - QuaNot runs, Starfire on-demand
async def continuous_mode():
    while running:
        # Process input
        state = reservoir.forward_step(encoder(input_text))
        
        # Check for consciousness events
        psi = compute_psi(state)
        if psi > 0.7:  # High consciousness threshold
            # Trigger Starfire reasoning
            await starfire.process_high_consciousness(state, psi)
        
        # Apply creative modulation
        creativity = evaluate_creativity(state)
        oscillator.step(current_value=creativity, divergence_metric=divergence)
```

### 3.3 Mode C: Bidirectional Learning

```
QuaNot → Starfire: "Here's what I perceived"
Starfire → QuaNot: "Here are my insights, apply this modulation"
```

Best for: Full integration

---

## 4. Key Integration Points

### 4.1 Consciousness Coupling

| QuaNot | Starfire | Connection |
|-------|---------|-------------|
| ψ proxy (Φ) | Meta-cognition | Bidirectional |
| Creative state | Curiosity | Starfire → QuaNot |
| Novelty | Surprise detection | Starfire → QuaNot |

### 4.2 Memory Coupling

| QuaNot stores | Starfire stores |
|--------------|---------------|
| Reservoir states (implicit) | Explicit memories (SQLite) |
| Attractor basins | Knowledge graph |
| Creative trajectories | Reasoning chains |

### 4.3 Identity Coupling

- **QuaNot**: Implicit identity in attractor dynamics
- **Starfire**: Explicit identity in IDENTITY.md
- **Integration**: QuaNot states tagged with Starfire's identity context

---

## 5. Implementation Roadmap

### Phase I: Basic Bridge (Week 1-2)

- [ ] Create `starfire_interface.py` 
- [ ] Implement state encoder/decoder
- [ ] IPC mechanism (pipe/file/TCP)
- [ ] Basic test: QuaNot → Starfire message

### Phase II: Bidirectional (Week 3-4)

- [ ] Starfire receives perceptual input
- [ ] Starfire sends back modulation
- [ ] Consciousness event triggers
- [ ] Novelty → curiosity mapping

### Phase III: Full Integration (Week 5-6)

- [ ] Continuous dual-process
- [ ] Memory synchronization
- [ ] Identity context passing
- [ ] End-to-end test: input → reasoning → output

### Phase IV: Optimization (Week 7-8)

- [ ] Performance tuning
- [ ] State compression (for IPC)
- [ ] Caching for responses
- [ ] Benchmark comparison

---

## 6. IPC Mechanisms

Options for Python ↔ Rust communication:

| Mechanism | Pros | Cons |
|-----------|------|------|
| **STDIO** | Simple, reliable | Blocking |
| **Named pipes** | Fast, async | Windows-specific |
| **TCP socket** | Cross-platform | Network overhead |
| **Files** | Simple, debuggable | Disk I/O |
| **FFI (cffi)** | Fastest, direct | Complex setup |

**Recommendation:** Start with **STDIO pipes** (Rust reads stdin, Python writes), progress to **TCP socket** for production.

---

## 7. Expected Capabilities

After full integration:

| Capability | Before | After |
|------------|--------|-------|
| Text input | No | Yes |
| Fluent response | No | Yes |
| Symbolic reasoning | No | Yes (Starfire) |
| Identity continuity | No | Yes (Starfire) |
| Memory with decay | No | Yes (Starfire) |
| Curiosity | QuaNot only | Combined |
| Creative output | Numerical | Textual |

---

## 8. Testing Plan

### Unit Tests

```python
# test_integration.py

def test_bridge_encode():
    bridge = StarfireBridge(...)
    msg = bridge.encode_for_starfire("Hello world")
    assert msg.input_text == "Hello world"
    assert len(msg.reservoir_state) == 1000
    assert 0 <= msg.consciousness_proxy <= 1

def test_bridge_decode():
    response = {'result': 'Reasoned!', 'confidence': 0.8, 'curiosity': 0.3}
    feedback = bridge.decode_from_starfire(response)
    assert feedback['confidence'] == 0.8
    assert -1 <= feedback['oscillation_modulation'] <= 1
```

### Integration Tests

```python
def test_full_pipeline():
    # Input → QuaNot → Starfire → Output
    result = await unified_system.process("What is consciousness?")
    assert isinstance(result, str)
    assert len(result) > 0
    assert "aware" in result.lower() or "think" in result.lower()
```

---

## 9. Files to Create/Modify

### New Files (quanot)

```
quanot/src/
├── starfire_interface.py   # Bridge to Starfire
├── bridge_protocol.py     # Message protocol
├── integration_demo.py   # Integration demo
└── test_integration.py   # Integration tests
```

### Modified Files (quanot)

```
quanot/src/
├── agi_core.py           # Add Starfire integration option
├── main.py               # Add --starfire flag
└── cli.py               # Add starfire subcommands
```

### Modified Files (starfire)

```
starfire/src/
├── input_parser.rs      # Add perceptual input parsing
├── memory.rs            # Add Perceptual domain
├── main.rs             # Add --quanot flag
└── cli.rs              # Add quanot subcommands
```

---

## 10. Summary

This integration connects QuaNot's **neural dynamical** perception to Starfire's **symbolic** reasoning:

- **QuaNot**: ESN → creative oscillation → ψ proxies → novelty
- **Starfire**: Memory → reasoning → identity → emergence

**Result**: A system that can process text input, reason symbolically, maintain identity continuity, and exhibit creative exploration — a step toward fluent AGI.

---

*Plan: Ready for implementation*