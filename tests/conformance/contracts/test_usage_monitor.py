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

    def deployment_functions(self) -> str:
        path = ROOT / "scripts/complete-usage-monitor-install.ps1"
        return "$ErrorActionPreference='Stop'\n$tokens=$null;$errors=$null\n" + (
            "$tree=[Management.Automation.Language.Parser]::ParseFile('" + str(path).replace("'", "''")
            + "',[ref]$tokens,[ref]$errors)\n"
            + "if($errors){throw 'Deployment script parsing failed'}\n"
            + "$tree.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]},$false) | ForEach-Object {Invoke-Expression $_.Extent.Text}\n"
        )

    def test_deployment_cli_forwards_the_complete_argument_array(self):
        source = self.deployment_functions() + r"""
function FakeHive { $global:forwarded=@($args);$global:LASTEXITCODE=0;'{"status":"success","code":"fixture"}' }
$native='FakeHive'
$value=Run-Hive -arguments @('usage','monitor','status','--output','json')
if($forwarded.Count -ne 5 -or $forwarded[4] -ne 'json'){throw 'Arguments were lost'}
@{forwarded=$true;code=$value.code} | ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["forwarded"])

    def test_deployment_preservation_hashes_cover_nested_canonical_knowledge(self):
        source = self.deployment_functions() + r"""
$userRoot=Join-Path $PSScriptRoot 'user'
try {
 $null=New-Item -ItemType Directory -Path (Join-Path $userRoot '.hive/config'),(Join-Path $userRoot '.hive/knowledge/nested') -Force
 [IO.File]::WriteAllText((Join-Path $userRoot '.hive/config/user-setup.yml'),'kept')
 [IO.File]::WriteAllText((Join-Path $userRoot '.hive/knowledge/nested/claim.md'),'canonical')
 $before=Preserved-Digests
 $after=Preserved-Digests
 if($before.Count -ne 2 -or $before['.hive\knowledge\nested\claim.md'] -ne $after['.hive\knowledge\nested\claim.md']){throw 'Missing protected bytes'}
 @{preserved=$true} | ConvertTo-Json -Compress
} finally { } # The Python TemporaryDirectory owns fixture cleanup.
"""
        self.assertTrue(self.powershell(source)["preserved"])

    def test_deployment_cannot_run_after_a_failed_app_stop(self):
        helper = str(ROOT / "scripts/complete-usage-monitor-install.ps1").replace("'", "''")
        source = r"""
$ErrorActionPreference='Stop'
$request=Join-Path $PSScriptRoot 'request.json'
$result=Join-Path $PSScriptRoot 'result.json'
[IO.File]::WriteAllText($request,'{"product_version":"0.12.1","install_after_pass":true,"always_on":true}')
[IO.File]::WriteAllText($result,'{"phase":"failed"}')
try {
 & 'HELPER' -Request $request -AcceptanceResult $result
 throw 'Conditional installation gate was bypassed'
} catch {
 if($_.Exception.Message -notmatch 'App-stop acceptance'){throw}
 @{rejected=$true} | ConvertTo-Json -Compress
}
""".replace("'HELPER'", "'" + helper + "'")
        self.assertTrue(self.powershell(source)["rejected"])

    def test_legacy_powershell_passes_spaced_paths_without_nested_shell_code(self):
        wrapper=str(ROOT / 'scripts/invoke-dev-install.sh').replace("'","''")
        source=r"""
$ErrorActionPreference='Stop'
$directory=Join-Path $PSScriptRoot 'folder with spaces'
$null=New-Item -ItemType Directory -Path $directory
$stub=Join-Path $directory 'installer.sh'
$body='[ "$1" = "--sandbox" ] || exit 8' + "`n" + '[ "$2" = "--state-dir" ] || exit 9' + "`n" + 'printf ''{"passed":true}\n'''
[IO.File]::WriteAllText($stub,$body,[Text.UTF8Encoding]::new($false))
& 'C:\Program Files\Git\bin\bash.exe' --noprofile --norc 'WRAPPER' sandbox $directory $stub (Join-Path $directory 'state') $directory
if($LASTEXITCODE -ne 0){throw 'Legacy PowerShell path forwarding failed'}
""".replace("'WRAPPER'","'"+wrapper+"'")
        self.assertTrue(self.powershell(source)["passed"])

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

    def test_child_exit_racing_kill_is_verified_by_its_final_handle_state(self):
        fake = r"""
class AlreadyExitedProcess {
 [int]$Id=123
 [bool]$HasExited=$false
 [void]Kill() { $this.HasExited=$true;throw 'Exited between HasExited and Kill' }
 [bool]WaitForExit([int]$timeout) { return $this.HasExited }
 [void]Dispose() { }
}
function Pin-Processes($tree) { return ,@([AlreadyExitedProcess]::new()) }
"""
        script=self.adapter_functions().replace("try {\n    switch ($request.operation) {",fake+"\ntry {\n    switch ($request.operation) {")
        result=self.powershell("$request=[pscustomobject]@{operation='halt';roots=@()}\n"+script)
        self.assertTrue(result["verified"])
        self.assertEqual(result["survivor_count"],0)
        self.assertGreater(result["failure_count"],0)
        # A failed cancellation with a genuinely live handle must still fail closed.
        live_script=script.replace("$this.HasExited=$true;throw", "throw")
        live=self.powershell("$request=[pscustomobject]@{operation='halt';roots=@()}\n"+live_script)
        self.assertFalse(live["verified"])
        self.assertEqual(live["survivor_count"],1)

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
