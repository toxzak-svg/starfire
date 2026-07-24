#!/usr/bin/env python3
"""STLM L1-D2 valid-surface benchmark and critic-v2 preflight."""

from l1d2_core import *
from l1d2_rankers import *
from l1d2_evaluation import *


def main() -> None:
    args = parse_args()
    seeds = parse_seeds(args.seeds)
    started = time.perf_counter()
    model, model_digest = load_model(args.model)
    tournaments = load_surface_corpus(args.surface_corpus)
    probes = load_invalid_corpus(args.semantic_invalid_corpus)
    first = deterministic_core(
        model,
        model_digest,
        args.surface_corpus,
        args.semantic_invalid_corpus,
        tournaments,
        probes,
        seeds,
    )
    second = deterministic_core(
        model,
        model_digest,
        args.surface_corpus,
        args.semantic_invalid_corpus,
        tournaments,
        probes,
        seeds,
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
    report["deterministic_report_sha256"] = sha256_bytes(
        canonical_bytes(strip_nondeterministic(report))
    )
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_md.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_bytes(canonical_bytes(report))
    args.output_md.write_text(render_markdown(report), encoding="utf-8")
    print(
        json.dumps(
            {
                "structural_gate_passed": report["structural_gate_passed"],
                "surface_tournaments": report["surface_corpus"]["tournaments"],
                "surface_candidates": report["surface_corpus"]["surface_candidates"],
                "semantic_invalid_probes": report["semantic_invalid_probe"]["probes"],
                "bounded_residual_top1_mean_bps": report["aggregate_metrics"]["bounded_residual"]["top1_accuracy_mean_bps"],
                "hashed_ngram_top1_mean_bps": report["aggregate_metrics"]["hashed_ngram"]["top1_accuracy_mean_bps"],
                "elapsed_ms": elapsed_ms,
                "report_sha256": report["deterministic_report_sha256"],
            },
            sort_keys=True,
        )
    )
    if not report["structural_gate_passed"]:
        raise SystemExit("STLM L1-D2 structural preflight failed")


if __name__ == "__main__":
    main()
