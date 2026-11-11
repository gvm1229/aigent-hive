"""Real CLI check of `hive storage` and `hive prune` in a disposable user root."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]
binary = Path(sys.argv[1]).resolve()
(ROOT / "tests/work").mkdir(exist_ok=True)
user = Path(tempfile.mkdtemp(prefix="storage-cli-", dir=ROOT / "tests/work")).resolve()
bin_dir = user / ".hive/runtime/usage-monitor/bin"
files = {
    ".hive/knowledge/Wiki/a.md": 12345, ".hive/index/hive.sqlite3": 54321,
    ".hive/index/vector/v": 999, ".hive/backups/b": 777,
}
for name in ("a", "b", "old", "prior", "new"):
    files[f".hive/runtime/usage-monitor/bin/hive-{name}.exe"] = 100
for relative, size in files.items():
    path = user / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(bytes([7]) * size)


def run(*arguments):
    result = subprocess.run([str(binary), *arguments, "--user-root", str(user), "--output", "json"],
                            capture_output=True, text=True, timeout=60)
    return result.returncode, json.loads(result.stdout)


def tree():
    return sorted((str(p.relative_to(user)), hashlib.sha256(p.read_bytes()).hexdigest())
                  for p in user.rglob("*") if p.is_file())


code, report = run("storage")
assert code == 0, report
total = sum(p.stat().st_size for p in (user / ".hive").rglob("*") if p.is_file())
assert report["data"]["total_bytes"] == total == sum(r["bytes"] for r in report["data"]["categories"])
code, refused = run("prune", "--confirm")
assert code != 0 and refused["code"] == "hive.storage-prune-refused", refused
assert len(list(bin_dir.iterdir())) == 5
config = user / ".hive/runtime/usage-monitor/config.json"
config.write_text(json.dumps({"schema_version": 1, "enabled": False, "version": "0.14.1",
    "executable": str(bin_dir / "hive-new.exe"), "executable_digest": "sha256:" + "0" * 64,
    "task_name": "AigentHive-UsageMonitor-0000000000000000",
    "arguments": f'usage monitor run --user-root "{user}" --output json'}))
(user / ".hive/runtime/usage-monitor/upgrade.json").write_text(json.dumps(
    {"from": {"executable": str(bin_dir / "hive-old.exe")}, "to": {"executable": str(bin_dir / "hive-new.exe")}}))
before = tree()
code, preview = run("prune", "--dry-run")
assert code == 0, preview
assert tree() == before
assert len(preview["data"]["report"]["candidates"]) == 3
code, applied = run("prune", "--confirm")
assert code == 0, applied
assert sorted(p.name for p in bin_dir.iterdir()) == ["hive-new.exe", "hive-old.exe"]
print(json.dumps({"total_bytes": total, "refused_without_config": True, "dry_run_unchanged": True,
                  "remaining": sorted(p.name for p in bin_dir.iterdir())}))
