#!/usr/bin/env python3
from __future__ import annotations
import argparse,csv,json,random
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument("input",type=Path); p.add_argument("output",type=Path); p.add_argument("--per-axis-tier",type=int,default=5); p.add_argument("--seed",type=int,default=1729); a=p.parse_args()
rows=[json.loads(x) for x in a.input.read_text(encoding="utf-8").splitlines() if x.strip() and json.loads(x).get("mirror_side")=="a_context"]
rng=random.Random(a.seed); buckets={}
for r in rows: buckets.setdefault((r["difference_axis"],r["quality_tier"]),[]).append(r)
selected=[]
for key,items in sorted(buckets.items()): rng.shuffle(items); selected.extend(items[:a.per_axis_tier])
rng.shuffle(selected); a.output.parent.mkdir(parents=True,exist_ok=True)
fields=["review_id","source_id","axis","tier","context_json","left","right","human_choice","confidence_1_5","notes"]
key_rows=[]
with a.output.open("w",newline="",encoding="utf-8") as f:
 w=csv.DictWriter(f,fieldnames=fields); w.writeheader()
 for i,r in enumerate(selected,1):
  flip=rng.choice((False,True)); left=r["candidate_b"] if flip else r["candidate_a"]; right=r["candidate_a"] if flip else r["candidate_b"]
  preferred_side=("left" if (r["preferred_candidate"]==("B" if flip else "A")) else "right")
  rid=f"review-{i:04d}"; w.writerow({"review_id":rid,"source_id":r["source_id"],"axis":r["difference_axis"],"tier":r["quality_tier"],"context_json":json.dumps(r["context"],sort_keys=True),"left":left,"right":right,"human_choice":"","confidence_1_5":"","notes":""})
  key_rows.append({"review_id":rid,"expected_choice":preferred_side,"source_id":r["source_id"]})
key=a.output.with_suffix(a.output.suffix+".key.jsonl")
with key.open("w",encoding="utf-8") as f:
 for r in key_rows: f.write(json.dumps(r,sort_keys=True)+"\n")
print(json.dumps({"rows":len(selected),"sheet":str(a.output),"blinded_key":str(key)}))
