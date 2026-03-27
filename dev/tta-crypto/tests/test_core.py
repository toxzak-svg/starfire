"""Tests for TTA-Crypto."""

import pytest
from datetime import datetime
from tta_crypto.core import TemporalFact, TemporalIndex


def test_temporal_fact_validity():
    """Test fact validity checking."""
    fact = TemporalFact(
        fact_id="test_1",
        content={"value": 100},
        t_valid_from=1000,
        t_valid_until=2000
    )
    
    # Should be valid at 1500
    assert fact.is_valid_at(1500) == True
    
    # Should not be valid at 500 (before)
    assert fact.is_valid_at(500) == False
    
    # Should not be valid at 2500 (after)
    assert fact.is_valid_at(2500) == False


def test_temporal_fact_still_valid():
    """Test fact with no end time (still valid)."""
    fact = TemporalFact(
        fact_id="test_2",
        content={"value": 200},
        t_valid_from=1000,
        t_valid_until=None  # Still valid
    )
    
    assert fact.is_valid_at(15000000) == True


def test_temporal_index_query():
    """Test querying temporal index."""
    index = TemporalIndex()
    
    # Add facts
    fact1 = TemporalFact(
        fact_id="f1",
        content={"price": 100, "asset": "BTC"},
        t_valid_from=1000,
        t_valid_until=2000,
        domain="price"
    )
    fact2 = TemporalFact(
        fact_id="f2",
        content={"price": 150, "asset": "BTC"},
        t_valid_from=2000,
        t_valid_until=None,
        domain="price"
    )
    
    index.add_fact(fact1)
    index.add_fact(fact2)
    
    # Query at time 1500 (first fact valid)
    results = index.query("price", 1500)
    assert len(results) == 1
    assert results[0].content["price"] == 100
    
    # Query at time 2500 (second fact valid)
    results = index.query("price", 2500)
    assert len(results) == 1
    assert results[0].content["price"] == 150


def test_temporal_diff():
    """Test temporal diff."""
    index = TemporalIndex()
    
    fact1 = TemporalFact(
        fact_id="f1",
        content={"price": 100, "asset": "BTC"},
        t_valid_from=1000,
        t_valid_until=2000,
        domain="price"
    )
    fact2 = TemporalFact(
        fact_id="f2",
        content={"price": 150, "asset": "BTC"},
        t_valid_from=2000,
        t_valid_until=None,
        domain="price"
    )
    
    index.add_fact(fact1)
    index.add_fact(fact2)
    
    result = index.diff("price", "BTC", 1500, 2500)
    
    assert result["at_time1"]["price"] == 100
    assert result["at_time2"]["price"] == 150
    assert result["changed"] == True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
