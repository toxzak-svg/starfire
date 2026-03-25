import json
nb = json.load(open('SRMoE_Training.ipynb', encoding='utf-8'))
for i, cell in enumerate(nb['cells']):
    ec = cell.get('execution_count')
    err = cell['cell_type']=='code' and any(o.get('output_type')=='error' for o in cell.get('outputs',[]))
    status = 'ERROR' if err else ('OK' if ec is not None else 'empty')
    print(f"Cell {i:2d}  exec={str(ec):5s}  {status:5s}  id={cell['id']}")
