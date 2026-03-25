import json, re

raw = open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)

for cell in nb['cells']:
    if cell['id'] != 'dagger-infra':
        continue
    lines = cell['source']
    new_lines = []
    added = False
    for i, line in enumerate(lines):
        if not added and line.strip() and not line.startswith('#') and 'class DAggerReplayBuffer' in line:
            # Add _O2I before the DAggerReplayBuffer class
            new_lines.append('# Map oracle action names to routing indices\n')
            new_lines.append('_O2I = {\'SKIP\': 0, \'READ\': 1, \'WRITE\': 2, \'PERCEIVE\': 0}\n')
            new_lines.append('\n')
            added = True
        new_lines.append(line)
    if added:
        cell['source'] = new_lines
        print('Added _O2I module-level constant')
    else:
        print('Could not find insertion point')

with open('C:/dev/notebooks/SRMoE_Training.ipynb', 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print('Saved')
