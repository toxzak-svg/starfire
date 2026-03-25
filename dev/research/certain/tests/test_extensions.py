import unittest

from metacog_confidence import (
    ConfidenceAdaptiveDecoder,
    ControllerConfig,
    MetacognitiveLoop,
    SequentialConfidenceBaseline,
    StepSnapshot,
    TransformerConfidenceAdapter,
    TrialResult,
    evaluate_results,
    run_controller_sweep,
)


class SequentialBaselineTests(unittest.TestCase):
    def test_sequential_baseline_keeps_policy_fixed_until_post_hoc_confidence(self) -> None:
        baseline = SequentialConfidenceBaseline()

        result = baseline.run_trial(0.6, seed=7)

        self.assertTrue(result.steps)
        self.assertTrue(all(step.threshold == 1.0 for step in result.steps))
        self.assertTrue(all(step.attention_scale == 1.0 for step in result.steps))
        self.assertEqual(result.steps[-1].confidence, result.final_confidence)

    def test_loop_can_compare_online_and_sequential_runs(self) -> None:
        loop = MetacognitiveLoop(
            controller=ControllerConfig(
                low_confidence=0.6,
                high_confidence=0.9,
                threshold_gain=0.8,
                threshold_relaxation=0.2,
                attention_gain=1.2,
            )
        )

        comparison = loop.compare_against_sequential_baseline((0.25 for _ in range(20)), seed=3)

        self.assertEqual(comparison.sequential.steps[0].threshold, loop.accumulator.base_threshold)
        self.assertEqual(comparison.sequential.steps[0].attention_scale, 1.0)
        self.assertNotEqual(comparison.online.steps[0].threshold, comparison.sequential.steps[0].threshold)


class EvaluationTests(unittest.TestCase):
    def test_evaluate_results_reports_expected_calibration_error(self) -> None:
        positive = TrialResult(
            steps=[
                StepSnapshot(1, 0.1, 0.2, 0.01, 0.4, 1.0, 1.0, None),
                StepSnapshot(2, 0.2, 0.4, 0.01, 0.6, 1.0, 1.0, None),
                StepSnapshot(3, 0.3, 1.1, 0.02, 0.9, 1.0, 1.0, 1),
            ],
            decision=1,
            decided=True,
            final_confidence=0.9,
        )
        negative = TrialResult(
            steps=[StepSnapshot(1, 0.1, -0.3, 0.02, 0.2, 1.0, 1.0, -1)],
            decision=-1,
            decided=True,
            final_confidence=0.2,
        )

        summary = evaluate_results([positive, negative], [1, 1], bins=2)

        self.assertEqual(summary.total_trials, 2)
        self.assertAlmostEqual(summary.accuracy, 0.5)
        self.assertAlmostEqual(summary.mean_confidence, 0.55)
        self.assertAlmostEqual(summary.mean_steps, 2.0)
        self.assertAlmostEqual(summary.mean_elapsed_time, 0.2)
        self.assertAlmostEqual(summary.expected_calibration_error, 0.15)

    def test_controller_sweep_returns_side_by_side_metrics(self) -> None:
        records = run_controller_sweep(
            threshold_gains=[0.4],
            threshold_relaxations=[0.2],
            attention_gains=[1.0],
            signal_strengths=[0.4],
            repeats=2,
        )

        self.assertEqual(len(records), 1)
        row = records[0].to_row()
        self.assertIn("online_ece", row)
        self.assertIn("sequential_ece", row)
        self.assertIn("mean_steps_delta", row)

    def test_controller_sweep_accepts_signal_traces(self) -> None:
        records = run_controller_sweep(
            threshold_gains=[0.4],
            threshold_relaxations=[0.2],
            attention_gains=[1.0],
            signal_traces=[
                [0.1, 0.2, 0.7],
                [-0.1, -0.2, -0.8],
            ],
            repeats=1,
        )

        self.assertEqual(len(records), 1)
        self.assertGreater(records[0].online.mean_steps, 0.0)
        self.assertGreater(records[0].sequential.mean_steps, 0.0)


class TransformerAdapterTests(unittest.TestCase):
    def test_logits_adapter_returns_control_state(self) -> None:
        adapter = TransformerConfidenceAdapter()

        state = adapter.from_logits([3.0, 1.0, -0.5], elapsed_time=2.0)

        self.assertGreater(state.evidence, 0.0)
        self.assertGreaterEqual(state.variance, 0.0)
        self.assertTrue(0.0 <= state.confidence <= 1.0)
        self.assertGreater(state.threshold, 0.0)

    def test_hidden_state_projection_requires_matching_sizes(self) -> None:
        adapter = TransformerConfidenceAdapter()

        with self.assertRaises(ValueError):
            adapter.from_hidden_state([0.1, 0.2], projection=[1.0])

    def test_adaptive_decoder_stops_early_on_confident_logits_trace(self) -> None:
        decoder = ConfidenceAdaptiveDecoder(min_stable_steps=1)

        result = decoder.decode_logits_trace(
            [
                [0.2, 0.1, 0.0],
                [3.0, 0.0, -1.0],
                [4.0, 0.1, -2.0],
            ]
        )

        self.assertTrue(result.stopped_early)
        self.assertEqual(result.stop_reason, "confidence_ready")
        self.assertEqual(result.token_ids, [0, 0])
        self.assertGreater(result.mean_confidence, 0.0)

    def test_hidden_state_decoder_supports_teacher_forced_tokens(self) -> None:
        decoder = ConfidenceAdaptiveDecoder(min_stable_steps=2)

        result = decoder.decode_hidden_state_trace(
            [
                [0.1, 0.2, 0.1],
                [0.8, 0.9, 1.0],
                [1.1, 1.2, 1.3],
            ],
            token_ids=[10, 11, 12],
        )

        self.assertEqual(result.token_ids, [10, 11, 12][: len(result.steps)])
        self.assertIn(result.stop_reason, {"confidence_ready", "trace_exhausted"})


if __name__ == "__main__":
    unittest.main()
