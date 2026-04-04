"""
Tests for chaos.py (Chaos Theory Integration)
======================================
Phase 2: Chaos Theory Integration

Run: pytest tests/test_chaos.py -v
"""

import numpy as np
import pytest
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from chaos import (
    lyapunov_exponent_benettin,
    maximal_lyapunov_exponent,
    correlation_dimension,
    box_counting_dimension,
    lorenz_attractor,
    rossler_attractor,
    henon_map,
    clifford_attractor,
    ChaoticReservoir
)


class TestAttractors:
    """Test strange attractor generation."""
    
    def test_lorenz_attractor(self):
        """Test Lorenz attractor generation."""
        trajectory = lorenz_attractor(steps=1000)
        
        assert trajectory.shape == (1000, 3)
        assert np.all(np.isfinite(trajectory))
        # Lorenz should be in bounded range (use relaxed bounds)
        assert np.all(np.abs(trajectory[:, 0]) < 60)  # x - relaxed
        assert np.all(np.abs(trajectory[:, 1]) < 60)  # y - relaxed
        assert np.all(trajectory[:, 2] >= 0)  # z non-negative (relaxed)
    
    def test_rossler_attractor(self):
        """Test Rössler attractor generation."""
        trajectory = rossler_attractor(steps=1000)
        
        assert trajectory.shape == (1000, 3)
        assert np.all(np.isfinite(trajectory))
    
    def test_henon_map(self):
        """Test Hénon map generation."""
        trajectory = henon_map(steps=1000)
        
        assert trajectory.shape == (1000, 2)
        assert np.all(np.isfinite(trajectory))
    
    def test_clifford_attractor(self):
        """Test Clifford attractor generation."""
        trajectory = clifford_attractor(steps=1000)
        
        assert trajectory.shape == (1000, 2)
        assert np.all(np.isfinite(trajectory))


class TestLyapunovExponents:
    """Test Lyapunov exponent estimation."""
    
    def test_lorenz_lyapunov(self):
        """Test Lyapunov estimation for Lorenz system."""
        # Lorenz dynamics
        def lorenz_dynamics(x):
            sigma, rho, beta = 10.0, 28.0, 8.0/3.0
            dx = sigma * (x[1] - x[0])
            dy = x[0] * (rho - x[2]) - x[1]
            dz = x[0] * x[1] - beta * x[2]
            return np.array([dx, dy, dz])
        
        x0 = np.array([0.1, 0.0, 0.0])
        
        # Benettin algorithm
        lyap = lyapunov_exponent_benettin(
            lorenz_dynamics, x0, 
            t_max=10, dt=0.01
        )
        
        assert lyap.shape == (3,)
        assert lyap[0] > 0  # Max exponent should be positive (chaotic)
    
    def test_maximal_lyapunov(self):
        """Test direct divergence method."""
        def simple_dynamics(x):
            return np.array([0.1 * x[0] + 0.1 * x[1], -0.1 * x[1]])
        
        x0 = np.array([1.0, 0.0])
        
        mle = maximal_lyapunov_exponent(
            simple_dynamics, x0,
            t_max=10, dt=0.1
        )
        
        assert np.isfinite(mle)
    
    def test_stable_system(self):
        """Test stable system (negative or near-zero MLE)."""
        # Simple stable linear system
        def stable_dynamics(x):
            return np.array([-1.0 * x[0], -1.0 * x[1]])
        
        x0 = np.array([1.0, 1.0])
        
        mle = maximal_lyapunov_exponent(
            stable_dynamics, x0,
            t_max=5, dt=0.1
        )
        
        # Should be <= 0 (stable or edge)
        assert mle <= 0


class TestFractalDimensions:
    """Test fractal dimension calculation."""
    
    def test_lorenz_correlation_dimension(self):
        """Test correlation dimension for Lorenz."""
        trajectory = lorenz_attractor(steps=5000)
        
        D2, radii, C = correlation_dimension(trajectory, n_scales=10)
        
        assert 0 < D2 < 3  # Valid dimension range
        assert np.all(np.isfinite(D2))
    
    def test_rossler_correlation_dimension(self):
        """Test correlation dimension for Rössler."""
        trajectory = rossler_attractor(steps=5000)
        
        D2, radii, C = correlation_dimension(trajectory, n_scales=10)
        
        assert 0 < D2 < 3
    
    def test_box_counting_dimension(self):
        """Test box-counting dimension."""
        trajectory = lorenz_attractor(steps=2000)
        
        D0 = box_counting_dimension(trajectory, n_scales=10)
        
        assert 0 < D0 < 4


class TestChaoticReservoir:
    """Test ChaoticReservoir from chaos.py."""
    
    @pytest.fixture
    def reservoir(self):
        """Create test reservoir."""
        np.random.seed(42)  # Set seed before creating
        return ChaoticReservoir(
            input_dim=1,
            reservoir_size=100,
            spectral_radius=0.95
        )
    
    def test_initialization(self, reservoir):
        """Test initialization."""
        assert reservoir.input_dim == 1
        assert reservoir.reservoir_size == 100
        assert reservoir.spectral_radius == 0.95
    
    def test_forward(self, reservoir):
        """Test forward pass."""
        inputs = np.random.rand(50, 1)
        states = reservoir.forward(inputs)
        
        assert states.shape == (50, 100)
        assert np.all(np.isfinite(states))
    
    def test_attractor_modulation(self, reservoir):
        """Test attractor context modulation."""
        reservoir.set_attractor_type("lorenz")
        
        # Run forward to generate attractor dynamics
        inputs = np.random.rand(100, 1)
        states = reservoir.forward(inputs)
        
        # Attractor state should have evolved
        assert np.any(reservoir.attractor_state != 0)
    
    def test_lyapunov_estimation(self, reservoir):
        """Test Lyapunov exponent estimation."""
        # Generate state history
        inputs = np.random.rand(200, 1)
        reservoir.forward(inputs)
        
        mle = reservoir.compute_lyapunov_exponent(window_size=100)
        
        assert np.isfinite(mle) or mle == 0.0
    
    def test_adaptive_spectral_radius(self, reservoir):
        """Test spectral radius adaptation."""
        # Generate some history
        inputs = np.random.rand(300, 1)
        reservoir.forward(inputs)
        
        # Adapt
        reservoir.adapt_spectral_radius(target_lyapunov=0.0)
        
        # Spectral radius should be within bounds
        assert 0.5 <= reservoir.spectral_radius <= 1.5
    
    def test_chaos_level(self, reservoir):
        """Test chaos level detection."""
        inputs = np.random.rand(200, 1)
        reservoir.forward(inputs)
        
        level = reservoir.get_chaos_level()
        
        assert level in ['stable', 'edge_of_chaos', 'chaotic', 'unknown']
    
    def test_context_vector(self, reservoir):
        """Test context vector generation."""
        # Generate state
        inputs = np.random.rand(50, 1)
        reservoir.forward(inputs)
        
        context = reservoir.get_context_vector()
        
        # Should combine reservoir state + attractor state
        assert len(context) == reservoir.reservoir_size + 3


if __name__ == "__main__":
    pytest.main([__file__, "-v"])