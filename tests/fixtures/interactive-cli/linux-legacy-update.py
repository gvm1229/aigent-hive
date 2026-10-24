"""Frozen-updater component acceptance, executed only inside the qualification container."""
import hashlib
import json
import os
from pathlib import Path
import pty
import select
import shutil
import subprocess
import sys
import time

if sys.platform != "linux" or os.environ.get("HIVE_QUALIFICATION_CONTAINER") != "1":
    raise SystemExit("requires the isolated Linux qualification container")
mode = sys.argv[1]
assert mode in {"npm", "direct", "confirm", "cancel", "cancel-resume", "deadline"}
root = Path("/tmp/hive-first-update")
root.mkdir()
user = root / "user"
user.mkdir()
package = root / "package"
(package / "bin").mkdir(parents=True)
shim = root / "shim"
shim.mkdir()
binary = package / "bin/hive"
predecessor = Path("/source/tests/work/public131-posix-0140/linux-x64/hive")
old_digest = hashlib.sha256(predecessor.read_bytes()).hexdigest()
assert old_digest == "c0ed023cd25463e9c2aa43a521d5717e5471e3748c3c3d65803e76b47de71c6d"
shutil.copyfile(predecessor, binary)
binary.chmod(0o755)
source_digest = hashlib.sha256(Path("/build/debug/hive").read_bytes()).hexdigest()
metadata = {"name": "@aigent-hive/linux-x64", "version": "0.13.1", "aigentHive": {"productVersion": "0.13.1"}}
(package / "package.json").write_text(json.dumps(metadata))
receipt = package / "share/aigent-hive/install-receipt.json"
if mode == "direct":
    receipt.parent.mkdir(parents=True)
    receipt.write_text(json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
                                  "version": "0.13.1", "package_version": "0.13.1", "artifact_sha256": "sha256:" + old_digest}))

installer = '''#!/usr/bin/python3
import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
root=Path('/tmp/hive-first-update'); package=root/'package'
direct=Path(sys.argv[0]).name=='sh'
if direct:
    assert os.environ['AIGENT_HIVE_VERSION']=='0.13.2'
    assert os.environ['AIGENT_HIVE_PACKAGE_VERSION']=='0.13.2-test.2'
    assert os.environ['AIGENT_HIVE_PREFIX']==str(package)
else:
    assert sys.argv[1:]==['install','-g','aigent-hive@0.13.2-test.2'],sys.argv
count=root/'install-count';count.write_text(str(int(count.read_text())+1) if count.exists() else '1')
binary=package/'bin/hive';binary.unlink();shutil.copyfile('/build/debug/hive',binary);binary.chmod(0o755)
subprocess.run(['/usr/bin/strip','--strip-debug',str(binary)],check=True)
digest=hashlib.sha256(binary.read_bytes()).hexdigest()
(root/'incoming.json').write_text(json.dumps({'sha256':digest,'installer':'env/sh' if direct else 'npm'}))
(package/'package.json').write_text(json.dumps({'name':'@aigent-hive/linux-x64','version':'0.13.2-test.2','aigentHive':{'productVersion':'0.13.2'}}))
if direct:
    (package/'share/aigent-hive/install-receipt.json').write_text(json.dumps({'schema_version':1,'owner':'direct','product':'aigent-hive','version':'0.13.2','package_version':'0.13.2-test.2','artifact_sha256':'sha256:'+digest}))
'''
for name in ("npm", "sh"):
    (shim / name).write_text(installer)
    (shim / name).chmod(0o755)
(shim / "env").symlink_to("/usr/bin/env")
(shim / "agy").write_text('''#!/usr/bin/python3
import json,shutil,sys
from pathlib import Path
root=Path('/tmp/hive-first-update/user');args=sys.argv[1:]
if args==['--version']:print('1.1.7');raise SystemExit(0)
stage=root/'.gemini/config/plugins/aigent-hive'
if args[:2]==['plugin','install']:
    if stage.exists():shutil.rmtree(stage)
    shutil.copytree(args[2],stage)
if args==['plugin','list']:
    print(json.dumps({'imports':[{'name':'aigent-hive','source':'antigravity','importedAt':'2026-07-27T00:00:00Z','components':['skills']}]} if stage.exists() else {'imports':[]}))
else:print('{}')
''')
(shim / "agy").chmod(0o755)
environment = {"HOME": str(user), "USERPROFILE": str(user), "PATH": str(shim), "LANG": "C.UTF-8",
               "LC_ALL": "C.UTF-8", "CODEX_HOME": str(root / "codex"),
               "NPM_CONFIG_PREFIX": str(root / "npm-prefix"), "NPM_CONFIG_CACHE": str(root / "npm-cache")}
command = [str(binary), "update", "--channel", "test", "--user-root", str(user)]
answers = []
terminal_runs = []

def terminal_run(command, stop=None):
    answers = []
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execve(str(binary), command, environment)
    began = time.monotonic()
    output, pending = b"", b""
    status = None
    try:
        while time.monotonic() - began < (560 if stop == "hold" else 180):
            readable, _, _ = select.select([descriptor], [], [], 0.2)
            if readable:
                try:
                    chunk = os.read(descriptor, 65536)
                except OSError:
                    chunk = b""
                if chunk:
                    output += chunk
                    pending += chunk
                    text = pending.decode("utf-8", errors="replace")
                    reply = None
                    if "Proceed? [y/N]: " in text:
                        reply, kind = "yes", "software"
                    elif "comma-separated): " in text:
                        reply, kind = "antigravity", "old-host"
                    elif text.endswith("> "):
                        if stop == "hold" and ("Korean" in text or "한국어" in text) and "English" in text:
                            reply, kind = None, "held"
                        elif "Apply these changes?" in text or "이 변경을 적용할까요?" in text:
                            reply, kind = "yes", "final"
                        elif ("Korean" in text or "한국어" in text) and "English" in text:
                            reply, kind = "1", "language"
                        elif stop == "cancel":
                            reply, kind = ":cancel", "cancel"
                        else:
                            reply, kind = "", "default"
                    if reply is not None:
                        os.write(descriptor, (reply + "\n").encode())
                        answers.append(kind)
                        pending = b""
            exited, value = os.waitpid(pid, os.WNOHANG)
            if exited:
                status = os.waitstatus_to_exitcode(value)
                break
        assert status == (2 if stop in {"cancel", "hold"} else 0), (status, output[-6000:].decode(errors="replace"))
    finally:
        if status is None:
            try:
                os.killpg(pid, 9)
                os.waitpid(pid, 0)
            except ProcessLookupError:
                pass
        os.close(descriptor)

    terminal_runs.append({"stop": stop, "elapsed_seconds": round(time.monotonic() - began, 3), "status": status})
    return answers

if mode == "confirm":
    def invoke(arguments):
        result = subprocess.run([str(binary), *arguments], env=environment, capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, (result.returncode, result.stdout[-4000:], result.stderr[-4000:])
    invoke(["install", "--scope", "user", "--host", "antigravity", "--user-root", str(user), "--apply", "--output", "json"])
    old = root / "old.json"
    old.write_text(json.dumps({"schema_version": 1, "interface_language": "ko", "wiki": {"language": "ko"},
        "profile": {"contexts": ["non-developer"]}, "persona": {"id": "balanced"}, "selected_hosts": ["antigravity"],
        "skills": {"mode": "all"}, "usage_guard": {"enabled": False, "quota_reset_guard_enabled": True, "stop_remaining_percent": 5}}))
    invoke(["setup", "--scope", "user", "--answers", str(old), "--user-root", str(user), "--apply", "--output", "json"])
    result = subprocess.run([*command, "--confirm"], env=environment, input="no\nyes\n", capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, (result.returncode, result.stdout[-4000:], result.stderr[-4000:])
else:
    answers = terminal_run(command, "cancel" if mode in {"cancel", "cancel-resume"} else "hold" if mode == "deadline" else None)

config = user / ".hive/config/user-setup.yml"
progress = (user / ".hive/config/setup-progress/user.yml").read_text()
if mode in {"cancel", "cancel-resume", "deadline"}:
    assert not config.exists() and "phase: awaiting-answers" in progress
    if mode != "deadline":
        assert "interface_language: ko" in progress
    else:
        assert 418 <= terminal_runs[0]["elapsed_seconds"] < 550, terminal_runs
    if mode in {"cancel-resume", "deadline"}:
        answers += terminal_run([str(binary), "update", "--user-root", str(user)])
        saved = config.read_text()
        progress = (user / ".hive/config/setup-progress/user.yml").read_text()
        assert "schema_version: 2" in saved and "interface_language: ko" in saved
        assert "enforcement_enabled: false" in saved and "phase: complete" in progress
        assert (root / "install-count").read_text() == "1"
else:
    saved = config.read_text()
    assert "schema_version: 2" in saved and "interface_language: ko" in saved and "phase: complete" in progress
    if mode == "confirm":
        assert "enforcement_enabled: true" in saved and "warnings_enabled: false" in saved
        shown = subprocess.run([str(binary), "setup", "--scope", "user", "--show",
            "--user-root", str(user), "--output", "json"], env=environment,
            capture_output=True, text=True, timeout=30)
        assert shown.returncode == 0, shown.stderr
        data = json.loads(shown.stdout)["data"]
        guard = data["stored"]["usage_guard"]
        assert guard["preflight"] == {"threshold_enabled": False, "reset_enabled": False,
            "zero_quota_enabled": True, "observation_failure": "block"}, guard
        assert not guard["midway"]["enabled"] and guard["stop_remaining_percent"] == 5
        assert data["pending_questions"] == ["usage-enforcement", "usage-warnings"], data
    else:
        assert "enforcement_enabled: false" in saved and "final" in answers
incoming = json.loads((root / "incoming.json").read_text())
assert incoming["sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
evidence = {"scenario": mode, "predecessor_sha256": old_digest, "incoming_source_sha256": source_digest,
    "incoming_stripped_sha256": incoming["sha256"], "installer_stub": incoming["installer"], "answers": answers,
    "original_parent": "public 0.13.1 Linux x64", "incoming": "development 0.14.0",
    "registry_target": "0.13.2-test.2", "private_user_mount": False, "terminal_runs": terminal_runs,
    "installer_invocations": int((root / "install-count").read_text()),
    "scope": "Linux Docker PTY/pipe component test; fake installer and Antigravity; no model, actual host or public14 acceptance"}
Path("/results", mode + ".json").write_text(json.dumps(evidence, indent=2) + "\n")
print(json.dumps(evidence))
