import json, re
raw = open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)
for cell in nb['cells']:
    if cell['id'] == '0397ebd3':
        src = ''.join(cell['source'])
        old = "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy', 'typing_extensions', '--upgrade', '-q'], check=True)"
        new = "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy<2.0', 'typing_extensions', '--upgrade', '-q'], check=True)"
        if old in src:
            new_src = src.replace(old, new)
            cell['source'] = [new_src]
            print('Fixed numpy pin')
        else:
            print('Could not find target string')
            for l in cell['source']:
                if 'pip' in l:
                    print(repr(l))
with open('C:/dev/notebooks/SRMoE_Training.ipynb', 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print('Saved')
