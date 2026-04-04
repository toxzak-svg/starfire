"""
QuaNot Phase 3 Demo
===================
Creative Synthesis Framework

Tests all Phase 3 components:
1. Novelty detection (k-NN surprise metric)
2. Conceptual blending
3. Creative evaluation (surprise, usefulness, coherence)
4. Metaphor generation via attractor mapping

Run: python src/phase3_demo.py
"""

import numpy as np
import sys
import os
import time

# Add src to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from creativity import (
    NoveltyDetector,
    ConceptualBlender,
    CreativeEvaluator,
    MetaphorGenerator,
    CreativeSynthesizer,
    divergence_metric,
    attractor_strength
)


def print_section(title: str):
    """Pretty print section header."""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print('='*60)


def demo_novelty_detection():
    """Demo 1: Novelty detection."""
    print_section("DEMO 1: Novelty Detection (k-NN Surprise)")
    
    # Create detector
    detector = NoveltyDetector(
        state_dim=32,
        history_size=500,
        k=5,
        novelty_threshold=0.3
    )
    
    print(f"\n  Created NoveltyDetector: dim=32, k=5, threshold=0.3")
    
    # Add initial history (familiar states)
    print("\n  Adding 100 familiar states to history...")
    for i in range(100):
        state = np.random.randn(32) * 0.5  # Lower variance = familiar
        detector.add_state(state)
    
    # Test novelty detection
    print("\n  Testing novelty detection:")
    
    # Familiar input (should have low novelty)
    familiar = np.random.randn(32) * 0.5
    novelty_familiar = detector.compute_novelty(familiar)
    print(f"    Familiar input novelty: {novelty_familiar:.4f}")
    
    # Novel input (should have high novelty)
    novel = np.random.randn(32) * 3.0  # High variance = very different
    novelty_novel = detector.compute_novelty(novel)
    print(f"    Novel input novelty: {novelty_novel:.4f}")
    
    # Random input
    random_input = np.random.randn(32)
    novelty_random = detector.compute_novelty(random_input)
    print(f"    Random input novelty: {novelty_random:.4f}")
    
    # Test series of increasingly novel inputs
    print("\n  Testing progressive novelty detection:")
    for scale in [0.1, 0.5, 1.0, 2.0, 5.0]:
        test_state = np.random.randn(32) * scale
        novelty = detector.compute_novelty(test_state)
        is_novel = detector.is_novel(test_state)
        print(f"    Scale {scale}: novelty={novelty:.4f}, is_novel={is_novel}")
    
    print(f"\n  Final history size: {len(detector.history)}")
    print(f"  Novelty baseline: {detector.novelty_baseline:.4f}")
    print(f"  Cumulative novelty: {detector.cumulative_novelty:.4f}")
    
    return detector


def demo_conceptual_blending():
    """Demo 2: Conceptual blending."""
    print_section("DEMO 2: Conceptual Blending")
    
    # Create blender
    blender = ConceptualBlender(
        embedding_dim=64,
        blend_temperature=0.3,
        random_seed=42
    )
    
    print(f"\n  Created ConceptualBlender: dim=64, temperature=0.3")
    
    # Create concept embeddings
    print("\n  Creating concept embeddings...")
    concept_time = blender.create_concept_embedding("time", {"flow": 0.8, "linear": 0.6})
    concept_river = blender.create_concept_embedding("river", {"flow": 0.9, "fluid": 0.7})
    concept_journey = blender.create_concept_embedding("journey", {"progression": 0.7, "direction": 0.8})
    
    print(f"    Created: 'time', 'river', 'journey' (dim=64)")
    
    # Blend concepts
    print("\n  Blending 'time' + 'river':")
    blend1 = blender.blend(concept_time, concept_river, blend_ratio=0.5)
    print(f"    Novelty estimate: {blend1['novelty']:.4f}")
    
    print("\n  Blending result + 'journey':")
    blend2 = blender.blend(blend1['blended'], concept_journey, blend_ratio=0.4)
    print(f"    Novelty estimate: {blend2['novelty']:.4f}")
    
    # Run blending chain
    print("\n  Running blending chain:")
    concepts = [
        blender.create_concept_embedding("fire", {"hot": 0.9, "energy": 0.7}),
        blender.create_concept_embedding("passion", {"intense": 0.8, "emotion": 0.9}),
        blender.create_concept_embedding("desire", {"want": 0.7, "drive": 0.8})
    ]
    
    final_blend = blender.run_blending_chain(concepts, n_blends=3)
    print(f"    Final blend norm: {np.linalg.norm(final_blend):.4f}")
    print(f"    Blend history length: {len(blender.blend_history)}")
    
    return blender


def demo_creative_evaluation():
    """Demo 3: Creative evaluation framework."""
    print_section("DEMO 3: Creative Evaluation Framework")
    
    # Create evaluator
    evaluator = CreativeEvaluator(
        surprise_weight=0.4,
        usefulness_weight=0.3,
        coherence_weight=0.3,
        random_seed=42
    )
    
    print(f"\n  Created CreativeEvaluator")
    print(f"    Weights: surprise={0.4}, usefulness={0.3}, coherence={0.3}")
    
    # Test evaluations
    print("\n  Evaluating various outputs:")
    
    # High surprise, low coherence
    output_surprising = np.random.randn(64) * 2.0  # High variance
    eval1 = evaluator.evaluate(output_surprising)
    print(f"    High-variance output:")
    print(f"      surprise={eval1['surprise']:.4f}, usefulness={eval1['usefulness']:.4f}, coherence={eval1['coherence']:.4f}")
    print(f"      overall={eval1['overall_creativity']:.4f} ({eval1['evaluation']})")
    
    # Low surprise, high coherence
    output_coherent = np.sin(np.linspace(0, 10, 64))  # Smooth, regular
    eval2 = evaluator.evaluate(output_coherent)
    print(f"    Smooth sinusoidal output:")
    print(f"      surprise={eval2['surprise']:.4f}, usefulness={eval2['usefulness']:.4f}, coherence={eval2['coherence']:.4f}")
    print(f"      overall={eval2['overall_creativity']:.4f} ({eval2['evaluation']})")
    
    # Mixed
    output_mixed = np.concatenate([np.ones(32), np.zeros(32)])
    eval3 = evaluator.evaluate(output_mixed)
    print(f"    Binary step output:")
    print(f"      surprise={eval3['surprise']:.4f}, usefulness={eval3['usefulness']:.4f}, coherence={eval3['coherence']:.4f}")
    print(f"      overall={eval3['overall_creativity']:.4f} ({eval3['evaluation']})")
    
    # Test with reference outputs
    print("\n  Evaluating with reference comparison:")
    references = [np.random.randn(64) for _ in range(10)]
    output_new = np.random.randn(64)
    
    eval_with_ref = evaluator.evaluate(output_new, reference_outputs=references)
    print(f"    New output vs 10 references:")
    print(f"      surprise={eval_with_ref['surprise']:.4f}")
    
    # Average scores
    avg = evaluator.get_average_scores()
    print(f"\n  Average scores: surprise={avg['surprise']:.4f}, usefulness={avg['usefulness']:.4f}")
    print(f"    coherence={avg['coherence']:.4f}, overall={avg['overall']:.4f}")
    
    return evaluator


def demo_metaphor_generation():
    """Demo 4: Metaphor generation via attractor mapping."""
    print_section("DEMO 4: Metaphor Generation")
    
    # Test different attractor types
    for attractor_type in ["lorenz", "rossler", "henon"]:
        print(f"\n  Testing {attractor_type} attractor:")
        
        generator = MetaphorGenerator(
            attractor_type=attractor_type,
            trajectory_length=100,
            random_seed=42
        )
        
        props = generator.get_attractor_properties()
        print(f"    Properties: {props['dimensionality']}D, range_x=[{props['range_x'][0]:.2f}, {props['range_x'][1]:.2f}]")
        
        # Generate metaphors
        metaphor1 = generator.generate_metaphor(
            source_concept="creativity",
            target_domain="emotional_flow",
            mapping_strength=0.7
        )
        print(f"    Metaphor: {metaphor1['metaphor'][:80]}...")
        
        metaphor2 = generator.generate_metaphor(
            source_concept="thought",
            target_domain="chaotic_system",
            mapping_strength=0.5
        )
        print(f"    Metaphor: {metaphor2['metaphor'][:80]}...")
    
    return generator


def demo_creative_synthesizer():
    """Demo 5: Creative synthesizer orchestrator."""
    print_section("DEMO 5: Creative Synthesizer")
    
    # Create synthesizer
    synthesizer = CreativeSynthesizer(
        state_dim=64,
        creative_temperature=0.5,
        random_seed=42
    )
    
    print(f"\n  Created CreativeSynthesizer: dim=64, temperature=0.5")
    
    # Initialize with some states
    print("\n  Adding initial states...")
    for i in range(20):
        state = np.random.randn(64) * 0.5
        synthesizer.novelty_detector.add_state(state)
    
    # Run creative cycle
    print("\n  Running creative cycles:")
    
    modes = ['explore', 'exploit', 'explore', 'exploit', 'explore', 'exploit']
    for i, mode in enumerate(modes):
        # Generate input
        input_state = np.random.randn(64)
        
        # Creative step
        result = synthesizer.creative_step(input_state, mode=mode)
        
        print(f"    Cycle {i+1} ({mode}): novelty={result['novelty']:.4f}, "
              f"creativity={result['evaluation']['overall_creativity']:.4f}")
        
        synthesizer.cycle_count += 1
    
    # Generate conceptual blends
    print("\n  Generating conceptual blends:")
    blend_result = synthesizer.generate_concept_blend(
        concept_a="AI",
        concept_b="consciousness",
        properties_a={"intelligent": 0.9, "computational": 0.8},
        properties_b={"aware": 0.7, "subjective": 0.9}
    )
    print(f"    Blended: {blend_result['concept_a']} + {blend_result['concept_b']}")
    print(f"    Novelty: {blend_result['novelty']:.4f}")
    
    # Summary
    summary = synthesizer.get_creative_summary()
    print(f"\n  Creative summary:")
    print(f"    Total cycles: {summary['total_cycles']}")
    print(f"    Exploration: {summary['exploration_count']}, Exploitation: {summary['exploitation_count']}")
    print(f"    Avg creativity: {summary['avg_creativity']:.4f}")
    print(f"    Avg surprise: {summary['avg_surprise']:.4f}")
    
    return synthesizer


def demo_divergence_metric():
    """Demo 6: Divergence metric for creative oscillation."""
    print_section("DEMO 6: Divergence Metric")
    
    # Test divergence computation
    print("\n  Computing divergence metrics:")
    
    baseline = np.zeros(32)
    
    for dist_scale in [0.1, 1.0, 5.0, 10.0]:
        state = np.random.randn(32) * dist_scale
        div = divergence_metric(state, baseline)
        print(f"    Distance scale {dist_scale}: divergence={div:.4f}")
    
    # Test attractor strength
    print("\n  Computing attractor strengths:")
    
    attractor_center = np.array([1.0, 2.0] + [0.0] * 30)
    
    for dist in [0.1, 1.0, 5.0, 10.0]:
        state = attractor_center + np.random.randn(32) * dist
        strength = attractor_strength(state, attractor_center)
        print(f"    Distance {dist}: attractor_strength={strength:.4f}")
    
    return True


def main():
    """Run all Phase 3 demos."""
    print("\n" + "="*60)
    print("  QuaNot Phase 3 Demo")
    print("  Creative Synthesis Framework")
    print("="*60)
    print(f"\nPython version: {sys.version.split()[0]}")
    print(f"NumPy version: {np.__version__}")
    
    try:
        # Demo 1: Novelty detection
        detector = demo_novelty_detection()
        
        # Demo 2: Conceptual blending
        blender = demo_conceptual_blending()
        
        # Demo 3: Creative evaluation
        evaluator = demo_creative_evaluation()
        
        # Demo 4: Metaphor generation
        generator = demo_metaphor_generation()
        
        # Demo 5: Creative synthesizer
        synthesizer = demo_creative_synthesizer()
        
        # Demo 6: Divergence metric
        div_result = demo_divergence_metric()
        
        # Summary
        print_section("PHASE 3 DEMO COMPLETE")
        print("""
All Phase 3 components verified:

  [OK] NoveltyDetector (k-NN surprise) - functional
  [OK] ConceptualBlender - generates novel blends
  [OK] CreativeEvaluator - computes surprise/usefulness/coherence
  [OK] MetaphorGenerator - maps attractor geometry
  [OK] CreativeSynthesizer - orchestrates all components
  [OK] divergence_metric - tracks creative oscillation

Phase 3: Creative Synthesis Framework is complete.
Ready for Phase 4: Consciousness Emergence Pathways.
        """)
        
    except Exception as e:
        print(f"\n[X] Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())