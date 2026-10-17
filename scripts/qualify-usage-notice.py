"""Qualify the actual notice entrypoint. No sensor, monitor, or session interruption."""
from __future__ import annotations

import argparse
import base64
import ctypes
from ctypes import wintypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import queue
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--app-pid", type=int, required=True)
    parser.add_argument("--app-created-ticks", type=int, required=True)
    args = parser.parse_args()
    fixture = args.fixture.resolve(strict=True)
    if os.name != "nt" or not fixture.is_relative_to(ROOT / "tests/work/usage-notice-preview"):
        parser.error("Owned Windows display fixture required")
    state_path = fixture / ".hive/runtime/usage-monitor/state.json"
    state = json.loads(state_path.read_text(encoding="utf-8-sig"))
    incident = state["incident"]
    before = hashlib.sha256(state_path.read_bytes()).hexdigest()
    spec = importlib.util.spec_from_file_location("scoped_adapter", ROOT / "scripts/qualify-session-adapter.py")
    adapter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(adapter)
    app = adapter.AppPin(args.app_pid, args.app_created_ticks)
    child = None
    record = {"schema_version": 1, "preview_only": True, "session_interruption": False,
              "monitor_started": False, "host": "Windows Codex"}
    try:
        if not app.alive_and_visible():
            raise RuntimeError("Original Codex app is not visible")
        source = (ROOT / "crates/hive-cli/src/usage_monitor/notice.ps1").read_bytes().replace(b"\r\n", b"\n")
        record["notice_sha256"] = hashlib.sha256(source).hexdigest()
        payload = base64.b64encode(source).decode()
        bootstrap = "& ([scriptblock]::Create([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('" + payload + "'))))"
        if len(bootstrap) > 32000:
            raise RuntimeError("Notice launch exceeds the production budget")
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        started = time.monotonic()
        child = subprocess.Popen([str(executable), "-NoLogo", "-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", bootstrap],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        request = {"operation": "notify", "preview": True, "user_root": str(fixture),
                   "logo_png_base64": base64.b64encode((ROOT / "harness/plugins/aigent-hive/assets/hive-logo-plugin.png").read_bytes()).decode(),
                   "incident": incident["digest"], "reason": incident["reason"],
                   "message": "Aigent Hive 표시 전용 미리보기. 실제 작업 중단 없음."}
        child.stdin.write(json.dumps(request).encode())
        child.stdin.close()
        packets = queue.Queue(maxsize=1)
        threading.Thread(target=lambda: packets.put(child.stdout.readline(1024)), daemon=True).start()
        packet = json.loads(packets.get(timeout=5))
        record["shown_ms"] = round((time.monotonic() - started) * 1000)
        if not packet.get("notified") or packet.get("fallback") or record["shown_ms"] > 5000:
            raise RuntimeError("Designed notice did not open within its production deadline")
        user = ctypes.WinDLL("user32", use_last_error=True)
        callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
        user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
        windows = []

        def inspect(window, _):
            owner = wintypes.DWORD()
            user.GetWindowThreadProcessId(window, ctypes.byref(owner))
            if owner.value == child.pid:
                windows.append(int(window))
            return True

        user.EnumWindows(callback_type(inspect), 0)
        # Only the still-held helper process can own the selected window.
        if child.poll() is not None or not windows:
            raise RuntimeError("Owned notice window disappeared")
        close = r"""
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$ids=@(HANDLES)
foreach($handle in $ids){
 $window=[Windows.Automation.AutomationElement]::FromHandle([IntPtr]$handle)
 if($window.Current.ProcessId -ne OWNER -or $window.Current.Name -ne 'Aigent Hive — 사용량 보호 상태'){continue}
 $condition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'안내 닫기, 작업 재개 없음')
 $button=$window.FindFirst([Windows.Automation.TreeScope]::Descendants,$condition)
 if($null -ne $button){$button.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke();exit 0}
}
throw 'Owned close button absent'
""".replace("HANDLES", ",".join(map(str, windows))).replace("OWNER", str(child.pid))
        closer = fixture / "close-owned-notice.ps1"
        closer.write_text(close, encoding="utf-8-sig")
        result = subprocess.run([str(executable), "-NoProfile", "-NonInteractive", "-File", str(closer)],
                                capture_output=True, timeout=8)
        if result.returncode:
            raise RuntimeError("Owned notice button could not be invoked")
        child.wait(timeout=5)
        record["closed_by_own_button"] = True
        record["incident_unchanged"] = hashlib.sha256(state_path.read_bytes()).hexdigest() == before
        record["original_codex_app_preserved"] = app.alive_and_visible()
        if not record["incident_unchanged"] or not record["original_codex_app_preserved"]:
            raise RuntimeError("Notice close changed protection or the original app")
        record["phase"] = "passed-display-only-entrypoint"
        return 0
    except Exception as error:
        record["phase"] = "failed"
        record["error"] = str(error)
        return 1
    finally:
        if child is not None and child.poll() is None:
            # This explicitly spawned display helper only; never a Codex process.
            child.terminate()
            child.wait(timeout=5)
        app.close()
        (fixture / "entrypoint.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8", newline="\n")
        print(json.dumps(record, separators=(",", ":")))


if __name__ == "__main__":
    raise SystemExit(main())
