"""Behavior tests for the Windows adapter; never targets the running Codex app."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "crates/hive-cli/src/usage_monitor/windows.ps1"


@unittest.skipUnless(os.name == "nt", "Windows process handles and Task Scheduler adapter")
class WindowsUsageMonitor(unittest.TestCase):
    def powershell(self, source: str) -> dict:
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        with tempfile.TemporaryDirectory(prefix="hive-monitor-test-") as directory:
            fixture_script = Path(directory) / "fixture.ps1"
            fixture_script.write_text(source, encoding="utf-8-sig")
            result = subprocess.run([str(executable), "-NoProfile", "-NonInteractive", "-File", str(fixture_script)],
                                    capture_output=True, timeout=25)
        self.assertEqual(result.returncode, 0, result.stderr.decode("utf-8", errors="replace")[-2000:])
        return json.loads(result.stdout.decode("utf-8-sig"))

    def adapter_functions(self) -> str:
        source = SCRIPT.read_text(encoding="utf-8")
        # Exercise production functions and switch without accepting arbitrary product targets.
        return source.replace("$request = [Console]::In.ReadToEnd() | ConvertFrom-Json", "# request supplied by the fixture")

    def test_emergency_stop_terminates_own_fixture_tree_and_preserves_unrelated_process(self):
        script = self.adapter_functions()
        source = r"""$ErrorActionPreference='Stop'
$fixture = Start-Process $env:ComSpec -ArgumentList '/c ping 127.0.0.1 -n 60 >nul' -WindowStyle Hidden -PassThru
$unrelated = Start-Process $env:ComSpec -ArgumentList '/c ping 127.0.0.1 -n 60 >nul' -WindowStyle Hidden -PassThru
try {
  Start-Sleep -Milliseconds 500
  $request = [pscustomobject]@{operation='halt'; roots=@(@{pid=$fixture.Id;created_ticks=$fixture.StartTime.ToUniversalTime().Ticks;executable=$fixture.MainModule.FileName})}
""" + script + r"""
  if (-not $fixture.WaitForExit(1000)) { throw 'Fixture parent still alive' }
  if ($unrelated.HasExited) { throw 'Unrelated process was terminated' }
} finally {
  foreach($p in @($fixture,$unrelated)) {
    try { $children=Get-CimInstance Win32_Process -Filter "ParentProcessId=$($p.Id)"; foreach($c in $children){Stop-Process -Id $c.ProcessId -Force -ErrorAction SilentlyContinue}; if(-not $p.HasExited){$p.Kill()} } catch{}
  }
}
"""
        result = self.powershell(source)
        self.assertTrue(result["verified"])
        self.assertGreaterEqual(result["targeted"], 2)
        self.assertEqual(result["survivor_count"], 0)
        self.assertLess(result["elapsed_ms"], 5000)

    def test_reused_pid_identity_is_rejected_before_killing(self):
        script = self.adapter_functions().replace("exit 1", "throw")
        source = r"""
$ErrorActionPreference='Stop'
$fixture=Start-Process $env:ComSpec -ArgumentList '/c ping 127.0.0.1 -n 60 >nul' -WindowStyle Hidden -PassThru
try {
  $request=[pscustomobject]@{operation='halt'; roots=@(@{pid=$fixture.Id;created_ticks=1;executable=$fixture.MainModule.FileName})}
  $rejected=$false
  try {
""" + script + r"""
  } catch { $rejected=$true }
  if (-not $rejected -or $fixture.HasExited) { throw 'Wrong identity was not safely rejected' }
  @{rejected=$true;preserved=$true} | ConvertTo-Json -Compress
} finally {
  $children=Get-CimInstance Win32_Process -Filter "ParentProcessId=$($fixture.Id)"; foreach($c in $children){Stop-Process -Id $c.ProcessId -Force -ErrorAction SilentlyContinue}
  if(-not $fixture.HasExited){$fixture.Kill()}
}
"""
        self.assertTrue(self.powershell(source)["preserved"])

    def test_discovered_current_app_identities_round_trip_to_exact_handle_time(self):
        source = "$request=[pscustomobject]@{operation='inspect'}\n$raw = & {\n" + self.adapter_functions() + r"""
}
$data=$raw | ConvertFrom-Json
foreach($root in $data.roots) {
  try {
    $p=[Diagnostics.Process]::GetProcessById([int]$root.pid)
    if($p.StartTime.ToUniversalTime().Ticks -ne [long]$root.created_ticks){throw 'Precision lost in cached identity'}
  } catch [ArgumentException] { }
}
@{valid=$true;scope='read-only-app-discovery'} | ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["valid"])

    def test_register_collision_and_foreign_task_removal_are_refused(self):
        # Real task registration is not needed to prove that foreign tasks stay untouched.
        script = self.adapter_functions().replace("exit 1", "throw")
        for operation in ("register", "unregister"):
            source = """
function Get-ScheduledTask { [pscustomobject]@{Description='foreign';Actions=@([pscustomobject]@{Execute='foreign';Arguments='foreign'})} }
function Register-ScheduledTask { throw 'Unexpected registration' }
function Unregister-ScheduledTask { throw 'Unexpected removal' }
$request=[pscustomobject]@{operation='OP';task_name='AigentHive-UsageMonitor-0123456789abcdef';executable='owned';arguments='owned'}
$rejected=$false
try {
""".replace("'OP'", repr(operation)) + script + """
} catch { $rejected=$true }
if(-not $rejected){throw 'Foreign task accepted'}
@{rejected=$rejected} | ConvertTo-Json -Compress
"""
            self.assertTrue(self.powershell(source)["rejected"])


if __name__ == "__main__":
    unittest.main()
