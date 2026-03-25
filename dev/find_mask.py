import json
nb = json.load(open('C:/dev/notebooks/SRMoE_Training.ipynb', encoding='utf-8'))
for i, c in enumerate(nb['cells']):
    if c['id'] == '7b05f1f6':
        src = ''.join(c['source'])
        # find the mask section
        idx = src.find('masked_r_t')
        print(repr(src[idx-200:idx+400]))
