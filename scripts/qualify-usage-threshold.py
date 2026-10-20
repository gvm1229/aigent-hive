"""Independent, exact-subject Windows floor acceptance. Never launches model work."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
OWNED = ROOT / "tests/work/usage-monitor-acceptance"


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def digest(value):
    return "sha256:" + hashlib.sha256(value.encode()).hexdigest()


def read(path):
    for attempt in range(5):
        try:
            return json.loads(path.read_text(encoding="utf-8-sig"))
        except (OSError, ValueError):
            if attempt == 4:
                raise
            time.sleep(0.02)


def notice(monitor_pid, close=False):
    # Read/invoke only the close action of the test monitor's own notice child.
    script = r"""
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$rows=@(Get-CimInstance Win32_Process -Filter 'ParentProcessId=OWNER')
$condition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'Aigent Hive — 사용량 보호 상태')
$windows=[Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children,$condition)
foreach($window in $windows){
 if($window.Current.ProcessId -notin @($rows | ForEach-Object {[int]$_.ProcessId}) -or $window.Current.IsOffscreen){continue}
 $controls=$window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
 $text=@($controls | ForEach-Object {$_.Current.Name}) -join "`n"
 $buttons=@($controls | Where-Object {$_.Current.ControlType -eq [Windows.Automation.ControlType]::Button -and -not $_.Current.IsOffscreen})
 $close=@($buttons | Where-Object {$_.Current.Name -eq '안내 닫기, 작업 재개 없음'})
 $result=@{visible=$true;floor_reason=$text.Contains('설정한 사용량 하한');numeric_reason=$text.Contains('하한 99%');confirmed=$text.Contains('작업 중단 확인');close_action=($close.Count -eq 1);no_unlock=(-not $text.Contains('보호 해제'))}
 if(CLOSE_ACTION){if($close.Count -ne 1){throw 'Exact owned close action missing'};$close[0].GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke();$result.closed=$true}
 $result | ConvertTo-Json -Compress;exit 0
}
'{}'
""".replace("OWNER", str(int(monitor_pid))).replace("CLOSE_ACTION", "$true" if close else "$false")
    completed = subprocess.run(
        ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
        capture_output=True, timeout=8, creationflags=subprocess.CREATE_NO_WINDOW,
    )
    if completed.returncode:
        raise RuntimeError("owned notice inspection failed")
    return json.loads(completed.stdout.decode("utf-8-sig"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--subject")
    parser.add_argument("--role", choices=("reasoning", "shell"))
    parser.add_argument("--app-pid", type=int)
    parser.add_argument("--app-created-ticks", type=int)
    args = parser.parse_args()
    fixture = args.fixture.resolve()
    if os.name != "nt" or not fixture.is_relative_to(OWNED) or fixture == OWNED:
        parser.error("owned Windows acceptance fixture required")
    if args.prepare:
        if (fixture / ".hive").exists():
            parser.error("a fresh owned fixture is required; previous evidence is preserved")
        sys.path.insert(0, str(ROOT))
        from tests.conformance.contracts.test_usage_control import USER_CONFIG
        (fixture / ".hive/config").mkdir(parents=True)
        (fixture / ".hive/install").mkdir()
        (fixture / ".hive/config/user-setup.yml").write_text(USER_CONFIG.replace("stop_remaining_percent: 20", "stop_remaining_percent: 99"), encoding="utf-8")
        (fixture / ".hive/install/codex.json").write_text(json.dumps({"product_version":"0.13.1","acceptance_fixture":True}), encoding="utf-8")
        shell = r"""param([Parameter(Mandatory=$true)][string]$Fixture)
$ErrorActionPreference='Stop'
$process=[Diagnostics.Process]::GetCurrentProcess()
$identity=@{pid=$PID;created_ticks=$process.StartTime.ToUniversalTime().Ticks}
[IO.File]::WriteAllText((Join-Path $Fixture 'shell-child.json'),($identity|ConvertTo-Json),[Text.UTF8Encoding]::new($false))
for($i=0;$i -lt 180;$i++){
 [IO.File]::WriteAllText((Join-Path $Fixture 'shell-heartbeat.txt'),[DateTime]::UtcNow.ToString('o'))
 Start-Sleep -Seconds 1
}
"""
        (fixture / "long-shell.ps1").write_text(shell, encoding="utf-8-sig")
        print(json.dumps({"prepared":True,"scope":"isolated source-only single conversation","monitor_started":False,"floor":99}))
        return 0
    if not args.role or not args.subject or not args.app_pid or not args.app_created_ticks:
        parser.error("role, approved subject and exact app identity are required")
    if not re.fullmatch(r"[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}", args.subject):
        parser.error("exact approved test conversation required")
    hive = ROOT / "target/debug/hive.exe"
    events = module("threshold_events", "qualify-session-events.py")
    pins = module("threshold_pins", "qualify-session-adapter.py")
    app = pins.AppPin(args.app_pid, args.app_created_ticks)
    child = None
    monitor = None
    output = fixture / "result.json"
    state_path = fixture / ".hive/runtime/usage-monitor/state.json"
    record = {"schema_version": 1, "phase": "waiting-subject", "role": args.role,
              "subject_digest": digest(args.subject), "synthetic_floor": 99,
              "real_provider_reset": False, "real_user_installation_changed": False,
              "binary_sha256": hashlib.sha256(hive.read_bytes()).hexdigest()}

    def save():
        output.write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding="utf-8")

    def native():
        return events.subject_state(args.subject)

    def disable():
        return subprocess.run([str(hive), "usage", "monitor", "disable", "--user-root", str(fixture), "--output", "json"],
                              capture_output=True, timeout=20, creationflags=subprocess.CREATE_NO_WINDOW)

    try:
        save()
        deadline = time.monotonic() + 120
        stable_since = None
        while time.monotonic() < deadline:
            before = native()
            eligible = before["status"] == "running"
            eligible &= before["pending_tools"] == 0 if args.role == "reasoning" else (fixture / "shell-child.json").is_file()
            if eligible:
                stable_since = stable_since or time.monotonic()
                if time.monotonic() - stable_since >= 6:
                    break
            else:
                stable_since = None
            time.sleep(0.1)
        else:
            raise RuntimeError("subject did not enter required active execution")
        if not app.alive_and_visible():
            raise RuntimeError("original Codex app missing")
        if args.role == "shell":
            identity = read(fixture / "shell-child.json")
            child = pins.AppPin(identity["pid"], identity["created_ticks"])
            if child.kernel.WaitForSingleObject(child.handle, 0) != 258:
                raise RuntimeError("long shell ended before test")
        record["before_turn_digest"] = digest(before["turn_id"])
        record["before_tool_calls"] = before["tool_calls"]
        record["tool_free_at_detection"] = before["pending_tools"] == 0
        with (fixture / "controller-output.json").open("wb") as log:
            monitor = subprocess.Popen([str(hive), "__usage-monitor-qualification", str(fixture), args.subject],
                                       stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.DEVNULL,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
        record["phase"] = "waiting-stop"
        save()
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            if monitor.poll() is not None:
                raise RuntimeError("scoped controller exited before stop")
            if state_path.is_file():
                state = read(state_path)
                incident = state.get("incident") or {}
                if incident.get("stop_confirmed"):
                    break
            time.sleep(0.05)
        else:
            raise RuntimeError("floor stop not confirmed")
        current = native()
        if current["turn_id"] != before["turn_id"] or current["status"] != "interrupted":
            raise RuntimeError("exact native turn was not interrupted")
        if args.role == "reasoning" and current["tool_calls"] != before["tool_calls"]:
            raise RuntimeError("reasoning subject used a tool during interruption")
        receipt = incident.get("confirmed_stop") or {}
        if incident["reason"] != "usage-threshold" or receipt.get("request_ms", 99999) > 2000 or receipt.get("elapsed_ms", 99999) > 5000:
            raise RuntimeError("wrong reason or missed interruption deadline")
        if state.get("known_sessions") != 1 or incident.get("targeted_turns") != 1 or incident.get("confirmed_turns") != 1:
            raise RuntimeError("stop was not exactly scoped to the test conversation")
        if child:
            deadline_ms = incident["detected_at_ms"] + 5000
            while child.kernel.WaitForSingleObject(child.handle, 0) != 0 and time.time_ns() // 1_000_000 <= deadline_ms:
                time.sleep(0.01)
            if child.kernel.WaitForSingleObject(child.handle, 0) != 0:
                raise RuntimeError("shell child remained five seconds after detection")
            record["shell_exit_observed_ms"] = time.time_ns() // 1_000_000 - incident["detected_at_ms"]
        view = notice(monitor.pid)
        if not all(view.get(key) for key in ("visible", "floor_reason", "numeric_reason", "confirmed", "close_action", "no_unlock")):
            raise RuntimeError("exact reason/status notice not visible")
        record.update(phase="awaiting-manual-resume", stop_receipt=receipt, notice=view,
                      shell_child_exited=bool(child), app_preserved=app.alive_and_visible(),
                      native_interruption_verified=True, generation=state["generation"])
        save()
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            current = native()
            if current["turn_id"] != before["turn_id"] and current["status"] == "completed":
                break
            time.sleep(0.1)
        else:
            raise RuntimeError("direct resume/new message was not completed")
        resumed = read(fixture / "resume-enforce.json")
        if resumed.get("code") != "hive.usage-allowed" or not (resumed.get("data") or {}).get("threshold_resume_current_conversation"):
            raise RuntimeError("manual preflight floor exemption was not verified")
        time.sleep(32)  # At least two more normal readings at the same synthetic floor.
        later = read(state_path)
        if later["generation"] != state["generation"] or native()["status"] != "completed":
            raise RuntimeError("same floor event reinterrupted resumed work")
        if disable().returncode:
            raise RuntimeError("test monitor disable failed")
        monitor.wait(timeout=6)
        stable = state_path.read_bytes()
        record["notice_closed"] = notice(monitor.pid, close=True).get("closed", False)
        record["notice_close_preserved_history"] = stable == state_path.read_bytes()
        record.update(phase="passed", direct_resume_verified=True, repeated_stop=False,
                      app_preserved=app.alive_and_visible(), monitor_removed=True)
        return 0
    except Exception as error:
        record.update(phase="failed", error=type(error).__name__ + ": " + str(error))
        return 1
    finally:
        if monitor is not None and monitor.poll() is None:
            try:
                disable()
                monitor.wait(timeout=6)
            except (OSError, subprocess.TimeoutExpired):
                monitor.terminate()  # Only this exact, owned non-model test controller.
                monitor.wait(timeout=5)
        if child:
            child.close()  # Read-only handle; never signals the shell child.
        app.close()
        save()
        print(json.dumps({key: value for key, value in record.items() if key not in ("stop_receipt", "notice")}, ensure_ascii=False))


if __name__ == "__main__":
    raise SystemExit(main())
