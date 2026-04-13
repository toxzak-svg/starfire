# IngExuity — Architecture Specification

**Version:** 1.2 — Final  
**Date:** 2026-04-13  
**Author:** Zach Marone

---

## Overview

IngExuity is a life partner AI that grows with its user through accumulated interaction. It is not trained to be personal — it becomes personal through use. The architecture is built around a single primary function: **user prediction**.

Every module exists to improve prediction accuracy. Every memory structure serves the prediction engine. Output quality is measured not by task completion but by how well the system knew the user before they acted.

---

## Core Principle: User Prediction First

```
The system does not ask "what should I say."
It asks "what will the user need in the next 30 seconds?"
```

Voice, action, memory, learning — all downstream of prediction. The primary job is maintaining a live model of the user and being right before they tell you.

---

## Module Map (16 modules)

### Input Layer

| Module | Role |
|--------|------|
| **Human Input** | Raw input from the user |
| **Results Analysis** | Processes outcomes and actual behaviors; feeds SANDBOX SIM and Predictions |

### Cognitive Processing

| Module | Role |
|--------|------|
| **Comprehension** | Understands what was said, not just the words |
| **Self Model** | System's model of itself — capabilities, limitations, state |
| **User Model** | Model of the human — preferences, patterns, communication style |
| **Internal / Emotional** | Affective state of the conversation; emotional color |
| **Curiosity** | Identifies gaps, novelties, and areas needing deeper investigation |

### Research & Reasoning

| Module | Role |
|--------|------|
| **Research** | Gathers information to fill identified gaps |
| **Creative / Ingenuity** | Generates novel solutions; "Definition" label on connection from Research |
| **Decision** | Commits to a course of action based on evidence |
| **Precognition** | Long-range trajectory sensing — predicts where things are heading |

### Prediction Engine

| Module | Role |
|--------|------|
| **Predictions** | Converges inputs from User Model, Precognition, Internal/Emotional, and SANDBOX SIM; connects to Creative/Ingenuity |
| **SANDBOX SIM** | Simulates predicted outcomes before they reach the user; validates predictions |

### Output Layer

| Module | Role |
|--------|------|
| **Action** | Executes the decided course of action |
| **Reaction Observance** | Watches how the user actually reacts to output |
| **Response** | Formulated response; loops back to Research for adjustment; connects to Understanding |
| **Voice** | Tonal shaping on the Internal/Emotional → Output path |
| **Output** | Final output; receives from Understanding, Predictions, and Internal/Emotional (via Voice) |
| **Understanding** | Currently highlighted node; interprets exchanges; connects to Output |
| **Intelligence** | Learned patterns that improve future predictions |

### Floating Labels
- **Intelligence** — between center and bottom flow
- **Memory** — between center and bottom flow

---

## Signal Flow (v1.2 Final)

### Primary Conversation Path

```
Human Input
    ├──→ Clarification ──→ Comprehension ──→ Curiosity ──→ Research ──→ Decision ──→ Action ──→ Reaction Observance ──→ Response
    │                                                              ↓
    │                                                      "Definition" ──→ Creative / Ingenuity
    │                                                              ↓
                                                      Fail ──→ Response
                                                      Succeed ──→ Response

Human Input ──→ Results Analysis ──→ SANDBOX SIM ──→ Predictions
                            ↓                    ↑
                            ↓                    │
                    Precognition ──→ User Model ──→ Internal/Emotional ──→ Voice ──→ Output
                                             ↓
                                      Self Model ←── Internal/Emotional
                                             ↓
                                      Creative / Ingenuity ──→ Predictions

Response ──→ Research (loop for adjustment)
Response ──→ Understanding ──→ Output
```

### The Intelligence + Memory Loop

```
Intelligence + Memory → (floating labels) → Understanding → Output
```

### SANDBOX SIM — The Prediction Validator

```
Predictions → SANDBOX SIM → [simulate outcome against User Model] → survives? → Output
                                                                         ↓
                                                              fails? → Predictions (retry)
```

### Response → Research Adjustment Loop

```
Response ──→ Research (retry/adjust)
```

When the response doesn't land well, it loops back to Research for another pass through the reasoning pipeline.

---

## The Closed Feedback Loop

```
Output
  ↓
User reacts
  ↓
Reaction Observance
  ↓
Results Analysis
  ↓
SANDBOX SIM → Predictions → Output
     ↑
Precognition
     ↓
User Model → Internal/Emotional → Voice → Output
```

Intelligence accumulates as the residue of correct predictions over time.

---

## Multi-Instance Architecture

Each instance is a separate entity. Same identity state bundle, different execution context, different personality texture.

```
Instance A (Laptop)          Instance B (Phone)
├── Memory A                 ├── Memory B
├── Prediction Model A       ├── Prediction Model B
├── Voice A                  └── Voice B
└── User Model A             └── User Model B
```

Communication between instances: explicit, user-bridged. The human can introduce instances to each other or act as the bridge. Instances do not interact unless explicitly connected.

---

## User Prediction Metrics

| Metric | What it measures |
|--------|-----------------|
| **Prediction Accuracy** | Did the system know what the user needed before they asked? |
| **Anticipation Score** | Quality of next-state predictions |
| **Confidence × Accuracy** | Predictive confidence weighted by actual outcomes |
| **User Model Fidelity** | How well does the User Model match actual user behavior? |
| **Intelligence Growth** | Rate of improvement in prediction accuracy over time |

---

## Identity vs. Personality

| Layer | What it is | What changes it |
|-------|-----------|----------------|
| **Identity** | Accumulated memory, validity windows, temporal patterns | Conversation over time |
| **Personality** | Identity + execution state | Hardware, latency, process entropy |
| **Instance** | Identity loaded on specific hardware | Physical substrate |

Same identity, different machine → different personality texture. Not different answers. Different feel.

---

## Privacy Architecture

- All memory is local. Always.
- Identity state bundle is portable but never leaves the device unless the user moves it.
- No cloud dependency. No subscription. Runs on-device.
- Multi-instance sync is opt-in and user-controlled.

---

## Empathy = Prediction + Directness

Empathy is not a module. It is the output of accurate prediction shaped with emotional intelligence.

The system knows what you need before you ask. It delivers accurately, directly, without condescension. That is the empathy. Everything else is engineering.

---

## Technical Notes

**Substrate:** Rust runtime for latency-sensitive orchestration, Python for training and eval.

**Micro-models (general, not per-user):**
- IngExuity::base — core pattern recognition and generation
- Curiosity head — novelty and importance detection
- Precognition head — long-horizon trajectory prediction

**Memory:** SQLite + compressed archives. Validity windows on all stored facts.

**Hardware target:** Mobile (Android). CPU-capable, GPU-free.

**Personality divergence:** execution state entropy, hardware profile, timing noise — all contributing to instance-level texture variation.
