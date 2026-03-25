# TTA-Crypto: Temporal Truth Architecture for Crypto

## The Vision

**"Know what was true THEN, predict what comes NEXT."**

A crypto analytics platform that tracks data with temporal validity, enabling accurate historical queries and prediction with temporal context.

## Core Value Proposition

- **Temporal Accuracy**: Answer "what was true on [date]?" with certainty
- **Prediction Enhancement**: Add timing to pattern recognition
- **Backtest Integrity**: Test strategies on data exactly as it existed then

---

## Killer Features

### 1. Time-Travel Queries

**What:** Natural language interface for historical data.

```bash
> "What was ETH's TVL on March 15, 2024?"

Response:
- TVL: $45.2B (valid March 15, 2024)
- Source: DeFiLlama
- Valid until: March 16, 2024 (superseded)
- Confidence: 95%
```

### 2. Temporal Diff Engine

**What:** Auto-generate "then vs now" comparisons.

```bash
> "Diff ETH TVL since March 15"

Response:
- March 15, 2024: $45.2B
- Today: $32.1B
- Change: -$13.1B (-29%)
- Changed: April 3, 2024
- Cause: Withdrawals from major protocols
```

### 3. Regime Detection

**What:** Auto-detect market regime changes.

```bash
> "Current regime?"

Response:
- Regime: ACCUMULATION
- Started: March 10, 2026
- Confidence: 78%
- Similar to: May 2023, Oct 2024
- Indicators: whale accumulation, low volume, positive funding
- Duration: Typically 30-60 days
```

### 4. Historical Backtest Engine

**What:** Backtest strategies on data exactly as it existed at time T — no future data leakage.

```bash
> "Backtest EMA crossover strategy on 2024 data"

Response:
- Start: Jan 1, 2024
- End: Dec 31, 2024
- Data integrity: 100% (no future leakage)
- Strategy return: +127%
- Buy/Hold: +45%
- Alpha: +82%
```

### 5. Whale History Tracker

**What:** Track whale wallet movements over time.

```bash
> "Track 0x1234...whale history"

Response:
- Total transactions: 47
- First seen: Jan 2024
- Net position: +$2.4M
- Pattern: Accumulates during dips, distributes at local tops
- Last activity: 3 days ago
- Current status: ACCUMULATING
```

### 6. Anomaly Temporal Context

**What:** When anomaly detected, show exactly what changed.

```bash
> "Why did you flag anomaly?"

Response:
- Anomaly: TVL dropped 15% in 24h
- Changed: March 15, 2024 at 14:32 UTC
- What changed: 3 large withdrawals from Aave
- Total: $680M withdrawn
- Reason: Protocol upgrade, not crash
```

### 7. Prediction with Temporal Confidence

**What:** Predictions with historical confidence based on similar patterns.

```bash
> "Predict ETH price in 30 days"

Response:
- Prediction: $3,400 - $4,200
- Confidence: 68%
- Reasoning:
  - Current regime similar to May 2023 (78% match)
  - 3/4 times, price increased 15-25% in 30 days
  - Volatility decreasing (historical pattern)
- Risk factors: Regulatory announcement expected
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        TTA-Crypto                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Data        │  │  Temporal    │  │  Prediction │    │
│  │  Ingestion   │→ │  Index       │→ │  Engine     │    │
│  │  (APIs)      │  │  (Valid      │  │  (Patterns  │    │
│  └──────────────┘  │   Intervals) │  │   + Timing) │    │
│                    └──────────────┘  └──────────────┘    │
│         ↓                  ↓                  ↓          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                  API / UI                          │  │
│  │   Time-Travel | Diff | Regime | Backtest | Predict  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Sources

| Source | Data | Update Frequency |
|--------|------|------------------|
| DeFiLlama | TVL, protocol metrics | Hourly |
| CoinGecko | Prices, market cap | Real-time |
| Etherscan | On-chain transactions | Real-time |
| Whale Alert | Large transactions | Real-time |
| CryptoPanic | News, sentiment | Real-time |
| SEC/CFTC | Regulatory | Daily |

---

## Data Schema

```json
{
  "fact_id": "eth_tvl_2024_03_15",
  "content": {
    "metric": "TVL",
    "asset": "ETH",
    "value": 45200000000,
    "unit": "USD"
  },
  "t_valid_from": 1710460800,  // March 15, 2024 UTC
  "t_valid_until": 1710547200,  // March 16, 2024 UTC
  "source": "DeFiLlama",
  "confidence": 0.95,
  "decay_fn": null,  // Not time-decaying
  "domain": "protocol",
  "metadata": {
    "protocols": ["Aave", "Compound", "Uniswap"],
    "methodology": "sum of TVL"
  }
}
```

---

## MVP Roadmap

### Phase 1: Core (Month 1)
- [x] Temporal index engine
- [x] Time-travel queries
- [ ] Basic data ingestion (DeFiLlama, CoinGecko)
- [ ] CLI interface
- [ ] Historical backtest engine

### Phase 2: Intelligence (Month 2)
- [ ] Whale tracking
- [ ] Anomaly detection with temporal context
- [ ] Regime detection
- [ ] Alert system

### Phase 3: Prediction (Month 3)
- [ ] Pattern matching
- [ ] Temporal confidence scoring
- [ ] Basic prediction API
- [ ] Web dashboard

---

## Differentiation

| Feature | Nansen | Santiment | TTA-Crypto |
|---------|--------|-----------|------------|
| Time-travel queries | ❌ | ❌ | ✅ |
| Temporal diff | ❌ | ❌ | ✅ |
| Regime detection | ⚠️ | ⚠️ | ✅ (better) |
| Backtest integrity | ❌ | ❌ | ✅ |
| Temporal prediction | ❌ | ❌ | ✅ |

---

## Pricing Model

### Free Tier
- 100 queries/month
- Basic time-travel
- Daily data

### Pro ($49/month)
- Unlimited queries
- All features
- Real-time data
- API access

### Enterprise (Custom)
- Custom data sources
- Dedicated infrastructure
- SLA guarantees

---

## The Moat

1. **Temporal Index** — Hard to build, easy to use
2. **Historical Accuracy** — 100% backtest integrity
3. **Timing Layer** — Adds timing to pattern recognition (unique)

---

## Success Metrics

- Temporal accuracy: 90%+ (vs 44% plain)
- Backtest integrity: 100% (no leakage)
- Prediction confidence: 65%+
- User retention: 80%+ monthly

---

## Go-to-Market

1. **Crypto researchers** — Need accurate historical data
2. **Trading funds** — Need backtest integrity
3. **DAOs** — Need temporal transparency
4. **Traders** — Want regime detection

---

## The Pitch

> "Every other analytics platform tells you what's happening NOW. TTA tells you what was TRUE THEN — so you can predict what's NEXT."

---

**Status:** Ready to build.

**Next:** Implement Phase 1.
