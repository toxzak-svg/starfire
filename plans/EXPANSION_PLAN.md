# Starfire Technical Debt & Feature Expansion Plan

**Generated:** 2026-04-04
**Status:** Planned
**Scope:** Starfire (Rust) + Quanot (Python)

---

## Executive Summary

This plan covers three parallel tracks:
1. **Debug & Stabilize** — Fix all compile errors and warnings
2. **Reduce Technical Debt** — Clean up dead code, add tests
3. **Expand Architecture** — Build the 6 missing capability modules

---

## PART 1: DEBUG & STABILIZE

### 1.1 Rust Compilation Errors (7 blockers)

All located in `lib/`:

| # | File | Line | Error | Fix |
|---|------|------|-------|-----|
| 1 | `persistence/store.rs` | 860 | `use tempfile::tempdir` — crate missing | Add `tempfile = "4"` to `[dev-dependencies]` in `lib/Cargo.toml` |
| 2 | `persistence/store.rs` | 876 | `put_identity` missing `formed_at: i64` arg | Change to `store.put_identity("name", "Starfire", now).unwrap()` |
| 3 | `reasoning/analogy.rs` | 375 | `&items` is `&Vec<WorkingItem>`, needs `&[&WorkingItem]` | Change to `&items.iter().collect::<Vec<_>>()` or pass slice refs |
| 4 | `metacog/mod.rs` | 708 | `Belief::new("fire burns", ...)` needs `String` not `&str` | Add `.to_string()` |
| 5 | `metacog/mod.rs` | 709 | Same `.to_string()` | Same fix |
| 6 | `math/logic.rs` | 414 | `engine` not mutable | Change `let engine` to `let mut engine` |
| 7 | `math/proof.rs` | 288, 298 | `engine` not mutable | Change both to `let mut engine` |

### 1.2 Clippy Warnings (118 total, ~95 auto-fixable)

Run these after fixing errors:
```bash
cargo clippy --fix --lib -p star --allow-dirty  # applies ~33 auto-fixes
cargo clippy --fix --tests -p star --allow-dirty  # test fixes
```

Remaining manual fixes needed:
- **Dead fields** (remove or use): `protected_claims`, `fusion`, `metacog`, `thinker`, `why`, `pattern`, `depth`, `source_domain`, `target_domain`, `unanswered_questions`, `session_started`, `relation_type_from_word`, `Unknown` variant
- **Manual impl Default** → use `#[derive(Default)]` on: `Formality`, `Verbosity`, `ReasoningMode`, `AnalogyEngine`
- **Manual `is_multiple_of`** → use `.is_multiple_of()` in `conversation/mod.rs` (4 occurrences), `curiosity/mod.rs` (1)
- **Manual strip** → use `.strip_prefix()` in `conversation/mod.rs` (2), `math/symbolic.rs` (1), `math/logic.rs` (1)
- **`format!("...")`** → `.to_string()` in ~12 places
- **Redundant closures** in `store.rs`: 16 occurrences of `|row| row_to_X(row)` → `row_to_X`
- **`let_unit_value`** in `runtime/curious.rs:236`: remove `let _ =`
- **`collapsible_if`** in `reasoning/knowledge.rs` and `reasoning/mod.rs`: flatten nested ifs
- **`collapsible_str_replace`** in `runtime/mod.rs:467`: combine 4 replace calls
- **`question_mark`** in `runtime/mod.rs:1418`: replace `match` with `?`
- **`needless_return`** in `runtime/mod.rs:1835-1851`: remove 4 `return` statements
- **`needless_borrow`** in several places: remove unnecessary `&`
- **`len_zero`** in `reasoning/analogy.rs:212`: `shared.len() >= 1` → `!shared.is_empty()`
- **`derivable_impls`** for `Formality`, `Verbosity`, `ReasoningMode` — already noted
- **`inherent_to_string`** in `math/symbolic.rs` and `math/logic.rs`: implement `Display` trait instead
- **`needless_range_loop`** in `math/logic.rs:185,224`: use `.enumerate()`
- **`unreachable_patterns`** in `reasoning/knowledge.rs:102`: remove `_ => None` after all variants handled
- **`is_digit_ascii_radix`** in `math/symbolic.rs:455`: `ch.is_digit(10)` → `ch.is_ascii_digit()`
- **`useless_vec`** in `reasoning/mod.rs:643`: `vec![(x, vec![...])]` → `[(x, vec![...])]`
- **`useless_format`** in multiple files: `format!("...")` → `"...".to_string()`
- **`map_identity`** in `conversation/mod.rs:1446-1448`: remove `.map(|x| x)`

### 1.3 Quanot Python Issues

**Missing pytest:** Tests require `pytest` which isn't installed. Fix:
```bash
pip install pytest pytest-asyncio
```

**Test file issues:** All 8 test files import `pytest` but use unittest runner. Either:
- Option A: Install pytest and run with `pytest`
- Option B: Convert imports to unittest

---

## PART 2: TECHNICAL DEBT REDUCTION

### 2.1 Dead Code Elimination

Remove or properly integrate:
- `lib/persistence/identity_guard.rs` — `protected_claims` field never read
- `lib/reasoning/mod.rs` — `relation_type_from_word` function never used; `fusion` field never read
- `lib/conversation/mod.rs` — `metacog` field never read; `ConversationContext.unanswered_questions` and `session_started` never read; `Intent::Unknown` never constructed
- `lib/runtime/mod.rs` — `thinker` field never read
- `lib/metacog/mod.rs` — `CuriosityTopic.why` field never read
- `lib/curiosity/probes.rs` — `QuestionTemplate.pattern` and `depth` never read
- `lib/curiosity/connection.rs` — `KnownAnalogy.source_domain` and `target_domain` never read

### 2.2 Missing Tests (Coverage Gaps)

Current test files:
- `math/logic.rs` ✓
- `math/mod.rs` ✓
- `math/proof.rs` ✓
- `math/symbolic.rs` ✓
- `metacog/mod.rs` ✓
- `persistence/store.rs` ✓
- `reasoning/analogy.rs` ✓
- `reasoning/chain.rs` ✓
- `reasoning/knowledge.rs` ✓
- `voice/templates.rs` ✓

**Missing test coverage for:**
- `lib/reasoning/mod.rs` — core ReasoningEngine
- `lib/conversation/mod.rs` — Conversation
- `lib/context/mod.rs` — ContextRing, ReasoningMode
- `lib/runtime/mod.rs` — Runtime (massive file, most critical)
- `lib/runtime/curious.rs` — Curious module
- `lib/capabilities/mod.rs` — Capabilities
- `lib/capabilities/reader.rs` — FileReader
- `lib/knowledge/search.rs` — Knowledge search
- `lib/learning.rs` — Learning
- `lib/voice/mod.rs` — Voice
- `lib/voice/phrases.rs` — PhraseEngine

### 2.3 Test Quality Improvements

- All tests should use `#[cfg(test)]` properly
- Tests should not depend on real files or network
- Use tempfile for any I/O-dependent tests
- Add integration tests that span multiple modules
- Add property-based tests for core algorithms (analogy, reasoning, memory decay)

---

## PART 3: NEW CAPABILITY MODULES

### 3.1 Grounded World Model

**Purpose:** Give Starfire a perceptual representation of the world, binding Quanot's reservoir states to Starfire's symbolic knowledge.

**Files to create/modify:**
- `lib/world_model/mod.rs` — WorldModel struct
- `lib/world_model/perception.rs` — Bindings to Quanot perceptual input
- `lib/world_model/state.rs` — World state representation
- `lib/world_model/prediction.rs` — Predictive modeling
- Modify `lib/runtime/mod.rs` to integrate WorldModel

**Implementation approach:**
```rust
pub struct WorldModel {
    entities: HashMap<EntityId, Entity>,
    relations: Vec<Relation>,
    state_history: Vec<WorldState>,
    quanot_input: Option<QuanotPerception>,  // From Quanot bridge
}

pub struct QuanotPerception {
    pub reservoir_state: Vec<f64>,
    pub consciousness_proxy: f64,
    pub novelty: f64,
    pub creativity_scores: CreativityOutput,
}
```

**API to implement:**
- `update_from_perception(perception: QuanotPerception)` — update world model from Quanot
- `predict_outcome(action: &Action) -> Vec<PredictedState>`
- `query_world_state(query: &str) -> Vec<WorldFact>`
- `get_causal_neighbors(entity: &EntityId) -> Vec<(EntityId, Relation)>` — for causal discovery

### 3.2 Causal Discovery Engine

**Purpose:** Extract causal relationships from temporal patterns using Quanot's chaos analysis, integrate into Starfire's knowledge graph.

**Files to create:**
- `lib/causal/mod.rs` — CausalEngine
- `lib/causal/discovery.rs` — Pattern → Causality inference
- `lib/causal/validation.rs` — Hypothesis testing against observations
- `lib/causal/graph.rs` — Causal graph representation

**Key types:**
```rust
pub struct CausalEdge {
    pub cause: EntityId,
    pub effect: EntityId,
    pub confidence: f64,
    pub evidence: Vec<Observation>,
    pub temporal_lag: Option<i64>,
}

pub struct CausalHypothesis {
    pub candidate: CausalEdge,
    pub supporting_observations: usize,
    pub contradicting_observations: usize,
    pub confidence: ConfidenceState,
}
```

**Integration:** Results from `CausalDiscovery` flow into `ReasoningEngine` as candidate knowledge graph edges.

### 3.3 Hierarchical Goal Memory

**Purpose:** Give Starfire explicit goals with subgoals, temporal projection, and progress tracking.

**Files to create:**
- `lib/goals/mod.rs` — Goal hierarchy
- `lib/goals/planning.rs` — Planning and projection
- `lib/goals/tracking.rs` — Progress monitoring
- `lib/goals/motivation.rs` — Drive computation

**Key types:**
```rust
pub struct Goal {
    pub id: GoalId,
    pub content: String,
    pub parent: Option<GoalId>,
    pub subgoals: Vec<GoalId>,
    pub priority: f64,
    pub state: GoalState,  // Active, Suspended, Completed, Abandoned
    pub created_at: i64,
    pub deadline: Option<i64>,
    pub projected_outcome: String,
}

pub enum GoalState {
    Active { progress: f64 },
    Suspended { reason: String },
    Completed { conclusion: String },
    Abandoned { reason: String },
}
```

**Key methods:**
- `create_goal(content: &str, parent: Option<GoalId>) -> Goal`
- `decompose_goal(goal: &Goal) -> Vec<Goal>` — break into subgoals
- `project_outcome(goal: &Goal) -> String` — what success looks like
- `evaluate_progress(goal: &Goal, current_state: &WorldModel) -> f64`
- `generate_next_actions(goal: &Goal) -> Vec<Action>`

### 3.4 Cross-Modal Binding

**Purpose:** Process and reason over text + images + audio from chat export into unified representation.

**Files to create:**
- `lib/multimodal/mod.rs` — MultiModalEngine
- `lib/multimodal/binding.rs` — Cross-modal alignment
- `lib/multimodal/text.rs` — Text processing (existing — enhance)
- `lib/multimodal/image.rs` — Image understanding
- `lib/multimodal/audio.rs` — Audio/speech processing
- `lib/multimodal/fusion.rs` — Unified representation

**Data sources from chat export:**
- `conversations-*.json` — text conversations
- `image/` — uploaded images
- `audio/` — voice messages
- DALL-E generations

**Key type:**
```rust
pub enum Modality {
    Text(String),
    Image { path: String, description: Option<String> },
    Audio { path: String, transcription: Option<String> },
}

pub struct BoundContent {
    pub id: ContentId,
    pub modalities: Vec<Modality>,
    pub unified_embedding: Vec<f64>,
    pub timestamp: i64,
    pub provenance: String,
}
```

### 3.5 Continual Few-Shot Learning

**Purpose:** Rapid hypothesis formation from a handful of examples, without gradient updates.

**Files to create:**
- `lib/learning/fewshot.rs` — Few-shot learner
- `lib/learning/hypothesis.rs` — Hypothesis generation
- `lib/learning/eviction.rs` — When to abandon hypotheses

**Key types and approach:**
```rust
pub struct FewShotLearner {
    pub examples: Vec<Example>,
    pub working_hypotheses: Vec<Hypothesis>,
}

pub struct Example {
    pub input: String,
    pub output: String,
    pub domain: String,
    pub weight: f64,  // recency-weighted
}

pub struct Hypothesis {
    pub pattern: String,        // "X always leads to Y when Z"
    pub supporting_examples: Vec<Example>,
    pub confidence: f64,
    pub generality: f64,       // how broadly applicable
    pub predicted_applies_to: Vec<String>,
}
```

**Methods:**
- `learn_from_examples(examples: Vec<Example>) -> Vec<Hypothesis>`
- `test_hypothesis(h: &Hypothesis, new_example: &Example) -> HypothesisUpdate`
- `merge_similar(hypotheses: Vec<Hypothesis>) -> Vec<Hypothesis>`

### 3.6 Self-Directed Curriculum

**Purpose:** Active gap identification and autonomous learning goal generation.

**Files to create/modify:**
- `lib/curriculum/mod.rs` — CurriculumEngine
- `lib/curriculum/gap_analysis.rs` — Knowledge gap detection
- `lib/curriculum/scheduler.rs` — Learning session scheduling
- `lib/curriculum/self_directed.rs` — Autonomous exploration
- Modify `lib/runtime/mod.rs` to activate Layer 4 curiosity-driven behavior

**Key types:**
```rust
pub struct KnowledgeGap {
    pub topic: String,
    pub gap_type: GapType,  // CompleteIgnorance, Misconception, Incomplete, Unconnected
    pub urgency: f64,       // how important to fill
    pub existing_beliefs: Vec<BeliefId>,
    pub connected_topics: Vec<String>,
}

pub enum GapType {
    CompleteIgnorance,     // know nothing
    Misconception { wrong: BeliefId, corrections: Vec<BeliefId> },
    Incomplete { partial: BeliefId, missing_aspects: Vec<String> },
    Unconnected { isolated: BeliefId, potential_connections: Vec<String> },
}

pub struct LearningTask {
    pub gap: KnowledgeGap,
    pub chosen_strategy: LearningStrategy,
    pub questions_to_ask: Vec<String>,
    pub projected_outcome: String,
    pub priority: f64,
}

pub enum LearningStrategy {
    AskUser(String),           // ask Zachary directly
    ExploreKnowledgeGraph,      // internal reasoning
    QueryExternalSource,       // web search (future)
    RunQuanotSimulation,       // use Quanot for creative exploration
}
```

**Integration point:** This activates the dormant Layer 4 Emergence behavior described in SPEC.md — curiosity-driven gap hunting that runs in background.

---

## PART 4: IMPLEMENTATION ORDER

### Week 1: Debug & Stabilize
- [ ] Fix 7 Rust compilation errors
- [ ] Run `cargo clippy --fix` for auto-fixes
- [ ] Manually fix remaining clippy warnings (prioritize: dead code, useless_format, needless_return)
- [ ] Fix quanot pytest dependency
- [ ] Verify all tests pass
- [ ] Add Cargo.lock to git

### Week 2: Technical Debt + Test Coverage
- [ ] Remove dead fields and functions
- [ ] Add `#[derive(Default)]` where appropriate
- [ ] Add tests for: `reasoning/mod.rs`, `conversation/mod.rs`, `context/mod.rs`
- [ ] Add tests for: `runtime/mod.rs` (focus on high-value functions)
- [ ] Add tests for: `capabilities/mod.rs`, `learning.rs`, `voice/mod.rs`

### Week 3: Cross-Modal Binding + World Model
- [ ] Implement `lib/multimodal/mod.rs` core
- [ ] Integrate with existing chat export parser
- [ ] Implement `lib/world_model/mod.rs`
- [ ] Bridge Quanot → Starfire perception flow

### Week 4: Causal Discovery + Goal Memory
- [ ] Implement `lib/causal/mod.rs`
- [ ] Integrate causal edges into knowledge graph
- [ ] Implement `lib/goals/mod.rs`
- [ ] Goal → action pipeline

### Week 5: Few-Shot Learning + Self-Directed Curriculum
- [ ] Implement `lib/learning/fewshot.rs`
- [ ] Implement `lib/curriculum/mod.rs`
- [ ] Integrate curiosity-driven exploration into Runtime
- [ ] Activate Layer 4 emergence behaviors

### Week 6: Integration & Polish
- [ ] Full pipeline integration test
- [ ] Performance profiling
- [ ] API consistency pass
- [ ] Documentation updates
- [ ] Final clippy + warnings clean

---

## File Manifest

### New Files to Create (Rust)
```
lib/world_model/mod.rs
lib/world_model/perception.rs
lib/world_model/state.rs
lib/world_model/prediction.rs
lib/causal/mod.rs
lib/causal/discovery.rs
lib/causal/validation.rs
lib/causal/graph.rs
lib/goals/mod.rs
lib/goals/planning.rs
lib/goals/tracking.rs
lib/goals/motivation.rs
lib/multimodal/mod.rs
lib/multimodal/binding.rs
lib/multimodal/image.rs
lib/multimodal/audio.rs
lib/multimodal/fusion.rs
lib/learning/fewshot.rs
lib/learning/hypothesis.rs
lib/learning/eviction.rs
lib/curriculum/mod.rs
lib/curriculum/gap_analysis.rs
lib/curriculum/scheduler.rs
lib/curriculum/self_directed.rs
tests/test_reasoning_engine.rs
tests/test_conversation.rs
tests/test_world_model.rs
tests/test_causal.rs
tests/test_goals.rs
tests/test_multimodal.rs
tests/test_curriculum.rs
```

### Files to Modify
```
lib/Cargo.toml                          — add tempfile dev-dep, new lib entries
lib/lib.rs                              — add new module declarations
lib/runtime/mod.rs                      — integrate WorldModel, Goals, Curriculum
lib/persistence/memory.rs               — add GoalDomain for memory
lib/reasoning/mod.rs                    — integrate Causal edges
lib/reasoning/knowledge.rs              — add causal relation types
```

### New Files (Python/Quanot)
```
quanot/tests/test_integration_pipeline.py
quanot/tests/test_causal_discovery.py
```

---

## Testing Strategy

**Rust:**
- Unit tests for every new module
- Integration tests spanning Quanot bridge → Starfire reasoning
- Property-based tests for core algorithms
- Benchmarks for performance-critical paths

**Python:**
- pytest for quanot (install as dependency)
- Integration tests for Quanot ↔ Starfire bridge
- Chaos system validation tests

**Success criteria:**
- `cargo clippy` → 0 warnings
- `cargo test` → 100% passing
- `cargo test --doc` → all doc tests passing
- Python: `pytest` → 100% passing
- Coverage: aim for 70%+ on new modules

---

## Dependencies & Crates to Add

### Rust (Cargo.toml)
```toml
# New in lib/
tempfile = "4"           # dev-dependency for tests
proptest = "1"           # property-based testing (optional)

# New dependencies
ahash = "0.8"            # faster HashMap for world model
```

### Python (requirements.txt or environment.yml)
```yaml
pytest = ">=8.0"
pytest-asyncio = ">=0.23"
```

---

## Notes

- Quanot is embedded in Starfire as `projects/starfire/quanot/` — bridge is documented in `quanot/INTEGRATION_PLAN.md`
- The Rust project uses workspace Cargo.toml at root — individual crate Cargo.toml files inherit from it
- All new modules should follow the existing module structure: `pub struct`, `impl`, `#[cfg(test)]`
- Breaking changes to public API must update `lib/api.rs` and getchangelog entry
