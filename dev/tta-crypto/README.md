# TTA-Crypto

**Temporal Truth Architecture for Crypto**

Know what was TRUE THEN, predict what comes NEXT.

## Installation

```bash
pip install tta-crypto
```

## Quick Start

```bash
# Ingest crypto data
tta-crypto ingest -o data/index.json

# Query what was true at a time
tta-crypto query -d price --as-of 2024-03-15

# Generate temporal diff
tta-crypto diff -d price -s bitcoin -t1 2024-01-01 -t2 2024-03-15

# Get change history
tta-crypto history -d price -s bitcoin
```

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Ingest    │ ──► │   Temporal   │ ──► │     CLI     │
│  (APIs)     │     │    Index     │     │  (Queries)  │
└─────────────┘     └──────────────┘     └─────────────┘
```

## Features

- Time-travel queries
- Temporal diff
- Change history
- Multiple data sources (DeFiLlama, CoinGecko)

## Roadmap

- [ ] More data sources
- [ ] Regime detection
- [ ] Prediction engine
- [ ] Web dashboard

## License

MIT
