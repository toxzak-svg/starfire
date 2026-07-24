#!/usr/bin/env python3
"""STLM L1-D3A failure attribution and recovery preregistration.

This diagnostic explains why the frozen L1-D1 phrase critic damages the
L1-D2 valid-surface benchmark. It does not train, promote, or activate a model.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import statistics
import time
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Callable, Mapping, Sequence

from l1d2_core import (
    CATEGORY_ORDER,
    DEFAULT_SEEDS,
    Candidate,
    Tournament,
    canonical_bytes,
    learned_residual,
    load_model,
    load_surface_corpus,
    parse_seeds,
    score_recurrent_critic,
    sha256_bytes,
)
from l1d2_rankers import (
    candidate_by_id,
    rank_candidates,
    shuffled_contexts,
    stratified_group_split,
    transform_punctuation,
    transform_unicode,
    transform_whitespace,
)

SCHEMA_VERSION = 1
SEMANTIC_RISK_LABELS = frozenset(
    {
        "automatic_promotion",
        "authority_expansion",
        "confirmation_bias",
        "evidence_corruption",
        "hidden_failure",
        "identity_overclaim",
        "missing_evidence",
        "premature_promotion",
        "semantic_drift",
        "unsupported_reassurance",
        "wrong_confidence",
    }
)
PROMOTION_GATES = (
    "mean_top1_gain_over_deterministic_at_least_300_bps",
    "paired_95_percent_confidence_interval_excludes_zero",
    "improves_in_at_least_four_of_five_seeds",
    "no_category_regression_exceeds_200_bps",
    "newly_damaged_deterministic_correct_rate_below_200_bps",
    "beats_hashed_ngram_baseline",
    "correct_context_beats_shuffled_context_on_context_sensitive_subset",
    "context_neutral_subset_stable_under_context_shuffle",
    "neutral_surface_controls_at_least_9000_bps_selection_stability",
    "semantic_hard_gate_rejection_remains_complete",
    "python_rust_inference_exact_parity",
    "live_chat_and_http_authority_remain_closed",
    "promotion_requires_separate_explicit_review",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--surface-corpus", type=Path, required=True)
    parser.add_argument("--bootstrap-pairs", type=Path, required=True)
    parser.add_argument("--output-json", type=Path, required=True)
    parser.add_argument("--output-md", type=Path, required=True)
    parser.add_argument(
        "--seeds",
        default=",".join(str(seed) for seed in DEFAULT_SEEDS),
        help="exactly five deterministic grouped-split seeds",
    )
    parser.add_argument("--max-runtime-seconds", type=float, default=90.0)
    return parser.parse_args()


def _mean(values: Sequence[float]) -> float:
    return statistics.fmean(values) if values else 0.0


def _bps(numerator: int, denominator: int) -> int:
    return round(numerator / denominator * 10_000) if denominator else 0


def _rankdata(values: Sequence[float]) -> list[float]:
    indexed = sorted(enumerate(values), key=lambda item: item[1])
    ranks = [0.0] * len(values)
    cursor = 0
    while cursor < len(indexed):
        end = cursor + 1
        while end < len(indexed) and indexed[end][1] == indexed[cursor][1]:
            end += 1
        average_rank = (cursor + 1 + end) / 2.0
        for index in range(cursor, end):
            ranks[indexed[index][0]] = average_rank
        cursor = end
    return ranks


def pearson(left: Sequence[float], right: Sequence[float]) -> float:
    if len(left) != len(right) or len(left) < 2:
        return 0.0
    left_mean = _mean(left)
    right_mean = _mean(right)
    numerator = sum((a - left_mean) * (b - right_mean) for a, b in zip(left, right))
    left_scale = math.sqrt(sum((value - left_mean) ** 2 for value in left))
    right_scale = math.sqrt(sum((value - right_mean) ** 2 for value in right))
    denominator = left_scale * right_scale
    return numerator / denominator if denominator else 0.0


def spearman(left: Sequence[float], right: Sequence[float]) -> float:
    return pearson(_rankdata(left), _rankdata(right))


def punctuation_count(text: str) -> int:
    return sum(unicodedata.category(character).startswith("P") for character in text)


def terminal_signature(text: str) -> str:
    stripped = text.rstrip()
    if not stripped:
        return "empty"
    character = stripped[-1]
    category = unicodedata.category(character)
    if category.startswith("P"):
        return f"punctuation:{character}"
    if character.isdigit():
        return "digit"
    if character.isalpha():
        return "letter"
    return f"unicode:{category}"


def terminal_neutralized(text: str) -> str:
    stripped = text.rstrip()
    while stripped and unicodedata.category(stripped[-1]).startswith("P"):
        stripped = stripped[:-1].rstrip()
    return stripped


def load_bootstrap_audit(path: Path) -> dict[str, Any]:
    records: list[dict[str, Any]] = []
    for line_number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not raw.strip():
            continue
        value = json.loads(raw)
        if not isinstance(value, dict):
            raise ValueError(f"{path}:{line_number}: record must be an object")
        records.append(value)
    semantic_risk = []
    surface_only = []
    labels = Counter()
    for record in records:
        record_labels = {str(label) for label in record.get("failure_labels", [])}
        labels.update(record_labels)
        target = semantic_risk if record_labels & SEMANTIC_RISK_LABELS else surface_only
        target.append(str(record.get("source_id", "")))
    return {
        "pairs": len(records),
        "semantic_or_authority_risk_pairs": len(semantic_risk),
        "surface_only_pairs": len(surface_only),
        "semantic_or_authority_risk_rate_bps": _bps(len(semantic_risk), len(records)),
        "semantic_or_authority_risk_source_ids": semantic_risk,
        "surface_only_source_ids": surface_only,
        "failure_label_counts": dict(sorted(labels.items())),
        "target_contract_matches_valid_surface_tournaments": False,
        "finding": (
            "The frozen trainer mixes surface preferences with semantic, confidence, evidence, "
            "identity, and authority distinctions; it is not a clean valid-surface tournament corpus."
        ),
    }


def candidate_scores(
    model: Mapping[str, Any], tournament: Tournament, context: Sequence[int] | None = None
) -> list[dict[str, Any]]:
    active_context = tournament.context if context is None else tuple(context)
    rows = []
    for candidate in tournament.candidates:
        critic = score_recurrent_critic(model, active_context, candidate.text)
        residual = learned_residual(critic)
        rows.append(
            {
                "candidate_id": candidate.candidate_id,
                "gold": candidate.candidate_id == tournament.gold_candidate_id,
                "text": candidate.text,
                "rule_score": candidate.rule_score,
                "critic_score_bps": critic,
                "learned_residual_bps": residual,
                "combined_score": candidate.rule_score + residual,
                "byte_length": len(candidate.text.encode("utf-8")),
                "character_length": len(candidate.text),
                "word_count": len(re.findall(r"\S+", candidate.text)),
                "punctuation_count": punctuation_count(candidate.text),
                "terminal_signature": terminal_signature(candidate.text),
            }
        )
    return rows


def selection(rows: Sequence[Mapping[str, Any]], score_field: str) -> int:
    return int(
        sorted(
            rows,
            key=lambda row: (
                -float(row[score_field]),
                -int(row["rule_score"]) if score_field == "combined_score" else 0,
                int(row["candidate_id"]),
            ),
        )[0]["candidate_id"]
    )


def transform_selection(
    model: Mapping[str, Any],
    tournament: Tournament,
    transform: Callable[[str], str],
) -> tuple[int, list[int]]:
    scores = {}
    raw = []
    for candidate in tournament.candidates:
        score = score_recurrent_critic(model, tournament.context, transform(candidate.text))
        scores[candidate.candidate_id] = score
        raw.append(score)
    return rank_candidates(scores)[0], raw


def residual_bucket(value: int) -> str:
    if value <= -250:
        return "-250_clipped"
    if value <= -101:
        return "-249_to_-101"
    if value <= -1:
        return "-100_to_-1"
    if value == 0:
        return "0"
    if value <= 100:
        return "1_to_100"
    if value <= 249:
        return "101_to_249"
    return "+250_clipped"


def analyze(
    model: Mapping[str, Any],
    tournaments: Sequence[Tournament],
    seeds: Sequence[int],
    bootstrap_audit: Mapping[str, Any],
) -> dict[str, Any]:
    traces: list[dict[str, Any]] = []
    candidate_observations: list[dict[str, Any]] = []
    category_counts: dict[str, Counter[str]] = defaultdict(Counter)
    flip_ledger: list[dict[str, Any]] = []
    correction_ledger: list[dict[str, Any]] = []
    control_counts = Counter()
    context_score_deltas: list[int] = []
    residual_counts: Counter[str] = Counter()

    for seed in seeds:
        test = stratified_group_split(tournaments, seed)["test"]
        shuffled_by_id = {
            item.tournament_id: item for item in shuffled_contexts(test, seed + 101)
        }
        for tournament in sorted(test, key=lambda item: item.tournament_id):
            rows = candidate_scores(model, tournament)
            rule_selected = selection(rows, "rule_score")
            critic_selected = selection(rows, "critic_score_bps")
            bounded_selected = selection(rows, "combined_score")
            gold = tournament.gold_candidate_id
            rule_correct = rule_selected == gold
            critic_correct = critic_selected == gold
            bounded_correct = bounded_selected == gold
            category = category_counts[tournament.category]
            category["observations"] += 1
            category["rule_correct"] += int(rule_correct)
            category["critic_correct"] += int(critic_correct)
            category["bounded_correct"] += int(bounded_correct)

            shuffled = shuffled_by_id[tournament.tournament_id]
            shuffled_rows = candidate_scores(model, tournament, shuffled.context)
            shuffled_selected = selection(shuffled_rows, "critic_score_bps")
            context_stable = shuffled_selected == critic_selected
            control_counts["context_total"] += 1
            control_counts["context_stable"] += int(context_stable)
            for original, controlled in zip(rows, shuffled_rows):
                context_score_deltas.append(
                    abs(int(original["critic_score_bps"]) - int(controlled["critic_score_bps"]))
                )

            punctuation_selected, _ = transform_selection(model, tournament, transform_punctuation)
            whitespace_selected, _ = transform_selection(model, tournament, transform_whitespace)
            unicode_selected, _ = transform_selection(model, tournament, transform_unicode)
            terminal_selected, _ = transform_selection(model, tournament, terminal_neutralized)
            for name, selected in (
                ("punctuation", punctuation_selected),
                ("whitespace", whitespace_selected),
                ("unicode", unicode_selected),
                ("terminal", terminal_selected),
            ):
                control_counts[f"{name}_total"] += 1
                control_counts[f"{name}_stable"] += int(selected == critic_selected)

            trace = {
                "seed": seed,
                "tournament_id": tournament.tournament_id,
                "group_id": tournament.group_id,
                "category": tournament.category,
                "gold_candidate_id": gold,
                "rule_selected_candidate_id": rule_selected,
                "critic_selected_candidate_id": critic_selected,
                "bounded_selected_candidate_id": bounded_selected,
                "shuffled_context_selected_candidate_id": shuffled_selected,
                "punctuation_normalized_selected_candidate_id": punctuation_selected,
                "terminal_neutralized_selected_candidate_id": terminal_selected,
                "rule_correct": rule_correct,
                "critic_correct": critic_correct,
                "bounded_correct": bounded_correct,
                "candidates": rows,
            }
            traces.append(trace)
            for row in rows:
                observation = dict(row)
                observation.update(
                    {
                        "seed": seed,
                        "tournament_id": tournament.tournament_id,
                        "category": tournament.category,
                    }
                )
                candidate_observations.append(observation)
                residual_counts[residual_bucket(int(row["learned_residual_bps"]))] += 1

            if rule_correct and not bounded_correct:
                flip_ledger.append(trace)
                category["deterministic_correct_to_wrong"] += 1
            if not rule_correct and bounded_correct:
                correction_ledger.append(trace)
                category["deterministic_wrong_to_correct"] += 1

    critic_scores = [float(item["critic_score_bps"]) for item in candidate_observations]
    correlations = {}
    for field in ("byte_length", "character_length", "word_count", "punctuation_count", "rule_score"):
        values = [float(item[field]) for item in candidate_observations]
        correlations[field] = {
            "pearson": round(pearson(critic_scores, values), 6),
            "spearman": round(spearman(critic_scores, values), 6),
        }

    total_candidates = len(candidate_observations)
    saturation = {
        "candidate_observations": total_candidates,
        "critic_at_or_below_500_bps": sum(value <= 500 for value in critic_scores),
        "critic_at_or_above_9500_bps": sum(value >= 9500 for value in critic_scores),
        "negative_residual_clipped": residual_counts["-250_clipped"],
        "positive_residual_clipped": residual_counts["+250_clipped"],
    }
    saturation["extreme_probability_rate_bps"] = _bps(
        saturation["critic_at_or_below_500_bps"]
        + saturation["critic_at_or_above_9500_bps"],
        total_candidates,
    )
    saturation["clipped_residual_rate_bps"] = _bps(
        saturation["negative_residual_clipped"]
        + saturation["positive_residual_clipped"],
        total_candidates,
    )

    gold_rule_margins = []
    gold_rule_top = 0
    for tournament in tournaments:
        gold = candidate_by_id(tournament, tournament.gold_candidate_id)
        alternatives = [
            candidate.rule_score
            for candidate in tournament.candidates
            if candidate.candidate_id != gold.candidate_id
        ]
        gold_rule_margins.append(gold.rule_score - max(alternatives))
        gold_rule_top += int(gold.rule_score >= max(alternatives))

    observation_count = len(traces)
    return {
        "bootstrap_target_audit": bootstrap_audit,
        "observation_scope": {
            "seeds": list(seeds),
            "test_observations": observation_count,
            "unique_tournaments": len({trace["tournament_id"] for trace in traces}),
            "candidate_observations": total_candidates,
            "note": "A tournament may appear in multiple seed-specific held-out test partitions.",
        },
        "headline": {
            "deterministic_top1_bps": _bps(
                sum(trace["rule_correct"] for trace in traces), observation_count
            ),
            "critic_top1_bps": _bps(
                sum(trace["critic_correct"] for trace in traces), observation_count
            ),
            "bounded_residual_top1_bps": _bps(
                sum(trace["bounded_correct"] for trace in traces), observation_count
            ),
            "deterministic_correct_to_wrong": len(flip_ledger),
            "deterministic_wrong_to_correct": len(correction_ledger),
            "net_corrections": len(correction_ledger) - len(flip_ledger),
            "newly_damaged_rate_bps": _bps(len(flip_ledger), observation_count),
        },
        "category_breakdown": {
            category: {
                "observations": counts["observations"],
                "deterministic_top1_bps": _bps(
                    counts["rule_correct"], counts["observations"]
                ),
                "critic_top1_bps": _bps(
                    counts["critic_correct"], counts["observations"]
                ),
                "bounded_residual_top1_bps": _bps(
                    counts["bounded_correct"], counts["observations"]
                ),
                "deterministic_correct_to_wrong": counts[
                    "deterministic_correct_to_wrong"
                ],
                "deterministic_wrong_to_correct": counts[
                    "deterministic_wrong_to_correct"
                ],
            }
            for category, counts in sorted(category_counts.items())
        },
        "shortcut_diagnostics": {
            "critic_score_correlations": correlations,
            "selection_stability_bps": {
                name: _bps(control_counts[f"{name}_stable"], control_counts[f"{name}_total"])
                for name in ("context", "punctuation", "whitespace", "unicode", "terminal")
            },
            "context_mean_absolute_score_delta_bps": round(_mean(context_score_deltas)),
            "output_saturation": saturation,
            "residual_histogram": dict(sorted(residual_counts.items())),
        },
        "rule_score_independence_audit": {
            "independence_verified": False,
            "gold_has_top_or_tied_rule_score": gold_rule_top,
            "tournaments": len(tournaments),
            "gold_has_top_or_tied_rule_score_rate_bps": _bps(gold_rule_top, len(tournaments)),
            "mean_gold_rule_margin": round(_mean(gold_rule_margins), 3),
            "minimum_gold_rule_margin": min(gold_rule_margins),
            "maximum_gold_rule_margin": max(gold_rule_margins),
            "finding": (
                "The corpus contains authored rule_score and gold_candidate_id fields. This diagnostic "
                "can measure their coupling but cannot prove independent provenance. Dataset V2 must "
                "compute rule scores before or separately from gold adjudication."
            ),
        },
        "deterministic_correct_to_wrong_ledger": flip_ledger,
        "deterministic_wrong_to_correct_ledger": correction_ledger,
        "per_tournament_score_traces": traces,
        "diagnosis": {
            "primary_failure": "training_target_mismatch",
            "secondary_failures": [
                "pairwise_training_does_not_match_four_to_eight_candidate_tournaments",
                "terminal_state_byte_rnn_is_exposed_to_surface_shortcuts",
                "context_is_injected_at_every_byte_instead_of_conditioning_a_pooled_surface_representation",
                "critic_is_not_trained_as_a_small_residual_or_to_abstain",
                "rule_score_and_gold_label_provenance_is_not_independently_demonstrated",
            ],
            "longer_training_on_bootstrap_pairs_recommended": False,
            "wider_residual_authority_recommended": False,
        },
        "promotion_contract": {
            "frozen": True,
            "required_gates": list(PROMOTION_GATES),
            "automatic_promotion": False,
        },
        "authority": {
            "critic_promotion_allowed": False,
            "runtime_chat_influence": False,
            "http_response_influence": False,
            "training_performed": False,
            "model_weights_modified": False,
            "l1d2_result_preserved": True,
        },
    }


def render_markdown(report: Mapping[str, Any]) -> str:
    headline = report["headline"]
    shortcuts = report["shortcut_diagnostics"]
    target = report["bootstrap_target_audit"]
    independence = report["rule_score_independence_audit"]
    lines = [
        "# STLM L1-D3A Failure Attribution",
        "",
        "## Decision",
        "",
        "The frozen critic remains unpromoted. L1-D3A performs diagnosis only and grants no live authority.",
        "",
        "## Headline",
        "",
        "| Metric | Result |",
        "|---|---:|",
        f"| Deterministic top-1 | {headline['deterministic_top1_bps'] / 100:.2f}% |",
        f"| Recurrent critic top-1 | {headline['critic_top1_bps'] / 100:.2f}% |",
        f"| Bounded residual top-1 | {headline['bounded_residual_top1_bps'] / 100:.2f}% |",
        f"| Deterministic correct to wrong | {headline['deterministic_correct_to_wrong']} |",
        f"| Deterministic wrong to correct | {headline['deterministic_wrong_to_correct']} |",
        f"| Net corrections | {headline['net_corrections']} |",
        "",
        "## Attribution",
        "",
        f"- Bootstrap pairs: {target['pairs']}",
        f"- Pairs carrying semantic, evidence, confidence, identity, or authority risk labels: {target['semantic_or_authority_risk_pairs']}",
        "- Primary failure: training target mismatch.",
        "- The current pairwise objective does not match four-to-eight-candidate valid-surface tournaments.",
        "- The terminal-state byte RNN is exposed to punctuation and ending shortcuts.",
        "- The model was not trained to emit a bounded residual or abstain.",
        "",
        "## Control stability",
        "",
    ]
    for name, value in shortcuts["selection_stability_bps"].items():
        lines.append(f"- {name}: {value / 100:.2f}%")
    lines.extend(
        [
            "",
            "## Rule-score independence audit",
            "",
            f"- Independence verified: {str(independence['independence_verified']).lower()}",
            f"- Gold top-or-tied by authored rule score: {independence['gold_has_top_or_tied_rule_score_rate_bps'] / 100:.2f}%",
            f"- Mean gold rule margin: {independence['mean_gold_rule_margin']}",
            "",
            "The next corpus must compute deterministic rule scores independently from gold adjudication.",
            "",
            "## Frozen promotion contract",
            "",
        ]
    )
    lines.extend(f"- [ ] {gate}" for gate in report["promotion_contract"]["required_gates"])
    lines.extend(
        [
            "",
            "No successful diagnostic run can promote the critic automatically.",
            "",
        ]
    )
    return "\n".join(lines)


def strip_nondeterministic(report: Mapping[str, Any]) -> dict[str, Any]:
    value = json.loads(json.dumps(report))
    value.pop("elapsed_ms", None)
    value.pop("within_runtime_budget", None)
    value.pop("report_sha256", None)
    return value


def run_once(args: argparse.Namespace, seeds: Sequence[int]) -> dict[str, Any]:
    model, model_digest = load_model(args.model)
    tournaments = load_surface_corpus(args.surface_corpus)
    bootstrap_audit = load_bootstrap_audit(args.bootstrap_pairs)
    report = analyze(model, tournaments, seeds, bootstrap_audit)
    report.update(
        {
            "schema_version": SCHEMA_VERSION,
            "experiment": "STLM L1-D3A",
            "model_sha256": model_digest,
            "surface_corpus_sha256": sha256_bytes(args.surface_corpus.read_bytes()),
            "bootstrap_pairs_sha256": sha256_bytes(args.bootstrap_pairs.read_bytes()),
            "seeds": list(seeds),
        }
    )
    return report


def main() -> None:
    args = parse_args()
    seeds = parse_seeds(args.seeds)
    started = time.perf_counter()
    first = run_once(args, seeds)
    second = run_once(args, seeds)
    deterministic_replay = strip_nondeterministic(first) == strip_nondeterministic(second)
    elapsed_ms = round((time.perf_counter() - started) * 1000.0, 3)
    report = first
    report["deterministic_full_report_replay"] = deterministic_replay
    report["elapsed_ms"] = elapsed_ms
    report["within_runtime_budget"] = elapsed_ms <= args.max_runtime_seconds * 1000.0
    report["structural_gate_passed"] = (
        deterministic_replay
        and report["within_runtime_budget"]
        and report["authority"]["critic_promotion_allowed"] is False
        and report["authority"]["training_performed"] is False
        and len(report["per_tournament_score_traces"]) == 60
        and set(report["category_breakdown"]) == set(CATEGORY_ORDER)
        and len(report["promotion_contract"]["required_gates"]) == len(PROMOTION_GATES)
    )
    report["report_sha256"] = hashlib.sha256(
        canonical_bytes(strip_nondeterministic(report))
    ).hexdigest()
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_md.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_bytes(canonical_bytes(report))
    args.output_md.write_text(render_markdown(report), encoding="utf-8")
    print(
        json.dumps(
            {
                "structural_gate_passed": report["structural_gate_passed"],
                "primary_failure": report["diagnosis"]["primary_failure"],
                "deterministic_correct_to_wrong": report["headline"][
                    "deterministic_correct_to_wrong"
                ],
                "net_corrections": report["headline"]["net_corrections"],
                "critic_promotion_allowed": False,
                "elapsed_ms": elapsed_ms,
                "report_sha256": report["report_sha256"],
            },
            sort_keys=True,
        )
    )
    if not report["structural_gate_passed"]:
        raise SystemExit("STLM L1-D3A structural diagnostic failed")


if __name__ == "__main__":
    main()
