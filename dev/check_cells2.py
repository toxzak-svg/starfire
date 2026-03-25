import json, re
raw = open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8', errors='replace').read()
clean = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', raw)
nb = json.loads(clean)
for i, cell in enumerate(nb['cells']):
    ec = cell.get('execution_count')
    err = cell['cell_type']=='code' and any(o.get('output_type')=='error' for o in cell.get('outputs',[]))
    status = 'ERROR' if err else ('OK' if ec is not None else 'empty')
    print('Cell %2d  exec=%5s  %5s  id=%s' % (i, str(ec), status, cell['id']))
