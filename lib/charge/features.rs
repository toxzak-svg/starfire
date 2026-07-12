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

/// Configuration for the H5 fixed-width, mask-blind residual projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedResidualProjectionConfig {
    pub bins: usize,
}

impl Default for FixedResidualProjectionConfig {
    fn default() -> Self {
        Self { bins: 8 }
    }
}

/// Fixed-width residual features that deliberately omit original residual length.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedResidualProjection {
    pub values: Vec<f32>,
}

impl FixedResidualProjection {
    const GLOBAL_FEATURES: usize = 9;
    const ABS_QUANTILES: usize = 5;
    const SIGNED_QUANTILES: usize = 5;
    const BIN_FEATURES: usize = 3;

    pub fn width(config: FixedResidualProjectionConfig) -> usize {
        Self::GLOBAL_FEATURES
            + Self::ABS_QUANTILES
            + Self::SIGNED_QUANTILES
            + config.bins * Self::BIN_FEATURES
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

/// Project a residual into a fixed-width H5 feature vector.
///
/// The projection is deterministic, finite, and label-blind. It intentionally
/// does not expose residual length, masks, charge kind, scope, or raw residuals.
pub fn fixed_residual_projection(
    residual: &[f32],
    config: FixedResidualProjectionConfig,
) -> FixedResidualProjection {
    let finite = sanitized_residual(residual);
    let count = finite.len().max(1) as f32;
    let sum = finite.iter().sum::<f32>();
    let mean = sum / count;
    let sum_squares = finite.iter().map(|value| value * value).sum::<f32>();
    let rms = (sum_squares / count).sqrt();
    let sum_abs = finite.iter().map(|value| value.abs()).sum::<f32>();
    let mean_abs = sum_abs / count;
    let variance = finite
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / count;
    let stddev = variance.sqrt();
    let max_abs = finite.iter().map(|value| value.abs()).fold(0.0, f32::max);
    let min = finite.iter().copied().fold(0.0, f32::min);
    let max = finite.iter().copied().fold(0.0, f32::max);
    let positive_fraction = finite
        .iter()
        .filter(|value| **value > NON_ZERO_EPSILON)
        .count() as f32
        / count;
    let negative_fraction = finite
        .iter()
        .filter(|value| **value < -NON_ZERO_EPSILON)
        .count() as f32
        / count;
    let near_zero_fraction = finite
        .iter()
        .filter(|value| value.abs() <= NON_ZERO_EPSILON)
        .count() as f32
        / count;

    let mut values = Vec::with_capacity(FixedResidualProjection::width(config));
    values.extend_from_slice(&[
        rms,
        mean_abs,
        stddev,
        max_abs,
        min,
        max,
        positive_fraction,
        negative_fraction,
        near_zero_fraction,
    ]);

    let mut abs_values: Vec<f32> = finite.iter().map(|value| value.abs()).collect();
    abs_values.sort_by(f32_total_cmp);
    for percentile in [0.10, 0.25, 0.50, 0.75, 0.90] {
        values.push(quantile(&abs_values, percentile));
    }

    let mut signed_values = finite.clone();
    signed_values.sort_by(f32_total_cmp);
    for percentile in [0.10, 0.25, 0.50, 0.75, 0.90] {
        values.push(quantile(&signed_values, percentile));
    }

    for bin in 0..config.bins {
        let start = bin * finite.len() / config.bins.max(1);
        let end = (bin + 1) * finite.len() / config.bins.max(1);
        let slice = &finite[start..end];
        values.extend_from_slice(&bin_summary(slice));
    }

    FixedResidualProjection { values }
}

/// Clone a charge and replace its residual with H5 fixed-width projection
/// features. The source charge is not mutated.
pub fn fixed_residual_feature_charge(
    charge: &Charge,
    config: FixedResidualProjectionConfig,
) -> Charge {
    let mut adapted = charge.clone();
    adapted.residual = fixed_residual_projection(&charge.residual, config).values;
    adapted
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

fn sanitized_residual(residual: &[f32]) -> Vec<f32> {
    if residual.is_empty() {
        return vec![0.0];
    }
    residual
        .iter()
        .map(|value| if value.is_finite() { *value } else { 0.0 })
        .collect()
}

fn f32_total_cmp(left: &f32, right: &f32) -> std::cmp::Ordering {
    left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
}

fn quantile(sorted_values: &[f32], percentile: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    let position = percentile.clamp(0.0, 1.0) * (sorted_values.len() - 1) as f32;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = position - lower as f32;
        sorted_values[lower] + (sorted_values[upper] - sorted_values[lower]) * weight
    }
}

fn bin_summary(values: &[f32]) -> [f32; 3] {
    if values.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let count = values.len() as f32;
    let mean = values.iter().sum::<f32>() / count;
    let mean_abs = values.iter().map(|value| value.abs()).sum::<f32>() / count;
    let rms = (values.iter().map(|value| value * value).sum::<f32>() / count).sqrt();
    [mean, mean_abs, rms]
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

        assert_eq!(
            residual_geometry(&gap.residual),
            residual_geometry(&contradiction.residual)
        );
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

    #[test]
    fn h5_fixed_projection_has_constant_width_across_residual_lengths() {
        let config = FixedResidualProjectionConfig { bins: 8 };
        let widths: Vec<usize> = [1usize, 3, 32, 64, 257]
            .into_iter()
            .map(|len| {
                let residual: Vec<f32> = (0..len)
                    .map(|index| ((index % 11) as f32 - 5.0) / 7.0)
                    .collect();
                fixed_residual_projection(&residual, config).values.len()
            })
            .collect();

        assert!(widths.iter().all(|width| *width == widths[0]));
        assert_eq!(widths[0], FixedResidualProjection::width(config));
    }

    #[test]
    fn h5_fixed_projection_is_deterministic_finite_and_label_blind() {
        let config = FixedResidualProjectionConfig { bins: 8 };
        let residual = [f32::NAN, -2.0, -0.5, 0.0, 0.25, f32::INFINITY, 4.0];
        let first = fixed_residual_projection(&residual, config);
        let second = fixed_residual_projection(&residual, config);

        assert_eq!(first, second);
        assert!(first.values.iter().all(|value| value.is_finite()));

        let gap = Charge::new(
            ChargeKind::EpistemicGap,
            residual.to_vec(),
            1.0,
            ChargeScope::Topic("dns".into()),
        );
        let contradiction = Charge::new(
            ChargeKind::Contradiction,
            residual.to_vec(),
            1.0,
            ChargeScope::Belief("prediction:7".into()),
        );

        assert_eq!(
            fixed_residual_projection(&gap.residual, config),
            fixed_residual_projection(&contradiction.residual, config)
        );
    }

    #[test]
    fn h5_fixed_projection_does_not_replace_h4_variable_adapter() {
        let short = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![1.0, -1.0, 0.0],
            1.0,
            ChargeScope::Global,
        );
        let long = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![1.0, -1.0, 0.0, 0.0, 0.0],
            1.0,
            ChargeScope::Global,
        );
        let fixed_short =
            fixed_residual_projection(&short.residual, FixedResidualProjectionConfig { bins: 8 });
        let fixed_long =
            fixed_residual_projection(&long.residual, FixedResidualProjectionConfig { bins: 8 });

        assert_eq!(fixed_short.values.len(), fixed_long.values.len());
        assert_ne!(
            ontology_feature_charge(&short).residual.len(),
            ontology_feature_charge(&long).residual.len()
        );
    }
}
