# Proprietary Insights - Not Publicly Available

## Compilation from COG-research, newage/DyFIN, and EXP

---

## Table 1: Core Technical Findings

| Insight | Project | What It Means | Why Not Public |
|--------|---------|---------------|----------------|
| **Spectral radius ≈ 1** predicts better generalization | COG | Self-model with spectral radius near 1 (near-critical dynamics) generalizes better than stable (radius < 1) or unstable (radius > 1) | Internal verification experiments |
| **Self-model first** architecture beats world-model 10-20x on AR1 | COG | Predicting self first, then validating world-predictions, outperforms direct world-modeling | Novel architecture, not published |
| **Near-unit circle dynamics** are optimal for time series | COG | Self-model learns eigenvalues near unit circle - not too stable, not too unstable | Mechanistic deep-dive findings |
| **VAE fails at forecasting** - reconstruction ≠ prediction | COG | VAE optimizes for input reconstruction, not future prediction - fundamental task mismatch | Internal analysis |
| **Anchor bottleneck** reduces context without attention | newage | Learned anchors attend to input, sparse graph message passing replaces full attention | Architecture spec |
| **Typed message passing** enables multi-task learning | newage | Different edge types (semantic, temporal, support) route to different experts | Novel mechanism |
| **CAAE 64% memory reduction** via request similarity | EXP | Grouping similar requests shares KV cache - 64% memory reduction | Production results |
| **3x throughput** via GPU coordination | EXP | NVLink microsharding coordinates GPUs, 3x more requests/second | Production results |
| **Spectral radius < 1** = stable but suboptimal | COG | World-model spectral radius 0.868 is too stable - can't capture dynamics | Internal metrics |

---

## Table 2: What Works / What Doesn't

| Approach | Works | Doesn't Work | Why |
|----------|-------|-------------|-----|
| Self-model for forecasting | ✅ AR1, ETTh1, ETTm1 | ? Weather, Traffic | Architecture matches task |
| VAE world-model | ❌ Forecasting | ✅ Generation | Task mismatch (recon vs predict) |
| Full attention | - | ❌ Long context | O(n²) scales poorly |
| Anchor + sparse graph | ✅ Long context efficient | ? Hard to train | Novel, untested |
| KV cache sharing | ✅ 64% memory | - | Production validated |
| Prefix caching | ✅ 42% memory | - | Document overlap helps |

---

## Table 3: Key Numbers (Proprietary)

| Metric | Self-Model | World-Model | Delta |
|--------|------------|-------------|-------|
| AR1 One-step MSE | 0.010 | 0.018 | 44% better |
| AR1 Spectral Radius | 1.0-1.2 | 180-5343 | Critical vs unstable |
| AR1 Parameters | 338-17K | 1.4K-83K | 10-20x smaller |
| ETTm1 Val Loss | 0.044 | 0.66 | 93% better |
| CAAE Memory | - | - | 64% reduction |
| CAAE Throughput | - | - | 3x improvement |

---

## Table 4: Unpublished Ideas

| Idea | Source | Potential | Risk |
|------|--------|-----------|------|
| **Float Permanence Memory** | TemporalAttention | Simple 0-1 memory system, pip-installable | Untested at scale |
| **Self-model + Anchor** | COG + newage | Combine spectral radius insight with anchor bottleneck | Hard to implement |
| **Drives + Self-Model** | Infant + COG | AI with motivation that learns what to learn | Needs scaling |
| **CAAE + Prefix Cache** | EXP | Layer KV sharing + document similarity | Production only |
| **Near-critical as training objective** | COG | Explicitly train to spectral radius ≈ 1 | Novel training |

---

## Unique Combinations (Not Explored)

### 1. Self-Model with Anchor Bottleneck
- Self-model learns dynamics
- Anchor bottleneck handles long context
- Sparse graph message passing for efficiency

### 2. Float Permanence + Self-Model  
- Float permanence (0-1) for fact memory
- Self-model for dynamics prediction
- Drives for motivation

### 3. CAAE + Reasoning
- Memory optimization from CAAE
- Add reasoning trace caching
- Self-model validates cache hits

### 4. Critical Dynamics Training
- Instead of VAE or standard RNN
- Explicitly optimize for spectral radius ≈ 1
- Use eigenvalue regularization

---

## What Could Be Breakthrough

| Combination | Why It Could Work | What's Missing |
|-------------|-------------------|----------------|
| Self-Model + Float Permanence | Self-model learns dynamics, Float handles facts | Integration code |
| Anchor + Self-Model | Efficient long context + accurate prediction | Benchmark results |
| Critical Dynamics + Infant | Baby learns with optimal dynamics from birth | Training infrastructure |

---

## Notes

- These insights come from internal experiments, not published papers
- Some results are from synthetic data (AR1) - may not generalize
- COG-research: Self-model wins on simple tasks, unclear on complex
- EXP/KViction: Production validated but requires integration
- newage/DyFIN: Architecture defined, not fully benchmarked
