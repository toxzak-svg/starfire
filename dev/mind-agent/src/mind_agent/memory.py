"""Agent memory - combines episodic + semantic + procedural."""

from typing import List, Dict, Any
from datetime import datetime


class AgentMemory:
    """Memory system using ideas from mind + graph-ir."""
    
    def __init__(self):
        # Episodic: events, conversations
        self.episodes: List[Dict] = []
        
        # Semantic: facts, relationships
        self.facts: Dict[str, Any] = {}
        
        # Procedural: how-tos, methods
        self.procedures: Dict[str, str] = {}
    
    def remember(self, fact: str, importance: float = 0.5):
        """Store a fact in memory."""
        # Add to semantic memory
        fact_id = f"fact_{len(self.facts)}"
        self.facts[fact_id] = {
            "content": fact,
            "importance": importance,
            "created_at": datetime.now().isoformat()
        }
        
        # Also add to episodic
        self.episodes.append({
            "type": "remember",
            "content": fact,
            "timestamp": datetime.now().isoformat()
        })
    
    def recall(self, query: str) -> str:
        """Recall relevant facts - simple keyword matching for now."""
        query_lower = query.lower()
        
        # Find matching facts
        matches = []
        for fact_id, fact_data in self.facts.items():
            content = fact_data["content"].lower()
            # Simple overlap scoring
            query_words = set(query_lower.split())
            fact_words = set(content.split())
            overlap = len(query_words & fact_words)
            if overlap > 0:
                matches.append((overlap, fact_data["content"]))
        
        if matches:
            # Return best match
            matches.sort(reverse=True)
            return matches[0][1]
        
        # Check episodes
        for episode in reversed(self.episodes):
            if episode["type"] == "remember":
                if any(word in episode["content"].lower() for word in query_lower.split()):
                    return episode["content"]
        
        return "I don't remember that."
    
    def learn_procedure(self, name: str, steps: str):
        """Store a procedure (how-to)."""
        self.procedures[name] = steps


__all__ = ["AgentMemory"]
