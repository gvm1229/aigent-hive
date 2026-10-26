"""Remove an actual candidate image only inside a disposable hosted OS."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

if not (os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires an isolated GitHub-hosted OS; local installations are excluded")

windows = os.name == "nt"
name = "hive.exe" if windows else "hive"
candidate = ROOT / "target/debug" / name
(ROOT / "tests/work").mkdir(parents=True, exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="native-core-removal-", dir=ROOT / "tests/work")).resolve()
user = work / "user"
prefix = work / "prefix"
binary = prefix / "bin" / name
binary.parent.mkdir(parents=True)
shutil.copyfile(candidate, binary)
binary.chmod(0o755)
sha = lambda data: hashlib.sha256(data).hexdigest()
binary_hash = sha(binary.read_bytes())
receipt = prefix / "share/aigent-hive/install-receipt.json"
receipt.parent.mkdir(parents=True)
receipt.write_text(json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
    "version": "0.14.0", "package_version": "0.14.0", "artifact_sha256": "sha256:" + binary_hash}))

settings = {"schema_version": 1, "interface_language": "en",
    "wiki": {"enabled": True, "language": "both"}, "profile": {"contexts": ["web-developer"]},
    "persona": {"id": "balanced"}, "selected_hosts": ["antigravity"], "skills": {"mode": "all"},
    "usage_guard": {"enabled": False, "stop_remaining_percent": 20, "codexbar_fallback_enabled": False,
                    "discord": {"enabled": False}}}
preserved = {
    user / ".hive/config/user-setup.yml": json.dumps(settings).encode(),
    user / ".hive/knowledge/note.md": b"Synthetic canonical knowledge.\n",
    user / ".hive/config/projects.yml": b"projects: []\n",
    user / "project/README.md": b"Synthetic project source.\r\n",
    prefix / "unrelated.txt": b"Not owned by Hive.\n",
}
for path, data in preserved.items():
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
derived = user / ".hive/index/fixture.db"
derived.parent.mkdir(parents=True)
derived.write_bytes(b"Disposable derived index fixture.")
environment = {"PATH": str(work / "closed-bin"), "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "CLAUDE_CONFIG_DIR",
            "GEMINI_CLI_HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "TEMP", "TMP",
            "NPM_CONFIG_PREFIX", "NPM_CONFIG_CACHE"]:
    directory = work / key
    directory.mkdir()
    environment[key] = str(directory)
if windows:
    for key in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"]:
        if key in os.environ:
            environment[key] = os.environ[key]

events = []
run = Run("격리 OS의 실제 직접 설치 본체 제거와 자료 보존",
    [sys.executable, str(Path(__file__).resolve())], paths=[work.relative_to(ROOT).as_posix()], binary=candidate)
code = 1

def invoke(*flags):
    command = [str(binary), "uninstall", "--user-root", str(user), *flags, "--output", "json"]
    result = subprocess.run(command, env=environment, input=b"", capture_output=True, timeout=60)
    response = json.loads(result.stdout)
    events.append({"arguments": ["uninstall", *flags, "--output", "json"], "exit_code": result.returncode,
                   "code": response.get("code"), "status": response.get("status"),
                   "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)})
    return result.returncode, response

def snapshot():
    return {p.relative_to(work).as_posix(): sha(p.read_bytes()) for p in work.rglob("*") if p.is_file()}

try:
    before = snapshot()
    exit_code, preview = invoke("--dry-run")
    assert exit_code == 0, preview
    plan = preview["data"]["preview"]
    assert plan["core"]["owner"] == "direct", preview
    assert not plan["hosts"] and not plan["monitor"]["configured"], preview
    assert snapshot() == before, "dry-run changed the installation"
    exit_code, refused = invoke()
    assert exit_code == 2 and refused["code"] == "hive.uninstall-needs-input", refused
    assert snapshot() == before, "unconfirmed removal changed the installation"
    exit_code, removed = invoke("--confirm")
    assert exit_code in (0, 2), removed
    assert removed["code"] in {"hive.user-uninstall-complete", "hive.user-uninstall-pending"}, removed
    history = user / removed["data"]["receipt"]
    assert history.resolve().is_relative_to((user / ".hive/history/lifecycle").resolve())
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        completed = json.loads(history.read_text())
        if completed["phase"] == "complete":
            break
        time.sleep(0.1)
    assert completed["phase"] == "complete", completed.get("last_error")
    assert not binary.exists() and not receipt.exists() and not derived.exists()
    for path, data in preserved.items():
        assert path.read_bytes() == data, path.relative_to(work)
    # Repeating the completed operation through the retained candidate is read-only and idempotent.
    repeat = subprocess.run([str(candidate), "uninstall", "--user-root", str(user), "--resume",
        removed["data"]["operation_id"], "--output", "json"], env=environment,
        input=b"", capture_output=True, timeout=30)
    repeated = json.loads(repeat.stdout)
    assert repeat.returncode == 0 and repeated["data"]["completion_verified"], repeated
    code = 0
finally:
    run.finish(code, details={"events": events, "original_candidate_sha256": binary_hash,
        "preserved_sha256": {p.relative_to(work).as_posix(): sha(data) for p, data in preserved.items()},
        "scope": "hosted isolated OS, actual copied candidate and direct-owner receipt, no registered app or monitor; preview/refusal/confirmed native removal/Windows delayed helper; not npm, actual app/monitor removal or public14 artifact acceptance"})
raise SystemExit(code)
