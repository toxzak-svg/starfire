import json, re

raw = open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)
for cell in nb['cells']:
    if cell['id'] == 'dagger-infra':
        src = ''.join(cell['source'])
        old = "_O2I = {'SKIP': 0, 'READ': 1, 'WRITE': 2, 'PERCEIVE': 0}"
        new = "_O2I = {'SKIP': 0, 'READ': 1, 'WRITE': 2, 'PERCEIVE': 3}"
        if old in src:
            src = src.replace(old, new)
            cell['source'] = [src]
            print('Fixed PERCEIVE->3')
        else:
            print('Could not find _O2I with PERCEIVE:0, checking current...')
            idx = src.find("_O2I")
            print(src[idx-5:idx+80])
with open('C:/dev/notebooks/SRMoE_Training.ipynb', 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print('Saved')
