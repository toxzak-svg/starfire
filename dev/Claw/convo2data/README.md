# Conversation Dataset

Personal conversation dataset for training/ fine-tuning.

## Files

```
conversation-data/
├── dataset_builder.py   # Core library
├── add_convo.py         # Add conversations
├── raw_conversations.jsonl   # Original messages
├── cleaned_conversations.jsonl   # Spell/grammar checked
└── metadata.json
```

## Usage

### Add a conversation
```bash
python add_convo.py "Your message" "Assistant response"
```

### Check stats
```bash
python add_convo.py
```

### Use in Python
```python
from dataset_builder import add_conversation, get_stats

# Add a conversation
add_conversation("Hello", "Hi there!")

# Get stats
stats = get_stats()
print(stats)
```

## What it does

1. **Spell checking** - Fixes common misspellings (thats → that's, im → I'm, etc.)
2. **Grammar fixing** - Basic grammar fixes (double negatives, subject-verb)
3. **Normalization** - Fixes multiple spaces, proper capitalization
4. **Preserves originals** - Keeps both raw and cleaned versions

## For Dataset Creation

The `cleaned_conversations.jsonl` file is ready for:
- Fine-tuning training
- Context building
- Personal knowledge base

Each line is a JSON object with:
```json
{
  "timestamp": "2026-03-17T22:30:00",
  "user": "Your cleaned message",
  "assistant": "Cleaned response",
  "user_original": "Original message",
  "assistant_original": "Original response"
}
```
