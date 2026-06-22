from pathlib import Path
from huggingface_hub import HfApi, create_repo

REPO_ID = "toxzak/invention"
ROOT = Path(__file__).resolve().parent

api = HfApi()
create_repo(REPO_ID, repo_type="dataset", private=True, exist_ok=True)
try:
    api.update_repo_visibility(REPO_ID, repo_type="dataset", private=True)
except Exception as exc:
    print(f"Visibility update skipped or failed: {exc}")
api.upload_folder(
    repo_id=REPO_ID,
    repo_type="dataset",
    folder_path=str(ROOT),
    path_in_repo="",
    commit_message="Upload INVENT-TRACE-ZACH-v0 curated invention traces",
)
print(f"Uploaded {ROOT} to https://huggingface.co/datasets/{REPO_ID}")
