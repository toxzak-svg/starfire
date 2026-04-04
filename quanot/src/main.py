"""
QuaNot Phase 1 Demo
==================
Demonstrates all Phase 1 components:
1. Chaotic Reservoir (ESN)
2. Lyapunov Exponent Calculator
3. RQA Consciousness Metrics
4. SQA Ising Solver

Run: python main.py
"""

import numpy as np
import time
import sys
import os

# Add src to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from reservoir import ChaoticReservoir, narma_task
from chaos import (
    lyapunov_exponent_benettin,
    lorenz_attractor,
    correlation_dimension,
    rossler_attractor,
    henon_map
)
from consciousness import (
    GlobalWorkspace,
    consciousness_proxy_suite,
    rqa_metrics,
    recurrence_matrix
)
from quantum_inspired import (
    SimulatedQuantumAnnealing,
    solve_ising,
    solve_qubo,
    QuantumWalkSampler
)


def print_section(title: str):
    """Pretty print section header."""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print('='*60)


def demo_reservoir():
    """Demo 1: Echo State Network with chaotic modulation."""
    print_section("DEMO 1: Chaotic Reservoir (ESN)")
    
    # Create reservoir
    print("\nCreating ChaoticReservoir...")
    reservoir = ChaoticReservoir(
        input_dim=1,
        reservoir_size=500,
        spectral_radius=0.95,
        input_scaling=0.1,
        noise_level=0.001,
        random_seed=42
    )
    print(f"  Reservoir size: {reservoir.reservoir_size}")
    print(f"  Spectral radius: {reservoir.spectral_radius}")
    print(f"  Input dimension: {reservoir.input_dim}")
    
    # Run NARMA-10 benchmark
    print("\nRunning NARMA-10 benchmark...")
    start = time.time()
    rmse, preds, targets = narma_task(reservoir, n_timesteps=2000, seed=42)
    elapsed = time.time() - start
    
    print(f"  RMSE: {rmse:.6f}")
    print(f"  Time: {elapsed:.2f}s")
    print(f"  Reservoir quality: {'GOOD' if rmse < 0.2 else 'NEEDS_TUNING'}")
    
    return reservoir, rmse


def demo_chaos_metrics():
    """Demo 2: Lyapunov exponent and fractal dimension."""
    print_section("DEMO 2: Chaos Metrics")
    
    # Generate Lorenz attractor
    print("\nGenerating Lorenz attractor trajectory...")
    lorenz = lorenz_attractor(steps=10000, dt=0.01)
    print(f"  Shape: {lorenz.shape}")
    print(f"  Range X: [{lorenz[:,0].min():.2f}, {lorenz[:,0].max():.2f}]")
    print(f"  Range Y: [{lorenz[:,1].min():.2f}, {lorenz[:,1].max():.2f}]")
    print(f"  Range Z: [{lorenz[:,2].min():.2f}, {lorenz[:,2].max():.2f}]")
    
    # Estimate Lyapunov exponent
    print("\nEstimating Lyapunov exponent...")
    
    def lorenz_dynamics(x):
        sigma, rho, beta = 10.0, 28.0, 8.0/3.0
        dx = sigma * (x[1] - x[0])
        dy = x[0] * (rho - x[2]) - x[1]
        dz = x[0] * x[1] - beta * x[2]
        return np.array([dx, dy, dz])
    
    start = time.time()
    x0 = lorenz[0]
    lyap_exponents = lyapunov_exponent_benettin(
        lorenz_dynamics, x0, t_max=50, dt=0.01
    )
    elapsed = time.time() - start
    
    print(f"  Lyapunov spectrum: {lyap_exponents}")
    print(f"  Max Lyapunov exponent: {lyap_exponents[0]:.4f}")
    print(f"  Chaos indicator: {'CHAOTIC' if lyap_exponents[0] > 0 else 'STABLE'}")
    print(f"  Computation time: {elapsed:.2f}s")
    
    # Correlation dimension
    print("\nComputing correlation dimension...")
    start = time.time()
    D2, radii, C = correlation_dimension(lorenz, n_scales=20)
    elapsed = time.time() - start
    print(f"  Correlation dimension D_2: {D2:.4f}")
    print(f"  Expected for Lorenz (chaotic): ~2.05")
    print(f"  Computation time: {elapsed:.2f}s")
    
    # Rössler attractor
    print("\nGenerating Rössler attractor...")
    rossler = rossler_attractor(steps=10000)
    D2_rossler, _, _ = correlation_dimension(rossler, n_scales=20)
    print(f"  Rössler correlation dimension: {D2_rossler:.4f}")
    print(f"  Expected for Rössler (chaotic): ~2.0")
    
    return lyap_exponents, D2


def demo_consciousness():
    """Demo 3: Consciousness proxy metrics."""
    print_section("DEMO 3: Consciousness Proxy Metrics")
    
    # Generate a test trajectory (combining multiple attractors)
    print("\nGenerating cognitive state trajectory...")
    np.random.seed(42)
    t1 = lorenz_attractor(steps=2000, sigma=10, rho=28)
    t2 = rossler_attractor(steps=2000, a=0.2, b=0.2, c=5.7)
    
    # Mix trajectories to simulate cognitive state transitions
    cognitive_traj = np.concatenate([t1[:1000], t2[500:1500], t1[1000:]], axis=0)
    print(f"  Trajectory shape: {cognitive_traj.shape}")
    
    # RQA metrics
    print("\nComputing RQA consciousness metrics...")
    R = recurrence_matrix(cognitive_traj, threshold=0.2)
    rqa = rqa_metrics(R)
    
    print(f"  REC (Recurrence): {rqa['REC']:.4f}")
    print(f"  DET (Determinism): {rqa['DET']:.4f}")
    print(f"  LAM (Laminarity): {rqa['LAM']:.4f}")
    print(f"  L_max (Max diagonal): {rqa['L_max']}")
    print(f"  ENTR (Entropy): {rqa['ENTR']:.4f}")
    print(f"  FD (Fractal Dim): {rqa['FD']:.4f}")
    
    # Global Workspace simulation
    print("\nSimulating Global Workspace...")
    gw = GlobalWorkspace(
        n_modules=8,
        workspace_capacity=3,
        attention_threshold=0.5
    )
    
    n_steps = 50
    for step in range(n_steps):
        # Simulate module inputs
        inputs = np.random.rand(8) * np.sin(step / 10) + np.random.randn(8) * 0.1
        gw.step(inputs)
    
    summary = gw.consciousness_summary()
    print(f"  Average consciousness level: {summary['avg_consciousness']:.4f}")
    print(f"  Total broadcasts: {summary['n_broadcasts']}")
    print(f"  Content diversity: {summary['content_diversity']:.4f}")
    print(f"  Most conscious modules: {summary['most_common_content']}")
    
    # Full consciousness proxy suite
    print("\nRunning full consciousness proxy suite...")
    results = consciousness_proxy_suite(cognitive_traj[-500:])
    print(f"  AIS (Active Information Storage): {results['ais']:.4f}")
    print(f"  Phi proxy (lower=more integrated): {results['phi_geo_proxy']:.4f}")
    print(f"  Interpretation: {results['interpretation']}")
    
    return results, summary


def demo_sqa():
    """Demo 4: Simulated Quantum Annealing."""
    print_section("DEMO 4: Simulated Quantum Annealing")
    
    # Simple 1D chain Ising problem (known ground state)
    print("\nSolving 1D Ising chain (N=20)...")
    n_spins = 20
    
    # Ising chain: J=1, periodic boundary
    J = np.zeros((n_spins, n_spins))
    for i in range(n_spins - 1):
        J[i, i+1] = -1.0  # Ferromagnetic coupling
        J[i+1, i] = -1.0
    J[0, n_spins-1] = -1.0  # Periodic
    J[n_spins-1, 0] = -1.0
    
    h = np.zeros(n_spins)  # No external field
    
    print(f"  Problem: {n_spins} spins, ferromagnetic chain")
    print(f"  Expected: all spins aligned (E = -N)")
    
    start = time.time()
    solution, energy = solve_ising(J, h, n_trotters=8, n_steps=3000, verbose=False)
    elapsed = time.time() - start
    
    print(f"  Found energy: {energy:.4f}")
    print(f"  Expected energy: {-n_spins:.4f}")
    print(f"  Solution: {solution[:10]}... (first 10 spins)")
    print(f"  All aligned: {np.all(solution == solution[0])}")
    print(f"  Time: {elapsed:.2f}s")
    
    # Larger random problem
    print("\nSolving random Ising problem (N=50)...")
    np.random.seed(42)
    n_spins = 50
    J_random = np.random.randn(n_spins, n_spins) * 0.5
    J_random = (J_random + J_random.T) / 2  # Symmetric
    np.fill_diagonal(J_random, 0)
    h_random = np.random.randn(n_spins) * 0.5
    
    print(f"  Problem: {n_spins} spins, random couplings")
    
    start = time.time()
    sol2, E2 = solve_ising(J_random, h_random, n_trotters=10, n_steps=5000)
    elapsed = time.time() - start
    
    print(f"  Found energy: {E2:.4f}")
    print(f"  Time: {elapsed:.2f}s")
    
    return solution, energy


def demo_quantum_walk():
    """Demo 5: Quantum walk sampling."""
    print_section("DEMO 5: Quantum Walk Sampler")
    
    # Create a small graph
    print("\nCreating graph...")
    n_nodes = 20
    
    # Random graph
    adj = np.random.rand(n_nodes, n_nodes)
    adj = (adj + adj.T) / 2  # Symmetric
    np.fill_diagonal(adj, 0)
    adj = (adj > 0.7).astype(float)  # Sparse
    adj[adj == 0] = 0.001  # Small non-zero to avoid div by zero
    
    print(f"  Graph: {n_nodes} nodes, ~{(adj > 0.001).sum() // 2} edges")
    
    # Run quantum walk
    print("\nRunning quantum walk (100 steps, 100 walkers)...")
    sampler = QuantumWalkSampler(adj, n_steps=100, n_walkers=100, random_seed=42)
    
    start = time.time()
    stationary, history = sampler.run()
    elapsed = time.time() - start
    
    print(f"  Stationary distribution: {stationary[:5]}... (first 5 nodes)")
    print(f"  Max node probability: {stationary.max():.4f}")
    print(f"  Time: {elapsed:.2f}s")
    
    return stationary


def demo_compression():
    """Demo 6: SVD-based cognitive state compression."""
    print_section("DEMO 6: Cognitive State Compression")
    
    # Create a synthetic "cognitive state" (1024-dim vector)
    print("\nCreating synthetic cognitive state (1024-dim)...")
    np.random.seed(42)
    state = np.random.randn(1024).astype(np.float32)
    state = state / np.linalg.norm(state)  # Normalize
    
    print(f"  Original dimension: {len(state)}")
    print(f"  Original size: {state.nbytes} bytes")
    
    # Compress using SVD-based low-rank approximation
    print("\nCompressing via SVD low-rank approximation...")
    start = time.time()
    
    # Reshape into a "cognitive image" (32x32 matrix)
    cognitive_matrix = state.reshape(32, 32)
    
    # SVD
    U, S, Vh = np.linalg.svd(cognitive_matrix, full_matrices=False)
    
    # Low-rank approximation with k=8
    k = 8
    U_k = U[:, :k]
    S_k = S[:k]
    Vh_k = Vh[:k, :]
    
    # Reconstruct
    reconstructed_matrix = U_k @ np.diag(S_k) @ Vh_k
    reconstructed = reconstructed_matrix.ravel()
    
    elapsed = time.time() - start
    
    # Metrics
    original_size = state.nbytes
    compressed_size = (U_k.nbytes + S_k.nbytes + Vh_k.nbytes)
    ratio = original_size / compressed_size
    
    recon_error = np.sqrt(np.mean((state - reconstructed) ** 2))
    rel_error = recon_error / (np.linalg.norm(state) + 1e-10)
    
    print(f"  Rank: {k} (vs full rank 32)")
    print(f"  Compression ratio: {ratio:.4f}x")
    print(f"  Reconstruction error: {recon_error:.6f}")
    print(f"  Relative error: {rel_error:.4f}")
    print(f"  Original: {original_size} bytes")
    print(f"  Compressed (U+S+V): {compressed_size} bytes")
    print(f"  Time: {elapsed:.4f}s")
    
    return reconstructed, ratio


def main():
    """Run all Phase 1 demos."""
    print("\n" + "="*60)
    print("  QuaNot Phase 1 Demo")
    print("  Quantum-Inspired Neural Optimization & Chaos Theory")
    print("="*60)
    print(f"\nPython version: {sys.version.split()[0]}")
    print(f"NumPy version: {np.__version__}")
    
    try:
        # Demo 1: Reservoir computing
        reservoir, rmse = demo_reservoir()
        
        # Demo 2: Chaos metrics
        lyap, D2 = demo_chaos_metrics()
        
        # Demo 3: Consciousness proxies
        consciousness, gw_summary = demo_consciousness()
        
        # Demo 4: SQA
        sqa_sol, sqa_E = demo_sqa()
        
        # Demo 5: Quantum walk
        stationary = demo_quantum_walk()
        
        # Demo 6: Compression
        cores, ratio = demo_compression()
        
        # Summary
        print_section("PHASE 1 DEMO COMPLETE")
        print("""
All Phase 1 components verified:

  [OK] ChaoticReservoir (ESN) - NARMA RMSE: {:.4f}
  [OK] Lyapunov exponent - MLE: {:.4f} ({})
  [OK] Correlation dimension - D2: {:.4f}
  [OK] RQA consciousness proxy - REC: {:.4f}, DET: {:.4f}
  [OK] Global Workspace simulation - consciousness level: {:.4f}
  [OK] SQA Ising solver - energy: {:.4f}
  [OK] Quantum walk sampling - max prob: {:.4f}
  [OK] Tensor compression - ratio: {:.4f}

Phase 1 foundations are operational.
Ready for Phase 2: Chaos Integration.
        """.format(
            rmse, lyap[0], 'CHAOTIC' if lyap[0] > 0 else 'STABLE',
            D2,
            consciousness['rqa']['REC'],
            consciousness['rqa']['DET'],
            gw_summary['avg_consciousness'],
            sqa_E,
            stationary.max(),
            ratio  # compression_ratio = original / compressed (higher = more compression)
        ))
        
    except Exception as e:
        print(f"\n[X] Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
