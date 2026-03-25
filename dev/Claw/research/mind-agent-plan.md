# Project Mind-Agent: LLM-Independent AI Agent

## The Vision

Build an AI agent that runs mostly WITHOUT an LLM, using:
- **mind** — consciousness as computation
- **sep-w** — fast LLM-free retrieval
- **graph-ir** — knowledge graphs
- **engine** — unified reasoning
- **novel** — LLM when truly needed (fallback)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    INPUT: "What's my name?"                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    mind.intent.classify()                    │
│                 (Is this a recall, action, or reasoning?)    │
└─────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
      ┌──────────┐    ┌──────────┐    ┌──────────┐
      │  RECALL  │    │  ACTION  │    │ REASON   │
      │          │    │          │    │          │
      │sep-w     │    │ mind.act │    │ novel    │
      │lookup()  │    │()        │    │(LLM)     │
      └──────────┘    └──────────┘    └──────────┘
            │                 │                 │
            └─────────────────┼─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ OUTPUT: "Your    │
                    │ name is Zach"   │
                    └─────────────────┘
```

## Components

### 1. Mind Core (the "I")
```python
class AgentMind:
    def __init__(self):
        self.memory = mind.memory.EpisodicMemory()
        self.kg = graphir.KnowledgeGraph()
        self.intent = mind.intent.IntentClassifier()
        self.llm = None  # Optional, use only when needed
    
    def process(self, input_text):
        # Step 1: Classify intent
        intent = self.intent.classify(input_text)
        
        if intent == "recall":
            # Fast path: no LLM needed!
            return self.recall(input_text)
        
        elif intent == "action":
            return self.act(input_text)
        
        elif intent == "reason":
            # Only here do we use LLM
            return self.reason_with_llm(input_text)
        
        else:
            return self.unknown(input_text)
```

### 2. Memory (episodic + semantic + procedural)
```python
# Based on mind.memory + graph-ir
class AgentMemory:
    def __init__(self):
        self.episodic = mind.memory.EpisodicMemory()  # Events
        self.semantic = graphir.KnowledgeGraph()      # Facts
        self.procedural = {}                           # How-tos
    
    def remember(self, fact, importance=0.5):
        # Store in both episodic and semantic
        self.episodic.add(fact)
        self.semantic.add_entity(fact)
    
    def recall(self, query):
        # Fast retrieval with sep-w
        return sepw.search(query, index=self.semantic)
```

### 3. Intent Classification (no LLM!)
```python
# Based on mind.intent
class IntentClassifier:
    RECALL_KEYWORDS = ["what", "who", "where", "remember", "name"]
    ACTION_KEYWORDS = ["do", "make", "create", "run", "start"]
    REASON_KEYWORDS = ["why", "how", "think", "explain"]
    
    def classify(self, text):
        text = text.lower()
        if any(kw in text for kw in self.RECALL_KEYWORDS):
            return "recall"
        elif any(kw in text for kw in self.ACTION_KEYWORDS):
            return "action"
        elif any(kw in text for kw in self.REASON_KEYWORDS):
            return "reason"
        else:
            return "unknown"
```

### 4. Reasoning (LLM only when needed)
```python
def reason_with_llm(self, query):
    # Gather context from memory
    context = self.memory.recall(query)
    
    # Only NOW use LLM
    prompt = f"Context: {context}\nQuestion: {query}"
    return self.llm.complete(prompt)
```

## The Numbers

| Task Type | LLM Needed? | Speed |
|-----------|-------------|-------|
| Recall ("What's my name?") | ❌ No | Instant |
| Action ("Run the thing") | ❌ No | Fast |
| Reasoning ("Why did X?") | ✅ Yes | Slow |

**Target: 80% of tasks NO LLM needed!**

## Implementation Plan

### Phase 1: Core (Week 1)
- [ ] Integrate mind.memory with sep-w for fast retrieval
- [ ] Build simple intent classifier
- [ ] Basic recall functionality

### Phase 2: Knowledge (Week 2)
- [ ] Integrate graph-ir for knowledge graphs
- [ ] Add temporal memory (from TTA!)
- [ ] Episodic + semantic storage

### Phase 3: Actions (Week 3)
- [ ] Tool/action system
- [ ] Integrate with OpenClaw tools
- [ ] Execute actions without LLM

### Phase 4: LLM Fallback (Week 4)
- [ ] Integrate novel for LLM calls
- [ ] Only use when intent = "reason"
- [ ] Optimize LLM usage

## Files Created

```
C:\dev\mind-agent\
├── src/
│   └── mind_agent/
│       ├── __init__.py
│       ├── mind.py       # Core AgentMind
│       ├── memory.py     # AgentMemory
│       ├── intent.py      # IntentClassifier
│       └── reason.py      # LLM fallback
├── pyproject.toml
└── README.md
```

## Why This Works

1. **Most queries are simple recall** — "what", "who", "where"
2. **Intent is easy to classify** — keyword matching works 80%
3. **sep-w is fast** — 80MB model, no GPU needed
4. **LLM only for reasoning** — 20% of queries

## Expected Results

- **Speed**: 10x faster for recall tasks
- **Cost**: 80% fewer LLM calls
- **Independence**: Can run without LLM for most tasks

---

**This is the solution to the LLM dependency problem!**
