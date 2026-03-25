from __future__ import annotations

from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from itertools import tee
import math
import random


@dataclass(frozen=True)
class AccumulatorConfig:
    dt: float = 0.05
    base_drift: float = 1.0
    noise_std: float = 0.2
    base_threshold: float = 1.0
    max_steps: int = 200


@dataclass(frozen=True)
class ConfidenceConfig:
    variance_floor: float = 0.05
    meta_uncertainty: float = 0.2
    slope: float = 1.5
    ema_decay: float = 0.9


@dataclass(frozen=True)
class ControllerConfig:
    low_confidence: float = 0.45
    high_confidence: float = 0.8
    threshold_gain: float = 0.6
    threshold_relaxation: float = 0.3
    attention_gain: float = 1.0
    min_threshold_scale: float = 0.5


@dataclass(frozen=True)
class StepSnapshot:
    step: int
    time: float
    evidence: float
    variance: float
    confidence: float
    threshold: float
    attention_scale: float
    decision: int | None


@dataclass(frozen=True)
class TrialResult:
    steps: list[StepSnapshot]
    decision: int | None
    decided: bool
    final_confidence: float


@dataclass(frozen=True)
class TrialComparison:
    online: TrialResult
    sequential: TrialResult


@dataclass(frozen=True)
class EvaluationSummary:
    total_trials: int
    decision_rate: float
    accuracy: float
    mean_confidence: float
    mean_steps: float
    mean_elapsed_time: float
    expected_calibration_error: float


class ConfidenceMonitor:
    def __init__(self, config: ConfidenceConfig) -> None:
        self.config = config

    def estimate(self, evidence: float, variance: float, elapsed_time: float) -> float:
        bounded_variance = max(variance, self.config.variance_floor)
        uncertainty = math.sqrt(bounded_variance)
        time_penalty = self.config.meta_uncertainty / math.sqrt(elapsed_time + 1.0)
        z_score = abs(evidence) / max(uncertainty + time_penalty, 1e-6)
        return 1.0 / (1.0 + math.exp(-self.config.slope * (z_score - 1.0)))


class MetacognitiveController:
    def __init__(self, config: ControllerConfig, base_threshold: float) -> None:
        self.config = config
        self.base_threshold = base_threshold

    def policy(self, confidence: float) -> tuple[float, float]:
        low_gap = max(0.0, self.config.low_confidence - confidence)
        high_gap = max(0.0, confidence - self.config.high_confidence)

        threshold_scale = 1.0 + self.config.threshold_gain * low_gap
        threshold_scale -= self.config.threshold_relaxation * high_gap
        threshold_scale = max(self.config.min_threshold_scale, threshold_scale)

        attention_scale = 1.0 + self.config.attention_gain * low_gap
        return self.base_threshold * threshold_scale, attention_scale


class SequentialConfidenceBaseline:
    def __init__(
        self,
        accumulator: AccumulatorConfig | None = None,
        confidence: ConfidenceConfig | None = None,
    ) -> None:
        self.accumulator = accumulator or AccumulatorConfig()
        self.confidence = confidence or ConfidenceConfig()
        self._monitor = ConfidenceMonitor(self.confidence)

    def run_trial(
        self,
        signal: float | Iterable[float],
        *,
        seed: int | None = None,
    ) -> TrialResult:
        rng = random.Random(seed)
        evidence = 0.0
        variance = self.confidence.variance_floor
        decision: int | None = None
        steps: list[StepSnapshot] = []
        stream = _signal_stream(signal)

        for step_index in range(1, self.accumulator.max_steps + 1):
            current_time = step_index * self.accumulator.dt
            sensory_evidence = next(stream)
            drift = self.accumulator.base_drift * sensory_evidence
            noise = rng.gauss(0.0, self.accumulator.noise_std) * math.sqrt(self.accumulator.dt)
            increment = drift * self.accumulator.dt + noise

            evidence += increment
            variance = _update_variance(self.confidence, variance, increment)

            if abs(evidence) >= self.accumulator.base_threshold:
                decision = 1 if evidence > 0 else -1

            steps.append(
                StepSnapshot(
                    step=step_index,
                    time=current_time,
                    evidence=evidence,
                    variance=variance,
                    confidence=0.0,
                    threshold=self.accumulator.base_threshold,
                    attention_scale=1.0,
                    decision=decision,
                )
            )

            if decision is not None:
                break

        final_time = steps[-1].time if steps else 0.0
        final_confidence = self._monitor.estimate(evidence, variance, elapsed_time=final_time)

        if steps:
            last_step = steps[-1]
            steps[-1] = StepSnapshot(
                step=last_step.step,
                time=last_step.time,
                evidence=last_step.evidence,
                variance=last_step.variance,
                confidence=final_confidence,
                threshold=last_step.threshold,
                attention_scale=last_step.attention_scale,
                decision=last_step.decision,
            )

        return TrialResult(
            steps=steps,
            decision=decision,
            decided=decision is not None,
            final_confidence=final_confidence,
        )


class MetacognitiveLoop:
    def __init__(
        self,
        accumulator: AccumulatorConfig | None = None,
        confidence: ConfidenceConfig | None = None,
        controller: ControllerConfig | None = None,
    ) -> None:
        self.accumulator = accumulator or AccumulatorConfig()
        self.confidence = confidence or ConfidenceConfig()
        self.controller = controller or ControllerConfig()

        self._monitor = ConfidenceMonitor(self.confidence)
        self._controller = MetacognitiveController(
            self.controller,
            self.accumulator.base_threshold,
        )

    def run_trial(
        self,
        signal: float | Iterable[float],
        *,
        seed: int | None = None,
    ) -> TrialResult:
        rng = random.Random(seed)
        evidence = 0.0
        variance = self.confidence.variance_floor
        decision: int | None = None
        steps: list[StepSnapshot] = []
        stream = _signal_stream(signal)

        for step_index in range(1, self.accumulator.max_steps + 1):
            current_time = step_index * self.accumulator.dt

            pre_confidence = self._monitor.estimate(
                evidence,
                variance,
                elapsed_time=current_time - self.accumulator.dt,
            )
            _, attention_scale = self._controller.policy(pre_confidence)

            sensory_evidence = next(stream)
            drift = self.accumulator.base_drift * sensory_evidence * attention_scale
            noise = rng.gauss(0.0, self.accumulator.noise_std) * math.sqrt(self.accumulator.dt)
            increment = drift * self.accumulator.dt + noise

            evidence += increment
            variance = _update_variance(self.confidence, variance, increment)

            post_confidence = self._monitor.estimate(evidence, variance, elapsed_time=current_time)
            threshold, adjusted_attention = self._controller.policy(post_confidence)

            if abs(evidence) >= threshold:
                decision = 1 if evidence > 0 else -1

            steps.append(
                StepSnapshot(
                    step=step_index,
                    time=current_time,
                    evidence=evidence,
                    variance=variance,
                    confidence=post_confidence,
                    threshold=threshold,
                    attention_scale=adjusted_attention,
                    decision=decision,
                )
            )

            if decision is not None:
                break

        final_confidence = steps[-1].confidence if steps else 0.0
        return TrialResult(
            steps=steps,
            decision=decision,
            decided=decision is not None,
            final_confidence=final_confidence,
        )

    def compare_against_sequential_baseline(
        self,
        signal: float | Iterable[float],
        *,
        seed: int | None = None,
    ) -> TrialComparison:
        if isinstance(signal, (int, float)):
            online_signal: float | Iterable[float] = float(signal)
            sequential_signal: float | Iterable[float] = float(signal)
        else:
            online_signal, sequential_signal = tee(signal)

        baseline = SequentialConfidenceBaseline(
            accumulator=self.accumulator,
            confidence=self.confidence,
        )
        return TrialComparison(
            online=self.run_trial(online_signal, seed=seed),
            sequential=baseline.run_trial(sequential_signal, seed=seed),
        )


def evaluate_results(
    results: Iterable[TrialResult],
    labels: Iterable[int],
    *,
    bins: int = 10,
) -> EvaluationSummary:
    result_list = list(results)
    label_list = list(labels)

    if not result_list:
        raise ValueError("at least one trial result is required")
    if len(result_list) != len(label_list):
        raise ValueError("results and labels must have the same length")
    if bins < 1:
        raise ValueError("bins must be at least 1")

    total_trials = len(result_list)
    correct_count = 0.0
    decision_count = 0.0
    total_confidence = 0.0
    total_steps = 0.0
    total_elapsed_time = 0.0
    binned_confidence = [0.0] * bins
    binned_accuracy = [0.0] * bins
    binned_counts = [0] * bins

    for result, label in zip(result_list, label_list):
        target = 1 if label >= 0 else -1
        confidence = _clamp_unit_interval(result.final_confidence)
        steps = len(result.steps)
        elapsed_time = result.steps[-1].time if result.steps else 0.0
        correct = 1.0 if result.decision == target else 0.0

        correct_count += correct
        decision_count += 1.0 if result.decided else 0.0
        total_confidence += confidence
        total_steps += steps
        total_elapsed_time += elapsed_time

        bin_index = min(int(confidence * bins), bins - 1)
        binned_confidence[bin_index] += confidence
        binned_accuracy[bin_index] += correct
        binned_counts[bin_index] += 1

    expected_calibration_error = 0.0
    for index, count in enumerate(binned_counts):
        if count == 0:
            continue

        average_confidence = binned_confidence[index] / count
        average_accuracy = binned_accuracy[index] / count
        expected_calibration_error += (count / total_trials) * abs(
            average_accuracy - average_confidence
        )

    return EvaluationSummary(
        total_trials=total_trials,
        decision_rate=decision_count / total_trials,
        accuracy=correct_count / total_trials,
        mean_confidence=total_confidence / total_trials,
        mean_steps=total_steps / total_trials,
        mean_elapsed_time=total_elapsed_time / total_trials,
        expected_calibration_error=expected_calibration_error,
    )


def _signal_stream(signal: float | Iterable[float]) -> Iterator[float]:
    if isinstance(signal, (int, float)):
        constant = float(signal)
        while True:
            yield constant

    iterator = iter(signal)
    last_value = 0.0
    while True:
        try:
            last_value = float(next(iterator))
        except StopIteration:
            pass
        yield last_value


def _update_variance(
    config: ConfidenceConfig,
    previous_variance: float,
    increment: float,
) -> float:
    signal_energy = increment * increment + config.variance_floor
    retained = config.ema_decay * previous_variance
    injected = (1.0 - config.ema_decay) * signal_energy
    return max(config.variance_floor, retained + injected)


def _clamp_unit_interval(value: float) -> float:
    return min(1.0, max(0.0, value))
