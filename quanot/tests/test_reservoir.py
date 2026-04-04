"""
Tests for reservoir.py (ChaoticReservoir, ESN)
========================================
Phase 1: Quantum-Inspired Foundations

Run: pytest tests/test_reservoir.py -v
"""

import numpy as np
import pytest
import sys
import os

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from reservoir import ChaoticReservoir, narma_task, CreativeOscillator


class TestChaoticReservoir:
    """Test ChaoticReservoir (ESN) implementation."""
    
    @pytest.fixture
    def reservoir(self):
        """Create a test reservoir."""
        return ChaoticReservoir(
            input_dim=1,
            reservoir_size=100,
            spectral_radius=0.95,
            input_scaling=0.1,
            noise_level=0.001,
            connectivity=0.1,
            random_seed=42
        )
    
    def test_initialization(self, reservoir):
        """Test reservoir initializes correctly."""
        assert reservoir.input_dim == 1
        assert reservoir.reservoir_size == 100
        assert reservoir.spectral_radius == 0.95
        assert reservoir.state.shape == (100,)
    
    def test_forward_single_step(self, reservoir):
        """Test single step forward pass."""
        input_vec = np.array([0.5])
        new_state = reservoir.forward_step(input_vec)
        
        assert new_state.shape == (100,)
        assert np.all(np.isfinite(new_state))
        assert np.all(np.abs(new_state) <= 1.0)  # tanh bounded
    
    def test_forward_sequence(self, reservoir):
        """Test processing input sequence."""
        input_sequence = np.random.rand(10, 1)
        states = reservoir.forward(input_sequence)
        
        assert states.shape == (10, 100)
        assert np.all(np.isfinite(states))
    
    def test_train_readout(self, reservoir):
        """Test readout training."""
        # Generate training data
        np.random.seed(42)
        inputs = np.random.rand(100, 1) * 0.5
        targets = np.sin(np.arange(100) * 0.1).reshape(-1, 1)
        
        states = reservoir.forward(inputs)
        rmse = reservoir.train_readout(states, targets)
        
        assert rmse < 1.0  # Should learn something
        assert reservoir.W_out is not None
    
    def test_predict(self, reservoir):
        """Test prediction after training."""
        # Train first
        np.random.seed(42)
        inputs = np.random.rand(100, 1) * 0.5
        targets = np.sin(np.arange(100) * 0.1).reshape(-1, 1)
        
        states = reservoir.forward(inputs)
        reservoir.train_readout(states, targets)
        
        # Predict
        predictions = reservoir.predict(states[:10])
        
        assert predictions.shape == (10, 1)
        assert np.all(np.isfinite(predictions))
    
    def test_reset(self, reservoir):
        """Test state reset."""
        # Run forward to change state
        reservoir.forward_step(np.array([0.5]))
        assert not np.all(reservoir.state == 0)
        
        # Reset
        reservoir.reset()
        assert np.all(reservoir.state == 0)
    
    def test_lyapunov_online(self, reservoir):
        """Test online Lyapunov estimation."""
        # Need some state history first
        for _ in range(50):
            reservoir.forward_step(np.array([0.1]))
        
        mle = reservoir.estimate_lyapunov_online(n_timesteps=20)
        
        assert np.isfinite(mle)
    
    def test_regime_detection(self, reservoir):
        """Test dynamical regime detection."""
        regime = reservoir.get_regime()
        
        assert regime in ['stable', 'edge_of_chaos', 'chaotic']


class TestNARMATask:
    """Test NARMA-10 benchmark."""
    
    def test_narma_task_completes(self):
        """Test NARMA-10 task runs without error."""
        reservoir = ChaoticReservoir(
            input_dim=1,
            reservoir_size=100,
            spectral_radius=0.95,
            random_seed=42
        )
        
        rmse, preds, targets = narma_task(
            reservoir, 
            n_timesteps=500,
            seed=42
        )
        
        assert rmse < 1.0
        assert preds.shape == targets.shape
        assert np.all(np.isfinite(preds))
    
    def test_narma_benchmark_quality(self):
        """Test NARMA benchmark meets quality threshold."""
        reservoir = ChaoticReservoir(
            input_dim=1,
            reservoir_size=200,
            spectral_radius=0.95,
            random_seed=42
        )
        
        rmse, preds, targets = narma_task(
            reservoir,
            n_timesteps=1000,
            seed=42
        )
        
        # Good ESN should achieve RMSE < 0.2
        assert rmse < 0.2, f"NARMA RMSE {rmse} exceeds threshold 0.2"


class TestCreativeOscillator:
    """Test CreativeOscillator for creative synthesis."""
    
    @pytest.fixture
    def oscillator(self):
        """Create test oscillator."""
        return CreativeOscillator(
            order_threshold=0.7,
            chaos_threshold=0.3,
            max_exploration_steps=10,
            convergence_rate=0.1,
            random_seed=42
        )
    
    def test_initialization(self, oscillator):
        """Test oscillator initializes correctly."""
        assert oscillator.state == 'ordered'
        assert oscillator.exploration_count == 0
    
    def test_step_converge(self, oscillator):
        """Test convergence step."""
        result = oscillator.step(
            current_value=0.8,
            divergence_metric=0.2
        )
        
        assert 'action' in result
        assert 'new_state' in result
        assert result['new_state'] in ['ordered', 'exploratory']
    
    def test_step_chaos_injection(self, oscillator):
        """Test chaos injection when too ordered."""
        result = oscillator.step(
            current_value=0.5,
            divergence_metric=0.1  # Below chaos_threshold
        )
        
        assert result['new_state'] == 'exploratory'
    
    def test_max_exploration_limit(self, oscillator):
        """Test max exploration limit triggers return."""
        # Force into exploratory state
        oscillator.state = 'exploratory'
        
        # Step many times beyond max
        for i in range(15):
            result = oscillator.step(
                current_value=0.4,
                divergence_metric=0.5
            )
        
        # After max_exploration_steps, should stabilize
        assert oscillator.exploration_count <= oscillator.max_exploration
    
    def test_history_tracking(self, oscillator):
        """Test history is tracked."""
        for _ in range(5):
            oscillator.step(0.5, 0.5)
        
        assert len(oscillator.history) == 5
    
    def test_status(self, oscillator):
        """Test status reporting."""
        oscillator.step(0.7, 0.4)
        status = oscillator.get_status()
        
        assert 'state' in status
        assert 'current_value' in status


if __name__ == "__main__":
    pytest.main([__file__, "-v"])