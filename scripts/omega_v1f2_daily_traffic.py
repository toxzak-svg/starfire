#!/usr/bin/env python3
"""Send one controlled ΩV1-F2 operational batch to the live /chat endpoint.

The server ledger stores no prompt or response text. This client prints only
status, byte length, and a SHA-256 prefix for the returned response body.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ELIGIBLE_PROMPTS = [
    "How are you right now?",
    "Are you sure you are functioning?",
    "What are you thinking about?",
    "What's been on your mind?",
    "What have you been researching?",
    "What are you curious about?",
    "What do you wonder about today?",
    "Do you love me?",
    "I love you.",
    "Who are you?",
    "Tell me about yourself.",
    "Can you look up information?",
    "Can you read files?",
    "Tell me a story.",
    "Are you conscious?",
    "Do you understand what you are?",
    "What do you know about Starfire?",
    "Learning means changing from evidence.",
    "I want you to grow.",
    "I want you to expand yourself carefully.",
]

INELIGIBLE_PROMPTS = [
    "The sky is overcast today.",
    "This is a plain statement.",
    "Bananas are yellow.",
    "I am testing an unclassified sentence.",
    "A quiet line with no request.",
    "Operational sample marker.",
    "No special intent here.",
    "Just recording an ordinary statement.",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--url",
        default="https://starfire-cuee.onrender.com",
        help="Starfire service root URL",
    )
    parser.add_argument("--eligible", type=int, default=30)
    parser.add_argument("--ineligible", type=int, default=8)
    parser.add_argument("--delay", type=float, default=0.75)
    parser.add_argument(
        "--state-file",
        type=Path,
        default=Path(".omega_v1f2_traffic_state.json"),
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="allow another batch on the same UTC day",
    )
    return parser.parse_args()


def load_state(path: Path) -> dict[str, object]:
    if not path.exists():
        return {"completed_utc_days": []}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"cannot read state file {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise RuntimeError("traffic state must be a JSON object")
    return value


def save_state(path: Path, state: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True), encoding="utf-8")
    temporary.replace(path)


def post_chat(endpoint: str, prompt: str, timeout: float = 120.0) -> tuple[int, bytes]:
    payload = json.dumps({"message": prompt}).encode("utf-8")
    request = urllib.request.Request(
        endpoint,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read()


def expanded(prompts: list[str], count: int) -> list[str]:
    if count < 0:
        raise ValueError("request counts cannot be negative")
    return [prompts[index % len(prompts)] for index in range(count)]


def main() -> int:
    args = parse_args()
    today = datetime.now(timezone.utc).date().isoformat()
    state = load_state(args.state_file)
    completed = state.setdefault("completed_utc_days", [])
    if not isinstance(completed, list):
        raise RuntimeError("completed_utc_days must be a list")
    if today in completed and not args.force:
        print(f"UTC day {today} already has a completed batch; use --force to repeat.")
        return 2

    endpoint = args.url.rstrip("/") + "/chat"
    schedule = [
        *(('eligible', prompt) for prompt in expanded(ELIGIBLE_PROMPTS, args.eligible)),
        *(('ineligible', prompt) for prompt in expanded(INELIGIBLE_PROMPTS, args.ineligible)),
    ]
    successes = 0
    failures = 0
    print(
        f"Starting ΩV1-F2 batch for UTC {today}: "
        f"{args.eligible} eligible + {args.ineligible} ineligible requests"
    )

    for index, (expected_class, prompt) in enumerate(schedule, start=1):
        status, body = post_chat(endpoint, prompt)
        digest = hashlib.sha256(body).hexdigest()[:16]
        ok = status == 200
        successes += int(ok)
        failures += int(not ok)
        print(
            f"{index:03d}/{len(schedule):03d} "
            f"expected={expected_class:<10} status={status} "
            f"bytes={len(body)} sha256={digest}"
        )
        if index != len(schedule):
            time.sleep(max(args.delay, 0.0))

    if failures:
        print(f"Batch failed: {failures} HTTP requests were non-200.", file=sys.stderr)
        return 1

    completed.append(today)
    state["completed_utc_days"] = sorted(set(str(day) for day in completed))
    state["last_batch"] = {
        "utc_day": today,
        "eligible_requests": args.eligible,
        "ineligible_requests": args.ineligible,
        "successful_requests": successes,
    }
    save_state(args.state_file, state)
    print(f"Completed ΩV1-F2 batch for UTC {today}: {successes} successful requests.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (RuntimeError, ValueError, urllib.error.URLError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
