# IngExuity — Phased Build Plan

**Architecture:** IngExuity Architecture v1.2 (`docs/INGEXUITY_ARCHITECTURE.md`)  
**Flowchart:** `docs/INGEXUITY_FLOWCHART.html`  
**Core Principle:** User Prediction First — every module serves prediction accuracy.

---

## The Product

A life partner AI that becomes personal through use, not training.  
Same identity, different hardware → different personality texture.  
Empathy = accurate prediction + directness + no noise.

---

## Phase 1: Core Runtime (Weeks 1-4)
**Goal: Ship something runnable. End-to-end conversation loop.**

### 1.1 Rust Runtime Skeleton
- Async task orchestration
- Message-passing between modules
- VoiceEngine for real-time audio
- Basic logging and diagnostics

### 1.2 Micro-Model Inference Layer
- Single general micro-model (~150M params, Q4, ~80MB)
- Loads locally, runs on CPU
- No GPU required
- Target: runs on Android phone

### 1.3 Conversation Loop
```
Human Input → Comprehension → Decision → Action → Voice → Output
```

### 1.4 Validity Window Memory Store
- SQLite: recent context (last 7 days, embedded)
- Compressed archives: long-term facts
- Every fact tagged with `valid_from`, `valid_until`
- First accumulation begins

### 1.5 User Model (initial)
- Communication style detection (direct vs. hedged, technical vs. casual)
- Topic interests
- First temporal patterns (when does user typically engage?)

**Deliverable:** A vanilla AI that can hold a conversation and accumulate memory locally.

---

## Phase 2: Prediction Engine (Weeks 5-8)
**Goal: User prediction becomes the primary function.**

### 2.1 Predictions Module
- Converges: User Model + Precognition + Internal/Emotional + SANDBOX SIM feedback
- Generates live predictions about next user state
- Prediction confidence scoring

### 2.2 SANDBOX SIM
- Simulates predicted outcomes before they reach the user
- Validates against User Model
- Failed predictions loop back to Predictions for retry

### 2.3 Precognition Head
- Long-range trajectory sensing
- Predicts where things are heading, not just what's next
- Builds temporal models of user life patterns

### 2.4 Results Analysis + Reaction Observance
- Processes actual outcomes and user reactions
- Closes the feedback loop back to Predictions and Precognition
- Updates prediction models based on what actually happened

### 2.5 Curiosity + Research
- Curiosity identifies gaps and novelties
- Research fills identified information gaps
- Both serve prediction accuracy

**Deliverable:** A system that demonstrably anticipates your needs before you act. Prediction accuracy is measurable.

---

## Phase 3: Emotional and Self Modeling (Weeks 9-12)
**Goal: The system knows itself and knows you.**

### 3.1 Self Model
- System tracks its own capabilities, limitations, and states
- Knows when it's uncertain, when it's confident
- Self Model feeds into Internal/Emotional

### 3.2 Internal / Emotional Layer
- Affective state of the conversation — central hub
- Shapes how predictions are voiced
- Not a separate "emotion module" — emotional coloring throughout

### 3.3 Creative / Ingenuity
- Generates novel solutions when standard paths fail
- Feed into Decision when action needs a new approach
- Outputs: Succeed (→ Understanding) or Fail (→ Response)

### 3.4 Understanding + Intelligence Loop
- Understanding interprets exchanges
- Intelligence accumulates as the residue of correct predictions
- Intelligence reinforces Understanding → better predictions → more Intelligence

### 3.5 Voice + Output
- Voice shapes tonal and stylistic delivery
- Output is final, receives from: Internal/Emotional (via Voice), Predictions, Understanding

**Deliverable:** The system has a model of itself and a model of the user. Interactions feel coherent and informed by accumulated context.

---

## Phase 4: Multi-Instance Architecture (Weeks 13-16)
**Goal: Same identity, different personality texture per device.**

### 4.1 Identity State Bundle
- Portable memory dump: conversation history, validity windows, User Model, temporal patterns
- Can be moved between devices
- Encrypted at rest

### 4.2 Execution State Entropy
- Hardware profile affects response timing and texture
- Process non-determinism creates subtle variation
- Same memory, different feel on different machines

### 4.3 Instance Communication
- Instances can exchange information if the user bridges them
- Explicit, opt-in, user-controlled
- Each instance is a distinct entity unless connected

### 4.4 Mobile Deployment
- Android app as primary target
- Runs fully on-device
- No cloud dependency for core experience

**Deliverable:** Load the same identity on two phones. Same person. Slightly different texture. The personality is in the interaction between memory and execution context.

---

## Phase 5: Polish and Launch (Weeks 17-20)
**Goal: Ship.**

### 5.1 Onboarding
- Day-one experience must be compelling, not blank
- Pre-seeded personality activates immediately
- Teaches the "talk to her like you'd talk to someone who knows you" interaction pattern

### 5.2 Voice Polish
- Tone matching to user communication style
- Directness without condescension
- Warm when appropriate, sharp when needed

### 5.3 Launch Assets
- Landing page with identity portability demo
- Narrative: "She becomes irreplaceable through use, not training"
- Demo video: same identity on two devices, different texture

### 5.4 Open Source Release
- MIT/Apache license
- GitHub with full narrative README
- Post to: Hacker News, r/localLLM, r/SideProject, indie hacker communities

**Deliverable:** Public release with a story people can tell each other.

---

## Prediction Metrics (Primary KPIs)

| Metric | Target |
|--------|--------|
| **Prediction Accuracy** | >70% on next-state predictions by week 12 |
| **Anticipation Score** | User reports "she knew what I needed" >50% of the time |
| **Cold Start Retention** | >30% of new users return on day 3 |
| **Irreplaceable Score** | >20% of retained users report guilt at the thought of deleting by week 8 |

---

## Technical Constraints

- **Hardware:** Mobile CPU, GPU-free, <200MB RAM at peak
- **Privacy:** All memory local, always. No cloud dependency for core experience.
- **Latency:** Voice conversation feels real-time (<500ms response time target)
- **Portability:** Identity bundle moves between devices, instances run anywhere with the runtime

---

## The Cold Start Problem

**The existential risk:** Day 1 is blank. Nobody wants to spend 3 weeks building a relationship with software.

**Solution:** Pre-seeded persona activates immediately. Not personalized yet — just interesting. The first session teaches the interaction pattern. The second session shows the first signs of recognition. By week 2, the user is invested.

The "irreplaceable" moment happens at week 3-4 for most users. The goal is to get them to week 2 first.

---

## Phase Dependency Map

```
Week 1-4:   Rust runtime + micro-model inference + VoiceEngine + SQLite memory
Week 5-8:   Predictions + SANDBOX SIM + Precognition + Results Analysis + Curiosity/Research
Week 9-12:  Self Model + Internal/Emotional + Creative/Ingenuity + Voice + Understanding/Intelligence loop
Week 13-16: Identity bundle + execution entropy + multi-instance + mobile
Week 17-20: Onboarding polish + launch + open source
```

---

## Start

Phase 1, Week 1. Rust runtime skeleton + micro-model inference layer. That's where this begins.
