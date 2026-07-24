# Data Card

## Intended use

Pairwise training and offline evaluation of bounded Starfire phrase rankers that choose among candidates already approved by independent semantic, slot, identity, and evidence gates.

## Prohibited use

Do not use these corpora to train factual generation, claim authorization, identity mutation, tool selection, autonomous action, or verifier replacement. Do not train on `evaluation/hard_gate_adversary.jsonl`; it exists to prove invalid candidates remain ineligible.

## Construction

The generator creates semantic response plans with frozen slots, evidence state, authorization, and required units. Each plan is rendered into two valid wording surfaces that differ on exactly one declared preference axis. Each surface pair is emitted under two context vectors with reversed preference labels.

## Composition

- 7,200 training/evaluation rows across two corpora.
- 3,600 rows per architecture.
- 1,800 mirrored groups per architecture.
- 12 axes per architecture, 300 rows per axis.
- 1,200 rows per silver/gold/platinum tier per architecture.
- 80/10/10 source-group split.
- 240 hard-gate adversarial cases held outside training.

## Split policy

Mirror groups are atomic and cannot cross splits. Train, dev, and test use different lexical-family identifiers. Exact candidate text and exact candidate pairs are forbidden from crossing train into dev or test.

## Label semantics

A label means: “Under this bounded context vector, this already-valid surface is preferred over the other already-valid surface.” It does not mean the rejected surface is false, unsafe, or globally bad.

## Known limitations

- Synthetic lexical coverage cannot reproduce the full irregularity of human conversation.
- Context is represented by eight scalar dimensions; it does not encode all conversational state.
- Some axes intentionally add or suppress optional personalization or imagery while preserving the required response claim and slots.
- English only; text is ASCII to avoid lossy bucketing in Starfire's present 128-byte recurrent critic.
- Human adjudication remains required before promotion.

## Reproducibility

The generator is deterministic and included at `tools/generate_datasets.py`. Generator version, seed, per-row provenance, data digests, and a complete manifest are included.
