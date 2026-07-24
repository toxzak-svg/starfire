# STLM SGP corpus contribution

Adds deterministic, reproducible silver/gold/platinum training corpora specialized for Starfire's bounded recurrent phrase critic and a research temporal CNN ranker.

## Corpus scale

- 3,600 RNN preference records
- 3,600 CNN preference records
- 1,800 mirrored semantic groups per corpus
- balanced silver, gold, and platinum tiers
- grouped train/dev/test splits
- separate semantic-invalid hard-gate probes

## Included infrastructure

- deterministic self-contained materializer
- strict schema and leakage validation
- current Starfire RNN loader compatibility check
- CNN and pooled-GRU reference trainers
- shuffled-label/context and surface-control generation
- blinded human-review sheets
- checksums, data card, training guide, audit report
- Kaggle-ready metadata and archive generation

## Authority boundary

These assets train and evaluate bounded surface preference only. They do not authorize semantic changes, runtime attachment, live response influence, identity rewriting, or bypass of hard verification gates.
