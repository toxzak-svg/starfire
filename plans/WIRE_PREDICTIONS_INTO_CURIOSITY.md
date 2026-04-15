# Wire Predictions Into Curiosity — SPEC

## Problem

`PredictionCenter::generate()` runs after every conversation exchange and produces real foresight — questions Starfire should explore, gaps in understanding, anticipated topics. But `_predictions` is discarded. Meanwhile `CuriousEngine::generate_probe()` uses a nanos% template rotation with no connection to actual conversation state.

## Design

Add a `PredictionDriven` strategy to the curiosity engine. When generating a probe, if predictions exist, use them. The prediction question becomes the curiosity probe question.

### Changes

**1. `curiosity/mod.rs` — add prediction-driven probe strategy**

```rust
// In CuriousEngine::generate_probe(), add:
let probe = match strategy {
    // ... existing strategies ...
};

// NEW: Prediction-driven probe
if let Some(prediction_probe) = self.generate_prediction_driven_probe(predictions) {
    return Some(prediction_probe);
}

Some(probe)
```

**2. New method: `generate_prediction_driven_probe()`**

```rust
fn generate_prediction_driven_probe(&self, predictions: &[Prediction]) -> Option<CuriosityProbe> {
    // Pick the highest-priority pending prediction
    let pred = predictions.iter()
        .filter(|p| p.status == PredictionStatus::Pending)
        .max_by_key(|p| p.priority)?;

    // Convert prediction into a curiosity probe question
    let question = format_prediction_as_question(pred);

    Some(CuriosityProbe {
        id: uuid_simple(),
        question,
        topic: pred.topic.clone(),
        why_interested: format!("I predicted this would be important: {}", pred.rationale),
        related_concepts: pred.related_entities.clone(),
        depth: CuriosityDepth::from_prediction_depth(&pred.depth),
        status: ProbeStatus::Probing,
        tentative_answer: None,
        confidence: BeliefState::Suspects,
        discovered_at: now_timestamp(),
    })
}
```

**3. Prediction types need `topic`, `rationale`, `related_entities` fields**

Check if `Prediction` struct already has these. If not, extend the type.

**4. Runtime wiring**

In `runtime.rs` line 1616, change:
```rust
let predictions = self.prediction_center.generate(&context);
//              ^^^^^^^^^ predictions now used
```

Pass predictions into curiosity engine:
```rust
// After generating predictions
if self.curious_engine.should_fire() {
    if let Some(probe) = self.curious_engine.generate_probe_with_predictions(&predictions) {
        self.curious_engine.start_probe(probe);
    }
}
```

**5. Expression: express predictions naturally**

When Star expresses a thought, instead of generic "I've been thinking about X", use the prediction context:
```
"I've been wondering about [prediction.topic] — specifically [prediction.question]"
```

## Files to modify

- `lib/curiosity/mod.rs` — add `generate_probe_with_predictions()`, `PredictionDriven` strategy
- `lib/prediction/types.rs` — ensure Prediction has needed fields (topic, rationale, related_entities)
- `lib/runtime/mod.rs` — wire predictions into curiosity engine, don't discard `_predictions`
- `lib/curiosity/expression.rs` — use prediction context when expressing thoughts

## Outcome

- Curiosity probes become anticipatory, not random template rotation
- Question gravity predictions feed directly into what Star explores next
- Predictions that get confirmed (user asks about the predicted topic) strengthen future prediction accuracy
- Predictions that get rejected teach the prediction engine to deprioritize that direction
