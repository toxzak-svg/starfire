from __future__ import annotations
import json, math, random
from collections import defaultdict
from pathlib import Path
from typing import Iterable
import torch
import torch.nn.functional as F
from torch import nn
from torch.utils.data import Dataset, DataLoader

CONTEXT_FIELDS=("directness_bps","warmth_bps","energy_bps","compression_bps","playfulness_bps","novelty_pressure_bps","identity_relevance_bps","semantic_specificity_bps")
PAD=256; VOCAB=257; MAX_BYTES=1024

def seed_all(seed:int):
    random.seed(seed); torch.manual_seed(seed)
    torch.use_deterministic_algorithms(True, warn_only=True)

def encode(text:str): return list(text.encode("utf-8")[:MAX_BYTES])

class PairDataset(Dataset):
    def __init__(self,path:Path,max_records:int|None=None):
        self.rows=[]
        with path.open(encoding="utf-8") as f:
            for line in f:
                if line.strip(): self.rows.append(json.loads(line))
                if max_records and len(self.rows)>=max_records: break
    def __len__(self): return len(self.rows)
    def __getitem__(self,i):
        r=self.rows[i]
        return encode(r["preferred"]),encode(r["rejected"]),[r["context"][k] for k in CONTEXT_FIELDS],r["quality_tier"],r["difference_axis"]

def collate(batch):
    p,n,c,tier,axis=zip(*batch); maxlen=max(max(map(len,p)),max(map(len,n)))
    def pad(items):
        out=torch.full((len(items),maxlen),PAD,dtype=torch.long); lengths=[]
        for i,x in enumerate(items): out[i,:len(x)]=torch.tensor(x); lengths.append(len(x))
        return out,torch.tensor(lengths)
    pt,pl=pad(p); nt,nl=pad(n)
    return pt,pl,nt,nl,torch.tensor(c,dtype=torch.float32),list(tier),list(axis)

class RankerBase(nn.Module):
    def __init__(self,feature_dim:int,context_dim:int=32,dropout:float=.1):
        super().__init__(); self.context=nn.Sequential(nn.Linear(8,context_dim),nn.Tanh())
        total=feature_dim+context_dim
        self.head=nn.Sequential(nn.Linear(total,total),nn.GELU(),nn.Dropout(dropout),nn.Linear(total,1))
        self.skip=nn.Linear(total,1,bias=False)
    def finish(self,features,context):
        ctx=self.context(context/5000.0-1.0); z=torch.cat([features,ctx],-1)
        return (self.head(z)+self.skip(z)).squeeze(-1)

class CnnRanker(RankerBase):
    def __init__(self,embed_dim=64,channels=32,widths=(2,3,4,5,7),context_dim=32,dropout=.1):
        self.widths=tuple(widths); feat=2*channels*len(widths)
        super().__init__(feat,context_dim,dropout)
        self.embedding=nn.Embedding(VOCAB,embed_dim,padding_idx=PAD)
        self.convs=nn.ModuleList(nn.Conv1d(embed_dim,channels,k) for k in self.widths)
    def encode(self,tokens,lengths):
        x=self.embedding(tokens).transpose(1,2); pooled=[]
        for conv,k in zip(self.convs,self.widths):
            y=torch.tanh(conv(x)); steps=y.shape[-1]
            valid=(lengths-k+1).clamp(min=1,max=steps)
            mask=torch.arange(steps,device=y.device)[None,:] < valid[:,None]
            mean=(y*mask[:,None,:]).sum(-1)/valid[:,None]
            maxv=y.masked_fill(~mask[:,None,:],-1e9).max(-1).values
            pooled.extend((mean,maxv))
        return torch.cat(pooled,-1)
    def forward(self,tokens,lengths,context): return self.finish(self.encode(tokens,lengths),context)

class RnnRanker(RankerBase):
    def __init__(self,embed_dim=64,hidden=64,layers=1,context_dim=32,dropout=.1):
        feat=2*hidden
        super().__init__(feat,context_dim,dropout)
        self.embedding=nn.Embedding(VOCAB,embed_dim,padding_idx=PAD)
        self.gru=nn.GRU(embed_dim,hidden,layers,batch_first=True,dropout=dropout if layers>1 else 0)
    def encode(self,tokens,lengths):
        x=self.embedding(tokens); y,_=self.gru(x); steps=y.shape[1]
        mask=torch.arange(steps,device=y.device)[None,:] < lengths[:,None]
        mean=(y*mask[:,:,None]).sum(1)/lengths[:,None]
        maxv=y.masked_fill(~mask[:,:,None],-1e9).max(1).values
        return torch.cat((mean,maxv),-1)
    def forward(self,tokens,lengths,context): return self.finish(self.encode(tokens,lengths),context)

def evaluate(model,loader,device):
    model.eval(); total=correct=0; margins=[]; groups=defaultdict(lambda:[0,0])
    with torch.no_grad():
        for p,pl,n,nl,c,tiers,axes in loader:
            p,pl,n,nl,c=[x.to(device) for x in (p,pl,n,nl,c)]
            m=model(p,pl,c)-model(n,nl,c); ok=m>0
            total+=len(m); correct+=int(ok.sum()); margins.extend(m.cpu().tolist())
            for i,(tier,axis) in enumerate(zip(tiers,axes)):
                groups[f"tier:{tier}"][1]+=1; groups[f"tier:{tier}"][0]+=int(ok[i])
                groups[f"axis:{axis}"][1]+=1; groups[f"axis:{axis}"][0]+=int(ok[i])
    return {"accuracy":correct/max(total,1),"mean_margin":sum(margins)/max(len(margins),1),"n":total,"breakdown":{k:v[0]/v[1] for k,v in sorted(groups.items())}}

def run_training(model,train_path,dev_path,test_path,output,epochs,batch_size,lr,seed,device,max_records=None):
    seed_all(seed); device=torch.device(device)
    train=PairDataset(train_path,max_records); dev=PairDataset(dev_path,max_records); test=PairDataset(test_path,max_records)
    loaders=[DataLoader(ds,batch_size=batch_size,shuffle=(i==0),collate_fn=collate,num_workers=0) for i,ds in enumerate((train,dev,test))]
    model.to(device); opt=torch.optim.AdamW(model.parameters(),lr=lr,weight_decay=1e-4)
    history=[]
    for epoch in range(1,epochs+1):
        model.train(); loss_sum=n=0
        for p,pl,neg,nl,c,_,_ in loaders[0]:
            p,pl,neg,nl,c=[x.to(device) for x in (p,pl,neg,nl,c)]
            margin=model(p,pl,c)-model(neg,nl,c); loss=F.softplus(-margin).mean()
            opt.zero_grad(set_to_none=True); loss.backward(); nn.utils.clip_grad_norm_(model.parameters(),2.0); opt.step()
            loss_sum+=float(loss.detach())*len(p); n+=len(p)
        metrics=evaluate(model,loaders[1],device); metrics.update(epoch=epoch,train_loss=loss_sum/max(n,1)); history.append(metrics)
        print(json.dumps(metrics,sort_keys=True))
    final={"dev":evaluate(model,loaders[1],device),"test":evaluate(model,loaders[2],device),"history":history}
    output.parent.mkdir(parents=True,exist_ok=True); torch.save({"state_dict":model.state_dict(),"metrics":final},output)
    output.with_suffix(output.suffix+".metrics.json").write_text(json.dumps(final,indent=2,sort_keys=True)+"\n")
    return final
