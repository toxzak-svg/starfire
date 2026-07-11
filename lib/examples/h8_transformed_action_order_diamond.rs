#![allow(dead_code)]

fn directed_normalized_motion(witness: &star::charge::OutcomeWitness) -> Option<f64> {
    if !witness.before.is_finite() || !witness.after.is_finite() {
        return None;
    }
    let raw = match witness.direction {
        star::charge::ImprovementDirection::HigherIsBetter => witness.after - witness.before,
        star::charge::ImprovementDirection::LowerIsBetter => witness.before - witness.after,
    };
    let scale = witness.before.abs().max(witness.after.abs()).max(1.0);
    Some((raw / scale).clamp(-1.0, 1.0))
}

include!("h8_transformed_action_order/part1.rs");
include!("h8_transformed_action_order/part2.rs");
include!("h8_transformed_action_order/part3.rs");
include!("h8_transformed_action_order/part4.rs");
