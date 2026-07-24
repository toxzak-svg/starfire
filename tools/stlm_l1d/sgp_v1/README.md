# Starfire SGP CNN/RNN training corpora

This directory contains the deterministic bootstrap and publication contract for the Starfire silver, gold, and platinum phrase-ranking corpora.

The materialization workflow reconstructs and validates:

- 3,600 recurrent phrase-ranking records;
- 3,600 convolutional phrase-ranking records;
- balanced silver, gold, and platinum tiers;
- grouped train, development, and test splits;
- mirrored context-conditioned preferences;
- hard-gate adversarial probes kept outside quality training;
- shuffled-label, shuffled-context, normalization, and length controls;
- schemas, checksums, human-review sheets, audit reports, and trainers;
- a Kaggle-ready archive and `dataset-metadata.json`.

Run locally from the repository root:

```bash
cat tools/stlm_l1d/sgp_v1/bootstrap_parts/part-*.part > /tmp/materialize_bundle.py
python /tmp/materialize_bundle.py \
  --source-root tools/stlm_l1d/sgp_v1/source \
  --output-root tools/stlm_l1d/data/sgp_v1
```

The generated corpora remain offline training and evaluation assets. They do not grant runtime language authority or bypass semantic, slot-preservation, identity-conflict, or evidence-integrity gates.
