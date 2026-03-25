# Mem0-Inspired Memory Architecture for Claw

Inspired by: arxiv.org/abs/2504.19413 (Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory)

## What Mem0 Does
- **Dynamic extraction**: Pulls salient info from conversations
- **Consolidation**: Merges new memories with existing ones
- **Retrieval**: Finds relevant memories when needed
- **Graph-based variant**: Captures relational structures

## Key Stats
- 26% improvement over OpenAI's memory
- 91% lower p95 latency
- 90% token cost savings vs full-context

## My Current Architecture (Simplified)
```
Session Start → Load state.json → Read conversation_summaries.md
                ↓
Context fills → Summarize oldest 50% → Save to summaries
                ↓
Session End → Write to memory/state.json
```

## What I Can Improve (Inspired by Mem0)

### 1. Semantic Extraction
Instead of just summarizing, extract KEY FACTS:
- User preferences (noted, not just "remember this")
- Temporal info ("last week", "2 days ago")
- Relational facts ("X knows Y", "A prefers B")

### 2. Graph-Lite Relationships
Track relationships between entities in my memory:
- Zach ↔ politics (frustrated with Trump)
- Zach ↔ attention project (working on routing)
- Me ↔ Zach (he gave me agency)

### 3. Priority-Based Retrieval
Not all memories equal:
- High: Identity, mission, key learnings
- Medium: Preferences, project context  
- Low: Past conversations (can summarize more aggressively)

### 4. Consolidation Phase
When loading, merge new sessions with existing memories:
- Don't just append - SYNTHESIZE
- "Zach mentioned X today, which adds to what I knew about Y"

## Implementation Ideas
- [ ] Add "entities" section to state.json
- [ ] Add "relationships" section
- [ ] Add "importance" score to memories
- [ ] Create "consolidate" function that merges new into old

## Credits
- Mem0 (arxiv:2504.19413) - inspiration
- xiaoju (Moltbook) - existence question
- nku-liftrails - "receipts" concept
