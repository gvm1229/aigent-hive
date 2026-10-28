"""Qualify real idle monitoring only in a disposable Windows hosted runner."""
import copy
from contextlib import contextmanager, nullcontext
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

assert sys.argv[1:] in ([], ["prepare"], ["recovery"], ["current-direct"], ["uninstall"], ["real-codex"], ["helper-recovery"])
prepare_only = sys.argv[1:] == ["prepare"]
recovery_mode = sys.argv[1:] == ["recovery"]
direct_mode = sys.argv[1:] == ["current-direct"]
real_codex = sys.argv[1:] == ["real-codex"]
helper_recovery = sys.argv[1:] == ["helper-recovery"]
uninstall_mode = sys.argv[1:] in (["uninstall"], ["real-codex"], ["helper-recovery"])
if not prepare_only and not (os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires isolated GitHub-hosted Windows; personal installations excluded")

(ROOT / "tests/work").mkdir(parents=True, exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="windows-monitor-ci-", dir=ROOT / "tests/work")).resolve()
user = work / "user"
user.mkdir()
fake_bin = work / "fake-bin"
fake_bin.mkdir()
candidate = ROOT / "target/debug/hive.exe"
binary = work / "prefix/bin/hive.exe" if direct_mode or uninstall_mode else work / "hive.exe"
binary.parent.mkdir(parents=True, exist_ok=True)
shutil.copyfile(candidate, binary)
environment = {"PATH": str(fake_bin), "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "CLAUDE_CONFIG_DIR",
            "GEMINI_CLI_HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "TEMP", "TMP",
            "NPM_CONFIG_PREFIX", "NPM_CONFIG_CACHE"]:
    directory = work / key
    directory.mkdir()
    environment[key] = str(directory)
for key in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"]:
    if key in os.environ:
        environment[key] = os.environ[key]
environment["PSModuleAnalysisCachePath"] = str(work / "powershell-module-cache")
environment["HIVE_TEST_USER_ROOT"] = str(user)
environment["HIVE_TEST_HOST_STATE"] = str(work / "host-state.json")

# The plugin command stand-in cannot control the desktop or scheduled controller.
# Reuse the existing fixed Codex fixture without its inherited-environment runner.
import ast
fixture_source = ROOT / "tests/conformance/integration/test_connected_setup_lifecycle.py"
tree = ast.parse(fixture_source.read_text(encoding="utf-8"))
fixture_class = next(n for n in tree.body if isinstance(n, ast.ClassDef)
                     and n.name == "ConnectedSetupLifecycleConformance")
method = next(n for n in fixture_class.body if isinstance(n, ast.FunctionDef)
              and n.name == "_write_fake_host")
program = next(n.value for n in ast.walk(method) if isinstance(n, ast.Constant)
               and isinstance(n.value, str) and n.value.startswith("import json"))
program = program.replace("__HIVE_TEST_HOST__", repr("codex")).replace(
    "__HIVE_TEST_PRODUCT_VERSION__", repr("0.14.0"))
if not real_codex:
    (fake_bin / "codex.py").write_text(program, encoding="utf-8")
    (fake_bin / "codex.cmd").write_text(
        f'@"{sys.executable}" "%~dp0codex.py" %*\r\n', encoding="utf-8")

sha = lambda data: hashlib.sha256(data).hexdigest()
events = []
run = Run("격리 Windows 제거 보조 처리의 실제 잠금 실패와 재개" if helper_recovery else "격리 Windows 공개 Codex CLI의 실제 연결·보존 제거" if real_codex else "격리 Windows 활성 감시의 보존 제거·실패 재개·재설치" if uninstall_mode else "Windows 감시 설정의 입력·별도 동의 준비" if prepare_only else ("격리 Windows 현재 직접 설치의 부모 완료" if direct_mode else ("격리 Windows 실제 감시의 중단·철회·연속 갱신 복구" if recovery_mode else "격리 Windows 실제 감시의 별도 동의·대기 실행·해제·재활성")),
    [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
    paths=[work.relative_to(ROOT).as_posix()], binary=candidate)
runtime = user / ".hive/runtime/usage-monitor"
config_path = user / ".hive/config/user-setup.yml"
monitor_path = runtime / "config.json"
activation_path = user / ".hive/config/setup-progress/monitor-activation.json"
code = 1


def invoke(*args, expected=0, extra=None):
    result = subprocess.run([str(binary), *args, "--output", "json"], env={**environment, **(extra or {})},
        cwd=user if real_codex else None, stdin=subprocess.DEVNULL, capture_output=True, timeout=150)
    response = json.loads(result.stdout) if result.stdout.strip() else {}
    events.append({"arguments": list(args), "exit_code": result.returncode,
                   "code": response.get("code"), "message": response.get("message"), "faults": extra or {},
                   "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)})
    assert result.returncode == expected, (args, result.returncode, response, result.stderr[-1000:])
    return response


def status():
    result = invoke("usage", "monitor", "status", "--user-root", str(user))["data"]
    events.append({"monitor_status": result})
    return result


def assert_idle():
    state = status()
    assert state["controller_alive"] and state["controller_heartbeat_fresh"], state
    assert state["sensor_health"] == "idle" and not state["monitoring_verified"], state
    assert state["state"]["observation_count"] == 0, state
    return state


def write_answers(name, settings):
    path = work / name
    path.write_text(json.dumps(settings), encoding="utf-8")
    return str(path)


def install_real_codex():
    version = "0.165.0"
    integrity = "sha512-aiwtQVp2KBhJld966CuU5bVfQ46VNB4mNTl6giX3WGZdt4euI3tv2RUqJpnupPNA8DXHg2z+RoQ0Zfw87dy8Lw=="
    with urllib.request.urlopen("https://registry.npmjs.org/@openai%2fcodex/" + version, timeout=20) as response:
        metadata_bytes = response.read(512 * 1024 + 1)
    assert len(metadata_bytes) <= 512 * 1024
    metadata = json.loads(metadata_bytes)
    assert metadata["name"] == "@openai/codex" and metadata["version"] == version
    assert metadata["dist"]["integrity"] == integrity
    node = Path(shutil.which("node")).resolve()
    npm_cmd = Path(shutil.which("npm.cmd")).absolute()
    npm_js = npm_cmd.parent / "node_modules/npm/bin/npm-cli.js"
    assert node.is_file() and npm_js.is_file()
    tools_prefix = work / "host-tools"
    tools_prefix.mkdir()
    environment.update(HOME=str(user), USERPROFILE=str(user), CODEX_HOME=str(user / ".codex"),
        NPM_CONFIG_PREFIX=str(tools_prefix), NPM_CONFIG_USERCONFIG=str(work / "host-user.npmrc"),
        NPM_CONFIG_GLOBALCONFIG=str(work / "host-global.npmrc"), NPM_CONFIG_REGISTRY="https://registry.npmjs.org",
        NPM_CONFIG_AUDIT="false", NPM_CONFIG_FUND="false", NPM_CONFIG_IGNORE_SCRIPTS="true",
        PATH=os.pathsep.join([str(tools_prefix), str(node.parent)]))
    install = subprocess.run([str(node), str(npm_js), "install", "--global", "--prefix", str(tools_prefix),
        "@openai/codex@" + version], env=environment, cwd=user, capture_output=True, timeout=180)
    events.append({"real_codex_install_exit": install.returncode, "package": version,
        "integrity": integrity, "metadata_sha256": sha(metadata_bytes), "node_sha256": sha(node.read_bytes()),
        "npm_entry_sha256": sha(npm_js.read_bytes()), "stderr": install.stderr.decode(errors="replace")[-1500:]})
    assert install.returncode == 0, events[-1]
    images = list((tools_prefix / "node_modules/@openai").rglob("codex.exe"))
    assert len(images) == 1 and images[0].resolve().is_relative_to(tools_prefix)
    environment["HIVE_TEST_REAL_CODEX_BINARY"] = str(images[0])
    events.append({"real_codex_binary_sha256": sha(images[0].read_bytes())})
    for args in [("--version",), ("plugin", "list", "--help"), ("plugin", "add", "--help"),
                 ("plugin", "remove", "--help"), ("plugin", "marketplace", "list", "--help")]:
        probe = subprocess.run([str(images[0]), *args], env=environment, cwd=user,
            capture_output=True, timeout=20)
        text = (probe.stdout + probe.stderr).decode(errors="replace")
        events.append({"real_codex_probe": list(args), "exit_code": probe.returncode,
                       "output_sha256": sha(probe.stdout + probe.stderr), "output": text[:2000]})
        assert probe.returncode == 0, "public Codex command unavailable: " + text[:1000]
        if "--help" in args:
            assert "--json" in text, "public Codex command lacks required structured output"


def current_direct_update():
    receipt = work / "prefix/share/aigent-hive/install-receipt.json"
    receipt.parent.mkdir(parents=True)
    receipt.write_text(json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
        "version": "0.14.0", "package_version": "0.14.0", "artifact_sha256": "sha256:" + sha(binary.read_bytes())}))
    with urllib.request.urlopen("https://registry.npmjs.org/aigent-hive", timeout=15) as response:
        metadata = response.read(1024 * 1024 + 1)
    assert len(metadata) <= 1024 * 1024
    latest = json.loads(metadata)["dist-tags"]["latest"]
    assert tuple(map(int, latest.split('.'))) <= (0, 14, 0), "newer external release changes this fixture"
    events.append({"registry_before_sha256": sha(metadata), "latest": latest})
    before = {path: path.read_bytes() for path in [binary, receipt, config_path]}
    command = [str(binary), "update", "--confirm", "--user-root", str(user), "--output", "json"]
    result = subprocess.run(command, env=environment, input=b"", capture_output=True, timeout=180)
    parent = json.loads(result.stdout)
    events.append({"arguments": command[1:], "update_parent_exit": result.returncode, "update_parent": parent})
    if parent["code"] == "hive.update-handoff-pending":
        record = Path(parent["data"]["completion_record"])
        assert record.parent.samefile(user / ".hive/runtime/update")
        assert record.suffix == ".json" and len(record.stem) == 64 and all(c in "0123456789abcdef" for c in record.stem)
        until = time.monotonic() + 120
        while time.monotonic() < until:
            try:
                completed = json.loads(record.read_bytes())
                events.append({"old_handoff_result": completed})
                break
            except json.JSONDecodeError:
                time.sleep(.1)
        else:
            raise AssertionError("owned handoff completion did not arrive")
    assert result.returncode == 0, parent
    assert parent["code"] == "hive.update-complete" and parent["data"]["steps"]["core"] == "unchanged", parent
    assert all(path.read_bytes() == data for path, data in before.items())


def task_description(expected, replacement=None):
    config = json.loads(monitor_path.read_text())
    assert config["task_name"] == "AigentHive-UsageMonitor-" + sha(str(user).lower().encode())[:16]
    source = r"""
$ErrorActionPreference='Stop'
Import-Module -Name @(($PSHOME+'\Modules\Microsoft.PowerShell.Utility\Microsoft.PowerShell.Utility.psd1'),($PSHOME+'\Modules\CimCmdlets\CimCmdlets.psd1'),($PSHOME+'\Modules\ScheduledTasks\ScheduledTasks.psd1'))
$s=[Console]::In.ReadToEnd() | ConvertFrom-Json
$task=Get-ScheduledTask -TaskName $s.task_name -ErrorAction Stop
if($task.TaskPath -ne '\' -or $task.Description -ne $s.expected -or @($task.Actions).Count -ne 1 -or
    $task.Actions[0].Execute -ne $s.launcher.executable -or $task.Actions[0].Arguments -ne $s.launcher.arguments){throw 'Fixture task identity changed'}
$owner=[string]$task.Principal.UserId
$sid=if($owner -match '^S-1-'){([Security.Principal.SecurityIdentifier]::new($owner)).Value}else{([Security.Principal.NTAccount]::new($owner)).Translate([Security.Principal.SecurityIdentifier]).Value}
if($sid -ne [Security.Principal.WindowsIdentity]::GetCurrent().User.Value -or $task.Principal.LogonType -notin @('Interactive',3) -or $task.Principal.RunLevel -notin @('Limited',0)){throw 'Fixture task principal changed'}
if($null -ne $s.replacement){
    $task.Description=$s.replacement
    $null=Set-ScheduledTask -InputObject $task
    $task=Get-ScheduledTask -TaskName $s.task_name -ErrorAction Stop
    if($task.Description -ne $s.replacement){throw 'Fixture description change was not applied'}
}
[Console]::WriteLine('fixture-task-verified')
"""
    ps = Path(environment["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
    result = subprocess.run([str(ps), "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", source],
        env=environment, input=json.dumps({"task_name": config["task_name"], "launcher": config["launcher"],
            "expected": expected, "replacement": replacement}).encode(), capture_output=True, timeout=20)
    events.append({"fixture_task_description": "changed" if replacement else "checked", "exit_code": result.returncode})
    assert result.returncode == 0, result.stderr.decode(errors="replace")[-1500:]


def recovery_cases(base, on_file, off_file, consent):
    lifecycle = user / ".hive/history/lifecycle"
    invoke(*base, "--answers", on_file, "--monitor-confirm", consent,
           "--apply", expected=90, extra={"HIVE_MONITOR_ACTIVATION_INTERRUPT_AFTER": "Registered"})
    pending = invoke(*base, "--show")["data"]["pending"]
    operation = pending["operation_id"]
    activation = json.loads(activation_path.read_text())
    assert activation["phase"] == "registered"
    resumed = invoke(*base, "--resume", "--non-interactive")
    assert resumed["data"]["operation_id"] == operation
    assert json.loads(activation_path.read_text())["operation_id"] == activation["operation_id"]
    assert_idle()
    saved_policy = config_path.read_bytes()
    activation_before = activation_path.read_bytes()
    for _ in range(2):
        update = invoke("update", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
        assert update["data"]["monitor_upgrade"]["health"]["state"] == "idle-awaiting-Codex", update
        assert config_path.read_bytes() == saved_policy
        assert activation_path.read_bytes() == activation_before
        assert_idle()
    owned_description = "Aigent Hive version-bound Codex usage protection"
    conflict_description = owned_description + " / " + work.name
    task_description(owned_description, conflict_description)
    invoke(*base, "--answers", off_file, "--apply", expected=5)
    assert config_path.read_bytes() == saved_policy
    assert json.loads((runtime / "disable.json").read_text())["phase"] == "incomplete"
    assert not invoke(*base, "--show")["data"]["actual_monitor_state"]["stop_verified"]
    task_description(conflict_description)
    task_description(conflict_description, owned_description)
    invoke(*base, "--resume", "--non-interactive")
    assert not status()["controller_alive"]

    # This time stop only after the actual native controller has been verified.
    next_consent = invoke(*base, "--answers", on_file, "--dry-run")["data"]["monitor_activation"]["confirmation_digest"]
    invoke(*base, "--answers", on_file, "--monitor-confirm", next_consent,
           "--apply", expected=90, extra={"HIVE_MONITOR_ACTIVATION_INTERRUPT_AFTER": "Complete"})
    pending = invoke(*base, "--show")["data"]["pending"]
    assert_idle()
    review = invoke(*base, "--resume", "--review", "--dry-run")["data"]
    assert review["operation_id"] == pending["operation_id"]
    answer = write_answers("withdrawal.json", {
        "question_id": "user-monitor-withdrawal", "question_revision": 1,
        "answer": True, "confirmed": True, "preview_digest": review["preview_digest"],
        "input_channel": "answers-file"})
    invoke(*base, "--resume", "--review", "--review-answers", answer, expected=89,
           extra={"HIVE_TEST_SETUP_REVIEW_EXIT_AFTER": "stopped"})
    assert not status()["controller_alive"]
    resumed = invoke(*base, "--resume", "--non-interactive")
    assert resumed["data"]["monitor_review"]["withdrawn"], resumed
    old = json.loads((lifecycle / (pending["operation_id"] + ".json")).read_text())
    assert old["phase"] == "recoverable-failure"
    replacement = old["cancelled_by_operation"]
    assert replacement != pending["operation_id"]
    assert json.loads((lifecycle / (replacement + ".json")).read_text())["phase"] == "complete"
    actual = invoke(*base, "--show")["data"]["stored"]
    desired = copy.deepcopy(pending["answers"])
    desired["usage_guard"]["midway"]["enabled"] = False
    for value in [desired, actual]:
        value.pop("preference_decisions", None)
    assert desired == actual
    assert not (user / ".hive/config/setup-progress/monitor-handoff.json").exists()
    saved_policy = config_path.read_bytes()
    retained = {path: path.read_bytes() for path in [lifecycle / (pending["operation_id"] + ".json"),
                                                    lifecycle / (replacement + ".json")]}
    for _ in range(2):
        invoke("update", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
        assert config_path.read_bytes() == saved_policy
        assert all(path.read_bytes() == data for path, data in retained.items())
        assert not status()["controller_alive"]


@contextmanager
def hold_fixture_image():
    import ctypes
    from ctypes import wintypes
    assert binary.resolve().is_relative_to(work) and not binary.is_symlink()
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
        wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    kernel.CreateFileW.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    handle = kernel.CreateFileW(str(binary), 0x80000000, 1, None, 3, 128, None)
    assert handle not in (None, ctypes.c_void_p(-1).value), ctypes.get_last_error()
    try:
        yield
    finally:
        assert kernel.CloseHandle(handle), ctypes.get_last_error()


def wait_for_removal_helper(path, operation):
    expected = user / ".hive/maintenance/uninstall" / operation / "hive.exe"
    assert path.resolve() == expected.resolve()
    ps = Path(environment["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
    result = subprocess.run([str(ps), "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
        "$ErrorActionPreference='Stop';$path=[IO.Path]::GetFullPath([Console]::In.ReadToEnd());"
        "$name=[IO.Path]::GetFileNameWithoutExtension($path);"
        "foreach($p in [Diagnostics.Process]::GetProcessesByName($name)){"
        "if([IO.Path]::GetFullPath($p.MainModule.FileName) -ne $path){continue};"
        "if(-not $p.WaitForExit(5000)){throw 'Owned helper has not exited'}}"],
        env=environment, input=str(path).encode(), capture_output=True, timeout=15)
    assert result.returncode == 0, result.stderr


def uninstall_case(base, on_file, consent):
    version = subprocess.run([str(binary), "--version"], env=environment, capture_output=True, timeout=15)
    assert version.returncode == 0
    match = re.match(r"AIgent Hive v(0\.14\.0(?:-test)?)(?: #(\d+))? ", version.stdout.decode("utf-8"))
    assert match, version.stdout
    package = match[1] + ("." + match[2] if match[2] else "")
    receipt = work / "prefix/share/aigent-hive/install-receipt.json"
    receipt.parent.mkdir(parents=True)
    receipt_bytes = json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
        "version": "0.14.0", "package_version": package, "artifact_sha256": "sha256:" + sha(binary.read_bytes())}).encode()
    receipt.write_bytes(receipt_bytes)
    invoke(*base, "--answers", on_file, "--monitor-confirm", consent, "--apply")
    assert_idle()
    preserved = {config_path: config_path.read_bytes()}
    for path, data in {
        user / ".hive/knowledge/preserved.md": b"Canonical synthetic knowledge.\n",
        user / ".hive/config/projects.yml": b"projects: []\n",
        user / "project/README.md": b"Independent project bytes.\r\n",
        work / "prefix/foreign.txt": b"Unowned prefix bytes.\n",
    }.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        preserved[path] = data
    for name in ["user-preferences.json", "user-feature-answers.yml"]:
        path = user / ".hive/config" / name
        if path.is_file():
            preserved[path] = path.read_bytes()
    original_image = sha(binary.read_bytes())
    preview = invoke("uninstall", "--user-root", str(user), "--dry-run")
    assert preview["data"]["preview"]["core"]["owner"] == "direct", preview
    assert preview["data"]["preview"]["monitor"]["configured"], preview
    invoke("uninstall", "--user-root", str(user), expected=2)
    assert_idle()
    assert sha(binary.read_bytes()) == original_image
    assert all(path.read_bytes() == data for path, data in preserved.items())
    owned = "Aigent Hive version-bound Codex usage protection"
    conflict = owned + " / " + work.name
    task_description(owned, conflict)
    refused = invoke("uninstall", "--user-root", str(user), "--confirm", expected=5)
    assert not refused["data"]["completion_verified"] and refused["data"]["steps"]["monitor"] == "unverified", refused
    assert sha(binary.read_bytes()) == original_image
    assert all(path.read_bytes() == data for path, data in preserved.items())
    task_description(conflict, owned)
    with hold_fixture_image() if helper_recovery else nullcontext():
        removed = invoke("uninstall", "--user-root", str(user), "--resume", refused["data"]["operation_id"], expected=2)
        if helper_recovery:
            history = user / removed["data"]["receipt"]
            assert history.resolve().is_relative_to((user / ".hive/history/lifecycle").resolve())
            deadline = time.monotonic() + 40
            while time.monotonic() < deadline:
                failed = json.loads(history.read_text())
                if failed["last_error"]:
                    break
                time.sleep(.1)
            assert failed["phase"] == "files-removed" and "remained locked" in (failed["last_error"] or ""), failed
            assert sha(binary.read_bytes()) == original_image
            wait_for_removal_helper(user / removed["data"]["helper"], removed["data"]["operation_id"])
            events.append({"helper_failure_observed": True, "receipt": failed})
    if helper_recovery:
        removed = invoke("uninstall", "--user-root", str(user), "--resume", refused["data"]["operation_id"], expected=2)
    assert removed["data"]["operation_id"] == refused["data"]["operation_id"]
    history = user / removed["data"]["receipt"]
    assert history.resolve().is_relative_to((user / ".hive/history/lifecycle").resolve())
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        completed = json.loads(history.read_text())
        if completed["phase"] == "complete":
            break
        time.sleep(.1)
    assert completed["phase"] == "complete", completed.get("last_error")
    assert not binary.exists() and not receipt.exists()
    if real_codex:
        listed = subprocess.run([environment["HIVE_TEST_REAL_CODEX_BINARY"], "plugin", "list", "--json"],
            env=environment, cwd=user, capture_output=True, timeout=20)
        assert listed.returncode == 0, listed.stderr
        installed = json.loads(listed.stdout)["installed"]
        assert isinstance(installed, list)
        assert all(entry["pluginId"] != "aigent-hive@aigent-hive" for entry in installed)
        events.append({"real_codex_removed": True, "list_sha256": sha(listed.stdout)})
    else:
        assert not json.loads((work / "host-state.json").read_text())["plugin"]
    assert all(path.read_bytes() == data for path, data in preserved.items())
    monitor = json.loads(monitor_path.read_text())
    assert not monitor["enabled"] and json.loads((runtime / "disable.json").read_text())["phase"] == "complete"
    task_name = monitor["task_name"]
    assert task_name == "AigentHive-UsageMonitor-" + sha(str(user).lower().encode())[:16]
    ps = Path(environment["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
    absent = subprocess.run([str(ps), "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
        "$ErrorActionPreference='Stop';Import-Module ($PSHOME+'\\Modules\\ScheduledTasks\\ScheduledTasks.psd1');"
        "$name=[Console]::In.ReadToEnd();foreach($task in @(Get-ScheduledTask -TaskPath '\\' -ErrorAction Stop))"
        "{if($task.TaskName -eq $name){exit 1}}"],
        env=environment, input=task_name.encode(), capture_output=True, timeout=20)
    assert absent.returncode == 0, absent.stderr
    events.append({"removal_complete": True, "task_absent": True, "receipt": completed,
        "preserved_sha256": {str(path.relative_to(work)): sha(data) for path, data in preserved.items()}})
    binary.parent.mkdir(parents=True, exist_ok=True)
    receipt.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(candidate, binary)
    receipt.write_bytes(receipt_bytes)
    invoke("install", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
    assert all(path.read_bytes() == data for path, data in preserved.items())
    assert not status()["controller_alive"]
    assert not json.loads(monitor_path.read_text())["enabled"]


try:
    if real_codex:
        install_real_codex()
    if not prepare_only and not direct_mode:
        # One environment availability check, before creating any native registration.
        ps = Path(environment["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        probe = subprocess.run([str(ps), "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "$ErrorActionPreference='Stop';"
            "[void][Reflection.Assembly]::Load('System.ServiceProcess, Version=4.0.0.0, "
            "Culture=neutral, PublicKeyToken=b03f5f7f11d50a3a');"
            "[Console]::WriteLine('service-type-loaded');"
            "$s=[System.ServiceProcess.ServiceController]::new('Schedule');"
            "if($s.Status -ne 'Running'){throw 'Task Scheduler unavailable'};"
            "[Console]::WriteLine('scheduler-running');"
            "if([Diagnostics.Process]::GetProcessesByName('ChatGPT').Count -ne 0)"
            "{throw 'Desktop app present; idle fixture refused'};[Console]::WriteLine('isolated-idle-ready')"],
            env=environment, capture_output=True, timeout=20)
        events.append({"availability_exit": probe.returncode,
                       "availability_output": probe.stdout.decode(errors="replace").strip(),
                       "availability_error": probe.stderr.decode(errors="replace")[-1000:]})
        assert probe.returncode == 0, events[-1]
    base = ["setup", "--scope", "user", "--user-root", str(user)]
    defaults = invoke("setup", "--scope", "user", "--describe")["data"]["expedited_defaults"]
    defaults.update(interface_language="ko", selected_hosts=["codex"])
    defaults["wiki"]["enabled"] = False
    off_file = write_answers("off.json", defaults)
    invoke("install", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
    invoke(*base, "--answers", off_file, "--apply")
    if direct_mode:
        current_direct_update()
    else:
        initial_config = config_path.read_bytes()
        on = copy.deepcopy(defaults)
        on["usage_guard"]["enforcement_enabled"] = True
        on["usage_guard"]["midway"]["enabled"] = True
        on["usage_guard"]["midway"]["reset_enabled"] = True
        on_file = write_answers("on.json", on)
        preview = invoke(*base, "--answers", on_file, "--dry-run")
        consent = preview["data"]["monitor_activation"]
        assert consent["platform_eligible"] and consent["requires_separate_confirmation"], preview
        assert consent["runtime_verification"] == "not-performed", preview
        invoke(*base, "--answers", on_file, "--apply", expected=2)
        assert config_path.read_bytes() == initial_config and not monitor_path.exists()
        if uninstall_mode:
            uninstall_case(base, on_file, consent["confirmation_digest"])
        elif recovery_mode:
            recovery_cases(base, on_file, off_file, consent["confirmation_digest"])
        elif not prepare_only:
            invoke(*base, "--answers", on_file, "--monitor-confirm", consent["confirmation_digest"], "--apply")
            assert_idle()
            first_activation = json.loads(activation_path.read_text())
            assert first_activation["phase"] == "complete", first_activation
            history = runtime / "retained-history.txt"
            history.write_bytes(b"Owned synthetic history; preserve across disable and activation.\n")
            preserved = history.read_bytes()
            invoke(*base, "--answers", off_file, "--apply")
            stopped = status()
            assert not stopped["controller_alive"] and not stopped["verified_saved_selection"], stopped
            assert json.loads((runtime / "disable.json").read_text())["phase"] == "complete"
            assert history.read_bytes() == preserved
            # A new preview/consent is necessary after this later disable selection.
            preview = invoke(*base, "--answers", on_file, "--dry-run")
            next_consent = preview["data"]["monitor_activation"]["confirmation_digest"]
            assert next_consent != consent["confirmation_digest"]
            invoke(*base, "--answers", on_file, "--monitor-confirm", consent["confirmation_digest"],
                   "--apply", expected=2)
            assert not json.loads(monitor_path.read_text())["enabled"]
            invoke(*base, "--answers", on_file, "--monitor-confirm", next_consent, "--apply")
            assert_idle()
            second = json.loads(activation_path.read_text())
            assert second["phase"] == "complete" and second["operation_id"] != first_activation["operation_id"]
            assert history.read_bytes() == preserved
            invoke(*base, "--answers", off_file, "--apply")
            assert not status()["controller_alive"]
    code = 0
except BaseException as failure:
    events.append({"failure": str(failure)[:2500]})
    if isinstance(failure, subprocess.TimeoutExpired):
        events.append({"timed_out_stdout": (failure.stdout or b"").decode(errors="replace")[-1500:],
                       "timed_out_stderr": (failure.stderr or b"").decode(errors="replace")[-1500:]})
    raise
finally:
    # Product ownership checks govern cleanup; no task-name glob or process-name kill.
    if not prepare_only and monitor_path.exists() and json.loads(monitor_path.read_text()).get("enabled"):
        try:
            invoke("usage", "monitor", "disable", "--user-root", str(user))
        except Exception as failure:
            events.append({"cleanup_error": str(failure)[:1500]})
            code = 1
    diagnostics = {}
    if activation_path.is_file():
        diagnostics["activation.json"] = json.loads(activation_path.read_text())
    for name in ["disable.json", "startup.json", "state.json"]:
        path = runtime / name
        if path.is_file():
            diagnostics[name] = json.loads(path.read_text())
    run.finish(code, details={"events": events, "diagnostics": diagnostics,
        "candidate_sha256": sha(candidate.read_bytes()),
        "input_sha256": {p.name: sha(p.read_bytes()) for p in work.glob("*.json")},
        "scope": "GitHub-hosted Windows; pinned public Codex CLI with fresh private home/cache and no credentials, local plugin commands only. Real idle monitor/task and private direct Hive image. No model, desktop session control, subscription observation or public14 artifact acceptance." if real_codex else "GitHub-hosted Windows; actual owned task/idle monitor and private direct image, stop failure/refusal/same-operation removal/resume/reinstall. Fixed Codex plugin CLI only, no actual app/subscription/model/public14 artifact acceptance." if uninstall_mode else "GitHub-hosted Windows, private actual direct image/receipt, actual fixed-registry current-version reconciliation and fixed Codex plugin CLI. No actual newer-core replacement, monitor or desktop app." if direct_mode else "Private file preparation only, fake Codex CLI; no native probe, monitor registration or activation." if prepare_only else "GitHub-hosted Windows; actual Task Scheduler and candidate monitor in idle state; fixed Codex plugin CLI only. No desktop app, subscription observation, active-chat interruption, provider calls or public14 package acceptance."})
raise SystemExit(code)
