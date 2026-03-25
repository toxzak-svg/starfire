import json, re

raw = open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)

for cell in nb['cells']:
    if cell['id'] != '0397ebd3':
        continue
    src = ''.join(cell['source'])

    # Replace the entire pip block with a clean, ordered one
    old = (
        "import subprocess, sys\n"
        "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy<2.0', 'typing_extensions', '--upgrade', '-q'], check=True)\n"
        "subprocess.run([sys.executable, '-m', 'pip', 'install', 'wandb==0.16.0', '-q'], check=True)"
    )
    new = (
        "import subprocess, sys\n"
        "# Upgrade typing_extensions FIRST so newer wandb can use TypeIs\n"
        "subprocess.run([sys.executable, '-m', 'pip', 'install', 'typing_extensions', '--upgrade', '-q'], check=True)\n"
        "# Pin numpy to last compatible with torch 2.1.1 (was 2.4.3 — too new)\n"
        "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy==1.26.4', '--force-reinstall', '-q'], check=True)\n"
        "# wandb 0.16.0 needs newer typing_extensions (fixed above); install after\n"
        "subprocess.run([sys.executable, '-m', 'pip', 'install', 'wandb==0.16.0', '-q'], check=True)"
    )

    if old in src:
        new_src = src.replace(old, new)
        cell['source'] = [new_src]
        print("Replaced pip block")
    else:
        print("Could not find old pip block, trying alternate match...")
        # Try each line individually
        lines = cell['source']
        new_lines = []
        replaced = False
        for l in lines:
            if 'numpy<2.0' in l and 'pip' in l:
                new_lines.append("import subprocess, sys\n")
                new_lines.append("# Upgrade typing_extensions FIRST\n")
                new_lines.append("subprocess.run([sys.executable, '-m', 'pip', 'install', 'typing_extensions', '--upgrade', '-q'], check=True)\n")
                new_lines.append("# Pin numpy to 1.26.4 (torch 2.1.1 compat)\n")
                new_lines.append("subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy==1.26.4', '--force-reinstall', '-q'], check=True)\n")
                new_lines.append("# wandb after deps are set\n")
                new_lines.append("subprocess.run([sys.executable, '-m', 'pip', 'install', 'wandb==0.16.0', '-q'], check=True)\n")
                replaced = True
                print("Replaced pip lines")
            elif 'import subprocess' in l or l.strip() == '':
                new_lines.append(l)
            else:
                new_lines.append(l)
        if replaced:
            cell['source'] = new_lines

with open('C:/dev/notebooks/SRMoE_Training.ipynb', 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print("Saved")
