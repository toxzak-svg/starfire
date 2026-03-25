import json, re

path = 'C:/dev/notebooks/SRMoE_Training.ipynb'
raw = open(path, encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)

# === FIX 1: pip install in cell 1 (id=0397ebd3) ===
# Current: wandb + typing_extensions in one call
# Fix: split into two calls (typing_extensions first, then wandb==0.16.0)
for cell in nb['cells']:
    if cell['id'] == '0397ebd3':
        lines = cell['source']
        new_lines = []
        for line in lines:
            if "'wandb'" in line and 'pip' in line and 'install' in line:
                # Replace with two calls
                new_lines.append(line.replace(
                    "'wandb'",
                    "'typing_extensions'"
                ).replace("'--quiet']", "'--upgrade', '-q']"))
                new_lines.append(line.replace(
                    "'wandb'",
                    "'wandb==0.16.0'"
                ).replace("'--quiet']", "'-q']"))
                print("Fix 1: pip install split into two calls")
            else:
                new_lines.append(line)
        cell['source'] = new_lines

# === FIX 2: dead mask in cell 7 (id=7b05f1f6) ===
# The issue: route_mask is computed but then L_route = F.cross_entropy(r_t, r_lbl, ...)
# uses the FULL (unmasked) tensors. masked_r_t/masked_r_lbl are computed but unused.
for cell in nb['cells']:
    if cell['id'] != '7b05f1f6':
        continue
    lines = cell['source']
    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # Look for the 'if route_mask.sum() > 0:' block and replace it
        if 'if route_mask.sum() > 0:' in line:
            # Replace the whole block: move r_wt outside the if, use masked tensors
            indent = len(line) - len(line.lstrip())
            base = ' ' * indent
            
            # Collect the old block lines
            old_block = []
            j = i
            while j < len(lines) and 'L_route = F.cross_entropy' not in lines[j]:
                old_block.append(lines[j])
                j += 1
            # Also collect the L_route line
            if j < len(lines):
                old_block.append(lines[j])
            
            old_text = '\n'.join(old_block)
            
            # Build new block
            new_block_lines = []
            # Find r_wt line (should be inside the if block)
            for old_line in old_block:
                if 'r_wt = torch.tensor(self.route_class_weights' in old_line:
                    # Move it before the if
                    new_block_lines.append(old_line)
                elif 'if route_mask.sum() > 0:' in old_line:
                    new_block_lines.append(f'{base}if route_mask.sum() > 0:')
                elif 'masked_r_t = r_t[' in old_line:
                    new_block_lines.append(f'{base}    L_route = F.cross_entropy(r_t[route_mask], r_lbl[route_mask], weight=r_wt)')
                elif 'masked_r_lbl = r_lbl[' in old_line:
                    pass  # skip, merged into above
                elif 'L_route = F.cross_entropy(r_t, r_lbl' in old_line:
                    pass  # skip, replaced by masked version
                elif old_line.strip() == '' or old_line.strip().startswith('#'):
                    new_block_lines.append(old_line)
            
            # Add else case for the L_route
            new_block_lines.append(f'{base}else:')
            new_block_lines.append(f'{base}    L_route = F.cross_entropy(r_t, r_lbl, weight=r_wt, ignore_index=IGNORE_INDEX)')
            
            new_lines.extend(new_block_lines)
            print(f"Fix 2: replaced dead mask block ({len(old_block)} lines -> {len(new_block_lines)} lines)")
            i = j + 1
            continue
        else:
            new_lines.append(line)
        i += 1
    cell['source'] = new_lines

# Save
with open(path, 'w', encoding='utf-8') as f:
    json.dump(nb, f, indent=1, ensure_ascii=False)
print("Saved to", path)
