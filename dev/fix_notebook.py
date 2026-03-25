import json, re

path = 'C:/dev/notebooks/SRMoE_Training.ipynb'

raw = open(path, encoding='utf-8', errors='replace').read()
# Strip ANSI escape sequences before parsing
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)

for cell in nb['cells']:
    if cell['id'] != '7b05f1f6':
        continue
    src = ''.join(cell['source'])
    
    old = """        r_t = torch.stack(r_logits_list)
        # FIX #5: mask out encode-phase SKIP labels from route CE
        # Oracle SKIP during encode is a crutch, not a real routing decision
        r_lbl = torch.tensor(oracle_labels, dtype=torch.long, device=DEVICE)
        route_mask = torch.ones_like(r_lbl, dtype=torch.bool)  # default: include
        for ti, oa in enumerate(oracle_labels):
            if oa == IGNORE_INDEX:
                route_mask[ti] = False
            elif oa == 0 and ti < QUERY_START:  # SKIP during encode — skip this label
                route_mask[ti] = False
        if route_mask.sum() > 0:
            masked_r_t = r_t[route_mask]
            masked_r_lbl = r_lbl[route_mask]
            r_wt = torch.tensor(self.route_class_weights, dtype=torch.float, device=DEVICE)
        L_route = F.cross_entropy(r_t, r_lbl, weight=r_wt, ignore_index=IGNORE_INDEX)"""
    
    new = """        r_t = torch.stack(r_logits_list)
        # FIX #5: mask out encode-phase SKIP labels from route CE
        # Oracle SKIP during encode is a crutch, not a real routing decision
        r_lbl = torch.tensor(oracle_labels, dtype=torch.long, device=DEVICE)
        route_mask = torch.ones_like(r_lbl, dtype=torch.bool)  # default: include
        for ti, oa in enumerate(oracle_labels):
            if oa == IGNORE_INDEX:
                route_mask[ti] = False
            elif oa == 0 and ti < QUERY_START:  # SKIP during encode — skip this label
                route_mask[ti] = False
        r_wt = torch.tensor(self.route_class_weights, dtype=torch.float, device=DEVICE)
        if route_mask.sum() > 0:
            L_route = F.cross_entropy(r_t[route_mask], r_lbl[route_mask], weight=r_wt)
        else:
            L_route = F.cross_entropy(r_t, r_lbl, weight=r_wt, ignore_index=IGNORE_INDEX)"""

    if old in src:
        new_src = src.replace(old, new)
        cell['source'] = [new_src]
        print("Fixed mask bug in cell 7")
    else:
        print("Could not find exact old text")
        idx = src.find('masked_r_t')
        if idx >= 0:
            print("Found masked_r_t at:", idx)
            print("Context:", repr(src[idx-300:idx+400]))

json.dump(nb, open(path, 'w', encoding='utf-8'), indent=1, ensure_ascii=False)
print("Saved")
