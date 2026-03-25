"""LLM fallback for complex reasoning."""

from typing import Optional


class LLMFallback:
    """Use LLM only when truly needed for complex reasoning."""
    
    def __init__(self, llm_client=None):
        self.llm = llm_client
    
    def complete(self, prompt: str) -> str:
        """Complete with LLM - only called for complex reasoning."""
        if self.llm is None:
            return "No LLM available."
        
        try:
            # Simple completion
            response = self.llm.chat(prompt)
            return response
        except Exception as e:
            return f"LLM Error: {str(e)}"


__all__ = ["LLMFallback"]
