"""Private Kaggle GPU trainer for a Rust-compatible Starfire CharRNN candidate."""
import hashlib, json, math, random, struct
from pathlib import Path
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import DataLoader, Dataset

SEED=20260722; VOCAB, EMBED, HIDDEN, LAYERS=227,64,256,2
SEQ,BATCH,EPOCHS,LR=128,64,30,3e-4
EOS,PAD,UNK=0,1,2
VOCABULARY_CONTRACT='starfire_rust_vocabulary_v1'
INPUT_ROOT=Path('/kaggle/input')
OUT=Path('/kaggle/working'); random.seed(SEED); torch.manual_seed(SEED); torch.cuda.manual_seed_all(SEED)
torch.backends.cudnn.deterministic=True; torch.backends.cudnn.benchmark=False
DEVICE=torch.device('cuda' if torch.cuda.is_available() else 'cpu')

def encode(text):
    """Exactly match lib/language_model/vocabulary.rs.

    The first v30 attempt used ``ord(c)-30`` and EOS=PAD=1.  That file can
    load in Rust but cannot be evaluated or promoted as a runtime reranker:
    Rust maps printable ASCII to 3..97, newline to 98, extended bytes to
    99..226, and keeps EOS/PAD/UNK distinct at 0/1/2.
    """
    out=[]
    for c in text:
        code=ord(c)
        if 32<=code<=126: out.append(code-29)
        elif c=='\n': out.append(98)
        elif 128<=code<=255: out.append(code-29)
        else: out.append(UNK)
    return out

def assert_vocabulary_contract():
    fixtures={' ':3, '~':97, '\n':98, chr(128):99, chr(255):226}
    for char, expected in fixtures.items():
        actual=encode(char)[0]
        if actual != expected: raise AssertionError(f'Rust vocabulary mismatch for {char!r}: {actual} != {expected}')
def documents(text):
    d=[x.strip() for x in text.replace('\r\n','\n').split('\n\n') if len(x.strip())>=64]
    random.Random(SEED).shuffle(d); cut=max(1,int(.95*len(d))); return d[:cut],d[cut:]
class Windows(Dataset):
    def __init__(self, docs):
        v=[]
        for d in docs: v.extend(encode(d)); v.append(EOS)
        self.v=torch.tensor(v); self.s=list(range(0,len(v)-SEQ-1,SEQ))
        if not self.s: raise ValueError('corpus does not contain a full window')
    def __len__(self): return len(self.s)
    def __getitem__(self,i):
        x=self.v[self.s[i]:self.s[i]+SEQ+1]; return x[:-1],x[1:]
class Cell(nn.Module):
    def __init__(self, inp):
        super().__init__(); total=inp+HIDDEN; scale=math.sqrt(2/total)
        for n,b in [('i',0),('f',1),('c',0),('o',0)]:
            setattr(self,n+'_weight',nn.Parameter(torch.randn(HIDDEN,total)*scale)); setattr(self,n+'_bias',nn.Parameter(torch.full((HIDDEN,),float(b))))
    def forward(self,x,h,c):
        z=torch.cat([x,h],-1); i=torch.sigmoid(F.linear(z,self.i_weight,self.i_bias)); f=torch.sigmoid(F.linear(z,self.f_weight,self.f_bias)); g=torch.tanh(F.linear(z,self.c_weight,self.c_bias)); o=torch.sigmoid(F.linear(z,self.o_weight,self.o_bias)); c=f*c+i*g; return o*torch.tanh(c),c
class CharRNN(nn.Module):
    def __init__(self):
        super().__init__(); self.embedding=nn.Embedding(VOCAB,EMBED); self.cells=nn.ModuleList([Cell(EMBED),Cell(HIDDEN)]); self.output_weight=nn.Parameter(torch.randn(VOCAB,HIDDEN)*.1); self.output_bias=nn.Parameter(torch.zeros(VOCAB))
    def forward(self,tokens):
        h=[torch.zeros(tokens.shape[0],HIDDEN,device=tokens.device) for _ in range(LAYERS)]; c=[torch.zeros_like(h[0]) for _ in range(LAYERS)]; out=[]
        for x in self.embedding(tokens).unbind(1):
            for i,cell in enumerate(self.cells): h[i],c[i]=cell(x,h[i],c[i]); x=h[i]
            out.append(F.linear(x,self.output_weight,self.output_bias))
        return torch.stack(out,1)
def run_epoch(model,loader,opt=None):
    model.train(opt is not None); total=count=0
    for x,y in loader:
        x,y=x.to(DEVICE),y.to(DEVICE)
        with torch.set_grad_enabled(opt is not None):
            logits=model(x); mask=y.ne(PAD); loss=F.cross_entropy(logits[mask],y[mask])
            if not torch.isfinite(loss): raise RuntimeError('non-finite loss; reject candidate')
            if opt:
                opt.zero_grad(set_to_none=True); loss.backward(); torch.nn.utils.clip_grad_norm_(model.parameters(),1.0); opt.step()
        total+=loss.item()*mask.sum().item(); count+=mask.sum().item()
    return total/count
def vec(f,t):
    x=t.detach().cpu().contiguous().view(-1).tolist(); f.write(struct.pack('<Q',len(x))); f.write(struct.pack(f'<{len(x)}f',*x))
def export(model,path):
    with open(path,'wb') as f:
        f.write(struct.pack('<IIIIf',VOCAB,EMBED,HIDDEN,LAYERS,0.0)); vec(f,model.embedding.weight)
        for c in model.cells:
            for n in ('i_weight','i_bias','f_weight','f_bias','c_weight','c_bias','o_weight','o_bias'): vec(f,getattr(c,n))
        vec(f,model.output_weight.t()); vec(f,model.output_bias)

assert_vocabulary_contract()
matches=list(INPUT_ROOT.rglob('star_corpus_v1.txt'))
if len(matches) != 1:
    available=[str(path.relative_to(INPUT_ROOT)) for path in INPUT_ROOT.rglob('*') if path.is_file()]
    raise FileNotFoundError(f'expected one star_corpus_v1.txt under {INPUT_ROOT}; found {matches}; available={available}')
corpus_bytes=matches[0].read_bytes()
corpus_text=corpus_bytes.decode('utf-8',errors='replace')
train_docs,val_docs=documents(corpus_text)
train=DataLoader(Windows(train_docs),batch_size=BATCH,shuffle=True,num_workers=2,pin_memory=True); val=DataLoader(Windows(val_docs),batch_size=BATCH,num_workers=2,pin_memory=True)
model=CharRNN().to(DEVICE); opt=torch.optim.AdamW(model.parameters(),lr=LR,weight_decay=.01); best=float('inf'); best_state=None; patience=0; history=[]
for epoch in range(1,EPOCHS+1):
    train_nll=run_epoch(model,train,opt); val_nll=run_epoch(model,val); row={'epoch':epoch,'train_nll':train_nll,'val_nll':val_nll,'val_ppl':math.exp(val_nll)}; history.append(row); print(row,flush=True)
    if val_nll < best-1e-4: best=val_nll; best_state={k:v.detach().cpu().clone() for k,v in model.state_dict().items()}; patience=0
    else: patience+=1
    if patience>=4: break
if best_state is None or not all(torch.isfinite(p).all() for p in model.parameters()): raise RuntimeError('candidate rejected')
model.load_state_dict(best_state); checkpoint=OUT/'starfire_personal_reranker_v30.bin'; export(model,checkpoint)
metrics={
    'status':'candidate_only',
    'device':str(DEVICE),
    'seed':SEED,
    'parameters':sum(p.numel() for p in model.parameters()),
    'best_val_nll':best,
    'best_val_ppl':math.exp(best),
    'checkpoint':checkpoint.name,
    'checkpoint_sha256':hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
    'rust_format':'CharRNN::save compatible v1',
    'vocabulary_contract':VOCABULARY_CONTRACT,
    'corpus_sha256':hashlib.sha256(corpus_bytes).hexdigest(),
    'split':{'document_separator':'\\n\\n','minimum_document_chars':64,'shuffle_seed':SEED,'development_fraction':0.95,'sequence_length':SEQ,'pad_token':PAD,'eos_token':EOS},
    'promotion_rule':'Rust load/smoke, reconstructed held-out metrics, and blinded human review; never auto-replace',
}
(OUT/'metrics.json').write_text(json.dumps(metrics,indent=2)); (OUT/'samples.json').write_text(json.dumps(history,indent=2)); print(json.dumps(metrics,indent=2))
