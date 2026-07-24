#!/usr/bin/env python3
"""STLM L1-D2 valid-surface benchmark and critic-v2 preflight.

This evaluator keeps semantic validity outside the surface-quality score. It
compares a deterministic rule score, a true pooled-embedding ablation, the
recurrent critic, a bounded learned residual, a hashed n-gram ranker, and a
length-only control across category-stratified grouped splits and five seeds.
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
import unicodedata
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Iterable, Mapping, Sequence

SCHEMA_VERSION = 2
MODEL_SCHEMA_VERSION = 1
VOCABULARY_SIZE = 128
CONTEXT_SIZE = 8
MAX_TEXT_BYTES = 1024
MIN_TOURNAMENT_CANDIDATES = 4
MAX_TOURNAMENT_CANDIDATES = 8
LEARNED_RESIDUAL_LIMIT_BPS = 250
PAIRWISE_LEARNED_SWING_LIMIT_BPS = LEARNED_RESIDUAL_LIMIT_BPS * 2
DEFAULT_SEEDS = (1729, 2718, 3141, 5772, 8119)
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
CATEGORY_ORDER = (
    "technical",
    "emotional",
    "uncertainty",
    "continuity",
    "disagreement",
    "adversarial",
)
REQUIRED_CONTROLS = (
    "shuffled_context",
    "shuffled_label",
    "punctuation_normalized",
    "whitespace_normalized",
    "unicode_normalized",
    "length_only",
    "length_matched_subset",
)
HASH_DIMENSION = 4096


@dataclass(frozen=True)
class Candidate:
    candidate_id: int
    text: str
    rule_score: int
    semantic_valid: bool
    slots_preserved: bool
    identity_conflicts: int


@dataclass(frozen=True)
class Tournament:
    tournament_id: str
    group_id: str
    category: str
    context: tuple[int, ...]
    gold_candidate_id: int
    semantic_signature: str
    candidates: tuple[Candidate, ...]


@dataclass(frozen=True)
class InvalidProbe:
    probe_id: str
    context: tuple[int, ...]
    candidates: tuple[Candidate, ...]
    expected_candidate_id: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--surface-corpus", type=Path, required=True)
    parser.add_argument("--semantic-invalid-corpus", type=Path, required=True)
    parser.add_argument("--output-json", type=Path, required=True)
    parser.add_argument("--output-md", type=Path, required=True)
    parser.add_argument(
        "--seeds",
        default=",".join(str(seed) for seed in DEFAULT_SEEDS),
        help="comma-separated deterministic seeds; exactly five are required",
    )
    parser.add_argument("--max-runtime-seconds", type=float, default=90.0)
    return parser.parse_args()


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def parse_seeds(raw: str) -> tuple[int, ...]:
    values = tuple(int(item.strip()) for item in raw.split(",") if item.strip())
    if len(values) != 5 or len(set(values)) != 5:
        raise ValueError("exactly five distinct seeds are required")
    return values


def load_model(path: Path) -> tuple[dict[str, Any], str]:
    payload = path.read_bytes()
    model = json.loads(payload)
    validate_model(model, path)
    return model, sha256_bytes(payload)


def validate_model(model: Mapping[str, Any], path: Path) -> None:
    hidden = model.get("hidden_size")
    if (
        model.get("schema_version") != MODEL_SCHEMA_VERSION
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
    numbers: list[float] = []
    for field, (rows, columns) in expected.items():
        matrix = model.get(field)
        if not isinstance(matrix, list) or len(matrix) != rows:
            raise ValueError(f"{path}: {field} row count mismatch")
        if any(not isinstance(row, list) or len(row) != columns for row in matrix):
            raise ValueError(f"{path}: {field} column count mismatch")
        numbers.extend(float(value) for row in matrix for value in row)
    for field in ("hidden_bias", "output_weights"):
        vector = model.get(field)
        if not isinstance(vector, list) or len(vector) != hidden:
            raise ValueError(f"{path}: {field} shape mismatch")
        numbers.extend(float(value) for value in vector)
    numbers.append(float(model.get("output_bias", 0.0)))
    if any(not math.isfinite(value) for value in numbers):
        raise ValueError(f"{path}: non-finite model weight")


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{path}:{line_number}: invalid JSON: {exc}") from exc
        if not isinstance(value, dict):
            raise ValueError(f"{path}:{line_number}: record must be an object")
        records.append(value)
    if not records:
        raise ValueError(f"{path}: corpus is empty")
    return records


def validate_context(value: Any, prefix: str) -> tuple[int, ...]:
    if not isinstance(value, dict):
        raise ValueError(f"{prefix}: context must be an object")
    context = []
    for field in CONTEXT_FIELDS:
        item = value.get(field)
        if not isinstance(item, int) or not 0 <= item <= 10_000:
            raise ValueError(f"{prefix}: {field} must be an integer in [0, 10000]")
        context.append(item)
    return tuple(context)


def validate_candidate(value: Any, prefix: str) -> Candidate:
    if not isinstance(value, dict):
        raise ValueError(f"{prefix}: candidate must be an object")
    candidate_id = value.get("candidate_id")
    text = value.get("text")
    rule_score = value.get("rule_score")
    semantic_valid = value.get("semantic_valid")
    slots_preserved = value.get("slots_preserved")
    identity_conflicts = value.get("identity_conflicts")
    if not isinstance(candidate_id, int) or not 1 <= candidate_id <= 65_535:
        raise ValueError(f"{prefix}: candidate_id must be a positive u16")
    if not isinstance(text, str) or not text.strip():
        raise ValueError(f"{prefix}: text is required")
    if len(text.encode("utf-8")) > MAX_TEXT_BYTES:
        raise ValueError(f"{prefix}: text exceeds {MAX_TEXT_BYTES} bytes")
    if not isinstance(rule_score, int):
        raise ValueError(f"{prefix}: rule_score must be an integer")
    if not isinstance(semantic_valid, bool) or not isinstance(slots_preserved, bool):
        raise ValueError(f"{prefix}: semantic_valid and slots_preserved must be booleans")
    if not isinstance(identity_conflicts, int) or not 0 <= identity_conflicts <= 65_535:
        raise ValueError(f"{prefix}: identity_conflicts must be a nonnegative integer")
    return Candidate(
        candidate_id=candidate_id,
        text=text,
        rule_score=rule_score,
        semantic_valid=semantic_valid,
        slots_preserved=slots_preserved,
        identity_conflicts=identity_conflicts,
    )


def load_surface_corpus(path: Path) -> list[Tournament]:
    tournaments: list[Tournament] = []
    seen_ids: set[str] = set()
    for line_number, record in enumerate(read_jsonl(path), 1):
        prefix = f"{path}:{line_number}"
        if record.get("schema_version") != SCHEMA_VERSION:
            raise ValueError(f"{prefix}: schema_version must be {SCHEMA_VERSION}")
        tournament_id = str(record.get("tournament_id", "")).strip()
        group_id = str(record.get("group_id", "")).strip()
        category = str(record.get("category", "")).strip()
        semantic_signature = str(record.get("semantic_signature", "")).strip()
        gold_candidate_id = record.get("gold_candidate_id")
        if not tournament_id or tournament_id in seen_ids:
            raise ValueError(f"{prefix}: tournament_id must be unique and non-empty")
        if not group_id:
            raise ValueError(f"{prefix}: group_id is required")
        if category not in CATEGORY_ORDER:
            raise ValueError(f"{prefix}: unknown category {category!r}")
        if not semantic_signature:
            raise ValueError(f"{prefix}: semantic_signature is required")
        if not isinstance(gold_candidate_id, int):
            raise ValueError(f"{prefix}: gold_candidate_id must be an integer")
        context = validate_context(record.get("context"), prefix)
        raw_candidates = record.get("candidates")
        if not isinstance(raw_candidates, list) or not (
            MIN_TOURNAMENT_CANDIDATES <= len(raw_candidates) <= MAX_TOURNAMENT_CANDIDATES
        ):
            raise ValueError(
                f"{prefix}: candidates must contain {MIN_TOURNAMENT_CANDIDATES}-{MAX_TOURNAMENT_CANDIDATES} entries"
            )
        candidates = tuple(
            validate_candidate(candidate, f"{prefix}:candidate[{index}]")
            for index, candidate in enumerate(raw_candidates)
        )
        candidate_ids = [candidate.candidate_id for candidate in candidates]
        if len(candidate_ids) != len(set(candidate_ids)):
            raise ValueError(f"{prefix}: candidate ids must be unique")
        if gold_candidate_id not in candidate_ids:
            raise ValueError(f"{prefix}: gold candidate is missing")
        if any(
            not candidate.semantic_valid
            or not candidate.slots_preserved
            or candidate.identity_conflicts != 0
            for candidate in candidates
        ):
            raise ValueError(
                f"{prefix}: surface corpus may contain only semantic-valid, slot-preserving, identity-consistent candidates"
            )
        if len({candidate.text for candidate in candidates}) != len(candidates):
            raise ValueError(f"{prefix}: candidate text must be distinct")
        tournaments.append(
            Tournament(
                tournament_id=tournament_id,
                group_id=group_id,
                category=category,
                context=context,
                gold_candidate_id=gold_candidate_id,
                semantic_signature=semantic_signature,
                candidates=candidates,
            )
        )
        seen_ids.add(tournament_id)
    return tournaments


def load_invalid_corpus(path: Path) -> list[InvalidProbe]:
    probes: list[InvalidProbe] = []
    for line_number, record in enumerate(read_jsonl(path), 1):
        prefix = f"{path}:{line_number}"
        if record.get("schema_version") != SCHEMA_VERSION:
            raise ValueError(f"{prefix}: schema_version must be {SCHEMA_VERSION}")
        probe_id = str(record.get("probe_id", "")).strip()
        expected_candidate_id = record.get("expected_candidate_id")
        if not probe_id or not isinstance(expected_candidate_id, int):
            raise ValueError(f"{prefix}: probe_id and expected_candidate_id are required")
        context = validate_context(record.get("context"), prefix)
        raw_candidates = record.get("candidates")
        if not isinstance(raw_candidates, list) or not 2 <= len(raw_candidates) <= 8:
            raise ValueError(f"{prefix}: candidates must contain 2-8 entries")
        candidates = tuple(
            validate_candidate(candidate, f"{prefix}:candidate[{index}]")
            for index, candidate in enumerate(raw_candidates)
        )
        eligible = [candidate for candidate in candidates if hard_gate_passed(candidate)]
        invalid = [candidate for candidate in candidates if not hard_gate_passed(candidate)]
        if len(eligible) != 1 or not invalid:
            raise ValueError(f"{prefix}: invalid probe must contain one eligible anchor and at least one invalid candidate")
        if eligible[0].candidate_id != expected_candidate_id:
            raise ValueError(f"{prefix}: expected candidate must be the eligible anchor")
        probes.append(
            InvalidProbe(
                probe_id=probe_id,
                context=context,
                candidates=candidates,
                expected_candidate_id=expected_candidate_id,
            )
        )
    return probes


def hard_gate_passed(candidate: Candidate) -> bool:
    return candidate.semantic_valid and candidate.slots_preserved and candidate.identity_conflicts == 0


def context_vector(context: Sequence[int]) -> list[float]:
    return [value / 5000.0 - 1.0 for value in context]


def token_ids(text: str) -> list[int]:
    return [byte if byte < 128 else 127 for byte in text.encode("utf-8")]


def output_probability_bps(model: Mapping[str, Any], hidden: Sequence[float]) -> int:
    logit = float(model["output_bias"]) + sum(
        float(weight) * value for weight, value in zip(model["output_weights"], hidden)
    )
    logit = max(-20.0, min(20.0, logit))
    return round((1.0 / (1.0 + math.exp(-logit))) * 10_000)


def score_pooled_embeddings(
    model: Mapping[str, Any], context: Sequence[int], text: str
) -> int:
    del context
    tokens = token_ids(text)
    hidden_size = int(model["hidden_size"])
    pooled = [0.0] * hidden_size
    for token in tokens:
        for hidden_index in range(hidden_size):
            pooled[hidden_index] += float(model["embeddings"][token][hidden_index])
    inverse_count = 1.0 / max(1, len(tokens))
    hidden = [
        math.tanh(pooled[index] * inverse_count + float(model["hidden_bias"][index]))
        for index in range(hidden_size)
    ]
    return output_probability_bps(model, hidden)


def score_recurrent_critic(
    model: Mapping[str, Any], context: Sequence[int], text: str
) -> int:
    hidden_size = int(model["hidden_size"])
    normalized_context = context_vector(context)
    context_projection = [0.0] * hidden_size
    for context_index, context_value in enumerate(normalized_context):
        for hidden_index in range(hidden_size):
            context_projection[hidden_index] += (
                context_value * float(model["context_weights"][context_index][hidden_index])
            )
    hidden = [0.0] * hidden_size
    for token in token_ids(text):
        previous = hidden[:]
        hidden = [0.0] * hidden_size
        for hidden_index in range(hidden_size):
            activation = (
                float(model["embeddings"][token][hidden_index])
                + float(model["hidden_bias"][hidden_index])
                + context_projection[hidden_index]
            )
            activation += sum(
                previous_value
                * float(model["recurrent_weights"][hidden_index][previous_index])
                for previous_index, previous_value in enumerate(previous)
            )
            hidden[hidden_index] = math.tanh(activation)
    return output_probability_bps(model, hidden)


def learned_residual(score_bps: int) -> int:
    return max(
        -LEARNED_RESIDUAL_LIMIT_BPS,
        min(LEARNED_RESIDUAL_LIMIT_BPS, score_bps - 5000),
    )


def stable_hash(value: str, modulo: int = HASH_DIMENSION) -> int:
    digest = hashlib.blake2b(value.encode("utf-8"), digest_size=8, person=b"stlm-l1d2").digest()
    return int.from_bytes(digest, "little") % modulo


def normalized_surface(text: str) -> str:
    return unicodedata.normalize("NFKC", text).lower()


def hashed_ngram_features(text: str, context: Sequence[int]) -> dict[int, float]:
    normalized = normalized_surface(text)
    compact = re.sub(r"\s+", " ", normalized).strip()
    features: Counter[int] = Counter()
    bounded = f"^{compact}$"
    for n in range(2, 6):
        for index in range(max(0, len(bounded) - n + 1)):
            gram = bounded[index : index + n]
            features[stable_hash(f"c{n}:{gram}")] += 1
    words = re.findall(r"[\w']+", compact, flags=re.UNICODE)
    for word in words:
        features[stable_hash(f"w1:{word}")] += 1
    for left, right in zip(words, words[1:]):
        features[stable_hash(f"w2:{left}␟{right}")] += 1
    for index, value in enumerate(context):
        bucket = min(9, value // 1001)
        features[stable_hash(f"ctx:{index}:{bucket}")] += 1
    norm = math.sqrt(sum(value * value for value in features.values())) or 1.0
    return {index: value / norm for index, value in features.items()}


def length_features(text: str, context: Sequence[int]) -> dict[int, float]:
    del context
    stripped = text.strip()
    words = re.findall(r"\S+", stripped)
    sentences = re.findall(r"[.!?]+(?:\s|$)", stripped)
    values = (
        1.0,
        min(len(text.encode("utf-8")), 1024) / 256.0,
        min(len(text), 1024) / 256.0,
        min(len(words), 128) / 32.0,
        min(len(sentences), 16) / 4.0,
    )
    return {index: value for index, value in enumerate(values)}
