"""Qualify real idle monitoring only in a disposable Windows hosted runner."""
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

if not (os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires isolated GitHub-hosted Windows; personal installations excluded")

(ROOT / "tests/work").mkdir(parents=True, exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="windows-monitor-ci-", dir=ROOT / "tests/work")).resolve()
user = work / "user"
user.mkdir()
fake_bin = work / "fake-bin"
fake_bin.mkdir()
candidate = ROOT / "target/debug/hive.exe"
binary = work / "hive.exe"
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
(fake_bin / "codex.py").write_text(program, encoding="utf-8")
(fake_bin / "codex.cmd").write_text(
    f'@"{sys.executable}" "%~dp0codex.py" %*\r\n', encoding="utf-8")

sha = lambda data: hashlib.sha256(data).hexdigest()
events = []
run = Run("격리 Windows 실제 감시의 별도 동의·대기 실행·해제·재활성",
    [sys.executable, str(Path(__file__).resolve())],
    paths=[work.relative_to(ROOT).as_posix()], binary=candidate)
runtime = user / ".hive/runtime/usage-monitor"
config_path = user / ".hive/config/user-setup.yml"
monitor_path = runtime / "config.json"
activation_path = user / ".hive/config/setup-progress/monitor-activation.json"
code = 1


def invoke(*args, expected=0):
    result = subprocess.run([str(binary), *args, "--output", "json"], env=environment,
        stdin=subprocess.DEVNULL, capture_output=True, timeout=150)
    response = json.loads(result.stdout) if result.stdout.strip() else {}
    events.append({"arguments": list(args), "exit_code": result.returncode,
                   "code": response.get("code"), "message": response.get("message"),
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


try:
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
    defaults = invoke(*base, "--describe")["data"]["expedited_defaults"]
    defaults.update(interface_language="ko", selected_hosts=["codex"])
    defaults["wiki"]["enabled"] = False
    off_file = write_answers("off.json", defaults)
    invoke("install", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
    invoke(*base, "--answers", off_file, "--apply")
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
    if monitor_path.exists() and json.loads(monitor_path.read_text()).get("enabled"):
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
        "scope": "GitHub-hosted Windows; actual Task Scheduler and candidate monitor in idle state; fixed Codex plugin CLI only. No desktop app, subscription observation, active-chat interruption, provider calls or public14 package acceptance."})
raise SystemExit(code)
