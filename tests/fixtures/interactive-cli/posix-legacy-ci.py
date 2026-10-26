"""Qualify the frozen updater in a disposable GitHub-hosted POSIX VM."""
import base64
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

if not (sys.platform in {"linux", "darwin"} and os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires an isolated GitHub-hosted POSIX VM")

platform = "darwin-arm64" if sys.platform == "darwin" else "linux-x64"
registry = json.loads((ROOT / "harness/setup/legacy-update-posix-images.json").read_text())
original = next(row for row in registry["images"] if row["platform"] == platform and row["version"] == "0.13.1")
(ROOT / "tests/work").mkdir(parents=True, exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="posix-legacy-ci-", dir=ROOT / "tests/work")).resolve()
image = ROOT / "tests/work/public131-posix-0140" / platform / "hive"
candidate = ROOT / "target/debug/hive"
command = [sys.executable, str(Path(__file__).resolve())]
run = Run("격리 POSIX VM의 공개0.13.1 첫 갱신 연결", command,
          paths=[work.relative_to(ROOT).as_posix(), image.parent.relative_to(ROOT).as_posix()], binary=candidate)
code = 1
completed = []
modes = sys.argv[1:] or ["npm", "direct", "confirm", "cancel-resume"]
assert modes and len(modes) == len(set(modes))
assert set(modes) <= {"npm", "direct", "confirm", "cancel-resume", "deadline", "slow", "parent-exit", "parent-exit-apply"}
try:
    if not image.is_file():
        url = f"https://registry.npmjs.org/@aigent-hive/{platform}/-/{platform}-0.13.1.tgz"
        with urllib.request.urlopen(url, timeout=30) as response:
            archive = response.read(128 * 1024 * 1024 + 1)
        assert len(archive) <= 128 * 1024 * 1024
        assert "sha512-" + base64.b64encode(hashlib.sha512(archive).digest()).decode() == original["package_integrity"]
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
            member = package.getmember("package/bin/hive")
            assert member.isfile() and member.size <= 128 * 1024 * 1024
            data = package.extractfile(member).read()
        assert hashlib.sha256(data).hexdigest() == original["binary_sha256"]
        image.parent.mkdir(parents=True, exist_ok=True)
        image.write_bytes(data)
        image.chmod(0o755)
    assert hashlib.sha256(image.read_bytes()).hexdigest() == original["binary_sha256"]
    for mode in modes:
        environment = dict(os.environ, HIVE_QUALIFICATION_VM="1", HIVE_LEGACY_FIXTURE_ROOT=str(work / mode))
        code = run.execute([sys.executable, str(ROOT / "tests/fixtures/interactive-cli/linux-legacy-update.py"), mode], env=environment)
        if code:
            break
        completed.append(json.loads((work / mode / "results" / (mode + ".json")).read_text()))
finally:
    run.finish(code, details={"platform": platform, "original": original, "completed": completed,
        "scope": "isolated hosted VM; original public parent, replaced development child, real PTY and pipes; synthetic installer and Antigravity; no model, real host registration, public14 package or personal installation"})
raise SystemExit(code)
