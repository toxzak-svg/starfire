from __future__ import annotations

from collections.abc import Iterable, Sequence
from dataclasses import dataclass

from .core import (
    ConfidenceConfig,
    ConfidenceMonitor,
    ControllerConfig,
    MetacognitiveController,
)


@dataclass(frozen=True)
class TransformerControlState:
    evidence: float
    variance: float
    confidence: float
    threshold: float
    attention_scale: float
    decision_ready: bool


@dataclass(frozen=True)
class DecodeStep:
    index: int
    token_id: int | None
    control: TransformerControlState


@dataclass(frozen=True)
class AdaptiveDecodeResult:
    steps: list[DecodeStep]
    token_ids: list[int | None]
    stopped_early: bool
    stop_reason: str
    mean_confidence: float
    mean_attention_scale: float


class TransformerConfidenceAdapter:
    def __init__(
        self,
        *,
        confidence: ConfidenceConfig | None = None,
        controller: ControllerConfig | None = None,
        base_threshold: float = 1.0,
    ) -> None:
        self.confidence = confidence or ConfidenceConfig()
        self.controller = controller or ControllerConfig()
        self.base_threshold = base_threshold
        self._monitor = ConfidenceMonitor(self.confidence)
        self._controller = MetacognitiveController(self.controller, self.base_threshold)

    def from_logits(
        self,
        logits: Sequence[float],
        *,
        elapsed_time: float = 0.0,
    ) -> TransformerControlState:
        values = self._coerce_values(logits, name="logits", minimum_length=2)
        top_score, runner_up = sorted(values, reverse=True)[:2]
        evidence = top_score - runner_up
        variance = self._sample_variance(values)
        return self._build_control_state(evidence, variance, elapsed_time)

    def from_hidden_state(
        self,
        hidden_state: Sequence[float],
        *,
        projection: Sequence[float] | None = None,
        elapsed_time: float = 0.0,
    ) -> TransformerControlState:
        values = self._coerce_values(
            hidden_state,
            name="hidden_state",
            minimum_length=1,
        )
        if projection is None:
            evidence = sum(values) / len(values)
        else:
            weights = self._coerce_values(
                projection,
                name="projection",
                minimum_length=len(values),
            )
            if len(weights) != len(values):
                raise ValueError("projection and hidden_state must have the same length")
            evidence = sum(value * weight for value, weight in zip(values, weights)) / len(values)

        variance = self._sample_variance(values)
        return self._build_control_state(evidence, variance, elapsed_time)

    def _build_control_state(
        self,
        evidence: float,
        variance: float,
        elapsed_time: float,
    ) -> TransformerControlState:
        confidence = self._monitor.estimate(
            evidence,
            variance,
            elapsed_time=max(0.0, elapsed_time),
        )
        threshold, attention_scale = self._controller.policy(confidence)
        return TransformerControlState(
            evidence=evidence,
            variance=variance,
            confidence=confidence,
            threshold=threshold,
            attention_scale=attention_scale,
            decision_ready=abs(evidence) >= threshold,
        )

    def _coerce_values(
        self,
        values: Sequence[float],
        *,
        name: str,
        minimum_length: int,
    ) -> list[float]:
        coerced = [float(value) for value in values]
        if len(coerced) < minimum_length:
            raise ValueError(f"{name} must contain at least {minimum_length} value(s)")
        return coerced

    def _sample_variance(self, values: Sequence[float]) -> float:
        mean_value = sum(values) / len(values)
        return sum((value - mean_value) ** 2 for value in values) / len(values)


class ConfidenceAdaptiveDecoder:
    def __init__(
        self,
        adapter: TransformerConfidenceAdapter | None = None,
        *,
        confidence: ConfidenceConfig | None = None,
        controller: ControllerConfig | None = None,
        base_threshold: float = 1.0,
        min_stable_steps: int = 1,
    ) -> None:
        if min_stable_steps < 1:
            raise ValueError("min_stable_steps must be at least 1")

        self.adapter = adapter or TransformerConfidenceAdapter(
            confidence=confidence,
            controller=controller,
            base_threshold=base_threshold,
        )
        self.min_stable_steps = min_stable_steps

    def decode_logits_trace(
        self,
        logits_trace: Iterable[Sequence[float]],
    ) -> AdaptiveDecodeResult:
        trace = [
            self.adapter._coerce_values(step, name="logits", minimum_length=2)
            for step in logits_trace
        ]
        if not trace:
            raise ValueError("logits_trace must contain at least one step")

        stable_steps = 0
        steps: list[DecodeStep] = []
        for index, logits in enumerate(trace):
            control = self.adapter.from_logits(logits, elapsed_time=float(index + 1))
            token_id = max(range(len(logits)), key=lambda token_index: logits[token_index])
            steps.append(
                DecodeStep(
                    index=index,
                    token_id=token_id,
                    control=control,
                )
            )

            stable_steps = stable_steps + 1 if control.decision_ready else 0
            if stable_steps >= self.min_stable_steps:
                return self._finalize_result(
                    steps,
                    stopped_early=index < len(trace) - 1,
                    stop_reason="confidence_ready",
                )

        return self._finalize_result(
            steps,
            stopped_early=False,
            stop_reason="trace_exhausted",
        )

    def decode_hidden_state_trace(
        self,
        hidden_state_trace: Iterable[Sequence[float]],
        *,
        projection: Sequence[float] | None = None,
        token_ids: Sequence[int] | None = None,
    ) -> AdaptiveDecodeResult:
        trace = [
            self.adapter._coerce_values(step, name="hidden_state", minimum_length=1)
            for step in hidden_state_trace
        ]
        if not trace:
            raise ValueError("hidden_state_trace must contain at least one step")
        if token_ids is not None and len(token_ids) < len(trace):
            raise ValueError("token_ids must be at least as long as hidden_state_trace")

        stable_steps = 0
        steps: list[DecodeStep] = []
        for index, hidden_state in enumerate(trace):
            control = self.adapter.from_hidden_state(
                hidden_state,
                projection=projection,
                elapsed_time=float(index + 1),
            )
            token_id = None if token_ids is None else int(token_ids[index])
            steps.append(
                DecodeStep(
                    index=index,
                    token_id=token_id,
                    control=control,
                )
            )

            stable_steps = stable_steps + 1 if control.decision_ready else 0
            if stable_steps >= self.min_stable_steps:
                return self._finalize_result(
                    steps,
                    stopped_early=index < len(trace) - 1,
                    stop_reason="confidence_ready",
                )

        return self._finalize_result(
            steps,
            stopped_early=False,
            stop_reason="trace_exhausted",
        )

    def _finalize_result(
        self,
        steps: list[DecodeStep],
        *,
        stopped_early: bool,
        stop_reason: str,
    ) -> AdaptiveDecodeResult:
        total_steps = len(steps)
        mean_confidence = sum(step.control.confidence for step in steps) / total_steps
        mean_attention_scale = (
            sum(step.control.attention_scale for step in steps) / total_steps
        )
        return AdaptiveDecodeResult(
            steps=steps,
            token_ids=[step.token_id for step in steps],
            stopped_early=stopped_early,
            stop_reason=stop_reason,
            mean_confidence=mean_confidence,
            mean_attention_scale=mean_attention_scale,
        )
