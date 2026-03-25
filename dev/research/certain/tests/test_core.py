import unittest

from metacog_confidence import (
    AccumulatorConfig,
    ConfidenceConfig,
    ControllerConfig,
    MetacognitiveLoop,
)


class MetacognitiveLoopTests(unittest.TestCase):
    def test_positive_signal_reaches_positive_decision(self) -> None:
        loop = MetacognitiveLoop(
            accumulator=AccumulatorConfig(
                dt=0.1,
                base_drift=1.2,
                noise_std=0.0,
                base_threshold=1.0,
                max_steps=100,
            ),
            confidence=ConfidenceConfig(
                variance_floor=0.01,
                meta_uncertainty=0.05,
                slope=2.0,
                ema_decay=0.8,
            ),
            controller=ControllerConfig(
                low_confidence=0.4,
                high_confidence=0.85,
                threshold_gain=0.4,
                threshold_relaxation=0.2,
                attention_gain=1.5,
            ),
        )

        result = loop.run_trial(0.8)

        self.assertTrue(result.decided)
        self.assertEqual(result.decision, 1)
        self.assertGreater(result.final_confidence, 0.5)

    def test_low_confidence_increases_threshold_and_attention(self) -> None:
        loop = MetacognitiveLoop(
            accumulator=AccumulatorConfig(
                dt=0.1,
                base_drift=0.8,
                noise_std=0.0,
                base_threshold=1.0,
                max_steps=6,
            ),
            confidence=ConfidenceConfig(
                variance_floor=0.01,
                meta_uncertainty=0.05,
                slope=1.8,
                ema_decay=0.9,
            ),
            controller=ControllerConfig(
                low_confidence=0.55,
                high_confidence=0.9,
                threshold_gain=0.8,
                threshold_relaxation=0.2,
                attention_gain=2.0,
            ),
        )

        result = loop.run_trial(0.2)

        self.assertGreater(result.steps[0].threshold, loop.accumulator.base_threshold)
        self.assertGreater(result.steps[0].attention_scale, 1.0)
        self.assertTrue(all(0.0 <= step.confidence <= 1.0 for step in result.steps))


if __name__ == "__main__":
    unittest.main()
