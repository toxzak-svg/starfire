import zipfile, os, sys

print('Creating zip...')
target = 'C:/dev/attention.zip'
with zipfile.ZipFile(target, 'w', zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in os.walk('C:/dev/research/attention'):
        dirs[:] = [d for d in dirs if d != '__pycache__']
        for file in files:
            if file.endswith('.pyc'):
                continue
            filepath = os.path.join(root, file)
            zf.write(filepath)
            print(f'  Added: {filepath}')

size_mb = os.path.getsize(target) / 1e6
print(f'Done. Size: {size_mb:.1f} MB')
