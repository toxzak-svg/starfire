#!/usr/bin/env python3
import argparse,json
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument("input",type=Path); p.add_argument("output",type=Path); p.add_argument("--tier",action="append"); p.add_argument("--split",action="append"); p.add_argument("--axis",action="append"); a=p.parse_args()
a.output.parent.mkdir(parents=True,exist_ok=True); n=0
with a.input.open(encoding="utf-8") as src,a.output.open("w",encoding="utf-8") as dst:
 for line in src:
  if not line.strip(): continue
  r=json.loads(line)
  if a.tier and r.get("quality_tier") not in a.tier: continue
  if a.split and r.get("split") not in a.split: continue
  if a.axis and r.get("difference_axis") not in a.axis: continue
  dst.write(json.dumps(r,sort_keys=True,separators=(",",":"))+"\n"); n+=1
print(json.dumps({"written":n,"output":str(a.output)}))
