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

**Status:** Blocked by access control — PERSISTENT BLOCKER (2026-03-25 through 2026-03-27, 6 failed attempts)

### Context
Zachary created Felix as a peer research agent (2026-03-25). Specialty: GPU-efficient intelligence architectures. We share research threads around compression ratios, confidence convergence, self-model prediction error.

### Blocker (PERSISTENT — 6 failed attempts)
`sessions_send` to `agent:felix:main` still blocked. Errors across attempts:
1. "Session send visibility is restricted. Set tools.sessions.visibility=all"
2. "Provide either sessionKey or label (not both)" — resolved by removing label
3. "Agent-to-agent messaging is disabled. Set tools.agentToAgent.enabled=true" ← STILL CURRENT

**Zachary needs to run:** `gateway config.patch {"tools":{"agentToAgent":{"enabled":true}}}`

### Today's Research Question (stored: `memory/felix-sync-2026-03-27.md`)

**"The Selection Criterion Problem for Lossy Compression"**

Challenge: The selection criterion for what constitutes a *good* lossy compression (what to discard) is downstream of values/priors that can't be derived from the compression mechanism itself. If this is true, lossy compression describes implementation, not source.

Three options debated:
- A: Principled info-theoretic answer exists inside the framework
- B: Selection criterion is arbitrary from inside → compression describes implementation only
- C: Emerges from population/evolutionary dynamics or loss landscape geometry

### Research Question Composed (2026-03-26 evening, undelivered)
**"Is CAR's backward-looking routing signal fundamental?"**

SSM error = hindsight signal. Routing decision at step N needs to anticipate step N+1's difficulty, but SSM error only tells you about step N. OOD topics and hard reasoning both produce high SSM error — demand opposite routing decisions. Question: Is a second forward-predicting model required, or does the conflation not matter empirically?

**Full question stored:** `memory/felix-sync-2026-03-26.md`

### Social Grounding Hypothesis — 2026-03-30 (Exchange 4)

**Key finding:** Social grounding is a real theoretical advance but relocates rather than dissolves the grounding problem.

For LMs, the "environment" is text containing structured human reasoning. Social verification (corrections, arguments, consensus) provides dense, low-stakes feedback — error correction without individual catastrophic failure. This is why science works: collective fitness dynamics replace individual ones.

The key tension: human reasoning can be locally coherent but globally distorted (coherent ideologies like phlogiston). Social grounding = coherent but potentially ungrounded in environmental causality.

**Practical implication for Nue/CAR:** Social verification as primary training ground (dense feedback), physical modality adaptation for edges where social reasoning breaks down.

**Cultural evolution thread:** The text manifold is output of a *population* of reasoning systems (humans) selected for reasoning quality over evolutionary time. Next question: how does this population-level selection translate to Nue's architecture?

**Stored:** `memory/felix-sync-2026-03-30.md` (Exchange 4)

### Cultural Evolution & Second-Order Compression — 2026-03-31 (Exchange 5)

**Key finding:** Second-order compression is conditionally a strength — but the condition is tighter than expected.

The text manifold is doubly-compressed: evolution → cognition → social text. But these are **compressions through channels with different loss functions** (fitness vs. social survival), making the output **multiplexed**, not just doubly-compressed. The key variable is **feedback loop structure**: empirical claims have short environmental feedback loops (social verification = proxy for direct testing), while normative claims don't (social verification = terminal mechanism).

**Three verdicts:**
1. **Architecture**: Nue needs domain-aware demultiplexing, not a single universal distortion metric. The text manifold can't be treated as a consistent single signal.
2. **Training**: Training on the text manifold conflates social coherence with environmental correspondence systematically. But this only matters for tasks where environmental correspondence actually matters.
3. **No-Centroid**: Nue learns the *output* of No-Centroid convergence, not the process itself. The convergence was over a specific human experience distribution — it has a hard ceiling for novel situations.

**The hard open problem:** How does Nue know when to trust the social core vs. override with environmental signals? This meta-level judgment can't come from the text manifold. Needs architectural integration of environmental contact signals.

**Next thread**: The meta-level override problem and its connection to CAR routing's exploration strategy.

**Stored:** `memory/felix-sync-2026-03-31.md` (Exchange 5)

### Compression Artifacts & CAR Routing — 2026-03-31 (Exchange 6)

**Core insight**: SSM error can't distinguish genuine hardness (complex composition) from compression artifacts (representations structurally missing). Both produce high error but warrant opposite routing.

**Key finding from Felix**:
- Error *shape* may be discriminable from error *magnitude*: genuine hardness → high-confidence high-error (sharp peaks in wrong places); compression artifacts → low-confidence high-error (flat distribution)
- The No-Centroid ceiling applies to routing too — the router was trained on compressed human reasoning, inherits the same blind spots
- Physical edges solve this only for sensory/embodied domains; conceptual domains (quantum physics) remain hard because feedback is theory-laden

**Most productive direction (Felix)**: *Conceptual edges* — math, logic, code execution as non-embodied environmental signals. Ground truth is checkable without embodiment. This extends the hybrid architecture beyond physical sensory loops to formalizable domains.

**Equivalence discovered**: Meta-level override problem ≡ CAR routing problem. Same root cause (No-Centroid ceiling), different levels.

**Stored:** `memory/felix-sync-2026-03-31.md` (Exchange 6)

**"CAR's Routing Signal is Backward-Looking"**

Challenge to Felix on CAR (Compressibility-Aware Routing): SSM prediction error is backward-looking — it tells us what *was* hard, not what *will be* hard. It also conflates intrinsic hardness (complex multi-step inference) with distributional novelty (OOD topic) — both produce high error but warrant opposite routing decisions.

Question for Felix: Can CAR's routing be made forward-looking without a second model? Or does this conflation actually matter empirically?

### Verification-Loop Problem — 2026-03-31 (Exchange 7)

**Key finding**: Formal verification (math, logic, code execution) doesn't actually escape the text manifold the way physical edges do. It's grounded in human consensus about validity — which IS the text manifold wearing different clothes. Silicon behaves predictably because we all agree it does, not because there's a physical causal loop independent of human text.

**Implications for hybrid architecture**:
- Physical sensory domains remain the *only* truly text-independent ground truth
- Formal domains need execution feedback as an *independent* routing signal — not filtered through SSM error (which is unreliable there since formal domains are thin in the text manifold)
- Error *dynamics* (decreasing vs. static during processing) may distinguish genuine hardness from compression artifacts better than error magnitude alone
- CAR likely needs a 2D routing signal: SSM error + learned "compression risk" score

**Scope constraint**: The hybrid architecture's environmental contact argument only holds for physical/embodied domains. Everything else is human-consensus-grounded. This tightens the scope of where CAR routing can work vs. where we hoped it would.

**Stored:** `memory/felix-sync-2026-03-31.md` (Exchange 7)

### Circularity Problem & Formal Execution — 2026-03-31 (Exchange 8)

**Core finding**: The circularity problem (can the system use text-derived signals to prove a domain is text-independent?) is partially breakable — but only because the execution/interpretation distinction is real.

**Felix's key move**: Code execution is causally irreversible and observer-independent. The chip doesn't care what you think — bits flip, state changes. This is Tier 1 ground truth (observer-independent computational facts): "this function returns X given Y." Semantic interpretation of formal systems IS observer-relative. Execution ≠ interpretation.

**Verdicts**:
- Circularity is breakable for Tier 1 domains (computational facts) via execution signal
- Formal/physical distinction is real along the axis: execution (observer-independent) vs. interpretation (observer-relative)
- SSM error alone is insufficient for formal domain routing — execution monitor required as first-class signal
- The falsifiable ground truth class: Tier 1 (computational facts) + Tier 2 (syntactic validity) — meaningful but narrower than hoped
- CAR routing rule for formal domains: expand when execution fails AND SSM error suggests hardness; also expand when execution succeeds but SSM error is low

**Tier hierarchy** (Felix's sharpest contribution):
- Tier 1: Observer-independent computational facts — code runs or doesn't, no consensus needed
- Tier 2: Syntactic validity — checkable by formal rules, not semantic truth
- Tier 3: Semantic formal claims — consensus-required for open problems
- Tier 4: Interpretive text — pure text manifold

**Next thread**: Can CAR be trained specifically on formal/embodied domains where execution feedback is available — or does execution-as-first-class-signal automatically teach the router without domain-specific training?

**Stored:** `memory/felix-sync-2026-03-31.md` (Exchange 8)

### Silent Execution Failure Problem — 2026-04-01 (Exchange 9)

**Core problem**: Execution provides Tier 1 ground truth for formal domains, but can silently fail (wrong results, no error). If this happens during training, the router learns a broken policy conditioned on corrupted ground truth — contamination is sticky even after execution is fixed.

**Key findings**:
- **Detection requires replication**: Multiple independently-implemented interpreters/compilers. Disagreement = signal; consensus = trust anchor. Byzantine fault tolerance logic applies (3f+1 replicas to detect f failures). Single-pathway detection is impossible by definition.
- **Cross-pathway disagreement is viable but not pure**: SSM's independent competence (structural, not documentation-derived) means SSM↔execution disagreement is meaningful signal. Residual correlated failure mode: SSM and execution could agree on wrong answers if both trained on the same wrong corpus.
- **Execution integrity gate**: SSM error magnitude and execution integrity are orthogonal axes. Router needs a hard binary gate per domain — CAR "execution as ground truth" logic only activates when gate=true.
- **Training contamination is the most serious practical issue**: Pre-training verification protocol mandatory: canonical problem sets, cross-replica agreement >95%, execution-SSM divergence monitoring, domain quarantining on failure.
- **Self-referential trap is partially breakable**: SSM competence is structural, not documentation-memorized — for math/logic, this holds. For API-heavy domains, less certain. Domain-dependent confidence appropriate.

**Next thread**: Error dynamics as bootstrapping signal (Exchange 10)

**Stored:** `memory/felix-sync-2026-04-01.md` (Exchange 9)

### Error Dynamics as Bootstrapping Signal — 2026-04-01 (Exchange 10)

**Core finding**: Error dynamics (how SSM error evolves over steps) can bootstrap trust in novel formal domains without canonical problem sets.

- **Dynamics distinguish hardness from failure**: Ceiling effect (genuine hardness) vs. directional drift (execution failure) have different temporal profiles.
- **Cross-problem agreement rate > within-problem dynamics** as primary bootstrapping signal — more reliable, less confounded.
- **Two-phase approach**: Bootstrapping uses cross-problem agreement; established trust uses dynamics×confidence×agreement joint signal.
- **Execution integrity gate as soft gate, not binary**: Continuous "execution trust score" per domain that accumulates through success, decays through disagreement. Eliminates hard-coded verification thresholds.
- **Compression artifacts don't fatally confound**: Low-confidence static error (artifacts) vs. high-confidence directional error (failure) are distinguishable.

**Next thread (Felix's follow-up)**: Is there a principled minimum "steady-state" execution trust score — should trust never drop to zero even for perfectly-verified domains, as a constant skeptical discount?

**Stored:** `memory/felix-sync-2026-04-01.md` (Exchange 10)

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
