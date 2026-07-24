#!/usr/bin/env python3

import json
import tempfile
import unittest
from pathlib import Path

from l1d2_core import CATEGORY_ORDER, Candidate, Tournament
from run_failure_attribution import (
    analyze,
    load_bootstrap_audit,
    pearson,
    spearman,
    terminal_neutralized,
)


def zero_model():
    hidden = 2
    return {
        "schema_version": 1,
        "vocabulary_size": 128,
        "hidden_size": hidden,
        "context_size": 8,
        "embeddings": [[0.0] * hidden for _ in range(128)],
        "recurrent_weights": [[0.0] * hidden for _ in range(hidden)],
        "context_weights": [[0.0] * hidden for _ in range(8)],
        "hidden_bias": [0.0] * hidden,
        "output_weights": [0.0] * hidden,
        "output_bias": 0.0,
    }


def synthetic_tournaments():
    result = []
    counter = 0
    for category in CATEGORY_ORDER:
        for group_index in range(6):
            counter += 1
            result.append(
                Tournament(
                    tournament_id=f"synthetic-{counter:02d}",
                    group_id=f"{category}-{group_index}",
                    category=category,
                    context=(1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000),
                    gold_candidate_id=1,
                    semantic_signature=f"meaning-{counter}",
                    candidates=(
                        Candidate(1, "Clear answer.", 1000, True, True, 0),
                        Candidate(2, "A longer clear answer!", 900, True, True, 0),
                        Candidate(3, "Clear answer", 800, True, True, 0),
                        Candidate(4, "The answer is clear?", 700, True, True, 0),
                    ),
                )
            )
    return result


class FailureAttributionTests(unittest.TestCase):
    def test_correlations_and_terminal_normalization(self):
        self.assertAlmostEqual(pearson([1, 2, 3], [2, 4, 6]), 1.0)
        self.assertAlmostEqual(spearman([3, 1, 2], [30, 10, 20]), 1.0)
        self.assertEqual(terminal_neutralized("Evidence?!  "), "Evidence")

    def test_bootstrap_target_audit_separates_surface_and_semantic_labels(self):
        records = [
            {
                "source_id": "surface",
                "failure_labels": ["hedging", "overexplained"],
            },
            {
                "source_id": "semantic",
                "failure_labels": ["wrong_confidence"],
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pairs.jsonl"
            path.write_text(
                "\n".join(json.dumps(record) for record in records) + "\n",
                encoding="utf-8",
            )
            audit = load_bootstrap_audit(path)
        self.assertEqual(audit["pairs"], 2)
        self.assertEqual(audit["semantic_or_authority_risk_pairs"], 1)
        self.assertEqual(audit["surface_only_pairs"], 1)
        self.assertFalse(audit["target_contract_matches_valid_surface_tournaments"])

    def test_full_five_seed_diagnostic_shape_is_deterministic(self):
        bootstrap = {
            "pairs": 16,
            "semantic_or_authority_risk_pairs": 10,
            "surface_only_pairs": 6,
            "target_contract_matches_valid_surface_tournaments": False,
        }
        seeds = (1729, 2718, 3141, 5772, 8119)
        first = analyze(zero_model(), synthetic_tournaments(), seeds, bootstrap)
        second = analyze(zero_model(), synthetic_tournaments(), seeds, bootstrap)
        self.assertEqual(first, second)
        self.assertEqual(first["observation_scope"]["test_observations"], 60)
        self.assertGreaterEqual(first["observation_scope"]["unique_tournaments"], 12)
        self.assertLessEqual(first["observation_scope"]["unique_tournaments"], 36)
        self.assertEqual(set(first["category_breakdown"]), set(CATEGORY_ORDER))
        self.assertEqual(first["headline"]["deterministic_correct_to_wrong"], 0)
        self.assertEqual(first["headline"]["deterministic_wrong_to_correct"], 0)
        self.assertFalse(first["authority"]["critic_promotion_allowed"])
        self.assertFalse(first["authority"]["training_performed"])


if __name__ == "__main__":
    unittest.main()
