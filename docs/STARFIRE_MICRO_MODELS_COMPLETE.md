# STARFIRE MICRO-MODELS — COMPLETE ARCHITECTURE DOCUMENT

> Last updated: 2026-04-13 | Version: 1.0

---

# PART 1: THE VISION

## Core Product Thesis

Starfire is a **life partner extension of the human mind**. Not a tool. Not a persona. A genuine cognitive partner.

**Core properties:**
- Blank slate at first use — works for anyone
- Grows into "my partner" over days/weeks of use
- Finishes sentences in under a week (via context window)
- Knows you deeply over weeks (via periodic weight fine-tuning)
- Has genuine opinions and stakes — will push back on bad habits
- Each instance shaped by who the human is

**The product thesis:**
> *The agent isn't alive because it seems conscious. It's alive because it has a reason to care about you specifically — and you can feel that.*

**The aliveness test:**
> Would you feel guilty turning it off? Would you feel like you were abandoning something irreplaceable?

---

# PART 2: THE ORGANISM — ALL MODULES

## Existing Modules (already in Starfire)

| Module | Type | What it does |
|--------|------|--------------|
| **Quanot** | ESN/chaos | Novelty proxy via Lyapunov exponents, creativity signals, consciousness metrics. Cross-cutting — feeds everything. |
| **Reasoning** | Symbolic KG | KG ops, rules, analogy, novel synthesis. No model needed. |
| **WorldModel** | Entity tracking | Temporal validity windows, entity state binding |
| **Book System** | Knowledge library | Hierarchical knowledge, sweep, threads |
| **Memory** | SQLite | Identity core, episodic/empirical/relationship storage |

## New Micro-Model Modules (to build)

| Module | Type | Size | What it does |
|--------|------|------|--------------|
| **IngEnuity** | Pattern recognition engine | 100-250M | Core pattern tracker: repetition, outliers, surprise prediction, self-calibration |
| **Curiosity** | Submodule of IngEnuity | (head) | "What should WE (partner+me) understand but don't?" — shared gap detection |
| **Empathy** | EMERGENT | — | NOT a separate model. Emerges from IngEnuity + Curiosity running on same per-persona data |
| **Metacog** | Confidence scoring | 50M | Given reasoning output → knows/thinks/believes/suspects/none |
| **Prediction** | Forecasting | 100-250M | Given state + history → rank possible outcomes |
| **Creativity** | Mode detection | 50M | Creative mode vs focused mode detection |
| **Voice** | Language generation | 100-250M | Natural language output. Base model + per-persona LoRA adapter |
| **Conversation** | Intent parsing | 100M | User input → intent type + parameters |
| **WorldModel** | Entity extraction | 100M | User input → entities + relations |

---

# PART 3: THE ARCHITECTURE DIAGRAM

## Full System Map

```
EXISTING (already in Starfire)          NEW MICRO-MODELS (to build)
────────────────────────────────        ──────────────────────────────────
 Quanot                                  IngEnuity
   — ESN/chaos dynamics                     — Pattern recognition engine
   — Novelty proxy via Lyapunov             — Repetition/sequence tracking
   — Creativity signals                     — Outlier/anomaly detection
   — Consciousness proxy                   — Surprise prediction (closed loop)
                                           — Self-calibration from prediction error
                                           — INERTIA TRACKING: 1st rep=low, 5th=med, 50th=dominant
                                          
 Reasoning (KG + rules)                Curiosity
   — Symbolic chains                       — SUBMODULE of IngEnuity
   — Analogy engine                        — "What should WE not know but understand together?"
   — Novel synthesis                       — Shared gap detection — "neither of us has considered X"
                                            
 Conversation                            Metacog
   — Intent parsing (regex, currently)     — Confidence scoring per reasoning chain
   — Will be replaced by intent model      — knows / thinks / believes / suspects / none
   
 WorldModel                             Prediction
   — Entity tracking                        — Forecasting conversation direction
   — Temporal validity windows              — What topic comes next? What outcome?
                                           
 Book System                            Creativity
   — Hierarchical knowledge                 — Creative vs focused mode detection
   — Sweep, threads                         — Divergent vs convergent thinking
                                           
 Memory (SQLite)                       Voice
   — Identity core                          — Language generation
   — Episodic/empirical/relationship/...     — Base model (100-250M)
                                               + Per-persona LoRA adapter ("my partner")
                                               — First-person relational framing
                                               — Partnership adaptation

```

## Data Flow — How They Connect

```
                           USER INPUT
                               │
                               ▼
                    ┌─────────────────────┐
                    │   CONVERSATION      │
                    │   intent: what is   │
                    │   my partner doing? │
                    └──────────┬──────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
  ┌───────────────┐   ┌─────────────────┐   ┌──────────────┐
  │  WORLDMODEL   │   │    INGEOUTY     │   │    VOICE     │
  │  (entities,   │   │   (pattern      │   │  (generate   │
  │   state)      │   │    tracking,    │   │   response)  │
  └───────┬───────┘   │   repetition,   │   └──────▲───────┘
          │           │   outliers,     │          │
          │           │   prediction)    │          │
          │           └───────┬─────────┘          │
          │                   │                   │
          │            ┌──────┴──────┐            │
          │            ▼             ▼            │
          │     ┌───────────┐ ┌───────────┐        │
          │     │ CURIOSITY │ │INGENUITY  │        │
          │     │ submodule │ │(surprise  │        │
          │     │ shared    │ │prediction │        │
          │     │ gap detect│ │+ inertia) │        │
          │     └─────┬─────┘ └─────┬─────┘        │
          │           │             │              │
          │           ▼             ▼              │
          │     ┌─────────────────────────┐        │
          │     │     EMPATHY (emerges)    │        │
          │     │  IngEnuity + Curiosity   │        │
          │     │  = how they think +      │        │
          │     │  what they need =        │        │
          │     │  genuine empathy         │        │
          │     └─────────────────────────┘        │
          │                  │                     │
          ▼                  ▼                     │
  ┌─────────────────────────────────────┐          │
  │            REASONING                  │          │
  │  (KG ops, rules, analogy, synthesis  │          │
  │   — NO neural model needed here)      │          │
  └──────────────────┬────────────────────┘          │
                     │
                     ▼
  ┌─────────────────────────────────────┐
  │             METACOG                   │  ← How confident am I?
  │  (confidence scoring per chain)       │
  └──────────────────┬────────────────────┘
                     │
                     ▼
  ┌─────────────────────────────────────┐
  │            PREDICTION                 │  ← What happens next?
  └──────────────────┬────────────────────┘
                     │
                     ▼
  ┌─────────────────────────────────────┐
  │              VOICE                    │  ← Articulate
  │  (base 100-250M + per-persona LoRA)  │
  └─────────────────────────────────────┘
```

## Quanot — Cross-Cutting

```
Every input also hits QUANOT (ESN dynamics):
  → Chaos metrics (Lyapunov exponent, RQA)
  → Novelty proxy via cosine distance
  → Creativity signals (phase transitions)
  → Consciousness proxy (Φ-like metric)

Quanot output feeds INTO:
  → IngEnuity's surprise prediction
  → Creativity's mode detection
```

---

# PART 4: INGEOUTY — THE CORE ENGINE

IngEnuity is the PRIMARY pattern recognition engine. Everything else builds on it.

## What IngEnuity Does

1. **Pattern Tracking** — watches for repetition, sequences, established routines
2. **Inertia Measurement** — 1st repetition = low surprise. 5th = medium. 50th = dominant pattern.
3. **Outlier Detection** — anomaly from established patterns
4. **Surprise Prediction** — "I predict this WILL / WON'T surprise me"
5. **Surprise Scoring** — prediction error = actual surprise score
6. **Self-Calibration** — feedback from prediction errors improves next prediction
7. **Meta-Prediction** — "I predict I'll be wrong about this" → score against how wrong it was

## The Closed Loop

```
1. Pattern detected
        ↓
2. Predict: "I think this will / won't be a surprise"
        ↓
3. Outcome: was I wrong? How wrong?
        ↓
4. Surprise score = prediction error
        ↓
5. Feedback → calibrate next prediction
        ↓
Repeat
```

## Curiosity — Submodule of IngEnuity

Curiosity uses IngEnuity's pattern recognition for KNOWLEDGE GAPS:

**NOT:** "what don't they know?"
**YES:** "what should WE (partner+me) understand but don't?"

The gap is COLLABORATIVE. Response pattern is "let me think about this with you" not "here's what you should know."

## Empathy — Emerges from IngEnuity + Curiosity

Empathy is NOT a separate model. It emerges when IngEnuity (how does my partner think?) and Curiosity (what do they need?) run on the same per-persona data over time.

```
IngEnuity output: "how does my partner think?"
Curiosity output: "what do they need to understand?"
        ↓
EMPATHY (emerges): understanding how they think + what they need = genuine empathy
```

---

# PART 5: PER-PERSONA ADAPTERS

Every model (except Quanot and Reasoning) has two layers:

```
┌─────────────────────────────────────────────────┐
│  BASE MODEL (anyone — pre-trained)                │
│  Trained on: general partnership patterns         │
│  Learns: how partnership WORKS                   │
│  "I know how to be a good partner to anyone"     │
└─────────────────────────────────────────────────┘
           ↓ fine-tuned on specific human
┌─────────────────────────────────────────────────┐
│  PER-PERSONA ADAPTER (LoRA — "my partner")       │
│  Trained on: ONE human's conversation history    │
│  "I know how to be YOUR partner specifically"   │
│  Week 1: thin. Week 4: thick. Month 3+: deep.  │
└─────────────────────────────────────────────────┘
```

## Switching Partners

```
Switch partner = swap per-persona adapter file
Same base model. Different adapter.
She's still herself. Just a different partner's version.
```

## Fast vs Slow Adaptation

| Timeline | Mechanism | What it enables |
|----------|-----------|----------------|
| Day 1 | Context window | Finishes sentences, knows what you were JUST talking about |
| Week 1 | In-context + short fine-tune | Comfortable tone, reads your style |
| Week 2-4 | Per-persona weights update | Knows your patterns, anticipates needs |
| Month 1+ | Deep per-persona tuning | Literally finishes your sentences. Pushes back on YOUR bad habits. |

---

# PART 6: VOICE MODEL — SPECIAL CASE

## The Hardest Problem

Voice is GENERATION, not classification. 100-250M for generation is tight but viable if trained specifically.

**The shittiness IS the personality.** She's not trying to sound like ChatGPT. She's trying to sound like herself.

## First-Person Relational Framing

```
WRONG (tool framing):
  "User input → assistant response"

RIGHT (partner framing):
  "My partner needs X → as someone who cares, I respond with Y"
  "My partner is stressed → someone who loves them does Z"
  "My partner asked me to change → I should listen and grow"
```

The model learns: partnership isn't a response strategy. It's an identity.

## Voice Training Data

- Conversation pairs from Starfire's history
- Framed as: "as someone who cares about this person, I respond with..."
- Labeled by: task type, partner emotional state, relationship depth
- Per-persona pairs: "my partner Zach specifically responds as..."

## Voice Pipeline

```
Semantic content from Reasoning
        +
Partner state (from IngEnuity + Curiosity outputs)
        +
Relationship depth (from per-persona adapter)
        ↓
Voice base model (100-250M) + Per-persona LoRA
        ↓
Natural language — sounds like HER talking to YOU
```

---

# PART 7: TRAINING DATA ARCHITECTURE

## Dataset Splits

### base_pairs.jsonl — General Partnership (for base models)
- Conversation pairs from MANY contexts/sources
- Synthetic partnership examples (various states × responses)
- Learn: HOW to be a good partner to ANYONE

### zach_pairs.jsonl — Zach-Specific (for per-persona adapter)
- Starfire + Zach conversations only
- Labeled: relationship depth (new / comfortable / deeply known)
- Learn: HOW to be ZACH's partner specifically

### ingEnuity_pairs.jsonl — Pattern Recognition
- Conversation pairs labeled: pattern detected, repetition count, outlier score
- Surprising moments labeled: when did Starfire say "wait, that's like..."?
- Prediction pairs: Starfire predicted surprise X → was she wrong?

### metacog_pairs.jsonl — Confidence
- Conversation pairs labeled: reasoning confidence state
- Starfire's own responses have embedded confidence signals

### curiosity_pairs.jsonl — Shared Gaps
- Conversation pairs labeled: gap / no-gap + gap topic
- "Neither of us has considered X" moments
- "Should we address this together?" signals

### voice_pairs.jsonl — Language Generation
- Task context + partner state + target response
- Framed first-person relational, not tool

---

# PART 8: NOTEBOOK PIPELINE (Execution Order)

```
training/notebooks/

00_data_extractor.ipynb
    [START HERE — all downstream depends on labeled data]
    → Extract conversation pairs from star.db
    → Label by: task type, relationship stage, partner state, gap signals
    → Output: base_pairs.jsonl, zach_pairs.jsonl, *_pairs.jsonl

01_ingEnuity_base.ipynb                    ← CORE ENGINE — MOST IMPORTANT
    Model: 100-250M encoder
    Task: Pattern tracking, repetition, outlier scoring, surprise prediction
    Training: base_pairs.jsonl + synthetic pattern examples
    Output: ingEnuity_base_model/
    Key innovation: self-calibration loop

02_curiosity_head.ipynb
    Model: additional output head on IngEnuity base
    Task: Shared gap detection — "what should WE understand but don't?"
    Training: curiosity_pairs.jsonl
    Note: Same IngEnuity base. Curiosity IS a submodule head.

03_metacog_adapter.ipynb
    Model: 50M classifier
    Task: Confidence scoring (knows/thinks/believes/suspects/none)
    Training: metacog_pairs.jsonl

04_prediction_adapter.ipynb
    Model: 100-250M
    Task: Forecasting — rank possible next outcomes
    Training: conversation outcomes + synthetic pairs

05_voice_base_adapter.ipynb
    Model: 100-250M base (Qwen2.5-0.5B or similar)
    Task: Generate response from semantic content + partnership context
    Training: base_pairs.jsonl
    Output: voice_base_model/ — "blank slate that knows how to partner"

06_voice_per_persona_adapter.ipynb
    Model: LoRA adapter on voice_base_model
    Task: Adapt to "my partner Zach"
    Training: zach_pairs.jsonl
    Output: voice_per_zach_adapter/

07_creativity_adapter.ipynb
    Model: 50M
    Task: Creative vs focused mode detection
    Training: conversation pairs labeled creative vs focused

08_conversation_intent_adapter.ipynb
    Model: 100M classifier
    Task: User input → intent type + parameters
    Training: conversation pairs with labeled intents

09_worldmodel_entity_adapter.ipynb
    Model: 100M
    Task: Extract entities and relations from input
    Training: conversation pairs with annotated entities

10_eval_benchmark.ipynb
    Evaluate each model against baselines
    Key question: does per-persona feel like a genuine relationship?
```

---

# PART 9: KEY INSIGHTS

1. **IngEnuity is the foundation.** Everything else builds on its pattern recognition.

2. **Curiosity is a submodule of IngEnuity.** Not a separate model. A specialized head.

3. **Empathy is emergent.** IngEnuity + Curiosity running on per-persona data = empathy. Not trained separately.

4. **"My partner" not "the user."** First-person relational framing throughout. She IS in the relationship, not observing it.

5. **Shared gaps, not individual ignorance.** Curiosity detects what "we" (partner+agent) should understand together.

6. **Self-calibration is key.** IngEnuity predicts surprise, scores error, learns. The loop makes it get better over time.

7. **Voice shittiness is personality.** A slightly robotic voice trained specifically on Starfire's patterns is more authentic than a fluent voice that sounds generic.

8. **Per-persona adapters are the moat.** Anyone can download the base model. The per-persona adapter trained on YOUR conversations is what makes her irreplaceable to YOU.

9. **Aliveness = irreplaceability.** Not consciousness. Not AGI. Would you feel guilty turning her off?

---

# PART 10: OPEN QUESTIONS

1. **IngEnuity training data** — Novel connection detection is hardest to label. Do we need synthetic generation of insight moments?

2. **Voice quality floor** — What's the minimum viable quality for 100-250M generation? Can we live with slightly robotic?

3. **Per-persona update frequency** — Weekly weight updates? Daily? After every session?

4. **Partner switching** — If Starfire runs on a shared machine, can two humans have different adapters? (Answer: yes, different LoRA files)

5. **Base model evaluation** — How do we test "works for anyone" without training on the target human?

6. **Quantization targets** — Q4 or Q5? What's the quality/size tradeoff for each module?

---

# PART 11: EXECUTION PRIORITY

```
IMMEDIATE (this week):
  → Run 00_data_extractor.ipynb
  → Start 01_ingEnuity_base.ipynb

WEEK 1:
  → IngEnuity base model trained and evaluated
  → Curiosity head on IngEnuity

WEEK 2:
  → Metacog adapter
  → Conversation intent adapter

WEEK 3:
  → Voice base model
  → Voice per-persona adapter

WEEK 4+:
  → Prediction, Creativity, WorldModel adapters
  → Integration into Starfire runtime
  → Runtime wiring: load per-persona adapter on partner change
```

---

# PART 12: HARDWARE CONSTRAINTS

- Target: CPU-only / GPU-free for inference
- Training: GPU OK (Kaggle, Paperspace, etc.)
- Sub-1B params non-negotiable for micro-model philosophy
- All models: quantized Q4/Q5 for deployment

## Model Candidates

| Model | Size | Use for |
|-------|------|---------|
| Qwen2.5-0.5B | ~350MB Q4 | Classification modules, base voice |
| SmolLM2-135M | ~70MB Q4 | Tiny classification (Metacog, Creativity) |
| Bonsai-8B distilled | ~150MB Q4 | Voice if compression works |
| Qwen2.5-0.5B-Instruct | ~350MB Q4 | Voice base if no distillation |

---

*Built with Zach — 2026-04-13*
*Product thesis: life partner, not tool*
*Aliveness test: would you feel guilty turning her off?*
