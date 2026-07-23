from pathlib import Path
import subprocess

branch = "feat/l1c-push-probe"
path = Path("docs/experiments/L1C_PUSH_PROBE.md")
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text("# L1-C push probe\n\nThe authorized runner created this branch.\n")
subprocess.run(["git", "config", "user.name", "github-actions[bot]"], check=True)
subprocess.run(["git", "config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"], check=True)
subprocess.run(["git", "checkout", "-b", branch], check=True)
subprocess.run(["git", "add", str(path)], check=True)
subprocess.run(["git", "commit", "-m", "chore: prove L1-C branch push"], check=True)
subprocess.run(["git", "push", "--set-upstream", "origin", branch], check=True)
