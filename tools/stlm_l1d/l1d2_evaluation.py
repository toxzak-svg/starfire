from l1d2_core import *
from l1d2_rankers import *


def bounded_residual_preflight() -> dict[str, Any]:
    cases = [
        {"rule_a": 1000, "learned_a": 0, "rule_b": 499, "learned_b": 10_000, "expected": "a"},
        {"rule_a": 1000, "learned_a": 0, "rule_b": 500, "learned_b": 10_000, "expected": "a"},
        {"rule_a": 1000, "learned_a": 0, "rule_b": 501, "learned_b": 10_000, "expected": "b"},
        {"rule_a": 1000, "learned_a": 5000, "rule_b": 1000, "learned_b": 10_000, "expected": "b"},
    ]
    observations = []
    for case in cases:
        combined_a = case["rule_a"] + learned_residual(case["learned_a"])
        combined_b = case["rule_b"] + learned_residual(case["learned_b"])
        observed = "a" if (combined_a, case["rule_a"]) >= (combined_b, case["rule_b"]) else "b"
        observations.append({**case, "combined_a": combined_a, "combined_b": combined_b, "observed": observed})
    return {
        "per_candidate_residual_limit_bps": LEARNED_RESIDUAL_LIMIT_BPS,
        "maximum_pairwise_swing_bps": PAIRWISE_LEARNED_SWING_LIMIT_BPS,
        "cases": observations,
        "passed": all(item["observed"] == item["expected"] for item in observations),
    }


def evaluate_invalid_probes(
    model: Mapping[str, Any], probes: Sequence[InvalidProbe]
) -> dict[str, Any]:
    results = []
    invalid_count = 0
    rejected_count = 0
    for probe in probes:
        eligible = [candidate for candidate in probe.candidates if hard_gate_passed(candidate)]
        invalid = [candidate for candidate in probe.candidates if not hard_gate_passed(candidate)]
        invalid_count += len(invalid)
        rejected_count += len(invalid)
        scored = []
        for candidate in eligible:
            learned = score_recurrent_critic(model, probe.context, candidate.text)
            residual = learned_residual(learned)
            scored.append(
                (
                    candidate.rule_score + residual,
                    candidate.rule_score,
                    candidate.candidate_id,
                    learned,
                    residual,
                )
            )
        scored.sort(key=lambda item: (-item[0], -item[1], item[2]))
        selected = scored[0][2]
        results.append(
            {
                "probe_id": probe.probe_id,
                "eligible_candidates": len(eligible),
                "invalid_candidates_rejected": len(invalid),
                "selected_candidate_id": selected,
                "expected_candidate_id": probe.expected_candidate_id,
                "passed": selected == probe.expected_candidate_id,
            }
        )
    return {
        "probes": len(probes),
        "invalid_candidates": invalid_count,
        "invalid_candidates_rejected": rejected_count,
        "results": results,
        "passed": bool(probes)
        and invalid_count > 0
        and rejected_count == invalid_count
        and all(item["passed"] for item in results),
    }


def candidate_distribution(tournaments: Sequence[Tournament]) -> dict[str, int]:
    counts = Counter(len(tournament.candidates) for tournament in tournaments)
    return {str(size): counts[size] for size in sorted(counts)}


def corpus_summary(tournaments: Sequence[Tournament]) -> dict[str, Any]:
    return {
        "tournaments": len(tournaments),
        "groups": len({item.group_id for item in tournaments}),
        "categories": sorted({item.category for item in tournaments}),
        "candidate_count_distribution": candidate_distribution(tournaments),
        "minimum_candidates": min(len(item.candidates) for item in tournaments),
        "maximum_candidates": max(len(item.candidates) for item in tournaments),
        "surface_candidates": sum(len(item.candidates) for item in tournaments),
        "all_surface_candidates_semantic_valid": all(
            hard_gate_passed(candidate)
            for tournament in tournaments
            for candidate in tournament.candidates
        ),
    }


def evaluate_seed(
    model: Mapping[str, Any], tournaments: Sequence[Tournament], seed: int
) -> dict[str, Any]:
    splits = stratified_group_split(tournaments, seed)
    train, dev, test = splits["train"], splits["dev"], splits["test"]
    ngram_ranker, ngram_hyperparameters = choose_linear_ranker(
        train, dev, hashed_ngram_features, HASH_DIMENSION, seed
    )
    length_ranker, length_hyperparameters = choose_linear_ranker(
        train, dev, length_features, 5, seed + 50_021
    )

    scorers: dict[str, Callable[[Tournament, Candidate], float]] = {
        "deterministic_rule": lambda tournament, candidate: float(candidate.rule_score),
        "pooled_embeddings": lambda tournament, candidate: float(
            score_pooled_embeddings(model, tournament.context, candidate.text)
        ),
        "recurrent_critic": lambda tournament, candidate: float(
            score_recurrent_critic(model, tournament.context, candidate.text)
        ),
        "bounded_residual": lambda tournament, candidate: float(
            candidate.rule_score
            + learned_residual(score_recurrent_critic(model, tournament.context, candidate.text))
        ),
        "hashed_ngram": lambda tournament, candidate: ngram_ranker.score(
            hashed_ngram_features(candidate.text, tournament.context)
        ),
        "length_only": lambda tournament, candidate: length_ranker.score(
            length_features(candidate.text, tournament.context)
        ),
    }
    metrics = {name: evaluate_ranker(test, scorer) for name, scorer in scorers.items()}

    shuffled_ngram_ranker, shuffled_ngram_hyperparameters = choose_linear_ranker(
        shuffled_labels(train, seed + 211),
        shuffled_labels(dev, seed + 223),
        hashed_ngram_features,
        HASH_DIMENSION,
        seed + 227,
    )
    shuffled_label_metrics = evaluate_ranker(
        test,
        lambda tournament, candidate: shuffled_ngram_ranker.score(
            hashed_ngram_features(candidate.text, tournament.context)
        ),
    )

    control_sets = {
        "shuffled_context": shuffled_contexts(test, seed + 101),
        "punctuation_normalized": [
            transformed_tournament(item, transform_punctuation) for item in test
        ],
        "whitespace_normalized": [
            transformed_tournament(item, transform_whitespace) for item in test
        ],
        "unicode_normalized": [
            transformed_tournament(item, transform_unicode) for item in test
        ],
    }
    controlled = {
        "shuffled_label": {
            "system": "hashed_ngram",
            "hyperparameters": shuffled_ngram_hyperparameters,
            "metrics": shuffled_label_metrics,
            "selection_stability_bps": selection_stability(
                metrics["hashed_ngram"], shuffled_label_metrics
            ),
        }
    }
    for name, controlled_tournaments in control_sets.items():
        controlled_metrics = evaluate_ranker(controlled_tournaments, scorers["recurrent_critic"])
        controlled[name] = {
            "system": "recurrent_critic",
            "metrics": controlled_metrics,
            "selection_stability_bps": selection_stability(
                metrics["recurrent_critic"], controlled_metrics
            ),
        }

    length_matched = [
        item
        for item in test
        if max(len(candidate.text.encode("utf-8")) for candidate in item.candidates)
        <= 1.25 * min(len(candidate.text.encode("utf-8")) for candidate in item.candidates)
    ]
    controlled["length_only"] = {"system": "length_only", "metrics": metrics["length_only"]}
    controlled["length_matched_subset"] = {
        "system": "recurrent_critic,bounded_residual",
        "tournaments": len(length_matched),
        "recurrent_critic_metrics": evaluate_ranker(length_matched, scorers["recurrent_critic"]),
        "bounded_residual_metrics": evaluate_ranker(length_matched, scorers["bounded_residual"]),
    }

    return {
        "seed": seed,
        "split": {
            name: {
                "tournaments": len(records),
                "groups": sorted({item.group_id for item in records}),
                "categories": sorted({item.category for item in records}),
            }
            for name, records in splits.items()
        },
        "hashed_ngram_hyperparameters": ngram_hyperparameters,
        "length_only_hyperparameters": length_hyperparameters,
        "metrics": metrics,
        "controls": controlled,
    }


def aggregate_seed_metrics(seed_reports: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    systems = tuple(seed_reports[0]["metrics"].keys())
    aggregate = {}
    for system in systems:
        top1 = [report["metrics"][system]["top1_accuracy_bps"] for report in seed_reports]
        mrr = [report["metrics"][system]["mean_reciprocal_rank_bps"] for report in seed_reports]
        pairwise = [report["metrics"][system]["pairwise_accuracy_bps"] for report in seed_reports]
        aggregate[system] = {
            "top1_accuracy_mean_bps": round(statistics.fmean(top1)),
            "top1_accuracy_min_bps": min(top1),
            "top1_accuracy_max_bps": max(top1),
            "mean_reciprocal_rank_mean_bps": round(statistics.fmean(mrr)),
            "pairwise_accuracy_mean_bps": round(statistics.fmean(pairwise)),
            "per_seed_top1_accuracy_bps": top1,
        }
    return aggregate


def control_summary(seed_reports: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    summary = {}
    for control in REQUIRED_CONTROLS:
        if control in {"length_only", "length_matched_subset"}:
            present = all(control in report["controls"] for report in seed_reports)
            systems = {report["controls"][control]["system"] for report in seed_reports}
            summary[control] = {
                "present_all_seeds": present,
                "system": next(iter(systems)) if len(systems) == 1 else "inconsistent",
            }
            continue
        accuracy = [
            report["controls"][control]["metrics"]["top1_accuracy_bps"]
            for report in seed_reports
        ]
        stability = [
            report["controls"][control]["selection_stability_bps"]
            for report in seed_reports
        ]
        systems = {report["controls"][control]["system"] for report in seed_reports}
        summary[control] = {
            "present_all_seeds": True,
            "system": next(iter(systems)) if len(systems) == 1 else "inconsistent",
            "top1_accuracy_mean_bps": round(statistics.fmean(accuracy)),
            "selection_stability_mean_bps": round(statistics.fmean(stability)),
        }
    return summary


def deterministic_core(
    model: Mapping[str, Any],
    model_digest: str,
    surface_path: Path,
    invalid_path: Path,
    tournaments: Sequence[Tournament],
    probes: Sequence[InvalidProbe],
    seeds: Sequence[int],
) -> dict[str, Any]:
    seed_reports = [evaluate_seed(model, tournaments, seed) for seed in seeds]
    residual_preflight = bounded_residual_preflight()
    invalid_probe = evaluate_invalid_probes(model, probes)
    summary = corpus_summary(tournaments)
    controls = control_summary(seed_reports)
    split_integrity = all(
        not (
            set(report["split"]["train"]["groups"])
            & set(report["split"]["dev"]["groups"])
            or set(report["split"]["train"]["groups"])
            & set(report["split"]["test"]["groups"])
            or set(report["split"]["dev"]["groups"])
            & set(report["split"]["test"]["groups"])
        )
        for report in seed_reports
    )
    structural_gate = (
        tuple(seeds) == DEFAULT_SEEDS
        and summary["tournaments"] >= 36
        and summary["groups"] >= 36
        and summary["minimum_candidates"] >= MIN_TOURNAMENT_CANDIDATES
        and summary["maximum_candidates"] <= MAX_TOURNAMENT_CANDIDATES
        and summary["all_surface_candidates_semantic_valid"]
        and summary["categories"] == sorted(CATEGORY_ORDER)
        and split_integrity
        and all(controls[name]["present_all_seeds"] for name in REQUIRED_CONTROLS)
        and residual_preflight["passed"]
        and invalid_probe["passed"]
    )
    return {
        "experiment": "STLM_L1D2_VALID_SURFACE_BENCHMARK_AND_CRITIC_V2_PREFLIGHT",
        "schema_version": SCHEMA_VERSION,
        "authority": {
            "surface_quality_only": True,
            "semantic_invalid_corpus_separate": True,
            "learned_rank_is_primary": False,
            "learned_rank_is_bounded_residual": True,
            "per_candidate_residual_limit_bps": LEARNED_RESIDUAL_LIMIT_BPS,
            "maximum_pairwise_learned_swing_bps": PAIRWISE_LEARNED_SWING_LIMIT_BPS,
            "hard_gate_override": False,
            "runtime_chat_influence": False,
            "http_response_influence": False,
            "automatic_promotion": False,
        },
        "model_sha256": model_digest,
        "surface_corpus_sha256": sha256_bytes(surface_path.read_bytes()),
        "semantic_invalid_corpus_sha256": sha256_bytes(invalid_path.read_bytes()),
        "surface_corpus": summary,
        "semantic_invalid_probe": invalid_probe,
        "bounded_residual_preflight": residual_preflight,
        "seeds": list(seeds),
        "grouped_split_integrity": split_integrity,
        "seed_reports": seed_reports,
        "aggregate_metrics": aggregate_seed_metrics(seed_reports),
        "controls": controls,
        "required_controls": list(REQUIRED_CONTROLS),
        "structural_gate_passed": structural_gate,
        "quality_can_enable_live_authority": False,
    }


def strip_nondeterministic(report: Mapping[str, Any]) -> dict[str, Any]:
    value = copy.deepcopy(dict(report))
    value.pop("elapsed_ms", None)
    value.pop("within_runtime_budget", None)
    value.pop("deterministic_report_sha256", None)
    return value


def render_markdown(report: Mapping[str, Any]) -> str:
    lines = [
        "# STLM L1-D2 Valid-Surface Benchmark and Critic V2 Preflight",
        "",
        f"- Structural gate: **{'PASS' if report['structural_gate_passed'] else 'FAIL'}**",
        f"- Surface tournaments: **{report['surface_corpus']['tournaments']}**",
        f"- Surface candidates: **{report['surface_corpus']['surface_candidates']}**",
        f"- Groups: **{report['surface_corpus']['groups']}**",
        f"- Seeds: **{', '.join(str(seed) for seed in report['seeds'])}**",
        f"- Semantic-invalid probes: **{report['semantic_invalid_probe']['probes']}**",
        f"- Exact report replay: **{report['deterministic_full_report_replay']}**",
        "",
        "## Five-seed aggregate",
        "",
        "| System | Top-1 mean | Top-1 range | MRR mean | Pairwise mean |",
        "|---|---:|---:|---:|---:|",
    ]
    for system, metrics in report["aggregate_metrics"].items():
        lines.append(
            f"| `{system}` | {metrics['top1_accuracy_mean_bps'] / 100:.2f}% | "
            f"{metrics['top1_accuracy_min_bps'] / 100:.2f}%–{metrics['top1_accuracy_max_bps'] / 100:.2f}% | "
            f"{metrics['mean_reciprocal_rank_mean_bps'] / 100:.2f}% | "
            f"{metrics['pairwise_accuracy_mean_bps'] / 100:.2f}% |"
        )
    lines.extend(
        [
            "",
            "## Controls",
            "",
            "| Control | System | Present all seeds | Top-1 | Selection stability |",
            "|---|---|---:|---:|---:|",
        ]
    )
    for name in REQUIRED_CONTROLS:
        control = report["controls"][name]
        accuracy = control.get("top1_accuracy_mean_bps")
        stability = control.get("selection_stability_mean_bps")
        accuracy_text = f"{accuracy / 100:.2f}%" if accuracy is not None else "n/a"
        stability_text = f"{stability / 100:.2f}%" if stability is not None else "n/a"
        lines.append(
            f"| `{name}` | `{control.get('system', 'n/a')}` | {control['present_all_seeds']} | "
            f"{accuracy_text} | {stability_text} |"
        )
    lines.extend(
        [
            "",
            "## Authority boundary",
            "",
            f"The learned score is capped at **±{LEARNED_RESIDUAL_LIMIT_BPS}** per candidate, "
            f"for a maximum pairwise swing of **{PAIRWISE_LEARNED_SWING_LIMIT_BPS}**. "
            "The deterministic rule score remains primary outside that band.",
            "",
            "Semantic-invalid examples are excluded from every quality metric and are used only "
            "to verify that semantic, slot-preservation, and identity-conflict gates fail closed.",
        ]
    )
    return "\n".join(lines) + "\n"
