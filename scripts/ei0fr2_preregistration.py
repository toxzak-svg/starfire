#!/usr/bin/env python3
"""Verify EI-0F-R2 package, schema, runner diff, and classifier semantics."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

import ei0e_preregistration as base

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/experiments/EI_0F_R2_TERMINAL_PREREGISTRATION.json"
LOCK = ROOT / "docs/experiments/EI_0F_R2_FREEZE_LOCK.json"
SCHEMA = ROOT / "docs/experiments/EI_0F_R2_TERMINAL_REPORT.schema.json"
ORIGINAL_MANIFEST = ROOT / "docs/experiments/EI_0E_TERMINAL_PREREGISTRATION.json"
ORIGINAL_SCHEMA = ROOT / "docs/experiments/EI_0F_TERMINAL_REPORT.schema.json"
ORIGINAL_RUNNER = ROOT / "lib/examples/ei_0f_terminal_experiment.rs"
V2_RUNNER = ROOT / "lib/examples/ei_0f_r2_terminal_experiment.rs"
PREREGISTRATION_ID = "ei-0f-remediation-v2"
FREEZE_BASE = "f400a6a139e1a8820f80310b86b84d2b124de1fd"
ORIGINAL_MANIFEST_SHA256 = "5b83b27e5c218b6af2c53409d60fa6bf285adcde7ccb05b42505a5d0da290d73"
ORIGINAL_RESULT_COMMIT = "2e74746eeb524a50d41749d964e59838b7fbd919"
INVALID_V1_FREEZE_COMMIT = "f400a6a139e1a8820f80310b86b84d2b124de1fd"
PACKAGE_VERIFIER_ID = "ei-0fr2-package-verifier-v1"
SCHEMA_ID = "starfire/ei-0f-r2-terminal-report/v1"
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


def expected_schema() -> dict[str, Any]:
    schema = copy.deepcopy(load_canonical(ORIGINAL_SCHEMA))
    schema["$id"] = SCHEMA_ID
    schema["properties"]["preregistration_id"]["const"] = PREREGISTRATION_ID
    return schema


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


def validate_schema(instance: Any, schema: dict[str, Any], path: str = "$.") -> None:
    expected_type = schema.get("type")
    type_ok = {
        "object": isinstance(instance, dict),
        "array": isinstance(instance, list),
        "string": isinstance(instance, str),
        "integer": isinstance(instance, int) and not isinstance(instance, bool),
        "boolean": isinstance(instance, bool),
    }
    if expected_type:
        require(type_ok.get(expected_type, True), f"schema type mismatch at {path}")
    if "const" in schema:
        require(instance == schema["const"], f"schema const mismatch at {path}")
    if "enum" in schema:
        require(instance in schema["enum"], f"schema enum mismatch at {path}")
    if isinstance(instance, str):
        if "minLength" in schema:
            require(len(instance) >= schema["minLength"], f"schema minLength at {path}")
        if "pattern" in schema:
            require(re.fullmatch(schema["pattern"], instance) is not None, f"schema pattern at {path}")
    if isinstance(instance, int) and not isinstance(instance, bool):
        if "minimum" in schema:
            require(instance >= schema["minimum"], f"schema minimum at {path}")
        if "maximum" in schema:
            require(instance <= schema["maximum"], f"schema maximum at {path}")
    if isinstance(instance, list):
        if "minItems" in schema:
            require(len(instance) >= schema["minItems"], f"schema minItems at {path}")
        if "maxItems" in schema:
            require(len(instance) <= schema["maxItems"], f"schema maxItems at {path}")
        if "items" in schema:
            for index, value in enumerate(instance):
                validate_schema(value, schema["items"], f"{path}[{index}]")
    if isinstance(instance, dict):
        properties = schema.get("properties", {})
        for key in schema.get("required", []):
            require(key in instance, f"schema required field missing at {path}{key}")
        if schema.get("additionalProperties") is False:
            require(set(instance) <= set(properties), f"schema additional property at {path}")
        for key, value in instance.items():
            if key in properties:
                validate_schema(value, properties[key], f"{path}{key}.")


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
    require(manifest.get("stage") == "EI-0F-R2", "wrong stage")
    require(manifest.get("status") == "frozen_specification_no_results", "wrong status")
    require(manifest.get("freeze_base_commit") == FREEZE_BASE, "wrong freeze base")
    require(
        manifest.get("claim") == "ei-0f-r2-frozen-terminal-remediation-only",
        "claim boundary changed",
    )
    for key in SCIENTIFIC_KEYS:
        require(manifest.get(key) == original.get(key), f"scientific field changed: {key}")
    for key, value in original["implementation_ids"].items():
        require(
            manifest["implementation_ids"].get(key) == value,
            f"implementation ID changed: {key}",
        )
    require(
        manifest["implementation_ids"].get("package_verifier") == PACKAGE_VERIFIER_ID,
        "package verifier ID changed",
    )
    require(
        manifest.get("parent_result", {}).get("commit") == ORIGINAL_RESULT_COMMIT,
        "original FAIL binding changed",
    )
    invalid = manifest.get("invalid_parent_freeze", {})
    require(invalid.get("commit") == INVALID_V1_FREEZE_COMMIT, "V1 freeze binding changed")
    require(invalid.get("executed") is False, "invalid V1 freeze was executed")
    require(
        invalid.get("reason") == "report_schema_preregistration_const_mismatch",
        "invalid V1 reason changed",
    )
    remediation = manifest.get("remediation", {})
    require(
        remediation.get("sole_additional_change")
        == "report_schema_id_and_preregistration_const",
        "R2 remediation scope changed",
    )
    require(remediation.get("validator_changed") is False, "validator changed")
    require(remediation.get("thresholds_changed") is False, "thresholds changed")
    require(
        manifest.get("terminal", {}).get("report_schema")
        == "docs/experiments/EI_0F_R2_TERMINAL_REPORT.schema.json",
        "manifest does not bind V2 schema",
    )

    actual_schema = load_canonical(SCHEMA)
    require(actual_schema == expected_schema(), "V2 schema has extra changes")

    for entry in manifest.get("source_files", []):
        path = ROOT / entry["path"]
        require(path.is_file(), f"source file missing: {entry['path']}")
        require(
            blob_sha1(path.read_bytes()) == entry.get("git_blob_sha1"),
            f"source mismatch: {entry['path']}",
        )

    runner = V2_RUNNER.read_bytes()
    require(
        runner == expected_runner(manifest_digest),
        "V2 runner has changes beyond frozen substitutions",
    )

    lock = load_canonical(LOCK)
    require(lock.get("schema_version") == 1, "wrong lock schema")
    require(lock.get("preregistration_id") == PREREGISTRATION_ID, "wrong lock ID")
    require(lock.get("manifest_sha256") == manifest_digest, "lock manifest mismatch")
    require(lock.get("schema_git_blob_sha1") == blob_sha1(SCHEMA.read_bytes()), "schema binding changed")
    require(lock.get("runner_git_blob_sha1") == blob_sha1(runner), "runner binding changed")
    files = lock.get("files", [])
    paths = [entry.get("path") for entry in files]
    require(paths == sorted(paths), "lock paths not sorted")
    require(len(paths) == len(set(paths)), "duplicate lock paths")
    require("docs/experiments/EI_0F_R2_FREEZE_LOCK.json" not in paths, "lock binds itself")
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
        "runner_git_blob_sha1": blob_sha1(runner),
        "schema_git_blob_sha1": blob_sha1(SCHEMA.read_bytes()),
        "verified": True,
    }


def self_test() -> dict[str, Any]:
    manifest = load_canonical(MANIFEST)
    schema = load_canonical(SCHEMA)
    result = base.self_test(manifest)
    require(result.get("self_test") == "pass", "base classifier self-test failed")
    pass_report = base.synthetic_pass_report(manifest)
    validate_schema(pass_report, schema)
    require(base.classify(pass_report, manifest).get("classification") == "PASS", "PASS vector failed")
    fail_report = copy.deepcopy(pass_report)
    fail_report["source_match"] = False
    validate_schema(fail_report, schema)
    require(base.classify(fail_report, manifest).get("classification") == "FAIL", "FAIL vector failed")
    return {
        "classifier_id": base.CLASSIFIER_ID,
        "preregistration_id": PREREGISTRATION_ID,
        "schema_id": SCHEMA_ID,
        "self_test": "pass",
        "vectors": {"pass": "PASS", "source_mismatch": "FAIL"},
    }


def classify_report(path: Path) -> tuple[dict[str, Any], int]:
    manifest = load_canonical(MANIFEST)
    schema = load_canonical(SCHEMA)
    report = base.load_json(path)
    validate_schema(report, schema)
    result = base.classify(report, manifest)
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
            require(bool(args.report), "classify requires report path")
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
