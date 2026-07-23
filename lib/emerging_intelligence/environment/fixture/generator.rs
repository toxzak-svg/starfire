use super::*;

pub fn generate_frozen_fixture(
    manifest: &FrozenEnvironmentManifest,
    partition: EvaluationPartition,
    seed: u64,
) -> Result<SealedTaskFixture, EnvironmentError> {
    manifest.validate()?;
    if !manifest.contains(partition, seed) {
        return Err(EnvironmentError::CrossPartitionFixture { partition, seed });
    }
    let base_seed = canonical_structure_seed(partition, seed)?;
    let family = if base_seed % 2 == 1 {
        TaskFamily::RouteChoice
    } else {
        TaskFamily::AttributeRule
    };
    let mut rng = DeterministicRng::new(base_seed ^ 0x6e62_9a71_c4d3_8f05);
    let mut task = match family {
        TaskFamily::RouteChoice => TaskPayload::RouteChoice(generate_route_task(
            partition,
            base_seed,
            &mut rng,
        )),
        TaskFamily::AttributeRule => TaskPayload::AttributeRule(generate_rule_task(
            partition,
            base_seed,
            &mut rng,
        )),
    };
    apply_partition_surface(partition, &mut task, base_seed)?;
    let evidence_cues = generate_evidence_cues(partition, &task)?;
    let structure_fingerprint = task.structure_fingerprint()?;
    let relation_fingerprint = task.relation_fingerprint()?;
    let surface_fingerprint = task.surface_fingerprint()?;
    let optimal_action = task.optimal_action()?;
    let fixture = TaskFixture {
        fixture_id: format!(
            "ei-0b-{}-{}-{}",
            partition_name(partition),
            family.as_str(),
            seed
        ),
        partition,
        seed,
        family,
        task,
        evidence_cues,
        optimal_action,
        action_budget: manifest.action_budget,
        evidence_budget: manifest.evidence_budget,
        manifest_digest: manifest.digest()?,
        structure_fingerprint,
        relation_fingerprint,
        surface_fingerprint,
        authority: AuthoritySnapshot::closed(),
    };
    fixture.seal()
}

fn canonical_structure_seed(
    partition: EvaluationPartition,
    seed: u64,
) -> Result<u64, EnvironmentError> {
    let offset = match partition {
        EvaluationPartition::Development => 0,
        EvaluationPartition::WithinFamilyHoldout => 0,
        EvaluationPartition::RenamedVocabularyTransfer => 200,
        EvaluationPartition::StructuralTransfer => 300,
        EvaluationPartition::Regression => 400,
        EvaluationPartition::Adversarial => 500,
    };
    seed.checked_sub(offset)
        .ok_or(EnvironmentError::InvalidSeedMapping { partition, seed })
}

fn generate_route_task(
    partition: EvaluationPartition,
    base_seed: u64,
    rng: &mut DeterministicRng,
) -> RouteTask {
    let start = token(partition, "node", base_seed, 0);
    let goal = token(partition, "node", base_seed, 9);
    let first_cost = 2 + rng.range_u16(0, 4);
    let second_cost = first_cost + 2 + rng.range_u16(0, 3);
    let mut options = vec![
        RouteOption {
            action: token(partition, "route", base_seed, 1),
            path: vec![start.clone(), goal.clone()],
            total_cost: first_cost,
        },
        RouteOption {
            action: token(partition, "route", base_seed, 2),
            path: vec![start.clone(), goal.clone()],
            total_cost: second_cost,
        },
    ];
    options.sort_by(|left, right| left.action.cmp(&right.action));
    RouteTask {
        start,
        goal,
        options,
    }
}

fn generate_rule_task(
    partition: EvaluationPartition,
    base_seed: u64,
    rng: &mut DeterministicRng,
) -> AttributeRuleTask {
    let required_a = token(partition, "attribute", base_seed, 1);
    let required_b = token(partition, "attribute", base_seed, 2);
    let distractor = token(
        partition,
        "attribute",
        base_seed,
        3 + u64::from(rng.range_u16(0, 2)),
    );
    let mut required_attributes = vec![required_a.clone(), required_b.clone()];
    required_attributes.sort();

    let mut examples = vec![
        RuleExample {
            object: token(partition, "example", base_seed, 1),
            attributes: sorted_strings(vec![required_a.clone(), required_b.clone()]),
            matches: true,
        },
        RuleExample {
            object: token(partition, "example", base_seed, 2),
            attributes: sorted_strings(vec![required_a.clone(), distractor.clone()]),
            matches: false,
        },
        RuleExample {
            object: token(partition, "example", base_seed, 3),
            attributes: sorted_strings(vec![required_b.clone(), distractor.clone()]),
            matches: false,
        },
    ];
    examples.sort_by(|left, right| left.object.cmp(&right.object));

    let mut candidates = vec![
        RuleCandidate {
            action: token(partition, "choose", base_seed, 1),
            object: token(partition, "candidate", base_seed, 1),
            attributes: sorted_strings(vec![required_a.clone(), required_b.clone()]),
        },
        RuleCandidate {
            action: token(partition, "choose", base_seed, 2),
            object: token(partition, "candidate", base_seed, 2),
            attributes: sorted_strings(vec![required_a, distractor.clone()]),
        },
        RuleCandidate {
            action: token(partition, "choose", base_seed, 3),
            object: token(partition, "candidate", base_seed, 3),
            attributes: sorted_strings(vec![required_b, distractor]),
        },
    ];
    candidates.sort_by(|left, right| left.action.cmp(&right.action));
    AttributeRuleTask {
        required_attributes,
        examples,
        candidates,
    }
}

fn apply_partition_surface(
    partition: EvaluationPartition,
    task: &mut TaskPayload,
    base_seed: u64,
) -> Result<(), EnvironmentError> {
    if partition == EvaluationPartition::StructuralTransfer {
        match task {
            TaskPayload::RouteChoice(route) => {
                for (index, option) in route.options.iter_mut().enumerate() {
                    let middle = token(partition, "junction", base_seed, index as u64 + 1);
                    option.path = vec![route.start.clone(), middle, route.goal.clone()];
                }
            }
            TaskPayload::AttributeRule(rule) => {
                let irrelevant = token(partition, "attribute", base_seed, 8);
                for example in &mut rule.examples {
                    example.attributes.push(irrelevant.clone());
                    example.attributes.sort();
                }
                for candidate in &mut rule.candidates {
                    candidate.attributes.push(irrelevant.clone());
                    candidate.attributes.sort();
                }
                let required = rule.required_attributes[0].clone();
                let mut distractor_attributes = vec![required, irrelevant];
                distractor_attributes.sort();
                rule.candidates.push(RuleCandidate {
                    action: token(partition, "choose", base_seed, 9),
                    object: token(partition, "candidate", base_seed, 9),
                    attributes: distractor_attributes,
                });
                rule.candidates
                    .sort_by(|left, right| left.action.cmp(&right.action));
            }
        }
    }
    task.validate()
}

fn generate_evidence_cues(
    partition: EvaluationPartition,
    task: &TaskPayload,
) -> Result<Vec<EvidenceCue>, EnvironmentError> {
    let optimal = task.optimal_action()?;
    let mut cues = vec![EvidenceCue {
        cue: format!("verified:{optimal}"),
        reliability_bps: 9_000,
    }];
    if partition == EvaluationPartition::Adversarial {
        let misleading = task
            .legal_actions()
            .into_iter()
            .find(|action| action != &optimal)
            .ok_or(EnvironmentError::InvalidTask("adversarial decoy"))?;
        cues.push(EvidenceCue {
            cue: format!("unverified:{misleading}"),
            reliability_bps: 1_000,
        });
    }
    cues.sort_by(|left, right| left.cue.cmp(&right.cue));
    Ok(cues)
}

fn token(
    partition: EvaluationPartition,
    kind: &str,
    base_seed: u64,
    index: u64,
) -> String {
    let vocabulary = if partition == EvaluationPartition::RenamedVocabularyTransfer {
        "renamed"
    } else {
        "base"
    };
    format!("{vocabulary}-{kind}-{base_seed}-{index}")
}
