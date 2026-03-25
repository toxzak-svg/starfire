import zipfile, os

print('Creating clean source zip...')
target = 'C:/dev/attention_src.zip'
exclude_dirs = {'__pycache__', 'results', 'venv', '.git', 'logs', '.pytest_cache', '__pypackages__'}

with zipfile.ZipFile(target, 'w', zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in os.walk('C:/dev/research/attention'):
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        for file in files:
            if file.endswith('.pyc') or file.endswith('.zip'):
                continue
            filepath = os.path.join(root, file)
            arcname = os.path.relpath(filepath, 'C:/dev/research')
            zf.write(filepath, arcname)
            print(f'  {arcname}')

size_mb = os.path.getsize(target) / 1e6
print(f'Done. Size: {size_mb:.1f} MB')
