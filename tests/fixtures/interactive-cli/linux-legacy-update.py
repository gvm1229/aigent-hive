"""Frozen-updater component acceptance in an isolated container or hosted POSIX VM."""
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

hosted = (os.environ.get("GITHUB_ACTIONS") == "true"
          and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"
          and os.environ.get("HIVE_QUALIFICATION_VM") == "1"
          and sys.platform in {"linux", "darwin"})
if not hosted and (sys.platform != "linux" or os.environ.get("HIVE_QUALIFICATION_CONTAINER") != "1"):
    raise SystemExit("requires the isolated qualification container or hosted POSIX VM")
mode = sys.argv[1]
assert mode in {"npm", "direct", "confirm", "cancel", "cancel-resume", "deadline", "slow", "late-slow", "parent-exit", "parent-exit-apply", "worker-unowned", "monitor-decline", "file-monitor-block", "project-file", "project-file-interrupt", "vector-choice", "host-deselect", "locale-input", "monitor-review", "migration-review"}
if hosted:
    assert mode in {"npm", "direct", "confirm", "cancel-resume", "deadline", "slow", "late-slow", "parent-exit", "parent-exit-apply"}
source = Path(__file__).resolve().parents[3] if hosted else Path("/source")
platform = "darwin-arm64" if sys.platform == "darwin" else "linux-x64"
candidate = source / "target/debug/hive" if hosted else Path("/build/debug/hive")
root = Path(os.environ["HIVE_LEGACY_FIXTURE_ROOT"]) if hosted else Path("/tmp/hive-first-update")
if hosted:
    assert root.resolve().is_relative_to((source / "tests/work").resolve())
root.mkdir()
user = root / "user"
user.mkdir()
package = root / "package"
(package / "bin").mkdir(parents=True)
shim = root / "shim"
shim.mkdir()
binary = package / "bin/hive"
predecessor = source / "tests/work/public131-posix-0140" / platform / "hive"
old_digest = hashlib.sha256(predecessor.read_bytes()).hexdigest()
registry = json.loads((source / "harness/setup/legacy-update-posix-images.json").read_text())
original = next(item for item in registry["images"] if item["platform"] == platform and item["version"] == "0.13.1")
assert old_digest == original["binary_sha256"]
shutil.copyfile(predecessor, binary)
binary.chmod(0o755)
source_digest = hashlib.sha256(candidate.read_bytes()).hexdigest()
metadata = {"name": "@aigent-hive/" + platform, "version": "0.13.1", "aigentHive": {"productVersion": "0.13.1"}}
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
shutil.copyfile("/usr/bin/env", shim / "env")
(shim / "env").chmod(0o755)
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
elif args==['plugin','uninstall','aigent-hive']:
    if stage.exists():shutil.rmtree(stage)
    print('{}')
else:print('{}')
''')
(shim / "agy").chmod(0o755)
if mode == "host-deselect":
    (shim/"codex").write_text('''#!/usr/bin/python3
import json,os,sys
from pathlib import Path
root=Path('/tmp/hive-first-update/user');args=sys.argv[1:];path=root/'fake-codex.json'
state=json.loads(path.read_text()) if path.exists() else {'marketplace':False,'plugin':False}
with (root/'fake-codex-calls.jsonl').open('a') as log:log.write(json.dumps(args)+'\\n')
market=str(root/'.hive/marketplaces/codex')
if args==['--version']:print('codex-cli 0.145.0');raise SystemExit(0)
if args==['plugin','marketplace','list','--json']:
    print(json.dumps({'marketplaces':[{'name':'aigent-hive','root':market}] if state['marketplace'] else []}));raise SystemExit(0)
if args==['plugin','list','--json']:
    entries=[{'pluginId':'aigent-hive@aigent-hive','version':'0.14.0','enabled':True,'source':{'path':market+'/plugins/aigent-hive'},'marketplaceSource':{'source':market}}] if state['plugin'] else []
    print(json.dumps({'installed':entries+[{'pluginId':'foreign@foreign'}],'available':[]}));raise SystemExit(0)
if args[:3]==['plugin','marketplace','add'] and args[3:]==[market,'--json']:state['marketplace']=True
elif args==['plugin','add','aigent-hive@aigent-hive','--json']:state['plugin']=True
elif args==['plugin','remove','aigent-hive@aigent-hive','--json']:state['plugin']=False
elif args==['plugin','marketplace','remove','aigent-hive','--json']:state['marketplace']=False
else:raise SystemExit('unsupported fake Codex command '+repr(args))
path.write_text(json.dumps(state));print('{}')
''')
    (shim/"codex").chmod(0o755)
if hosted:
    for script in shim.iterdir():
        if script.is_symlink() or script.name == "env":
            continue
        content = script.read_text().replace("/usr/bin/python3", sys.executable)
        content = content.replace("/tmp/hive-first-update", str(root)).replace("/build/debug/hive", str(candidate))
        content = content.replace("@aigent-hive/linux-x64", "@aigent-hive/" + platform)
        if sys.platform == "darwin":
            content = content.replace("subprocess.run(['/usr/bin/strip','--strip-debug',str(binary)],check=True)",
                                      "# Keep the incoming Mach-O code signature intact.")
        script.write_text(content)

environment = {"HOME": str(user), "USERPROFILE": str(user), "PATH": str(shim), "LANG": "C.UTF-8",
               "LC_ALL": "C.UTF-8", "CODEX_HOME": str(root / "codex"),
               "NPM_CONFIG_PREFIX": str(root / "npm-prefix"), "NPM_CONFIG_CACHE": str(root / "npm-cache")}
if mode in {"slow", "late-slow", "parent-exit-apply"}:
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
owned_children = set()


def capture_owned_children(parent):
    if sys.platform != "darwin":
        return
    rows = subprocess.check_output(["/bin/ps", "-axo", "pid=,ppid="], text=True, timeout=5)
    tree = [tuple(map(int, line.split())) for line in rows.splitlines() if line.strip()]
    family = {parent}
    for _ in range(8):
        added = {pid for pid, ppid in tree if ppid in family} - family
        if not added:
            break
        family.update(added)
        assert len(family) < 64
    owned_children.update(family - {parent})
    assert owned_children, "updater child was not observed before terminating its parent"


def mac_process_alive(pid):
    result = subprocess.run(["/bin/ps", "-p", str(pid), "-o", "stat="],
                            capture_output=True, text=True, timeout=5)
    return bool(result.stdout.strip()) and not result.stdout.strip().startswith("Z")

def terminal_run(command, stop=None):
    answers = []
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execve(str(binary), command, environment)
    began = time.monotonic()
    output, pending = b"", b""
    status = None
    killed = False
    first_question = None
    deferred_final = False
    try:
        while time.monotonic() - began < (560 if stop in {"hold", "late-slow"} else 15 if stop == "ctrl-c" else 180):
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
                        if first_question is None:
                            first_question = time.monotonic()
                        if stop == "ctrl-c":
                            if not killed:os.write(descriptor,b"\x03");killed=True
                            reply, kind = None, "ctrl-c"
                        elif stop == "parent-exit":
                            if not killed:
                                capture_owned_children(pid)
                                os.kill(pid, 15)
                                killed = True
                            reply, kind = None, "parent-exit"
                        elif stop == "hold" and ("Korean" in text or "한국어" in text) and "English" in text:
                            reply, kind = None, "held"
                        elif "Apply these changes?" in text or "이 변경을 적용할까요?" in text:
                            if stop == "late-slow":
                                deferred_final = True
                                reply, kind = None, "deferred-final"
                            else:
                                reply, kind = ("no", "decline-final") if stop == "decline-final" else ("yes", "final")
                        elif mode == "host-deselect" and "사용할 앱" in text:
                            reply, kind = "antigravity", "select-antigravity"
                        elif mode == "vector-choice" and "뜻이 비슷한 지식도 찾는 의미 검색" in text:
                            reply, kind = (":cancel", "vector-cancel") if stop == "vector-cancel" else ("yes", "vector-yes") if not terminal_runs else ("no", "vector-no")
                        elif mode == "migration-review" and "사용량에 따라 작업을 막거나 중단하는 기능" in text:
                            reply, kind = "no", "migration-opt-out"
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
                        elif "Choose a language." in text or "사용할 언어를 골라" in text:
                            reply, kind = ("2", "language-en") if mode == "locale-input" and terminal_runs else ("1", "language")
                        elif stop == "cancel":
                            reply, kind = ":cancel", "cancel"
                        else:
                            reply, kind = "", "default"
                    if reply is not None:
                        os.write(descriptor, (reply + "\n").encode())
                        answers.append(kind)
                        pending = b""
            if deferred_final and first_question is not None and time.monotonic() - first_question >= 417:
                os.write(descriptor, b"yes\n")
                answers.append("late-final")
                pending = b""
                deferred_final = False
            if stop == "parent-exit-apply" and not killed and (user / "slow-pid").exists():
                capture_owned_children(pid)
                if sys.platform == "darwin":
                    owned_children.add(int((user / "slow-pid").read_text()))
                os.kill(pid, 15)
                killed = True
            exited, value = os.waitpid(pid, os.WNOHANG)
            if exited:
                status = os.waitstatus_to_exitcode(value)
                break
        assert status == (-2 if stop == "ctrl-c" else -15 if stop in {"parent-exit", "parent-exit-apply"} else 2 if stop in {"cancel", "hold", "slow", "late-slow", "vector-cancel", "decline-final"} else 0), (status, output[-6000:].decode(errors="replace"))
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

if mode in {"confirm", "migration-review"}:
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
    answers = terminal_run(command, "cancel" if mode in {"cancel", "cancel-resume"} else "hold" if mode == "deadline" else mode if mode in {"slow", "late-slow", "parent-exit", "parent-exit-apply"} else None)

config = user / ".hive/config/user-setup.yml"
progress = (user / ".hive/config/setup-progress/user.yml").read_text()
if mode in {"cancel", "cancel-resume", "deadline", "slow", "late-slow", "parent-exit", "parent-exit-apply"}:
    assert "phase: complete" not in progress
    if mode not in {"slow", "late-slow", "parent-exit-apply"}:
        assert not config.exists() and "phase: awaiting-answers" in progress
    if mode in {"cancel", "cancel-resume"}:
        assert "interface_language: ko" in progress
    elif mode == "deadline":
        assert 418 <= terminal_runs[0]["elapsed_seconds"] < 550, terminal_runs
    elif mode in {"slow", "late-slow"}:
        lower, upper = (505, 540) if mode == "late-slow" else (88, 135)
        assert lower <= terminal_runs[0]["elapsed_seconds"] < upper, terminal_runs
        if mode == "late-slow":
            assert "late-final" in answers
        slow_pid = int((user / "slow-pid").read_text())
        if sys.platform == "darwin":
            assert not mac_process_alive(slow_pid), "owned host subprocess survived worker timeout"
        else:
            proc = Path(f"/proc/{slow_pid}/stat")
            assert not proc.exists() or proc.read_text().rsplit(") ", 1)[1].split()[0] == "Z", "owned host subprocess survived worker timeout"
    elif mode in {"parent-exit", "parent-exit-apply"}:
        def live_children():
            if sys.platform == "darwin":
                return [pid for pid in owned_children if mac_process_alive(pid)]
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
    if mode in {"cancel-resume", "deadline", "slow", "late-slow", "parent-exit", "parent-exit-apply"}:
        answers += terminal_run([str(binary), "update", "--user-root", str(user)])
        saved = config.read_text()
        progress = (user / ".hive/config/setup-progress/user.yml").read_text()
        assert "schema_version: 2" in saved and "interface_language: ko" in saved
        assert "enforcement_enabled: false" in saved and "phase: complete" in progress
        assert (root / "install-count").read_text() == "1"
else:
    saved = config.read_text()
    assert "schema_version: 2" in saved and "interface_language: ko" in saved and "phase: complete" in progress
    if mode in {"confirm", "migration-review"}:
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
if mode == "locale-input":
    setup=[str(binary),"setup","--scope","user","--user-root",str(user)]
    def inspect_input(arguments):
        before=config.read_bytes()
        p=subprocess.run(arguments,env=environment,input="",capture_output=True,text=True,timeout=5)
        assert p.returncode!=0 and config.read_bytes()==before,(p.returncode,p.stdout,p.stderr)
        return json.loads(p.stdout)
    invalid=inspect_input([*setup,"--section","missing","--output","json"])
    assert "설정" in invalid["message"],invalid
    conflict=inspect_input([*setup,"--interactive","--output","json"])
    assert "입력" in conflict["message"],conflict
    for flags in [["--interactive"],["--interactive","--non-interactive"],[]]:
        before=config.read_bytes()
        human=subprocess.run([*setup,*flags],env=environment,input="",capture_output=True,text=True,timeout=5)
        assert human.returncode==2 and human.stdout=="" and config.read_bytes()==before,(human.returncode,human.stdout,human.stderr)
    for flags in [["--interactive"],["--non-interactive"],[]]:
        pending=inspect_input([*setup,*flags,"--output","json"])
        if "--interactive" not in flags:assert pending["status"]=="needs-input",pending
    before=config.read_bytes()
    terminal_run([*setup,"--section","language"],"decline-final")
    assert config.read_bytes()==before
    pending=inspect_input([*setup,"--non-interactive","--output","json"])
    assert pending["status"]=="needs-input" and not any('\uac00'<=c<='\ud7a3' for c in pending["message"]),pending
    terminal_run([*setup,"--resume"])
    invalid=inspect_input([*setup,"--section","missing","--output","json"])
    assert "setting" in invalid["message"].lower() and not any('\uac00'<=c<='\ud7a3' for c in invalid["message"]),invalid
    before=config.read_bytes()
    terminal_run([*setup,"--section","language"],"ctrl-c")
    assert config.read_bytes()==before
    terminal_run([*setup,"--resume"])
    project=root/"locale-project";project.mkdir()
    pending=inspect_input([str(binary),"setup","--target",str(project),"--user-root",str(user),"--non-interactive","--output","json"])
    assert pending["status"]=="needs-input" and not (project/".hive").exists()
    assert not any('\uac00'<=c<='\ud7a3' for c in pending["message"]),pending
if mode == "host-deselect":
    setup=[str(binary),"setup","--scope","user","--user-root",str(user)]
    def call(args,expected=0,env=environment):
        p=subprocess.run(args,env=env,capture_output=True,text=True,timeout=90)
        assert p.returncode==expected,(p.returncode,p.stdout[-4000:],p.stderr[-4000:])
        return json.loads(p.stdout) if expected==0 else None
    settings=call([*setup,"--show","--output","json"])["data"]["stored"]
    settings["selected_hosts"]=["codex","antigravity"]
    document=root/"user-answers.json";document.write_text(json.dumps(settings))
    file_args=[*setup,"--answers",str(document),"--input-channel","agent","--output","json"]
    call([*file_args,"--apply"])
    state=user/"fake-codex.json";before=state.read_bytes();config_before=config.read_bytes()
    manifest=json.loads((user/".hive/install/codex.json").read_text())
    guidance=user/manifest["guidance_path"]
    guidance.write_bytes(b"User preface.\n"+guidance.read_bytes()+b"User footer.\n")
    sources={entry["path"]:hashlib.sha256((user/entry["path"]).read_bytes()).hexdigest()
        for entry in manifest["entries"] if entry["ownership"] not in {"shared-marker","shared-migration-state","canonical-data-protected"}}
    foreign=user/".gemini/config/plugins/foreign";foreign.mkdir();(foreign/"data").write_bytes(b"foreign bytes")
    settings["selected_hosts"]=["antigravity"];document.write_text(json.dumps(settings))
    dry=call([*file_args,"--dry-run"])
    assert [plan["host"] for plan in dry["data"]["host_deactivations"]]==["codex"]
    assert state.read_bytes()==before and config.read_bytes()==config_before
    terminal_run([*setup,"--section","hosts"],"decline-final")
    assert state.read_bytes()==before and config.read_bytes()==config_before
    terminal_run([*setup,"--resume"])
    assert json.loads(state.read_text())=={"marketplace":False,"plugin":False}
    assert guidance.read_bytes()==b"User preface.\nUser footer.\n"
    for path,digest in sources.items():assert hashlib.sha256((user/path).read_bytes()).hexdigest()==digest
    assert (foreign/"data").read_bytes()==b"foreign bytes"
    settings["selected_hosts"]=["codex","antigravity"];document.write_text(json.dumps(settings))
    call([*file_args,"--apply"])
    assert all(json.loads(state.read_text()).values())
    settings["selected_hosts"]=["antigravity"];document.write_text(json.dumps(settings))
    call([*file_args,"--apply"],87,dict(environment,HIVE_USER_SETUP_INTERRUPT_AFTER="0"))
    def removes():return [line for line in (user/"fake-codex-calls.jsonl").read_text().splitlines() if '"remove"' in line]
    removed=removes()
    call([*setup,"--resume","--output","json"])
    assert removes()==removed and not any(json.loads(state.read_text()).values())
    assert guidance.read_bytes()==b"User preface.\nUser footer.\n"
    assert (foreign/"data").read_bytes()==b"foreign bytes"
if mode == "vector-choice":
    ledger=user/".hive/config/user-feature-answers.yml"
    first=ledger.read_bytes()
    assert 'vector_search: "yes"' in first.decode() and "input_channel: terminal" in first.decode()
    assert "vector-yes" in answers
    setup=[str(binary),"setup","--scope","user","--user-root",str(user)]
    kept=terminal_run([*setup,"--section","persona"])
    assert "vector-yes" not in kept and "vector-no" not in kept and ledger.read_bytes()==first
    terminal_run([*setup,"--section","vector"],"vector-cancel")
    assert ledger.read_bytes()==first
    resumed=terminal_run([*setup,"--resume"])
    assert "vector-no" in resumed and 'vector_search: "no"' in ledger.read_text()
    assert not (user/".hive/index/vector").exists()
    assert not (user/".hive/config/vector-state").exists()
    shown=subprocess.run([*setup,"--show","--output","json"],env=environment,capture_output=True,text=True,check=True)
    feature=json.loads(shown.stdout)["data"]
    assert feature["feature_preferences"]["vector_search"]=="no"
    assert feature["vector_runtime_verification"]=="not-performed"
    assert not (user/".hive/runtime/usage-monitor/config.json").exists()
    assert (root/"install-count").read_text()=="1"
if mode in {"project-file", "project-file-interrupt"}:
    project=root/"project";project.mkdir()
    (project/"AGENTS.md").write_text("User-owned preface.\n")
    document=root/"project-answers.yml"
    document.write_bytes(Path("/source/tests/fixtures/setup/answers-no-role-no-hook.yml").read_bytes())
    capfile=root/"project-caps.json"
    capfile.write_bytes(Path("/source/tests/fixtures/setup/capabilities-codex-host-native.json").read_bytes())
    args=[str(binary),"setup","--target",str(project),"--user-root",str(user),"--answers",str(document),
        "--capabilities",str(capfile),"--input-channel","agent","--output","json"]
    def project_command(command,expected=0,env=None):
        p=subprocess.run(command,env=environment if env is None else env,capture_output=True,text=True,timeout=90)
        assert p.returncode==expected,(p.returncode,p.stdout[-4000:],p.stderr[-4000:])
        return json.loads(p.stdout) if expected==0 else None
    global_before=config.read_bytes()
    project_command([*args,"--dry-run"])
    assert not (project/".hive").exists()
    if mode=="project-file-interrupt":
        interrupted=dict(environment,HIVE_PROJECT_SETUP_INTERRUPT_AFTER="intent")
        project_command([*args,"--apply"],87,interrupted)
        records=list((user/".hive/config/setup-progress/projects").glob("*.yml"))
        assert len(records)==1 and "phase: applying" in records[0].read_text()
        changed=root/"different-project.yml";changed.write_text(document.read_text().replace("phase1-parity","different-project"))
        changed_args=[str(changed) if arg==str(document) else arg for arg in args]
        p=subprocess.run([*changed_args,"--apply"],env=environment,capture_output=True,text=True,timeout=30)
        assert p.returncode!=0 and not (project/".hive/setup-answers.yml").exists()
        capfile.write_bytes(Path("/source/tests/fixtures/setup/capabilities-claude-host-native.json").read_bytes())
        p=subprocess.run([*args,"--apply"],env=environment,capture_output=True,text=True,timeout=30)
        assert p.returncode!=0 and "capability evidence differs" in p.stdout,(p.returncode,p.stdout[-1000:])
        assert not (project/".hive/setup-answers.yml").exists()
        capfile.write_bytes(Path("/source/tests/fixtures/setup/capabilities-codex-host-native.json").read_bytes())
        p=subprocess.run([*args,"--reconfigure-role","unexpected-role","--apply"],env=environment,capture_output=True,text=True,timeout=30)
        assert p.returncode!=0 and "differs from unfinished setup" in p.stdout,(p.returncode,p.stdout[-1000:])
        assert not (project/".hive/setup-answers.yml").exists()
        document.unlink();capfile.unlink()
        result=project_command([str(binary),"setup","--target",str(project),"--user-root",str(user),"--resume","--output","json"])
        assert result["code"]=="hive.project-setup-complete"
    else:
        result=project_command([*args,"--apply"]);assert result["code"]=="hive.setup-complete"
        saved=(project/".hive/setup-answers.yml").read_bytes()
        project_command([*args,"--validate"])
        assert project_command([*args,"--apply"])["changed_paths"]==[]
        assert (project/".hive/setup-answers.yml").read_bytes()==saved
        document.write_text(document.read_text().replace("phase1-parity","renamed-project"))
        (project/"AGENTS.md").write_text((project/"AGENTS.md").read_text()+"\nUser footer.\n")
        project_command([*args,"--apply"])
        assert (project/"AGENTS.md").read_text().endswith("User footer.\n")
    shown=project_command([str(binary),"setup","--target",str(project),"--user-root",str(user),"--show","--output","json"])
    stored=shown["data"]["stored"]
    assert stored["schema_version"]==2 and stored["preference_decisions"]["/project_name"]["input_channel"]=="agent"
    assert stored["preference_decisions"]["/interface_language"]["origin"]=="inherited"
    assert (project/"AGENTS.md").read_text().startswith("User-owned preface.\n")
    assert config.read_bytes()==global_before
if mode == "migration-review":
    lifecycle = user / ".hive/history/lifecycle"
    migrated = [(path, json.loads(path.read_text())) for path in lifecycle.glob("*.json")]
    migrated = [(path, record) for path, record in migrated if record.get("migration", {}).get("from_schema") == 1]
    assert len(migrated) == 1, migrated
    migration_path, record = migrated[0]
    assert record["migration"]["to_schema"] == 2 and record["migration"]["policy_version"] == "0.14.0"
    assert record["phase"] == "complete" and record["before_digest"] != record["after_digest"]
    migration_receipt = migration_path.read_bytes()
    answers += terminal_run([str(binary), "setup", "--scope", "user", "--user-root", str(user)])
    after_choice = config.read_bytes()
    assert b"enforcement_enabled: false" in after_choice and "migration-opt-out" in answers
    shown = subprocess.run([str(binary), "setup", "--scope", "user", "--show", "--user-root", str(user), "--output", "json"],
                           env=environment, capture_output=True, text=True, check=True)
    settings = json.loads(shown.stdout)["data"]
    decision = settings["stored"]["preference_decisions"]["/usage_guard/enforcement_enabled"]
    assert decision["origin"] == "explicit" and decision["input_channel"] == "terminal"
    assert settings["pending_questions"] == []
    # The frozen 0.13.1 handshake used the public 0.13.2-test.2 label. The following
    # command is owned by development 0.14.0, so authenticate its private fixture as 0.14.0.
    (package / "package.json").write_text(json.dumps({"name": "@aigent-hive/linux-x64",
        "version": "0.14.0", "aigentHive": {"productVersion": "0.14.0"}}))
    later = subprocess.run([str(binary), "update", "--channel", "test", "--confirm", "--non-interactive", "--user-root", str(user), "--output", "json"],
                           env=environment, input="", capture_output=True, text=True, timeout=30)
    assert later.returncode == 0, (later.stdout, later.stderr)
    assert config.read_bytes() == after_choice and migration_path.read_bytes() == migration_receipt
    assert (root / "install-count").read_text() == "1"
if mode == "monitor-review":
    import runpy
    runpy.run_path("/source/tests/fixtures/interactive-cli/linux-monitor-review.py", init_globals={
        "binary": binary, "root": root, "user": user, "environment": environment, "terminal_run": terminal_run})
incoming = json.loads((root / "incoming.json").read_text())
assert incoming["sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
evidence = {"scenario": mode, "predecessor_sha256": old_digest, "incoming_source_sha256": source_digest,
    "incoming_stripped_sha256": incoming["sha256"], "installer_stub": incoming["installer"], "answers": answers,
    "original_parent": "public 0.13.1 " + platform, "incoming": "development 0.14.0",
    "registry_target": "0.13.2-test.2", "private_user_mount": False, "terminal_runs": terminal_runs,
    "installer_invocations": int((root / "install-count").read_text()),
    "scope": ("Hosted isolated POSIX VM" if hosted else "Linux Docker") + " PTY/pipe component test; fake installer and Antigravity; no model, actual host or public14 acceptance"}
if mode == "migration-review":
    evidence["followup"] = {"private_package_version": "0.14.0", "registry_candidate": "0.13.2-test.2",
        "settings_preserved": True, "original_migration_receipt_preserved": True,
        "no_reinstall": True, "scope": "current development package reconciliation; not public 0.14 acceptance"}
results = root / "results" if hosted else Path("/results")
results.mkdir(exist_ok=True)
(results / (mode + ".json")).write_text(json.dumps(evidence, indent=2) + "\n")
print(json.dumps(evidence))
