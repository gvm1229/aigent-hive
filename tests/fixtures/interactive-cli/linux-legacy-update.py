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
assert mode in {"npm", "direct", "confirm", "cancel", "cancel-resume", "deadline", "slow", "parent-exit", "parent-exit-apply", "worker-unowned", "monitor-decline", "file-monitor-block"}
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
import json,os,shutil,sys,time
from pathlib import Path
root=Path('/tmp/hive-first-update/user');args=sys.argv[1:]
if args==['--version']:print('1.1.7');raise SystemExit(0)
stage=root/'.gemini/config/plugins/aigent-hive'
if args[:2]==['plugin','install']:
    if os.environ.get('HIVE_TEST_SLOW_ONCE')=='1' and not (root/'slow-once').exists():
        (root/'slow-once').write_text('used');(root/'slow-pid').write_text(str(os.getpid()));time.sleep(105)
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
if mode in {"slow", "parent-exit-apply"}:
    environment["HIVE_TEST_SLOW_ONCE"] = "1"
if mode == "worker-unowned":
    request = {"schema_version": 1, "package_version": "0.14.0", "catalog_digest": "sha256:" + "0" * 64,
        "deadline_unix_millis": int(time.time() * 1000) + 90000, "parent_process_id": os.getpid(),
        "user_root": str(user), "operation_id": "a" * 64, "answers": "e30=", "expected_config": None}
    result = subprocess.run(["/build/debug/hive", "__setup-apply-worker"], input=json.dumps(request),
        env=environment, capture_output=True, text=True, timeout=15)
    reply = json.loads(result.stdout)
    assert result.returncode > 0 and reply["phase"] == "incomplete", (result.returncode, reply)
    assert "own process group" in reply["message"] and not (user / ".hive").exists(), reply
    evidence = {"scenario": mode, "incoming_source_sha256": source_digest,
        "caller_survived": True, "shared_group_refused_without_writes": True,
        "scope": "Linux source14 worker rejected in caller group; no old updater, model or personal installation"}
    Path("/results", mode + ".json").write_text(json.dumps(evidence, indent=2) + "\n")
    print(json.dumps(evidence))
    raise SystemExit(0)
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
    killed = False
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
                        if stop == "parent-exit":
                            if not killed:
                                os.kill(pid, 15)
                                killed = True
                            reply, kind = None, "parent-exit"
                        elif stop == "hold" and ("Korean" in text or "한국어" in text) and "English" in text:
                            reply, kind = None, "held"
                        elif "Apply these changes?" in text or "이 변경을 적용할까요?" in text:
                            reply, kind = "yes", "final"
                        elif mode == "monitor-decline" and "설정 방식을" in text and "맞춤 설정" in text:
                            reply, kind = "2", "custom"
                        elif mode == "monitor-decline" and "사용량에 따라 작업을 막거나 중단하는 기능" in text and "이 변경을 적용할까요?" not in text:
                            reply, kind = "yes", "enforcement"
                        elif mode == "monitor-decline" and "실행 중 사용량을 계속 확인하는 감시" in text:
                            reply, kind = "yes", "monitor-choice"
                        elif mode == "monitor-decline" and "이 사용자 계정의 Codex 대화 전체" in text:
                            reply, kind = ("yes", "unsupported-yes") if "unsupported-yes" not in answers else ("no", "runtime-no")
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
            if stop == "parent-exit-apply" and not killed and (user / "slow-pid").exists():
                os.kill(pid, 15)
                killed = True
            exited, value = os.waitpid(pid, os.WNOHANG)
            if exited:
                status = os.waitstatus_to_exitcode(value)
                break
        assert status == (-15 if stop in {"parent-exit", "parent-exit-apply"} else 2 if stop in {"cancel", "hold", "slow"} else 0), (status, output[-6000:].decode(errors="replace"))
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
    answers = terminal_run(command, "cancel" if mode in {"cancel", "cancel-resume"} else "hold" if mode == "deadline" else mode if mode in {"slow", "parent-exit", "parent-exit-apply"} else None)

config = user / ".hive/config/user-setup.yml"
progress = (user / ".hive/config/setup-progress/user.yml").read_text()
if mode in {"cancel", "cancel-resume", "deadline", "slow", "parent-exit", "parent-exit-apply"}:
    assert "phase: complete" not in progress
    if mode not in {"slow", "parent-exit-apply"}:
        assert not config.exists() and "phase: awaiting-answers" in progress
    if mode in {"cancel", "cancel-resume"}:
        assert "interface_language: ko" in progress
    elif mode == "deadline":
        assert 418 <= terminal_runs[0]["elapsed_seconds"] < 550, terminal_runs
    elif mode == "slow":
        assert 88 <= terminal_runs[0]["elapsed_seconds"] < 135, terminal_runs
        slow_pid = int((user / "slow-pid").read_text())
        proc = Path(f"/proc/{slow_pid}/stat")
        assert not proc.exists() or proc.read_text().rsplit(") ", 1)[1].split()[0] == "Z", "owned host subprocess survived worker timeout"
    elif mode in {"parent-exit", "parent-exit-apply"}:
        def live_children():
            result = []
            for entry in Path("/proc").glob("[0-9]*/comm"):
                try:
                    if entry.read_text().strip() == "hive" and (entry.parent / "stat").read_text().rsplit(") ", 1)[1].split()[0] != "Z":
                        result.append(entry.parent.name)
                except FileNotFoundError:
                    pass
            if mode == "parent-exit-apply":
                host_pid = int((user / "slow-pid").read_text())
                host_stat = Path(f"/proc/{host_pid}/stat")
                if host_stat.exists() and host_stat.read_text().rsplit(") ", 1)[1].split()[0] != "Z":
                    result.append("owned-host:" + str(host_pid))
            return result
        until = time.monotonic() + 15
        while live_children() and time.monotonic() < until:
            time.sleep(0.1)
        assert not live_children(), ("orphaned setup remained after parent exit", live_children())
    if mode in {"cancel-resume", "deadline", "slow", "parent-exit", "parent-exit-apply"}:
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
    elif mode == "monitor-decline":
        shown=subprocess.run([str(binary),"setup","--scope","user","--show","--user-root",str(user),"--output","json"],env=environment,capture_output=True,text=True,timeout=30)
        assert shown.returncode==0,shown.stderr
        stored=json.loads(shown.stdout)["data"]["stored"]
        assert stored["usage_guard"]["enforcement_enabled"] and not stored["usage_guard"]["midway"]["enabled"]
        decision=stored["preference_decisions"]["/usage_guard/midway/enabled"]
        assert decision["question_id"]=="user-monitor-activation" and decision["origin"]=="explicit"
        assert answers.count("unsupported-yes")==1 and answers.count("runtime-no")==1
        assert answers[-1]=="final",answers
        assert not (user/".hive/runtime/usage-monitor/config.json").exists()
    else:
        assert "enforcement_enabled: false" in saved and "final" in answers
        if mode == "file-monitor-block":
            shown=subprocess.run([str(binary),"setup","--scope","user","--show","--user-root",str(user),"--output","json"],env=environment,capture_output=True,text=True,check=True)
            settings=json.loads(shown.stdout)["data"]["stored"]
            settings["usage_guard"]["enforcement_enabled"]=True
            settings["usage_guard"]["midway"]["enabled"]=True
            document=root/"monitor-answers.json";document.write_text(json.dumps(settings))
            args=[str(binary),"setup","--scope","user","--answers",str(document),"--user-root",str(user),"--output","json"]
            dry=subprocess.run([*args,"--dry-run"],env=environment,capture_output=True,text=True,timeout=30)
            assert dry.returncode==0,(dry.stdout,dry.stderr)
            scope=json.loads(dry.stdout)["data"]["monitor_activation"]
            assert not scope["platform_eligible"] and scope["requires_separate_confirmation"]
            misuse=subprocess.run([*args,"--dry-run","--monitor-confirm",scope["confirmation_digest"]],env=environment,capture_output=True,text=True,timeout=30)
            assert misuse.returncode!=0 and "monitor confirmation requires --apply" in misuse.stdout,("monitor confirmation option was discarded",misuse.returncode)
            before=config.read_bytes()
            for flags in [[],["--monitor-confirm",scope["confirmation_digest"]]]:
                blocked=subprocess.run([*args,"--apply",*flags],env=environment,capture_output=True,text=True,timeout=30)
                assert blocked.returncode!=0,(blocked.stdout,blocked.stderr)
                assert config.read_bytes()==before
                assert not (user/".hive/runtime/usage-monitor/config.json").exists()
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
