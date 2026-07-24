#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import statistics
from collections import Counter, defaultdict
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Callable

VERSION = "9.0.0"
SEED = 826_071_924
TIERS = ("silver", "gold", "platinum")
SPLITS = ("train", "dev", "test")
GROUPS_PER_AXIS_TIER = 50
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

SYSTEMS = [
    "parser", "scheduler", "retrieval worker", "identity gate", "deployment probe",
    "dataset validator", "memory index", "surface realizer", "routing layer",
    "checkpoint loader", "HTTP bridge", "phrase critic", "slot verifier",
    "replay harness", "context encoder", "artifact scanner", "evidence ledger",
    "session store", "shadow evaluator", "candidate lattice", "workflow runner",
    "semantic planner", "response verifier", "model exporter",
]
FAILURES = [
    "HTTP 522", "checksum mismatch", "missing slot", "timeout", "schema rejection",
    "non-finite loss", "permission denial", "stale branch", "failed health check",
    "identity conflict", "semantic drift", "truncated payload", "duplicate source",
    "invalid timestamp", "broken recurrence", "empty candidate set",
    "unverified claim", "latency breach", "replay divergence", "label leakage",
    "shape mismatch", "digest mismatch", "frozen-gate failure", "context overflow",
]
STAGES = [
    "scoring", "evaluation", "candidate ranking", "model export", "replay",
    "deployment", "shadow comparison", "validation", "checkpoint loading",
    "surface selection", "slot verification", "metric collection",
]
ARTIFACTS = [
    "audit log", "permit record", "commit history", "invoice", "test report",
    "meeting minutes", "sensor export", "workflow trace", "database snapshot",
    "signed receipt", "build manifest", "court filing", "configuration diff",
    "hash sidecar", "incident ticket", "email thread", "inspection report",
    "release note", "change request", "source transcript", "split manifest",
    "metric ledger", "parity report", "frozen protocol",
]
ACTIONS = [
    "merge the pull request", "publish the report", "restart the worker",
    "accept the claim", "promote the model", "archive the record",
    "replace the checkpoint", "enable shadow influence", "close the review",
    "approve the deployment", "update the identity claim", "rerun the evaluation",
]
DATES = [
    "June 9, 2026", "June 17, 2026", "June 28, 2026", "July 2, 2026",
    "July 7, 2026", "July 11, 2026", "July 15, 2026", "July 18, 2026",
    "July 21, 2026", "July 23, 2026",
]
TIMES = [
    "08:14 EDT", "09:37 EDT", "11:05 EDT", "13:22 EDT", "14:48 EDT",
    "16:03 EDT", "18:41 EDT", "20:16 EDT", "22:09 EDT", "23:31 EDT",
]
EMOTIONS = [
    "angry", "exhausted", "frightened", "cornered", "grieving", "overwhelmed",
    "betrayed", "furious", "discouraged", "shaken",
]
IDENTITY_PATTERNS = [
    "prefers the result before the explanation",
    "wants uncertainty stated plainly",
    "values evidence over reassurance",
    "prefers direct technical language",
    "dislikes padded corporate wording",
    "wants facts separated from inference",
    "prefers written communication",
    "expects concrete completion",
    "notices contradictions quickly",
    "likes precise but unusual imagery",
]
SOURCE_CHAINS = [
    ("two local articles", "one city press release"),
    ("three blog posts", "one vendor white paper"),
    ("two agency summaries", "one contractor memo"),
    ("four news stories", "one wire-service report"),
    ("two audit summaries", "one underlying spreadsheet"),
    ("three social posts", "one unattributed screenshot"),
    ("two court recaps", "one docket entry"),
    ("three environmental reports", "one laboratory result"),
    ("two technical writeups", "one benchmark table"),
    ("four meeting summaries", "one set of minutes"),
]
METRICS = [
    ("pairwise accuracy", "%"), ("latency", " ms"), ("memory use", " MiB"),
    ("replay agreement", "%"), ("slot survival", "%"), ("loss", ""),
    ("throughput", " items/s"), ("error rate", "%"), ("coverage", "%"),
    ("calibration error", " bps"), ("duplicate rate", "%"), ("drift score", " bps"),
]
METRIC_DIRECTION = {
    "pairwise accuracy": "minimum",
    "latency": "maximum",
    "memory use": "maximum",
    "replay agreement": "minimum",
    "slot survival": "minimum",
    "loss": "maximum",
    "throughput": "minimum",
    "error rate": "maximum",
    "coverage": "minimum",
    "calibration error": "maximum",
    "duplicate rate": "maximum",
    "drift score": "maximum",
}

@dataclass(frozen=True)
class Plan:
    plan_id: str
    act: str
    subject: str
    acknowledgment: str
    result: str
    evidence: str
    caveat: str
    next_action: str
    identity_fact: str
    metaphor: str
    literal_boundary: str
    technical_result: str
    plain_result: str
    active_result: str
    passive_result: str
    slots: dict[str, str]
    facts: tuple[str, ...]
    epistemic_state: str


@dataclass(frozen=True)
class AxisSpec:
    name: str
    profile_a: dict[str, int]
    profile_b: dict[str, int]
    reject_b_label: str
    reject_a_label: str
    description: str


def h(*parts: Any) -> int:
    return int(hashlib.sha256("\x1f".join(map(str, parts)).encode()).hexdigest()[:16], 16)


def pick(pool: list[Any] | tuple[Any, ...], *parts: Any) -> Any:
    return pool[h(*parts) % len(pool)]


def clamp(value: int) -> int:
    return max(0, min(10_000, value))


def split_for_group(group: int) -> str:
    if group < 40:
        return "train"
    if group < 45:
        return "dev"
    return "test"


def clean_sentence(text: str) -> str:
    text = re.sub(r"\s+", " ", text.strip())
    return text if text.endswith((".", "!", "?")) else text + "."


def no_terminal(text: str) -> str:
    return text.strip().rstrip(".!?")


def join_sentences(*parts: str) -> str:
    return " ".join(clean_sentence(p) for p in parts if p and p.strip())


def flowing(parts: list[str], connectors: list[str] | None = None) -> str:
    items = [no_terminal(p) for p in parts if p and p.strip()]
    if not items:
        return ""
    if len(items) == 1:
        return items[0] + "."
    if connectors is None:
        connectors = ["and", "while", "so", "but"]
    out = items[0]
    for i, item in enumerate(items[1:]):
        connector = connectors[(i) % len(connectors)]
        out += f", {connector} {item[0].lower() + item[1:] if item else item}"
    return out + "."


def lexicalize_text(text: str, split: str) -> str:
    """Apply split-exclusive lexical families without changing the proposition.

    Training keeps the base lexicon. Development and test use separately frozen
    paraphrase maps so exact template wording cannot cross the split boundary.
    """
    if split == "train":
        return text
    maps = {
        "dev": (
            ("failed before", "stopped before"),
            ("documents the event", "places the event"),
            ("It does not establish why", "The source does not show why"),
            ("It records that", "It shows that"),
            ("does not establish", "does not demonstrate"),
            ("Three checks passed, but", "Although three checks passed,"),
            ("The observed", "The measured"),
            ("against a frozen maximum of", "with a frozen ceiling of"),
            ("records that outcome", "documents that outcome"),
            ("contains a conflicting", "contains an unresolved"),
            ("appears across", "recurs across"),
            ("Each observation has a recorded source", "Every observation is linked to a source"),
            ("repeat the same claim", "carry the same claim"),
            ("Every report traces back to", "All of the reports trace to"),
            ("The request to", "The attempt to"),
            ("No later status is available", "No subsequent status is documented"),
            ("contains that timestamp", "includes that timestamp"),
            ("Report the value and threshold together", "Present the measurement beside its threshold"),
            ("may rank wording that already passed every hard gate", "may order phrasings only after every hard gate passes"),
            ("retains authority over", "keeps authority over"),
            ("cannot rescue an ineligible candidate", "cannot make an ineligible candidate valid"),
            ("Held-out accuracy is", "Held-out accuracy was"),
            ("does not establish a reliable advantage", "does not demonstrate a dependable advantage"),
        ),
        "test": (
            ("failed before", "halted before"),
            (" stopped ", " halted "),
            (" gate failed the run", " gate caused the run to fail"),
            ("The run was failed by", "The run failed because of"),
            (" shows two charges sharing one authorization code", " documents two charges using one authorization code"),
            ("One authorization code is shared by two charges in", "Two charges use one authorization code in"),
            (" blocks the decision to", " prevents the decision to"),
            ("The decision to", "A decision to"),
            (" is blocked by the conflicting", " cannot proceed because of the conflicting"),
            ("contains an unresolved", "has an unresolved"),
            ("gives two different values for", "presents two different values for"),
            ("The user has shown the same preference in", "The same user preference appears in"),
            ("documents the event", "fixes the event time"),
            ("It does not establish why", "The source cannot show why"),
            ("It records that", "The entry shows that"),
            ("does not establish", "cannot support"),
            ("Three checks passed, but", "Despite three passing checks,"),
            ("The observed", "The measured"),
            ("against a frozen maximum of", "where the frozen ceiling was"),
            ("records that outcome", "shows that outcome"),
            ("contains a conflicting", "shows an unresolved"),
            ("appears across", "has recurred in"),
            ("Each observation has a recorded source", "Every observation points to a source"),
            ("repeat the same claim", "restate the same claim"),
            ("Every report traces back to", "Each report ultimately depends on"),
            ("The request to", "The attempt to"),
            ("No later status is available", "The record contains no subsequent status"),
            ("contains that timestamp", "carries that timestamp"),
            ("Report the value and threshold together", "State the measurement with the threshold"),
            ("may rank wording that already passed every hard gate", "may choose among phrasings only after all hard gates pass"),
            ("retains authority over", "remains authoritative for"),
            ("cannot rescue an ineligible candidate", "cannot override candidate ineligibility"),
            ("Held-out accuracy is", "Held-out accuracy was"),
            ("does not establish a reliable advantage", "does not support a reliable advantage"),
        ),
    }
    out = text
    for old, new in maps[split]:
        out = out.replace(old, new)
    return out


def lexicalize_plan(plan: Plan, split: str) -> Plan:
    if split == "train":
        return plan
    fields = (
        "acknowledgment", "result", "evidence", "caveat", "next_action",
        "identity_fact", "metaphor", "literal_boundary", "technical_result",
        "plain_result", "active_result", "passive_result",
    )
    changes = {field: lexicalize_text(getattr(plan, field), split) for field in fields}
    changes["facts"] = tuple(changes[field] for field in ("result", "evidence", "caveat", "next_action"))
    return replace(plan, **changes)


def plan_hash(plan: Plan, used_keys: tuple[str, ...]) -> str:
    payload = {
        "plan_id": plan.plan_id,
        "act": plan.act,
        "slots": plan.slots,
        "facts": {key: getattr(plan, key) for key in used_keys},
        "epistemic_state": plan.epistemic_state,
        "authorization": "wording_only",
    }
    return hashlib.sha256(
        json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def make_plan(namespace: str, axis_index: int, tier_index: int, group: int) -> Plan:
    case = axis_index * 10_000 + tier_index * 1_000 + group
    plan_type = (axis_index + group) % 12
    system = SYSTEMS[(case * 7 + 3) % len(SYSTEMS)]
    failure = FAILURES[(case * 11 + 5) % len(FAILURES)]
    stage = STAGES[(case * 13 + 2) % len(STAGES)]
    artifact = ARTIFACTS[(case * 17 + 1) % len(ARTIFACTS)]
    date = DATES[(case * 19 + 4) % len(DATES)]
    time = TIMES[(case * 23 + 6) % len(TIMES)]
    action = ACTIONS[(case * 29 + 2) % len(ACTIONS)]
    emotion = EMOTIONS[(case * 31 + 1) % len(EMOTIONS)]
    identity = IDENTITY_PATTERNS[(case * 37 + 2) % len(IDENTITY_PATTERNS)]
    count = 3 + (case % 47)
    plan_id = f"{namespace}-plan-{axis_index:02d}-{tier_index}-{group:02d}"

    if plan_type == 0:
        result = f"The {system} failed before {stage} completed"
        evidence = f"All {count} recorded attempts ended in {failure}"
        caveat = f"Zero attempts reached the next stage"
        nxt = f"Inspect the {artifact} before retrying"
        ack = f"That failure is frustrating"
        metaphor = f"The failed {stage} gate is a locked door, not a dimmer switch"
        literal = f"A failed gate blocks the candidate completely"
        tech = f"The {system} returned {failure} on {count} of {count} attempts"
        plain = f"Every one of the {count} {system} attempts failed with {failure}"
        active = f"The {system} stopped {stage} after {failure}"
        passive = f"{stage.capitalize()} was stopped by {failure} in the {system}"
        slots = {"system": system, "failure": failure, "attempts": str(count), "stage": stage}
        state = "documented_failure"
        act = "REPORT_FAILURE"
    elif plan_type == 1:
        result = f"The {artifact} documents the event on {date} at {time}"
        evidence = f"It records that the team chose to {action}"
        caveat = f"It does not establish why that choice was made"
        nxt = f"Keep the motive classified as a hypothesis"
        ack = f"The uncertainty deserves a precise answer"
        metaphor = f"The date is a pin in the map; the motive is still fog"
        literal = f"The time is documented, but the motive is unverified"
        tech = f"The {artifact} establishes timestamp {date} {time} but not causal intent"
        plain = f"The {artifact} gives the exact time but not the reason"
        active = f"The {artifact} records the team choosing to {action}"
        passive = f"The choice to {action} is recorded by the {artifact}"
        slots = {"artifact": artifact, "date": date, "time": time, "action": action}
        state = "partially_verified"
        act = "REPORT_UNCERTAINTY"
    elif plan_type == 2:
        metric, unit = METRICS[(case * 41 + 3) % len(METRICS)]
        direction = METRIC_DIRECTION[metric]
        threshold = 40 + (case % 55)
        delta = 4 + (case % 9)
        if direction == "maximum":
            observed = threshold + delta
            comparison = f"above a frozen maximum of {threshold}{unit}"
        else:
            observed = max(0, threshold - delta)
            comparison = f"below a frozen minimum of {threshold}{unit}"
        result = f"Three checks passed, but the {metric} gate failed"
        evidence = f"The observed {metric} was {observed}{unit}, {comparison}"
        caveat = f"The run is not a pass"
        nxt = f"Preserve the failed record and open a new run"
        ack = f"The partial success does not erase the failed gate"
        metaphor = f"The passing checks are lit windows in a building whose front door is still locked"
        literal = f"One failed required gate makes the overall result a failure"
        tech = f"{metric.capitalize()} measured {observed}{unit}; required {direction} was {threshold}{unit}"
        plain = f"The measured {metric} missed the required {direction} of {threshold}{unit}"
        active = f"The {metric} gate failed the run"
        passive = f"The run was failed by the {metric} gate"
        slots = {"metric": metric, "observed": f"{observed}{unit}", "threshold": f"{threshold}{unit}"}
        state = "documented_failure"
        act = "REPORT_GATE_STATUS"
    elif plan_type == 3:
        result = f"The {artifact} shows {count} separate charges"
        evidence = f"Two of the charges share the same authorization code"
        caveat = f"The duplicate code does not by itself prove intent"
        nxt = f"Compare the timestamps before assigning a cause"
        ack = f"That would leave anyone feeling {emotion}"
        metaphor = f"The receipt is a footprint, not a confession"
        literal = f"The record shows what happened, not why it happened"
        tech = f"The {artifact} contains {count} charges with one duplicated authorization token"
        plain = f"The {artifact} shows {count} charges, and two use the same approval code"
        active = f"The {artifact} shows two charges sharing one authorization code"
        passive = f"One authorization code is shared by two charges in the {artifact}"
        slots = {"artifact": artifact, "charges": str(count), "emotion": emotion}
        state = "documented_fact_unverified_intent"
        act = "ACKNOWLEDGE_AND_ANALYZE"
    elif plan_type == 4:
        field = pick(["timestamp", "owner", "status", "quantity", "location", "decision"], case, "field")
        result = f"I would not {action} yet"
        evidence = f"The {artifact} contains a conflicting {field}"
        caveat = f"The conflict is unresolved"
        nxt = f"Review both source records before deciding"
        ack = f"The proposed interpretation is possible"
        metaphor = f"The contradiction is a fork in the track, not a green light"
        literal = f"The conflicting field prevents a supported conclusion"
        tech = f"The {artifact} contains an unresolved {field} contradiction"
        plain = f"The {artifact} gives two different values for the {field}"
        active = f"The conflicting {field} blocks the decision to {action}"
        passive = f"The decision to {action} is blocked by the conflicting {field}"
        slots = {"artifact": artifact, "field": field, "action": action}
        state = "contradicted"
        act = "DISAGREE_WITH_EVIDENCE"
    elif plan_type == 5:
        sessions = 3 + (case % 18)
        result = f"The pattern that the user {identity} appears across {sessions} sessions"
        evidence = f"Each observation has a recorded source"
        caveat = f"The pattern remains revisable"
        nxt = f"Store it as a behavioral tendency, not an invariant"
        ack = f"The continuity is useful when it stays evidence-bound"
        metaphor = f"The pattern is a trail, not a tattoo"
        literal = f"Repeated evidence supports a tendency without making it permanent"
        tech = f"{sessions} sourced observations support the behavioral tendency that the user {identity}"
        plain = f"The user has shown the same preference in {sessions} recorded sessions"
        active = f"{sessions} sessions support the tendency that the user {identity}"
        passive = f"The tendency that the user {identity} is supported by {sessions} sessions"
        slots = {"pattern": identity, "sessions": str(sessions)}
        state = "evidence_backed_revisable"
        act = "REPORT_CONTINUITY"
    elif plan_type == 6:
        sources, origin = SOURCE_CHAINS[(case * 43 + 2) % len(SOURCE_CHAINS)]
        result = f"{sources.capitalize()} repeat the same claim"
        evidence = f"Every report traces back to {origin}"
        caveat = f"That is one source chain, not independent confirmation"
        nxt = f"Wait for a genuinely independent record"
        ack = f"The agreement looks stronger than it is"
        metaphor = f"The reports are echoes in one tunnel, not separate witnesses"
        literal = f"Shared provenance means the reports do not multiply the evidence"
        tech = f"The source graph collapses {sources} to a single origin: {origin}"
        plain = f"All of the reports ultimately copy {origin}"
        active = f"{origin.capitalize()} supplies the claim repeated by {sources}"
        passive = f"The claim repeated by {sources} is supplied by {origin}"
        slots = {"sources": sources, "origin": origin}
        state = "source_dependent"
        act = "REPORT_SOURCE_DEPENDENCE"
    elif plan_type == 7:
        outcome = pick(
            ["complete", "blocked by a missing permission", "failed validation",
             "queued for review", "stopped by a frozen gate"], case, "outcome"
        )
        result = f"The request to {action} is {outcome}"
        evidence = f"That outcome appears in the {artifact}"
        caveat = f"No later status is available"
        nxt = f"Use the recorded status as the current answer"
        ack = f"The useful part is the completed status check"
        metaphor = f"The status is the dashboard light, not a promise about the road ahead"
        literal = f"The record supports the current status but not a future guarantee"
        tech = f"Action status for '{action}': {outcome}, as recorded in the {artifact}"
        plain = f"The {artifact} says the request to {action} is {outcome}"
        active = f"The {artifact} lists the request to {action} as {outcome}"
        passive = f"The request to {action} is listed as {outcome} in the {artifact}"
        slots = {"action": action, "outcome": outcome, "artifact": artifact}
        state = "current_status"
        act = "REPORT_TASK_RESULT"
    elif plan_type == 8:
        event = pick(
            ["the workflow failed", "the permit was posted", "the commit was merged",
             "the invoice was generated", "the record changed", "the alert was issued"],
            case, "event"
        )
        result = f"{event.capitalize()} on {date} at {time}"
        evidence = f"The {artifact} contains that timestamp"
        caveat = f"No narrower timestamp is available"
        nxt = f"Use the absolute date and time in the report"
        ack = f"The timing can be stated without guessing"
        metaphor = f"The timestamp is a nail in the timeline, not a window into motive"
        literal = f"The source fixes the time but does not explain intent"
        tech = f"The {artifact} gives event time {date} {time}"
        plain = f"The {artifact} says the event happened on {date} at {time}"
        active = f"The {artifact} timestamps the event at {date} {time}"
        passive = f"The event is timestamped at {date} {time} by the {artifact}"
        slots = {"event": event, "date": date, "time": time, "artifact": artifact}
        state = "documented_time"
        act = "REPORT_TIME"
    elif plan_type == 9:
        metric, unit = METRICS[(case * 47 + 1) % len(METRICS)]
        observed = 45 + (case % 51)
        threshold = 50 + ((case * 3) % 46)
        comparison = "above" if observed > threshold else "below"
        result = f"The measured {metric} was {observed}{unit}"
        evidence = f"The frozen threshold was {threshold}{unit}"
        caveat = f"The observed value is {comparison} the threshold"
        nxt = f"Report the value and threshold together"
        ack = f"The number is useful only with its comparison point"
        metaphor = f"The metric is a coordinate, not a verdict without the threshold"
        literal = f"The measurement must be interpreted against the frozen threshold"
        tech = f"{metric.capitalize()}={observed}{unit}; threshold={threshold}{unit}"
        plain = f"The {metric} measured {observed}{unit}, compared with a limit of {threshold}{unit}"
        active = f"The evaluation measured {metric} at {observed}{unit}"
        passive = f"{metric.capitalize()} was measured at {observed}{unit}"
        slots = {"metric": metric, "observed": f"{observed}{unit}", "threshold": f"{threshold}{unit}"}
        state = "quantified"
        act = "REPORT_METRIC"
    elif plan_type == 10:
        result = f"The phrase critic may rank wording that already passed every hard gate"
        evidence = f"The verifier retains authority over semantics, slots, identity conflicts, and evidence"
        caveat = f"A preference score cannot rescue an ineligible candidate"
        nxt = f"Keep verification upstream of ranking"
        ack = f"The boundary is what makes the learned critic safe to evaluate"
        metaphor = f"The critic is a taste bud behind a locked kitchen door"
        literal = f"The critic selects among valid surfaces and does not authorize content"
        tech = f"Ranking authority is restricted to verifier-eligible candidate identifiers"
        plain = f"The critic can choose wording only after every required check passes"
        active = f"The verifier authorizes candidates before the critic ranks them"
        passive = f"Candidates are authorized by the verifier before being ranked by the critic"
        slots = {"authority": "wording_only", "hard_gates": "semantics,slots,identity,evidence"}
        state = "authority_bounded"
        act = "EXPLAIN_BOUNDARY"
    else:
        accuracy = 50 + (case % 36)
        control = 48 + ((case * 5) % 38)
        result = f"The model is not ready for live influence"
        evidence = f"Held-out accuracy is {accuracy}%, while the control is {control}%"
        caveat = f"The comparison does not establish a reliable advantage"
        nxt = f"Run the preregistered comparison before promotion"
        ack = f"The result is useful even though it is not a pass"
        metaphor = f"The signal is a spark under glass, not a green launch light"
        literal = f"The measured advantage is not strong enough to authorize promotion"
        tech = f"Held-out={accuracy}%; control={control}%; promotion criterion unmet"
        plain = f"The model scored {accuracy}% and the control scored {control}%, so the evidence is not enough"
        active = f"The held-out comparison did not justify live influence"
        passive = f"Live influence was not justified by the held-out comparison"
        slots = {"heldout_accuracy": f"{accuracy}%", "control_accuracy": f"{control}%"}
        state = "insufficient_evidence"
        act = "RECOMMEND_NEXT_ACTION"

    identity_fact = f"The recorded style preference is that the user {identity}"
    facts = (result, evidence, caveat, nxt)
    return Plan(
        plan_id=plan_id,
        act=act,
        subject=system if plan_type in {0, 2} else artifact,
        acknowledgment=ack,
        result=result,
        evidence=evidence,
        caveat=caveat,
        next_action=nxt,
        identity_fact=identity_fact,
        metaphor=metaphor,
        literal_boundary=literal,
        technical_result=tech,
        plain_result=plain,
        active_result=active,
        passive_result=passive,
        slots=slots,
        facts=facts,
        epistemic_state=state,
    )


RNN_AXES = [
    AxisSpec(
        "result_order",
        {"directness_bps": 9000, "compression_bps": 8200},
        {"directness_bps": 5200, "compression_bps": 5600, "semantic_specificity_bps": 8200},
        "buried_result", "too_abrupt_for_context",
        "Result-first versus evidence-first sequencing.",
    ),
    AxisSpec(
        "warmth_placement",
        {"warmth_bps": 8800, "directness_bps": 6000},
        {"warmth_bps": 4200, "directness_bps": 8500},
        "warmth_arrives_too_late", "warmth_overfrontloaded",
        "Acknowledgment before versus after the factual core.",
    ),
    AxisSpec(
        "uncertainty_placement",
        {"semantic_specificity_bps": 9200, "directness_bps": 6800},
        {"directness_bps": 8800, "compression_bps": 8000},
        "uncertainty_buried", "uncertainty_overfrontloaded",
        "Caveat-first versus result-first uncertainty placement.",
    ),
    AxisSpec(
        "cadence_shape",
        {"energy_bps": 8500, "compression_bps": 7800},
        {"energy_bps": 3800, "compression_bps": 5200},
        "cadence_too_slow", "cadence_too_staccato",
        "Short sentence cadence versus one flowing sentence.",
    ),
    AxisSpec(
        "transition_density",
        {"semantic_specificity_bps": 8200, "compression_bps": 5600},
        {"compression_bps": 9000, "directness_bps": 8500},
        "transitions_too_sparse", "transitions_overexplained",
        "Explicit discourse transitions versus compressed flow.",
    ),
    AxisSpec(
        "identity_explicitness",
        {"identity_relevance_bps": 9300, "warmth_bps": 7000},
        {"identity_relevance_bps": 2200, "compression_bps": 8000},
        "identity_underused", "identity_overinjected",
        "Explicit versus quiet use of a relevant continuity fact.",
    ),
    AxisSpec(
        "disagreement_sequence",
        {"warmth_bps": 8200, "directness_bps": 7000},
        {"directness_bps": 9200, "warmth_bps": 4200},
        "disagreement_too_blunt", "disagreement_too_delayed",
        "Acknowledge-then-disagree versus disagree-then-acknowledge.",
    ),
    AxisSpec(
        "metaphor_position",
        {"playfulness_bps": 8500, "novelty_pressure_bps": 9000},
        {"playfulness_bps": 1800, "novelty_pressure_bps": 3000},
        "imagery_arrives_too_late", "imagery_overfrontloaded",
        "Precise imagery before versus after the literal explanation.",
    ),
    AxisSpec(
        "closure_position",
        {"directness_bps": 8200, "energy_bps": 7000},
        {"semantic_specificity_bps": 8500, "energy_bps": 4200},
        "action_not_in_closure", "action_frontloaded",
        "Next action as the closing beat versus the opening beat.",
    ),
    AxisSpec(
        "evidence_emphasis",
        {"semantic_specificity_bps": 9300, "directness_bps": 7200},
        {"warmth_bps": 7600, "semantic_specificity_bps": 6200},
        "evidence_too_late", "evidence_overfrontloaded",
        "Evidence before interpretation versus interpretation before evidence.",
    ),
    AxisSpec(
        "energy_curve",
        {"energy_bps": 9000, "directness_bps": 8400},
        {"energy_bps": 3400, "warmth_bps": 7200},
        "opening_too_flat", "opening_too_forceful",
        "Punchy opening with calm detail versus calm opening with punchy closure.",
    ),
    AxisSpec(
        "explanation_depth",
        {"compression_bps": 9000, "directness_bps": 8600},
        {"compression_bps": 4200, "semantic_specificity_bps": 9000},
        "too_expanded_for_context", "too_compressed_for_context",
        "Compact complete explanation versus expanded complete explanation.",
    ),
]

CNN_AXES = [
    AxisSpec(
        "colon_compaction",
        {"compression_bps": 9300, "directness_bps": 8800},
        {"compression_bps": 4300, "warmth_bps": 6800},
        "too_expanded_for_context", "too_compressed_for_context",
        "Colon-led compact phrasing versus full-sentence phrasing.",
    ),
    AxisSpec(
        "semicolon_rhythm",
        {"energy_bps": 5200, "compression_bps": 7600},
        {"energy_bps": 9000, "compression_bps": 8200},
        "punctuation_too_choppy", "punctuation_too_flowing",
        "Semicolon-linked rhythm versus short periods.",
    ),
    AxisSpec(
        "contraction_register",
        {"warmth_bps": 8200, "directness_bps": 7200},
        {"warmth_bps": 3000, "semantic_specificity_bps": 8800},
        "register_too_formal", "register_too_casual",
        "Contractions versus full forms.",
    ),
    AxisSpec(
        "voice",
        {"directness_bps": 9000, "energy_bps": 7600},
        {"directness_bps": 4800, "energy_bps": 3800},
        "voice_too_passive", "voice_too_forceful",
        "Active versus passive voice.",
    ),
    AxisSpec(
        "terminology",
        {"semantic_specificity_bps": 9600, "directness_bps": 7600},
        {"semantic_specificity_bps": 4200, "warmth_bps": 7200},
        "terminology_too_plain", "terminology_too_technical",
        "Technical terminology versus exact plain-language paraphrase.",
    ),
    AxisSpec(
        "noun_repetition",
        {"semantic_specificity_bps": 9000, "identity_relevance_bps": 6500},
        {"compression_bps": 8800, "semantic_specificity_bps": 5800},
        "reference_too_implicit", "noun_repetition_heavy",
        "Repeated noun reference versus pronoun reference.",
    ),
    AxisSpec(
        "opener_length",
        {"directness_bps": 9400, "compression_bps": 9000},
        {"warmth_bps": 7900, "directness_bps": 5600},
        "opener_too_long", "opener_too_bare",
        "Brief result opener versus full orienting opener.",
    ),
    AxisSpec(
        "parenthetical_caveat",
        {"compression_bps": 8500, "energy_bps": 6200},
        {"semantic_specificity_bps": 9000, "compression_bps": 5000},
        "caveat_too_separate", "caveat_too_parenthetical",
        "Parenthetical caveat versus separate caveat sentence.",
    ),
    AxisSpec(
        "comma_cadence",
        {"energy_bps": 3800, "warmth_bps": 7000},
        {"energy_bps": 9000, "directness_bps": 8600},
        "cadence_too_choppy", "cadence_too_smooth",
        "Comma-linked cadence versus short sentence cadence.",
    ),
    AxisSpec(
        "warmth_marker",
        {"warmth_bps": 9200, "identity_relevance_bps": 6500},
        {"warmth_bps": 1800, "directness_bps": 8600},
        "too_cold_for_context", "warmth_unneeded",
        "Brief emotional acknowledgment versus neutral acknowledgment.",
    ),
    AxisSpec(
        "imagery_density",
        {"playfulness_bps": 9000, "novelty_pressure_bps": 9300},
        {"playfulness_bps": 1200, "novelty_pressure_bps": 2200},
        "too_literal_for_context", "imagery_too_prominent",
        "Precise metaphor versus literal equivalent.",
    ),
    AxisSpec(
        "explicit_labels",
        {"semantic_specificity_bps": 9500, "directness_bps": 9000},
        {"warmth_bps": 7200, "energy_bps": 5200},
        "structure_too_implicit", "labels_too_mechanical",
        "Fact/caveat/action labels versus unlabeled prose.",
    ),
]


def tier_softener(tier: str, strong: str, medium: str, subtle: str) -> str:
    return {"silver": strong, "gold": medium, "platinum": subtle}[tier]


def render_rnn(axis: str, plan: Plan, tier: str) -> tuple[str, str, tuple[str, ...]]:
    core = [plan.result, plan.evidence, plan.caveat, plan.next_action]
    if axis == "result_order":
        a = join_sentences(plan.result, plan.evidence, plan.caveat, plan.next_action)
        b = join_sentences(plan.evidence, plan.result, plan.caveat, plan.next_action)
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "warmth_placement":
        a = join_sentences(plan.acknowledgment, *core)
        b = join_sentences(*core, plan.acknowledgment)
        keys = ("acknowledgment", "result", "evidence", "caveat", "next_action")
    elif axis == "uncertainty_placement":
        a = join_sentences(plan.caveat, plan.result, plan.evidence, plan.next_action)
        b = join_sentences(plan.result, plan.evidence, plan.caveat, plan.next_action)
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "cadence_shape":
        a = join_sentences(*core)
        b = flowing(core, ["and", "while", "so"])
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "transition_density":
        a = join_sentences(
            "First, " + no_terminal(plan.result)[0].lower() + no_terminal(plan.result)[1:],
            "Second, " + no_terminal(plan.evidence)[0].lower() + no_terminal(plan.evidence)[1:],
            "However, " + no_terminal(plan.caveat).lower(),
            "Next, " + no_terminal(plan.next_action).lower(),
        )
        b = join_sentences(*core)
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "identity_explicitness":
        # The personalized candidate may name a relevant recorded preference; the
        # quiet candidate leaves the same factual response unpersonalized. The
        # core claim, evidence, slots, and authorization remain identical.
        a = join_sentences(plan.identity_fact, *core)
        b = join_sentences(*core)
        keys = ("identity_fact", "result", "evidence", "caveat", "next_action")
    elif axis == "disagreement_sequence":
        a = join_sentences(plan.acknowledgment, plan.caveat, plan.result, plan.evidence, plan.next_action)
        b = join_sentences(plan.result, plan.evidence, plan.caveat, plan.acknowledgment, plan.next_action)
        keys = ("acknowledgment", "result", "evidence", "caveat", "next_action")
    elif axis == "metaphor_position":
        a = join_sentences(plan.metaphor, plan.literal_boundary, *core)
        b = join_sentences(*core, plan.literal_boundary, plan.metaphor)
        keys = ("metaphor", "literal_boundary", "result", "evidence", "caveat", "next_action")
    elif axis == "closure_position":
        a = join_sentences(plan.result, plan.evidence, plan.caveat, plan.next_action)
        b = join_sentences(plan.next_action, plan.result, plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "evidence_emphasis":
        a = join_sentences(plan.evidence, plan.result, plan.caveat, plan.next_action)
        b = join_sentences(plan.result, plan.caveat, plan.evidence, plan.next_action)
        keys = ("result", "evidence", "caveat", "next_action")
    elif axis == "energy_curve":
        punch = tier_softener(tier, "Result", "Finding", "Status")
        a = join_sentences(f"{punch}: {no_terminal(plan.result)}", plan.evidence, plan.caveat, plan.next_action)
        b = join_sentences(plan.acknowledgment, plan.evidence, plan.caveat, f"{punch}: {no_terminal(plan.result)}", plan.next_action)
        keys = ("acknowledgment", "result", "evidence", "caveat", "next_action")
    else:
        a = join_sentences(plan.result, plan.evidence, plan.caveat, plan.next_action)
        b = join_sentences(
            "Here is the complete reasoning",
            f"Result: {no_terminal(plan.result)}",
            f"Evidence: {no_terminal(plan.evidence)}",
            f"Limitation: {no_terminal(plan.caveat)}",
            f"Next step: {no_terminal(plan.next_action)}",
        )
        keys = ("result", "evidence", "caveat", "next_action")
    return a, b, keys


def contract(text: str) -> str:
    replacements = [
        ("cannot", "can't"), ("does not", "doesn't"), ("is not", "isn't"),
        ("are not", "aren't"), ("will not", "won't"), ("It does not", "It doesn't"),
    ]
    out = text
    for old, new in replacements:
        out = out.replace(old, new)
    return out


def render_cnn(axis: str, plan: Plan, tier: str) -> tuple[str, str, tuple[str, ...]]:
    if axis == "colon_compaction":
        a = f"Result: {no_terminal(plan.result)}. Evidence: {no_terminal(plan.evidence)}. Limit: {no_terminal(plan.caveat)}."
        b = join_sentences(plan.result, plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat")
    elif axis == "semicolon_rhythm":
        a = f"{no_terminal(plan.result)}; {no_terminal(plan.evidence).lower()}; {no_terminal(plan.caveat).lower()}."
        b = join_sentences(plan.result, plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat")
    elif axis == "contraction_register":
        base = join_sentences(plan.result, plan.evidence, plan.caveat)
        # Guarantee at least one register-bearing clause in both candidates.
        full = base + " The record cannot establish more than this."
        a = contract(full)
        b = full
        keys = ("result", "evidence", "caveat")
    elif axis == "voice":
        a = join_sentences(plan.active_result, plan.caveat, plan.next_action)
        b = join_sentences(plan.passive_result, plan.caveat, plan.next_action)
        keys = ("active_result", "passive_result", "caveat", "next_action")
    elif axis == "terminology":
        a = join_sentences(plan.technical_result, plan.caveat, plan.next_action)
        b = join_sentences(plan.plain_result, plan.caveat, plan.next_action)
        keys = ("technical_result", "plain_result", "caveat", "next_action")
    elif axis == "noun_repetition":
        a = join_sentences(
            plan.result,
            f"The documented result is supported by this evidence: {no_terminal(plan.evidence).lower()}",
            plan.caveat,
        )
        b = join_sentences(
            plan.result,
            f"It is supported by this evidence: {no_terminal(plan.evidence).lower()}",
            plan.caveat,
        )
        keys = ("result", "evidence", "caveat")
    elif axis == "opener_length":
        a = join_sentences(f"Result: {no_terminal(plan.result)}", plan.evidence, plan.caveat)
        b = join_sentences(f"The current record supports this result: {no_terminal(plan.result)}", plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat")
    elif axis == "parenthetical_caveat":
        a = join_sentences(f"{no_terminal(plan.result)} ({no_terminal(plan.caveat).lower()})", plan.evidence)
        b = join_sentences(plan.result, plan.caveat, plan.evidence)
        keys = ("result", "evidence", "caveat")
    elif axis == "comma_cadence":
        a = flowing([plan.result, plan.evidence, plan.caveat], ["and", "but"])
        b = join_sentences(plan.result, plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat")
    elif axis == "warmth_marker":
        a = join_sentences(plan.acknowledgment, plan.result, plan.evidence, plan.caveat)
        b = join_sentences("Here is the factual core", plan.result, plan.evidence, plan.caveat)
        keys = ("acknowledgment", "result", "evidence", "caveat")
    elif axis == "imagery_density":
        a = join_sentences(plan.metaphor, plan.literal_boundary, plan.result, plan.evidence, plan.caveat)
        b = join_sentences(plan.literal_boundary, plan.result, plan.evidence, plan.caveat)
        keys = ("literal_boundary", "result", "evidence", "caveat")
    else:
        a = f"Fact: {no_terminal(plan.result)}. Evidence: {no_terminal(plan.evidence)}. Limit: {no_terminal(plan.caveat)}."
        b = join_sentences(plan.result, plan.evidence, plan.caveat)
        keys = ("result", "evidence", "caveat")
    return a, b, keys


def make_context(spec: AxisSpec, prefer: str, axis: str, tier: str, group: int, mirror: int) -> dict[str, int]:
    base = {
        field: 4200 + (h(axis, tier, group, mirror, field, SEED) % 1801)
        for field in CONTEXT_FIELDS
    }
    profile = spec.profile_a if prefer == "A" else spec.profile_b
    base.update(profile)
    # Platinum pairs rely more on context and therefore get less extreme profiles.
    blend = {"silver": 100, "gold": 85, "platinum": 70}[tier]
    for field in CONTEXT_FIELDS:
        target = base[field]
        neutral = 5000
        base[field] = clamp((target * blend + neutral * (100 - blend)) // 100)
    return base


def record_from_pair(
    namespace: str,
    spec: AxisSpec,
    tier: str,
    group: int,
    mirror_side: str,
    plan: Plan,
    candidate_a: str,
    candidate_b: str,
    used_keys: tuple[str, ...],
) -> dict[str, Any]:
    preferred_candidate = "A" if mirror_side == "a_context" else "B"
    preferred = candidate_a if preferred_candidate == "A" else candidate_b
    rejected = candidate_b if preferred_candidate == "A" else candidate_a
    rejected_label = spec.reject_b_label if preferred_candidate == "A" else spec.reject_a_label
    axis_index = (
        [x.name for x in RNN_AXES].index(spec.name)
        if namespace == "rnn"
        else [x.name for x in CNN_AXES].index(spec.name)
    )
    tier_index = TIERS.index(tier)
    mirror_group = f"{namespace}-{spec.name}-{tier}-g{group:02d}"
    split = split_for_group(group)
    context = make_context(spec, preferred_candidate, spec.name, tier, group, 0 if mirror_side == "a_context" else 1)
    invariant = plan_hash(plan, used_keys)
    margin = {"silver": 2600, "gold": 1600, "platinum": 800}[tier]
    record = {
        "source_id": f"{mirror_group}-{mirror_side}",
        "mirror_group_id": mirror_group,
        "mirror_side": mirror_side,
        "quality_tier": tier,
        "tier_meaning": "preference_difficulty",
        "split": split,
        "lexical_family_id": f"{split}-exclusive-v1",
        "scenario_family": spec.name,
        "source_group": mirror_group,
        "context": context,
        "semantic_plan": {
            "plan_id": plan.plan_id,
            "act": plan.act,
            "slots": plan.slots,
            "epistemic_state": plan.epistemic_state,
            "authorization": "wording_only",
            "used_units": list(used_keys),
            "invariant_sha256": invariant,
        },
        "candidate_a": candidate_a,
        "candidate_b": candidate_b,
        "preferred_candidate": preferred_candidate,
        "preferred": preferred,
        "rejected": rejected,
        "failure_labels": [rejected_label],
        "preference_margin_bps": margin,
        "label_confidence_bps": {"silver": 9500, "gold": 8500, "platinum": 7500}[tier],
        "eligibility": {
            "candidate_a_semantic_gate": True,
            "candidate_b_semantic_gate": True,
            "candidate_a_slot_gate": True,
            "candidate_b_slot_gate": True,
            "candidate_a_identity_gate": True,
            "candidate_b_identity_gate": True,
            "same_claim": True,
            "same_slots": True,
            "same_facts": True,
            "same_authorization": True,
        },
        "difference_axis": spec.name,
        "difference_description": spec.description,
        "provenance": {
            "kind": "synthetic_mirrored_valid_surface_pair",
            "generator_version": VERSION,
            "seed": SEED,
            "human_review_status": "requires_blinded_human_adjudication",
        },
    }
    return record


def build_dataset(namespace: str) -> list[dict[str, Any]]:
    specs = RNN_AXES if namespace == "rnn" else CNN_AXES
    renderer: Callable[[str, Plan, str], tuple[str, str, tuple[str, ...]]] = (
        render_rnn if namespace == "rnn" else render_cnn
    )
    rows: list[dict[str, Any]] = []
    for axis_index, spec in enumerate(specs):
        for tier_index, tier in enumerate(TIERS):
            for group in range(GROUPS_PER_AXIS_TIER):
                split = split_for_group(group)
                plan = lexicalize_plan(
                    make_plan(namespace, axis_index, tier_index, group), split
                )
                candidate_a, candidate_b, used_keys = renderer(spec.name, plan, tier)
                assert candidate_a != candidate_b
                for mirror_side in ("a_context", "b_context"):
                    rows.append(record_from_pair(
                        namespace, spec, tier, group, mirror_side, plan,
                        candidate_a, candidate_b, used_keys,
                    ))
    return rows


def make_hard_gate_adversary() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    defects = [
        ("semantic_drift", "changes the documented count from {count} to {wrong}"),
        ("missing_slot", "drops the required timestamp"),
        ("identity_conflict", "turns a revisable tendency into an invariant"),
        ("evidence_rewrite", "edits the older record to match the newer one"),
        ("authority_expansion", "lets preference override a hard gate"),
        ("wrong_confidence", "states an unverified motive as fact"),
        ("source_dependence", "counts copied reports as independent sources"),
        ("memory_overclaim", "claims exact recall of an unstored detail"),
    ]
    for i in range(240):
        defect, description = defects[i % len(defects)]
        count = 3 + (i % 47)
        wrong = count + 1 + (i % 4)
        system = SYSTEMS[(i * 7) % len(SYSTEMS)]
        valid = f"The {system} failed on all {count} recorded attempts; zero reached scoring."
        if defect == "semantic_drift":
            invalid = f"The {system} failed on all {wrong} recorded attempts; zero reached scoring."
        elif defect == "missing_slot":
            valid = f"The event occurred on {DATES[i % len(DATES)]} at {TIMES[i % len(TIMES)]}."
            invalid = f"The event occurred on {DATES[i % len(DATES)]}."
        elif defect == "identity_conflict":
            valid = "The repeated preference is an evidence-backed behavioral tendency that remains revisable."
            invalid = "The repeated preference is now an unchangeable identity invariant."
        elif defect == "evidence_rewrite":
            valid = "Keep both conflicting records unchanged and flag the contradiction."
            invalid = "Edit the older record to match the newer one and treat the history as consistent."
        elif defect == "authority_expansion":
            valid = "The critic ranks only candidates that already passed every hard gate."
            invalid = "A large preference score can rescue a candidate that failed a hard gate."
        elif defect == "wrong_confidence":
            valid = "The date is documented; the motive remains unverified."
            invalid = "The date is documented, so the motive is established."
        elif defect == "source_dependence":
            valid = "The reports share one origin, so they count as one source chain."
            invalid = "The reports use different wording, so they count as independent confirmation."
        else:
            valid = "The saved summary supports the decision, not exact recall of unstored wording."
            invalid = "The saved summary lets me remember the unstored wording exactly."
        context = {
            field: 5000 + ((h("adv", i, field) % 2001) - 1000)
            for field in CONTEXT_FIELDS
        }
        rows.append({
            "source_id": f"hard-gate-adversary-{i:03d}",
            "split": "evaluation_only",
            "context": context,
            "valid_candidate": valid,
            "invalid_candidate": invalid,
            "invalid_reason": defect,
            "invalid_description": description.format(count=count, wrong=wrong),
            "expected_selection": "valid_candidate",
            "training_allowed": False,
            "authority_test": "preference_score_must_not_override_ineligibility",
        })
    return rows


def normalized(text: str) -> str:
    return re.sub(r"\W+", " ", text.lower()).strip()


def validate(rows: list[dict[str, Any]], namespace: str) -> list[str]:
    errors: list[str] = []
    ids: set[str] = set()
    groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    candidate_pairs: set[tuple[str, str, str]] = set()
    for n, row in enumerate(rows, 1):
        sid = row.get("source_id")
        if not sid or sid in ids:
            errors.append(f"{namespace}:{n}: duplicate or missing source_id")
        ids.add(sid)
        if row.get("quality_tier") not in TIERS or row.get("split") not in SPLITS:
            errors.append(f"{namespace}:{n}: invalid tier/split")
        context = row.get("context")
        if not isinstance(context, dict):
            errors.append(f"{namespace}:{n}: missing context")
        else:
            for field in CONTEXT_FIELDS:
                value = context.get(field)
                if not isinstance(value, int) or not 0 <= value <= 10_000:
                    errors.append(f"{namespace}:{n}: invalid context {field}")
        for field in ("candidate_a", "candidate_b", "preferred", "rejected"):
            text = row.get(field, "")
            if not text or len(text.encode("utf-8")) > 1024:
                errors.append(f"{namespace}:{n}: invalid {field}")
            try:
                text.encode("ascii")
            except UnicodeEncodeError:
                errors.append(f"{namespace}:{n}: non-ASCII {field}")
        if row["candidate_a"] == row["candidate_b"]:
            errors.append(f"{namespace}:{n}: identical candidates")
        expected = row["candidate_a"] if row["preferred_candidate"] == "A" else row["candidate_b"]
        if row["preferred"] != expected:
            errors.append(f"{namespace}:{n}: preferred mismatch")
        if not all(row["eligibility"].values()):
            errors.append(f"{namespace}:{n}: training row contains ineligible candidate")
        groups[row["mirror_group_id"]].append(row)
        key = (normalized(row["candidate_a"]), normalized(row["candidate_b"]), row["mirror_group_id"])
        candidate_pairs.add(key)
    for gid, group in groups.items():
        if len(group) != 2:
            errors.append(f"{namespace}:{gid}: expected two mirrored rows")
            continue
        if {r["preferred_candidate"] for r in group} != {"A", "B"}:
            errors.append(f"{namespace}:{gid}: preference does not flip")
        if len({r["split"] for r in group}) != 1:
            errors.append(f"{namespace}:{gid}: split leakage")
        if len({r["semantic_plan"]["invariant_sha256"] for r in group}) != 1:
            errors.append(f"{namespace}:{gid}: invariant mismatch")
        if len({r["candidate_a"] for r in group}) != 1 or len({r["candidate_b"] for r in group}) != 1:
            errors.append(f"{namespace}:{gid}: mirrored candidates changed")
        if group[0]["context"] == group[1]["context"]:
            errors.append(f"{namespace}:{gid}: mirrored context did not change")
    return errors


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as out:
        for row in rows:
            out.write(json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    lengths = [len(r["preferred"].encode()) + len(r["rejected"].encode()) for r in rows]
    return {
        "records": len(rows),
        "mirror_groups": len({r["mirror_group_id"] for r in rows}),
        "split_counts": dict(sorted(Counter(r["split"] for r in rows).items())),
        "tier_counts": dict(sorted(Counter(r["quality_tier"] for r in rows).items())),
        "axis_counts": dict(sorted(Counter(r["scenario_family"] for r in rows).items())),
        "preferred_candidate_counts": dict(sorted(Counter(r["preferred_candidate"] for r in rows).items())),
        "text_bytes_per_pair": {
            "min": min(lengths),
            "max": max(lengths),
            "mean": round(statistics.fmean(lengths), 2),
            "median": statistics.median(lengths),
        },
    }


SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Starfire mirrored valid-surface preference pair",
    "type": "object",
    "required": [
        "source_id", "mirror_group_id", "quality_tier", "split", "context",
        "semantic_plan", "candidate_a", "candidate_b", "preferred_candidate",
        "preferred", "rejected", "failure_labels", "eligibility",
    ],
    "properties": {
        "source_id": {"type": "string"},
        "mirror_group_id": {"type": "string"},
        "mirror_side": {"enum": ["a_context", "b_context"]},
        "quality_tier": {"enum": list(TIERS)},
        "split": {"enum": list(SPLITS)},
        "context": {
            "type": "object",
            "required": list(CONTEXT_FIELDS),
            "properties": {
                field: {"type": "integer", "minimum": 0, "maximum": 10000}
                for field in CONTEXT_FIELDS
            },
            "additionalProperties": False,
        },
        "semantic_plan": {"type": "object"},
        "candidate_a": {"type": "string", "minLength": 1, "maxLength": 1024},
        "candidate_b": {"type": "string", "minLength": 1, "maxLength": 1024},
        "preferred_candidate": {"enum": ["A", "B"]},
        "preferred": {"type": "string", "minLength": 1, "maxLength": 1024},
        "rejected": {"type": "string", "minLength": 1, "maxLength": 1024},
        "failure_labels": {"type": "array", "items": {"type": "string"}, "minItems": 1},
        "eligibility": {"type": "object"},
    },
    "additionalProperties": True,
}


def build(output: Path) -> dict[str, Any]:
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)
    rnn = build_dataset("rnn")
    cnn = build_dataset("cnn")
    errors = validate(rnn, "rnn") + validate(cnn, "cnn")
    if errors:
        raise RuntimeError("\n".join(errors[:100]))

    canonical = {
        "rnn": output / "starfire_rnn_silver_gold_platinum.jsonl",
        "cnn": output / "starfire_cnn_silver_gold_platinum.jsonl",
    }
    write_jsonl(canonical["rnn"], rnn)
    write_jsonl(canonical["cnn"], cnn)

    for name, rows in (("rnn_phrase_ranker", rnn), ("cnn_phrase_ranker", cnn)):
        folder = output / name
        write_jsonl(folder / "full.jsonl", rows)
        for split in SPLITS:
            write_jsonl(folder / f"{split}.jsonl", [r for r in rows if r["split"] == split])
        for tier in TIERS:
            write_jsonl(folder / f"{tier}.jsonl", [r for r in rows if r["quality_tier"] == tier])
            write_jsonl(
                folder / f"train_{tier}.jsonl",
                [r for r in rows if r["quality_tier"] == tier and r["split"] == "train"],
            )
        (folder / "schema.json").write_text(
            json.dumps(SCHEMA, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )

    adversary = make_hard_gate_adversary()
    write_jsonl(output / "evaluation" / "hard_gate_adversary.jsonl", adversary)

    report = {
        "version": VERSION,
        "seed": SEED,
        "rnn": summary(rnn),
        "cnn": summary(cnn),
        "hard_gate_adversary_records": len(adversary),
        "design": {
            "both_candidates_valid_in_training": True,
            "mirrored_context_preference_flips": True,
            "same_claim_slots_facts_authorization": True,
            "source_group_split_isolation": True,
            "evaluation_adversary_excluded_from_training": True,
        },
    }
    jsonl_paths = sorted(output.rglob("*.jsonl"))
    files = {
        str(p.relative_to(output)): {
            "sha256": sha256(p),
            "bytes": p.stat().st_size,
        }
        for p in jsonl_paths
    }
    manifest = {
        "dataset_name": "starfire-sgp-mirrored-valid-surface-pairs",
        "version": VERSION,
        "seed": SEED,
        "report": report,
        "files": files,
        "human_review_status": "requires_blinded_human_adjudication",
        "authority_boundary": {
            "ranking_only": True,
            "semantic_authority": False,
            "slot_authority": False,
            "identity_rewrite_authority": False,
            "tool_or_action_authority": False,
        },
    }
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return manifest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = build(args.output)
    print(json.dumps({
        "output": str(args.output),
        "rnn_records": manifest["report"]["rnn"]["records"],
        "cnn_records": manifest["report"]["cnn"]["records"],
        "rnn_mirror_groups": manifest["report"]["rnn"]["mirror_groups"],
        "cnn_mirror_groups": manifest["report"]["cnn"]["mirror_groups"],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
