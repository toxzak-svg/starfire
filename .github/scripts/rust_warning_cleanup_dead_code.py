from pathlib import Path
import re


def edit(path: str, literals=(), regexes=()) -> None:
    file = Path(path)
    text = file.read_text()
    original = text
    for old, new in literals:
        if old in text and new not in text:
            text = text.replace(old, new, 1)
    for pattern, replacement in regexes:
        text = re.sub(pattern, replacement, text, count=1, flags=re.S)
    if text != original:
        file.write_text(text)


edit(
    "lib/reasoning/mod.rs",
    regexes=[(
        r"\n    /// Check if a query looks like a math expression\..*?\n    fn looks_like_math\(&self, query: &str\) -> bool \{.*?\n    \}\n\n(?=    /// Handle a math query\.)",
        "\n",
    )],
)

edit(
    "lib/metacog/critic.rs",
    regexes=[
        (
            r"struct CritiqueRecord \{\n.*?\n\}",
            "struct CritiqueRecord {\n    was_correct: bool,\n}",
        ),
        (
            r"self\.critique_history\.push\(CritiqueRecord \{\n\s*concerns: concerns\.clone\(\),\n\s*answer: result\.answer\.clone\(\)\.unwrap_or_default\(\),\n\s*was_correct: false, // Will be updated later\n\s*\}\);",
            "self.critique_history.push(CritiqueRecord {\n            was_correct: false, // Will be updated later\n        });",
        ),
    ],
)

edit(
    "lib/prediction/belief_revision.rs",
    literals=[
        (
            '''            let confidence = self.compute_projection_confidence(&trajectory_field, horizon);

            vec![Prediction::new(
''',
            '''            let confidence = self.compute_projection_confidence(&trajectory_field, horizon);
            let revision_distance =
                self.cosine_distance(&conclusion.pre_state, &conclusion.post_state);

            vec![Prediction::new(
''',
        ),
        (
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
        ),
        (
            '''                    format!("Nearest known conclusion: {}", conclusion.predicate),
                    format!("Projection confidence: {:.3}", confidence),
''',
            '''                    format!("Nearest known conclusion: {}", conclusion.predicate),
                    format!("Template revision distance: {:.3}", revision_distance),
                    format!("Projection confidence: {:.3}", confidence),
''',
        ),
    ],
)

edit(
    "lib/prediction/basin.rs",
    literals=[
        ('''    /// Energy of the current state
    energy: f64,
''', ""),
        ('            energy: f64::INFINITY,\n', ""),
        (
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
        ),
        (
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
        ),
        (
            '''        let node = self.nodes.get(&node_id)?;
        
        Some(Prediction::new(
''',
            '''        let node = self.nodes.get(&node_id)?;
        let active_constraint_ids = self
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
        ),
        (
            '''                format!("Constraint count: {}", self.node_constraint_count(&node_id)),
''',
            '''                format!("Constraint count: {}", self.node_constraint_count(&node_id)),
                format!("Active constraints: {}", active_constraint_ids),
''',
        ),
        (
            '''                .filter(|c| c.constraint_type == ConstraintType::Causation && c.from == current)
''',
            '''                .filter(|constraint| {
                    constraint.satisfied
                        && constraint.constraint_type == ConstraintType::Causation
                        && constraint.from == current
                })
''',
        ),
    ],
)

edit(
    "lib/prediction/meta_prediction.rs",
    literals=[
        ('    pub engine: PredictionEngine,\n', ""),
        ('    fn new(engine: PredictionEngine) -> Self {\n', '    fn new() -> Self {\n'),
        ('    pub kind: PredictionKind,\n', ""),
        ('    fn new(kind: PredictionKind) -> Self {\n', '    fn new() -> Self {\n'),
        (
            '            engine_histories.insert(engine, EngineHistory::new(engine));\n',
            '            engine_histories.insert(engine, EngineHistory::new());\n',
        ),
        (
            '            kind_histories.insert(kind, KindHistory::new(kind));\n',
            '            kind_histories.insert(kind, KindHistory::new());\n',
        ),
    ],
    regexes=[
        (
            r"(fn new\(\) -> Self \{\n\s*EngineHistory \{\n)\s*engine,\n",
            r"\1",
        ),
        (
            r"(fn new\(\) -> Self \{\n\s*KindHistory \{\n)\s*kind,\n",
            r"\1",
        ),
    ],
)

edit(
    "lib/personality/mod.rs",
    regexes=[(
        r"\n    /// Detect what style Zach is using in this message\.\n    fn detect_zach_style\(.*?\n    \}\n\n(?=    /// Detect what style Zach is using from the RAW)",
        "\n",
    )],
)

edit(
    "lib/pain/cost_model.rs",
    literals=[(
        '        pattern_cost * source_mult * (1.0 + historical_pain)\n',
        '        pattern_cost * source_mult * (1.0 + historical_pain * self.decay_rate)\n',
    )],
)

edit(
    "lib/dreaming/dream_engine.rs",
    literals=[
        ('    validation_history: Vec<ValidationRecord>,\n', '    validation_history: Vec<bool>,\n'),
        ('            .filter(|v| v.supported)\n', '            .filter(|&&supported| supported)\n'),
    ],
    regexes=[
        (r"\n#\[derive\(Debug, Clone\)\]\nstruct ValidationRecord \{.*?\n\}\n", "\n"),
        (
            r"self\.validation_history\.push\(ValidationRecord \{.*?\n\s*\}\);",
            "self.validation_history.push(supported);",
        ),
    ],
)

edit(
    "lib/user_model/memory_model.rs",
    literals=[
        ('''    /// Topics that faded or were not retained
    forgotten_topics: Vec<String>,
''', ""),
        ('            forgotten_topics: Vec::new(),\n', ""),
    ],
)

edit(
    "lib/neural/neurons/quanot_neuron.rs",
    literals=[('use crate::quanot::{Quanot, QuanotResult};\n', 'use crate::quanot::Quanot;\n')],
    regexes=[(
        r"\n    /// Encode QuanotResult as a vector for downstream neurons\n    fn encode_result\(.*?\n    \}\n",
        "\n",
    )],
)
edit(
    "lib/neural/neurons/causal_neuron.rs",
    regexes=[(r"\n    fn decode_vector\(&self, vec: &\[f32\]\) -> String \{.*?\n    \}\n", "\n")],
)
edit(
    "lib/neural/neurons/goals_neuron.rs",
    literals=[('use crate::goals::{GoalEngine, GoalId, Goal};\n', 'use crate::goals::{GoalEngine, GoalId};\n')],
    regexes=[(r"\n    fn encode_goal\(&self, goal: &Goal\) -> Vec<f32> \{.*?\n    \}\n", "\n")],
)
edit(
    "lib/neural/neurons/fewshot_neuron.rs",
    regexes=[(r"\n    fn encode_text\(&self, text: &str\) -> Vec<f32> \{.*?\n    \}\n", "\n")],
)

edit(
    "lib/language_model/model.rs",
    literals=[
        ('    seq_len: usize,\n', ""),
        ('            seq_len,\n', ""),
    ],
    regexes=[(
        r"\n    // Mutable slice accessors used by backward_sequence.*?\n    fn o_bias_mut\(&mut self\) -> &mut \[f32\] \{.*?\n    \}\n",
        "\n",
    )],
)

edit(
    "lib/latent_roles.rs",
    literals=[
        ('    nodes: Vec<Atom>,\n', ""),
        ('            nodes,\n', ""),
    ],
)

edit(
    "lib/quanot/quantum_inspired.rs",
    literals=[
        ('        let J = vec![vec![0.0; n]; n];\n', '        let j = vec![vec![0.0; n]; n];\n'),
        ('        let result = sqa.solve(&J, &h);\n', '        let result = sqa.solve(&j, &h);\n'),
    ],
)
