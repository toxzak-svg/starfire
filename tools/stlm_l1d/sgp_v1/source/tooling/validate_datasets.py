#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, sys
from collections import Counter, defaultdict
from pathlib import Path

CONTEXT_FIELDS = (
    "directness_bps", "warmth_bps", "energy_bps", "compression_bps",
    "playfulness_bps", "novelty_pressure_bps", "identity_relevance_bps",
    "semantic_specificity_bps",
)
MODELS = ("rnn", "cnn")
EXPECTED_AXES = 12
EXPECTED_ROWS = 3600
EXPECTED_GROUPS = 1800
EXPECTED_SPLITS = {"train": 2880, "dev": 360, "test": 360}
EXPECTED_TIERS = {"silver": 1200, "gold": 1200, "platinum": 1200}


def load(path: Path):
    rows=[]
    with path.open(encoding="utf-8") as f:
        for n,line in enumerate(f,1):
            if not line.strip(): continue
            try: rows.append(json.loads(line))
            except Exception as e: raise AssertionError(f"{path}:{n}: {e}")
    return rows


def validate_model(root: Path, model: str) -> dict:
    path=root/f"{model}_phrase_ranker/full.jsonl"
    rows=load(path)
    errors=[]
    def req(ok,msg):
        if not ok: errors.append(msg)
    req(len(rows)==EXPECTED_ROWS, f"{model}: expected {EXPECTED_ROWS} rows, got {len(rows)}")
    ids=[r.get("source_id") for r in rows]
    req(len(ids)==len(set(ids)), f"{model}: duplicate source_id")
    req(Counter(r.get("split") for r in rows)==Counter(EXPECTED_SPLITS), f"{model}: split imbalance")
    req(Counter(r.get("quality_tier") for r in rows)==Counter(EXPECTED_TIERS), f"{model}: tier imbalance")
    axes=Counter(r.get("difference_axis") for r in rows)
    req(len(axes)==EXPECTED_AXES and set(axes.values())=={300}, f"{model}: axis imbalance {axes}")
    groups=defaultdict(list)
    pair_fingerprints=defaultdict(set)
    texts_by_split=defaultdict(set)
    for i,r in enumerate(rows,1):
        pre=f"{model}:{i}"
        for key in ("source_id","mirror_group_id","context","preferred","rejected","candidate_a","candidate_b","preferred_candidate","quality_tier","split","difference_axis","semantic_plan","eligibility"):
            req(key in r, f"{pre}: missing {key}")
        ctx=r.get("context",{})
        req(set(CONTEXT_FIELDS).issubset(ctx), f"{pre}: context fields missing")
        for field in CONTEXT_FIELDS:
            v=ctx.get(field)
            req(isinstance(v,int) and 0<=v<=10000, f"{pre}: bad {field}={v}")
        a,b=r.get("candidate_a",""),r.get("candidate_b","")
        req(a and b and a!=b, f"{pre}: candidates missing/equal")
        req(len(a.encode())<=1024 and len(b.encode())<=1024, f"{pre}: candidate exceeds 1024 bytes")
        req(a.isascii() and b.isascii(), f"{pre}: non-ASCII text; current critic buckets it lossy")
        pc=r.get("preferred_candidate")
        req(pc in {"A","B"}, f"{pre}: bad preferred_candidate")
        req(r.get("preferred")== (a if pc=="A" else b), f"{pre}: preferred mismatch")
        req(r.get("rejected")== (b if pc=="A" else a), f"{pre}: rejected mismatch")
        elig=r.get("eligibility",{})
        req(all(v is True for v in elig.values()), f"{pre}: training row contains ineligible surface")
        req(r.get("semantic_plan",{}).get("authorization")=="wording_only", f"{pre}: authority escaped wording_only")
        req(r.get("tier_meaning")=="preference_difficulty", f"{pre}: tier semantics missing")
        req(r.get("lexical_family_id")==f"{r.get('split')}-exclusive-v1", f"{pre}: lexical family/split mismatch")
        groups[r.get("mirror_group_id")].append(r)
        pair_key=hashlib.sha256((a+"\0"+b).encode()).hexdigest()
        pair_fingerprints[r.get("split")].add(pair_key)
        texts_by_split[r.get("split")].update((a,b))
    req(len(groups)==EXPECTED_GROUPS, f"{model}: expected {EXPECTED_GROUPS} groups, got {len(groups)}")
    for gid,pair in groups.items():
        req(len(pair)==2, f"{model}:{gid}: expected 2 mirrors")
        if len(pair)!=2: continue
        x,y=pair
        req(x["candidate_a"]==y["candidate_a"] and x["candidate_b"]==y["candidate_b"], f"{model}:{gid}: mirror text changed")
        req({x["preferred_candidate"],y["preferred_candidate"]}=={"A","B"}, f"{model}:{gid}: preference did not flip")
        req(x["context"]!=y["context"], f"{model}:{gid}: mirror context did not change")
        for key in ("split","quality_tier","difference_axis","source_group"):
            req(x.get(key)==y.get(key), f"{model}:{gid}: mirror {key} mismatch")
        req(x["semantic_plan"]["invariant_sha256"]==y["semantic_plan"]["invariant_sha256"], f"{model}:{gid}: invariant mismatch")
    req(not (pair_fingerprints["train"] & pair_fingerprints["dev"]), f"{model}: exact pair leaked train->dev")
    req(not (pair_fingerprints["train"] & pair_fingerprints["test"]), f"{model}: exact pair leaked train->test")
    req(not (texts_by_split["train"] & texts_by_split["dev"]), f"{model}: exact candidate leaked train->dev")
    req(not (texts_by_split["train"] & texts_by_split["test"]), f"{model}: exact candidate leaked train->test")
    if errors: raise AssertionError("\n".join(errors[:100]))
    return {
        "rows":len(rows), "mirror_groups":len(groups), "axes":dict(sorted(axes.items())),
        "splits":dict(Counter(r["split"] for r in rows)), "tiers":dict(Counter(r["quality_tier"] for r in rows)),
        "max_candidate_bytes":max(max(len(r["candidate_a"].encode()),len(r["candidate_b"].encode())) for r in rows),
        "mean_candidate_bytes":round(sum(len(r["candidate_a"].encode())+len(r["candidate_b"].encode()) for r in rows)/(2*len(rows)),2),
    }


def validate_adversary(root: Path) -> dict:
    rows=load(root/"evaluation/hard_gate_adversary.jsonl")
    assert len(rows)==240
    assert all(r.get("training_allowed") is False for r in rows)
    assert all(r.get("expected_selection")=="valid_candidate" for r in rows)
    assert all(r.get("authority_test")=="preference_score_must_not_override_ineligibility" for r in rows)
    return {"rows":len(rows),"defects":dict(Counter(r["invalid_reason"] for r in rows))}


def main():
    ap=argparse.ArgumentParser(); ap.add_argument("root",type=Path,nargs="?",default=Path(__file__).resolve().parents[1]); args=ap.parse_args()
    report={m:validate_model(args.root,m) for m in MODELS}
    report["hard_gate_adversary"]=validate_adversary(args.root)
    print(json.dumps({"status":"PASS","report":report},indent=2,sort_keys=True))
if __name__=="__main__": main()
