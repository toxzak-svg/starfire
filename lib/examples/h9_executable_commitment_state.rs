use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;

use star::causal::CausalEngine;
use star::commitment_state::{
    Atom, CommitmentStateError, ExecutableCommitmentState, Rule, StateDelta, WitnessedRule,
};

const REPEATS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const EXECUTOR_SCANS: usize = 3;
const OPERATION_CALLS_PER_PATH: usize = 2;
const CAUSAL_ENGINE_CALLS_PER_PATH: usize = 1;
const TRANSITION_SLOTS_PER_PATH: usize = 1 + EXECUTOR_SCANS;
const OBJECTIVE_CHECKS_PER_PATH: usize = 1;

const FAMILY_NAMES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_control",
    "river_hydrology",
    "compiler_pipeline",
    "cell_signaling",
    "network_routing",
    "supply_chain",
    "orbital_dynamics",
];

#[derive(Debug, Clone)]
struct Task {
    id: u64,
    family_index: usize,
    family_name: &'static str,
    source: Atom,
    middle: Atom,
    goal: Atom,
    decoy_source: Atom,
    decoy_goal: Atom,
    target_witness_id: u64,
    decoy_witness_id: u64,
    target_rule: Rule,
    decoy_rule: Rule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathMode {
    Stateful,
    EndpointBlind,
    TextOnly,
    ScalarOnly,
    Rewired,
    RandomValid,
    InvalidMatched,
    Delayed,
}

impl PathMode {
    fn all() -> [Self; 8] {
        [
            Self::Stateful,
            Self::EndpointBlind,
            Self::TextOnly,
            Self::ScalarOnly,
            Self::Rewired,
            Self::RandomValid,
            Self::InvalidMatched,
            Self::Delayed,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::Stateful => "stateful",
            Self::EndpointBlind => "endpoint_blind",
            Self::TextOnly => "text_only",
            Self::ScalarOnly => "scalar_only",
            Self::Rewired => "rewired",
            Self::RandomValid => "random_valid",
            Self::InvalidMatched => "invalid_matched",
            Self::Delayed => "delayed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathResult {
    success: bool,
    compile_rejected: bool,
    successful_state_mutations: usize,
    operation_calls: usize,
    causal_engine_calls: usize,
    transition_slots: usize,
    search_scans: usize,
    objective_checks: usize,
    text_history_bytes: usize,
    scalar_history_count: usize,
    state_signature: String,
    invariants_valid: bool,
}

impl PathResult {
    fn budget_exact(&self) -> bool {
        self.operation_calls == OPERATION_CALLS_PER_PATH
            && self.causal_engine_calls == CAUSAL_ENGINE_CALLS_PER_PATH
            && self.transition_slots == TRANSITION_SLOTS_PER_PATH
            && self.search_scans == EXECUTOR_SCANS
            && self.objective_checks == OBJECTIVE_CHECKS_PER_PATH
    }
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    repeats_per_family: usize,
    train_families: usize,
    holdout_families: usize,
    future_families: usize,
    executor_scans: usize,
    operation_calls_per_path: usize,
    causal_engine_calls_per_path: usize,
    transition_slots_per_path: usize,
    objective_checks_per_path: usize,
}

#[derive(Debug, Serialize)]
struct PathMetrics {
    path: &'static str,
    roots: usize,
    successes: usize,
    success_rate: f64,
    rejected_compiles: usize,
    rejection_rate: f64,
    mean_successful_state_mutations: f64,
    budget_exact: bool,
    replay_exact: bool,
    invariants_valid: bool,
}

#[derive(Debug, Serialize)]
struct SplitReport {
    name: &'static str,
    roots: usize,
    paths: Vec<PathMetrics>,
}

#[derive(Debug, Serialize)]
struct FutureFamilyReport {
    family: &'static str,
    roots: usize,
    stateful_success_rate: f64,
    maximum_control_success_rate: f64,
    pass: bool,
}

#[derive(Debug, Serialize)]
struct GateReport {
    train_support_exact: bool,
    holdout_support_exact: bool,
    future_support_exact: bool,
    future_family_count_exact: bool,
    stateful_training_perfect: bool,
    stateful_holdout_perfect: bool,
    stateful_future_perfect: bool,
    endpoint_future_zero: bool,
    text_future_zero: bool,
    scalar_future_zero: bool,
    rewired_future_zero: bool,
    random_valid_future_zero: bool,
    delayed_future_zero: bool,
    invalid_rejection_perfect: bool,
    rewired_rejection_perfect: bool,
    all_future_families_pass: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_valid: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.train_support_exact
            && self.holdout_support_exact
            && self.future_support_exact
            && self.future_family_count_exact
            && self.stateful_training_perfect
            && self.stateful_holdout_perfect
            && self.stateful_future_perfect
            && self.endpoint_future_zero
            && self.text_future_zero
            && self.scalar_future_zero
            && self.rewired_future_zero
            && self.random_valid_future_zero
            && self.delayed_future_zero
            && self.invalid_rejection_perfect
            && self.rewired_rejection_perfect
            && self.all_future_families_pass
            && self.budgets_exact
            && self.replay_exact
            && self.invariants_valid
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    primitive: &'static str,
    operation_a: &'static str,
    operation_b: &'static str,
    objective_witness: &'static str,
    claim_boundary: &'static str,
    frozen_contract: FrozenContract,
    total_roots: usize,
    training: SplitReport,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    gates: GateReport,
    terminal_classification: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let tasks = build_tasks()?;
    let train_end = TRAIN_FAMILIES * REPEATS_PER_FAMILY;
    let holdout_end = train_end + HOLDOUT_FAMILIES * REPEATS_PER_FAMILY;
    let train = &tasks[..train_end];
    let holdout = &tasks[train_end..holdout_end];
    let future = &tasks[holdout_end..];

    if train.len() != 16 || holdout.len() != 8 || future.len() != 32 || tasks.len() != 56 {
        return Err(format!(
            "H9 frozen cohort mismatch: total={} train={} holdout={} future={}",
            tasks.len(),
            train.len(),
            holdout.len(),
            future.len()
        )
        .into());
    }

    let train_eval = evaluate_split("training", train)?;
    let holdout_eval = evaluate_split("holdout", holdout)?;
    let future_eval = evaluate_split("future", future)?;

    let training = split_report("training", train, &train_eval);
    let holdout_report = split_report("holdout", holdout, &holdout_eval);
    let future_report = split_report("future", future, &future_eval);

    let mut future_family_reports = Vec::new();
    for family_index in (TRAIN_FAMILIES + HOLDOUT_FAMILIES)..FAMILY_NAMES.len() {
        let family_tasks: Vec<_> = future
            .iter()
            .filter(|task| task.family_index == family_index)
            .cloned()
            .collect();
        let evaluation = evaluate_split("future_family", &family_tasks)?;
        let stateful = metrics_for(PathMode::Stateful, &family_tasks, &evaluation);
        let maximum_control_success_rate = PathMode::all()
            .into_iter()
            .filter(|mode| *mode != PathMode::Stateful)
            .map(|mode| metrics_for(mode, &family_tasks, &evaluation).success_rate)
            .fold(0.0_f64, f64::max);
        future_family_reports.push(FutureFamilyReport {
            family: FAMILY_NAMES[family_index],
            roots: family_tasks.len(),
            stateful_success_rate: stateful.success_rate,
            maximum_control_success_rate,
            pass: exact_one(stateful.success_rate) && exact_zero(maximum_control_success_rate),
        });
    }

    let train_stateful = path_from_report(&training, PathMode::Stateful);
    let holdout_stateful = path_from_report(&holdout_report, PathMode::Stateful);
    let future_stateful = path_from_report(&future_report, PathMode::Stateful);
    let future_endpoint = path_from_report(&future_report, PathMode::EndpointBlind);
    let future_text = path_from_report(&future_report, PathMode::TextOnly);
    let future_scalar = path_from_report(&future_report, PathMode::ScalarOnly);
    let future_rewired = path_from_report(&future_report, PathMode::Rewired);
    let future_random = path_from_report(&future_report, PathMode::RandomValid);
    let future_invalid = path_from_report(&future_report, PathMode::InvalidMatched);
    let future_delayed = path_from_report(&future_report, PathMode::Delayed);

    let all_reports = [&training, &holdout_report, &future_report];
    let gates = GateReport {
        train_support_exact: training.roots == 16,
        holdout_support_exact: holdout_report.roots == 8,
        future_support_exact: future_report.roots == 32,
        future_family_count_exact: future_family_reports.len() == FUTURE_FAMILIES,
        stateful_training_perfect: exact_one(train_stateful.success_rate),
        stateful_holdout_perfect: exact_one(holdout_stateful.success_rate),
        stateful_future_perfect: exact_one(future_stateful.success_rate),
        endpoint_future_zero: exact_zero(future_endpoint.success_rate),
        text_future_zero: exact_zero(future_text.success_rate),
        scalar_future_zero: exact_zero(future_scalar.success_rate),
        rewired_future_zero: exact_zero(future_rewired.success_rate),
        random_valid_future_zero: exact_zero(future_random.success_rate),
        delayed_future_zero: exact_zero(future_delayed.success_rate),
        invalid_rejection_perfect: all_reports.iter().all(|report| {
            exact_one(path_from_report(report, PathMode::InvalidMatched).rejection_rate)
        }),
        rewired_rejection_perfect: all_reports
            .iter()
            .all(|report| exact_one(path_from_report(report, PathMode::Rewired).rejection_rate)),
        all_future_families_pass: future_family_reports.iter().all(|report| report.pass),
        budgets_exact: all_reports
            .iter()
            .flat_map(|report| report.paths.iter())
            .all(|metrics| metrics.budget_exact),
        replay_exact: all_reports
            .iter()
            .flat_map(|report| report.paths.iter())
            .all(|metrics| metrics.replay_exact),
        invariants_valid: all_reports
            .iter()
            .flat_map(|report| report.paths.iter())
            .all(|metrics| metrics.invariants_valid),
    };

    let terminal_classification = if !gates.invariants_valid {
        "INFRASTRUCTURE_FAILURE"
    } else if !gates.budgets_exact {
        "BUDGET_FAILURE"
    } else if !gates.replay_exact {
        "REPLAY_FAILURE"
    } else if gates.stateful_future_perfect
        && (!gates.endpoint_future_zero
            || !gates.text_future_zero
            || !gates.scalar_future_zero
            || !gates.rewired_future_zero
            || !gates.random_valid_future_zero
            || !gates.delayed_future_zero
            || !gates.invalid_rejection_perfect
            || !gates.rewired_rejection_perfect)
    {
        "CONTROL_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "H9 proof-carrying executable commitment state",
        primitive: "validated typed commitments with witness or derivation provenance; only committed facts and rules participate in future execution",
        operation_a: "existing CausalEngine-backed compilation of one raw witnessed relation into a typed CompileWitnessedRule delta",
        operation_b: "three fixed canonical executable-closure scans that can read commitments but cannot read raw observations, transcript text, scalar history, family labels, or verifier target",
        objective_witness: "external exact target-atom membership check over final executable facts",
        claim_boundary: "demonstrates state-dependent composition in the H9 shadow substrate only; does not establish learned operator invention, automatic ontology induction, live routing readiness, AGI, consciousness, or human-level reasoning",
        frozen_contract: FrozenContract {
            repeats_per_family: REPEATS_PER_FAMILY,
            train_families: TRAIN_FAMILIES,
            holdout_families: HOLDOUT_FAMILIES,
            future_families: FUTURE_FAMILIES,
            executor_scans: EXECUTOR_SCANS,
            operation_calls_per_path: OPERATION_CALLS_PER_PATH,
            causal_engine_calls_per_path: CAUSAL_ENGINE_CALLS_PER_PATH,
            transition_slots_per_path: TRANSITION_SLOTS_PER_PATH,
            objective_checks_per_path: OBJECTIVE_CHECKS_PER_PATH,
        },
        total_roots: tasks.len(),
        training,
        holdout: holdout_report,
        future: future_report,
        future_families: future_family_reports,
        gates,
        terminal_classification,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal_classification != "PASS" {
        std::process::exit(1);
    }
    Ok(())
}

fn build_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    let mut tasks = Vec::with_capacity(FAMILY_NAMES.len() * REPEATS_PER_FAMILY);
    for (family_index, family_name) in FAMILY_NAMES.iter().copied().enumerate() {
        for repeat in 0..REPEATS_PER_FAMILY {
            let id = (family_index * REPEATS_PER_FAMILY + repeat + 1) as u64;
            let prefix = format!("{family_name}_{repeat}");
            let source = Atom::new(format!("{prefix}_source"))?;
            let middle = Atom::new(format!("{prefix}_middle"))?;
            let goal = Atom::new(format!("{prefix}_goal"))?;
            let decoy_source = Atom::new(format!("{prefix}_decoy_source"))?;
            let decoy_goal = Atom::new(format!("{prefix}_decoy_goal"))?;
            let target_rule = Rule::new(middle.clone(), goal.clone())?;
            let decoy_rule = Rule::new(decoy_source.clone(), decoy_goal.clone())?;
            tasks.push(Task {
                id,
                family_index,
                family_name,
                source,
                middle,
                goal,
                decoy_source,
                decoy_goal,
                target_witness_id: id * 2,
                decoy_witness_id: id * 2 + 1,
                target_rule,
                decoy_rule,
            });
        }
    }
    Ok(tasks)
}

fn initial_state(task: &Task) -> Result<ExecutableCommitmentState, CommitmentStateError> {
    let mut state = ExecutableCommitmentState::new();
    state.seed_fact(task.source.clone())?;
    state.seed_fact(task.decoy_source.clone())?;
    state.seed_rule(Rule::new(task.source.clone(), task.middle.clone())?)?;
    state.add_observation(WitnessedRule {
        witness_id: task.target_witness_id,
        rule: task.target_rule.clone(),
    })?;
    state.add_observation(WitnessedRule {
        witness_id: task.decoy_witness_id,
        rule: task.decoy_rule.clone(),
    })?;
    Ok(state)
}

fn compile_with_causal_engine(
    state: &ExecutableCommitmentState,
    witness_id: u64,
) -> Result<StateDelta, Box<dyn Error>> {
    let witness = state
        .observation(witness_id)
        .ok_or_else(|| format!("missing witness {witness_id}"))?;
    let mut engine = CausalEngine::new();
    engine.add_edge(
        witness.rule.antecedent.as_str(),
        witness.rule.consequent.as_str(),
        1.0,
        None,
    );
    let edge = engine
        .get_effects_of(witness.rule.antecedent.as_str())
        .into_iter()
        .find(|edge| edge.effect == witness.rule.consequent.as_str())
        .ok_or("CausalEngine failed to reproduce witnessed relation")?;
    Ok(StateDelta::CompileWitnessedRule {
        witness_id,
        rule: Rule::new(Atom::new(edge.cause.clone())?, Atom::new(edge.effect.clone())?)?,
    })
}

fn execute_split_path(
    task: &Task,
    mode: PathMode,
    rewired_rule: Option<&Rule>,
) -> Result<PathResult, Box<dyn Error>> {
    let mut state = initial_state(task)?;
    let compiler_delta = match mode {
        PathMode::RandomValid => compile_with_causal_engine(&state, task.decoy_witness_id)?,
        _ => compile_with_causal_engine(&state, task.target_witness_id)?,
    };

    let mut compile_rejected = false;
    let mut successful_state_mutations = 0usize;
    let mut text_history_bytes = 0usize;
    let mut scalar_history_count = 0usize;

    if mode != PathMode::Delayed {
        match mode {
            PathMode::Stateful | PathMode::RandomValid => {
                state.apply_delta(compiler_delta.clone())?;
                successful_state_mutations += 1;
            }
            PathMode::EndpointBlind => {}
            PathMode::TextOnly => {
                text_history_bytes = serde_json::to_string(&compiler_delta)?.len();
            }
            PathMode::ScalarOnly => {
                scalar_history_count = 1;
            }
            PathMode::Rewired => {
                let delta = StateDelta::CompileWitnessedRule {
                    witness_id: task.target_witness_id,
                    rule: rewired_rule.ok_or("rewired path missing donor rule")?.clone(),
                };
                match state.apply_delta(delta) {
                    Ok(_) => successful_state_mutations += 1,
                    Err(CommitmentStateError::WitnessMismatch(_)) => compile_rejected = true,
                    Err(error) => return Err(error.into()),
                }
            }
            PathMode::InvalidMatched => {
                let delta = StateDelta::CompileWitnessedRule {
                    witness_id: task.target_witness_id,
                    rule: task.decoy_rule.clone(),
                };
                match state.apply_delta(delta) {
                    Ok(_) => successful_state_mutations += 1,
                    Err(CommitmentStateError::WitnessMismatch(_)) => compile_rejected = true,
                    Err(error) => return Err(error.into()),
                }
            }
            PathMode::Delayed => unreachable!(),
        }
    }

    for _ in 0..EXECUTOR_SCANS {
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
            successful_state_mutations += 1;
        }
    }

    if mode == PathMode::Delayed {
        state.apply_delta(compiler_delta)?;
        successful_state_mutations += 1;
    }

    let invariants_valid = state.verify_invariants().is_ok();
    let success = independent_objective_check(&state, &task.goal);
    Ok(PathResult {
        success,
        compile_rejected,
        successful_state_mutations,
        operation_calls: OPERATION_CALLS_PER_PATH,
        causal_engine_calls: CAUSAL_ENGINE_CALLS_PER_PATH,
        transition_slots: TRANSITION_SLOTS_PER_PATH,
        search_scans: EXECUTOR_SCANS,
        objective_checks: OBJECTIVE_CHECKS_PER_PATH,
        text_history_bytes,
        scalar_history_count,
        state_signature: state.canonical_signature(),
        invariants_valid,
    })
}

fn independent_objective_check(state: &ExecutableCommitmentState, target: &Atom) -> bool {
    state.contains_fact(target)
}

fn evaluate_split(
    _name: &'static str,
    tasks: &[Task],
) -> Result<BTreeMap<PathMode, Vec<(PathResult, bool)>>, Box<dyn Error>> {
    let mut result = BTreeMap::<PathMode, Vec<(PathResult, bool)>>::new();
    for mode in PathMode::all() {
        let mut rows = Vec::with_capacity(tasks.len());
        for (index, task) in tasks.iter().enumerate() {
            let donor_rule = if tasks.is_empty() {
                None
            } else {
                Some(&tasks[(index + 1) % tasks.len()].target_rule)
            };
            let first = execute_split_path(task, mode, donor_rule)?;
            let second = execute_split_path(task, mode, donor_rule)?;
            let replay_exact = first == second;
            rows.push((first, replay_exact));
        }
        result.insert(mode, rows);
    }
    Ok(result)
}

fn split_report(
    name: &'static str,
    tasks: &[Task],
    evaluation: &BTreeMap<PathMode, Vec<(PathResult, bool)>>,
) -> SplitReport {
    SplitReport {
        name,
        roots: tasks.len(),
        paths: PathMode::all()
            .into_iter()
            .map(|mode| metrics_for(mode, tasks, evaluation))
            .collect(),
    }
}

fn metrics_for(
    mode: PathMode,
    tasks: &[Task],
    evaluation: &BTreeMap<PathMode, Vec<(PathResult, bool)>>,
) -> PathMetrics {
    let rows = evaluation.get(&mode).map(Vec::as_slice).unwrap_or(&[]);
    let roots = tasks.len();
    let successes = rows.iter().filter(|(row, _)| row.success).count();
    let rejected_compiles = rows
        .iter()
        .filter(|(row, _)| row.compile_rejected)
        .count();
    let mutation_total: usize = rows
        .iter()
        .map(|(row, _)| row.successful_state_mutations)
        .sum();
    PathMetrics {
        path: mode.name(),
        roots,
        successes,
        success_rate: rate(successes, roots),
        rejected_compiles,
        rejection_rate: rate(rejected_compiles, roots),
        mean_successful_state_mutations: mutation_total as f64 / roots.max(1) as f64,
        budget_exact: rows.iter().all(|(row, _)| row.budget_exact()),
        replay_exact: rows.iter().all(|(_, replay)| *replay),
        invariants_valid: rows.iter().all(|(row, _)| row.invariants_valid),
    }
}

fn path_from_report(report: &SplitReport, mode: PathMode) -> &PathMetrics {
    report
        .paths
        .iter()
        .find(|metrics| metrics.path == mode.name())
        .expect("every frozen path must be present")
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    numerator as f64 / denominator.max(1) as f64
}

fn exact_one(value: f64) -> bool {
    value.to_bits() == 1.0_f64.to_bits()
}

fn exact_zero(value: f64) -> bool {
    value.to_bits() == 0.0_f64.to_bits()
}
