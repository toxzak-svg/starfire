fn moving_suite() -> TransformationSuite {
    TransformationSuite {
        transformations: vec![
            BlockPermutation {
                block_width: 2,
                block_order: vec![1, 0, 2, 3],
            },
            BlockPermutation {
                block_width: 2,
                block_order: vec![1, 2, 0, 3],
            },
        ],
        correspondence: CorrespondenceMode::SameHistory,
    }
}

fn stationary_suite() -> TransformationSuite {
    TransformationSuite {
        transformations: vec![
            BlockPermutation {
                block_width: 2,
                block_order: vec![0, 2, 1, 3],
            },
            BlockPermutation {
                block_width: 2,
                block_order: vec![0, 3, 2, 1],
            },
        ],
        correspondence: CorrespondenceMode::SameHistory,
    }
}

fn rewired_suite() -> TransformationSuite {
    let mut suite = moving_suite();
    suite.correspondence = CorrespondenceMode::CyclicNext;
    suite
}

fn heldout_transformations() -> [BlockPermutation; HELDOUT_TRANSFORMATIONS] {
    [
        BlockPermutation {
            block_width: 2,
            block_order: vec![1, 2, 3, 0],
        },
        BlockPermutation {
            block_width: 2,
            block_order: vec![3, 2, 1, 0],
        },
    ]
}

fn build_roots() -> Result<Vec<RootTask>, Box<dyn Error>> {
    let mut roots = Vec::with_capacity(FAMILIES.len() * ROOTS_PER_FAMILY);
    for (family_index, family) in FAMILIES.iter().enumerate() {
        for root_index in 0..ROOTS_PER_FAMILY {
            let root_id = 30_000_u64
                .saturating_add((family_index as u64).saturating_mul(100))
                .saturating_add(root_index as u64);
            let atoms = [
                Atom::new(format!("{family}_a"))?,
                Atom::new(format!("{family}_b"))?,
                Atom::new(format!("{family}_c"))?,
                Atom::new(format!("{family}_d"))?,
                Atom::new(format!("{family}_e"))?,
                Atom::new(format!("{family}_f"))?,
                Atom::new(format!("{family}_g"))?,
                Atom::new(format!("{family}_h"))?,
            ];
            let intervention = Atom::new(format!("{family}_{root_index}_probe"))?;
            let outcome_zero = Atom::new(format!("{family}_{root_index}_p0"))?;
            let outcome_one = Atom::new(format!("{family}_{root_index}_p1"))?;

            let mut discovery = Vec::with_capacity(DISCOVERY_HISTORIES);
            let mut index = 0_u64;
            for p in [false, true] {
                for q in [false, true] {
                    for r in [false, true] {
                        for s in [false, true] {
                            discovery.push(WitnessedHistory {
                                evidence_id: root_id
                                    .saturating_mul(10_000)
                                    .saturating_add(index),
                                history: pair_block_history(
                                    root_id.saturating_mul(1_000).saturating_add(index),
                                    &atoms,
                                    [p, q, r, s],
                                ),
                                intervention: intervention.clone(),
                                outcome: if p {
                                    outcome_one.clone()
                                } else {
                                    outcome_zero.clone()
                                },
                            });
                            index = index.saturating_add(1);
                        }
                    }
                }
            }

            roots.push(RootTask {
                root_id,
                family,
                problem: RefinementProblem { root_id, discovery },
            });
        }
    }
    Ok(roots)
}

fn pair_block_history(
    history_id: u64,
    atoms: &[Atom; RAW_ATOMS],
    bits: [bool; 4],
) -> RawHistory {
    let pairs = [
        (&atoms[0], &atoms[1], bits[0]),
        (&atoms[2], &atoms[3], bits[1]),
        (&atoms[4], &atoms[5], bits[2]),
        (&atoms[6], &atoms[7], bits[3]),
    ];
    let mut events = Vec::with_capacity(RAW_ATOMS);
    for (left, right, bit) in pairs {
        if bit {
            events.push(left.clone());
            events.push(right.clone());
        } else {
            events.push(right.clone());
            events.push(left.clone());
        }
    }
    RawHistory { history_id, events }
}

fn predict_transport_language(
    root: &RootTask,
    language: &TransportStateLanguage,
    budget: &mut BudgetLedger,
) -> Result<usize, Box<dyn Error>> {
    budget.discovery_key_index_passes = budget.discovery_key_index_passes.saturating_add(1);
    let mut index = BTreeMap::<_, BTreeSet<Atom>>::new();
    for episode in &root.problem.discovery {
        let key = language.state_key(&episode.history, &episode.intervention);
        index.entry(key).or_default().insert(episode.outcome.clone());
    }

    let mut correct = 0_usize;
    for transformation in heldout_transformations() {
        for episode in &root.problem.discovery {
            budget.heldout_transformation_applications = budget
                .heldout_transformation_applications
                .saturating_add(1);
            let transformed = transformation.apply(&episode.history)?;
            let key = language.state_key(&transformed, &episode.intervention);
            let prediction = index.get(&key).and_then(|outcomes| {
                if outcomes.len() == 1 {
                    outcomes.iter().next().cloned()
                } else {
                    None
                }
            });
            budget.prediction_attempts = budget.prediction_attempts.saturating_add(1);
            budget.objective_checks = budget.objective_checks.saturating_add(1);
            if prediction.as_ref() == Some(&episode.outcome) {
                correct = correct.saturating_add(1);
            }
        }
    }
    Ok(correct)
}

fn predict_partition_language(
    root: &RootTask,
    language: &StateLanguage,
    budget: &mut BudgetLedger,
) -> Result<usize, Box<dyn Error>> {
    budget.discovery_key_index_passes = budget.discovery_key_index_passes.saturating_add(1);
    let mut index = BTreeMap::<_, BTreeSet<Atom>>::new();
    for episode in &root.problem.discovery {
        let key = language.state_key(&episode.history, &episode.intervention);
        index.entry(key).or_default().insert(episode.outcome.clone());
    }

    let mut correct = 0_usize;
    for transformation in heldout_transformations() {
        for episode in &root.problem.discovery {
            budget.heldout_transformation_applications = budget
                .heldout_transformation_applications
                .saturating_add(1);
            let transformed = transformation.apply(&episode.history)?;
            let key = language.state_key(&transformed, &episode.intervention);
            let prediction = index.get(&key).and_then(|outcomes| {
                if outcomes.len() == 1 {
                    outcomes.iter().next().cloned()
                } else {
                    None
                }
            });
            budget.prediction_attempts = budget.prediction_attempts.saturating_add(1);
            budget.objective_checks = budget.objective_checks.saturating_add(1);
            if prediction.as_ref() == Some(&episode.outcome) {
                correct = correct.saturating_add(1);
            }
        }
    }
    Ok(correct)
}

fn evaluate_split(roots: &[RootTask]) -> Result<SplitReport, Box<dyn Error>> {
    let mut executions = BTreeMap::<PathKind, Vec<(Execution, bool)>>::new();
    for root in roots {
        for path in PathKind::all() {
            let first = execute_path(root, path)?;
            let second = execute_path(root, path)?;
            let replay_exact = first == second;
            executions
                .entry(path)
                .or_default()
                .push((first, replay_exact));
        }
    }

    let mut paths = BTreeMap::new();
    for path in PathKind::all() {
        let entries = executions.remove(&path).unwrap_or_default();
        paths.insert(path.name().to_string(), metrics_from_entries(&entries));
    }
    Ok(SplitReport {
        roots: roots.len(),
        paths,
    })
}

fn path_metrics<'a>(report: &'a SplitReport, path: PathKind) -> &'a PathMetrics {
    report
        .paths
        .get(path.name())
        .unwrap_or_else(|| panic!("missing ΩR1 path metrics for {}", path.name()))
}
