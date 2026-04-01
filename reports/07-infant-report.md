# Infant — Minimal Self-Model Language Model
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/infant/`
**Language:** Python (PyTorch)
**Status:** Active; multiple architecture variants
**Last Updated:** 2026-04-01

---

## 1. Overview

Infant is an exploration of the **minimal architecture required for self-model capability**.

The name comes from developmental psychology: an infant has a primitive self-model — it learns to distinguish self-generated sensations from externally-driven ones, predicts consequences of its own actions, and builds increasingly sophisticated representations over time.

Key question: What is the smallest model that can learn a self-model?

---

## 2. Architecture Variants

| File | Focus |
|------|-------|
| `infant_talk.py` | Conversational interface |
| `chat_infant.py` | Chat-based interaction |
| `proof_test.py` | Mathematical proof capability |
| `infant_kaggle.py` / `infant_kaggle_final.py` | Kaggle competition entry |

---

## 3. Relationship to COG-Research

Infant and COG-research share the self-model question. The difference:

- **COG-research**: Large-scale experiments, critical dynamics, timeseries
- **Infant**: Minimal architectures, proof-of-concept, conversational

Infant is the "can we build the smallest possible thing that does X?" complement to COG's "how does X work at scale?" research.

---

## 4. Kaggle Integration

Infant models are submitted to Kaggle competitions for external validation — same approach as Nue. Real benchmarks, not just internal metrics.

**Path:** `infant_kaggle.py` — Kaggle competition entry point
**Path:** `infant_kaggle_final.py` — Final submission variant
