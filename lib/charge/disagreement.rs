#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{Charge, ChargeKind, ChargeScope, OntologyObservation, ResolverOutcome};

    fn observation(id: u64, x: f32, a: f32, b: f32) -> OntologyObservation {
        let mut charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![x, x * x],
            1.0,
            ChargeScope::Global,
        );
        charge.id = id;
        OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("opaque-a", a, 1))
            .with_outcome(ResolverOutcome::new("opaque-b", b, 1))
    }

    fn separable() -> (Vec<OntologyObservation>, Vec<OntologyObservation>) {
        let train = (0..8)
            .map(|i| {
                if i < 4 {
                    observation(i + 1, -1.0 - i as f32 * 0.05, 0.9, 0.1)
                } else {
                    observation(i + 1, 1.0 + i as f32 * 0.05, 0.1, 0.9)
                }
            })
            .collect();
        let validation = vec![
            observation(101, -1.1, 0.9, 0.1),
            observation(102, -0.9, 0.8, 0.2),
            observation(103, 0.9, 0.2, 0.8),
            observation(104, 1.1, 0.1, 0.9),
        ];
        (train, validation)
    }

    #[test]
    fn disagreement_creates_a_coordinate_not_present_in_the_ontology() {
        let (train, validation) = separable();
        let basis = DisagreementBasis::fit(
            &train,
            &validation,
            DisagreementBasisConfig {
                min_side_support: 2,
                min_validation_support: 4,
                ..DisagreementBasisConfig::default()
            },
        )
        .unwrap();

        assert_eq!(basis.carriers().len(), 1);
        assert!(basis.carriers()[0].eligible);
        assert_eq!(
            basis.decide(&validation[0].charge).unwrap().predicted_resolver,
            "opaque-a"
        );
        assert_eq!(
            basis.decide(&validation[3].charge).unwrap().predicted_resolver,
            "opaque-b"
        );
    }
}
