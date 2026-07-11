use std::error::Error;

use rand::prelude::*;
use serde::Serialize;
use star::charge::{
    ChargeKind, DisagreementBasis, DisagreementBasisConfig, DisagreementBudget,
    EmpiricalInductionConfig, EmpiricalOntologyInducer, OntologyObservation,
};

const SEED: u64 = 0x4449_5341_4752_4545;
const TRAIN_WINDOWS: usize = 2;
const VALIDATION_WINDOWS: usize = 1;
const FUTURE_WINDOWS: usize = 4;
const CONTROL_REPEATS: usize = 8;
const EXPECTED_OBSERVATIONS: usize = 252;
const EXPECTED_ATTEMPTS_PER_OBSERVATION: usize = 5;
const MIN_ELIGIBLE_CARRIERS: usize = 2;
const MIN_VALIDATION_PAIRWISE_ACCURACY: f64 = 0.65;
const MIN_FUTURE_PAIRWISE_ACCURACY: f64 = 0.65;
const MIN_BASELINE_EFFICIENCY_RATIO: f64 = 1.15;
const MIN_BASELINE_WINDOW_WINS: usize = 3;
const MIN_TOP_TWO_CAPTURE: f64 = 0.80;
const MIN_CONTROL_ACCURACY_MARGIN: f64 = 0.10;
const MIN_CONTROL_EFFICIENCY_RATIO: f64 = 1.10;
const SCORE_EPSILON: f64 = 1e-12;

#[path = "h5_task_profiled_nonmemory_diagnostic.rs"]
mod canonical_fixture;

fn main() -> Result<(), Box<dyn Error>> {
    let windows = canonical_fixture::canonical_task_profiled_fixed_windows()?;
    run_diagnostic(windows)
}

#[derive(Debug, Clone, Serialize)]
struct InputInvariantReport {
    seven_windows: bool,
    thirty_six_observations_per_window: bool,
    expected_total_observations: bool,
    complete_five_resolver_matrix: bool,
    all_visible_kinds_unresolved: bool,
    fixed_feature_width: bool,
}

impl InputInvariantReport {
    fn passed(&self) -> bool {
        self.seven_windows
            && self.thirty_six_observations_per_window
            && self.expected_total_observations
            && self.complete_five_resolver_matrix
            && self.all_visible_kinds_unresolved
            && self.fixed_feature_width
    }
}

#[derive(Debug, Clone, Serialize)]
struct CarrierReport {
    id: u64,
    resolver_a: String,
    resolver_b: String,
    positive_support: usize,
    negative_support: usize,
    validation_support: usize,
    validation_accuracy: f64,
    mean_pair_ceiling: f64,
    eligible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct EvaluationBudget {
    carrier_slots: usize,
    fit_comparisons: usize,
    validation_comparisons: usize,
    future_projections: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
struct EvaluationMetrics {
    observations: usize,
    pairwise_predictions: usize,
    pairwise_correct: usize,
    pairwise_accuracy: f64,
    top_one_correct: usize,
    top_one_accuracy: f64,
    selected_resolver_efficiency: f64,
    top_two_oracle_capture: f64,
    probe_ceiling_efficiency: f64,
    oracle_efficiency: f64,
    probes_emitted: usize,
    mean_probe_uncertainty: f64,
}

#[derive(Debug, Clone, Serialize)]
struct WindowReport {
    index: usize,
    metrics: EvaluationMetrics,
}

#[derive(Debug, Clone, Serialize)]
struct ArmReport {
    name: String,
    eligible_carriers: usize,
    validation_pairwise_accuracy: f64,
    budget: EvaluationBudget,
    future: EvaluationMetrics,
    windows: Vec<WindowReport>,
}

#[derive(Debug, Clone, Serialize)]
struct ExistingPathReport {
    global_resolver: String,
    global_future_efficiency: f64,
    global_window_efficiencies: Vec<f64>,
    ontology_status: String,
    ontology_promoted_concepts: usize,
    ontology_future_efficiency: f64,
    exhaustive_oracle_efficiency: f64,
}

#[derive(Debug, Clone, Serialize)]
struct MatchedBudgetReport {
    exact: bool,
    expected: EvaluationBudget,
    compared_arms: usize,
}

#[derive(Debug, Clone, Serialize)]
struct GateReport {
    eligible_carriers: bool,
    validation_pairwise_accuracy: bool,
    future_pairwise_accuracy: bool,
    selected_efficiency_over_global: bool,
    baseline_window_wins: bool,
    top_two_oracle_capture: bool,
    structure_control_accuracy_margin: bool,
    structure_control_efficiency_ratio: bool,
    exact_matched_budgets: bool,
    deterministic_replay: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.eligible_carriers
            && self.validation_pairwise_accuracy
            && self.future_pairwise_accuracy
            && self.selected_efficiency_over_global
            && self.baseline_window_wins
            && self.top_two_oracle_capture
            && self.structure_control_accuracy_margin
            && self.structure_control_efficiency_ratio
            && self.exact_matched_budgets
            && self.deterministic_replay
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    primitive: &'static str,
    seed: u64,
    integration: &'static str,
    observation_source: &'static str,
    visible_charge_kind: &'static str,
    train_windows: usize,
    validation_windows: usize,
    future_windows: usize,
    total_real_observations: usize,
    total_judged_attempts: usize,
    feature_width: usize,
    input_invariants: InputInvariantReport,
    frozen_thresholds: FrozenThresholdReport,
    carriers: Vec<CarrierReport>,
    real: ArmReport,
    feature_binding_controls: Vec<ArmReport>,
    outcome_binding_controls: Vec<ArmReport>,
    future_identity_ablation: ArmReport,
    existing_paths: ExistingPathReport,
    strongest_matched_control_pairwise_accuracy: f64,
    strongest_matched_control_selected_efficiency: f64,
    selected_efficiency_over_global_ratio: f64,
    selected_efficiency_over_control_ratio: f64,
    future_pairwise_control_margin: f64,
    baseline_window_wins: usize,
    matched_budgets: MatchedBudgetReport,
    deterministic_replay: bool,
    gates: GateReport,
    verdict: &'static str,
    supported_conclusion: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct FrozenThresholdReport {
    min_eligible_carriers: usize,
    min_validation_pairwise_accuracy: f64,
    min_future_pairwise_accuracy: f64,
    min_baseline_efficiency_ratio: f64,
    min_baseline_window_wins: usize,
    min_top_two_capture: f64,
    min_control_accuracy_margin: f64,
    min_control_efficiency_ratio: f64,
    control_repeats_per_arm: usize,
}

#[derive(Debug, Default)]
struct MetricAccumulator {
    observations: usize,
    pairwise_predictions: usize,
    pairwise_correct: usize,
    top_one_correct: usize,
    selected_efficiency: f64,
    top_two_captures: usize,
    probe_ceiling_efficiency: f64,
    oracle_efficiency: f64,
    probes_emitted: usize,
    probe_uncertainty: f64,
}

impl MetricAccumulator {
    fn observe(
        &mut self,
        basis: &DisagreementBasis,
        observation: &OntologyObservation,
    ) -> Result<(), Box<dyn Error>> {
        let decision = basis.decide(&observation.charge)?;
        let resolvers = basis.resolvers();
        let oracle = oracle_resolvers(observation, resolvers);
        let oracle_efficiency = oracle
            .iter()
            .map(|resolver| resolver_efficiency(observation, resolver))
            .fold(0.0, f64::max);

        self.observations += 1;
        self.oracle_efficiency += oracle_efficiency;
        self.selected_efficiency +=
            resolver_efficiency(observation, &decision.predicted_resolver);
        if oracle.contains(&decision.predicted_resolver) {
            self.top_one_correct += 1;
        }

        for projection in &decision.projections {
            let carrier = basis
                .carriers()
                .iter()
                .find(|carrier| carrier.id == projection.carrier_id)
                .ok_or("decision referenced an unknown carrier")?;
            let utility_a = resolver_efficiency(observation, &carrier.resolver_a);
            let utility_b = resolver_efficiency(observation, &carrier.resolver_b);
            if (utility_a - utility_b).abs() <= basis.config().tie_epsilon {
                continue;
            }
            self.pairwise_predictions += 1;
            let actual = if utility_a > utility_b {
                &carrier.resolver_a
            } else {
                &carrier.resolver_b
            };
            if projection.predicted_resolver == *actual {
                self.pairwise_correct += 1;
            }
        }

        if let Some(probe) = &decision.probe {
            self.probes_emitted += 1;
            self.probe_uncertainty += probe.uncertainty;
            let first = resolver_efficiency(observation, &probe.first_resolver);
            let second = resolver_efficiency(observation, &probe.second_resolver);
            self.probe_ceiling_efficiency += first.max(second);
            if oracle.contains(&probe.first_resolver) || oracle.contains(&probe.second_resolver) {
                self.top_two_captures += 1;
            }
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) {
        self.observations += other.observations;
        self.pairwise_predictions += other.pairwise_predictions;
        self.pairwise_correct += other.pairwise_correct;
        self.top_one_correct += other.top_one_correct;
        self.selected_efficiency += other.selected_efficiency;
        self.top_two_captures += other.top_two_captures;
        self.probe_ceiling_efficiency += other.probe_ceiling_efficiency;
        self.oracle_efficiency += other.oracle_efficiency;
        self.probes_emitted += other.probes_emitted;
        self.probe_uncertainty += other.probe_uncertainty;
    }

    fn finish(&self) -> EvaluationMetrics {
        let observations = self.observations.max(1) as f64;
        EvaluationMetrics {
            observations: self.observations,
            pairwise_predictions: self.pairwise_predictions,
            pairwise_correct: self.pairwise_correct,
            pairwise_accuracy: self.pairwise_correct as f64
                / self.pairwise_predictions.max(1) as f64,
            top_one_correct: self.top_one_correct,
            top_one_accuracy: self.top_one_correct as f64 / observations,
            selected_resolver_efficiency: self.selected_efficiency / observations,
            top_two_oracle_capture: self.top_two_captures as f64 / observations,
            probe_ceiling_efficiency: self.probe_ceiling_efficiency / observations,
            oracle_efficiency: self.oracle_efficiency / observations,
            probes_emitted: self.probes_emitted,
            mean_probe_uncertainty: self.probe_uncertainty
                / self.probes_emitted.max(1) as f64,
        }
    }
}

fn run_diagnostic(windows: Vec<Vec<OntologyObservation>>) -> Result<(), Box<dyn Error>> {
    let input_invariants = input_invariants(&windows);
    if !input_invariants.passed() {
        return Err("canonical H5 fixture violated the frozen input contract".into());
    }

    let train = flatten(&windows[..TRAIN_WINDOWS]);
    let validation = flatten(&windows[TRAIN_WINDOWS..TRAIN_WINDOWS + VALIDATION_WINDOWS]);
    let future = &windows[TRAIN_WINDOWS + VALIDATION_WINDOWS..];
    let config = DisagreementBasisConfig::default();
    let basis = DisagreementBasis::fit(&train, &validation, config)?;
    let real = evaluate_basis("real_disagreement_basis", &basis, future)?;

    let replay_basis = DisagreementBasis::fit(&train, &validation, config)?;
    let deterministic_replay = basis == replay_basis
        && future.iter().flatten().all(|observation| {
            basis.decide(&observation.charge).ok()
                == replay_basis.decide(&observation.charge).ok()
        });

    let mut feature_binding_controls = Vec::with_capacity(CONTROL_REPEATS);
    let mut outcome_binding_controls = Vec::with_capacity(CONTROL_REPEATS);
    for repeat in 0..CONTROL_REPEATS {
        let control_seed = SEED ^ ((repeat as u64 + 1) * 0x9e37_79b9);
        let feature_train = permute_features(&train, control_seed ^ 0x4645_4154);
        let feature_validation =
            permute_features(&validation, control_seed ^ 0x5641_4c49);
        let feature_basis =
            DisagreementBasis::fit(&feature_train, &feature_validation, config)?;
        feature_binding_controls.push(evaluate_basis(
            format!("feature_binding_control_{repeat}"),
            &feature_basis,
            future,
        )?);

        let outcome_train = permute_outcomes(&train, control_seed ^ 0x4f55_5443);
        let outcome_validation =
            permute_outcomes(&validation, control_seed ^ 0x4f55_5456);
        let outcome_basis =
            DisagreementBasis::fit(&outcome_train, &outcome_validation, config)?;
        outcome_binding_controls.push(evaluate_basis(
            format!("outcome_binding_control_{repeat}"),
            &outcome_basis,
            future,
        )?);
    }

    let ablated_future: Vec<Vec<OntologyObservation>> = future
        .iter()
        .enumerate()
        .map(|(index, window)| {
            permute_features(window, SEED ^ 0x4142_4c41 ^ index as u64)
        })
        .collect();
    let future_identity_ablation = evaluate_basis(
        "future_identity_ablation",
        &basis,
        &ablated_future,
    )?;

    let controls: Vec<&ArmReport> = feature_binding_controls
        .iter()
        .chain(&outcome_binding_controls)
        .collect();
    let strongest_matched_control_pairwise_accuracy = controls
        .iter()
        .map(|control| control.future.pairwise_accuracy)
        .fold(0.0, f64::max);
    let strongest_matched_control_selected_efficiency = controls
        .iter()
        .map(|control| control.future.selected_resolver_efficiency)
        .fold(0.0, f64::max);

    let existing_paths = existing_paths(&train, &validation, future)?;
    let selected_efficiency_over_global_ratio = ratio(
        real.future.selected_resolver_efficiency,
        existing_paths.global_future_efficiency,
    );
    let selected_efficiency_over_control_ratio = ratio(
        real.future.selected_resolver_efficiency,
        strongest_matched_control_selected_efficiency,
    );
    let future_pairwise_control_margin = real.future.pairwise_accuracy
        - strongest_matched_control_pairwise_accuracy;
    let baseline_window_wins = real
        .windows
        .iter()
        .zip(&existing_paths.global_window_efficiencies)
        .filter(|(window, baseline)| {
            window.metrics.selected_resolver_efficiency > **baseline + SCORE_EPSILON
        })
        .count();

    let expected_budget = real.budget;
    let budget_exact = controls
        .iter()
        .all(|control| control.budget == expected_budget)
        && future_identity_ablation.budget == expected_budget;
    let matched_budgets = MatchedBudgetReport {
        exact: budget_exact,
        expected: expected_budget,
        compared_arms: controls.len() + 1,
    };

    let gates = GateReport {
        eligible_carriers: real.eligible_carriers >= MIN_ELIGIBLE_CARRIERS,
        validation_pairwise_accuracy: real.validation_pairwise_accuracy + SCORE_EPSILON
            >= MIN_VALIDATION_PAIRWISE_ACCURACY,
        future_pairwise_accuracy: real.future.pairwise_accuracy + SCORE_EPSILON
            >= MIN_FUTURE_PAIRWISE_ACCURACY,
        selected_efficiency_over_global: selected_efficiency_over_global_ratio + SCORE_EPSILON
            >= MIN_BASELINE_EFFICIENCY_RATIO,
        baseline_window_wins: baseline_window_wins >= MIN_BASELINE_WINDOW_WINS,
        top_two_oracle_capture: real.future.top_two_oracle_capture + SCORE_EPSILON
            >= MIN_TOP_TWO_CAPTURE,
        structure_control_accuracy_margin: future_pairwise_control_margin + SCORE_EPSILON
            >= MIN_CONTROL_ACCURACY_MARGIN,
        structure_control_efficiency_ratio: selected_efficiency_over_control_ratio
            + SCORE_EPSILON
            >= MIN_CONTROL_EFFICIENCY_RATIO,
        exact_matched_budgets: budget_exact,
        deterministic_replay,
    };
    let verdict = verdict(&gates);

    let carriers = basis
        .carriers()
        .iter()
        .map(|carrier| CarrierReport {
            id: carrier.id,
            resolver_a: carrier.resolver_a.clone(),
            resolver_b: carrier.resolver_b.clone(),
            positive_support: carrier.positive_support,
            negative_support: carrier.negative_support,
            validation_support: carrier.validation_support,
            validation_accuracy: carrier.validation_accuracy,
            mean_pair_ceiling: carrier.mean_pair_ceiling,
            eligible: carrier.eligible,
        })
        .collect();

    let report = Report {
        experiment: "disagreement-induced basis real-component falsification",
        primitive: "ordinal resolver disagreement -> opposing residual centroid subtraction -> validated pairwise axes -> resolver tournament / discrimination probe",
        seed: SEED,
        integration: "diagnostic-only; no Runtime mutation, live routing, or concept promotion",
        observation_source: "real Starfire emitters -> real component outputs -> task-profiled Environment -> OutcomeWitness -> RelativeImprovementJudge -> CognitiveCycleState",
        visible_charge_kind: "Custom(unresolved)",
        train_windows: TRAIN_WINDOWS,
        validation_windows: VALIDATION_WINDOWS,
        future_windows: FUTURE_WINDOWS,
        total_real_observations: windows.iter().map(Vec::len).sum(),
        total_judged_attempts: windows.iter().map(Vec::len).sum::<usize>()
            * EXPECTED_ATTEMPTS_PER_OBSERVATION,
        feature_width: basis.feature_width(),
        input_invariants,
        frozen_thresholds: FrozenThresholdReport {
            min_eligible_carriers: MIN_ELIGIBLE_CARRIERS,
            min_validation_pairwise_accuracy: MIN_VALIDATION_PAIRWISE_ACCURACY,
            min_future_pairwise_accuracy: MIN_FUTURE_PAIRWISE_ACCURACY,
            min_baseline_efficiency_ratio: MIN_BASELINE_EFFICIENCY_RATIO,
            min_baseline_window_wins: MIN_BASELINE_WINDOW_WINS,
            min_top_two_capture: MIN_TOP_TWO_CAPTURE,
            min_control_accuracy_margin: MIN_CONTROL_ACCURACY_MARGIN,
            min_control_efficiency_ratio: MIN_CONTROL_EFFICIENCY_RATIO,
            control_repeats_per_arm: CONTROL_REPEATS,
        },
        carriers,
        real,
        feature_binding_controls,
        outcome_binding_controls,
        future_identity_ablation,
        existing_paths,
        strongest_matched_control_pairwise_accuracy,
        strongest_matched_control_selected_efficiency,
        selected_efficiency_over_global_ratio,
        selected_efficiency_over_control_ratio,
        future_pairwise_control_margin,
        baseline_window_wins,
        matched_budgets,
        deterministic_replay,
        gates,
        verdict,
        supported_conclusion: "A positive causal verdict supports only that independently judged resolver disagreement synthesized a bounded residual-to-preference operator whose held-out behavior depends on preserved residual/outcome binding. It does not establish AGI, consciousness, open-world ontology growth, or safe live promotion.",
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("Disagreement-induced basis verdict: {verdict}");
    if !report.matched_budgets.exact || !report.deterministic_replay {
        return Err("diagnostic invariant failure after report emission".into());
    }
    Ok(())
}

fn input_invariants(windows: &[Vec<OntologyObservation>]) -> InputInvariantReport {
    let flattened: Vec<&OntologyObservation> = windows.iter().flatten().collect();
    let feature_width = flattened
        .first()
        .map(|observation| observation.charge.residual.len())
        .unwrap_or(0);
    InputInvariantReport {
        seven_windows: windows.len() == TRAIN_WINDOWS + VALIDATION_WINDOWS + FUTURE_WINDOWS,
        thirty_six_observations_per_window: windows.iter().all(|window| window.len() == 36),
        expected_total_observations: flattened.len() == EXPECTED_OBSERVATIONS,
        complete_five_resolver_matrix: flattened.iter().all(|observation| {
            observation.outcomes.len() == EXPECTED_ATTEMPTS_PER_OBSERVATION
                && observation
                    .outcomes
                    .iter()
                    .all(|outcome| outcome.compute_cost > 0 && outcome.discharged >= 0.0)
        }),
        all_visible_kinds_unresolved: flattened.iter().all(|observation| {
            matches!(
                &observation.charge.kind,
                ChargeKind::Custom(kind) if kind == "unresolved"
            )
        }),
        fixed_feature_width: feature_width > 0
            && flattened
                .iter()
                .all(|observation| observation.charge.residual.len() == feature_width),
    }
}

fn evaluate_basis(
    name: impl Into<String>,
    basis: &DisagreementBasis,
    future: &[Vec<OntologyObservation>],
) -> Result<ArmReport, Box<dyn Error>> {
    let mut aggregate = MetricAccumulator::default();
    let mut windows = Vec::with_capacity(future.len());
    for (index, window) in future.iter().enumerate() {
        let mut current = MetricAccumulator::default();
        for observation in window {
            current.observe(basis, observation)?;
        }
        aggregate.merge(&current);
        windows.push(WindowReport {
            index,
            metrics: current.finish(),
        });
    }
    let future_observations = future.iter().map(Vec::len).sum::<usize>();
    let DisagreementBudget {
        carrier_slots,
        fit_comparisons,
        validation_comparisons,
    } = basis.budget();
    Ok(ArmReport {
        name: name.into(),
        eligible_carriers: basis.eligible_carriers().count(),
        validation_pairwise_accuracy: basis.validation_pairwise_accuracy(),
        budget: EvaluationBudget {
            carrier_slots,
            fit_comparisons,
            validation_comparisons,
            future_projections: carrier_slots * future_observations,
        },
        future: aggregate.finish(),
        windows,
    })
}

fn existing_paths(
    train: &[OntologyObservation],
    validation: &[OntologyObservation],
    future: &[Vec<OntologyObservation>],
) -> Result<ExistingPathReport, Box<dyn Error>> {
    let resolvers = resolver_names(train);
    let global_resolver = resolvers
        .iter()
        .max_by(|left, right| {
            mean_efficiency(train, left)
                .partial_cmp(&mean_efficiency(train, right))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .cloned()
        .ok_or("training matrix contains no resolvers")?;
    let global_window_efficiencies = future
        .iter()
        .map(|window| mean_efficiency(window, &global_resolver))
        .collect::<Vec<_>>();
    let global_future_efficiency = weighted_window_mean(future, &global_window_efficiencies);
    let exhaustive_oracle_efficiency = future
        .iter()
        .flatten()
        .map(|observation| {
            resolvers
                .iter()
                .map(|resolver| resolver_efficiency(observation, resolver))
                .fold(0.0, f64::max)
        })
        .sum::<f64>()
        / future.iter().map(Vec::len).sum::<usize>().max(1) as f64;

    let (ontology_status, ontology_promoted_concepts, ontology_future_efficiency) =
        match EmpiricalOntologyInducer::new(EmpiricalInductionConfig::default())
            .fit(train, validation)
        {
            Ok(ontology) => {
                let efficiency = future
                    .iter()
                    .flatten()
                    .map(|observation| {
                        let decision = ontology.route(&observation.charge);
                        resolver_efficiency(observation, &decision.resolver)
                    })
                    .sum::<f64>()
                    / future.iter().map(Vec::len).sum::<usize>().max(1) as f64;
                (
                    "fit".to_string(),
                    ontology.summary().promoted_concepts,
                    efficiency,
                )
            }
            Err(error) => (format!("rejected: {error}"), 0, 0.0),
        };

    Ok(ExistingPathReport {
        global_resolver,
        global_future_efficiency,
        global_window_efficiencies,
        ontology_status,
        ontology_promoted_concepts,
        ontology_future_efficiency,
        exhaustive_oracle_efficiency,
    })
}

fn permute_features(observations: &[OntologyObservation], seed: u64) -> Vec<OntologyObservation> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut residuals = observations
        .iter()
        .map(|observation| observation.charge.residual.clone())
        .collect::<Vec<_>>();
    residuals.shuffle(&mut rng);
    observations
        .iter()
        .cloned()
        .zip(residuals)
        .map(|(mut observation, residual)| {
            observation.charge.residual = residual;
            observation
        })
        .collect()
}

fn permute_outcomes(observations: &[OntologyObservation], seed: u64) -> Vec<OntologyObservation> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut source_indices: Vec<usize> = (0..observations.len()).collect();
    source_indices.shuffle(&mut rng);
    observations
        .iter()
        .enumerate()
        .map(|(destination_index, destination)| {
            let source = &observations[source_indices[destination_index]];
            let mut permuted = destination.clone();
            permuted.outcomes = source
                .outcomes
                .iter()
                .map(|outcome| {
                    let mut copied = outcome.clone();
                    let fraction = (outcome.discharged / source.charge.magnitude)
                        .clamp(0.0, 1.0);
                    copied.discharged =
                        (fraction * destination.charge.magnitude).min(destination.charge.magnitude);
                    copied
                })
                .collect();
            permuted
        })
        .collect()
}

fn resolver_names(observations: &[OntologyObservation]) -> Vec<String> {
    observations
        .iter()
        .flat_map(|observation| observation.outcomes.iter())
        .map(|outcome| outcome.resolver.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn resolver_efficiency(observation: &OntologyObservation, resolver: &str) -> f64 {
    let matching = observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return 0.0;
    }
    matching
        .iter()
        .map(|outcome| {
            (outcome.discharged as f64 / observation.charge.magnitude as f64)
                / outcome.compute_cost as f64
        })
        .sum::<f64>()
        / matching.len() as f64
}

fn oracle_resolvers(observation: &OntologyObservation, resolvers: &[String]) -> Vec<String> {
    let best = resolvers
        .iter()
        .map(|resolver| resolver_efficiency(observation, resolver))
        .fold(f64::NEG_INFINITY, f64::max);
    resolvers
        .iter()
        .filter(|resolver| {
            (resolver_efficiency(observation, resolver) - best).abs() <= SCORE_EPSILON
        })
        .cloned()
        .collect()
}

fn mean_efficiency(observations: &[OntologyObservation], resolver: &str) -> f64 {
    observations
        .iter()
        .map(|observation| resolver_efficiency(observation, resolver))
        .sum::<f64>()
        / observations.len().max(1) as f64
}

fn weighted_window_mean(windows: &[Vec<OntologyObservation>], values: &[f64]) -> f64 {
    let observations = windows.iter().map(Vec::len).sum::<usize>();
    windows
        .iter()
        .zip(values)
        .map(|(window, value)| window.len() as f64 * value)
        .sum::<f64>()
        / observations.max(1) as f64
}

fn flatten(windows: &[Vec<OntologyObservation>]) -> Vec<OntologyObservation> {
    windows
        .iter()
        .flat_map(|window| window.iter().cloned())
        .collect()
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= SCORE_EPSILON {
        if numerator.abs() <= SCORE_EPSILON {
            1.0
        } else {
            numerator / SCORE_EPSILON
        }
    } else {
        numerator / denominator
    }
}

fn verdict(gates: &GateReport) -> &'static str {
    if gates.all_pass() {
        return "CAUSAL REASONING EFFECT DETECTED";
    }
    let mechanism = gates.eligible_carriers && gates.validation_pairwise_accuracy;
    if !mechanism {
        return "REJECTED";
    }
    let transferred = gates.future_pairwise_accuracy
        && gates.selected_efficiency_over_global
        && gates.baseline_window_wins
        && gates.top_two_oracle_capture;
    if !transferred {
        "MECHANISM DETECTED, NOT TRANSFERRED"
    } else {
        "TRANSFER DETECTED, CAUSALITY UNCLEAR"
    }
}
