//! CHARGE-native feature adapters for empirical ontology search.
//!
//! These adapters derive deterministic geometry from an existing residual vector.
//! They do not inspect charge kind, scope names, emitter identity, resolver labels,
//! or outcome history. The resulting features can therefore be replayed against
//! future CHARGE snapshots without carrying a human-authored class answer key.

use super::Charge;

const NON_ZERO_EPSILON: f32 = 1e-6;

/// Deterministic summary of one CHARGE residual vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResidualGeometry {
    pub rms: f32,
    pub mean_abs: f32,
    pub non_zero_fraction: f32,
    pub positive_fraction: f32,
    pub max_abs: f32,
}

impl ResidualGeometry {
    pub const FEATURE_COUNT: usize = 5;

    pub fn as_features(self) -> [f32; Self::FEATURE_COUNT] {
        [
            self.rms,
            self.mean_abs,
            self.non_zero_fraction,
            self.positive_fraction,
            self.max_abs,
        ]
    }
}

/// Measure residual geometry without using semantic metadata.
pub fn residual_geometry(residual: &[f32]) -> ResidualGeometry {
    if residual.is_empty() {
        return ResidualGeometry {
            rms: 0.0,
            mean_abs: 0.0,
            non_zero_fraction: 0.0,
            positive_fraction: 0.0,
            max_abs: 0.0,
        };
    }

    let finite: Vec<f32> = residual
        .iter()
        .copied()
        .map(|value| if value.is_finite() { value } else { 0.0 })
        .collect();
    let count = finite.len() as f32;
    let sum_squares = finite.iter().map(|value| value * value).sum::<f32>();
    let sum_abs = finite.iter().map(|value| value.abs()).sum::<f32>();
    let non_zero = finite
        .iter()
        .filter(|value| value.abs() > NON_ZERO_EPSILON)
        .count() as f32;
    let positive = finite
        .iter()
        .filter(|value| **value > NON_ZERO_EPSILON)
        .count() as f32;
    let max_abs = finite.iter().map(|value| value.abs()).fold(0.0, f32::max);

    ResidualGeometry {
        rms: (sum_squares / count).sqrt(),
        mean_abs: sum_abs / count,
        non_zero_fraction: non_zero / count,
        positive_fraction: positive / count,
        max_abs,
    }
}

/// Clone a charge and prepend residual-geometry features to its raw residual.
///
/// Layout:
///
/// ```text
/// [rms, mean_abs, non_zero_fraction, positive_fraction, max_abs, ...raw residual]
/// ```
///
/// The original charge is not mutated. Callers must apply the same adapter before
/// both fitting and routing a learned ontology.
pub fn ontology_feature_charge(charge: &Charge) -> Charge {
    let geometry = residual_geometry(&charge.residual);
    let mut adapted = charge.clone();
    let mut residual = Vec::with_capacity(ResidualGeometry::FEATURE_COUNT + charge.residual.len());
    residual.extend_from_slice(&geometry.as_features());
    residual.extend_from_slice(&charge.residual);
    adapted.residual = residual;
    adapted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

    #[test]
    fn geometry_is_deterministic_and_label_blind() {
        let gap = Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0, 0.0, -1.0, 0.0],
            0.5,
            ChargeScope::Topic("dns".into()),
        );
        let contradiction = Charge::new(
            ChargeKind::Contradiction,
            vec![1.0, 0.0, -1.0, 0.0],
            0.9,
            ChargeScope::Belief("prediction:7".into()),
        );

        assert_eq!(residual_geometry(&gap.residual), residual_geometry(&contradiction.residual));
        assert_eq!(
            &ontology_feature_charge(&gap).residual[..ResidualGeometry::FEATURE_COUNT],
            &ontology_feature_charge(&contradiction).residual[..ResidualGeometry::FEATURE_COUNT]
        );
    }

    #[test]
    fn adapter_preserves_raw_residual_after_geometry_prefix() {
        let charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![0.5, -0.25, 0.0, 0.75],
            1.0,
            ChargeScope::Global,
        );
        let adapted = ontology_feature_charge(&charge);
        let geometry = residual_geometry(&charge.residual).as_features();

        assert_eq!(
            &adapted.residual[..ResidualGeometry::FEATURE_COUNT],
            geometry.as_slice()
        );
        assert_eq!(
            &adapted.residual[ResidualGeometry::FEATURE_COUNT..],
            charge.residual.as_slice()
        );
        assert_eq!(adapted.kind, charge.kind);
        assert_eq!(adapted.scope, charge.scope);
        assert_eq!(adapted.magnitude, charge.magnitude);
    }

    #[test]
    fn non_finite_values_do_not_poison_geometry() {
        let geometry = residual_geometry(&[f32::NAN, 1.0, f32::INFINITY, -1.0]);
        for value in geometry.as_features() {
            assert!(value.is_finite());
        }
        assert_eq!(geometry.non_zero_fraction, 0.5);
    }
}
