#!/usr/bin/env python3
"""Materialize digest-bound STLM L1-D1 models from compact q16 fixtures."""

from __future__ import annotations

import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
from typing import Any

MATRIX_TENSORS = ("embeddings", "recurrent_weights", "context_weights")
VECTOR_TENSORS = ("hidden_bias", "output_weights")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    return parser.parse_args()


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def expand_quantized(payload: dict[str, Any]) -> dict[str, Any]:
    if payload.get("fixture_schema_version") != 1:
        raise ValueError("unsupported compact fixture schema")
    if payload.get("quantization") != "symmetric_i16_per_tensor":
        raise ValueError("unsupported quantization scheme")

    model = dict(payload["model_header"])
    shapes = payload["shapes"]
    scales = payload["scales"]
    tensors = payload["tensors"]

    for name in MATRIX_TENSORS:
        rows, columns = shapes[name]
        integers = tensors[name]
        if len(integers) != rows * columns:
            raise ValueError(f"{name}: compact tensor shape mismatch")
        values = [int(value) * float(scales[name]) for value in integers]
        model[name] = [
            values[index * columns : (index + 1) * columns] for index in range(rows)
        ]

    for name in VECTOR_TENSORS:
        expected = shapes[name][0]
        integers = tensors[name]
        if len(integers) != expected:
            raise ValueError(f"{name}: compact tensor shape mismatch")
        model[name] = [int(value) * float(scales[name]) for value in integers]

    if shapes["output_bias"] != []:
        raise ValueError("output_bias: compact tensor shape mismatch")
    model["output_bias"] = int(tensors["output_bias"]) * float(scales["output_bias"])
    return model


def materialize(entry: dict[str, Any], output_dir: Path) -> dict[str, Any]:
    encoded_path = Path(entry["encoded_path"])
    encoded = encoded_path.read_bytes()
    observed_encoded = sha256(encoded)
    if observed_encoded != entry["encoded_sha256"]:
        raise ValueError(
            f"{encoded_path}: encoded SHA-256 {observed_encoded} "
            f"!= {entry['encoded_sha256']}"
        )

    try:
        canonical_base64 = b"".join(encoded.split())
        compact = json.loads(
            gzip.decompress(base64.b64decode(canonical_base64, validate=True))
        )
    except Exception as error:
        raise ValueError(f"{encoded_path}: invalid compact fixture: {error}") from error

    model = expand_quantized(compact)
    raw = (json.dumps(model, sort_keys=True, separators=(",", ":")) + "\n").encode()
    observed_raw = sha256(raw)
    if observed_raw != entry["sha256"]:
        raise ValueError(
            f"{entry['materialized_filename']}: raw SHA-256 {observed_raw} "
            f"!= {entry['sha256']}"
        )

    output_dir.mkdir(parents=True, exist_ok=True)
    output_path = output_dir / entry["materialized_filename"]
    output_path.write_bytes(raw)
    return {
        "encoded_path": str(encoded_path),
        "encoded_sha256": observed_encoded,
        "output_path": str(output_path),
        "raw_sha256": observed_raw,
        "raw_bytes": len(raw),
    }


def main() -> None:
    args = parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    results = {
        name: materialize(entry, args.output_dir)
        for name, entry in sorted(manifest["models"].items())
    }
    print(json.dumps({"models": results}, sort_keys=True))


if __name__ == "__main__":
    main()
