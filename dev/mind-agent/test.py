"""Test mind-agent."""

from mind_agent import AgentMind

# Create agent
agent = AgentMind()

# Teach it something
agent.remember("Your name is Zach")
agent.remember("You like AI research")

# Test recall (no LLM!)
print("Test 1:", agent.process("What is my name?"))
print("Test 2:", agent.process("What do I like?"))

# Test action
print("Test 3:", agent.process("Do something"))
