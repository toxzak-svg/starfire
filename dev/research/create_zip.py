import os
import zipfile

# Remove existing zip if exists
if os.path.exists('high_value_experiments.zip'):
    os.remove('high_value_experiments.zip')

# Directories to zip (excluding .venv, __pycache__, .git, etc.)
dirs_to_zip = ['214/hgsel-moe', 'attention', 'circuit_lm']

def exclude_func(item):
    # Skip these directories/files
    skip_patterns = ['.venv', '__pycache__', '.git', 'node_modules', '.pytest_cache', '.egg-info']
    for pattern in skip_patterns:
        if pattern in item:
            return True
    return False

# Create zip
with zipfile.ZipFile('high_value_experiments.zip', 'w', zipfile.ZIP_DEFLATED) as zipf:
    for base_dir in dirs_to_zip:
        for root, dirs, files in os.walk(base_dir):
            # Filter out directories
            dirs[:] = [d for d in dirs if not exclude_func(d)]
            
            for file in files:
                if exclude_func(file):
                    continue
                file_path = os.path.join(root, file)
                arcname = file_path  # Keep the directory structure
                zipf.write(file_path, arcname)
                print(f'Added: {arcname}')

print('Done!')
print(f'Zip file size: {os.path.getsize("high_value_experiments.zip") / 1024:.2f} KB')
