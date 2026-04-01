# Cell X: BPE Tokenizer Setup — run once, replaces cell 6
# Install tokenizers (BPE training library)
import subprocess, sys
subprocess.check_call([sys.executable, "-m", "pip", "install", "--quiet", "tokenizers"])
print("✅ tokenizers installed")

import os
from tokenizers import Tokenizer, models, trainers, pre_tokenizers
import requests

# --- 1. Download tiny_shakespeare ---
data_url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt"
data_path = "tiny_shakespeare.txt"

if not os.path.exists(data_path):
    print("Downloading tiny_shakespeare...")
    text = requests.get(data_url).text
    with open(data_path, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"✅ Downloaded ({len(text):,} chars)")
else:
    with open(data_path, "r", encoding="utf-8") as f:
        text = f.read()
    print(f"✅ Loaded from disk ({len(text):,} chars)")

# --- 2. Train BPE tokenizer ---
VOCAB_SIZE = 4096

tokenizer = Tokenizer(models.BPE(unk_token="[UNK]"))
trainer = trainers.BpeTrainer(
    vocab_size=VOCAB_SIZE,
    special_tokens=["[PAD]", "[UNK]", "[BOS]", "[EOS]"],
    min_frequency=2,
)

# Pre-tokenize: split on whitespace + punctuation (byte-level fallback)
tokenizer.pre_tokenizer = pre_tokenizers.ByteLevel(add_prefix_space=False)

print(f"Training BPE tokenizer (vocab={VOCAB_SIZE})...")
tokenizer.train_from_iterator([text], trainer=trainer)
print("✅ Tokenizer trained")

# Save tokenizer for later use
tokenizer.save("bpe_tokenizer.json")

# --- 3. Encode the full dataset ---
encoded = tokenizer.encode(text)
print(f"✅ Encoded: {len(encoded.ids):,} tokens | vocab={tokenizer.get_vocab_size()}")

# --- 4. Split into train/val ---
SEQ_LEN = 128
split = int(len(encoded.ids) * 0.9)
train_ids = encoded.ids[:split]
val_ids = encoded.ids[split:]

# Chunk into sequences of SEQ_LEN
def chunk_ids(ids, seq_len):
    chunks = []
    for i in range(0, len(ids) - seq_len, seq_len):
        chunks.append(ids[i : i + seq_len])
    return chunks

train_chunks = chunk_ids(train_ids, SEQ_LEN)
val_chunks = chunk_ids(val_ids, SEQ_LEN)

print(f"Train: {len(train_chunks):,} sequences | Val: {len(val_chunks):,} sequences")
print(f"✅ BPE setup complete! VOCAB_SIZE={VOCAB_SIZE}, SEQ_LEN={SEQ_LEN}")
