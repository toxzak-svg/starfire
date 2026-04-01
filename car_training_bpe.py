# Cell X: Load BPE data and train CAR
import json, time, math, traceback, os
import torch
import torch.nn as nn
from torch.optim import AdamW
from tokenizers import Tokenizer

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
print(f"Device: {device}")

# --- Load tokenizer and data ---
tokenizer = Tokenizer.from_file("bpe_tokenizer.json")
VOCAB_SIZE = tokenizer.get_vocab_size()
print(f"Vocabulary: {VOCAB_SIZE} tokens")

# Reload and re-encode (in case notebook restarted)
import requests
text = requests.get("https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt").text
encoded = tokenizer.encode(text)
SEQ_LEN = 128
split = int(len(encoded.ids) * 0.9)
train_ids = encoded.ids[:split]
val_ids = encoded.ids[split:]

def chunk_ids(ids, seq_len):
    chunks = []
    for i in range(0, len(ids) - seq_len, seq_len):
        chunks.append(torch.tensor(ids[i : i + seq_len], dtype=torch.long))
    return chunks

train_data = chunk_ids(train_ids, SEQ_LEN)
val_data = chunk_ids(val_ids, SEQ_LEN)
print(f"Train: {len(train_data):,} | Val: {len(val_data):,}")

# --- Model config (same as before, but with correct VOCAB_SIZE) ---
SCALE = "small"
SCALES = {
    "tiny":   dict(d_model=128, n_layers=4, d_state=32),
    "small":  dict(d_model=256, n_layers=4, d_state=32),
    "medium": dict(d_model=512, n_layers=8, d_state=64),
}
cfg = SCALES[SCALE]

BATCH_SIZE = 8
GRAD_ACCUM = 4
MAX_STEPS = 20000      # ← run longer now that data is fixed
EVAL_EVERY = 500
SAVE_EVERY = 2000
LR = 3e-4
THRESHOLD = 1.0        # adjust after first run

# --- Model classes (same as cell 4) ---
class RMSNorm(nn.Module):
    def __init__(self, d, eps=1e-6):
        super().__init__()
        self.scale = nn.Parameter(torch.ones(d))
        self.eps = eps
    def forward(self, x):
        norm = x.pow(2).mean(-1, keepdim=True).add(self.eps).rsqrt()
        return x * norm * self.scale

class SSMLayer(nn.Module):
    def __init__(self, d_model, d_state=64, expand=2):
        super().__init__()
        d_inner = d_model * expand
        self.proj = nn.Sequential(
            nn.Linear(d_model, d_inner),
            nn.Tanh(),
            nn.Linear(d_inner, d_state),
        )
        self.out = nn.Linear(d_state, d_model)
        self.h = None
    def forward(self, x):
        B, T, D = x.shape
        state = self.proj(x)
        h_new = state.mean(dim=1)
        if self.h is not None:
            h_new = 0.9 * self.h + 0.1 * h_new
        self.h = h_new.detach()
        out = self.out(h_new).unsqueeze(1).expand(-1, T, -1)
        return out

class CheapPath(nn.Module):
    def __init__(self, d_model, ratio=4):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(d_model, d_model // ratio),
            nn.GELU(),
            nn.Linear(d_model // ratio, d_model),
        )
    def forward(self, x):
        return self.net(x)

class CARState(nn.Module):
    def __init__(self, d_model, d_state=64):
        super().__init__()
        self.update = nn.GRUCell(d_model, d_state)
        self.predict = nn.Sequential(
            nn.Linear(d_state, d_state * 2),
            nn.Tanh(),
            nn.Linear(d_state * 2, d_model),
        )
        self.d_state = d_state
    def init_state(self, B, device, dtype):
        return torch.zeros(B, self.d_state, device=device, dtype=dtype)
    def forward(self, x, h):
        summary = x.mean(dim=1)
        h_new = self.update(summary, h)
        pred = self.predict(h_new)
        return h_new, pred

class CARModel(nn.Module):
    def __init__(self, vocab_size, d_model=256, n_layers=4, d_state=64,
                 cheap_ratio=4, threshold=1.0):
        super().__init__()
        self.threshold = threshold
        self.embed = nn.Embedding(vocab_size, d_model)
        self.layers = nn.ModuleList([
            nn.ModuleDict({
                "ssm":   SSMLayer(d_model, d_state),
                "cheap": CheapPath(d_model, cheap_ratio),
                "norm":  RMSNorm(d_model),
            })
            for _ in range(n_layers)
        ])
        self.car_state = CARState(d_model, d_state)
        self.norm_f = RMSNorm(d_model)
        self.lm_head = nn.Linear(d_model, vocab_size, bias=False)
        self.lm_head.weight = self.embed.weight

    def forward(self, input_ids, targets=None):
        B, T = input_ids.shape
        x = self.embed(input_ids)
        h = self.car_state.init_state(B, x.device, x.dtype)
        cheap_ratios = []
        for layer in self.layers:
            xn = layer["norm"](x)
            h, pred = self.car_state(xn, h)
            with torch.no_grad():
                pred_expanded = pred.unsqueeze(1).expand(-1, T, -1)
                error = (pred_expanded - xn).pow(2).mean(-1).mean(-1)
            cheap_w = torch.sigmoid(self.threshold - error)
            cheap_ratios.append(cheap_w.mean().item())
            ssm_out   = layer["ssm"](xn)
            cheap_out = layer["cheap"](xn)
            cw = cheap_w.view(B, 1, 1)
            x = x + cw * cheap_out + (1 - cw) * ssm_out
        x = self.norm_f(x)
        logits = self.lm_head(x)
        result = {"logits": logits, "cheap_ratio": sum(cheap_ratios)/max(len(cheap_ratios),1)}
        if targets is not None:
            result["loss"] = nn.functional.cross_entropy(
                logits.view(-1, logits.size(-1)),
                targets.view(-1))
        return result
    def param_count(self):
        return sum(p.numel() for p in self.parameters())

# --- Build model ---
model = CARModel(
    vocab_size=VOCAB_SIZE,
    d_model=cfg["d_model"],
    n_layers=cfg["n_layers"],
    d_state=cfg["d_state"],
    cheap_ratio=4,
    threshold=THRESHOLD,
).to(device)

n_params = model.param_count() / 1e6
print(f"Model: {n_params:.2f}M params | Vocab: {VOCAB_SIZE}")

# Quick forward test
model.eval()
with torch.no_grad():
    test_ids = torch.randint(0, VOCAB_SIZE, (2, SEQ_LEN), device=device)
    out = model(test_ids)
    print(f"Test forward: logits={out['logits'].shape} | loss={out['loss'].item():.4f}")
model.train()

# --- Train ---
optimizer = AdamW(model.parameters(), lr=LR, weight_decay=0.1, betas=(0.9, 0.95))

def sched(step):
    w = 100
    if step < w:
        return max(0.1, step/w)
    p = (step - w) / max(MAX_STEPS - w, 1)
    return max(1e-5/LR, 0.5 * (1 + math.cos(math.pi * p)))

scheduler = torch.optim.lr_scheduler.LambdaLR(optimizer, sched)

print(f"""
============================================================
  CAR Training | {SCALE} | {n_params:.2f}M params
  Data: tiny_shakespeare → BPE | Vocab: {VOCAB_SIZE}
  Batch: {BATCH_SIZE}×{GRAD_ACCUM} | Steps: {MAX_STEPS:,}
  Cheap threshold: {THRESHOLD}
============================================================
""")

# Local CSV logger
import csv
csv_path = "car_log.csv"
csv_file = open(csv_path, "w", newline="")
csv_writer = csv.writer(csv_file)
csv_writer.writerow(["step", "loss", "cheap_ratio", "lr", "tok/s", "val_loss"])

step, accum, t0, total_tokens = 0, 0, time.time(), 0
model.train()

print(f"{'step':>6} | {'loss':>8} | {'cheap':>6} | {'lr':>9} | {'tok/s':>8}")
print("-" * 65)

try:
    while step < MAX_STEPS:
        for seq in train_data:
            if step >= MAX_STEPS:
                break
            ids = seq.unsqueeze(0).to(device)
            tgt = seq.unsqueeze(0).to(device)

            r = model(ids, tgt)
            loss = r["loss"] / GRAD_ACCUM
            loss.backward()
            accum += 1
            cheap = r["cheap_ratio"]

            if accum >= GRAD_ACCUM:
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                optimizer.step()
                scheduler.step()
                optimizer.zero_grad()
                accum = 0

                total_tokens += ids.numel() * GRAD_ACCUM
                dt = time.time() - t0
                tok_s = total_tokens / max(dt, 0.1)
                lr_now = scheduler.get_last_lr()[0]

                if step % 10 == 0:
                    print(f"{step:>6} | {loss.item()*GRAD_ACCUM:>8.4f} | "
                          f"{cheap:>6.2%} | {lr_now:>9.2e} | {tok_s:>8.0f}")
                    csv_writer.writerow([step, loss.item()*GRAD_ACCUM, cheap, lr_now, tok_s, ""])
                    csv_file.flush()

                if step > 0 and step % EVAL_EVERY == 0:
                    model.eval()
                    vl = []
                    with torch.no_grad():
                        for s in val_data[:50]:
                            iv = s.unsqueeze(0).to(device)
                            tv = s.unsqueeze(0).to(device)
                            vl.append(model(iv, tv)["loss"].item())
                    vloss = sum(vl) / len(vl)
                    
                    # Update last row with val_loss
                    print(f"  [EVAL @{step:>6}] val={vloss:.4f}")
                    csv_writer.writerow([step, loss.item()*GRAD_ACCUM, cheap, lr_now, tok_s, vloss])
                    csv_file.flush()
                    model.train()

                if step > 0 and step % SAVE_EVERY == 0:
                    ckpt = f"car-bpe-{SCALE}-s{step}.pt"
                    torch.save({"step": step, "state": model.state_dict()}, ckpt)
                    print(f"  [SAVE @{step}] → {ckpt}")

                step += 1

    # Final save
    torch.save({"step": step, "state": model.state_dict()}, f"car-bpe-{SCALE}-final.pt")
    csv_file.close()
    print(f"\n✅ Done! {total_tokens/1e6:.1f}M tokens in {time.time()-t0:.0f}s")
    print(f"📊 Log saved to {csv_path}")

except Exception as e:
    print(f"\n❌ CRASH @{step}: {e}")
    traceback.print_exc()
    csv_file.close()
    try:
        torch.save({"step": step, "state": model.state_dict()}, f"car-bpe-{SCALE}-crash.pt")
    except:
        pass
    raise
