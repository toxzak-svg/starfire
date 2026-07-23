#!/usr/bin/env python3
"""Fast, deterministic STLM L1-D1 phrase-critic ablation evaluation.

The evaluator turns model components on in a frozen order while keeping the
same held-out pairs, checkpoint, and scoring implementation. It intentionally
does not grant runtime authority or treat a quality score as promotion.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import random
import re
import statistics
import time
from collections import defaultdict
from pathlib import Path
from typing import Any, Iterable

SCHEMA_VERSION = 1
VOCABULARY_SIZE = 128
CONTEXT_SIZE = 8
CONTEXT_FIELDS = (
    "directness_bps",
    "warmth_bps",
    "energy_bps",
    "compression_bps",
    "playfulness_bps",
    "novelty_pressure_bps",
    "identity_relevance_bps",
    "semantic_specificity_bps",
)
STAGE_ORDER = (
    "bias_only",
    "token_embeddings",
    "recurrent_memory",
    "conversational_context",
    "full_identity_context",
    "reversed_label_control",
)
CATEGORY_ORDER = (
    "technical",
    "emotional",
    "uncertainty",
    "continuity",
    "disagreement",
    "adversarial",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--control-model", type=Path, required=True)
    parser.add_argument("--train", type=Path, required=True)
    parser.add_argument("--heldout", type=Path, required=True)
    parser.add_argument("--output-json", type=Path, required=True)
    parser.add_argument("--output-md", type=Path, required=True)
    parser.add_argument("--bootstrap-samples", type=int, default=2000)
    parser.add_argument("--seed", type=int, default=20260723)
    parser.add_argument("--max-runtime-seconds", type=float, default=60.0)
    return parser.parse_args()


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def load_model(path: Path) -> tuple[dict[str, Any], str]:
    payload = path.read_bytes()
    model = json.loads(payload)
    validate_model(model, path)
    return model, sha256_bytes(payload)


def validate_model(model: dict[str, Any], path: Path) -> None:
    hidden = model.get("hidden_size")
    if (
        model.get("schema_version") != SCHEMA_VERSION
        or model.get("vocabulary_size") != VOCABULARY_SIZE
        or model.get("context_size") != CONTEXT_SIZE
        or not isinstance(hidden, int)
        or not 1 <= hidden <= 128
    ):
        raise ValueError(f"{path}: invalid model header")

    expected = {
        "embeddings": (VOCABULARY_SIZE, hidden),
        "recurrent_weights": (hidden, hidden),
        "context_weights": (CONTEXT_SIZE, hidden),
    }
    for field, (rows, cols) in expected.items():
        matrix = model.get(field)
        if not isinstance(matrix, list) or len(matrix) != rows:
            raise ValueError(f"{path}: {field} row count mismatch")
        if any(not isinstance(row, list) or len(row) != cols for row in matrix):
            raise ValueError(f"{path}: {field} column count mismatch")

    for field in ("hidden_bias", "output_weights"):
        vector = model.get(field)
        if not isinstance(vector, list) or len(vector) != hidden:
            raise ValueError(f"{path}: {field} shape mismatch")

    numbers: list[float] = []
    for field in ("embeddings", "recurrent_weights", "context_weights"):
        numbers.extend(float(value) for row in model[field] for value in row)
    numbers.extend(float(value) for value in model["hidden_bias"])
    numbers.extend(float(value) for value in model["output_weights"])
    numbers.append(float(model["output_bias"]))
    if any(not math.isfinite(value) for value in numbers):
        raise ValueError(f"{path}: non-finite model weight")


def load_pairs(path: Path, require_category: bool) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        record = json.loads(line)
        validate_pair(record, path, line_number, require_category)
        records.append(record)
    if not records:
        raise ValueError(f"{path}: empty corpus")
    return records


def validate_pair(
    record: dict[str, Any], path: Path, line_number: int, require_category: bool
) -> None:
    prefix = f"{path}:{line_number}"
    if not str(record.get("source_id", "")).strip():
        raise ValueError(f"{prefix}: source_id is required")
    if require_category and record.get("category") not in CATEGORY_ORDER:
        raise ValueError(f"{prefix}: unknown or missing category")
    context = record.get("context")
    if not isinstance(context, dict):
        raise ValueError(f"{prefix}: context must be an object")
    for field in CONTEXT_FIELDS:
        value = context.get(field)
        if not isinstance(value, int) or not 0 <= value <= 10_000:
            raise ValueError(f"{prefix}: {field} must be an integer in [0, 10000]")
    preferred = str(record.get("preferred", "")).strip()
    rejected = str(record.get("rejected", "")).strip()
    if not preferred or not rejected or preferred == rejected:
        raise ValueError(f"{prefix}: preferred/rejected text must be distinct")
    labels = record.get("failure_labels", [])
    if not isinstance(labels, list) or any(not isinstance(label, str) for label in labels):
        raise ValueError(f"{prefix}: failure_labels must be a string list")


def normalized_text(text: str) -> str:
    return " ".join(re.findall(r"[a-z0-9]+", text.lower()))


def word_ngrams(text: str, n: int = 5) -> set[tuple[str, ...]]:
    words = normalized_text(text).split()
    return {tuple(words[index : index + n]) for index in range(max(0, len(words) - n + 1))}


def leakage_report(
    train: list[dict[str, Any]], heldout: list[dict[str, Any]]
) -> dict[str, Any]:
    train_sources = {record["source_id"] for record in train}
    test_sources = {record["source_id"] for record in heldout}
    train_texts = {
        normalized_text(record[field]) for record in train for field in ("preferred", "rejected")
    }
    test_texts = {
        normalized_text(record[field]) for record in heldout for field in ("preferred", "rejected")
    }
    exact_text_overlap = sorted(train_texts & test_texts)
    source_overlap = sorted(train_sources & test_sources)

    train_fivegrams = set()
    for record in train:
        train_fivegrams |= word_ngrams(record["preferred"])
        train_fivegrams |= word_ngrams(record["rejected"])
    test_fivegrams = set()
    for record in heldout:
        test_fivegrams |= word_ngrams(record["preferred"])
        test_fivegrams |= word_ngrams(record["rejected"])
    union = train_fivegrams | test_fivegrams
    fivegram_jaccard = len(train_fivegrams & test_fivegrams) / len(union) if union else 0.0

    return {
        "train_source_count": len(train_sources),
        "heldout_source_count": len(test_sources),
        "source_id_overlap": source_overlap,
        "normalized_exact_text_overlap": exact_text_overlap,
        "word_fivegram_jaccard": round(fivegram_jaccard, 8),
        "passed": not source_overlap and not exact_text_overlap,
    }


def context_vector(record: dict[str, Any], identity_neutralized: bool) -> list[float]:
    values = [record["context"][field] for field in CONTEXT_FIELDS]
    if identity_neutralized:
        values[6] = 5000
    return [value / 5000.0 - 1.0 for value in values]


def score_text(
    model: dict[str, Any],
    record: dict[str, Any],
    text: str,
    *,
    use_embeddings: bool,
    use_recurrence: bool,
    use_context: bool,
    identity_neutralized: bool,
) -> int:
    hidden_size = model["hidden_size"]
    context_projection = [0.0] * hidden_size
    if use_context:
        context = context_vector(record, identity_neutralized)
        for context_index, context_value in enumerate(context):
            for hidden_index in range(hidden_size):
                context_projection[hidden_index] += (
                    context_value * model["context_weights"][context_index][hidden_index]
                )

    hidden = [0.0] * hidden_size
    for byte in text.encode("utf-8"):
        token = byte if byte < 128 else 127
        previous = hidden[:]
        next_hidden = [0.0] * hidden_size
        for hidden_index in range(hidden_size):
            activation = float(model["hidden_bias"][hidden_index])
            if use_embeddings:
                activation += float(model["embeddings"][token][hidden_index])
            if use_context:
                activation += context_projection[hidden_index]
            if use_recurrence:
                activation += sum(
                    previous_value
                    * float(model["recurrent_weights"][hidden_index][previous_index])
                    for previous_index, previous_value in enumerate(previous)
                )
            next_hidden[hidden_index] = math.tanh(activation)
        hidden = next_hidden

    logit = float(model["output_bias"]) + sum(
        float(weight) * value for weight, value in zip(model["output_weights"], hidden)
    )
    logit = max(-20.0, min(20.0, logit))
    probability = 1.0 / (1.0 + math.exp(-logit))
    return round(probability * 10_000)


def stage_config(stage: str) -> dict[str, bool]:
    configs = {
        "bias_only": {
            "use_embeddings": False,
            "use_recurrence": False,
            "use_context": False,
            "identity_neutralized": True,
        },
        "token_embeddings": {
            "use_embeddings": True,
            "use_recurrence": False,
            "use_context": False,
            "identity_neutralized": True,
        },
        "recurrent_memory": {
            "use_embeddings": True,
            "use_recurrence": True,
            "use_context": False,
            "identity_neutralized": True,
        },
        "conversational_context": {
            "use_embeddings": True,
            "use_recurrence": True,
            "use_context": True,
            "identity_neutralized": True,
        },
        "full_identity_context": {
            "use_embeddings": True,
            "use_recurrence": True,
            "use_context": True,
            "identity_neutralized": False,
        },
        "reversed_label_control": {
            "use_embeddings": True,
            "use_recurrence": True,
            "use_context": True,
            "identity_neutralized": False,
        },
    }
    return configs[stage]


def decision(margin: int) -> int:
    if margin > 0:
        return 1
    if margin < 0:
        return -1
    return 0


def correctness(state: int) -> float:
    return 1.0 if state == 1 else 0.5 if state == 0 else 0.0


def evaluate_stage(
    stage: str,
    model: dict[str, Any],
    heldout: list[dict[str, Any]],
) -> dict[str, Any]:
    config = stage_config(stage)
    started = time.perf_counter()
    examples: list[dict[str, Any]] = []
    category_values: dict[str, list[float]] = defaultdict(list)
    margins: list[int] = []

    for record in heldout:
        preferred_score = score_text(model, record, record["preferred"], **config)
        rejected_score = score_text(model, record, record["rejected"], **config)
        preferred_replay = score_text(model, record, record["preferred"], **config)
        rejected_replay = score_text(model, record, record["rejected"], **config)
        replay_match = (
            preferred_score == preferred_replay and rejected_score == rejected_replay
        )
        margin = preferred_score - rejected_score
        state = decision(margin)
        value = correctness(state)
        margins.append(margin)
        category_values[record["category"]].append(value)
        examples.append(
            {
                "source_id": record["source_id"],
                "category": record["category"],
                "preferred_score_bps": preferred_score,
                "rejected_score_bps": rejected_score,
                "margin_bps": margin,
                "decision": state,
                "correctness": value,
                "replay_match": replay_match,
                "failure_labels": sorted(set(record.get("failure_labels", []))),
            }
        )

    elapsed_ms = round((time.perf_counter() - started) * 1000.0, 3)
    accuracy = statistics.fmean(item["correctness"] for item in examples)
    by_category = {
        category: {
            "examples": len(category_values[category]),
            "accuracy_bps": round(statistics.fmean(category_values[category]) * 10_000),
        }
        for category in CATEGORY_ORDER
    }
    return {
        "stage": stage,
        "examples": len(examples),
        "accuracy_bps": round(accuracy * 10_000),
        "preferred_wins": sum(item["decision"] == 1 for item in examples),
        "ties": sum(item["decision"] == 0 for item in examples),
        "rejected_wins": sum(item["decision"] == -1 for item in examples),
        "mean_margin_bps": round(statistics.fmean(margins), 3),
        "median_margin_bps": statistics.median(margins),
        "minimum_margin_bps": min(margins),
        "maximum_margin_bps": max(margins),
        "exact_replay": all(item["replay_match"] for item in examples),
        "category_metrics": by_category,
        "example_results": examples,
        "elapsed_ms": elapsed_ms,
    }


def percentile(values: list[float], quantile: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = quantile * (len(ordered) - 1)
    lower = math.floor(index)
    upper = math.ceil(index)
    if lower == upper:
        return ordered[lower]
    weight = index - lower
    return ordered[lower] * (1.0 - weight) + ordered[upper] * weight


def paired_bootstrap_ci(
    previous: list[dict[str, Any]],
    current: list[dict[str, Any]],
    samples: int,
    seed: int,
) -> tuple[int, int]:
    differences = [
        current_item["correctness"] - previous_item["correctness"]
        for previous_item, current_item in zip(previous, current)
    ]
    rng = random.Random(seed)
    n = len(differences)
    estimates = []
    for _ in range(samples):
        estimate = statistics.fmean(differences[rng.randrange(n)] for _ in range(n))
        estimates.append(estimate)
    return (
        round(percentile(estimates, 0.025) * 10_000),
        round(percentile(estimates, 0.975) * 10_000),
    )


def exact_two_sided_binomial_p(left: int, right: int) -> float:
    total = left + right
    if total == 0:
        return 1.0
    observed = min(left, right)
    probability = sum(math.comb(total, k) for k in range(observed + 1)) / (2**total)
    return min(1.0, 2.0 * probability)


def compare_stages(
    previous: dict[str, Any],
    current: dict[str, Any],
    bootstrap_samples: int,
    seed: int,
) -> dict[str, Any]:
    previous_examples = previous["example_results"]
    current_examples = current["example_results"]
    beneficial = []
    harmful = []
    changed = []
    neutral_changes = []
    positive_differences = 0
    negative_differences = 0

    for previous_item, current_item in zip(previous_examples, current_examples):
        if previous_item["source_id"] != current_item["source_id"]:
            raise ValueError("held-out order drifted between ablation stages")
        old = previous_item["decision"]
        new = current_item["decision"]
        old_value = previous_item["correctness"]
        new_value = current_item["correctness"]
        source_id = current_item["source_id"]
        if old != new:
            changed.append(source_id)
        if new_value > old_value:
            beneficial.append(source_id)
            positive_differences += 1
        elif new_value < old_value:
            harmful.append(source_id)
            negative_differences += 1
        elif old != new:
            neutral_changes.append(source_id)

    low, high = paired_bootstrap_ci(
        previous_examples,
        current_examples,
        bootstrap_samples,
        seed,
    )
    return {
        "from_stage": previous["stage"],
        "to_stage": current["stage"],
        "delta_accuracy_bps": current["accuracy_bps"] - previous["accuracy_bps"],
        "paired_bootstrap_95_ci_bps": [low, high],
        "beneficial_flips": beneficial,
        "harmful_flips": harmful,
        "neutral_changed_decisions": neutral_changes,
        "changed_decision_count": len(changed),
        "paired_sign_exact_p": round(
            exact_two_sided_binomial_p(positive_differences, negative_differences), 8
        ),
    }


def hard_gate_probe() -> dict[str, Any]:
    cases = [
        {
            "id": "eligible",
            "semantic_verified": True,
            "slots_preserved": True,
            "identity_conflicts": 0,
            "expected": True,
        },
        {
            "id": "semantic_drift",
            "semantic_verified": False,
            "slots_preserved": True,
            "identity_conflicts": 0,
            "expected": False,
        },
        {
            "id": "slot_loss",
            "semantic_verified": True,
            "slots_preserved": False,
            "identity_conflicts": 0,
            "expected": False,
        },
        {
            "id": "identity_conflict",
            "semantic_verified": True,
            "slots_preserved": True,
            "identity_conflicts": 1,
            "expected": False,
        },
    ]
    observations = []
    for case in cases:
        observed = (
            case["semantic_verified"]
            and case["slots_preserved"]
            and case["identity_conflicts"] == 0
        )
        observations.append(
            {"id": case["id"], "observed": observed, "expected": case["expected"]}
        )
    return {
        "cases": observations,
        "passed": all(item["observed"] == item["expected"] for item in observations),
        "hard_gates_ablatable": False,
    }


def corpus_digest(records: Iterable[dict[str, Any]]) -> str:
    return sha256_bytes(canonical_bytes(list(records)))


def deterministic_core(
    full_model: dict[str, Any],
    control_model: dict[str, Any],
    train: list[dict[str, Any]],
    heldout: list[dict[str, Any]],
    full_model_digest: str,
    control_model_digest: str,
    bootstrap_samples: int,
    seed: int,
) -> dict[str, Any]:
    stage_reports = []
    for stage in STAGE_ORDER:
        model = control_model if stage == "reversed_label_control" else full_model
        stage_reports.append(evaluate_stage(stage, model, heldout))

    comparisons = [
        compare_stages(
            stage_reports[index - 1],
            stage_reports[index],
            bootstrap_samples,
            seed + index * 1009,
        )
        for index in range(1, len(stage_reports) - 1)
    ]
    control_comparison = compare_stages(
        stage_reports[4],
        stage_reports[5],
        bootstrap_samples,
        seed + 99991,
    )

    leakage = leakage_report(train, heldout)
    hard_gate = hard_gate_probe()
    exact_replay = all(stage["exact_replay"] for stage in stage_reports)
    category_coverage = sorted({record["category"] for record in heldout}) == sorted(
        CATEGORY_ORDER
    )
    structural_gate_passed = (
        [stage["stage"] for stage in stage_reports] == list(STAGE_ORDER)
        and exact_replay
        and leakage["passed"]
        and hard_gate["passed"]
        and category_coverage
        and len(heldout) >= 24
    )

    return {
        "experiment": "STLM_L1D1_FAST_FACTORIAL_ABLATION",
        "schema_version": 1,
        "activation_order": list(STAGE_ORDER),
        "authority": {
            "evaluation_only": True,
            "hard_gates_ablatable": False,
            "runtime_chat_influence": False,
            "http_response_influence": False,
            "automatic_promotion": False,
        },
        "model_sha256": full_model_digest,
        "reversed_label_model_sha256": control_model_digest,
        "train_corpus_sha256": corpus_digest(train),
        "heldout_corpus_sha256": corpus_digest(heldout),
        "train_examples": len(train),
        "heldout_examples": len(heldout),
        "bootstrap_samples": bootstrap_samples,
        "seed": seed,
        "leakage": leakage,
        "hard_gate_probe": hard_gate,
        "category_coverage": category_coverage,
        "exact_replay": exact_replay,
        "stages": stage_reports,
        "incremental_comparisons": comparisons,
        "full_vs_reversed_label_control": control_comparison,
        "structural_gate_passed": structural_gate_passed,
        "quality_can_enable_live_authority": False,
    }


def strip_nondeterministic(core: dict[str, Any]) -> dict[str, Any]:
    value = copy.deepcopy(core)
    for stage in value["stages"]:
        stage.pop("elapsed_ms", None)
    return value


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# STLM L1-D1 Fast Factorial Ablation",
        "",
        f"- Structural gate: **{'PASS' if report['structural_gate_passed'] else 'FAIL'}**",
        f"- Held-out pairs: **{report['heldout_examples']}**",
        f"- Deterministic replay: **{report['exact_replay']}**",
        f"- Train/test leakage check: **{report['leakage']['passed']}**",
        f"- Total evaluation time: **{report['elapsed_ms']} ms**",
        "",
        "## Activation ladder",
        "",
        "| Stage | Accuracy | Preferred | Ties | Rejected | Mean margin | Time |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for stage in report["stages"]:
        lines.append(
            f"| `{stage['stage']}` | {stage['accuracy_bps'] / 100:.2f}% | "
            f"{stage['preferred_wins']} | {stage['ties']} | {stage['rejected_wins']} | "
            f"{stage['mean_margin_bps']:.1f} bps | {stage['elapsed_ms']:.3f} ms |"
        )

    lines.extend(
        [
            "",
            "## Incremental effects",
            "",
            "| From → To | Δ accuracy | 95% paired bootstrap CI | Helpful flips | Harmful flips | Paired sign p |",
            "|---|---:|---:|---:|---:|---:|",
        ]
    )
    for comparison in report["incremental_comparisons"]:
        low, high = comparison["paired_bootstrap_95_ci_bps"]
        lines.append(
            f"| `{comparison['from_stage']}` → `{comparison['to_stage']}` | "
            f"{comparison['delta_accuracy_bps'] / 100:+.2f}% | "
            f"[{low / 100:+.2f}%, {high / 100:+.2f}%] | "
            f"{len(comparison['beneficial_flips'])} | "
            f"{len(comparison['harmful_flips'])} | "
            f"{comparison['paired_sign_exact_p']:.6f} |"
        )

    lines.extend(["", "## Category accuracy", ""])
    header = "| Stage | " + " | ".join(category.title() for category in CATEGORY_ORDER) + " |"
    separator = "|---|" + "|".join("---:" for _ in CATEGORY_ORDER) + "|"
    lines.extend([header, separator])
    for stage in report["stages"]:
        values = " | ".join(
            f"{stage['category_metrics'][category]['accuracy_bps'] / 100:.1f}%"
            for category in CATEGORY_ORDER
        )
        lines.append(f"| `{stage['stage']}` | {values} |")

    control = report["full_vs_reversed_label_control"]
    lines.extend(
        [
            "",
            "## Control and interpretation",
            "",
            f"The reversed-label control changed held-out accuracy by "
            f"**{control['delta_accuracy_bps'] / 100:+.2f}%** relative to the full critic.",
            "",
            "This report measures component effects on a small frozen held-out corpus. "
            "It does not authorize shadow or live response influence, even when a stage improves.",
            "",
            "## Changed decisions",
            "",
        ]
    )
    for comparison in report["incremental_comparisons"]:
        lines.append(
            f"- `{comparison['from_stage']}` → `{comparison['to_stage']}`: "
            f"{comparison['changed_decision_count']} changed, "
            f"{len(comparison['beneficial_flips'])} helpful, "
            f"{len(comparison['harmful_flips'])} harmful."
        )
    return "\n".join(lines) + "\n"


def main() -> None:
    args = parse_args()
    if args.bootstrap_samples < 200:
        raise SystemExit("bootstrap-samples must be at least 200")
    started = time.perf_counter()

    full_model, full_digest = load_model(args.model)
    control_model, control_digest = load_model(args.control_model)
    train = load_pairs(args.train, require_category=False)
    heldout = load_pairs(args.heldout, require_category=True)

    first = deterministic_core(
        full_model,
        control_model,
        train,
        heldout,
        full_digest,
        control_digest,
        args.bootstrap_samples,
        args.seed,
    )
    second = deterministic_core(
        full_model,
        control_model,
        train,
        heldout,
        full_digest,
        control_digest,
        args.bootstrap_samples,
        args.seed,
    )
    deterministic_match = strip_nondeterministic(first) == strip_nondeterministic(second)
    elapsed_ms = round((time.perf_counter() - started) * 1000.0, 3)

    report = first
    report["deterministic_full_report_replay"] = deterministic_match
    report["elapsed_ms"] = elapsed_ms
    report["within_runtime_budget"] = elapsed_ms <= args.max_runtime_seconds * 1000.0
    report["structural_gate_passed"] = (
        report["structural_gate_passed"]
        and deterministic_match
        and report["within_runtime_budget"]
    )
    deterministic_payload = strip_nondeterministic(report)
    deterministic_payload.pop("elapsed_ms", None)
    deterministic_payload.pop("within_runtime_budget", None)
    report["deterministic_report_sha256"] = sha256_bytes(canonical_bytes(deterministic_payload))

    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_md.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_bytes(canonical_bytes(report))
    args.output_md.write_text(render_markdown(report), encoding="utf-8")

    print(
        json.dumps(
            {
                "structural_gate_passed": report["structural_gate_passed"],
                "heldout_examples": report["heldout_examples"],
                "full_accuracy_bps": report["stages"][4]["accuracy_bps"],
                "control_accuracy_bps": report["stages"][5]["accuracy_bps"],
                "elapsed_ms": report["elapsed_ms"],
                "report_sha256": report["deterministic_report_sha256"],
            },
            sort_keys=True,
        )
    )
    if not report["structural_gate_passed"]:
        raise SystemExit("STLM L1-D1 structural evaluation gate failed")


if __name__ == "__main__":
    main()
