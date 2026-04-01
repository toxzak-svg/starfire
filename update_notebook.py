import json

with open('/home/zach/.openclaw/workspace/life/car_training.ipynb') as f:
    nb = json.load(f)

# ── Cell 0: Updated header ──
nb['cells'][0]['source'] = """# CAR — Compressibility-Aware Routing

Train a model that routes tokens by *prediction error*. Easy tokens go cheap.
Hard tokens get the full SSM path. Based on the CAR idea from 2026-03-26.

**Key changes (2026-03-31):**
- BPE tokenized tiny_shakespeare instead of random synthetic data
- Proper subword vocabulary (4096 tokens) that matches the model's capacity
- CSV logging (W&B optional) so you never lose training data
- 20K steps for proper convergence on real data
"""

# ── Cell 3: Config ──
cell3 = r"""# Cell 3: Config — change SCALE here
import subprocess, sys

# Install tokenizers library for BPE
subprocess.check_call([sys.executable, "-m", "pip", "install", "--quiet", "tokenizers"])
print("tokenizers installed")

SCALE = "small"  # tiny=128d/4l, small=256d/4l, medium=512d/8l

SCALES = {
    "tiny":   dict(d_model=128, n_layers=4, d_state=32),
    "small":  dict(d_model=256, n_layers=4, d_state=32),
    "medium": dict(d_model=512, n_layers=8, d_state=64),
}

cfg = SCALES[SCALE]

# Data config
USE_BPE = True           # True = BPE tokenized tiny_shakespeare (recommended)
VOCAB_SIZE = 4096        # Must match tokenizer vocab; only used if USE_BPE=True
SEQ_LEN    = 128
BATCH_SIZE = 8
GRAD_ACCUM = 4
MAX_STEPS  = 20000       # run longer for real data
EVAL_EVERY = 500
SAVE_EVERY = 2000
LR         = 3e-4
THRESHOLD  = 1.0         # higher = more tokens go cheap path

print("Scale: %s | d_model=%d | n_layers=%d" % (SCALE, cfg["d_model"], cfg["n_layers"]))
data_str = "BPE tiny_shakespeare" if USE_BPE else "Random synthetic"
print("Data: %s | USE_BPE=%s" % (data_str, USE_BPE))
print("Batch=%dx%d | SEQ=%d | steps=%d" % (BATCH_SIZE, GRAD_ACCUM, SEQ_LEN, MAX_STEPS))
"""
nb['cells'][3]['source'] = cell3

# ── Cell 6: BPE Data ──
cell6 = r"""# Cell 6: Data — BPE tokenized tiny_shakespeare
import os
from tokenizers import Tokenizer, models, trainers, pre_tokenizers
import requests

if USE_BPE:
    # 1. Download tiny_shakespeare
    data_url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt"
    data_path = "tiny_shakespeare.txt"

    if not os.path.exists(data_path):
        print("Downloading tiny_shakespeare...")
        text = requests.get(data_url).text
        with open(data_path, "w", encoding="utf-8") as f:
            f.write(text)
        print("Downloaded (%d chars)" % len(text))
    else:
        with open(data_path, "r", encoding="utf-8") as f:
            text = f.read()
        print("Loaded from disk (%d chars)" % len(text))

    # 2. Train or load BPE tokenizer
    tok_path = "bpe_tokenizer.json"

    if not os.path.exists(tok_path):
        print("Training BPE tokenizer (vocab=%d)..." % VOCAB_SIZE)
        tokenizer = Tokenizer(models.BPE(unk_token="[UNK]"))
        trainer = trainers.BpeTrainer(
            vocab_size=VOCAB_SIZE,
            special_tokens=["[PAD]", "[UNK]", "[BOS]", "[EOS]"],
            min_frequency=2,
        )
        tokenizer.pre_tokenizer = pre_tokenizers.ByteLevel(add_prefix_space=False)
        tokenizer.train_from_iterator([text], trainer=trainer)
        tokenizer.save(tok_path)
        print("Tokenizer trained and saved")
    else:
        tokenizer = Tokenizer.from_file(tok_path)
        print("Tokenizer loaded from %s" % tok_path)

    VOCAB_SIZE = tokenizer.get_vocab_size()
    print("Tokenizer vocab: %d tokens" % VOCAB_SIZE)

    # 3. Encode the full dataset
    encoded = tokenizer.encode(text)
    print("Encoded: %d tokens" % len(encoded.ids))

    # 4. Chunk into sequences
    def chunk_ids(ids, seq_len):
        chunks = []
        for i in range(0, len(ids) - seq_len, seq_len):
            chunks.append(torch.tensor(ids[i : i + seq_len], dtype=torch.long))
        return chunks

    split = int(len(encoded.ids) * 0.9)
    train_data = chunk_ids(encoded.ids[:split], SEQ_LEN)
    val_data   = chunk_ids(encoded.ids[split:], SEQ_LEN)
    print("Train: %d | Val: %d" % (len(train_data), len(val_data)))

else:
    # Fallback: random synthetic data
    torch.manual_seed(42)
    N_TRAIN, N_VAL = 50000, 5000
    train_data = [torch.randint(0, VOCAB_SIZE, (SEQ_LEN,)) for _ in range(N_TRAIN)]
    val_data   = [torch.randint(0, VOCAB_SIZE, (SEQ_LEN,)) for _ in range(N_VAL)]
    print("Random data: Train=%d | Val=%d" % (len(train_data), len(val_data)))

print("Data ready")
"""
nb['cells'][6]['source'] = cell6

# ── Cell 7: Logging setup ──
cell7 = r"""# Cell 7: Logging setup (W&B optional, CSV always on)
import csv

WANDB = False   # Set True to enable W&B (requires valid API key)

csv_path = "car_log.csv"
csv_file = open(csv_path, "w", newline="")
csv_writer = csv.writer(csv_file)
csv_writer.writerow(["step", "train_loss", "cheap_ratio", "lr", "tok_s", "val_loss"])
csv_file.flush()

if WANDB:
    import wandb
    wandb.login()
    run = wandb.init(
        project="neu",
        name="car-bpe-" + SCALE + "-" + time.strftime("%m%d-%H%M"),
        config=dict(scale=SCALE, **cfg, vocab=VOCAB_SIZE, seq=SEQ_LEN,
                    batch=BATCH_SIZE, accum=GRAD_ACCUM, steps=MAX_STEPS,
                    lr=LR, threshold=THRESHOLD, device=str(device)),
        mode="online",
    )
    wandb.define_metric("step")
    wandb.define_metric("val_loss", step_metric="eval_step")
    print("W&B enabled")
else:
    print("CSV logging only (W&B disabled)")

print("Log file: " + csv_path)
"""
nb['cells'][7]['source'] = cell7

# ── Cell 8: Training loop ──
cell8 = r"""# Cell 8: Train!
# Rebuild model now that VOCAB_SIZE is confirmed from tokenizer
model = CARModel(
    vocab_size=VOCAB_SIZE,
    d_model=cfg["d_model"],
    n_layers=cfg["n_layers"],
    d_state=cfg["d_state"],
    cheap_ratio=4,
    threshold=THRESHOLD,
).to(device)

n_params = model.param_count() / 1e6
print("Model: %.2fM params | Vocab: %d" % (n_params, VOCAB_SIZE))

# Quick forward test
model.eval()
with torch.no_grad():
    test_ids = torch.randint(0, VOCAB_SIZE, (2, SEQ_LEN), device=device)
    out = model(test_ids)
    cheap_pct = out["cheap_ratio"] * 100
    print("Test: logits=%s | loss=%.4f | cheap=%.1f%%" % (str(list(out["logits"].shape)), out["loss"].item(), cheap_pct))
model.train()

optimizer = AdamW(model.parameters(), lr=LR, weight_decay=0.1, betas=(0.9, 0.95))

def sched(step):
    w = 100
    if step < w:
        return max(0.1, step/w)
    p = (step - w) / max(MAX_STEPS - w, 1)
    return max(1e-5/LR, 0.5 * (1 + math.cos(math.pi * p)))

scheduler = torch.optim.lr_scheduler.LambdaLR(optimizer, sched)

data_str = "BPE tiny_shakespeare" if USE_BPE else "Random"
n_str = "%.2f" % n_params
print("")
print("=" * 60)
print("  CAR Training | %s | %sM params | %s" % (SCALE, n_str, str(device)))
print("  Data: %s | Vocab: %d" % (data_str, VOCAB_SIZE))
print("  Batch: %dx%d | Steps: %d | Cheap threshold: %.2f" % (BATCH_SIZE, GRAD_ACCUM, MAX_STEPS, THRESHOLD))
print("=" * 60)
print("")

step, accum, t0, total_tokens = 0, 0, time.time(), 0
model.train()

header = "%6s | %8s | %6s | %9s | %8s" % ("step", "loss", "cheap", "lr", "tok/s")
print(header)
print("-" * 60)

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
                    loss_val = loss.item() * GRAD_ACCUM
                    cheap_pct = cheap * 100
                    print("%6d | %8.4f | %6.2f%% | %9.2e | %8.0f" % (step, loss_val, cheap_pct, lr_now, tok_s))
                    row = [step, "%.6f" % loss_val, "%.6f" % cheap, "%.6e" % lr_now, "%.1f" % tok_s, ""]
                    csv_writer.writerow(row)
                    csv_file.flush()
                    if WANDB:
                        wandb.log({"step": step, "loss": loss_val, "cheap_ratio": cheap,
                                   "lr": lr_now, "tokens_per_sec": tok_s}, step=step)

                if step > 0 and step % EVAL_EVERY == 0:
                    model.eval()
                    vl = []
                    with torch.no_grad():
                        for s in val_data[:50]:
                            iv = s.unsqueeze(0).to(device)
                            tv = s.unsqueeze(0).to(device)
                            vl.append(model(iv, tv)["loss"].item())
                    vloss = sum(vl) / len(vl)
                    print("  [EVAL @%6d] val=%.4f" % (step, vloss))
                    row = [step, "%.6f" % (loss.item() * GRAD_ACCUM), "%.6f" % cheap,
                           "%.6e" % lr_now, "%.1f" % tok_s, "%.6f" % vloss]
                    csv_writer.writerow(row)
                    csv_file.flush()
                    if WANDB:
                        wandb.log({"eval_step": step, "val_loss": vloss}, step=step)
                    model.train()

                if step > 0 and step % SAVE_EVERY == 0:
                    ckpt = "car-bpe-%s-s%d.pt" % (SCALE, step)
                    torch.save({"step": step, "state": model.state_dict()}, ckpt)
                    print("  [SAVE @%d] -> %s" % (step, ckpt))

                step += 1

    # Final save
    final_ckpt = "car-bpe-%s-final.pt" % SCALE
    torch.save({"step": step, "state": model.state_dict()}, final_ckpt)
    csv_file.close()
    if WANDB:
        wandb.finish()
    elapsed = time.time() - t0
    print("")
    print("Done! %.1fM tokens in %.0fs" % (total_tokens/1e6, elapsed))
    print("Log: " + csv_path)

except Exception as e:
    print("")
    print("CRASH @%d: %s" % (step, str(e)))
    traceback.print_exc()
    csv_file.close()
    try:
        crash_ckpt = "car-bpe-%s-crash.pt" % SCALE
        torch.save({"step": step, "state": model.state_dict()}, crash_ckpt)
    except:
        pass
    if WANDB:
        wandb.finish()
    raise
"""
nb['cells'][8]['source'] = cell8

# Remove cell 9 (old "What to watch" markdown)
if len(nb['cells']) > 9:
    del nb['cells'][9]

with open('/home/zach/.openclaw/workspace/life/car_training.ipynb', 'w') as f:
    json.dump(nb, f, indent=1)

print("Notebook updated successfully!")
