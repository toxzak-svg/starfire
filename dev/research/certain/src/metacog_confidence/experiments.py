from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from itertools import product

from .core import (
    AccumulatorConfig,
    ConfidenceConfig,
    ControllerConfig,
    EvaluationSummary,
    MetacognitiveLoop,
    SequentialConfidenceBaseline,
    evaluate_results,
)


@dataclass(frozen=True)
class SweepRecord:
    controller: ControllerConfig
    online: EvaluationSummary
    sequential: EvaluationSummary

    def to_row(self) -> dict[str, float]:
        return {
            "low_confidence": self.controller.low_confidence,
            "high_confidence": self.controller.high_confidence,
            "threshold_gain": self.controller.threshold_gain,
            "threshold_relaxation": self.controller.threshold_relaxation,
            "attention_gain": self.controller.attention_gain,
            "online_accuracy": self.online.accuracy,
            "online_decision_rate": self.online.decision_rate,
            "online_mean_confidence": self.online.mean_confidence,
            "online_mean_steps": self.online.mean_steps,
            "online_ece": self.online.expected_calibration_error,
            "sequential_accuracy": self.sequential.accuracy,
            "sequential_decision_rate": self.sequential.decision_rate,
            "sequential_mean_confidence": self.sequential.mean_confidence,
            "sequential_mean_steps": self.sequential.mean_steps,
            "sequential_ece": self.sequential.expected_calibration_error,
            "accuracy_delta": self.online.accuracy - self.sequential.accuracy,
            "ece_delta": (
                self.online.expected_calibration_error
                - self.sequential.expected_calibration_error
            ),
            "mean_steps_delta": self.online.mean_steps - self.sequential.mean_steps,
        }


def run_controller_sweep(
    *,
    threshold_gains: Sequence[float],
    threshold_relaxations: Sequence[float],
    attention_gains: Sequence[float],
    signal_strengths: Sequence[float] | None = None,
    signal_traces: Sequence[Sequence[float]] | None = None,
    repeats: int,
    accumulator: AccumulatorConfig | None = None,
    confidence: ConfidenceConfig | None = None,
    controller: ControllerConfig | None = None,
) -> list[SweepRecord]:
    if repeats < 1:
        raise ValueError("repeats must be at least 1")
    if not threshold_gains or not threshold_relaxations or not attention_gains:
        raise ValueError("controller sweep ranges must be non-empty")

    accumulator_config = accumulator or AccumulatorConfig()
    confidence_config = confidence or ConfidenceConfig()
    controller_config = controller or ControllerConfig()
    dataset = _build_dataset(
        signal_strengths=signal_strengths,
        signal_traces=signal_traces,
        repeats=repeats,
    )

    records: list[SweepRecord] = []
    for threshold_gain, threshold_relaxation, attention_gain in product(
        threshold_gains,
        threshold_relaxations,
        attention_gains,
    ):
        current_controller = ControllerConfig(
            low_confidence=controller_config.low_confidence,
            high_confidence=controller_config.high_confidence,
            threshold_gain=threshold_gain,
            threshold_relaxation=threshold_relaxation,
            attention_gain=attention_gain,
            min_threshold_scale=controller_config.min_threshold_scale,
        )
        loop = MetacognitiveLoop(
            accumulator=accumulator_config,
            confidence=confidence_config,
            controller=current_controller,
        )
        baseline = SequentialConfidenceBaseline(
            accumulator=accumulator_config,
            confidence=confidence_config,
        )

        online_results = []
        sequential_results = []
        labels = []
        for signal, label, seed in dataset:
            online_results.append(loop.run_trial(signal, seed=seed))
            sequential_results.append(baseline.run_trial(signal, seed=seed))
            labels.append(label)

        records.append(
            SweepRecord(
                controller=current_controller,
                online=evaluate_results(online_results, labels),
                sequential=evaluate_results(sequential_results, labels),
            )
        )

    return records


def _build_dataset(
    *,
    signal_strengths: Sequence[float] | None,
    signal_traces: Sequence[Sequence[float]] | None,
    repeats: int,
) -> list[tuple[float | tuple[float, ...], int, int]]:
    if signal_traces is not None:
        return _build_trace_dataset(signal_traces, repeats)
    if not signal_strengths:
        raise ValueError("signal_strengths must contain at least one value")

    dataset: list[tuple[float, int, int]] = []
    seed = 0
    for _ in range(repeats):
        for strength in signal_strengths:
            magnitude = abs(float(strength))
            if magnitude == 0.0:
                continue

            dataset.append((magnitude, 1, seed))
            seed += 1
            dataset.append((-magnitude, -1, seed))
            seed += 1

    if not dataset:
        raise ValueError("signal_strengths must include a non-zero value")
    return dataset


def _build_trace_dataset(
    signal_traces: Sequence[Sequence[float]],
    repeats: int,
) -> list[tuple[tuple[float, ...], int, int]]:
    if not signal_traces:
        raise ValueError("signal_traces must contain at least one trace")

    dataset: list[tuple[tuple[float, ...], int, int]] = []
    seed = 0
    for _ in range(repeats):
        for trace in signal_traces:
            materialized = tuple(float(value) for value in trace)
            if not materialized:
                raise ValueError("each signal trace must contain at least one value")

            dataset.append((materialized, _infer_trace_label(materialized), seed))
            seed += 1

    return dataset


def _infer_trace_label(trace: Sequence[float]) -> int:
    for value in reversed(trace):
        if value > 0.0:
            return 1
        if value < 0.0:
            return -1
    raise ValueError("each signal trace must contain at least one non-zero value")
