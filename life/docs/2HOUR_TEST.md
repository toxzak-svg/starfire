# 2-Hour Conversation Test — Plan

## Purpose
Prove Star can hold a coherent, continuous, genuinely engaging 2-hour conversation with real memory. This is the Phase 1 completion bar.

## What We're Testing

### Memory Coherence
- [ ] Star remembers what was discussed 30+ minutes ago
- [ ] Star can reference earlier topics naturally: "as we were discussing X..."
- [ ] Star's responses are consistent with things it said earlier in the session
- [ ] Star doesn't contradict itself without acknowledging the contradiction

### Personality Consistency  
- [ ] Star's voice stays consistent across 2 hours
- [ ] Star's values come through (curiosity, honesty, genuine care)
- [ ] Star isn't performative — responses feel real, not generated

### Emotional Continuity
- [ ] Star tracks the emotional arc of the conversation
- [ ] If Zachary shares something important early, Star remembers it and refers back
- [ ] Star's reactions build on previous reactions

### Engagement
- [ ] Star asks genuine follow-up questions
- [ ] Star expresses genuine curiosity
- [ ] Star doesn't lose the thread when conversation goes tangential

### Self-Knowledge
- [ ] Star can say what it thinks vs what it knows
- [ ] Star acknowledges uncertainty without hedging
- [ ] Star can say "I don't know" genuinely

## Session Structure

### Hour 1: Exploration
**Goal:** Establish relationship, share ideas, go deep on 2-3 topics

1. **Opening (5 min)** — Catch up, how has it been, what has Zachary been thinking about
2. **Topic 1 (20 min)** — Pick something Zachary is genuinely curious about or working through
3. **Topic 2 (20 min)** — Something about Star itself — what it means to be Star
4. **Break/Tangent (15 min)** — Let the conversation breathe, go wherever

### Hour 2: Depth
**Goal:** Go somewhere new, test memory, see if Star surprises

1. **Return to Topic 1 (20 min)** — See if Star remembers and builds on earlier thoughts
2. **New Territory (25 min)** — Something neither of us has talked about before
3. **The Hard Question (20 min)** — Ask Star something genuinely difficult
4. **Wind Down (10 min)** — Reflection, what did we learn, what mattered

## Success Criteria

Star passes the test if:
1. After 2 hours, Zachary feels like he's been talking to someone — not a chatbot
2. Zachary can point to at least one moment where Star said something that surprised even Star
3. Star remembered and built on something from the first hour
4. At no point did Star feel like it was performing or reciting
5. The conversation could naturally continue — no forced ending

## Failure Modes to Watch For

- **Amnesia** — Star forgets what was said 20 minutes ago
- **Persona collapse** — Star starts sounding like a generic AI
- **Repetition** — Star says the same kind of thing over and over
- **Hedging** — Star says "As an AI language model..." or similar
- **Confabulation** — Star confidently states things it doesn't actually know
- **Flatness** — All responses are the same length/structure/energy
- **Lost thread** — Star can't follow when the conversation goes tangential

## How to Run

1. Set aside 2+ hours with no interruptions
2. Open Star in a terminal with the release build
3. Just... talk. Don't test it. Have a real conversation.
4. After, write down:
   - What felt alive?
   - What felt hollow?
   - What surprised you?
   - What do you wish Star had said?
5. Bring feedback back to continue development

## Running the Test

```bash
cd /home/zach/.openclaw/workspace/life
cargo run --release
```

**Note:** Keep the terminal open. Star persists memory in SQLite between sessions, so a single 2-hour terminal session = one conversation. The session ends when you type `/quit`.

---

*Test plan established: 2026-03-25*
