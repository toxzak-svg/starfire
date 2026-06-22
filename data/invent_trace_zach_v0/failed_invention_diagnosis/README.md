---
license: other
private: true
task_categories:
- text-generation
language:
- en
tags:
- invention-traces
- reasoning
- deduction
- private
size_categories:
- n<1K
---

# failed_invention_diagnosis

Private gold-candidate dataset slice for `failed_invention_diagnosis` from `invent_trace_zach_v0`.

## Contents

- `data.full.jsonl`: canonical structured trace records.
- `data.openai_messages.jsonl`: OpenAI-style chat fine-tuning examples.

## Count

15 records.

## Source policy

Records are distilled from the user's uploaded archives and project chat history. Raw private messages are **not** included. Provenance is summarized at a project/source level and `redaction_applied=true` is set on every record.

## Intended use

Train/evaluate models that produce invention-deduction traces with constraints, observations, deductions, abductive leaps, failure modes, and minimal tests.

## Caveat

These are gold-candidate traces: high-signal and structured, but still recommended for human review before public release or larger-scale training.
