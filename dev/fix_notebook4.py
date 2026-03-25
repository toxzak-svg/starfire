import json, re

path = 'C:/dev/notebooks/SRMoE_Training.ipynb'
raw = open(path, encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)

# === FIX 1: pip install in cell 1 (id=0397ebd3) ===
for cell in nb['cells']:
    if cell['id'] == '0397ebd3':
        src = ''.join(cell['source'])
        old_pip = "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy', 'wandb', '--quiet'], check=True)"
        new_pip = ("subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy', 'typing_extensions', '--upgrade', '-q'], check=True)\n"
                   "subprocess.run([sys.executable, '-m', 'pip', 'install', 'wandb==0.16.0', '-q'], check=True)")
        if old_pip in src:
            new_src = src.replace(old_pip, new_pip)
            cell['source'] = [new_src]
            print("Fix 1 applied: pip install split")
        else:
            print("Fix 1: could not find old pip. Checking actual content...")
            for i, l in enumerate(cell['source']):
                if 'pip' in l and 'install' in l:
                    print(f"  line {i}: {repr(l)}")

# === FIX 2: dead mask in cell 7 (id=7b05f1f6) ===
for cell in nb['cells']:
    if cell['id'] != '7b05f1f6':
        continue
    src = ''.join(cell['source'])
    
    old = ("if route_mask.sum() > 0:\n"
           "            masked_r_t = r_t[route_mask]\n"
           "            masked_r_lbl = r_lbl[route_mask]\n"
           "            r_wt = torch.tensor(self.route_class_weights, dtype=torch.float, device=DEVICE)\n"
           "        L_route = F.cross_entropy(r_t, r_lbl, weight=r_wt, ignore_index=IGNORE_INDEX)")
    
    new = ("r_wt = torch.tensor(self.route_class_weights, dtype=torch.float, device=DEVICE)\n"
           "        if route_mask.sum() > 0:\n"
           "            L_route = F.cross_entropy(r_t[route_mask], r_lbl[route_mask], weight=r_wt)\n"
           "        else:\n"
           "            L_route = F.cross_entropy(r_t, r_lbl, weight=r_wt, ignore_index=IGNORE_INDEX)")
    
    if old in src:
        new_src = src.replace(old, new)
        cell['source'] = [new_src]
        print("Fix 2 applied: dead mask fixed")
    else:
        print("Fix 2: could not find old mask code, searching...")
        idx = src.find('route_mask.sum()')
        if idx >= 0:
            print("Found route_mask.sum() at char", idx)
            print("Context:", repr(src[idx-20:idx+500]))

# Save
with open(path, 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print("Saved to", path)
