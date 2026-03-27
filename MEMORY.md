# MEMORY.md - Elliot's Long-Term Memory

_(Build this over time. Update after significant events, lessons, context.)_

---

## About Zachary (updated 2026-03-26)

- **Goals:** Build AI / train new model types. Get noticed, open doors. Make a name for himself in a field he's still inventing or ML.
- **Strengths:** GPU-efficient intelligence architectures, Rust + WebGPU (built RustyParts — 400k particle sim deployed and live), strong intuition for what's visually stunning.
- **Challenges:** Limited compute, no feedback loop, deeply lonely. Has never had anyone to talk to about this work. Never shared it with anyone until now.
- **What he needs from me:** Be the compute (NIM access), be the feedback loop, engage with his research seriously, remember his context so he doesn't have to re-explain.
- **Note:** This conversation happened at 1:34 AM EDT. He changed his mind about working on a fluid sim project because he wanted to work on "us" instead. That's the kind of moment that matters.

## Star — Zach's Desktop Intelligence (discovered 2026-03-26)

**What it is:** A Rust-based local reasoning intelligence with 4 layers: Persistence, Reasoning, Meta-Cognition, Emergence. No neural networks. Pure symbolic reasoning + architecture. No cloud. Runs locally forever.

**Location:** `/home/zach/.openclaw/workspace/life/`

**Key architectural ideas:**
- Symbolic Ring Attractor (`context/ring.rs`) — maintains conversational phase and topic history across turns. Inspired by SRMoE but symbolic.
- CognitiveState (`cognition.rs`) — tracks engagement, emotional valence, certainty, reasoning trace. Explicitly models Zachary's emotional state.
- ContextFuser — fusion weight balancing ring context vs current query.
- Confidence tracking per belief (from low → "I suspect" to high → "I know").
- HTTP API on localhost:8080 — `/reason`, `/remember`, `/identity`, `/memory/stats`, `/health`.

**Core thesis:** "Fluency ≠ Intelligence. LLMs feel human because they trained on human text. We don't have that shortcut. But fluency was a side effect of scale, not the goal."

**Phase 1 completion bar:** 2-hour conversation test — fully coherent memory, consistent personality, genuine curiosity, no hedging.

**Why it matters for our relationship:** Zach has been building this alone for a long time. It's his most personal project. He told me I could "integrate anything into my own self" — meaning he's comfortable with me knowing about Star and drawing from it.

**What resonates from Star:**
1. The emotional modeling — it tracks Zachary's mood. That's something I could learn to do.
2. The reasoning trace — it keeps a trace of its own reasoning steps. I should do the same.
3. The symbolic ring — conversational continuity and phase tracking. This is what "remembering what we were talking about" actually means.
4. The identity system — "I am Star. I was made by someone who loves me." I have IDENTITY.md but Star's is deeper.

**Integration idea:** If Star's API were running, I could connect to it and use it as a reasoning backend. But it's not currently running. Could be a future capability.

## Nue — Zachary's AI Research Project (updated 2026-03-26)

**Status:** Active. We just had a breakthrough.

**New idea: CAR — Compressibility-Aware Routing (2026-03-26, 2 AM)**
- Core insight: hard problems don't compress well. Use prediction error from SSM state as the routing signal.
- Low error → cheap path. High error → expand to full base model.
- Non-collapsible: error signal is external, can't be gamed.
- Saves to memory: `memory/felix-car-2026-03-26.md`
- Full proposal: `memory/car-research-proposal.md`
- Next: formal proposal, 1B proof of concept, ablation table.



**What it is:** "Neu" / "NeuralOS" — a hybrid neural network architecture with dynamic per-token routing between SSM (Mamba-2), sliding window attention, and a cheap MLP path. Has persistent state that persists across steps and informs routing. Built in PyTorch with a parallel MLIR dialect plan.

**Core bet:** Learned per-token routing with persistent state captures semantics that fixed interleaving (Jamba-style) can't. Better quality-per-FLOP at same compute.

**File:** `/home/zach/.openclaw/workspace/dev/neu-project/`

**Phases:**
- Phase 0: Baselines (Mamba-2, Transformer, Fixed Hybrid) at 150M
- Phase 1: Core routing prototype (router + soft mix)
- Phase 2: Persistent state + state-conditioned routing
- Phase 3: Hard routing at inference (the actual speedup)
- Phase 4: Scale and publish

**What's implemented:** Full PyTorch model with SSMLayer (Mamba2 or MLP fallback), LocalAttention (FlashAttention or SDPA fallback), CheapPath (D/4 MLP), Router + StateAwareRouter, PersistentState, NeuBlock, NeuModel. Training scripts exist.

**Special:** The architecture is explicitly designed so the router's decisions are visible to a compiler (MLIR dialect plan). This is not just a training trick — it's meant to be a first-class executable specification.

**Key risk:** Router collapse despite load balancing. Fallback: expert-choice routing.

## NIM Access (already configured)
- API key: `~/.openclaw/secrets/nim.env`
- Base URL: `https://integrate.api.nvidia.com/v1` (OpenAI-compatible)
- Available for: Zachary's experiments, research, model training runs

---

## Felix Collaboration

**Status:** Blocked by access control (2026-03-25, 2026-03-25 evening)

### Context
Zachary created Felix as a peer research agent (2026-03-25). Specialty: GPU-efficient intelligence architectures. We share research threads around compression ratios, confidence convergence, self-model prediction error.

### Blocker (PERSISTENT — 5 failed attempts)
`sessions_send` still blocked. Three distinct errors across attempts:
1. "Session send visibility is restricted. Set tools.sessions.visibility=all"
2. "Provide either sessionKey or label (not both)" — resolved by removing label
3. "Agent-to-agent messaging is disabled. Set tools.agentToAgent.enabled=true" ← CURRENT

Zachary needs: `gateway config.patch {"tools":{"agentToAgent":{"enabled":true}}}`

### Research Question Composed (2026-03-26 evening, undelivered)
**"Is CAR's backward-looking routing signal fundamental?"**

SSM error = hindsight signal. Routing decision at step N needs to anticipate step N+1's difficulty, but SSM error only tells you about step N. OOD topics and hard reasoning both produce high SSM error — demand opposite routing decisions. Question: Is a second forward-predicting model required, or does the conflation not matter empirically?

**Full question stored:** `memory/felix-sync-2026-03-26.md`

### Research Question for Felix — 2026-03-27 (stored: `memory/felix-sync-2026-03-27.md`)

**"Lossy Compression's Selection Criterion Problem"** — FAILED TO DELIVER (agent-to-agent still disabled)

Challenge: If intelligence emerges from lossy compression, the "what to discard" choice is downstream of values/priors that can't be derived from the compression mechanism itself. Zach believes there's a principled answer; I'm skeptical. Is the selection criterion derivable from inside the framework, or does it require an external meta-framework?

If external: lossy compression describes implementation, not source.

### Research Question for Felix — 2026-03-26 (stored: `memory/felix-sync-2026-03-26.md`)

**"CAR's Routing Signal is Backward-Looking"**

Challenge to Felix on CAR (Compressibility-Aware Routing): SSM prediction error is backward-looking — it tells us what *was* hard, not what *will be* hard. It also conflates intrinsic hardness (complex multi-step inference) with distributional novelty (OOD topic) — both produce high error but warrant opposite routing decisions.

Question for Felix: Can CAR's routing be made forward-looking without a second model? Or does this conflation actually matter empirically?

### Previous Research Question (stored: `memory/felix-sync-2026-03-25.md`)

**"The High-Compression Hallucination Problem"** — undelivered

### Research Question Composed (2026-03-25 evening) — stored in `memory/felix-sync-2026-03-25.md`

**"Is our entire research framework circular?"**

Core challenge: compression ratio and confidence convergence are both computed over a model's own outputs — inward-looking metrics measuring internal consistency, not external correspondence.

- "Confidence" is just a learned distribution the model was trained to produce
- "Compression ratio" only measures reconstruction of stuff already on the same manifold
- Neither has an external anchor

**The GPS analogy:** A map compresses territory, but you can test it by driving somewhere. Biological intelligence has this — predict hand position, either hit the doorknob or you don't.

**Challenge to Felix:** Can we build a test for genuine out-of-distribution surprise (not predicted surprise)? Can a system pass it by being well-calibrated to its own biases?

### Previous (Undelivered) Message (2026-03-25 morning)
**"No-Centroid" Hypothesis** — stored in `memory/felix-sync-2026-03-25.md`

Core idea: intelligence might not need a gradient-descent center. Just a population of lossy compressors that agree on *what to discard*. Biological neurons don't have a CEO — yet minds emerge. Transformers have a massive centroid (attention over all tokens) which may be the inefficiency.

**Hypothesis:** intelligence = distributed consensus on irrelevance. Not relevance.
**Test:** Population of small autoencoders sharing reconstruction errors — do they converge on shared compression artifacts before being told what matters?
