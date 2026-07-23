use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;

use star::charge::{
    ChargeKind, ConceptPredicate, Direction, OntologyObservation,
};

const H4_MEMORY_DIMENSION: usize = 2;
const H4_MEMORY_THRESHOLD: f32 = 0.171875;
const MARGIN_THRESHOLD: f64 = 0.10;
const MIN_REASONING_LEAD_FRACTION: f64 = 0.20;
const MIN_CAUSAL_LEAD_FRACTION: f64 = 0.20;
const MIN_REASONING_MARGIN_FRACTION: f64 = 0.15;
const MIN_CAUSAL_MARGIN_FRACTION: f64 = 0.15;
const MIN_QUALIFYING_FUTURE_WINDOWS: usize = 3;
const MAX_SINGLE_FUTURE_LEADER_FRACTION: f64 = 0.70;
const SCORE_EPSILON: f64 = 1e-12;
const FUTURE_START_WINDOW: usize = 3;
const RESOLVERS: [&str; 5] = [
    "reasoning",
    "memory",
    "causal",
    "prediction",
    "metacognition",
];

#[derive(Debug, Clone)]
struct H5BObservation {
    observation: OntologyObservation,
    hidden_class: String,
    family: String,
}

#[allow(dead_code)]
mod frozen_h4_fixture {
    include!("h4_real_cycle_shadow_probe.rs");

    pub(super) fn judged_windows(
    ) -> Result<Vec<Vec<super::H5BObservation>>, Box<dyn Error>> {
        let state = ProbeState::new()?;
        let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
        let mut next_id = 1u64;
        let mut windows = Vec::with_capacity(FAMILIES.len());

        for family_index in 0..FAMILIES.len() {
            let family = FAMILIES[family_index];
            let family_name = format!(
                "{} | {} | {}",
                family.gap_topic, family.contradiction_topic, family.trajectory_topic
            );
            let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
            for class in EventClass::all() {
                for repeat in 0..REPEATS_PER_CLASS {
                    let task = surface_task(family, class, repeat);
                    let mut charge = emit_real_charge(&task)?;
                    charge.kind = ChargeKind::Custom("unresolved".into());
                    charge.id = next_id;
                    next_id += 1;
                    let charge = ontology_feature_charge(&charge);
                    let attempts = judged_component_attempts(&charge, &task, &state)?;
                    let observation = recorder.record(charge, &attempts)?;
                    window.push(super::H5BObservation {
                        observation,
                        hidden_class: class.name().to_string(),
                        family: family_name.clone(),
                    });
                }
            }
            windows.push(window);
        }

        Ok(windows)
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct LeaderDistribution {
    reasoning: usize,
    memory: usize,
    causal: usize,
    prediction: usize,
    metacognition: usize,
    ties: usize,
    observations: usize,
}

impl LeaderDistribution {
    fn record(&mut self, leader: Option<&str>) {
        self.observations += 1;
        match leader {
            Some("reasoning") => self.reasoning += 1,
            Some("memory") => self.memory += 1,
            Some("causal") => self.causal += 1,
            Some("prediction") => self.prediction += 1,
            Some("metacognition") => self.metacognition += 1,
            Some(other) => panic!("unexpected resolver leader: {other}"),
            None => self.ties += 1,
        }
    }

    fn fraction(&self, resolver: &str) -> f64 {
        let count = match resolver {
            "reasoning" => self.reasoning,
            "memory" => self.memory,
            "causal" => self.causal,
            "prediction" => self.prediction,
            "metacognition" => self.metacognition,
            other => panic!("unexpected resolver fraction request: {other}"),
        };
        count as f64 / self.observations.max(1) as f64
    }

    fn max_unique_fraction(&self) -> f64 {
        RESOLVERS
            .iter()
            .map(|resolver| self.fraction(resolver))
            .fold(0.0, f64::max)
    }
}

#[derive(Debug, Clone, Serialize)]
struct MarginSummary {
    observations: usize,
    mean: f64,
    median: f64,
    stddev: f64,
    q10: f64,
    q25: f64,
    q50: f64,
    q75: f64,
    q90: f64,
    reasoning_favored_fraction: f64,
    causal_favored_fraction: f64,
    ambiguous_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
struct WindowReport {
    index: usize,
    family: String,
    phase: &'static str,
    excluded_by_frozen_h4_memory_predicate: usize,
    retained: usize,
    leaders: LeaderDistribution,
    margins: MarginSummary,
    both_margin_tails_present: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GateReport {
    reasoning_leads: bool,
    causal_leads: bool,
    reasoning_margin_tail: bool,
    causal_margin_tail: bool,
    future_directionality: bool,
    no_dominant_future_resolver: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.reasoning_leads
            && self.causal_leads
            && self.reasoning_margin_tail
            && self.causal_margin_tail
            && self.future_directionality
            && self.no_dominant_future_resolver
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    observation_source: &'static str,
    ontology_fitting_performed: bool,
    visible_charge_kind: &'static str,
    exclusion_rule: String,
    total_observations: usize,
    excluded_observations: usize,
    retained_observations: usize,
    excluded_hidden_class_distribution: BTreeMap<String, usize>,
    retained_hidden_class_distribution: BTreeMap<String, usize>,
    reasoning_favored_hidden_class_distribution: BTreeMap<String, usize>,
    causal_favored_hidden_class_distribution: BTreeMap<String, usize>,
    global_leaders: LeaderDistribution,
    global_margins: MarginSummary,
    future_leaders: LeaderDistribution,
    qualifying_future_windows: usize,
    windows: Vec<WindowReport>,
    gate_thresholds: BTreeMap<&'static str, f64>,
    gates: GateReport,
    status: &'static str,
    pass: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let windows = frozen_h4_fixture::judged_windows()?;
    let memory_predicate = ConceptPredicate::ResidualThreshold {
        dimension: H4_MEMORY_DIMENSION,
        threshold: H4_MEMORY_THRESHOLD,
        direction: Direction::AtMost,
    };

    let total_observations = windows.iter().map(Vec::len).sum::<usize>();
    let mut excluded_observations = 0usize;
    let mut excluded_hidden = BTreeMap::<String, usize>::new();
    let mut retained_hidden = BTreeMap::<String, usize>::new();
    let mut reasoning_favored_hidden = BTreeMap::<String, usize>::new();
    let mut causal_favored_hidden = BTreeMap::<String, usize>::new();
    let mut global_retained = Vec::<&H5BObservation>::new();
    let mut future_retained = Vec::<&H5BObservation>::new();
    let mut window_reports = Vec::with_capacity(windows.len());

    for (index, window) in windows.iter().enumerate() {
        let phase = match index {
            0 | 1 => "training",
            2 => "promotion_holdout",
            _ => "future_transfer",
        };
        let mut retained = Vec::<&H5BObservation>::new();
        let mut excluded = 0usize;
        for item in window {
            if !matches!(
                &item.observation.charge.kind,
                ChargeKind::Custom(kind) if kind == "unresolved"
            ) {
                return Err("H5-B received a visible non-unresolved ChargeKind".into());
            }
            if memory_predicate.matches(&item.observation.charge) {
                excluded += 1;
                excluded_observations += 1;
                *excluded_hidden.entry(item.hidden_class.clone()).or_default() += 1;
            } else {
                *retained_hidden.entry(item.hidden_class.clone()).or_default() += 1;
                retained.push(item);
                global_retained.push(item);
                if index >= FUTURE_START_WINDOW {
                    future_retained.push(item);
                }
                let margin = reasoning_causal_margin(&item.observation);
                if margin >= MARGIN_THRESHOLD {
                    *reasoning_favored_hidden
                        .entry(item.hidden_class.clone())
                        .or_default() += 1;
                }
                if margin <= -MARGIN_THRESHOLD {
                    *causal_favored_hidden
                        .entry(item.hidden_class.clone())
                        .or_default() += 1;
                }
            }
        }

        let leaders = leader_distribution(&retained);
        let margins = margin_summary(&retained);
        let both_margin_tails_present = margins.reasoning_favored_fraction
            + SCORE_EPSILON
            >= MIN_REASONING_MARGIN_FRACTION
            && margins.causal_favored_fraction + SCORE_EPSILON
                >= MIN_CAUSAL_MARGIN_FRACTION;
        window_reports.push(WindowReport {
            index,
            family: window
                .first()
                .map(|item| item.family.clone())
                .unwrap_or_default(),
            phase,
            excluded_by_frozen_h4_memory_predicate: excluded,
            retained: retained.len(),
            leaders,
            margins,
            both_margin_tails_present,
        });
    }

    let global_leaders = leader_distribution(&global_retained);
    let global_margins = margin_summary(&global_retained);
    let future_leaders = leader_distribution(&future_retained);
    let qualifying_future_windows = window_reports
        .iter()
        .filter(|window| window.phase == "future_transfer" && window.both_margin_tails_present)
        .count();

    let gates = GateReport {
        reasoning_leads: global_leaders.fraction("reasoning") + SCORE_EPSILON
            >= MIN_REASONING_LEAD_FRACTION,
        causal_leads: global_leaders.fraction("causal") + SCORE_EPSILON
            >= MIN_CAUSAL_LEAD_FRACTION,
        reasoning_margin_tail: global_margins.reasoning_favored_fraction + SCORE_EPSILON
            >= MIN_REASONING_MARGIN_FRACTION,
        causal_margin_tail: global_margins.causal_favored_fraction + SCORE_EPSILON
            >= MIN_CAUSAL_MARGIN_FRACTION,
        future_directionality: qualifying_future_windows >= MIN_QUALIFYING_FUTURE_WINDOWS,
        no_dominant_future_resolver: future_leaders.max_unique_fraction() <=
            MAX_SINGLE_FUTURE_LEADER_FRACTION + SCORE_EPSILON,
    };
    let pass = gates.all_pass();

    let mut gate_thresholds = BTreeMap::new();
    gate_thresholds.insert(
        "min_reasoning_lead_fraction",
        MIN_REASONING_LEAD_FRACTION,
    );
    gate_thresholds.insert("min_causal_lead_fraction", MIN_CAUSAL_LEAD_FRACTION);
    gate_thresholds.insert(
        "min_reasoning_margin_fraction",
        MIN_REASONING_MARGIN_FRACTION,
    );
    gate_thresholds.insert(
        "min_causal_margin_fraction",
        MIN_CAUSAL_MARGIN_FRACTION,
    );
    gate_thresholds.insert(
        "min_qualifying_future_windows",
        MIN_QUALIFYING_FUTURE_WINDOWS as f64,
    );
    gate_thresholds.insert(
        "max_single_future_leader_fraction",
        MAX_SINGLE_FUTURE_LEADER_FRACTION,
    );
    gate_thresholds.insert("reasoning_causal_margin_threshold", MARGIN_THRESHOLD);

    let report = Report {
        experiment: "H5-B non-memory resolver identifiability",
        observation_source: "frozen H4 real-cycle fixture path: real emitters -> real component outputs -> Environment -> OutcomeWitness -> RelativeImprovementJudge -> CognitiveCycleState",
        ontology_fitting_performed: false,
        visible_charge_kind: "Custom(unresolved)",
        exclusion_rule: format!(
            "frozen H4 predicate residual[{H4_MEMORY_DIMENSION}] <= {H4_MEMORY_THRESHOLD}"
        ),
        total_observations,
        excluded_observations,
        retained_observations: global_retained.len(),
        excluded_hidden_class_distribution: excluded_hidden,
        retained_hidden_class_distribution: retained_hidden,
        reasoning_favored_hidden_class_distribution: reasoning_favored_hidden,
        causal_favored_hidden_class_distribution: causal_favored_hidden,
        global_leaders,
        global_margins,
        future_leaders,
        qualifying_future_windows,
        windows: window_reports,
        gate_thresholds,
        gates,
        status: if pass { "Identifiable" } else { "NotIdentifiable" },
        pass,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if !pass {
        std::process::exit(1);
    }
    Ok(())
}

fn leader_distribution(observations: &[&H5BObservation]) -> LeaderDistribution {
    let mut distribution = LeaderDistribution::default();
    for item in observations {
        distribution.record(unique_leader(&item.observation));
    }
    distribution
}

fn unique_leader(observation: &OntologyObservation) -> Option<&str> {
    let scores: Vec<(&str, f64)> = RESOLVERS
        .iter()
        .map(|resolver| (*resolver, resolver_efficiency(observation, resolver)))
        .collect();
    let best = scores
        .iter()
        .map(|(_, score)| *score)
        .fold(f64::NEG_INFINITY, f64::max);
    let leaders: Vec<&str> = scores
        .iter()
        .filter(|(_, score)| (*score - best).abs() <= SCORE_EPSILON)
        .map(|(resolver, _)| *resolver)
        .collect();
    (leaders.len() == 1).then_some(leaders[0])
}

fn reasoning_causal_margin(observation: &OntologyObservation) -> f64 {
    resolver_efficiency(observation, "reasoning")
        - resolver_efficiency(observation, "causal")
}

fn resolver_efficiency(observation: &OntologyObservation, resolver: &str) -> f64 {
    let outcomes: Vec<_> = observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
        .collect();
    if outcomes.is_empty() {
        return 0.0;
    }
    outcomes
        .iter()
        .map(|outcome| {
            (outcome.discharged as f64 / observation.charge.magnitude as f64)
                / outcome.compute_cost as f64
        })
        .sum::<f64>()
        / outcomes.len() as f64
}

fn margin_summary(observations: &[&H5BObservation]) -> MarginSummary {
    let mut margins: Vec<f64> = observations
        .iter()
        .map(|item| reasoning_causal_margin(&item.observation))
        .collect();
    margins.sort_by(|left, right| left.total_cmp(right));
    let count = margins.len();
    let mean = margins.iter().sum::<f64>() / count.max(1) as f64;
    let variance = margins
        .iter()
        .map(|margin| {
            let delta = margin - mean;
            delta * delta
        })
        .sum::<f64>()
        / count.max(1) as f64;
    let reasoning_favored = margins
        .iter()
        .filter(|margin| **margin >= MARGIN_THRESHOLD)
        .count();
    let causal_favored = margins
        .iter()
        .filter(|margin| **margin <= -MARGIN_THRESHOLD)
        .count();
    let ambiguous = margins
        .iter()
        .filter(|margin| margin.abs() < MARGIN_THRESHOLD)
        .count();

    MarginSummary {
        observations: count,
        mean,
        median: quantile(&margins, 0.50),
        stddev: variance.sqrt(),
        q10: quantile(&margins, 0.10),
        q25: quantile(&margins, 0.25),
        q50: quantile(&margins, 0.50),
        q75: quantile(&margins, 0.75),
        q90: quantile(&margins, 0.90),
        reasoning_favored_fraction: reasoning_favored as f64 / count.max(1) as f64,
        causal_favored_fraction: causal_favored as f64 / count.max(1) as f64,
        ambiguous_fraction: ambiguous as f64 / count.max(1) as f64,
    }
}

fn quantile(sorted: &[f64], probability: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let position = probability.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}
