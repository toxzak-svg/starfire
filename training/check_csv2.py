import csv
from pathlib import Path
from collections import Counter

input_file = Path.home() / "Downloads" / "LLM Research.csv"

prompt_count = 0
non_empty_prompts = []

with open(input_file, 'r', encoding='utf-8', errors='replace') as f:
    reader = csv.DictReader(f)
    
    for i, row in enumerate(reader):
        prompt = row.get('prompt_1_input', '').strip()
        response = row.get('prompt_1_output', '').strip()
        if prompt:
            prompt_count += 1
            if len(non_empty_prompts) < 3:
                non_empty_prompts.append((prompt[:200], response[:200] if response else 'EMPTY'))

print(f"Total rows: {i+1}")
print(f"Rows with prompts: {prompt_count}")
for p, r in non_empty_prompts:
    print(f"\nPrompt: {p}")
    print(f"Response: {r}")
