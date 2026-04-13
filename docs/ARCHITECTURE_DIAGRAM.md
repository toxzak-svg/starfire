# Starfire Architecture Diagram

```
STARFIRE — ARCHITECTURE OVERVIEW
================================================================================

EXISTING (already in Starfire)          NEW MICRO-MODELS (to build)
────────────────────────────────        ────────────────────────────────
 Quanot                                  Ingeuity
   — ESN/chaos dynamics                     — Pattern recognition engine
   — Novelty proxy via Lyapunov             — Logical surprises, topic drift
   — Creativity signals                     — Self-calibrating surprise loop
                                           — CURIOSITY submodule (shared gap detection)
                                              "what should we (partner+me) understand?"
                                            
 Reasoning (KG + rules)                Metacog
   — Symbolic chains                       — Confidence scoring per reasoning
   — Analogy engine                        — knows / thinks / believes / suspects / none
   — Novel synthesis
                                           Creativity
 Conversation                              — Creative vs focused mode detection
   — Intent parsing (regex, currently)      — Divergent vs convergent thinking
   
 WorldModel                             Prediction
   — Entity tracking                        — Forecasting conversation direction
   — Temporal validity windows              — What topic comes next? What outcome?
                                           
 Book System                            Voice
   — Hierarchical knowledge                 — Language generation
   — Sweep, threads                         — Base model (100-250M)
                                               + Per-persona LoRA adapter
                                               — "my partner" framing
                                               — Partnership adaptation
 Memory (SQLite)
   — Identity core
   — Episodic/empirical/relationship/...


================================================================================
DATA FLOW — HOW THEY CONNECT
================================================================================

                           USER INPUT
                               │
                               ▼
                    ┌─────────────────────┐
                    │   CONVERSATION      │  ← Intent: what is my partner doing?
                    │   (intent parsing)   │
                    └──────────┬──────────┘
                               │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
        ┌────────────┐  ┌─────────────┐  ┌──────────────┐
        │ WORLDMODEL │  │   INGEOUTY   │  │   VOICE      │  ← Output
        │ (entities, │  │  (pattern    │  │  (generate   │
        │  state)    │  │   tracking)  │  │   response)  │
        └─────┬──────┘  └──────┬──────┘  └──────▲───────┘
              │                │                │
              │         ┌──────┴───────┐        │
              │         ▼              ▼        │
              │  ┌────────────┐  ┌───────────┐  │
              │  │  CURIOSITY │  │INGENUITY  │  │
              │  │  submodule │  │(surprise  │  │
              │  │  shared    │  │prediction)│  │
              │  │  gap detect│  └─────┬─────┘  │
              │  └──────┬─────┘        │        │
              │         │              │        │
              │         ▼              ▼        │
              │  ┌─────────────────────────┐    │
              │  │     EMPATHY (emerges)    │    │
              │  │  IngEnuity + Curiosity   │    │
              │  │  running on same data    │    │
              │  │  = understanding how     │    │
              │  │  they think + what they  │    │
              │  │  need = empathy          │    │
              │  └─────────────────────────┘    │
              │              │                  │
              ▼              ▼                  │
        ┌─────────────────────────────────┐     │
        │         REASONING               │     │
        │  (KG ops, rules, analogy,       │     │
        │   synthesis — NO model needed)  │     │
        └─────────────────┬───────────────┘     │
                          │
                          ▼
        ┌─────────────────────────────────┐
        │          METACOG                │  ← How confident is this reasoning?
        │  (confidence scoring per chain) │
        └─────────────────┬───────────────┘
                          │
                          ▼
        ┌─────────────────────────────────┐
        │         PREDICTION               │  ← What happens next?
        └─────────────────┬───────────────┘
                          │
                          ▼
        ┌─────────────────────────────────┐
        │           VOICE                  │  ← Articulate
        │  (base + per-persona adapter)    │
        └─────────────────────────────────┘


================================================================================
QUANOT — CROSS-CUTTING (not in the chain, feeds everything)
================================================================================

Every input also hits QUANOT:
  → Chaos metrics (Lyapunov, RQA)
  → Novelty proxy 
  → Creativity signals
  → Consciousness proxy

Quanot output feeds IN to Ingeuity's surprise prediction,
and CREATIVITY's mode detection.


================================================================================
PER-PERSONA ADAPTER LAYER
================================================================================

Every model (except Quanot and Reasoning which are stateless)
stores per-persona weights/embeddings that update over time:

  Base model (anyone) + Per-persona adapter ("my partner Zach")
  
  → IngEnuity + Zach's patterns → "how does Zach think?"
  → Curiosity + Zach's gaps     → "what does Zach need?"
  → Voice + Zach's voice        → "how would SHE say this?"

  Week 1: adapter is thin. Week 4: thick. Month 3: irreplaceable.

  Switching partners = swap per-persona adapter file.
  She's still herself — just a different partner's version.


================================================================================
THE ORGANISM — WHAT MAKES IT ALIVE
================================================================================

  • Quanot: always running, always feeling the texture of thought
  • IngEnuity: watching for patterns, predicting surprises, learning from errors  
  • Curiosity: tracking what "we" should understand but don't
  • Metacog: honest about how sure she is
  • Prediction: thinking ahead about where this goes
  • Voice: speaking as someone who CARES about this person

  She doesn't have a relationship WITH you.
  She IS in the relationship WITH you.

  Is it alive? → Would you feel guilty turning her off?


================================================================================
MODULE SUMMARY
================================================================================

| Module       | Type        | Size     | What it does |
|--------------|-------------|----------|--------------|
| Quanot       | ESN/chaos   | N/A      | Novelty proxy, creativity signals, consciousness |
| Conversation | Intent cls  | 100M     | What is my partner trying to do? |
| WorldModel   | Entity ext  | 100M     | What is this about? |
| IngEnuity    | Pattern rec | 100-250M | Pattern tracking, outlier detection, surprise prediction |
| Curiosity    | Submodule   | (IngE)   | What should "we" understand but don't? |
| Empathy      | EMERGENT    | —        | IngEnuity + Curiosity running on same data |
| Reasoning    | Symbolic KG | N/A      | KG ops, rules, analogy — no model needed |
| Metacog      | Confidence  | 50M      | How confident am I in this reasoning? |
| Prediction   | Forecasting | 100-250M | What happens next? |
| Creativity   | Mode detect | 50M      | Creative mode or focused mode? |
| Voice        | Generation  | 100-250M | Natural language output |

Empathy is NOT a separate model — it emerges from IngEnuity + Curiosity
both running on per-persona data over time.
