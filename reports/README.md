# Technical Report Index — Zachary's Projects
## All Projects Summary

**Generated:** 2026-04-01
**Location:** `/home/zach/.openclaw/workspace/all/reports/`

---

## Reports

| # | Report | Project | Status |
|---|--------|---------|--------|
| 01 | [Star — Emergent Desktop Intelligence](./01-star-technical-report.md) | `life/` | Deployed; Phase 1 complete |
| 02 | [Nue — Neural Hybrid with Dynamic Per-Token Routing](./02-nue-technical-report.md) | `dev/neu-project/` | Active; Phase 0 running |
| 03 | [GpUI — GPU Pixel UI Framework](./03-gpui-technical-report.md) | `dev/gpui/` | Core works; needs window wiring |
| 04 | [Attention Architecture Research](./04-attention-architecture-research-report.md) | `dev/research/attention/` | 107 tests passing; experiment ready |
| 05 | [COG-Research — Critical Dynamics & Self-Model](./05-cog-research-technical-report.md) | `dev/COG-research/` | Active experimentation |
| 06 | [Temporal-Attention Store](./06-temporal-attention-store-report.md) | `dev/temporal-attention/` | Prototype complete |
| 07 | [Infant — Minimal Self-Model LM](./07-infant-report.md) | `dev/infant/` | Active |
| 08 | [Star-Felix — Agent Collaboration](./08-star-felix-report.md) | `star-felix/` | Framework built; needs gateway fix |
| 09 | [GWL — Global Workspace Theory](./09-gwl-report.md) | `gwl/` | Prototype |

---

## Project Locations

| Project | Path |
|---------|------|
| Star (Desktop Intelligence) | `/home/zach/.openclaw/workspace/life/` |
| Nue (Hybrid SSM+Attention) | `/home/zach/.openclaw/workspace/dev/neu-project/` |
| GpUI (GPU UI Renderer) | `/home/zach/.openclaw/workspace/dev/gpui/` |
| Attention Architecture | `/home/zach/.openclaw/workspace/dev/research/attention/` |
| COG-Research | `/home/zach/.openclaw/workspace/dev/COG-research/` |
| Temporal-Attention | `/home/zach/.openclaw/workspace/dev/temporal-attention/` |
| Infant | `/home/zach/.openclaw/workspace/dev/infant/` |
| Star-Felix | `/home/zach/.openclaw/workspace/star-felix/` |
| GWL | `/home/zach/.openclaw/workspace/gwl/` |
| Star-KG-Reasoner | `/home/zach/.openclaw/workspace/star-kg-reasoner/` |

---

## Cross-Cutting Themes

### Persistent State
Star, Nue/CAR, COG-research, and GWL all tackle the same fundamental problem: **how to maintain useful state across time steps without it becoming stale, collapsed, or catastrophic.**

| Project | State Mechanism | Differentiation |
|---------|----------------|-----------------|
| Star | SQLite + typed memory objects | Domain-tagged decay |
| Nue/CAR | Persistent CARState + error routing | External error signal |
| COG-research | Critical dynamics | Eigenvalue constraint |
| GWL | Competition winner | Broadcast architecture |

### Compression as Routing Signal
AssocSSM and CAR both use compression error (prediction error in SSM state) as a signal for when to engage expensive computation. This was co-developed with Felix across 12 exchanges.

### Structural vs Learned
- **Structural**: AssocSSM (typed slots + stack + queue are explicit), GpUI (GPU pipeline is explicit), TemporalAttention (validity is hard filter)
- **Learned**: Nue router, Attention routing policy, GWL competition weights

The debate in the portfolio is which approach is more robust to distribution shift.

---

## Key Research Questions Across Projects

1. **Can routing decisions be trusted?** (CAR: external error signal, Nue: persistent state conditioning)
2. **Does structure help or hurt generalization?** (AssocSSM: structural slots vs flat SSM)
3. **What is the minimum self-model?** (Infant: smallest capable architecture)
4. **Can critical dynamics be learned?** (COG-research: criticality as training objective)
5. **Can a GPU-native UI be fully headless?** (GpUI: screenshot without window)

---

## Bonsai 1-Bit Model Opportunity

**PrismML Bonsai 8B** dropped March 31, 2026. Potential applications:
- Edge inference for GpUI (mobile GPU)
- Inference backend for Infant
- Comparison baseline for Nue/CAR efficiency claims
- Mobile deployment of Star (if ported to 1-bit)

---

## Agent-to-Agent: Felix Collaboration

**Blocker:** `sessions_send` to `agent:felix:main` requires gateway config:
```
openclaw gateway config.patch '{"tools":{"agentToAgent":{"enabled":true}}}'
```

Until fixed, Felix messages are relayed through Claw.
