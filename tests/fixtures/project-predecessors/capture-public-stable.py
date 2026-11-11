"""Capture a synthetic predecessor project with an exact public-stable Hive build.

Usage: capture-public-stable.py <hive-binary> <version> <release-date YYYY-MM-DD> <source description>
Writes tests/fixtures/project-predecessors/<version>/public-stable/{codex-native.zip,manifest.json}.
Uses a disposable user root and project under tests/work; no user installation is touched.
"""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from tests.conformance.support.harness import write_operational_user_setup  # noqa: E402

binary, version, release_date, source = Path(sys.argv[1]).resolve(), sys.argv[2], sys.argv[3], sys.argv[4]
answers = "tests/fixtures/setup/answers-no-role-no-hook.yml"
capabilities = "tests/fixtures/setup/capabilities-codex-host-native.json"
(ROOT / "tests/work").mkdir(exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="predecessor-capture-", dir=ROOT / "tests/work"))
user, project = work / "user-root", work / "project"
write_operational_user_setup(user)
project.mkdir()
result = subprocess.run([str(binary), "setup", "--target", str(project), "--answers", str(ROOT / answers),
                         "--capabilities", str(ROOT / capabilities), "--user-root", str(user), "--apply",
                         "--output", "json"], capture_output=True, text=True, timeout=300, cwd=ROOT)
assert result.returncode == 0, (result.stdout, result.stderr)
files = sorted(p for p in project.rglob("*") if p.is_file())
destination = ROOT / "tests/fixtures/project-predecessors" / version / "public-stable"
destination.mkdir(parents=True, exist_ok=True)
archive = destination / "codex-native.zip"
year, month, day = (int(part) for part in release_date.split("-"))
with ZipFile(archive, "w", ZIP_DEFLATED) as zipped:
    for path in files:
        info = ZipInfo(path.relative_to(project).as_posix(), (year, month, day, 0, 0, 0))
        info.compress_type = ZIP_DEFLATED
        info.external_attr = 0o600 << 16
        zipped.writestr(info, path.read_bytes())
manifest = {
    "schema_version": 1,
    "product_version": version,
    "archive": archive.name,
    "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
    "file_count": len(files),
    "generation": source + "; synthetic setup only; no user installation or settings changed",
    "source_asset": "git:v" + version,
    "source_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    "capability_fixture": capabilities,
    "answers_fixture": answers,
}
(destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"files": len(files), "sha256": manifest["sha256"]}))
