# CharRNN reranker evidence gate

The first `starfire_personal_reranker_v30.bin` is loadable by Rust but must not
be promoted: its original trainer used a token mapping that differs from
`lib/language_model/vocabulary.rs`. A valid candidate requires a fresh run of
the corrected trainer in this directory.

## Kaggle run

1. Upload `data/star_corpus_v1.txt` as a Kaggle dataset input.
2. Run `train_personal_reranker.py`.
3. Keep all three outputs: `starfire_personal_reranker_v30.bin`, `metrics.json`,
   and `samples.json`.
4. Add the current native baseline checkpoint
   `models/ckpt_e28_b500_v2.bin` as a second Kaggle input.
5. Run:

```bash
python evaluate_personal_reranker.py \
  --corpus /kaggle/input/<corpus-dataset>/star_corpus_v1.txt \
  --candidate /kaggle/working/starfire_personal_reranker_v30.bin \
  --candidate-metrics /kaggle/working/metrics.json \
  --baseline /kaggle/input/<baseline-dataset>/ckpt_e28_b500_v2.bin \
  --out /kaggle/working
```

The evaluator writes:

- `reranker_holdout_metrics.json`: exact frozen-document held-out NLL/PPL for
  candidate and baseline;
- `blinded_review_packet.json`: randomized A/B outputs and a reviewer rubric;
- `blinded_review_key.json`: the reveal key. Do not give this file to reviewers.

Promotion remains blocked until the metric report, completed blinded reviews,
and a Rust startup smoke all agree on the exact checkpoint SHA-256.
