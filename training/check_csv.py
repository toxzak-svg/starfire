import csv
from pathlib import Path

input_file = Path.home() / "Downloads" / "LLM Research.csv"

with open(input_file, 'r', encoding='utf-8', errors='replace') as f:
    reader = csv.DictReader(f)
    
    headers = reader.fieldnames
    print(f"Columns: {headers}")
    print(f"\nFirst 3 rows:")
    
    for i, row in enumerate(reader):
        if i >= 3:
            break
        print(f"\n--- Row {i+1} ---")
        for k, v in row.items():
            if v and len(str(v)) > 0:
                print(f"  {k}: {str(v)[:150]}")
