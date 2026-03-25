from __future__ import annotations

import argparse
import csv
from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = PROJECT_ROOT / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from metacog_confidence import run_controller_sweep


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Sweep controller settings and report calibration/compute tradeoffs.",
    )
    parser.add_argument(
        "--threshold-gains",
        default="0.2,0.6,1.0",
        help="Comma-separated threshold_gain values.",
    )
    parser.add_argument(
        "--threshold-relaxations",
        default="0.1,0.3,0.5",
        help="Comma-separated threshold_relaxation values.",
    )
    parser.add_argument(
        "--attention-gains",
        default="0.5,1.0,1.5",
        help="Comma-separated attention_gain values.",
    )
    parser.add_argument(
        "--signal-strengths",
        default="0.2,0.4,0.8",
        help="Comma-separated absolute evidence strengths. Ignored if --signal-traces is set.",
    )
    parser.add_argument(
        "--signal-traces",
        help=(
            "Optional semicolon-separated traces, where each trace is comma-separated. "
            "Example: 0.1,0.3,0.8;-0.1,-0.4,-0.9"
        ),
    )
    parser.add_argument(
        "--repeats",
        type=int,
        default=8,
        help="How many seeded repeats to run per signal workload.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional CSV output path.",
    )
    args = parser.parse_args()

    records = run_controller_sweep(
        threshold_gains=_parse_float_list(args.threshold_gains),
        threshold_relaxations=_parse_float_list(args.threshold_relaxations),
        attention_gains=_parse_float_list(args.attention_gains),
        signal_strengths=None if args.signal_traces else _parse_float_list(args.signal_strengths),
        signal_traces=None if not args.signal_traces else _parse_trace_list(args.signal_traces),
        repeats=args.repeats,
    )
    rows = [record.to_row() for record in records]
    _write_rows(rows, output_path=args.output)


def _parse_float_list(raw_values: str) -> list[float]:
    return [float(value) for value in raw_values.split(",") if value.strip()]


def _parse_trace_list(raw_values: str) -> list[list[float]]:
    return [_parse_float_list(trace) for trace in raw_values.split(";") if trace.strip()]


def _write_rows(
    rows: list[dict[str, float]],
    *,
    output_path: Path | None,
) -> None:
    if not rows:
        return

    fieldnames = list(rows[0].keys())
    writer = csv.DictWriter(sys.stdout, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(rows)

    if output_path is None:
        return

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        file_writer = csv.DictWriter(handle, fieldnames=fieldnames)
        file_writer.writeheader()
        file_writer.writerows(rows)


if __name__ == "__main__":
    main()
