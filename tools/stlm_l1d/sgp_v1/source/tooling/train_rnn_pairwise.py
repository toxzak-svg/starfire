#!/usr/bin/env python3
from pathlib import Path
import argparse, json
from pairwise_common import RnnRanker, run_training
p=argparse.ArgumentParser(description="Train a byte-level GRU phrase ranker with sequence pooling")
p.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]); p.add_argument("--output",type=Path,default=Path("models/rnn_phrase_ranker_gru.pt"))
p.add_argument("--epochs",type=int,default=30); p.add_argument("--batch-size",type=int,default=64); p.add_argument("--lr",type=float,default=1e-3); p.add_argument("--seed",type=int,default=1729); p.add_argument("--device",default="cpu"); p.add_argument("--max-records",type=int)
p.add_argument("--embed-dim",type=int,default=64); p.add_argument("--hidden",type=int,default=64); p.add_argument("--layers",type=int,default=1); p.add_argument("--context-dim",type=int,default=32); p.add_argument("--dropout",type=float,default=.1)
a=p.parse_args(); out=a.output if a.output.is_absolute() else a.root/a.output
model=RnnRanker(a.embed_dim,a.hidden,a.layers,a.context_dim,a.dropout)
metrics=run_training(model,a.root/"rnn_phrase_ranker/train.jsonl",a.root/"rnn_phrase_ranker/dev.jsonl",a.root/"rnn_phrase_ranker/test.jsonl",out,a.epochs,a.batch_size,a.lr,a.seed,a.device,a.max_records)
print(json.dumps({"status":"complete","model":str(out),"metrics":metrics},sort_keys=True))
