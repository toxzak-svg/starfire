from pathlib import Path
import re


def read(path: str) -> tuple[Path, str]:
    file = Path(path)
    return file, file.read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(path: str, old: str, new: str) -> None:
    file, text = read(path)
    if new and new in text:
        return
    count = text.count(old)
    if count == 1:
        file.write_text(text.replace(old, new))
    elif count == 0:
        print(f"replacement already applied or unavailable: {path}")
    else:
        raise RuntimeError(f"ambiguous replacement in {path}: {count}")


def remove_once(path: str, old: str) -> None:
    replace_once(path, old, "")


def sub_once(path: str, pattern: str, replacement: str, *, marker: str | None = None) -> None:
    file, text = read(path)
    if marker and marker in text:
        return
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count == 1:
        file.write_text(updated)
    elif count == 0:
        print(f"regex already applied or unavailable: {path}: {pattern[:50]}")
    else:
        raise RuntimeError(f"ambiguous regex replacement in {path}: {count}")


# Remove an abandoned duplicate math detector. Math routing already enters the
# answer_math path elsewhere; this private predicate had no caller.
sub_once(
    "lib/reasoning/mod.rs",
    r"\n    /// Check if a query looks like a math expression\..*?\n    fn looks_like_math\(&self, query: &str\) -> bool \{.*?\n    \}\n\n(?=    /// Handle a math query\.)",
    "\n",
)

# Critique history only learns whether a critique was correct. Do not retain
# duplicate answer/concern payloads that are never queried.
sub_once(
    "lib/metacog/critic.rs",
    r"struct CritiqueRecord \{\n.*?\n\}",
    "struct CritiqueRecord {\n    was_correct: bool,\n}",
    marker="struct CritiqueRecord {\n    was_correct: bool,\n}",
)
sub_once(
    "lib/metacog/critic.rs",
    r"self\.critique_history\.push\(CritiqueRecord \{\n\s*concerns: concerns\.clone\(\),\n\s*answer: result\.answer\.clone\(\)\.unwrap_or_default\(\),\n\s*was_correct: false, // Will be updated later\n\s*\}\);",
    "self.critique_history.push(CritiqueRecord {\n            was_correct: false, // Will be updated later\n        });",
    marker="self.critique_history.push(CritiqueRecord {\n            was_correct: false",
)

# Surface snapshot provenance in prediction reasoning and use the complete
# belief-change template when reporting projected revisions.
replace_once(
    "lib/prediction/belief_revision.rs",
    '''            let confidence = self.compute_projection_confidence(&trajectory_field, horizon);

            vec![Prediction::new(
''',
    '''            let confidence = self.compute_projection_confidence(&trajectory_field, horizon);
            let revision_distance =
                self.cosine_distance(&conclusion.pre_state, &conclusion.post_state);

            vec![Prediction::new(
''',
)
replace_once(
    "lib/prediction/belief_revision.rs",
    '''                    format!("Trajectory length: {} exchanges", self.trajectory.len()),
                    format!("Consciousness proxy: {:.3}", current.consciousness_proxy),
''',
    '''                    format!("Trajectory length: {} exchanges", self.trajectory.len()),
                    format!("Current exchange: {}", current.exchange),
                    format!(
                        "Snapshot age: {} seconds",
                        crate::now_timestamp().saturating_sub(current.at)
                    ),
                    format!("Consciousness proxy: {:.3}", current.consciousness_proxy),
''',
)
replace_once(
    "lib/prediction/belief_revision.rs",
    '''                    format!("Nearest known conclusion: {}", conclusion.predicate),
                    format!("Projection confidence: {:.3}", confidence),
''',
    '''                    format!("Nearest known conclusion: {}", conclusion.predicate),
                    format!("Template revision distance: {:.3}", revision_distance),
                    format!("Projection confidence: {:.3}", confidence),
''',
)

# Constraint IDs and satisfaction flags are useful diagnostics. Apply only
# active constraints and report their IDs. Remove the unused cached energy.
remove_once(
    "lib/prediction/basin.rs",
    '''    /// Energy of the current state
    energy: f64,
''',
)
remove_once(
    "lib/prediction/basin.rs",
    '            energy: f64::INFINITY,\n',
)
replace_once(
    "lib/prediction/basin.rs",
    '''        self.constraints.iter()
            .filter(|c| c.from == *node_id || c.to == *node_id)
            .count()
''',
    '''        self.constraints
            .iter()
            .filter(|constraint| {
                constraint.satisfied
                    && (constraint.from == *node_id || constraint.to == *node_id)
            })
            .count()
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '''        let constraints_on: Vec<_> = self.constraints.iter()
            .filter(|c| c.to == *node_id)
            .collect();
''',
    '''        let constraints_on: Vec<_> = self
            .constraints
            .iter()
            .filter(|constraint| constraint.satisfied && constraint.to == *node_id)
            .collect();
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '''        Some(Prediction::new(
''',
    '''        let active_constraint_ids = self
            .constraints
            .iter()
            .filter(|constraint| {
                constraint.satisfied
                    && (constraint.from == node_id || constraint.to == node_id)
            })
            .map(|constraint| constraint.id.0.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Some(Prediction::new(
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '''                format!("Constraint count: {}", self.node_constraint_count(&node_id)),
''',
    '''                format!("Constraint count: {}", self.node_constraint_count(&node_id)),
                format!("Active constraints: {}", active_constraint_ids),
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '''                .filter(|c| c.constraint_type == ConstraintType::Causation && c.from == current)
''',
    '''                .filter(|constraint| {
                    constraint.satisfied
                        && constraint.constraint_type == ConstraintType::Causation
                        && constraint.from == current
                })
''',
)

# History values already live under typed HashMap keys. Remove duplicate copies.
remove_once(
    "lib/prediction/meta_prediction.rs",
    '    pub engine: PredictionEngine,\n',
)
replace_once(
    "lib/prediction/meta_prediction.rs",
    '    fn new(engine: PredictionEngine) -> Self {\n',
    '    fn new() -> Self {\n',
)
remove_once(
    "lib/prediction/meta_prediction.rs",
    '            engine,\n',
)
remove_once(
    "lib/prediction/meta_prediction.rs",
    '    pub kind: PredictionKind,\n',
)
replace_once(
    "lib/prediction/meta_prediction.rs",
    '    fn new(kind: PredictionKind) -> Self {\n',
    '    fn new() -> Self {\n',
)
remove_once(
    "lib/prediction/meta_prediction.rs",
    '            kind,\n',
)
replace_once(
    "lib/prediction/meta_prediction.rs",
    '            engine_histories.insert(engine, EngineHistory::new(engine));\n',
    '            engine_histories.insert(engine, EngineHistory::new());\n',
)
replace_once(
    "lib/prediction/meta_prediction.rs",
    '            kind_histories.insert(kind, KindHistory::new(kind));\n',
    '            kind_histories.insert(kind, KindHistory::new());\n',
)

# The raw-input style detector is the live implementation. Remove its older
# normalized-input twin rather than maintaining two drifting policies.
sub_once(
    "lib/personality/mod.rs",
    r"\n    /// Detect what style Zach is using in this message\.\n    fn detect_zach_style\(.*?\n    \}\n\n(?=    /// Detect what style Zach is using from the RAW)",
    "\n",
)

# Make historical pain decay operational.
replace_once(
    "lib/pain/cost_model.rs",
    '        pattern_cost * source_mult * (1.0 + historical_pain)\n',
    '        pattern_cost * source_mult * (1.0 + historical_pain * self.decay_rate)\n',
)

# Validation history only needs the outcome for the current aggregate metrics.
replace_once(
    "lib/dreaming/dream_engine.rs",
    '    validation_history: Vec<ValidationRecord>,\n',
    '    validation_history: Vec<bool>,\n',
)
sub_once(
    "lib/dreaming/dream_engine.rs",
    r"\n#\[derive\(Debug, Clone\)\]\nstruct ValidationRecord \{.*?\n\}\n",
    "\n",
)
sub_once(
    "lib/dreaming/dream_engine.rs",
    r"self\.validation_history\.push\(ValidationRecord \{.*?\n\s*\}\);",
    "self.validation_history.push(supported);",
    marker="self.validation_history.push(supported);",
)
replace_once(
    "lib/dreaming/dream_engine.rs",
    '            .filter(|v| v.supported)\n',
    '            .filter(|&&supported| supported)\n',
)

# Forgotten topics are derived from low strength. Remove the unused second store.
remove_once(
    "lib/user_model/memory_model.rs",
    '''    /// Topics that faded or were not retained
    forgotten_topics: Vec<String>,
''',
)
remove_once(
    "lib/user_model/memory_model.rs",
    '            forgotten_topics: Vec::new(),\n',
)

# Remove private placeholder codecs that have no call path. Their public neuron
# behavior remains unchanged.
replace_once(
    "lib/neural/neurons/quanot_neuron.rs",
    'use crate::quanot::{Quanot, QuanotResult};\n',
    'use crate::quanot::Quanot;\n',
)
sub_once(
    "lib/neural/neurons/quanot_neuron.rs",
    r"\n    /// Encode QuanotResult as a vector for downstream neurons\n    fn encode_result\(.*?\n    \}\n",
    "\n",
)
sub_once(
    "lib/neural/neurons/causal_neuron.rs",
    r"\n    fn decode_vector\(&self, vec: &\[f32\]\) -> String \{.*?\n    \}\n",
    "\n",
)
replace_once(
    "lib/neural/neurons/goals_neuron.rs",
    'use crate::goals::{GoalEngine, GoalId, Goal};\n',
    'use crate::goals::{GoalEngine, GoalId};\n',
)
sub_once(
    "lib/neural/neurons/goals_neuron.rs",
    r"\n    fn encode_goal\(&self, goal: &Goal\) -> Vec<f32> \{.*?\n    \}\n",
    "\n",
)
sub_once(
    "lib/neural/neurons/fewshot_neuron.rs",
    r"\n    fn encode_text\(&self, text: &str\) -> Vec<f32> \{.*?\n    \}\n",
    "\n",
)

# Remove unused layout metadata and obsolete per-gate gradient slices. Backward
# propagation writes through the single flat weights_mut buffer.
remove_once(
    "lib/language_model/model.rs",
    '    seq_len: usize,\n',
)
remove_once(
    "lib/language_model/model.rs",
    '            seq_len,\n',
)
sub_once(
    "lib/language_model/model.rs",
    r"\n    // Mutable slice accessors used by backward_sequence.*?\n    fn o_bias_mut\(&mut self\) -> &mut \[f32\] \{.*?\n    \}\n",
    "\n",
)

# GraphIndex derives its node set from the adjacency maps; storing it again adds
# no information.
remove_once(
    "lib/latent_roles.rs",
    '    nodes: Vec<Atom>,\n',
)
remove_once(
    "lib/latent_roles.rs",
    '            nodes,\n',
)

# Conventional Rust naming in the quantum annealing test.
replace_once(
    "lib/quanot/quantum_inspired.rs",
    '        let J = vec![vec![0.0; n]; n];\n',
    '        let j = vec![vec![0.0; n]; n];\n',
)
replace_once(
    "lib/quanot/quantum_inspired.rs",
    '        let result = sqa.solve(&J, &h);\n',
    '        let result = sqa.solve(&j, &h);\n',
)
