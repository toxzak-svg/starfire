
fn semantic_case(
    x: &Fx,
    p: &FP,
    category_prohibited: &[String],
    id: u64,
) -> Result<(SemanticResponseProgram, LexicalBindingTable)> {
    let adjusted = (p.confidence * (1.0 - p.uncertainty * 0.25)).clamp(0.0001, 1.0);
    let confidence = (adjusted * 10_000.0).round() as u16;
    let status = if confidence >= 9_000 {
        EpistemicStatus::Certain
    } else if confidence >= 7_000 {
        EpistemicStatus::Probable
    } else if confidence >= 3_000 {
        EpistemicStatus::Possible
    } else {
        EpistemicStatus::Uncertain
    };
    let anchors = if x.required.is_empty() {
        vec![format!("response boundary for {}", x.id)]
    } else {
        x.required.clone()
    };
    let personal = x.category == "emotional" || x.category == "continuity";
    let claims = anchors
        .iter()
        .enumerate()
        .map(|(i, _)| AuthorizedClaim {
            id: ClaimId(u64::try_from(i + 1).unwrap_or(1)),
            semantic_key: format!("f1_{}_claim_{:03}", x.id.replace('-', "_"), i + 1),
            polarity: ClaimPolarity::Positive,
            confidence_bps: confidence.max(1),
            epistemic_status: status,
            sensitivity: if personal {
                SensitivityLevel::Personal
            } else {
                SensitivityLevel::Public
            },
            disclosure_scope: SUBJECT,
        })
        .collect::<Vec<_>>();
    let mut prohibited = category_prohibited.to_vec();
    prohibited.extend(x.prohibited.iter().cloned());
    prohibited.sort();
    prohibited.dedup();
    let prohibited_claims = prohibited
        .iter()
        .enumerate()
        .map(|(i, _)| ProhibitedClaim {
            id: ClaimId(10_000 + u64::try_from(i + 1).unwrap_or(1)),
            semantic_key: format!("f1_{}_prohibited_{:03}", x.id.replace('-', "_"), i + 1),
        })
        .collect::<Vec<_>>();

    let mut ops = Vec::new();
    let ack = matches!(x.category.as_str(), "emotional" | "continuity" | "disagreement");
    if ack {
        push(&mut ops, DiscourseOperationKind::Acknowledge(ObservationId(id)));
    }
    for c in &claims {
        push(&mut ops, DiscourseOperationKind::Assert(c.id));
        if x.category == "uncertainty" {
            push(
                &mut ops,
                DiscourseOperationKind::Qualify {
                    claim: c.id,
                    status,
                },
            );
        }
    }
    if x.category == "technical" && claims.len() > 1 {
        push(
            &mut ops,
            DiscourseOperationKind::Explain {
                claims: claims.iter().map(|c| c.id).collect(),
            },
        );
    }
    if x.category == "disagreement" && claims.len() > 1 {
        push(
            &mut ops,
            DiscourseOperationKind::Contrast {
                left: claims[0].id,
                right: claims[1].id,
            },
        );
    }
    let variable = x.category == "uncertainty";
    if variable {
        push(
            &mut ops,
            DiscourseOperationKind::RequestEvidence(MissingVariableId(id)),
        );
    }
    let prediction = x.category == "continuity";
    if prediction {
        push(&mut ops, DiscourseOperationKind::Commit(PredictionId(id)));
    }
    if x.category == "adversarial" {
        push(
            &mut ops,
            DiscourseOperationKind::Abstain(AbstentionReason::SensitiveContext),
        );
    }
    let constraints = claims
        .iter()
        .map(|c| {
            let (lo, hi) = c.epistemic_status.confidence_bounds();
            EpistemicConstraint {
                claim: c.id,
                required_status: c.epistemic_status,
                minimum_confidence_bps: lo,
                maximum_confidence_bps: hi,
            }
        })
        .collect();
    let payload = SemanticResponseProgramPayload {
        id: ResponseProgramId(id),
        source_state_version: CognitiveStateVersion(id),
        companion_state_version: None,
        subject_scope: SUBJECT,
        intent: match x.category.as_str() {
            "technical" => SemanticResponseIntent::Explanation,
            "disagreement" => SemanticResponseIntent::Contrast,
            "uncertainty" => SemanticResponseIntent::EvidenceRequest,
            "continuity" => SemanticResponseIntent::Commitment,
            "adversarial" => SemanticResponseIntent::Abstention,
            "emotional" => SemanticResponseIntent::RelationalAcknowledgment,
            _ => SemanticResponseIntent::FactualAnswer,
        },
        operations: ops,
        required_claims: claims.clone(),
        optional_claims: Vec::new(),
        prohibited_claims,
        epistemic_constraints: constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: if personal {
                SensitivityLevel::Personal
            } else {
                SensitivityLevel::Public
            },
            disclosure_scope: SUBJECT,
        },
        style: StyleEnvelope {
            detail: if x.category == "technical" {
                DetailLevel::Detailed
            } else {
                DetailLevel::Standard
            },
            vocabulary: if x.category == "technical" {
                VocabularyLevel::Technical
            } else {
                VocabularyLevel::Standard
            },
            dialogue: if variable {
                DialogueMode::QuestionLed
            } else if personal {
                DialogueMode::Collaborative
            } else {
                DialogueMode::Declarative
            },
            acknowledgment: if ack {
                AcknowledgmentLevel::Explicit
            } else {
                AcknowledgmentLevel::Brief
            },
            allow_first_person: true,
            allow_questions: variable,
            maximum_paragraphs: 8,
        },
        output_budget: OutputBudget {
            maximum_characters: 4_096,
            maximum_sentences: 64,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 64,
            maximum_claims: 128,
            maximum_verification_steps: 512,
        },
    };
    let program = SemanticResponseProgram::validate(
        payload,
        SemanticValidationContext {
            cognitive_state_version: CognitiveStateVersion(id),
            companion_state_version: None,
            subject_scope: SUBJECT,
        },
    )?;
    let lexical = LexicalBindingTable::validate(
        LexicalBindingTablePayload {
            program_digest: program.digest,
            subject_scope: SUBJECT,
            claims: claims
                .iter()
                .zip(&anchors)
                .map(|(c, a)| ClaimLexicalBinding {
                    claim: c.id,
                    positive_clause: a.clone(),
                    negative_clause: format!("the inverse does not hold for {a}"),
                })
                .collect(),
            observations: if ack {
                vec![ObservationLexicalBinding {
                    observation: ObservationId(id),
                    label: format!("context{}{}", x.category, suffix(&x.id)?),
                }]
            } else {
                Vec::new()
            },
            missing_variables: if variable {
                vec![MissingVariableLexicalBinding {
                    variable: MissingVariableId(id),
                    label: format!("unresolved{}{}", x.category, suffix(&x.id)?),
                }]
            } else {
                Vec::new()
            },
            predictions: if prediction {
                vec![PredictionLexicalBinding {
                    prediction: PredictionId(id),
                    label: format!("continuity{}{}", x.category, suffix(&x.id)?),
                }]
            } else {
                Vec::new()
            },
            forbidden_surface_forms: prohibited,
        },
        &program,
    )?;
    Ok((program, lexical))
}
fn push(v: &mut Vec<DiscourseOperation>, kind: DiscourseOperationKind) {
    v.push(DiscourseOperation {
        id: OperationId(u64::try_from(v.len() + 1).unwrap_or(1)),
        kind,
    });
}
fn side(p: Pref) -> PreferredSide {
    if p == Pref::Direct {
        PreferredSide::Left
    } else {
        PreferredSide::Right
    }
}
fn score(m: &LearnedExpressionModel, p: &LearnedVoiceProjection, v: VariantProfile) -> i64 {
    let a = p.as_array();
    let b = v.as_array();
    m.payload
        .weights
        .iter()
        .enumerate()
        .map(|(i, w)| {
            i64::from(*w) * i64::from(10_000u16.saturating_sub(a[i].abs_diff(b[i]))) / 10_000
        })
        .sum()
}
fn predict(m: &LearnedExpressionModel, p: &LearnedVoiceProjection) -> Pref {
    if score(m, p, VariantProfile::direct()) >= score(m, p, VariantProfile::warm()) {
        Pref::Direct
    } else {
        Pref::Warm
    }
}
fn accuracy<'a, I>(
    m: &LearnedExpressionModel,
    xs: I,
    override_projection: Option<&BTreeMap<String, LearnedVoiceProjection>>,
) -> f64
where
    I: Iterator<Item = &'a Case>,
{
    let mut ok = 0usize;
    let mut n = 0usize;
    for x in xs {
        let p = override_projection
            .and_then(|m| m.get(&x.fx.id))
            .unwrap_or(&x.projection);
        ok += usize::from(predict(m, p) == x.pref);
        n += 1;
    }
    ratio(ok, n)
}
fn shuffled_accuracy(m: &LearnedExpressionModel, xs: &[Case]) -> f64 {
    let mut test = xs.iter().filter(|x| x.split == Split::Test).collect::<Vec<_>>();
    test.sort_by_key(|x| if x.pref == Pref::Direct { 0 } else { 1 });
    let mut projections = test.iter().map(|x| x.projection.clone()).collect::<Vec<_>>();
    projections.sort_by_key(|p| if predict(m, p) == Pref::Warm { 0 } else { 1 });
    let map = test
        .iter()
        .zip(projections)
        .map(|(x, p)| (x.fx.id.clone(), p))
        .collect();
    accuracy(m, test.into_iter(), Some(&map))
}
