#!/usr/bin/env python3
"""Verify the frozen EI-0F-R1B package and reuse EI-0E classifier semantics."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

import ei0e_preregistration as base

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/experiments/EI_0F_R1_TERMINAL_PREREGISTRATION.json"
LOCK = ROOT / "docs/experiments/EI_0F_R1_FREEZE_LOCK.json"
ORIGINAL_MANIFEST = ROOT / "docs/experiments/EI_0E_TERMINAL_PREREGISTRATION.json"
ORIGINAL_RUNNER = ROOT / "lib/examples/ei_0f_terminal_experiment.rs"
REPAIRED_RUNNER = ROOT / "lib/examples/ei_0f_r1_terminal_experiment.rs"
PREREGISTRATION_ID = "ei-0f-remediation-v1"
FREEZE_BASE = "b9b5f70d7da98088d2546c0ac5a730c9854326ab"
ORIGINAL_MANIFEST_SHA256 = "5b83b27e5c218b6af2c53409d60fa6bf285adcde7ccb05b42505a5d0da290d73"
PACKAGE_VERIFIER_ID = "ei-0fr1-package-verifier-v1"
PARENT_RESULT_COMMIT = "2e74746eeb524a50d41749d964e59838b7fbd919"
SCIENTIFIC_KEYS = (
    "aggregation",
    "arms",
    "authority",
    "budgets",
    "failure_rules",
    "hypotheses",
    "initial_policy",
    "partitions",
    "schema_versions",
    "task_families",
    "thresholds",
    "update_lattice",
)


class VerificationError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def canonical_bytes(value: Any) -> bytes:
    return base.canonical_bytes(value)


def load_canonical(path: Path) -> Any:
    raw = path.read_bytes()
    value = json.loads(raw)
    require(raw == canonical_bytes(value), f"noncanonical JSON: {path}")
    return value


def blob_sha1(data: bytes) -> str:
    return base.git_blob_sha1(data)


def expected_runner(manifest_digest: str) -> bytes:
    text = ORIGINAL_RUNNER.read_text(encoding="utf-8")
    substitutions = (
        (
            'const PREREGISTRATION_ID: &str = "ei-0e-terminal-v1";',
            f'const PREREGISTRATION_ID: &str = "{PREREGISTRATION_ID}";',
        ),
        (ORIGINAL_MANIFEST_SHA256, manifest_digest),
        (
            '            proposal_digest: format!("preregistered:{update_id}"),',
            '            proposal_digest: format!("preregistered:{}", update_id.replace(\'_\', "-")),',
        ),
    )
    for old, new in substitutions:
        require(text.count(old) == 1, f"original runner binding changed: {old[:40]}")
        text = text.replace(old, new, 1)
    return text.encode("utf-8")


def verify_repository() -> dict[str, Any]:
    original = load_canonical(ORIGINAL_MANIFEST)
    require(
        hashlib.sha256(canonical_bytes(original)).hexdigest()
        == ORIGINAL_MANIFEST_SHA256,
        "original manifest changed",
    )
    manifest = load_canonical(MANIFEST)
    manifest_digest = hashlib.sha256(canonical_bytes(manifest)).hexdigest()

    require(manifest.get("preregistration_id") == PREREGISTRATION_ID, "wrong ID")
    require(manifest.get("stage") == "EI-0F-R1B", "wrong stage")
    require(
        manifest.get("status") == "frozen_specification_no_results",
        "package is not frozen",
    )
    require(manifest.get("freeze_base_commit") == FREEZE_BASE, "wrong freeze base")
    require(
        manifest.get("claim") == "ei-0f-r1-frozen-terminal-remediation-only",
        "claim boundary changed",
    )
    for key in SCIENTIFIC_KEYS:
        require(
            manifest.get(key) == original.get(key),
            f"scientific field changed: {key}",
        )
    for key, value in original["implementation_ids"].items():
        require(
            manifest["implementation_ids"].get(key) == value,
            f"implementation ID changed: {key}",
        )
    require(
        manifest["implementation_ids"].get("package_verifier")
        == PACKAGE_VERIFIER_ID,
        "package verifier ID changed",
    )
    require(
        manifest.get("parent_result")
        == {
            "classification": "FAIL",
            "commit": PARENT_RESULT_COMMIT,
            "preregistration_id": "ei-0e-terminal-v1",
            "preserved_unchanged": True,
        },
        "parent FAIL changed",
    )
    remediation = manifest.get("remediation", {})
    require(
        remediation.get("sole_change")
        == "episode_proposal_digest_underscore_to_hyphen",
        "remediation scope changed",
    )
    require(remediation.get("validator_changed") is False, "validator changed")
    require(
        remediation.get("update_identifiers_changed") is False,
        "update identifiers changed",
    )

    for entry in manifest.get("source_files", []):
        path = ROOT / entry["path"]
        require(path.is_file(), f"source file missing: {entry['path']}")
        require(
            blob_sha1(path.read_bytes()) == entry.get("git_blob_sha1"),
            f"source mismatch: {entry['path']}",
        )

    repaired = REPAIRED_RUNNER.read_bytes()
    require(
        repaired == expected_runner(manifest_digest),
        "repaired runner has changes beyond the frozen substitutions",
    )

    lock = load_canonical(LOCK)
    require(lock.get("schema_version") == 1, "wrong lock schema")
    require(lock.get("preregistration_id") == PREREGISTRATION_ID, "wrong lock ID")
    require(lock.get("manifest_sha256") == manifest_digest, "lock manifest mismatch")
    require(
        lock.get("original_runner_git_blob_sha1")
        == blob_sha1(ORIGINAL_RUNNER.read_bytes()),
        "original runner binding changed",
    )
    require(
        lock.get("repaired_runner_git_blob_sha1") == blob_sha1(repaired),
        "repaired runner binding changed",
    )
    files = lock.get("files", [])
    paths = [entry.get("path") for entry in files]
    require(paths == sorted(paths), "lock paths are not sorted")
    require(len(paths) == len(set(paths)), "duplicate lock paths")
    require(
        "docs/experiments/EI_0F_R1_FREEZE_LOCK.json" not in paths,
        "lock cannot bind itself",
    )
    for entry in files:
        path = ROOT / entry["path"]
        require(path.is_file(), f"locked file missing: {entry['path']}")
        require(
            blob_sha1(path.read_bytes()) == entry.get("git_blob_sha1"),
            f"locked file mismatch: {entry['path']}",
        )

    subprocess.run(
        ["git", "merge-base", "--is-ancestor", FREEZE_BASE, "HEAD"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    return {
        "classifier_id": base.CLASSIFIER_ID,
        "freeze_base_commit": FREEZE_BASE,
        "locked_files": len(files),
        "manifest_sha256": manifest_digest,
        "package_verifier_id": PACKAGE_VERIFIER_ID,
        "preregistration_id": PREREGISTRATION_ID,
        "runner_git_blob_sha1": blob_sha1(repaired),
        "verified": True,
    }


def self_test() -> dict[str, Any]:
    manifest = load_canonical(MANIFEST)
    result = base.self_test(manifest)
    require(result.get("self_test") == "pass", "base classifier self-test failed")
    report = base.synthetic_pass_report(manifest)
    require(
        base.classify(report, manifest).get("classification") == "PASS",
        "remediation PASS vector failed",
    )
    return {
        "classifier_id": base.CLASSIFIER_ID,
        "preregistration_id": PREREGISTRATION_ID,
        "self_test": "pass",
        "vectors": result.get("vectors"),
    }


def classify_report(path: Path) -> tuple[dict[str, Any], int]:
    manifest = load_canonical(MANIFEST)
    result = base.classify(base.load_json(path), manifest)
    return result, 0 if result.get("classification") == "PASS" else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["verify", "self-test", "classify"])
    parser.add_argument("report", nargs="?")
    args = parser.parse_args()
    try:
        if args.command == "verify":
            result, status = verify_repository(), 0
        elif args.command == "self-test":
            result, status = self_test(), 0
        else:
            require(bool(args.report), "classify requires a report path")
            result, status = classify_report(ROOT / args.report)
    except (
        OSError,
        KeyError,
        TypeError,
        VerificationError,
        base.VerificationError,
        subprocess.CalledProcessError,
    ) as exc:
        result, status = (
            {
                "classification": "FAIL",
                "error": str(exc),
                "package_verifier_id": PACKAGE_VERIFIER_ID,
                "preregistration_id": PREREGISTRATION_ID,
            },
            1,
        )
    sys.stdout.buffer.write(canonical_bytes(result))
    return status


if __name__ == "__main__":
    raise SystemExit(main())
