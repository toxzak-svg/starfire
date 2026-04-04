"""
Tests for creativity.py (Phase 3: Creative Synthesis)
=======================================================
Phase 3: Creative components

Run: pytest tests/test_creativity.py -v
"""

import numpy as np
import pytest
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from creativity import (
    NoveltyDetector,
    ConceptualBlender,
    CreativeEvaluator,
    MetaphorGenerator,
    CreativeSynthesizer,
    divergence_metric,
    attractor_strength
)


class TestNoveltyDetector:
    """Test NoveltyDetector (k-NN surprise metric)."""
    
    @pytest.fixture
    def detector(self):
        """Create test detector."""
        return NoveltyDetector(
            state_dim=10,
            history_size=100,
            k=5,
            novelty_threshold=0.3,
            learning_rate=0.01
        )
    
    def test_initialization(self, detector):
        """Test detector initializes correctly."""
        assert detector.state_dim == 10
        assert detector.k == 5
        assert detector.novelty_threshold == 0.3
        assert len(detector.history) == 0
    
    def test_add_state(self, detector):
        """Test adding states to history."""
        state = np.random.randn(10)
        detector.add_state(state)
        
        assert len(detector.history) == 1
    
    def test_add_state_wrong_dim(self, detector):
        """Test adding state with wrong dimension."""
        with pytest.raises(ValueError):
            detector.add_state(np.random.randn(5))
    
    def test_compute_novelty_empty_history(self, detector):
        """Test novelty with empty history."""
        state = np.random.randn(10)
        novelty = detector.compute_novelty(state)
        
        # Should return 1.0 (maximally novel) for empty history
        assert novelty == 1.0
    
    def test_compute_novelty_with_history(self, detector):
        """Test novelty with existing history."""
        # Add some states to history
        for _ in range(50):
            detector.add_state(np.random.randn(10))
        
        # New state should have some novelty
        new_state = np.random.randn(10)
        novelty = detector.compute_novelty(new_state)
        
        assert 0 <= novelty <= 1
    
    def test_is_novel(self, detector):
        """Test novelty check."""
        # Add history first
        for _ in range(30):
            detector.compute_novelty(np.random.randn(10))
        
        # Should have some novel and some not novel states
        state1 = np.random.randn(10) * 0.01  # Close to history (not novel)
        state2 = np.random.randn(10) * 10    # Far from history (novel)
        
        result1 = detector.is_novel(state1)
        result2 = detector.is_novel(state2)
        
        # At least one should be different
        assert isinstance(result1, bool)
        assert isinstance(result2, bool)
    
    def test_get_novelty_trend(self, detector):
        """Test novelty trend computation."""
        # Add some novelty evaluations
        for i in range(60):
            # Vary novelty
            noise = 0.1 + 0.01 * i
            detector.compute_novelty(np.random.randn(10) * noise)
        
        trend = detector.get_novelty_trend(window=50)
        
        assert np.isfinite(trend)
    
    def test_reset(self, detector):
        """Test reset."""
        # Add some history
        for _ in range(20):
            detector.compute_novelty(np.random.randn(10))
        
        detector.reset()
        
        assert len(detector.history) == 0
        assert detector.cumulative_novelty == 0.0


class TestConceptualBlender:
    """Test ConceptualBlender (Fauconnier & Turner framework)."""
    
    @pytest.fixture
    def blender(self):
        """Create test blender."""
        np.random.seed(42)
        return ConceptualBlender(
            embedding_dim=32,
            blend_temperature=0.5,
            random_seed=42
        )
    
    def test_initialization(self, blender):
        """Test blender initializes correctly."""
        assert blender.embedding_dim == 32
        assert blender.blend_temperature == 0.5
        assert len(blender.blend_history) == 0
    
    def test_create_concept_embedding(self, blender):
        """Test concept embedding creation."""
        embedding = blender.create_concept_embedding("test_concept")
        
        assert embedding.shape == (32,)
        assert np.all(np.isfinite(embedding))
        # Should be normalized
        assert np.isclose(np.linalg.norm(embedding), 1.0, atol=0.01)
    
    def test_create_concept_with_properties(self, blender):
        """Test embedding with properties."""
        properties = {'size': 0.5, 'color': 0.3}
        embedding = blender.create_concept_embedding("test", properties)
        
        assert embedding.shape == (32,)
        assert np.any(embedding != blender.create_concept_embedding("test"))
    
    def test_blend(self, blender):
        """Test blending two concepts."""
        concept_a = np.random.randn(32)
        concept_a = concept_a / np.linalg.norm(concept_a)
        
        concept_b = np.random.randn(32)
        concept_b = concept_b / np.linalg.norm(concept_b)
        
        result = blender.blend(concept_a, concept_b, blend_ratio=0.5)
        
        # Check result keys
        assert 'blended' in result
        assert 'linear' in result
        assert 'emergent' in result
        assert 'novelty' in result
        
        # Check shapes
        assert result['blended'].shape == (32,)
        assert result['linear'].shape == (32,)
        assert result['emergent'].shape == (32,)
    
    def test_blend_wrong_dim(self, blender):
        """Test blending with wrong dimensions."""
        concept_a = np.random.randn(16)  # Wrong dim
        concept_b = np.random.randn(32)
        
        with pytest.raises(ValueError):
            blender.blend(concept_a, concept_b)
    
    def test_run_blending_chain(self, blender):
        """Test blending chain."""
        concepts = [np.random.randn(32) for _ in range(5)]
        concepts = [c / np.linalg.norm(c) for c in concepts]
        
        final = blender.run_blending_chain(concepts, n_blends=3)
        
        assert final.shape == (32,)
        assert len(blender.blend_history) == 3
    
    def test_run_blending_chain_error(self, blender):
        """Test blending chain with insufficient concepts."""
        concepts = [np.random.randn(32)]
        
        with pytest.raises(ValueError):
            blender.run_blending_chain(concepts)


class TestCreativeEvaluator:
    """Test CreativeEvaluator (surprise, usefulness, coherence)."""
    
    @pytest.fixture
    def evaluator(self):
        """Create test evaluator."""
        return CreativeEvaluator(
            surprise_weight=0.4,
            usefulness_weight=0.3,
            coherence_weight=0.3,
            random_seed=42
        )
    
    def test_initialization(self, evaluator):
        """Test evaluator initializes correctly."""
        assert evaluator.surprise_weight == 0.4
        assert evaluator.usefulness_weight == 0.3
        assert evaluator.coherence_weight == 0.3
        assert len(evaluator.evaluation_history) == 0
    
    def test_evaluate_without_references(self, evaluator):
        """Test evaluation without reference outputs."""
        output = np.random.randn(32)
        
        result = evaluator.evaluate(output)
        
        # Check all keys
        assert 'surprise' in result
        assert 'usefulness' in result
        assert 'coherence' in result
        assert 'overall_creativity' in result
        assert 'evaluation' in result
        
        # Check values in range
        assert 0 <= result['surprise'] <= 1
        assert 0 <= result['usefulness'] <= 1
        assert 0 <= result['coherence'] <= 1
        assert 0 <= result['overall_creativity'] <= 1
    
    def test_evaluate_with_references(self, evaluator):
        """Test evaluation with reference outputs."""
        output = np.random.randn(32)
        references = [np.random.randn(32) for _ in range(5)]
        
        result = evaluator.evaluate(output, reference_outputs=references)
        
        assert 0 <= result['surprise'] <= 1
    
    def test_evaluate_with_context(self, evaluator):
        """Test evaluation with context."""
        output = np.random.randn(32)
        context = {'goals': ['goal1', 'goal2', 'goal3']}
        
        result = evaluator.evaluate(output, context=context)
        
        assert 0 <= result['usefulness'] <= 1
    
    def test_interpret_score(self, evaluator):
        """Test score interpretation."""
        assert evaluator._interpret_score(0.1) == "low_creativity"
        assert evaluator._interpret_score(0.3) == "moderate_creativity"
        assert evaluator._interpret_score(0.5) == "good_creativity"
        assert evaluator._interpret_score(0.7) == "high_creativity"
        assert evaluator._interpret_score(0.9) == "exceptional_creativity"
    
    def test_get_average_scores(self, evaluator):
        """Test average scores computation."""
        # Add some evaluations
        for _ in range(20):
            evaluator.evaluate(np.random.randn(32))
        
        avg = evaluator.get_average_scores(window=10)
        
        assert 'surprise' in avg
        assert 'usefulness' in avg
        assert 'coherence' in avg
        assert 'overall' in avg
    
    def test_get_average_scores_empty(self, evaluator):
        """Test average scores with no history."""
        avg = evaluator.get_average_scores()
        
        assert avg['surprise'] == 0
        assert avg['overall'] == 0


class TestMetaphorGenerator:
    """Test MetaphorGenerator (attractor-based metaphors)."""
    
    @pytest.fixture
    def generator(self):
        """Create test generator."""
        np.random.seed(42)
        return MetaphorGenerator(
            attractor_type="lorenz",
            trajectory_length=100,
            random_seed=42
        )
    
    def test_initialization(self, generator):
        """Test generator initializes correctly."""
        assert generator.attractor_type == "lorenz"
        assert generator.trajectory_length == 100
        assert generator.base_trajectory.shape[0] == 100
        assert len(generator.mappings) == 0
    
    def test_lorenz_attractor(self, generator):
        """Test Lorenz attractor."""
        assert generator.base_trajectory.shape[1] == 3
    
    def test_rossler_attractor(self):
        """Test Rössler attractor."""
        np.random.seed(42)
        gen = MetaphorGenerator(attractor_type="rossler", trajectory_length=100)
        
        assert gen.base_trajectory.shape[1] == 3
    
    def test_henon_attractor(self):
        """Test Hénon attractor."""
        np.random.seed(42)
        gen = MetaphorGenerator(attractor_type="henon", trajectory_length=100)
        
        assert gen.base_trajectory.shape[1] == 3
    
    def test_generate_metaphor(self, generator):
        """Test metaphor generation."""
        mapping = generator.generate_metaphor(
            source_concept="freedom",
            target_domain="emotion",
            mapping_strength=0.7
        )
        
        # Check keys
        assert 'source' in mapping
        assert 'target' in mapping
        assert 'attractor_type' in mapping
        assert 'metaphor' in mapping
        assert 'structure' in mapping
        assert 'dynamics' in mapping
        assert 'mapping_strength' in mapping
        
        assert mapping['source'] == "freedom"
        assert mapping['target'] == "emotion"
        assert mapping['mapping_strength'] == 0.7
    
    def test_get_attractor_properties(self, generator):
        """Test getting attractor properties."""
        props = generator.get_attractor_properties()
        
        assert 'type' in props
        assert 'length' in props
        assert 'dimensionality' in props
        assert 'range_x' in props
        assert 'range_y' in props


class TestCreativeSynthesizer:
    """Test CreativeSynthesizer (orchestrator)."""
    
    @pytest.fixture
    def synthesizer(self):
        """Create test synthesizer."""
        np.random.seed(42)
        return CreativeSynthesizer(
            state_dim=32,
            creative_temperature=0.5,
            random_seed=42
        )
    
    def test_initialization(self, synthesizer):
        """Test synthesizer initializes correctly."""
        assert synthesizer.state_dim == 32
        assert synthesizer.creative_temperature == 0.5
        
        # Check components exist
        assert hasattr(synthesizer, 'novelty_detector')
        assert hasattr(synthesizer, 'conceptual_blender')
        assert hasattr(synthesizer, 'creative_evaluator')
        assert hasattr(synthesizer, 'metaphor_generator')
    
    def test_creative_step_explore(self, synthesizer):
        """Test creative step in explore mode."""
        input_state = np.random.randn(32)
        
        result = synthesizer.creative_step(input_state, mode='explore')
        
        # Check keys
        assert 'output' in result
        assert 'novelty' in result
        assert 'evaluation' in result
        assert 'mode' in result
        
        assert result['output'].shape == (32,)
        assert result['mode'] == 'explore'
        assert synthesizer.exploration_count == 1
    
    def test_creative_step_exploit(self, synthesizer):
        """Test creative step in exploit mode."""
        # Need some history for exploitation
        for _ in range(5):
            synthesizer.creative_step(np.random.randn(32), mode='explore')
        
        input_state = np.random.randn(32)
        result = synthesizer.creative_step(input_state, mode='exploit')
        
        assert result['mode'] == 'exploit'
        assert synthesizer.exploitation_count == 1
    
    def test_generate_concept_blend(self, synthesizer):
        """Test concept blend generation."""
        result = synthesizer.generate_concept_blend(
            concept_a="fire",
            concept_b="water",
            properties_a={'hot': 0.8},
            properties_b={'cold': 0.6}
        )
        
        assert 'concept_a' in result
        assert 'concept_b' in result
        assert 'blended_embedding' in result
        assert 'novelty' in result
        
        assert result['concept_a'] == "fire"
        assert result['concept_b'] == "water"
    
    def test_get_creative_summary(self, synthesizer):
        """Test getting creative summary."""
        # Add some creative cycles
        for _ in range(10):
            synthesizer.creative_step(np.random.randn(32), mode='explore')
        
        summary = synthesizer.get_creative_summary()
        
        assert 'total_cycles' in summary
        assert 'exploration_count' in summary
        assert 'exploitation_count' in summary
        assert 'avg_surprise' in summary
        assert 'avg_creativity' in summary
    
    def test_reset(self, synthesizer):
        """Test reset."""
        # Add some history
        for _ in range(10):
            synthesizer.creative_step(np.random.randn(32))
        
        synthesizer.reset()
        
        assert np.all(synthesizer.current_state == 0)
        assert synthesizer.exploration_count == 0
        assert synthesizer.exploitation_count == 0


class TestHelperFunctions:
    """Test helper functions."""
    
    def test_divergence_metric(self):
        """Test divergence metric."""
        state = np.array([1.0, 2.0, 3.0])
        baseline = np.array([1.0, 2.0, 3.0])
        
        div = divergence_metric(state, baseline)
        
        assert div == 0.0
        
        # Different states
        state2 = np.array([10.0, 20.0, 30.0])
        div2 = divergence_metric(state2, baseline)
        
        assert 0 <= div2 <= 1
    
    def test_attractor_strength(self):
        """Test attractor strength."""
        state = np.array([0.0, 0.0, 0.0])
        center = np.array([0.0, 0.0, 0.0])
        
        strength = attractor_strength(state, center)
        
        assert strength == 1.0  # Same position = max strength
        
        # Different positions
        state2 = np.array([10.0, 10.0, 10.0])
        strength2 = attractor_strength(state2, center)
        
        assert 0 <= strength2 <= 1


class TestIntegration:
    """Integration tests."""
    
    def test_full_creative_pipeline(self):
        """Test full creative pipeline."""
        np.random.seed(42)
        
        synthesizer = CreativeSynthesizer(state_dim=32, random_seed=42)
        
        # Run several creative cycles
        for i in range(20):
            input_state = np.random.randn(32)
            mode = 'explore' if i % 2 == 0 else 'exploit'
            
            result = synthesizer.creative_step(input_state, mode=mode)
            
            assert 'output' in result
            assert 'novelty' in result
            assert 'evaluation' in result
        
        # Get summary
        summary = synthesizer.get_creative_summary()
        
        assert summary['total_cycles'] == 20
        assert summary['exploration_count'] > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])