fn build_roots() -> Result<Vec<RootTask>, Box<dyn Error>> {
    let mut roots = Vec::with_capacity(FAMILIES.len() * ROOTS_PER_FAMILY);
    for (family_index, family) in FAMILIES.iter().enumerate() {
        for root_index in 0..ROOTS_PER_FAMILY {
            let root_id = 20_000_u64
                .saturating_add((family_index as u64).saturating_mul(100))
                .saturating_add(root_index as u64);
            let raw_atoms = [
                Atom::new(format!("{family}_a"))?,
                Atom::new(format!("{family}_b"))?,
                Atom::new(format!("{family}_c"))?,
                Atom::new(format!("{family}_d"))?,
                Atom::new(format!("{family}_e"))?,
                Atom::new(format!("{family}_f"))?,
            ];
            let stage1_intervention = Atom::new(format!("{family}_{root_index}_stage1_probe"))?;
            let wrong_intervention = Atom::new(format!("{family}_{root_index}_wrong_probe"))?;
            let stage2_intervention = Atom::new(format!("{family}_{root_index}_stage2_probe"))?;
            let stage1_zero = Atom::new(format!("{family}_{root_index}_p0"))?;
            let stage1_one = Atom::new(format!("{family}_{root_index}_p1"))?;
            let wrong_zero = Atom::new(format!("{family}_{root_index}_r0"))?;
            let wrong_one = Atom::new(format!("{family}_{root_index}_r1"))?;
            let stage2_zero = Atom::new(format!("{family}_{root_index}_z0"))?;
            let stage2_one = Atom::new(format!("{family}_{root_index}_z1"))?;

            let mut correct_discovery = Vec::with_capacity(DISCOVERY_HISTORIES);
            let mut wrong_discovery = Vec::with_capacity(DISCOVERY_HISTORIES);
            let mut stage2_discovery = Vec::with_capacity(DISCOVERY_HISTORIES);
            let mut transfer = Vec::with_capacity(TRANSFER_HISTORIES);
            let mut index = 0_u64;
            for p in [false, true] {
                for q in [false, true] {
                    for r in [false, true] {
                        let discovery_history = cube_history(
                            root_id.saturating_mul(1_000).saturating_add(index),
                            &raw_atoms,
                            [p, q, r],
                            [0, 1, 2],
                        );
                        correct_discovery.push(WitnessedHistory {
                            evidence_id: root_id
                                .saturating_mul(10_000)
                                .saturating_add(1_000)
                                .saturating_add(index),
                            history: discovery_history.clone(),
                            intervention: stage1_intervention.clone(),
                            outcome: if p {
                                stage1_one.clone()
                            } else {
                                stage1_zero.clone()
                            },
                        });
                        wrong_discovery.push(WitnessedHistory {
                            evidence_id: root_id
                                .saturating_mul(10_000)
                                .saturating_add(2_000)
                                .saturating_add(index),
                            history: discovery_history.clone(),
                            intervention: wrong_intervention.clone(),
                            outcome: if r {
                                wrong_one.clone()
                            } else {
                                wrong_zero.clone()
                            },
                        });
                        stage2_discovery.push(WitnessedHistory {
                            evidence_id: root_id
                                .saturating_mul(10_000)
                                .saturating_add(3_000)
                                .saturating_add(index),
                            history: discovery_history,
                            intervention: stage2_intervention.clone(),
                            outcome: if p == q {
                                stage2_one.clone()
                            } else {
                                stage2_zero.clone()
                            },
                        });

                        let transfer_history = cube_history(
                            root_id
                                .saturating_mul(1_000)
                                .saturating_add(500)
                                .saturating_add(index),
                            &raw_atoms,
                            [p, q, r],
                            [2, 0, 1],
                        );
                        transfer.push((
                            transfer_history,
                            if p == q {
                                stage2_one.clone()
                            } else {
                                stage2_zero.clone()
                            },
                        ));
                        index = index.saturating_add(1);
                    }
                }
            }

            let stage2_problem = RefinementProblem {
                root_id,
                discovery: stage2_discovery,
            };
            let mut shuffled_stage2_problem = stage2_problem.clone();
            let mut shuffled_outcomes = shuffled_stage2_problem
                .discovery
                .iter()
                .map(|episode| episode.outcome.clone())
                .collect::<Vec<_>>();
            shuffled_outcomes.rotate_left(1);
            for (episode, outcome) in shuffled_stage2_problem
                .discovery
                .iter_mut()
                .zip(shuffled_outcomes)
            {
                episode.outcome = outcome;
            }

            roots.push(RootTask {
                root_id,
                family,
                correct_ancestor_problem: RefinementProblem {
                    root_id,
                    discovery: correct_discovery,
                },
                wrong_ancestor_problem: RefinementProblem {
                    root_id,
                    discovery: wrong_discovery,
                },
                stage2_problem,
                shuffled_stage2_problem,
                transfer,
            });
        }
    }
    Ok(roots)
}

fn cube_history(
    history_id: u64,
    atoms: &[Atom; RAW_ATOMS],
    bits: [bool; 3],
    block_order: [usize; 3],
) -> RawHistory {
    let pairs = [
        (atoms[0].clone(), atoms[1].clone(), bits[0]),
        (atoms[2].clone(), atoms[3].clone(), bits[1]),
        (atoms[4].clone(), atoms[5].clone(), bits[2]),
    ];
    let mut events = Vec::with_capacity(RAW_ATOMS);
    for block in block_order {
        let (left, right, bit) = &pairs[block];
        if *bit {
            events.push(left.clone());
            events.push(right.clone());
        } else {
            events.push(right.clone());
            events.push(left.clone());
        }
    }
    RawHistory { history_id, events }
}

fn predict_transfer(
    root: &RootTask,
    language: &DescendantStateLanguage,
    budget: &mut BudgetLedger,
) -> Result<usize, Box<dyn Error>> {
    budget.downstream_key_index_passes =
        budget.downstream_key_index_passes.saturating_add(1);
    let mut index = BTreeMap::<_, BTreeSet<Atom>>::new();
    for episode in &root.stage2_problem.discovery {
        let key = language.state_key(&episode.history, &episode.intervention)?;
        index.entry(key).or_default().insert(episode.outcome.clone());
    }

    let intervention = root
        .stage2_problem
        .discovery
        .first()
        .ok_or("stage-2 discovery unexpectedly empty")?
        .intervention
        .clone();
    let mut correct = 0_usize;
    for (history, expected) in &root.transfer {
        budget.transfer_predictions = budget.transfer_predictions.saturating_add(1);
        let key = language.state_key(history, &intervention)?;
        let prediction = index.get(&key).and_then(|outcomes| {
            if outcomes.len() == 1 {
                outcomes.iter().next().cloned()
            } else {
                None
            }
        });
        budget.objective_checks = budget.objective_checks.saturating_add(1);
        if prediction.as_ref() == Some(expected) {
            correct = correct.saturating_add(1);
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
        .unwrap_or_else(|| panic!("missing Ω2 path metrics for {}", path.name()))
}
