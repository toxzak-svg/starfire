---
license: other
language:
- en
task_categories:
- text-generation
- question-answering
pretty_name: INVENT-TRACE-ZACH-v0
size_categories:
- n<1K
configs:
- config_name: full
  data_files:
  - split: train
    path: invent_trace_zach_v0.full.jsonl
- config_name: openai_messages
  data_files:
  - split: train
    path: invent_trace_zach_v0.openai_messages.jsonl
- config_name: successful_invention_traces
  data_files:
  - split: train
    path: successful_invention_traces/data.full.jsonl
- config_name: failed_invention_diagnosis
  data_files:
  - split: train
    path: failed_invention_diagnosis/data.full.jsonl
- config_name: contradiction_repair
  data_files:
  - split: train
    path: contradiction_repair/data.full.jsonl
- config_name: experiment_design
  data_files:
  - split: train
    path: experiment_design/data.full.jsonl
- config_name: productization_deployment_reasoning
  data_files:
  - split: train
    path: productization_deployment_reasoning/data.full.jsonl
---

# INVENT-TRACE-ZACH-v0

Private gold-candidate invention-deduction dataset distilled from two uploaded archives plus prior project chat history. Uploaded to `toxzak/invention` as a single private Hugging Face dataset repo.

## Requested mix and delivered counts

| Trace type | Requested | Count | Actual |
|---|---:|---:|---:|
| successful invention traces | 40% | 24 | 40.0% |
| failed invention diagnosis | 25% | 15 | 25.0% |
| contradiction repair | 20% | 12 | 20.0% |
| experiment design | 10% | 6 | 10.0% |
| productization / deployment reasoning | 5% | 3 | 5.0% |
| **Total** | **100%** | **60** | **100.0%** |

## Files

- `invent_trace_zach_v0.full.jsonl` — all structured records.
- `invent_trace_zach_v0.openai_messages.jsonl` — all records as chat fine-tuning examples.
- `schema.json` — canonical fields and counts.
- `manifest.json` — source summary and packaging metadata.
- One folder per trace type, each with `data.full.jsonl`, `data.openai_messages.jsonl`, and `README.md`.
- `push_to_hf_private.py` — local upload script for this repo.

## Privacy note

The raw ZIP contents are **not copied** into this dataset. Each record is distilled into a project-level trace with summarized source basis and `redaction_applied=true`. Keep this dataset private unless every record is manually reviewed.

## Loading examples

```python
from datasets import load_dataset

full = load_dataset("toxzak/invention", "full", split="train")
messages = load_dataset("toxzak/invention", "openai_messages", split="train")
success = load_dataset("toxzak/invention", "successful_invention_traces", split="train")
```

## Training recommendation

Use the split folders for controlled sampling, or the combined OpenAI-message file for quick LoRA/instruction-tuning experiments. The strongest eval tasks are contradiction traps, failure diagnosis, and minimal-test generation.
