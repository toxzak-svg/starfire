"""
Tests for consciousness_enhanced.py (Phase 4: Consciousness Emergence)
=====================================================================
Phase 4: Enhanced consciousness components

Run: pytest tests/test_consciousness_enhanced.py -v
"""

import numpy as np
import pytest
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from consciousness_enhanced import (
    PhiCalculator,
    MetacognitionLoop,
    PredictiveCodingLayer,
    RecurrentProcessingLoop,
    ConsciousnessCore,
    recurrence_matrix,
    rqa_metrics,
    consciousness_proxy_suite
)


class TestPhiCalculator:
    """Test PhiCalculator (enhanced Φ proxy calculators)."""
    
    @pytest.fixture
    def calculator(self):
        """Create test calculator."""
        np.random.seed(42)
        return PhiCalculator(random_seed=42)
    
    @pytest.fixture
    def structured_states(self):
        """Create structured states (high integration)."""
        np.random.seed(42)
        # Create states where parts are correlated
        base = np.random.randn(200, 4)
        # Make them correlated
        base[:, 1] = base[:, 0] + 0.1 * np.random.randn(200)
        base[:, 3] = 0.5 * base[:, 0] + 0.5 * base[:, 1]
        return base
    
    def test_initialization(self, calculator):
        """Test calculator initializes correctly."""
        assert hasattr(calculator, 'history')
        assert calculator.history == []
    
    def test_geometric_phi(self, calculator, structured_states):
        """Test geometric Φ calculation."""
        phi = calculator.compute_geometric_phi(structured_states, n_partitions=10)
        
        assert np.isfinite(phi)
        assert 0 <= phi <= 1
    
    def test_geometric_phi_single_element(self, calculator):
        """Test geometric Φ with single element."""
        states = np.random.randn(100, 1)
        phi = calculator.compute_geometric_phi(states)
        
        assert phi == 0.0
    
    def test_geometric_phi_1d(self, calculator):
        """Test geometric Φ with 1D array."""
        phi = calculator.compute_geometric_phi(np.array([1, 2, 3, 4, 5]))
        
        assert phi == 0.0
    
    def test_spectral_phi(self, calculator, structured_states):
        """Test spectral Φ calculation."""
        phi = calculator.compute_spectral_phi(structured_states)
        
        assert np.isfinite(phi)
        assert 0 <= phi <= 1
    
    def test_spectral_phi_single_element(self, calculator):
        """Test spectral Φ with single element."""
        states = np.random.randn(100, 1)
        phi = calculator.compute_spectral_phi(states)
        
        assert phi == 0.0
    
    def test_spectral_phi_1d(self, calculator):
        """Test spectral Φ with 1D array."""
        phi = calculator.compute_spectral_phi(np.array([1, 2, 3, 4, 5]))
        
        # Should reshape to 2D
        assert np.isfinite(phi)
    
    def test_information_phi(self, calculator, structured_states):
        """Test information-theoretic Φ."""
        phi = calculator.compute_information_phi(structured_states)
        
        assert np.isfinite(phi)
        assert 0 <= phi <= 1
    
    def test_information_phi_single_element(self, calculator):
        """Test information Φ with single element."""
        states = np.random.randn(100, 1)
        phi = calculator.compute_information_phi(states)
        
        assert phi == 0.0
    
    def test_recurrence_phi(self, calculator):
        """Test recurrence-based Φ."""
        # Create trajectory
        t = np.linspace(0, 20, 300)
        trajectory = np.column_stack([
            np.sin(t),
            np.cos(t),
            np.sin(t) * np.cos(t)
        ])
        
        phi = calculator.compute_recurrence_phi(trajectory)
        
        assert np.isfinite(phi)
        assert 0 <= phi <= 1
    
    def test_recurrence_phi_short(self, calculator):
        """Test recurrence Φ with short trajectory."""
        trajectory = np.random.randn(30, 2)
        
        phi = calculator.compute_recurrence_phi(trajectory)
        
        assert phi == 0.0
    
    def test_compute_all(self, calculator, structured_states):
        """Test computing all Φ proxies."""
        trajectory = np.random.randn(200, 3)
        
        results = calculator.compute_all(structured_states, trajectory)
        
        # Check all keys
        required = ['geometric_phi', 'spectral_phi', 'information_phi', 
                    'recurrence_phi', 'phi_composite', 'interpretation']
        for key in required:
            assert key in results
        
        # Check composite
        assert 0 <= results['phi_composite'] <= 1
        
        # Check interpretation
        assert 'integration_level' in results['interpretation']
        assert 'mechanism' in results['interpretation']
        
        # Check history was updated
        assert len(calculator.history) == 1
    
    def test_interpret_mechanism(self, calculator):
        """Test mechanism interpretation."""
        results = {
            'geometric_phi': 0.5,
            'spectral_phi': 0.3,
            'information_phi': 0.1,
            'recurrence_phi': 0.2
        }
        
        mechanism = calculator._interpret_mechanism(results)
        
        assert mechanism == 'geometric'


class TestMetacognitionLoop:
    """Test MetacognitionLoop (self-monitoring and self-modeling)."""
    
    @pytest.fixture
    def metacog(self):
        """Create test metacognition loop."""
        return MetacognitionLoop(
            state_dim=10,
            memory_size=50,
            learning_rate=0.1,
            confidence_decay=0.95,
            attention_threshold=1.0
        )
    
    def test_initialization(self, metacog):
        """Test metacognition initializes correctly."""
        assert metacog.state_dim == 10
        assert metacog.self_model.shape == (10,)
        assert metacog.self_model_confidence == 0.5
        assert metacog.attention_demands == 0
    
    def test_monitor(self, metacog):
        """Test monitoring cognitive state."""
        cognitive_state = np.random.randn(10)
        
        result = metacog.monitor(cognitive_state)
        
        # Check result keys
        assert 'error' in result
        assert 'confidence' in result
        assert 'rethink_needed' in result
        assert 'insight_detected' in result
        assert 'self_model' in result
        
        # Check values are valid
        assert np.isfinite(result['error'])
        assert 0 <= result['confidence'] <= 1
    
    def test_monitor_updates_self_model(self, metacog):
        """Test self-model gets updated."""
        initial_model = metacog.self_model.copy()
        
        # Give consistent states
        for _ in range(10):
            state = np.ones(10) * 0.5 + np.random.randn(10) * 0.1
            metacog.monitor(state)
        
        # Self-model should have changed
        assert not np.allclose(metacog.self_model, initial_model)
    
    def test_evaluate_confidence(self, metacog):
        """Test confidence evaluation."""
        # Need some history first
        for _ in range(20):
            metacog.monitor(np.random.randn(10))
        
        confidence = metacog.evaluate_confidence()
        
        assert 'level' in confidence
        assert 'trend' in confidence
        assert 'reliability' in confidence
    
    def test_evaluate_confidence_early(self, metacog):
        """Test confidence evaluation with insufficient data."""
        confidence = metacog.evaluate_confidence()
        
        assert confidence['level'] == 'unknown'
        assert confidence['reliability'] == 0.5
    
    def test_need_reevaluation(self, metacog):
        """Test reevaluation check."""
        # Initially should not need reevaluation
        assert not metacog.need_reevaluation()
        
        # Add high error states
        for _ in range(15):
            metacog.monitor(np.random.randn(10) * 5)  # High error
        
        # Should need reevaluation
        assert metacog.need_reevaluation()
    
    def test_get_self_awareness_level(self, metacog):
        """Test self-awareness level."""
        # Need history
        for _ in range(20):
            metacog.monitor(np.random.randn(10))
        
        awareness = metacog.get_self_awareness_level()
        
        assert 0 <= awareness <= 1
    
    def test_get_self_awareness_early(self, metacog):
        """Test self-awareness with insufficient data."""
        awareness = metacog.get_self_awareness_level()
        
        assert awareness == 0.0
    
    def test_reset(self, metacog):
        """Test reset."""
        # Add some data
        for _ in range(10):
            metacog.monitor(np.random.randn(10))
        
        metacog.reset()
        
        assert np.all(metacog.self_model == 0)
        assert metacog.self_model_confidence == 0.5
        assert len(metacog.history) == 0


class TestPredictiveCodingLayer:
    """Test PredictiveCodingLayer (top-down prediction)."""
    
    @pytest.fixture
    def pc_layer(self):
        """Create test predictive coding layer."""
        np.random.seed(42)
        return PredictiveCodingLayer(
            n_neurons=20,
            prediction_learning_rate=0.01,
            error_learning_rate=0.1
        )
    
    def test_initialization(self, pc_layer):
        """Test layer initializes correctly."""
        assert pc_layer.n_neurons == 20
        assert pc_layer.W_predict.shape == (20, 20)
        assert pc_layer.current_prediction.shape == (20,)
        assert pc_layer.current_error.shape == (20,)
    
    def test_forward(self, pc_layer):
        """Test forward pass."""
        input_activity = np.random.randn(20)
        
        result = pc_layer.forward(input_activity)
        
        # Check result keys
        assert 'activity' in result
        assert 'error' in result
        assert 'prediction' in result
        assert 'error_magnitude' in result
        
        # Check shapes
        assert result['activity'].shape == (20,)
        assert result['error'].shape == (20,)
    
    def test_forward_with_top_down(self, pc_layer):
        """Test forward with top-down prediction."""
        input_activity = np.random.randn(20)
        top_down = np.random.randn(20)
        
        result = pc_layer.forward(input_activity, top_down_prediction=top_down)
        
        assert 'error' in result
        assert np.isfinite(result['error_magnitude'])
    
    def test_prediction_error_stats(self, pc_layer):
        """Test prediction error statistics."""
        # Need some history
        for _ in range(20):
            pc_layer.forward(np.random.randn(20))
        
        stats = pc_layer.get_prediction_error_stats()
        
        assert 'mean' in stats
        assert 'std' in stats
        assert 'trend' in stats
        assert 'total_error' in stats
    
    def test_prediction_error_stats_early(self, pc_layer):
        """Test stats with no history."""
        stats = pc_layer.get_prediction_error_stats()
        
        assert stats['mean'] == 0.0
        assert stats['std'] == 0.0
    
    def test_reset(self, pc_layer):
        """Test reset."""
        # Add some data
        for _ in range(10):
            pc_layer.forward(np.random.randn(20))
        
        pc_layer.reset()
        
        assert pc_layer.W_predict.shape == (20, 20)
        assert pc_layer.n_steps == 0
        assert len(pc_layer.predictions) == 0


class TestRecurrentProcessingLoop:
    """Test RecurrentProcessingLoop (feedback loops)."""
    
    @pytest.fixture
    def recurrent_loop(self):
        """Create test recurrent loop."""
        np.random.seed(42)
        return RecurrentProcessingLoop(
            input_dim=10,
            hidden_dim=32,
            n_recurrence_steps=2,
            feedback_strength=0.5
        )
    
    def test_initialization(self, recurrent_loop):
        """Test loop initializes correctly."""
        assert recurrent_loop.input_dim == 10
        assert recurrent_loop.hidden_dim == 32
        assert recurrent_loop.n_recurrence == 2
        assert recurrent_loop.hidden_state.shape == (32,)
    
    def test_process_default(self, recurrent_loop):
        """Test process without return_all_stages."""
        input_vector = np.random.randn(10)
        
        result = recurrent_loop.process(input_vector)
        
        # Check result keys
        assert 'output' in result
        assert 'consciousness_level' in result
        
        # Check shapes
        assert result['output'].shape == (32,)
        assert 0 <= result['consciousness_level'] <= 1
    
    def test_process_all_stages(self, recurrent_loop):
        """Test process with return_all_stages."""
        input_vector = np.random.randn(10)
        
        result = recurrent_loop.process(input_vector, return_all_stages=True)
        
        assert 'stages' in result
        assert 'final_consciousness_level' in result
        
        # Should have 1 feedforward + 2 recurrence stages
        assert len(result['stages']) == 3
        
        # Check stage types
        assert result['stages'][0]['stage'] == 'feedforward'
        assert result['stages'][1]['stage'] == 'recurrence_1'
        assert result['stages'][2]['stage'] == 'recurrence_2'
    
    def test_consciousness_increases_with_recurrence(self, recurrent_loop):
        """Test consciousness level increases with recurrence depth."""
        input_vector = np.random.randn(10)
        
        result = recurrent_loop.process(input_vector, return_all_stages=True)
        
        # Feedforward should have lowest consciousness
        assert result['stages'][0]['consciousness_level'] < result['stages'][-1]['consciousness_level']
    
    def test_get_working_memory_content(self, recurrent_loop):
        """Test getting working memory content."""
        # Process something first
        recurrent_loop.process(np.random.randn(10))
        
        memory = recurrent_loop.get_working_memory_content()
        
        assert memory.shape == (32,)
        assert np.all(np.isfinite(memory))
    
    def test_clear_working_memory(self, recurrent_loop):
        """Test clearing working memory."""
        # Process something first
        recurrent_loop.process(np.random.randn(10))
        
        recurrent_loop.clear_working_memory()
        
        assert np.all(recurrent_loop.hidden_state == 0)
        assert np.all(recurrent_loop.previous_state == 0)
        assert len(recurrent_loop.processing_stages) == 0


class TestConsciousnessCore:
    """Test ConsciousnessCore (integrated orchestration)."""
    
    @pytest.fixture
    def core(self):
        """Create test consciousness core."""
        np.random.seed(42)
        return ConsciousnessCore(
            state_dim=32,
            n_modules=4,
            random_seed=42
        )
    
    def test_initialization(self, core):
        """Test core initializes correctly."""
        assert core.state_dim == 32
        assert core.n_modules == 4
        
        # Check all components exist
        assert hasattr(core, 'phi_calculator')
        assert hasattr(core, 'metacognition')
        assert hasattr(core, 'predictive_coding')
        assert hasattr(core, 'recurrent_loop')
        
        assert core.current_state.shape == (32,)
        assert core.consciousness_level == 0.0
    
    def test_process_early(self, core):
        """Test processing with insufficient history."""
        input_state = np.random.randn(32)
        
        result = core.process(input_state, compute_consciousness=True)
        
        # Should still return something
        assert 'input_state' in result
        assert 'recurrent' in result
        assert 'predictive' in result
        assert 'metacognition' in result
    
    def test_process_with_sufficient_history(self, core):
        """Test processing after collecting history."""
        # Generate enough history
        for _ in range(60):
            core.process(np.random.randn(32))
        
        result = core.process(np.random.randn(32))
        
        # Should now include full metrics
        assert 'phi' in result
        assert 'consciousness_level' in result
        assert result['consciousness_level'] >= 0
    
    def test_get_consciousness_summary(self, core):
        """Test getting consciousness summary."""
        # Need some history
        for _ in range(50):
            core.process(np.random.randn(32))
        
        summary = core.get_consciousness_summary()
        
        # Check required keys
        required = ['consciousness_level', 'phi_composite', 'self_awareness', 
                    'prediction_error', 'confidence', 'integration_interpretation']
        for key in required:
            assert key in summary
    
    def test_get_consciousness_summary_early(self, core):
        """Test summary with insufficient data."""
        summary = core.get_consciousness_summary()
        
        assert summary['status'] == 'insufficient_data'
    
    def test_reset(self, core):
        """Test core reset."""
        # Add some history
        for _ in range(20):
            core.process(np.random.randn(32))
        
        core.reset()
        
        assert np.all(core.current_state == 0)
        assert core.consciousness_level == 0.0
        assert len(core.state_history) == 0


class TestConsciousnessProxySuite:
    """Test backward compatibility function."""
    
    def test_suite_basic(self):
        """Test consciousness proxy suite."""
        trajectory = np.sin(np.linspace(0, 20, 500))
        
        results = consciousness_proxy_suite(trajectory, window_size=200)
        
        # Check all components
        assert 'rqa' in results
        assert 'ais' in results
        assert 'phi_geo_proxy' in results
        assert 'interpretation' in results
    
    def test_suite_2d_trajectory(self):
        """Test suite with 2D trajectory."""
        t = np.linspace(0, 20, 500)
        trajectory = np.column_stack([np.sin(t), np.cos(t)])
        
        results = consciousness_proxy_suite(trajectory, window_size=200)
        
        assert 'rqa' in results
        assert 'ais' in results


class TestIntegration:
    """Integration tests combining multiple components."""
    
    def test_full_consciousness_pipeline(self):
        """Test full pipeline from raw states to consciousness metrics."""
        np.random.seed(42)
        
        # Create core
        core = ConsciousnessCore(state_dim=32, n_modules=8, random_seed=42)
        
        # Generate varied inputs
        for i in range(100):
            # Vary the input patterns
            if i < 30:
                state = np.sin(np.linspace(0, 10, 32)) + np.random.randn(32) * 0.1
            elif i < 60:
                state = np.cos(np.linspace(0, 10, 32)) + np.random.randn(32) * 0.2
            else:
                state = np.random.randn(32) * 0.5
            
            result = core.process(state)
            
            # Should always get output
            assert 'recurrent' in result
            assert 'predictive' in result
            assert 'metacognition' in result
        
        # Get final summary
        summary = core.get_consciousness_summary()
        
        assert 'consciousness_level' in summary
        assert 0 <= summary['consciousness_level'] <= 1
    
    def test_awareness_tracking_over_time(self):
        """Test that awareness changes over time."""
        np.random.seed(42)
        
        core = ConsciousnessCore(state_dim=16, random_seed=42)
        
        awareness_levels = []
        
        # First 50: random noise (low awareness expected)
        for _ in range(50):
            core.process(np.random.randn(16) * 2)
            summary = core.get_consciousness_summary()
            if 'self_awareness' in summary:
                awareness_levels.append(summary['self_awareness'])
        
        # Next 50: structured patterns (higher awareness expected)
        for i in range(50):
            pattern = np.sin(np.linspace(0, 4*np.pi, 16)) * (i/50)
            core.process(pattern)
            summary = core.get_consciousness_summary()
            if 'self_awareness' in summary:
                awareness_levels.append(summary['self_awareness'])
        
        # Should have tracked some awareness levels
        assert len(awareness_levels) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])