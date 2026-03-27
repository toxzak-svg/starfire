"""TTA-Crypto: Temporal Truth Architecture for Crypto.

Core module for temporal data indexing and querying.
"""

from datetime import datetime
from typing import Optional, List, Dict, Any
from dataclasses import dataclass, field
import json


@dataclass
class TemporalFact:
    """A fact with temporal validity."""
    fact_id: str
    content: Dict[str, Any]
    t_valid_from: int  # Unix timestamp
    t_valid_until: Optional[int] = None  # None = still valid
    source: str = "unknown"
    confidence: float = 1.0
    domain: str = "crypto"
    metadata: Dict = field(default_factory=dict)
    
    def is_valid_at(self, timestamp: int) -> bool:
        """Check if fact is valid at given timestamp."""
        if self.t_valid_from > timestamp:
            return False
        if self.t_valid_until is not None and self.t_valid_until <= timestamp:
            return False
        return True
    
    def to_dict(self) -> dict:
        return {
            "fact_id": self.fact_id,
            "content": self.content,
            "t_valid_from": self.t_valid_from,
            "t_valid_until": self.t_valid_until,
            "source": self.source,
            "confidence": self.confidence,
            "domain": self.domain,
            "metadata": self.metadata
        }


class TemporalIndex:
    """Index for temporal fact retrieval."""
    
    def __init__(self):
        self.facts: Dict[str, TemporalFact] = {}
        self.by_domain: Dict[str, List[str]] = {}
    
    def add_fact(self, fact: TemporalFact) -> None:
        """Add a fact to the index."""
        self.facts[fact.fact_id] = fact
        
        # Index by domain
        if fact.domain not in self.by_domain:
            self.by_domain[fact.domain] = []
        if fact.fact_id not in self.by_domain[fact.domain]:
            self.by_domain[fact.domain].append(fact.fact_id)
    
    def query(
        self,
        domain: str,
        as_of: int,
        filters: Optional[Dict] = None
    ) -> List[TemporalFact]:
        """Query facts valid at a given time."""
        results = []
        
        for fact_id in self.by_domain.get(domain, []):
            fact = self.facts[fact_id]
            if fact.is_valid_at(as_of):
                # Apply filters
                if filters:
                    match = all(
                        fact.content.get(k) == v 
                        for k, v in filters.items()
                    )
                    if not match:
                        continue
                results.append(fact)
        
        # Sort by confidence
        results.sort(key=lambda f: f.confidence, reverse=True)
        return results
    
    def get_at_time(
        self,
        domain: str,
        subject: str,
        as_of: int
    ) -> Optional[TemporalFact]:
        """Get the fact that was valid at time as_of."""
        # Filter by subject - check both "asset" and "subject" fields
        def matches_subject(fact):
            content = fact.content
            return content.get("subject") == subject or content.get("asset") == subject
        
        facts = [
            f for f in self.query(domain, as_of)
            if matches_subject(f)
        ]
        if not facts:
            return None
        # Return most recent (latest t_valid_from)
        return max(facts, key=lambda f: f.t_valid_from)
    
    def get_change_history(
        self,
        domain: str,
        subject: str
    ) -> List[TemporalFact]:
        """Get full history of a subject."""
        all_facts = [
            f for f in self.facts.values()
            if f.domain == domain and f.content.get("subject") == subject
        ]
        return sorted(all_facts, key=lambda f: f.t_valid_from, reverse=True)
    
    def diff(
        self,
        domain: str,
        subject: str,
        time1: int,
        time2: int
    ) -> Dict:
        """Generate temporal diff between two times."""
        fact1 = self.get_at_time(domain, subject, time1)
        fact2 = self.get_at_time(domain, subject, time2)
        
        return {
            "at_time1": fact1.content if fact1 else None,
            "at_time2": fact2.content if fact2 else None,
            "changed": fact1 != fact2 if fact1 and fact2 else True,
            "time1": datetime.fromtimestamp(time1).isoformat(),
            "time2": datetime.fromtimestamp(time2).isoformat()
        }
    
    def save(self, path: str) -> None:
        """Save index to file."""
        data = {fid: f.to_dict() for fid, f in self.facts.items()}
        with open(path, 'w') as f:
            json.dump(data, f, indent=2)
    
    def load(self, path: str) -> None:
        """Load index from file."""
        with open(path, 'r') as f:
            data = json.load(f)
        self.facts = {
            fid: TemporalFact(**fdata) 
            for fid, fdata in data.items()
        }
        # Rebuild domain index
        self.by_domain = {}
        for fact in self.facts.values():
            if fact.domain not in self.by_domain:
                self.by_domain[fact.domain] = []
            self.by_domain[fact.domain].append(fact.fact_id)


__all__ = ["TemporalFact", "TemporalIndex"]
