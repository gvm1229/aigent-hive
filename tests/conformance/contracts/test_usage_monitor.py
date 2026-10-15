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

    def desktop_functions(self) -> str:
        path=ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1"
        return "$ErrorActionPreference='Stop'\n$tokens=$null;$errors=$null\n" + (
            "$tree=[Management.Automation.Language.Parser]::ParseFile('" + str(path).replace("'", "''")
            + "',[ref]$tokens,[ref]$errors)\nif($errors){throw 'Desktop script parsing failed'}\n"
            + "$tree.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]},$false) | ForEach-Object {Invoke-Expression $_.Extent.Text}\n"
        )

    def test_former_app_termination_operation_is_rejected(self):
        script=self.adapter_functions().replace("exit 1", "throw")
        source="$request=[pscustomobject]@{operation='halt';roots=@()}\ntry {\n"+script+r"""
 throw 'Former termination operation remained available'
} catch {
 if($_.Exception.Message -notmatch 'Unsupported monitor operation'){throw}
 @{rejected=$true} | ConvertTo-Json -Compress
}
"""
        self.assertTrue(self.powershell(source)["rejected"])

    def test_desktop_snapshot_extracts_current_turn_without_retaining_history(self):
        source=self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew()
$sessions=@{fixture=@{id='fixture';ready=$false;dirty=$true;active=@();statuses=@{}}}
$snapshot=[pscustomobject]@{hostId='local';conversationId='fixture';change=[pscustomobject]@{type='snapshot';conversationState=[pscustomobject]@{
 id='fixture';hostId='local';turns=@();title='fixture';threadRuntimeStatus=@{type='active'};threadGoal=@{status='active'}
 turnHistory=@{history=@{entitiesByKey=[pscustomobject]@{one=@{turnId='old';status='completed';items=@('not retained')};two=@{turnId='current';status='inProgress';items=@('not retained')}}}}
}}}
Snapshot $snapshot
if(-not $sessions.fixture.ready -or $sessions.fixture.active.Count -ne 1 -or $sessions.fixture.active[0] -ne 'current'){throw 'Active turn omitted'}
if($sessions.fixture.ContainsKey('items') -or $sessions.fixture.ContainsKey('turnHistory')){throw 'History retained'}
$snapshot.change.conversationState.turnHistory.history.entitiesByKey.two.status='interrupted'
Snapshot $snapshot
if($sessions.fixture.active.Count -ne 0 -or $sessions.fixture.statuses.current -ne 'interrupted'){throw 'Interruption was not observed'}
@{active_extracted=$true;interruption_observed=$true;history_discarded=$true} | ConvertTo-Json -Compress
"""
        result=self.powershell(source)
        self.assertTrue(result["interruption_observed"])
        self.assertTrue(result["history_discarded"])

    def test_wrong_desktop_snapshot_cannot_confirm_a_different_session(self):
        source=self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew()
$sessions=@{fixture=@{id='fixture';ready=$false}}
$snapshot=[pscustomobject]@{hostId='local';conversationId='fixture';change=[pscustomobject]@{type='snapshot';conversationState=@{id='different';hostId='local'}}}
try {Snapshot $snapshot;throw 'Wrong session accepted'} catch {if($_.Exception.Message -notmatch 'Mismatched desktop snapshot'){throw}}
if($sessions.fixture.ready){throw 'Wrong session changed readiness'}
@{rejected=$true} | ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["rejected"])

    def test_desktop_pipe_handles_split_frames_and_rejects_oversized_frames(self):
        source=(ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1").read_text(encoding='utf-8')
        declaration=source.split("Add-Type -TypeDefinition @'",1)[1].split("'@",1)[0]
        declaration=declaration.replace('"codex-ipc"','"hive-frame-test-" + System.Diagnostics.Process.GetCurrentProcess().Id')
        fixture="$ErrorActionPreference='Stop'\nAdd-Type -TypeDefinition @'\n"+declaration+"\n'@\n"+r"""
$name='hive-frame-test-'+$PID
$server=[IO.Pipes.NamedPipeServerStream]::new($name,[IO.Pipes.PipeDirection]::InOut,1,[IO.Pipes.PipeTransmissionMode]::Byte,[IO.Pipes.PipeOptions]::Asynchronous)
$waiting=$server.WaitForConnectionAsync()
$client=[HiveDesktopPipe]::new();$null=$waiting.GetAwaiter().GetResult();$client.Start()
try {
 $bytes=[Text.Encoding]::UTF8.GetBytes('{"fixture":true}')
 $header=[BitConverter]::GetBytes([int]$bytes.Length)
 $server.Write($header,0,2);$server.Flush();Start-Sleep -Milliseconds 20
 if($null -ne $client.Poll()){throw 'Partial header delivered'}
 $server.Write($header,2,2);$server.Write($bytes,0,3);$server.Flush();Start-Sleep -Milliseconds 20
 if($null -ne $client.Poll()){throw 'Partial body delivered'}
 $server.Write($bytes,3,$bytes.Length-3);$server.Flush()
 $deadline=[DateTime]::UtcNow.AddSeconds(2);$frame=$null
 do {Start-Sleep -Milliseconds 10;$frame=$client.Poll()} while($null -eq $frame -and [DateTime]::UtcNow -lt $deadline)
 if(-not ($frame|ConvertFrom-Json).fixture){throw 'Complete frame missing'}
 $bad=[BitConverter]::GetBytes([int]16777217);$server.Write($bad,0,4);$server.Flush()
 $deadline=[DateTime]::UtcNow.AddSeconds(2)
 while($client.Alive -and [DateTime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 10}
 if($client.Alive){throw 'Oversized frame accepted'}
 @{split_frame_verified=$true;oversized_rejected=$true} | ConvertTo-Json -Compress
} finally {$client.Dispose();$server.Dispose()}
"""
        self.assertTrue(self.powershell(fixture)["oversized_rejected"])

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
