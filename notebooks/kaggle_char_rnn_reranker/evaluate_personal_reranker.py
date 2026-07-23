"""Held-out and blinded-review gate for a Starfire CharRNN reranker candidate.

Run this after `train_personal_reranker.py` in Kaggle.  It never promotes a
model: it reconstructs the frozen document holdout, compares a candidate with a
baseline, writes an anonymous review packet, and writes the candidate-label key
to a separate file that must not be shown to reviewers.
"""
import argparse, hashlib, json, math, random, struct
from pathlib import Path

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import DataLoader, Dataset

SEED, VOCAB, EMBED, HIDDEN, LAYERS, SEQ, PAD, EOS, UNK = 20260722, 227, 64, 256, 2, 128, 1, 0, 2
VOCABULARY_CONTRACT = 'starfire_rust_vocabulary_v1'

def encode(text):
    out=[]
    for char in text:
        code=ord(char)
        if 32 <= code <= 126: out.append(code-29)
        elif char == '\n': out.append(98)
        elif 128 <= code <= 255: out.append(code-29)
        else: out.append(UNK)
    return out

def decode(tokens):
    out=[]
    for token in tokens:
        if 3 <= token <= 97: out.append(chr(token+29))
        elif token == 98: out.append('\n')
        elif 99 <= token <= 226: out.append(chr(token+29))
        elif token == UNK: out.append('�')
    return ''.join(out)

def documents(text):
    docs=[x.strip() for x in text.replace('\r\n','\n').split('\n\n') if len(x.strip()) >= 64]
    random.Random(SEED).shuffle(docs)
    cut=max(1, int(.95*len(docs)))
    return docs[:cut], docs[cut:]

class Windows(Dataset):
    def __init__(self, docs):
        tokens=[]
        for doc in docs: tokens.extend(encode(doc)); tokens.append(EOS)
        self.tokens=torch.tensor(tokens, dtype=torch.long)
        self.starts=list(range(0, len(tokens)-SEQ-1, SEQ))
        if not self.starts: raise ValueError('holdout corpus has no complete windows')
    def __len__(self): return len(self.starts)
    def __getitem__(self, index):
        window=self.tokens[self.starts[index]:self.starts[index]+SEQ+1]
        return window[:-1], window[1:]

class Cell(nn.Module):
    def __init__(self, inp):
        super().__init__(); total=inp+HIDDEN
        for name in ('i','f','c','o'):
            setattr(self, name+'_weight', nn.Parameter(torch.empty(HIDDEN,total)))
            setattr(self, name+'_bias', nn.Parameter(torch.empty(HIDDEN)))
    def forward(self,x,h,c):
        z=torch.cat([x,h],-1)
        i=torch.sigmoid(F.linear(z,self.i_weight,self.i_bias)); f=torch.sigmoid(F.linear(z,self.f_weight,self.f_bias))
        g=torch.tanh(F.linear(z,self.c_weight,self.c_bias)); o=torch.sigmoid(F.linear(z,self.o_weight,self.o_bias))
        c=f*c+i*g; return o*torch.tanh(c),c

class CharRNN(nn.Module):
    def __init__(self):
        super().__init__(); self.embedding=nn.Embedding(VOCAB,EMBED); self.cells=nn.ModuleList([Cell(EMBED),Cell(HIDDEN)])
        self.output_weight=nn.Parameter(torch.empty(VOCAB,HIDDEN)); self.output_bias=nn.Parameter(torch.empty(VOCAB))
    def forward(self,tokens):
        h=[torch.zeros(tokens.shape[0],HIDDEN,device=tokens.device) for _ in range(LAYERS)]; c=[torch.zeros_like(h[0]) for _ in range(LAYERS)]; out=[]
        for x in self.embedding(tokens).unbind(1):
            for index, cell in enumerate(self.cells): h[index],c[index]=cell(x,h[index],c[index]); x=h[index]
            out.append(F.linear(x,self.output_weight,self.output_bias))
        return torch.stack(out,1)

def read_tensor(handle, shape):
    (count,)=struct.unpack('<Q',handle.read(8)); expected=math.prod(shape)
    if count != expected: raise ValueError(f'checkpoint tensor length {count} != expected {expected}')
    values=struct.unpack(f'<{count}f',handle.read(count*4))
    return torch.tensor(values,dtype=torch.float32).reshape(shape)

def load_checkpoint(path):
    with open(path,'rb') as handle:
        vocab,embed,hidden,layers,dropout=struct.unpack('<IIIIf',handle.read(20))
        if (vocab,embed,hidden,layers) != (VOCAB,EMBED,HIDDEN,LAYERS):
            raise ValueError(f'incompatible checkpoint header {(vocab,embed,hidden,layers)}')
        model=CharRNN(); model.embedding.weight.data.copy_(read_tensor(handle,(VOCAB,EMBED)))
        for layer, inp in zip(model.cells,(EMBED,HIDDEN)):
            for name,shape in ((n+'_weight',(HIDDEN,inp+HIDDEN)) for n in ('i','f','c','o')):
                getattr(layer,name).data.copy_(read_tensor(handle,shape))
                getattr(layer,name.replace('_weight','_bias')).data.copy_(read_tensor(handle,(HIDDEN,)))
        # Export writes output_weight transposed to match Rust's flat row order.
        model.output_weight.data.copy_(read_tensor(handle,(HIDDEN,VOCAB)).t())
        model.output_bias.data.copy_(read_tensor(handle,(VOCAB,)))
        if handle.read(1): raise ValueError('checkpoint has trailing bytes')
    if not all(torch.isfinite(parameter).all() for parameter in model.parameters()): raise ValueError('checkpoint contains non-finite parameters')
    return model.eval()

@torch.no_grad()
def nll(model, docs, batch_size):
    total=0.0; count=0
    for x,y in DataLoader(Windows(docs),batch_size=batch_size):
        logits=model(x); mask=y.ne(PAD); loss=F.cross_entropy(logits[mask],y[mask],reduction='sum')
        total += loss.item(); count += mask.sum().item()
    return total/count, count

@torch.no_grad()
def generate(model,prompt,seed,max_chars=160,temperature=.7,top_k=20):
    rng=random.Random(seed); tokens=[token for token in encode(prompt) if token < VOCAB]
    if not tokens: return ''
    h=[torch.zeros(1,HIDDEN) for _ in range(LAYERS)]; c=[torch.zeros(1,HIDDEN) for _ in range(LAYERS)]
    def step(token):
        nonlocal h,c
        x=model.embedding(torch.tensor([token]))
        for index,cell in enumerate(model.cells): h[index],c[index]=cell(x,h[index],c[index]); x=h[index]
        return F.linear(x,model.output_weight,model.output_bias).flatten()
    for token in tokens: step(token)
    last=tokens[-1]; generated=[]
    for _ in range(max_chars):
        logits=step(last)/temperature; values,indices=torch.topk(logits,min(top_k,VOCAB)); probabilities=torch.softmax(values,0).tolist()
        draw=rng.random(); cumulative=0.0; next_token=indices[-1].item()
        for probability,index in zip(probabilities,indices.tolist()):
            cumulative += probability
            if draw <= cumulative: next_token=index; break
        if next_token == EOS: break
        generated.append(next_token); last=next_token
    return decode(generated).strip()

def review_cases():
    return [
        ('story_prompt','intent:story style:warm unc:0.20 con:0.40 nov:0.10 cre:0.35 val:0.20 eng:0.60 body:Yes. I\'m listening. Star:'),
        ('uncertainty','intent:consciousness style:warm unc:0.70 con:0.35 nov:0.20 cre:0.30 val:-0.10 eng:0.65 body:I process what you say and may have gaps. What are you wondering about? Star:'),
        ('research','intent:research_status style:direct unc:0.45 con:0.50 nov:0.55 cre:0.45 val:0.10 eng:0.55 topic:causal reasoning body:Causal reasoning keeps coming up. Star:'),
        ('capability','intent:capability style:direct unc:0.20 con:0.40 nov:0.10 cre:0.20 val:0.10 eng:0.45 body:I can inspect local information and explain my limits. Star:'),
    ]

def require_candidate_metrics(path, checkpoint, corpus_hash):
    metrics=json.loads(path.read_text(encoding='utf-8'))
    if metrics.get('vocabulary_contract') != VOCABULARY_CONTRACT: raise ValueError('candidate metrics do not attest to the Rust vocabulary contract')
    if metrics.get('corpus_sha256') != corpus_hash: raise ValueError('candidate metrics were produced from a different corpus')
    if metrics.get('checkpoint_sha256') != hashlib.sha256(checkpoint.read_bytes()).hexdigest(): raise ValueError('candidate metrics do not match checkpoint bytes')
    return metrics

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--corpus',type=Path,required=True); parser.add_argument('--candidate',type=Path,required=True)
    parser.add_argument('--candidate-metrics',type=Path,required=True); parser.add_argument('--baseline',type=Path,required=True)
    parser.add_argument('--out',type=Path,default=Path('/kaggle/working')); parser.add_argument('--batch-size',type=int,default=64)
    args=parser.parse_args(); args.out.mkdir(parents=True,exist_ok=True)
    corpus_bytes=args.corpus.read_bytes(); corpus_hash=hashlib.sha256(corpus_bytes).hexdigest()
    candidate_training_metrics=require_candidate_metrics(args.candidate_metrics,args.candidate,corpus_hash)
    _, holdout=documents(corpus_bytes.decode('utf-8',errors='replace'))
    candidate=load_checkpoint(args.candidate); baseline=load_checkpoint(args.baseline)
    candidate_nll,tokens=nll(candidate,holdout,args.batch_size); baseline_nll,_=nll(baseline,holdout,args.batch_size)
    report={'status':'candidate_only','seed':SEED,'vocabulary_contract':VOCABULARY_CONTRACT,'corpus_sha256':corpus_hash,'holdout_documents':len(holdout),'holdout_tokens':tokens,'candidate':{'sha256':hashlib.sha256(args.candidate.read_bytes()).hexdigest(),'nll':candidate_nll,'ppl':math.exp(candidate_nll)},'baseline':{'sha256':hashlib.sha256(args.baseline.read_bytes()).hexdigest(),'nll':baseline_nll,'ppl':math.exp(baseline_nll)},'candidate_minus_baseline_nll':candidate_nll-baseline_nll,'training_metrics':candidate_training_metrics,'promotion':'blocked_pending_blinded_human_review'}
    (args.out/'reranker_holdout_metrics.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    packet=[]; key=[]; labels=random.Random(SEED).sample(['A','B'],2)
    for index,(case_id,prompt) in enumerate(review_cases()):
        outputs={'candidate':generate(candidate,prompt,SEED+index),'baseline':generate(baseline,prompt,SEED+index)}
        assignment={labels[0]:'candidate',labels[1]:'baseline'} if index%2==0 else {labels[0]:'baseline',labels[1]:'candidate'}
        packet.append({'case_id':case_id,'prompt_context':prompt,'response_a':outputs[assignment['A']],'response_b':outputs[assignment['B']],'rubric':'Choose A, B, or tie. Score relevance, factual preservation, readability, and voice fit 1-5. Do not infer model identity.'})
        key.append({'case_id':case_id,'A':assignment['A'],'B':assignment['B']})
    (args.out/'blinded_review_packet.json').write_text(json.dumps({'status':'review_pending','cases':packet},indent=2),encoding='utf-8')
    (args.out/'blinded_review_key.json').write_text(json.dumps({'DO_NOT_SHARE_WITH_REVIEWERS':key},indent=2),encoding='utf-8')
    print(json.dumps({'heldout_metrics':'reranker_holdout_metrics.json','review_packet':'blinded_review_packet.json','review_key':'blinded_review_key.json','status':'candidate_only'},indent=2))

if __name__ == '__main__': main()
