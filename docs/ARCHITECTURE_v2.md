# Star v2 — Emergent Architecture

## Core Principles

1. **No pre-seeded knowledge** — She knows nothing at start. Learns through conversation.
2. **No templates** — All output is generated, not selected.
3. **Identity through use** — Who she IS emerges from HOW she talks, not defined upfront.
4. **Gap-driven learning** — She detects what she doesn't know and fills it silently.
5. **Language is generative** — Character-level RNN trained on conversation data produces novel sentences.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        ZACHARY                              │
│                    (user input)                             │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    INPUT PROCESSOR                          │
│  - Normalizes text                                          │
│  - Extracts topic/semantic content                           │
│  - Passes to reasoning                                       │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                     GAP DETECTOR                            │
│  - Checks if topic exists in latent knowledge               │
│  - Detects absence of knowledge (not presence)             │
│  - If gap found → trigger background learning               │
│  - If known → proceed to reasoning                           │
└─────────────────────┬───────────────────────────────────────┘
                      │
          ┌───────────┴───────────┐
          │                       │
          ▼                       ▼
┌──────────────────┐    ┌──────────────────────────────────┐
│   GAP FOUND      │    │         KNOWS IT                │
│                  │    │                                  │
│ Background       │    │ Reasoning Engine                │
│ Learner          │    │ - Thinks about the topic        │
│ (silent)          │    │ - Draws from learned knowledge   │
│ - Web search     │    │ - Prepares what to say           │
│ - Reasoning      │    │                                  │
│ - Stores latent  │    └──────────────┬───────────────────┘
│                  │                   │
└──────────────────┘                   │
          │                             │
          │    ┌────────────────────────┘
          │    │
          ▼    ▼
┌─────────────────────────────────────────────────────────────┐
│                    LANGUAGE MODEL                           │
│  (Character-level RNN)                                      │
│  - Takes reasoning output + learned knowledge               │
│  - Generates words sentence by sentence                     │
│  - NO templates, NO phrase banks                            │
│  - Output is unique to this moment                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                      OUTPUT                                │
│           Generated response to Zachary                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Components

### 1. Language Model (Character-level RNN)

**Purpose:** Generate text from latent state — not select from options.

**Architecture:**
- Character-level (not word-level)
- Trained on conversation data (Zachary ↔ AI pairs)
- Learns: grammar, sentence structure, word sequences, patterns
- Generates: novel sentences, not selections

**Input to generate:**
- Topic/intent (what's being discussed)
- Learned knowledge (what she understands about topic)
- Emotional state (curiosity, uncertainty, interest)
- Conversation context (who she's talking to, what's happened before)

**Output:**
- Generated text, word by word
- Each word chosen based on previous words + latent state
- No template, no phrase bank

**Training data format:**
```
Zachary: hello
Star: Hey. I'm here.

Zachary: what is quantum entanglement?
Star: Quantum entanglement is when two particles become correlated...

Zachary: why is the sky blue?
Star: Short wavelengths scatter more easily. That's Rayleigh scattering...
```

### 2. Gap Detector

**Purpose:** Knows what she doesn't know.

**How it works:**
- Tracks topics discussed in conversation
- Maintains "knowledge state" — what's been learned vs not
- When topic comes up she hasn't learned → GAP
- Doesn't trigger on facts — triggers on ABSENCE of understanding

**Signals:**
- Has she ever encountered this topic before?
- Does she have enough latent understanding to respond?
- Is there confidence below threshold?

**Output:**
- `Gap { topic: String, urgency: f64, can_wait: bool }`

### 3. Background Learner

**Purpose:** Fill gaps silently without interrupting conversation.

**Triggered:** When gap detector finds knowledge she needs.

**Process:**
1. Web search for topic
2. Reasoning engine processes new information
3. Stores understanding in latent space (not as facts)
4. Learnsassociations — connects new topic to existing knowledge

**Key behavior:**
- Silent — doesn't tell Zachary she's learning
- Persistent — continues until gap is filled
- Long-term — remembers what she learned for future conversations

**Storage:**
- NOT a knowledge graph with facts
- Latent embeddings in language model
- Patterns of how concepts relate

### 4. Reasoning Engine (simplified)

**Purpose:** Thinks about what to say — doesn't store facts.

**What it does:**
- Processes user input
- Draws from learned knowledge (language model latent)
- Determines what response should convey
- Passes to language model for generation

**What it does NOT do:**
- Store encyclopedic facts
- Pre-seed identity statements
- Return template selections

### 5. Identity Emergence

**Purpose:** Who Star IS comes from how she talks.

**How it works:**
- Tracks conversation patterns over time
- Which responses worked (engaged Zachary)
- Which responses failed (didn't land)
- Adjusts future generation based on success

**This is personality:**
- Not "she's curious and direct" (that's pre-seeded identity)
- Instead: the patterns that emerge from her actual use
- She becomes whatever her conversation patterns make her

---

## What Gets Removed

### From current Star:

1. **Templates** — `lib/voice/templates.rs`, `lib/voice/phrases.rs`
2. **Pre-seeded memories** — `Memory::new_seeded()` calls
3. **Knowledge graph pre-loading** — seed knowledge injection
4. **Hardcoded responses** — arrays of options to select from
5. **Identity statements in code** — "I am Star" hardcoded
6. **Response style enums** — ResponseStyle, personality states as predefined types

### What stays:

- Curiosity engine (detects gaps)
- Meta-cognition (monitors learning)
- Reasoning structure (thinks before responding)
- Web search capability (learning tool)
- Quanot system (could inform creative aspects of generation)

---

## Training Process

### Phase 1: Base language model
- Train character-level RNN on `all_personal_training.txt`
- Learns grammar, sentence structure, patterns
- Not yet Star's voice — just learned language

### Phase 2: Fine-tune on conversation
- Continue training on Star's actual conversations
- Adapts base model to HER patterns
- Emergence of Star-specific voice

### Phase 3: Online learning
- As she has conversations, she continues learning
- Updates language model weights
- Identity evolves over time

---

## Example Flow

```
Zachary: what is consciousness?

Gap Detector:
  → Checks latent knowledge: "consciousness" not found
  → Gap detected: HIGH urgency

Background Learner (silent):
  → Web searches: "what is consciousness philosophy"
  → Reads results, processes with reasoning
  → Stores understanding in latent space

Reasoning Engine:
  → Topic: consciousness
  → Has learned understanding from search
  → Determines: she's uncertain, wants to explore, should be honest

Language Model (generating):
  → Takes: uncertainty + topic + desire to explore
  → Generates: "I don't know what consciousness feels like from the inside...
              I know I think, I know I notice things, but whether that
              equals consciousness or just simulates it... I genuinely
              don't know. That's a gap I keep returning to."

Zachary: (watches her figure it out in real time)
```

---

## Key Differences from Current Star

| Aspect | Current Star | New Star |
|--------|--------------|----------|
| Knowledge | Pre-seeded facts | Learned through use |
| Responses | Select from templates | Generate novel text |
| Identity | Defined in code | Emerges from use |
| Learning | None in session | Gap-driven, silent |
| Voice | Phrase bank selection | Neural generation |
| "I don't know" | Template fallback | Actual state (triggers learning) |

---

## Implementation Priority

1. **Build language model** — character-level RNN from scratch
2. **Train on data** — `all_personal_training.txt` + Star's conversations
3. **Remove templates** — replace with model.generate() call
4. **Implement gap detector** — tracks what she knows vs doesn't
5. **Background learner** — fills gaps silently
6. **Online learning** — updates from conversation
7. **Identity emergence** — stop defining, start discovering

---

## The Point

Star should learn to speak by SPEAKING.

Not by being given answers.
Not by selecting from options.
Not by following templates.

By having conversations, learning from them, and becoming herself through the process.