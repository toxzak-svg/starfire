#!/usr/bin/env python3
"""Materialize digest-bound STLM L1-D1 model fixtures from gzip/base64 text."""

from __future__ import annotations

import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    return parser.parse_args()


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def materialize(entry: dict[str, Any], output_dir: Path) -> dict[str, Any]:
    encoded_path = Path(entry["encoded_path"])
    encoded = encoded_path.read_bytes()
    expected_encoded = entry.get("encoded_sha256")
    observed_encoded = sha256(encoded)
    if expected_encoded and observed_encoded != expected_encoded:
        raise ValueError(
            f"{encoded_path}: encoded SHA-256 {observed_encoded} != {expected_encoded}"
        )

    try:
        compressed = base64.b64decode(encoded, validate=True)
        raw = gzip.decompress(compressed)
    except Exception as error:
        raise ValueError(f"{encoded_path}: invalid gzip/base64 fixture: {error}") from error

    observed_raw = sha256(raw)
    expected_raw = entry["sha256"]
    if observed_raw != expected_raw:
        raise ValueError(
            f"{encoded_path}: raw SHA-256 {observed_raw} != {expected_raw}"
        )

    payload = json.loads(raw)
    required = {
        "schema_version",
        "vocabulary_size",
        "hidden_size",
        "context_size",
        "embeddings",
        "recurrent_weights",
        "context_weights",
        "hidden_bias",
        "output_weights",
        "output_bias",
    }
    missing = sorted(required - payload.keys())
    if missing:
        raise ValueError(f"{encoded_path}: model JSON missing {missing}")

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
