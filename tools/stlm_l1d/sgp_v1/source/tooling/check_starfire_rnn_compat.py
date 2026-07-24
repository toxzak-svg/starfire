#!/usr/bin/env python3
"""Validate the exact required JSONL contract used by Starfire's current RNN trainer."""
import argparse,json
from pathlib import Path
FIELDS=("directness_bps","warmth_bps","energy_bps","compression_bps","playfulness_bps","novelty_pressure_bps","identity_relevance_bps","semantic_specificity_bps")
p=argparse.ArgumentParser(); p.add_argument("input",type=Path,nargs="?",default=Path(__file__).resolve().parents[1]/"rnn_phrase_ranker/train.jsonl"); a=p.parse_args(); n=0
for line_no,line in enumerate(a.input.open(encoding="utf-8"),1):
 if not line.strip(): continue
 r=json.loads(line); n+=1
 assert str(r.get("source_id","")).strip(),f"line {line_no}: source_id"
 assert isinstance(r.get("context"),dict),f"line {line_no}: context"
 for f in FIELDS:
  v=r["context"].get(f); assert isinstance(v,int) and 0<=v<=10000,f"line {line_no}: {f}"
 assert str(r.get("preferred","")).strip() and str(r.get("rejected","")).strip(),f"line {line_no}: text"
 assert r["preferred"]!=r["rejected"],f"line {line_no}: equal pair"
 assert len(r["preferred"].encode())<=1024 and len(r["rejected"].encode())<=1024,f"line {line_no}: byte limit"
 labels=r.get("failure_labels",[]); assert isinstance(labels,list) and all(isinstance(x,str) for x in labels),f"line {line_no}: labels"
assert n>=8
print(json.dumps({"status":"PASS","records":n,"input":str(a.input)}))
