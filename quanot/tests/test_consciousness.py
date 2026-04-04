"""
Tests for consciousness.py (Consciousness Proxies)
============================================
Phase 1 & 2: Consciousness Emergence

Run: pytest tests/test_consciousness.py -v
"""

import numpy as np
import pytest
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from consciousness import (
    recurrence_matrix,
    rqa_metrics,
    moving_rqa,
    active_information_storage,
    GlobalWorkspace,
    geometric_integrated_information,
    consciousness_proxy_suite
)


class TestRecurrenceMatrix:
    """Test recurrence matrix computation."""
    
    def test_recurrence_matrix_1d(self):
        """Test 1D trajectory."""
        trajectory = np.sin(np.linspace(0, 10, 100))
        R = recurrence_matrix(trajectory, threshold=0.5)
        
        assert R.shape == (100, 100)
        assert R.dtype == np.float32
        assert np.all((R == 0) | (R == 1))
    
    def test_recurrence_matrix_2d(self):
        """Test 2D trajectory."""
        trajectory = np.random.randn(50, 3)
        R = recurrence_matrix(trajectory, threshold=1.0)
        
        assert R.shape == (50, 50)
    
    def test_auto_threshold(self):
        """Test auto threshold computation."""
        trajectory = np.random.randn(100, 2)
        R = recurrence_matrix(trajectory, auto_threshold=True)
        
        # Should compute valid threshold
        assert R.shape == (100, 100)


class TestRQA:
    """Test Recurrence Quantification Analysis."""
    
    def test_rqa_basic(self):
        """Test basic RQA metrics."""
        trajectory = np.sin(np.linspace(0, 20, 200))
        R = recurrence_matrix(trajectory, threshold=0.3)
        metrics = rqa_metrics(R)
        
        # Check all metrics exist
        required = ['REC', 'DET', 'LAM', 'TT', 'L_max', 'ENTR', 'FD']
        for key in required:
            assert key in metrics
        
        # Check values are valid
        assert 0 <= metrics['REC'] <= 1
        assert 0 <= metrics['DET'] <= 1
        assert 0 <= metrics['LAM'] <= 1
    
    def test_rqa_lorenz_like(self):
        """Test RQA on chaotic trajectory."""
        # Create pseudo-Lorenz with recurrence
        t = np.linspace(0, 50, 500)
        x = np.sin(t) * np.exp(-0.1 * t)
        # Add chaotic-like returns
        trajectory = np.column_stack([
            x + 0.1 * np.random.randn(500),
            np.cos(t) + 0.1 * np.random.randn(500),
            x * np.cos(t) + 0.1 * np.random.randn(500)
        ])
        
        R = recurrence_matrix(trajectory, threshold=0.5)
        metrics = rqa_metrics(R)
        
        assert 0 <= metrics['REC'] <= 1
        assert metrics['L_max'] >= 0
    
    def test_moving_rqa(self):
        """Test sliding window RQA."""
        trajectory = np.sin(np.linspace(0, 30, 300))
        
        metrics_series, centers = moving_rqa(
            trajectory, 
            window_size=50, 
            hop_size=25
        )
        
        assert 'REC' in metrics_series
        assert len(centers) > 0


class TestActiveInformationStorage:
    """Test Active Information Storage estimation."""
    
    def test_ais_basic(self):
        """Test AIS on simple time series."""
        t = np.linspace(0, 10, 200)
        # Autoregressive-like process
        trajectory = np.sin(t) + 0.5 * np.sin(0.5 * t)
        
        ais = active_information_storage(trajectory, k=3)
        
        assert np.isfinite(ais)
        assert ais >= 0
    
    def test_ais_random(self):
        """Test AIS on random (low AIS) process."""
        np.random.seed(42)
        trajectory = np.random.randn(200)
        
        ais = active_information_storage(trajectory, k=3)
        
        # Random should have lower AIS than structured
        assert np.isfinite(ais)


class TestGlobalWorkspace:
    """Test Global Workspace simulation."""
    
    @pytest.fixture
    def workspace(self):
        """Create test workspace."""
        return GlobalWorkspace(
            n_modules=4,
            workspace_capacity=2,
            attention_threshold=0.5
        )
    
    def test_initialization(self, workspace):
        """Test workspace initialization."""
        assert workspace.n_modules == 4
        assert workspace.workspace_capacity == 2
    
    def test_step(self, workspace):
        """Test single workspace step."""
        inputs = np.array([0.8, 0.3, 0.5, 0.2])
        
        result = workspace.step(inputs)
        
        assert 'workspace_content' in result
        assert len(result['workspace_content']) <= 2
    
    def test_consciousness_summary(self, workspace):
        """Test consciousness summary."""
        # Run several steps
        for i in range(20):
            inputs = np.random.rand(4) * np.sin(i / 5)
            workspace.step(inputs)
        
        summary = workspace.consciousness_summary()
        
        assert 'avg_consciousness' in summary
        assert 'n_broadcasts' in summary
        assert summary['n_broadcasts'] == 20
    
    def test_get_conscious_attention(self, workspace):
        """Test getting conscious content."""
        inputs = np.array([0.9, 0.1, 0.8, 0.3])
        workspace.step(inputs)
        
        content = workspace.get_conscious_attention()
        
        assert isinstance(content, list)


class TestIntegratedInformation:
    """Test integrated information proxy."""
    
    def test_geometric_phi(self):
        """Test geometric Φ calculation."""
        # Structured states (not independent)
        states = np.random.randn(100, 4)
        states[:, 1] = states[:, 0] + 0.1 * np.random.randn(100)
        states[:, 3] = 0.5 * states[:, 0] + 0.5 * states[:, 1]
        
        phi = geometric_integrated_information(states, n_partitions=5)
        
        assert np.isfinite(phi)
        assert 0 <= phi <= 1
    
    def test_phi_random(self):
        """Test Φ on random states."""
        np.random.seed(42)
        states = np.random.randn(50, 3)
        
        phi = geometric_integrated_information(states, n_partitions=3)
        
        assert np.isfinite(phi)


class TestConsciousnessProxySuite:
    """Test full consciousness proxy suite."""
    
    def test_suite_lorenz(self):
        """Test suite on Lorenz-like trajectory."""
        t = np.linspace(0, 20, 400)
        # Create pseudo-attractor trajectory
        trajectory = np.column_stack([
            np.sin(t) * np.exp(-0.05 * t),
            np.cos(t) * np.exp(-0.05 * t),
            np.sin(t) * np.cos(t)
        ])
        
        results = consciousness_proxy_suite(trajectory, window_size=200)
        
        # Check all components
        assert 'rqa' in results
        assert 'ais' in results
        assert 'phi_geo_proxy' in results
        
        # Check values are valid
        assert results['ais'] >= 0
        assert 0 <= results['phi_geo_proxy'] <= 1
    
    def test_interpretation(self):
        """Test interpretation output."""
        trajectory = np.sin(np.linspace(0, 10, 300))
        
        results = consciousness_proxy_suite(trajectory)
        
        assert 'interpretation' in results
        interp = results['interpretation']
        
        assert 'recurrence' in interp
        assert 'determinism' in interp
        assert interp['recurrence'] in ['high', 'low']


if __name__ == "__main__":
    pytest.main([__file__, "-v"])