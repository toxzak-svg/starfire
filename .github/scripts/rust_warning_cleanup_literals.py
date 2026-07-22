from pathlib import Path


def edit(path: str, replacements) -> None:
    file = Path(path)
    text = file.read_text()
    original = text
    for old, new in replacements:
        if old in text:
            text = text.replace(old, new, 1)
    if text != original:
        file.write_text(text)


edit(
    "lib/prediction/belief_revision.rs",
    [
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
    [
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
    [
        ('    pub engine: PredictionEngine,\n', ""),
        ('    fn new(engine: PredictionEngine) -> Self {\n', '    fn new() -> Self {\n'),
        ('            engine,\n', ""),
        ('    pub kind: PredictionKind,\n', ""),
        ('    fn new(kind: PredictionKind) -> Self {\n', '    fn new() -> Self {\n'),
        ('            kind,\n', ""),
        (
            '            engine_histories.insert(engine, EngineHistory::new(engine));\n',
            '            engine_histories.insert(engine, EngineHistory::new());\n',
        ),
        (
            '            kind_histories.insert(kind, KindHistory::new(kind));\n',
            '            kind_histories.insert(kind, KindHistory::new());\n',
        ),
    ],
)

edit(
    "lib/pain/cost_model.rs",
    [(
        '        pattern_cost * source_mult * (1.0 + historical_pain)\n',
        '        pattern_cost * source_mult * (1.0 + historical_pain * self.decay_rate)\n',
    )],
)

edit(
    "lib/dreaming/dream_engine.rs",
    [
        ('    validation_history: Vec<ValidationRecord>,\n', '    validation_history: Vec<bool>,\n'),
        ('            .filter(|v| v.supported)\n', '            .filter(|&&supported| supported)\n'),
    ],
)

edit(
    "lib/user_model/memory_model.rs",
    [
        ('''    /// Topics that faded or were not retained
    forgotten_topics: Vec<String>,
''', ""),
        ('            forgotten_topics: Vec::new(),\n', ""),
    ],
)

edit(
    "lib/neural/neurons/quanot_neuron.rs",
    [('use crate::quanot::{Quanot, QuanotResult};\n', 'use crate::quanot::Quanot;\n')],
)
edit(
    "lib/neural/neurons/goals_neuron.rs",
    [('use crate::goals::{GoalEngine, GoalId, Goal};\n', 'use crate::goals::{GoalEngine, GoalId};\n')],
)

edit(
    "lib/language_model/model.rs",
    [
        ('    seq_len: usize,\n', ""),
        ('            seq_len,\n', ""),
    ],
)

edit(
    "lib/latent_roles.rs",
    [
        ('    nodes: Vec<Atom>,\n', ""),
        ('            nodes,\n', ""),
    ],
)

edit(
    "lib/quanot/quantum_inspired.rs",
    [
        ('        let J = vec![vec![0.0; n]; n];\n', '        let j = vec![vec![0.0; n]; n];\n'),
        ('        let result = sqa.solve(&J, &h);\n', '        let result = sqa.solve(&j, &h);\n'),
    ],
)
