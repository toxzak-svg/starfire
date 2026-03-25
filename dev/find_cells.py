import json
nb = json.load(open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8'))
for i, c in enumerate(nb['cells']):
    if c['id'] in ['0397ebd3', '7b05f1f6']:
        src = ''.join(c['source'])
        print(f'=== Cell {i} (id={c["id"]}) ===')
        print(src[:600])
        print()
