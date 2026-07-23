#!/usr/bin/env python3
"""Train and export the STLM L1-D recurrent phrase critic.

The script is intentionally repository- and Kaggle-friendly. It consumes JSONL
preference pairs, trains a tiny deterministic tanh RNN with pairwise ranking
loss, and exports the exact JSON tensor layout consumed by lib/phrase_critic.rs.

The exported model ranks wording only. It is not a semantic verifier and must
remain behind the hard semantic, slot-preservation, and identity-conflict gates.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

try:
    import torch
    import torch.nn.functional as F
except ImportError as exc:  # pragma: no cover - exercised in Kaggle/training jobs
    raise SystemExit(
        "PyTorch is required for training. Kaggle notebooks include it; locally install torch first."
    ) from exc

SCHEMA_VERSION = 1
VOCABULARY_SIZE = 128
CONTEXT_SIZE = 8
MAX_TEXT_BYTES = 1024
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


@dataclass(frozen=True)
class PreferencePair:
    source_id: str
    context: tuple[int, ...]
    preferred: str
    rejected: str
    failure_labels: tuple[str, ...]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--input",
        type=Path,
        default=Path("tools/stlm_l1d/data/bootstrap_pairs.jsonl"),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("tools/stlm_l1d/out/phrase_critic_v1.json"),
    )
    parser.add_argument("--epochs", type=int, default=160)
    parser.add_argument("--hidden-size", type=int, default=24)
    parser.add_argument("--learning-rate", type=float, default=0.01)
    parser.add_argument("--seed", type=int, default=1729)
    return parser.parse_args()


def load_pairs(path: Path) -> list[PreferencePair]:
    pairs: list[PreferencePair] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError as exc:
                raise ValueError(f"{path}:{line_number}: invalid JSON: {exc}") from exc
            pairs.append(validate_record(record, path, line_number))
    if len(pairs) < 8:
        raise ValueError("at least eight preference pairs are required")
    return pairs


def validate_record(record: dict[str, Any], path: Path, line_number: int) -> PreferencePair:
    source_id = str(record.get("source_id", "")).strip()
    context_object = record.get("context")
    preferred = str(record.get("preferred", "")).strip()
    rejected = str(record.get("rejected", "")).strip()
    labels = record.get("failure_labels", [])
    if not source_id:
        raise ValueError(f"{path}:{line_number}: source_id is required")
    if not isinstance(context_object, dict):
        raise ValueError(f"{path}:{line_number}: context must be an object")
    context: list[int] = []
    for field in CONTEXT_FIELDS:
        value = context_object.get(field)
        if not isinstance(value, int) or not 0 <= value <= 10_000:
            raise ValueError(f"{path}:{line_number}: {field} must be an integer in [0, 10000]")
        context.append(value)
    if not preferred or not rejected or preferred == rejected:
        raise ValueError(f"{path}:{line_number}: preferred/rejected text must be distinct")
    if len(preferred.encode("utf-8")) > MAX_TEXT_BYTES or len(rejected.encode("utf-8")) > MAX_TEXT_BYTES:
        raise ValueError(f"{path}:{line_number}: text exceeds {MAX_TEXT_BYTES} bytes")
    if not isinstance(labels, list) or any(not isinstance(label, str) for label in labels):
        raise ValueError(f"{path}:{line_number}: failure_labels must be strings")
    return PreferencePair(
        source_id=source_id,
        context=tuple(context),
        preferred=preferred,
        rejected=rejected,
        failure_labels=tuple(sorted(set(labels))),
    )


def encode_text(text: str) -> torch.Tensor:
    tokens = [byte if byte < 128 else 127 for byte in text.encode("utf-8")]
    return torch.tensor(tokens, dtype=torch.long)


class PhraseCriticRnn(torch.nn.Module):
    """Exact training counterpart of the Rust tanh-RNN inference path."""

    def __init__(self, hidden_size: int) -> None:
        super().__init__()
        if not 1 <= hidden_size <= 128:
            raise ValueError("hidden_size must be in [1, 128]")
        self.hidden_size = hidden_size
        self.embeddings = torch.nn.Parameter(torch.empty(VOCABULARY_SIZE, hidden_size))
        self.recurrent_weights = torch.nn.Parameter(torch.empty(hidden_size, hidden_size))
        self.context_weights = torch.nn.Parameter(torch.empty(CONTEXT_SIZE, hidden_size))
        self.hidden_bias = torch.nn.Parameter(torch.zeros(hidden_size))
        self.output_weights = torch.nn.Parameter(torch.empty(hidden_size))
        self.output_bias = torch.nn.Parameter(torch.zeros(()))
        torch.nn.init.normal_(self.embeddings, mean=0.0, std=0.04)
        torch.nn.init.orthogonal_(self.recurrent_weights, gain=0.65)
        torch.nn.init.normal_(self.context_weights, mean=0.0, std=0.05)
        torch.nn.init.normal_(self.output_weights, mean=0.0, std=0.05)

    def forward(self, context_bps: torch.Tensor, tokens: torch.Tensor) -> torch.Tensor:
        normalized_context = context_bps.to(torch.float32) / 5000.0 - 1.0
        context_projection = normalized_context @ self.context_weights
        hidden = torch.zeros(self.hidden_size, dtype=torch.float32)
        for token in tokens:
            hidden = torch.tanh(
                self.embeddings[token]
                + self.recurrent_weights @ hidden
                + context_projection
                + self.hidden_bias
            )
        return self.output_weights @ hidden + self.output_bias


def iter_pairs_deterministically(
    pairs: list[PreferencePair], epoch: int, seed: int
) -> Iterable[PreferencePair]:
    indices = list(range(len(pairs)))
    random.Random(seed + epoch).shuffle(indices)
    for index in indices:
        yield pairs[index]


def train(
    pairs: list[PreferencePair],
    hidden_size: int,
    epochs: int,
    learning_rate: float,
    seed: int,
) -> PhraseCriticRnn:
    random.seed(seed)
    torch.manual_seed(seed)
    torch.use_deterministic_algorithms(True)
    model = PhraseCriticRnn(hidden_size)
    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)

    for epoch in range(epochs):
        epoch_loss = 0.0
        correct = 0
        for pair in iter_pairs_deterministically(pairs, epoch, seed):
            context = torch.tensor(pair.context, dtype=torch.float32)
            preferred_logit = model(context, encode_text(pair.preferred))
            rejected_logit = model(context, encode_text(pair.rejected))
            margin = preferred_logit - rejected_logit
            loss = F.softplus(-margin)
            optimizer.zero_grad(set_to_none=True)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), max_norm=2.0)
            optimizer.step()
            epoch_loss += float(loss.detach())
            correct += int(float(margin.detach()) > 0.0)
        if epoch in {0, epochs - 1} or (epoch + 1) % 25 == 0:
            accuracy = correct / len(pairs)
            print(
                json.dumps(
                    {
                        "epoch": epoch + 1,
                        "mean_pairwise_loss": epoch_loss / len(pairs),
                        "training_pair_accuracy": accuracy,
                    },
                    sort_keys=True,
                )
            )
    return model


def export_model(model: PhraseCriticRnn, output: Path) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "vocabulary_size": VOCABULARY_SIZE,
        "hidden_size": model.hidden_size,
        "context_size": CONTEXT_SIZE,
        "embeddings": model.embeddings.detach().cpu().tolist(),
        "recurrent_weights": model.recurrent_weights.detach().cpu().tolist(),
        "context_weights": model.context_weights.detach().cpu().tolist(),
        "hidden_bias": model.hidden_bias.detach().cpu().tolist(),
        "output_weights": model.output_weights.detach().cpu().tolist(),
        "output_bias": float(model.output_bias.detach().cpu()),
    }
    encoded = (json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(encoded)
    digest = hashlib.sha256(encoded).hexdigest()
    output.with_suffix(output.suffix + ".sha256").write_text(
        f"{digest}  {output.name}\n", encoding="utf-8"
    )
    return digest


def main() -> None:
    args = parse_args()
    if args.epochs <= 0 or args.learning_rate <= 0:
        raise SystemExit("epochs and learning-rate must be positive")
    pairs = load_pairs(args.input)
    model = train(
        pairs=pairs,
        hidden_size=args.hidden_size,
        epochs=args.epochs,
        learning_rate=args.learning_rate,
        seed=args.seed,
    )
    digest = export_model(model, args.output)
    print(
        json.dumps(
            {
                "model": str(args.output),
                "sha256": digest,
                "pairs": len(pairs),
                "hidden_size": args.hidden_size,
                "seed": args.seed,
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
