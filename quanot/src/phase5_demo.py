"""
Phase 5 Demo: AGI Architecture Integration
============================================
Phase 5: AGI Architecture Integration

This demo verifies the unified AGI architecture combining all phases:
- Quantum-inspired encoding
- Chaotic reservoir
- Creative synthesis
- Consciousness core
- World model
- Multi-modal integration

Run: python src/phase5_demo.py
"""

import numpy as np
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '.'))

from agi_core import AGISystem, AGICore, create_agi_system


def test_basic_processing():
    """Test basic AGI processing."""
    print("\n" + "=" * 50)
    print("Test 1: Basic Processing")
    print("=" * 50)
    
    agi = create_agi_system(state_dim=64, reservoir_size=128, random_seed=42)
    
    # Single input
    input_data = np.random.randn(64)
    result = agi.run({'default': input_data})
    
    print(f"  Input shape: {input_data.shape}")
    print(f"  Cycles completed: {result['status']['cycle_count']}")
    print(f"  Creativity: {result['creative']['creativity']:.4f}")
    print(f"  Novelty: {result['novelty']:.4f}")
    print(f"  Consciousness: {result['consciousness']['consciousness_level']:.4f}")
    print(f"  Reservoir regime: {result['reservoir']['regime']}")
    
    return result['status']['cycle_count'] == 1


def test_multi_modal():
    """Test multi-modal integration."""
    print("\n" + "=" * 50)
    print("Test 2: Multi-Modal Integration")
    print("=" * 50)
    
    agi = create_agi_system(state_dim=64, random_seed=42)
    
    # Different modalities - process each separately
    vision_input = np.random.randn(64)
    language_input = np.random.randn(64)
    sensor_input = np.random.randn(64)
    
    # Process each modality
    agi.core.process(vision_input, modality='vision')
    agi.core.process(language_input, modality='language')
    agi.core.process(sensor_input, modality='sensor')
    
    status = agi.get_system_info()
    modalities = status['status']['modalities']
    
    print(f"  Modalities stored: {modalities}")
    print(f"  Cycles: {status['status']['cycle_count']}")
    
    # The modalities list should contain the three we processed
    return len(modalities) >= 3


def test_goal_orientation():
    """Test goal-oriented reasoning."""
    print("\n" + "=" * 50)
    print("Test 3: Goal-Oriented Reasoning")
    print("=" * 50)
    
    agi = create_agi_system(state_dim=64, random_seed=42)
    
    # Set a goal
    goal = {
        'description': 'reach_target_state',
        'target': np.random.randn(64),
        'priority': 0.8,
        'deadline': 10
    }
    
    # Process with goal
    for i in range(5):
        result = agi.run({'default': np.random.randn(64)}, goal=goal)
    
    print(f"  Goal: {result['world']['goal_progress']['description']}")
    print(f"  Progress: {result['world']['goal_progress']['progress']:.4f}")
    print(f"  Active goals: {result['world']['goal_progress']['n_goals']}")
    print(f"  Total cycles: {result['status']['metrics']['total_cycles']}")
    
    return result['world']['goal_progress']['status'] == 'active'


def test_continuous_learning():
    """Test continuous learning."""
    print("\n" + "=" * 50)
    print("Test 4: Continuous Learning")
    print("=" * 50)
    
    agi = create_agi_system(state_dim=64, random_seed=42)
    
    # Run many cycles
    for i in range(20):
        result = agi.run({'default': np.random.randn(64)})
    
    status = agi.get_system_info()
    metrics = status['status']['metrics']
    
    print(f"  Total cycles: {metrics['total_cycles']}")
    print(f"  Avg creativity: {metrics['avg_creativity']:.4f}")
    print(f"  Avg consciousness: {metrics['avg_consciousness']:.4f}")
    print(f"  Avg novelty: {metrics['avg_novelty']:.4f}")
    
    return metrics['total_cycles'] == 20


def test_core_components():
    """Test individual AGI core components."""
    print("\n" + "=" * 50)
    print("Test 5: Core Components")
    print("=" * 50)
    
    core = AGICore(state_dim=32, random_seed=42)
    
    # Process input
    input_state = np.random.randn(32)
    result = core.process(input_state)
    
    print(f"  State dimension: {core.state_dim}")
    print(f"  Reservoir size: {core.reservoir_size}")
    print(f"  Cycle count: {core.cycle_count}")
    print(f"  Consciousness level: {result['consciousness']['consciousness_level']:.4f}")
    
    # Check components exist
    has_components = (
        hasattr(core, 'sqa') and
        hasattr(core, 'reservoir') and
        hasattr(core, 'creative_synthesizer') and
        hasattr(core, 'consciousness_core') and
        hasattr(core, 'world_model') and
        hasattr(core, 'goal_manager')
    )
    
    print(f"  All components initialized: {has_components}")
    
    return has_components


def test_world_model():
    """Test world model functionality."""
    print("\n" + "=" * 50)
    print("Test 6: World Model")
    print("=" * 50)
    
    from agi_core import WorldModel, GoalManager
    
    # Test World Model
    wm = WorldModel(state_dim=32)
    
    # Update with states
    for _ in range(10):
        state = np.random.randn(32)
        wm.update(state)
    
    prediction = wm.predict()
    state_info = wm.get_state()
    
    print(f"  History size: {state_info['history_size']}")
    print(f"  Prediction shape: {prediction.shape}")
    print(f"  Transition norm: {state_info['transition_norm']:.4f}")
    
    # Test Goal Manager
    gm = GoalManager(state_dim=32)
    
    goal = {
        'description': 'test_goal',
        'target': np.random.randn(32),
        'priority': 0.7
    }
    gm.set_goal(goal)
    
    # Update progress
    current = np.random.randn(32)
    gm.update_progress(current)
    
    progress = gm.get_progress()
    print(f"  Goal status: {progress['status']}")
    print(f"  Goal description: {progress['description']}")
    
    return state_info['history_size'] == 10 and progress['status'] == 'active'


def test_system_reset():
    """Test system reset functionality."""
    print("\n" + "=" * 50)
    print("Test 7: System Reset")
    print("=" * 50)
    
    agi = create_agi_system(state_dim=32, random_seed=42)
    
    # Run some cycles
    for _ in range(5):
        agi.run({'default': np.random.randn(32)})
    
    status_before = agi.get_system_info()
    print(f"  Cycles before reset: {status_before['status']['cycle_count']}")
    
    # Reset
    agi.reset()
    
    status_after = agi.get_system_info()
    print(f"  Cycles after reset: {status_after['status']['cycle_count']}")
    
    return status_before['status']['cycle_count'] == 5 and status_after['status']['cycle_count'] == 0


def run_full_demo():
    """Run the complete Phase 5 demo."""
    print("\n" + "=" * 60)
    print("QuaNot Phase 5: AGI Architecture Integration")
    print("=" * 60)
    
    tests = [
        ("Basic Processing", test_basic_processing),
        ("Multi-Modal Integration", test_multi_modal),
        ("Goal-Oriented Reasoning", test_goal_orientation),
        ("Continuous Learning", test_continuous_learning),
        ("Core Components", test_core_components),
        ("World Model", test_world_model),
        ("System Reset", test_system_reset),
    ]
    
    results = []
    for name, test_fn in tests:
        try:
            passed = test_fn()
            results.append((name, "PASS" if passed else "FAIL", passed))
        except Exception as e:
            results.append((name, f"ERROR: {str(e)}", False))
            print(f"  ERROR: {e}")
    
    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    
    passed_count = 0
    for name, status, passed in results:
        symbol = "PASS" if passed else "FAIL"
        print(f"  [{symbol}] {name}: {status}")
        if passed:
            passed_count += 1
    
    print(f"\nTotal: {passed_count}/{len(results)} tests passed")
    
    if passed_count == len(results):
        print("\n[SUCCESS] Phase 5 AGI Core verified successfully!")
    else:
        print(f"\n[WARNING] {len(results) - passed_count} tests failed")
    
    return passed_count == len(results)


if __name__ == "__main__":
    success = run_full_demo()
    sys.exit(0 if success else 1)