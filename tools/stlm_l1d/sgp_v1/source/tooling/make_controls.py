#!/usr/bin/env python3
from __future__ import annotations
import argparse,copy,json,random
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument("input",type=Path); p.add_argument("output_dir",type=Path); p.add_argument("--seed",type=int,default=1729); a=p.parse_args()
rows=[json.loads(x) for x in a.input.read_text(encoding="utf-8").splitlines() if x.strip()]; a.output_dir.mkdir(parents=True,exist_ok=True)
def write(name,data):
 path=a.output_dir/name
 with path.open("w",encoding="utf-8") as f:
  for r in data: f.write(json.dumps(r,sort_keys=True,separators=(",",":"))+"\n")
 print(json.dumps({"control":name,"rows":len(data),"path":str(path)}))
rev=[]
for src in rows:
 r=copy.deepcopy(src); r["preferred"],r["rejected"]=r["rejected"],r["preferred"]; r["preferred_candidate"]="B" if r["preferred_candidate"]=="A" else "A"; r["control_type"]="reversed_label"; rev.append(r)
write("reversed_labels.jsonl",rev)
rng=random.Random(a.seed); shuffled=[]
for src in rows:
 r=copy.deepcopy(src); choose=rng.choice(("A","B")); r["preferred_candidate"]=choose; r["preferred"]=r["candidate_a"] if choose=="A" else r["candidate_b"]; r["rejected"]=r["candidate_b"] if choose=="A" else r["candidate_a"]; r["control_type"]="random_label"; shuffled.append(r)
write("random_labels.jsonl",shuffled)
zero=[]; noid=[]
for src in rows:
 r=copy.deepcopy(src); r["context"]={k:5000 for k in r["context"]}; r["control_type"]="context_zeroed"; zero.append(r)
 q=copy.deepcopy(src); q["context"]["identity_relevance_bps"]=5000; q["control_type"]="identity_relevance_zeroed"; noid.append(q)
write("context_zeroed.jsonl",zero); write("identity_relevance_zeroed.jsonl",noid)
