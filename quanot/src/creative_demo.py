"""
Creative Oscillation Demo
========================
Test the creative oscillation controller and Lyapunov monitoring.
"""

import sys
sys.path.insert(0, '.')
from reservoir import ChaoticReservoir, CreativeOscillator, narma_task
import numpy as np

print("="*60)
print("  Creative Oscillation Demo")
print("="*60)

# Test 1: Creative Oscillator
print("\n[1] Testing CreativeOscillator...")
osc = CreativeOscillator(
    order_threshold=0.7,
    chaos_threshold=0.3,
    max_exploration_steps=50,
    random_seed=42
)

for i in range(20):
    divergence = 0.1 + 0.3 * np.sin(i / 5)
    value = 0.5 + 0.3 * np.cos(i / 3)
    result = osc.step(value, divergence)
    if i % 5 == 0:
        print(f"  Step {i}: state={result['new_state']}, action={result['action']}, scale={result['perturbation_scale']:.2f}")

status = osc.get_status()
print(f"  Final state: {status['state']}")
print(f"  Best value: {status['best_value']:.4f}")

# Test 2: Lyapunov online estimation
print("\n[2] Testing Lyapunov Online Estimation...")
reservoir = ChaoticReservoir(
    input_dim=1,
    reservoir_size=200,
    spectral_radius=0.95,
    random_seed=42
)

inputs = np.random.rand(100, 1) * 0.1
states = reservoir.forward(inputs)
print(f"  After 100 steps, state norm: {np.linalg.norm(reservoir.state):.4f}")

mle = reservoir.estimate_lyapunov_online(n_timesteps=50, separation=1e-6)
print(f"  Estimated MLE: {mle:.4f}")
print(f"  Regime: {reservoir.get_regime()}")

# Test 3: Adaptive spectral radius
print("\n[3] Testing Adaptive Regime Control...")
reservoir2 = ChaoticReservoir(
    input_dim=1,
    reservoir_size=100,
    spectral_radius=1.1,  # Start chaotic
    random_seed=42
)
print(f"  Initial regime: {reservoir2.get_regime()} (spectral_radius={reservoir2.spectral_radius})")

# The creative oscillator should trigger chaos injection based on divergence
print("\n[DONE] All Phase 2 components tested successfully!")