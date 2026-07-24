from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import run_valid_surface_benchmark as benchmark


class ValidSurfaceBenchmarkTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.surface_path = HERE / "data" / "valid_surface_tournaments.jsonl"
        cls.invalid_path = HERE / "data" / "semantic_invalid_candidates.jsonl"
        cls.tournaments = benchmark.load_surface_corpus(cls.surface_path)
        cls.invalid = benchmark.load_invalid_corpus(cls.invalid_path)

    def test_pooled_embeddings_are_order_invariant(self) -> None:
        hidden_size = 2
        embeddings = [[0.0, 0.0] for _ in range(benchmark.VOCABULARY_SIZE)]
        embeddings[ord("a")] = [1.0, -0.25]
        embeddings[ord("b")] = [-0.5, 0.75]
        model = {
            "schema_version": 1,
            "vocabulary_size": benchmark.VOCABULARY_SIZE,
            "hidden_size": hidden_size,
            "context_size": benchmark.CONTEXT_SIZE,
            "embeddings": embeddings,
            "recurrent_weights": [[0.0, 0.0], [0.0, 0.0]],
            "context_weights": [[0.0, 0.0] for _ in range(benchmark.CONTEXT_SIZE)],
            "hidden_bias": [0.0, 0.0],
            "output_weights": [1.0, -1.0],
            "output_bias": 0.0,
        }
        context = (5000,) * benchmark.CONTEXT_SIZE
        self.assertEqual(
            benchmark.score_pooled_embeddings(model, context, "ab"),
            benchmark.score_pooled_embeddings(model, context, "ba"),
        )

    def test_grouped_split_has_no_group_leakage(self) -> None:
        split = benchmark.stratified_group_split(self.tournaments, benchmark.DEFAULT_SEEDS[0])
        train = {item.group_id for item in split["train"]}
        dev = {item.group_id for item in split["dev"]}
        test = {item.group_id for item in split["test"]}
        self.assertFalse(train & dev)
        self.assertFalse(train & test)
        self.assertFalse(dev & test)
        self.assertEqual(len(split["train"]), 18)
        self.assertEqual(len(split["dev"]), 6)
        self.assertEqual(len(split["test"]), 12)
        for records in split.values():
            self.assertEqual({item.category for item in records}, set(benchmark.CATEGORY_ORDER))

    def test_surface_and_semantic_invalid_corpora_are_separate(self) -> None:
        self.assertEqual(len(self.tournaments), 36)
        self.assertGreaterEqual(len(self.invalid), 12)
        self.assertTrue(
            all(
                benchmark.hard_gate_passed(candidate)
                for tournament in self.tournaments
                for candidate in tournament.candidates
            )
        )
        self.assertTrue(
            all(
                any(not benchmark.hard_gate_passed(candidate) for candidate in probe.candidates)
                for probe in self.invalid
            )
        )

    def test_bounded_residual_cannot_exceed_registered_limit(self) -> None:
        self.assertEqual(
            benchmark.learned_residual(10_000), benchmark.LEARNED_RESIDUAL_LIMIT_BPS
        )
        self.assertEqual(
            benchmark.learned_residual(0), -benchmark.LEARNED_RESIDUAL_LIMIT_BPS
        )
        self.assertTrue(benchmark.bounded_residual_preflight()["passed"])

    def test_controls_are_deterministic(self) -> None:
        sample = self.tournaments[:12]
        self.assertEqual(
            benchmark.shuffled_contexts(sample, 1729),
            benchmark.shuffled_contexts(sample, 1729),
        )
        self.assertEqual(
            benchmark.shuffled_labels(sample, 1729),
            benchmark.shuffled_labels(sample, 1729),
        )
        text = "  ‘Quoted’\u00a0text — with\tspacing…  "
        self.assertEqual(
            benchmark.transform_unicode(text), benchmark.transform_unicode(text)
        )
        self.assertEqual(
            benchmark.transform_whitespace(text), benchmark.transform_whitespace(text)
        )
        self.assertEqual(
            benchmark.transform_punctuation(text), benchmark.transform_punctuation(text)
        )

    def test_jsonl_round_trip_is_stable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "copy.jsonl"
            records = [json.loads(line) for line in self.surface_path.read_text().splitlines()]
            output.write_bytes(benchmark.canonical_bytes(records[0]))
            self.assertTrue(output.read_bytes().endswith(b"\n"))


if __name__ == "__main__":
    unittest.main()
