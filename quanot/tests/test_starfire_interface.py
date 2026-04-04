"""
Tests for Starfire Interface
==========================
"""

import pytest
import numpy as np
from dataclasses import asdict

from src.starfire_interface import (
    BridgeMessage,
    StarfireBridge,
    create_bridge,
    compute_consciousness_proxy,
    compute_novelty_score,
    compute_creativity_scores
)
from src.reservoir import ChaoticReservoir


class TestBridgeMessage:
    """Test BridgeMessage serialization."""
    
    def test_create_message(self):
        msg = BridgeMessage(
            message_id="test-123",
            timestamp=1234567890.0,
            input_text="Hello world",
            reservoir_state=[0.1] * 100,
            consciousness_proxy=0.75,
            novelty_score=0.3
        )
        assert msg.message_id == "test-123"
        assert msg.input_text == "Hello world"
        assert len(msg.reservoir_state) == 100
    
    def test_serialize_json(self):
        msg = BridgeMessage(
            message_id="test-456",
            timestamp=1234567890.0,
            input_text="Test",
            consciousness_proxy=0.5
        )
        json_str = msg.to_json()
        assert "test-456" in json_str
        assert "Test" in json_str
    
    def test_deserialize_json(self):
        original = BridgeMessage(
            message_id="test-789",
            timestamp=1234567890.0,
            input_text="Deserialized",
            confidence=0.8
        )
        json_str = original.to_json()
        restored = BridgeMessage.from_json(json_str)
        
        assert restored.message_id == "test-789"
        assert restored.input_text == "Deserialized"
        assert restored.confidence == 0.8


class TestStarfireBridge:
    """Test StarfireBridge functionality."""
    
    def test_bridge_initialization(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100,
            random_seed=42
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire',
            mode='stdio'
        )
        
        assert bridge.reservoir is not None
        assert bridge.oscillator is not None
        assert bridge.state_history == []
    
    def test_encode_for_starfire(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100,
            random_seed=42
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire'
        )
        
        # Initialize reservoir with some states
        for _ in range(10):
            state = np.random.randn(64)
            reservoir.forward_step(state)
        
        message = bridge.encode_for_starfire("Test input")
        
        assert message.input_text == "Test input"
        assert message.reservoir_state is not None
        assert len(message.reservoir_state) == 100
        assert 0 <= message.consciousness_proxy <= 1
    
    def test_divergence_computation(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100,
            random_seed=42
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire'
        )
        
        # Add various states
        for i in range(20):
            state = np.random.randn(64) * (i + 1) * 0.1
            bridge.state_history.append(state)
        
        divergence = bridge._compute_divergence()
        
        assert 0 <= divergence <= 1
    
    def test_modulation_computation(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire'
        )
        
        # High curiosity, low confidence = chaos
        modulation = bridge._compute_modulation(0.2, 0.8, 0.0)
        assert modulation > 0  # Should drive exploration
        
        # Low curiosity, high confidence = order
        modulation = bridge._compute_modulation(0.8, 0.2, 0.0)
        assert modulation < 0  # Should drive exploitation
    
    def test_local_processing(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100,
            random_seed=42
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire'
        )
        
        message = BridgeMessage(
            message_id="test",
            timestamp=1234567890.0,
            input_text="Test",
            consciousness_proxy=0.6,
            novelty_score=0.4
        )
        
        feedback = bridge._local_processing(message)
        
        assert 'reasoning' in feedback
        assert 'confidence' in feedback
        assert 'oscillation_modulation' in feedback
        assert feedback['confidence'] == 0.6
    
    def test_get_status(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100,
            random_seed=42
        )
        bridge = StarfireBridge(
            reservoir=reservoir,
            starfire_path='../starfire'
        )
        
        status = bridge.get_status()
        
        assert 'starfire_available' in status
        assert 'mode' in status
        assert 'last_psi' in status


class TestCreateBridge:
    """Test convenience function."""
    
    def test_create_bridge_function(self):
        reservoir = ChaoticReservoir(
            input_dim=64,
            reservoir_size=100
        )
        bridge = create_bridge(reservoir)
        
        assert isinstance(bridge, StarfireBridge)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])