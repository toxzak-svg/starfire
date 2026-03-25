"""
Star Client for Jupyter Notebooks
=================================
Use Star as the reasoning brain for infant in a Jupyter notebook.

Usage:
    from star_client import StarClient
    
    star = StarClient("http://localhost:8080")
    
    # Check health
    star.health()
    
    # Reason about something
    result = star.reason("What is consciousness?")
    print(result['answer'])
    
    # Get Star's identity
    identity = star.identity()
    print(identity['summary'])
    
    # Search memories
    memories = star.remember("fire", limit=5)
    for m in memories:
        print(m['content'])
    
    # Get memory stats
    stats = star.memory_stats()
    print(f"Memories: {stats['memory_count']}")
"""

import requests
from typing import Optional, List, Dict, Any


class StarClient:
    """Client for interacting with Star's HTTP API."""
    
    def __init__(self, base_url: str = "http://localhost:8080", timeout: int = 30):
        """
        Initialize Star client.
        
        Args:
            base_url: Base URL where Star API is running
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
    
    def _post(self, endpoint: str, data: dict) -> Dict[str, Any]:
        """POST to an API endpoint."""
        resp = requests.post(
            f"{self.base_url}{endpoint}",
            json=data,
            timeout=self.timeout
        )
        resp.raise_for_status()
        return resp.json()
    
    def _get(self, endpoint: str) -> Dict[str, Any]:
        """GET an API endpoint."""
        resp = requests.get(f"{self.base_url}{endpoint}", timeout=self.timeout)
        resp.raise_for_status()
        return resp.json()
    
    def health(self) -> Dict[str, str]:
        """Check if Star is running."""
        return self._get("/health")
    
    def identity(self) -> Dict[str, Any]:
        """Get Star's identity information."""
        return self._get("/identity")
    
    def memory_stats(self) -> Dict[str, Any]:
        """Get memory statistics."""
        return self._get("/memory/stats")
    
    def reason(self, query: str, memories: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Ask Star to reason about something.
        
        Args:
            query: The question or topic to reason about
            memories: Optional list of memory strings to provide as context
            
        Returns:
            Dict with keys:
                - answer: Star's response
                - confidence: low/medium/high
                - confidence_score: numeric score (0-1)
                - reasoning_chain: list of reasoning steps
        """
        data = {"query": query}
        if memories is not None:
            data["memories"] = memories
        return self._post("/reason", data)
    
    def remember(self, topic: str, limit: int = 5) -> List[Dict[str, Any]]:
        """
        Search Star's memories about a topic.
        
        Args:
            topic: Topic to search for
            limit: Maximum number of memories to return
            
        Returns:
            List of memory dicts with keys:
                - content: the memory text
                - domain: memory domain (episodic, identity, etc.)
                - importance: importance score (0-1)
                - confidence: current confidence (0-1)
        """
        data = {"topic": topic, "limit": limit}
        return self._post("/remember", data)
    
    def think(self, query: str, context: Optional[List[str]] = None) -> str:
        """
        High-level convenience method: get just the answer text.
        
        Args:
            query: The question to ask
            context: Optional context memories
            
        Returns:
            Star's answer as a string
        """
        result = self.reason(query, context)
        return result.get('answer', '')


class ConversationBuffer:
    """Buffer to accumulate context for Star."""
    
    def __init__(self, client: StarClient, max_memories: int = 10):
        self.client = client
        self.max_memories = max_memories
        self.history: List[Dict[str, str]] = []
    
    def add_turn(self, user: str, star: str):
        """Add a conversation turn."""
        self.history.append({"user": user, "star": star})
        if len(self.history) > self.max_memories:
            self.history.pop(0)
    
    def get_context(self) -> List[str]:
        """Get memories relevant to the conversation history."""
        # Extract key topics from history
        topics = []
        for turn in self.history:
            topics.extend(turn['user'].split()[:5])  # First 5 words as topics
        
        # Get memories for these topics
        memories = []
        seen = set()
        for topic in topics[:10]:
            if topic.lower() in seen:
                continue
            try:
                results = self.client.remember(topic, limit=3)
                for m in results:
                    if m['content'] not in seen:
                        seen.add(m['content'])
                        memories.append(m['content'])
            except:
                pass
        
        return memories[:self.max_memories]
    
    def ask(self, query: str) -> str:
        """Ask Star with conversation context."""
        context = self.get_context()
        result = self.client.reason(query, context)
        answer = result.get('answer', '')
        self.add_turn(query, answer)
        return answer


if __name__ == "__main__":
    # Quick test
    star = StarClient()
    print("Star health:", star.health())
    print("Star identity:", star.identity())
