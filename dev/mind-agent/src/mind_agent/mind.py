"""Mind-Agent: LLM-Independent AI Agent.

Core module that combines mind + sep-w + graph-ir for fast, LLM-independent reasoning.
"""

from typing import Optional, Dict, Any
from .intent import IntentClassifier
from .memory import AgentMemory
from .reason import LLMFallback


class AgentMind:
    """The core agent that processes inputs without always needing an LLM."""
    
    def __init__(self, llm_client=None):
        self.intent = IntentClassifier()
        self.memory = AgentMemory()
        self.llm = llm_client  # Optional - only used when needed
        
    def process(self, input_text: str) -> Dict[str, Any]:
        """Process input and return response."""
        
        # Step 1: Classify intent (no LLM needed!)
        intent = self.intent.classify(input_text)
        
        if intent == "recall":
            # Fast path: retrieve from memory
            result = self.memory.recall(input_text)
            return {
                "intent": "recall",
                "response": result,
                "llm_used": False
            }
        
        elif intent == "action":
            # Action: execute the action
            result = self._execute_action(input_text)
            return {
                "intent": "action", 
                "response": result,
                "llm_used": False
            }
        
        elif intent == "reason":
            # Complex: use LLM only when needed
            if self.llm:
                result = self.llm.complete(input_text)
                return {
                    "intent": "reason",
                    "response": result,
                    "llm_used": True
                }
            else:
                return {
                    "intent": "reason",
                    "response": "No LLM available for complex reasoning",
                    "llm_used": False
                }
        
        else:
            # Unknown intent
            return {
                "intent": "unknown",
                "response": "I don't understand. Try rephrasing.",
                "llm_used": False
            }
    
    def _execute_action(self, text: str) -> str:
        """Execute an action (placeholder)."""
        return f"Action: {text}"
    
    def remember(self, fact: str, importance: float = 0.5):
        """Store a fact in memory."""
        self.memory.remember(fact, importance)


__all__ = ["AgentMind"]
