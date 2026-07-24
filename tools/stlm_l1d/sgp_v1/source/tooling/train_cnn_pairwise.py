#!/usr/bin/env python3
from pathlib import Path
import argparse, json
from pairwise_common import CnnRanker, run_training
p=argparse.ArgumentParser(description="Train a byte-level temporal CNN phrase ranker")
p.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]); p.add_argument("--output",type=Path,default=Path("models/cnn_phrase_ranker.pt"))
p.add_argument("--epochs",type=int,default=30); p.add_argument("--batch-size",type=int,default=64); p.add_argument("--lr",type=float,default=1e-3); p.add_argument("--seed",type=int,default=1729); p.add_argument("--device",default="cpu"); p.add_argument("--max-records",type=int)
p.add_argument("--embed-dim",type=int,default=64); p.add_argument("--channels",type=int,default=32); p.add_argument("--context-dim",type=int,default=32); p.add_argument("--dropout",type=float,default=.1)
a=p.parse_args(); out=a.output if a.output.is_absolute() else a.root/a.output
model=CnnRanker(a.embed_dim,a.channels,(2,3,4,5,7),a.context_dim,a.dropout)
metrics=run_training(model,a.root/"cnn_phrase_ranker/train.jsonl",a.root/"cnn_phrase_ranker/dev.jsonl",a.root/"cnn_phrase_ranker/test.jsonl",out,a.epochs,a.batch_size,a.lr,a.seed,a.device,a.max_records)
print(json.dumps({"status":"complete","model":str(out),"metrics":metrics},sort_keys=True))
