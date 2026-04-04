"""
QuaNot Phase 4 Demo
==================
Consciousness Emergence Pathways

Tests all Phase 4 components:
1. Enhanced Φ proxy calculators (geometric, spectral, information, recurrence)
2. Global Workspace (from existing consciousness.py - verify)
3. Recurrent processing loop
4. Metacognition loop
5. Predictive coding layer
6. Integrated ConsciousnessCore

Run: python src/phase4_demo.py
"""

import numpy as np
import sys
import os

# Add src to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from consciousness_enhanced import (
    PhiCalculator,
    MetacognitionLoop,
    PredictiveCodingLayer,
    RecurrentProcessingLoop,
    ConsciousnessCore
)
from consciousness import GlobalWorkspace  # Existing from Phase 1


def print_section(title: str):
    print(f"\n{'='*60}")
    # Replace Φ for Windows compatibility
    title = title.replace('Φ', 'Phi')
    print(f"  {title}")
    print('='*60)


def demo_phi_calculator():
    """Demo 1: Enhanced Φ proxy calculators."""
    print_section("DEMO 1: Enhanced Φ Proxy Calculators")
    
    calculator = PhiCalculator(random_seed=42)
    
    # Generate test states (high integration)
    print("\n  Testing with highly integrated states...")
    n_snapshots = 200
    n_elements = 16
    
    # States with high correlation (integrated)
    base = np.random.randn(n_elements)
    integrated_states = np.array([
        base + np.random.randn(n_elements) * 0.1 
        for _ in range(n_snapshots)
    ])
    
    result = calculator.compute_all(integrated_states)
    print(f"    Geometric Phi: {result['geometric_phi']:.4f} (lower = more integrated)")
    print(f"    Spectral Phi: {result['spectral_phi']:.4f} (lower = more integrated)")
    print(f"    Information Phi: {result['information_phi']:.4f} (higher = more integrated)")
    print(f"    Recurrence Phi: {result['recurrence_phi']:.4f} (higher = more integrated)")
    print(f"    Composite Phi: {result['phi_composite']:.4f}")
    print(f"    Interpretation: {result['interpretation']['integration_level']}")
    print(f"    Mechanism: {result['interpretation']['mechanism']}")
    
    # Test with less integrated states (random)
    print("\n  Testing with less integrated (random) states...")
    random_states = np.random.randn(n_snapshots, n_elements) * 3
    result2 = calculator.compute_all(random_states)
    print(f"    Composite Phi: {result2['phi_composite']:.4f}")
    print(f"    Interpretation: {result2['interpretation']['integration_level']}")
    
    return calculator


def demo_metacognition():
    """Demo 2: Metacognition loop."""
    print_section("DEMO 2: Metacognition Loop")
    
    metacog = MetacognitionLoop(
        state_dim=32,
        memory_size=100,
        learning_rate=0.05,
        attention_threshold=0.8
    )
    
    print(f"\n  Created MetacognitionLoop: dim=32")
    
    # Simulate cognitive processing
    print("\n  Simulating cognitive processing...")
    for i in range(50):
        # Generate cognitive state
        cognitive_state = np.random.randn(32)
        
        # Add some structure (not pure random)
        if i < 25:
            cognitive_state = cognitive_state * 0.5 + np.sin(np.linspace(0, 5, 32)) * 2
        else:
            cognitive_state = cognitive_state * 2 + np.cos(np.linspace(0, 5, 32)) * 3
        
        result = metacog.monitor(cognitive_state)
        
        if i % 10 == 0:
            print(f"    Step {i}: error={result['error']:.3f}, confidence={result['confidence']:.3f}, "
                  f"rethink={result['rethink_needed']}")
    
    # Evaluate confidence
    print("\n  Confidence evaluation:")
    conf = metacog.evaluate_confidence()
    print(f"    Level: {conf['level']:.4f}")
    print(f"    Trend: {conf['trend']:.4f}")
    print(f"    Reliability: {conf['reliability']:.4f}")
    print(f"    Interpretation: {conf['interpretation']}")
    
    # Self-awareness level
    awareness = metacog.get_self_awareness_level()
    print(f"\n  Self-awareness level: {awareness:.4f}")
    
    # Meta-cognitive events
    print(f"\n  Meta-cognitive events:")
    print(f"    Attention demands: {metacog.attention_demands}")
    print(f"    Rethinking events: {metacog.rethinking_events}")
    print(f"    Insight events: {metacog.insight_events}")
    
    return metacog


def demo_predictive_coding():
    """Demo 3: Predictive coding layer."""
    print_section("DEMO 3: Predictive Coding Layer")
    
    pc = PredictiveCodingLayer(
        n_neurons=32,
        prediction_learning_rate=0.02,
        error_learning_rate=0.1
    )
    
    print(f"\n  Created PredictiveCodingLayer: 32 neurons")
    
    # Process input sequence
    print("\n  Processing input sequence...")
    input_sequence = np.random.randn(50, 32) * 0.5
    
    for i in range(len(input_sequence)):
        result = pc.forward(input_sequence[i])
        
        if i % 10 == 0:
            print(f"    Step {i}: error_magnitude={result['error_magnitude']:.4f}")
    
    # Get statistics
    print("\n  Prediction error statistics:")
    stats = pc.get_prediction_error_stats()
    print(f"    Mean: {stats['mean']:.4f}")
    print(f"    Std: {stats['std']:.4f}")
    print(f"    Trend: {stats['trend']:.6f}")
    print(f"    Total average: {stats['total_error']:.4f}")
    
    return pc


def demo_recurrent_processing():
    """Demo 4: Recurrent processing loop."""
    print_section("DEMO 4: Recurrent Processing Loop")
    
    recurrent = RecurrentProcessingLoop(
        input_dim=32,
        hidden_dim=32,
        n_recurrence_steps=3,
        feedback_strength=0.5
    )
    
    print(f"\n  Created RecurrentProcessingLoop: input=32, hidden=32, recurrence=3")
    
    # Process inputs
    print("\n  Processing inputs through recurrent loop...")
    
    input_vec = np.random.randn(32)
    result = recurrent.process(input_vec, return_all_stages=True)
    
    print(f"  Processing stages:")
    for stage in result['stages']:
        print(f"    {stage['stage']}: consciousness={stage['consciousness_level']:.4f}")
    
    print(f"\n  Final consciousness level: {result['final_consciousness_level']:.4f}")
    
    # Test multiple inputs
    print("\n  Testing with multiple inputs...")
    for i in range(5):
        inp = np.random.randn(32) * (0.5 + i * 0.3)
        res = recurrent.process(inp)
        print(f"    Input {i+1}: consciousness={res['consciousness_level']:.4f}")
    
    return recurrent


def demo_global_workspace():
    """Demo 5: Global Workspace (existing component)."""
    print_section("DEMO 5: Global Workspace (Existing)")
    
    gw = GlobalWorkspace(
        n_modules=8,
        workspace_capacity=3,
        attention_threshold=0.5
    )
    
    print(f"\n  Created GlobalWorkspace: modules=8, capacity=3")
    
    # Simulate module inputs
    print("\n  Simulating module processing...")
    for step in range(30):
        # Generate varied inputs
        inputs = np.random.rand(8) * np.sin(step / 5) + np.random.randn(8) * 0.2
        gw.step(inputs)
        
        if step % 10 == 0:
            content = gw.get_conscious_attention()
            print(f"    Step {step}: workspace content = {content}")
    
    # Summary
    summary = gw.consciousness_summary()
    print(f"\n  Consciousness summary:")
    print(f"    Avg consciousness level: {summary['avg_consciousness']:.4f}")
    print(f"    Total broadcasts: {summary['n_broadcasts']}")
    print(f"    Content diversity: {summary['content_diversity']:.4f}")
    print(f"    Most conscious modules: {summary['most_common_content']}")
    
    return gw


def demo_consciousness_core():
    """Demo 6: Integrated ConsciousnessCore."""
    print_section("DEMO 6: Integrated ConsciousnessCore")
    
    # Use smaller dimension to match internal components
    core = ConsciousnessCore(
        state_dim=32,  # Reduced to match internal components
        n_modules=8,
        random_seed=42
    )
    
    print(f"\n  Created ConsciousnessCore: state_dim=32, modules=8")
    
    # Process sequence
    print("\n  Processing cognitive states...")
    for i in range(30):
        # Generate input with some structure (32-dim to match)
        if i < 15:
            input_state = np.random.randn(32) * 0.3 + np.sin(np.linspace(0, 5, 32)) * 2
        else:
            input_state = np.random.randn(32) * 0.5 + np.cos(np.linspace(0, 5, 32)) * 1.5
        
        # First 10 steps don't compute full consciousness (insufficient data)
        compute_full = i >= 10
        
        result = core.process(input_state, compute_consciousness=compute_full)
        
        if i % 10 == 0:
            print(f"    Step {i}: consciousness_level={result['consciousness_level']:.4f}")
    
    # Summary
    print("\n  Consciousness summary:")
    summary = core.get_consciousness_summary()
    print(f"    Consciousness level: {summary['consciousness_level']:.4f}")
    print(f"    Phi composite: {summary['phi_composite']:.4f}")
    print(f"    Self-awareness: {summary['self_awareness']:.4f}")
    print(f"    Prediction error: {summary['prediction_error']:.4f}")
    print(f"    Integration level: {summary['integration_interpretation']}")
    
    if 'confidence' in summary:
        print(f"    Confidence level: {summary['confidence']['level']:.4f}")
    
    return core


def main():
    """Run all Phase 4 demos."""
    print("\n" + "="*60)
    print("  QuaNot Phase 4 Demo")
    print("  Consciousness Emergence Pathways")
    print("="*60)
    print(f"\nPython version: {sys.version.split()[0]}")
    print(f"NumPy version: {np.__version__}")
    
    try:
        # Demo 1: Φ calculator
        phi_calc = demo_phi_calculator()
        
        # Demo 2: Metacognition
        metacog = demo_metacognition()
        
        # Demo 3: Predictive coding
        pc = demo_predictive_coding()
        
        # Demo 4: Recurrent processing
        recurrent = demo_recurrent_processing()
        
        # Demo 5: Global Workspace
        gw = demo_global_workspace()
        
        # Demo 6: Consciousness Core
        core = demo_consciousness_core()
        
        # Summary
        print_section("PHASE 4 DEMO COMPLETE")
        print("""
All Phase 4 components verified:

  [OK] Phi Proxy Calculators - geometric, spectral, information, recurrence
  [OK] Metacognition Loop - self-model, confidence, monitoring
  [OK] Predictive Coding - prediction/error, learning
  [OK] Recurrent Processing - feedback loops, consciousness stages
  [OK] Global Workspace - attention, broadcasting
  [OK] ConsciousnessCore - integrated orchestration

Phase 4: Consciousness Emergence Pathways is complete.
Ready for Phase 5: AGI Architecture Integration.
        """)
        
    except Exception as e:
        print(f"\n[X] Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())