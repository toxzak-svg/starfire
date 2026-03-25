"""Intent classification - no LLM needed!"""

from typing import Literal


IntentType = Literal["recall", "action", "reason", "unknown"]


class IntentClassifier:
    """Classify input intent using keyword matching - no LLM needed!"""
    
    RECALL_KEYWORDS = [
        "what", "who", "where", "when", "remember",
        "name", "tell", "show", "find", "get",
        "list", "give", "explain"
    ]
    
    ACTION_KEYWORDS = [
        "do", "make", "create", "run", "start",
        "stop", "open", "close", "send", "call",
        "execute", "perform", "trigger"
    ]
    
    REASON_KEYWORDS = [
        "why", "how", "think", "explain", "reason",
        "because", "figure", "understand", "analyze"
    ]
    
    def classify(self, text: str) -> IntentType:
        """Classify the intent of the input text."""
        text_lower = text.lower()
        
        # Check recall keywords
        if any(kw in text_lower for kw in self.RECALL_KEYWORDS):
            return "recall"
        
        # Check action keywords  
        if any(kw in text_lower for kw in self.ACTION_KEYWORDS):
            return "action"
        
        # Check reason keywords
        if any(kw in text_lower for kw in self.REASON_KEYWORDS):
            return "reason"
        
        # Default to recall (most common)
        return "unknown"


__all__ = ["IntentClassifier", "IntentType"]
