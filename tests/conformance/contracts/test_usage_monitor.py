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

    def desktop_json(self) -> str:
        source=(ROOT/'crates/hive-cli/src/usage_monitor/desktop.ps1').read_text(encoding='utf-8')
        begin=source.index('Add-Type -ReferencedAssemblies')
        end=source.index("'@",begin)+2
        return "$ErrorActionPreference='Stop'\n"+source[begin:end]+"\n"

    def test_large_stream_keeps_only_turn_metadata_before_powershell_json(self):
        source=self.desktop_json()+self.desktop_functions()+r"""
$state=@{id='fixture';hostId='local';threadRuntimeStatus=@{type='busy'};threadGoal=@{status='active'};
 turns=@(@{turnId='latest';status='inProgress';itemsPagination=@{oldest='page'};items=@(@{text=('x'*20000000)})});
 turnHistory=@{kind='canonical';history=@{islands=@(@{entries=@(@{value='old'},@{value='latest'})});entitiesByKey=@{
 old=@{turnId='old';status='completed'};latest=@{turnId='latest';status='interrupted';itemsPagination=@{oldest='page'}}}}}}
$inputJson=@{type='broadcast';method='thread-stream-state-changed';params=@{hostId='local';conversationId='fixture';change=@{type='snapshot';conversationState=$state}}}|ConvertTo-Json -Depth 20 -Compress
if([Text.Encoding]::UTF8.GetByteCount($inputJson) -le 16777216){throw 'Large input not exercised'}
$filtered=[HiveDesktopJson]::Compact($inputJson)
$view=($filtered|ConvertFrom-Json).params.change.conversationState
$turns=@(Visible-Turns $view)
if($turns.Count -ne 2 -or $turns[-1].status -ne 'interrupted' -or $view.threadGoal.status -ne 'active'){throw 'Turn metadata changed'}
if($filtered.Contains('20000000') -or $filtered.Contains('"items":') -or $filtered.Length -gt 4096){throw 'History content retained'}
@{large_input=$true;ordered_status=$true;content_removed=$true}|ConvertTo-Json -Compress
"""
        self.assertEqual(self.powershell(source),{'large_input':True,'ordered_status':True,'content_removed':True})

    def test_stream_delta_keeps_paths_and_discards_values(self):
        source=self.desktop_json()+r"""
$frame='{"type":"broadcast","method":"thread-stream-state-changed","params":{"hostId":"local","conversationId":"fixture","change":{"type":"delta","patches":[{"path":["turns",0,"status"],"value":"private-content"}]}}}'
$filtered=[HiveDesktopJson]::Compact($frame)
$parsed=$filtered|ConvertFrom-Json
if($filtered.Contains('private-content') -or $parsed.params.change.patches[0].path[2] -ne 'status'){throw 'Delta pruning changed status paths'}
@{paths_preserved=$true;values_removed=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)['paths_preserved'])

    def test_compact_response_preserves_interrupt_confirmation_and_rejects_bad_input(self):
        source=self.desktop_json()+r"""
$frame='{"type":"response","requestId":"r","resultType":"success","handledByClientId":"owner","result":{"ok":true,"interruptedTurnId":"turn","goalPauseError":false,"unused":{"id":"lower","ID":"upper","text":"private-content"}}}'
$filtered=[HiveDesktopJson]::Compact($frame);$parsed=$filtered|ConvertFrom-Json
if($parsed.result.interruptedTurnId -ne 'turn' -or -not $parsed.result.ok -or $parsed.result.goalPauseError -or $filtered.Contains('private-content')){throw 'Interrupt response changed'}
$rejected=$false;try{[HiveDesktopJson]::Compact('{invalid')}catch{$rejected=$true}
if(-not $rejected){throw 'Invalid JSON accepted'}
$rejected=$false;try{[HiveDesktopJson]::Compact(('{"type":"response","error":"'+('x'*262145)+'"}'))}catch{$rejected=$true}
if(-not $rejected){throw 'Oversized retained metadata accepted'}
@{confirmation_preserved=$true;bounds_enforced=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)['bounds_enforced'])

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
 if($_.Exception.Message -notmatch 'Legacy global developer deployment is retired'){throw}
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

    def test_legacy_partial_pass_cannot_authorize_current_installation(self):
        helper = str(ROOT / "scripts/complete-usage-monitor-install.ps1").replace("'", "''")
        source = r"""
$ErrorActionPreference='Stop'
$request=Join-Path $PSScriptRoot 'request.json';$result=Join-Path $PSScriptRoot 'result.json'
[IO.File]::WriteAllText($request,'{"product_version":"0.12.1","install_after_pass":true,"always_on":true,"binary_sha256":"fixture"}')
[IO.File]::WriteAllText($result,'{"phase":"passed-local-session-interruption","incident":{"confirmed_stop":{"targeted_turns":1}},"app_preserved":true,"stop_reason_visible":true,"fixture_task_removed":true}')
try {
 & 'HELPER' -Request $request -AcceptanceResult $result
 throw 'Legacy partial acceptance authorized deployment'
} catch {
 if($_.Exception.Message -notmatch 'Legacy global developer deployment is retired'){throw}
 @{legacy_partial_rejected=$true}|ConvertTo-Json -Compress
}
""".replace("'HELPER'", "'" + helper + "'")
        self.assertTrue(self.powershell(source)["legacy_partial_rejected"])

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
$clock=[Diagnostics.Stopwatch]::StartNew();$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
$sessions=@{fixture=@{id='fixture';subscribed=$true;ready=$false;dirty=$true;active=@();statuses=@{}}}
$snapshot=[pscustomobject]@{hostId='local';conversationId='fixture';change=[pscustomobject]@{type='snapshot';conversationState=[pscustomobject]@{
 id='fixture';hostId='local';turns=@();title='fixture';threadRuntimeStatus=@{type='active'};threadGoal=@{status='active'}
 turnHistory=@{kind='canonical';history=@{islands=@(@{entries=@(@{value='one'},@{value='two'})});entitiesByKey=[pscustomobject]@{one=@{turnId='old';status='completed';items=@('not retained')};two=@{turnId='current';status='inProgress';items=@('not retained')}}}}
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

    def test_reasoning_text_deltas_do_not_trigger_full_history_resubscription(self):
        source=self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew()
$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
$sessions=@{fixture=@{id='fixture';subscribed=$true;ready=$true;dirty=$false}}
$value=[pscustomobject]@{hostId='local';conversationId='fixture';change=[pscustomobject]@{type='patches';patches=@(@{path=@('turnHistory','history','entitiesByKey','current','items',0,'text');op='replace';value='delta'})}}
Snapshot $value
if($sessions.fixture.dirty){throw 'Text delta caused a history resubscription'}
$value.change.patches=@(@{path=@('turnHistory','history','entitiesByKey','current','status');op='replace';value='interrupted'})
Snapshot $value
if(-not $sessions.fixture.dirty){throw 'Turn status change was lost'}
@{content_ignored=$true;status_observed=$true} | ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["status_observed"])

    def test_wrong_desktop_snapshot_cannot_confirm_a_different_session(self):
        source=self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew()
$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
$sessions=@{fixture=@{id='fixture';subscribed=$true;ready=$false}}
$snapshot=[pscustomobject]@{hostId='local';conversationId='fixture';change=[pscustomobject]@{type='snapshot';conversationState=@{id='different';hostId='local'}}}
try {Snapshot $snapshot;throw 'Wrong session accepted'} catch {if($_.Exception.Message -notmatch 'Mismatched desktop snapshot'){throw}}
if($sessions.fixture.ready){throw 'Wrong session changed readiness'}
@{rejected=$true} | ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["rejected"])

    def test_desktop_pipe_handles_split_frames_and_rejects_oversized_frames(self):
        source=(ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1").read_text(encoding='utf-8')
        declaration=source.split("-TypeDefinition @'",1)[1].split("'@",1)[0]
        declaration=declaration.replace('"codex-ipc"','"hive-frame-test-" + System.Diagnostics.Process.GetCurrentProcess().Id')
        fixture="$ErrorActionPreference='Stop'\nAdd-Type -ReferencedAssemblies 'System.Web.Extensions','System.Core' -TypeDefinition @'\n"+declaration+"\n'@\n"+r"""
$name='hive-frame-test-'+$PID
$server=[IO.Pipes.NamedPipeServerStream]::new($name,[IO.Pipes.PipeDirection]::InOut,1,[IO.Pipes.PipeTransmissionMode]::Byte,[IO.Pipes.PipeOptions]::Asynchronous)
$waiting=$server.WaitForConnectionAsync()
$client=[HiveDesktopPipe]::new();$null=$waiting.GetAwaiter().GetResult();$client.Start()
try {
 $bytes=[Text.Encoding]::UTF8.GetBytes('{"type":"response","requestId":"fixture"}')
 $header=[BitConverter]::GetBytes([int]$bytes.Length)
 $server.Write($header,0,2);$server.Flush();Start-Sleep -Milliseconds 20
 if($null -ne $client.Poll()){throw 'Partial header delivered'}
 $server.Write($header,2,2);$server.Write($bytes,0,3);$server.Flush();Start-Sleep -Milliseconds 20
 if($null -ne $client.Poll()){throw 'Partial body delivered'}
 $server.Write($bytes,3,$bytes.Length-3);$server.Flush()
 $deadline=[DateTime]::UtcNow.AddSeconds(2);$frame=$null
 do {Start-Sleep -Milliseconds 10;$frame=$client.Poll()} while($null -eq $frame -and [DateTime]::UtcNow -lt $deadline)
 if(($frame|ConvertFrom-Json).requestId -ne 'fixture'){throw 'Complete frame missing'}
 $bad=[BitConverter]::GetBytes([int]67108865);$server.Write($bad,0,4);$server.Flush()
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

    def test_unresponsive_pipe_writer_times_out_without_contacting_the_app(self):
        source=(ROOT / 'crates/hive-cli/src/usage_monitor/desktop.ps1').read_text(encoding='utf-8')
        declaration=source.split("-TypeDefinition @'",1)[1].split("'@",1)[0]
        declaration=declaration.replace('"codex-ipc"','"hive-write-test-" + System.Diagnostics.Process.GetCurrentProcess().Id')
        fixture="$ErrorActionPreference='Stop'\nAdd-Type -ReferencedAssemblies 'System.Web.Extensions','System.Core' -TypeDefinition @'\n"+declaration+"\n'@\n"+r"""
$name='hive-write-test-'+$PID
$server=[IO.Pipes.NamedPipeServerStream]::new($name,[IO.Pipes.PipeDirection]::InOut,1,[IO.Pipes.PipeTransmissionMode]::Byte,[IO.Pipes.PipeOptions]::Asynchronous,4096,4096)
$waiting=$server.WaitForConnectionAsync();$client=[HiveDesktopPipe]::new();$null=$waiting.GetAwaiter().GetResult();$client.Start()
try {
 $clock=[Diagnostics.Stopwatch]::StartNew();$timedOut=$false
 try {1..64|ForEach-Object {$client.Send(('x'*16300))}}
 catch {if($_.Exception.ToString() -notmatch 'Desktop write timeout'){throw};$timedOut=$true}
 if(-not $timedOut -or $client.Alive -or $clock.ElapsedMilliseconds -gt 2500){throw 'Blocked writer escaped its bounded deadline'}
 @{writer_bounded=$true;fixture_pipe_only=$true}|ConvertTo-Json -Compress
} finally {$client.Dispose();$server.Dispose()}
"""
        self.assertTrue(self.powershell(fixture)["writer_bounded"])

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

    def test_background_catalog_requires_owner_confirmation(self):
        source=(ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1").read_text(encoding="utf-8")
        body=source.split("} elseif ($request.method -eq 'thread-owner-discovery') {",1)[1].split("} elseif",1)[0]
        setup="$clock=[Diagnostics.Stopwatch]::StartNew();$catalogChecked=@{};$tracked=0;$catalogReady=$true;$request=@{thread='fixture'};function Track($id){$script:tracked++}\n"
        for response, expected, healthy in [
            ("@{resultType='error';error='no-client-found'}",0,True),
            ("@{resultType='success';handledByClientId='owner'}",1,True),
            ("@{resultType='error';error='timeout'}",0,False),
        ]:
            result=self.powershell(setup+"$message="+response+"\n"+body+"\n@{tracked=$tracked;healthy=$catalogReady}|ConvertTo-Json -Compress")
            self.assertEqual(result["tracked"],expected)
            self.assertEqual(result["healthy"],healthy)

    def test_background_catalog_rejects_duplicate_and_invalid_identifiers(self):
        source=(ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1").read_text(encoding="utf-8")
        body=source.split("} elseif ($command.operation -eq 'discover') {",1)[1].split("} elseif",1)[0]
        setup="$clock=[Diagnostics.Stopwatch]::StartNew();$catalogQueue=[Collections.Generic.Queue[string]]::new();$pending=@{};$sessions=@{};\n"
        for identities in ["@('bad')", "@('01234567-0123-0123-0123-0123456789ab','01234567-0123-0123-0123-0123456789ab')"]:
            code=setup+"$command=@{valid=$true;ids="+identities+"};$rejected=$false;try {\n"+body+"\n}catch{$rejected=$true};@{rejected=$rejected}|ConvertTo-Json -Compress"
            self.assertTrue(self.powershell(code)["rejected"])

    def test_confirmed_absent_owner_is_not_a_new_unknown_during_refresh(self):
        source=self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew()
$catalogSeen=@{old=$true;new=$true;owned=$true}
$sessions=@{owned=@{ready=$true}}
$catalogChecked=@{old=$clock.ElapsedMilliseconds}
$catalogQueue=@('old','new');$pending=@{refresh=@{method='thread-owner-discovery';thread='old'}}
if((Count-Unqualified) -ne 1){throw 'Previously qualified absence became unknown again'}
$catalogChecked.old=$clock.ElapsedMilliseconds-20001
if((Count-Unqualified) -ne 2){throw 'Expired qualification was accepted'}
@{refresh_preserved=$true;expiration_enforced=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["expiration_enforced"])

    def test_acceptance_failure_is_a_nonzero_command_result(self):
        path=str(ROOT / 'scripts/qualify-usage-monitor.ps1').replace("'","''")
        source="$ErrorActionPreference='Stop';$tokens=$null;$errors=$null;"+(
            "$tree=[Management.Automation.Language.Parser]::ParseFile('"+path+"',[ref]$tokens,[ref]$errors);if($errors){throw 'Acceptance script parsing failed'};"
            "$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)|ForEach-Object{Invoke-Expression $_.Extent.Text};\n"
        )+r"""
if((Acceptance-ExitCode @{phase='failed';fixture_task_removed=$true}) -ne 1){throw 'Failed acceptance returned success'}
if((Acceptance-ExitCode @{phase='passed-local-session-interruption';fixture_task_removed=$false}) -ne 1){throw 'Incomplete cleanup returned success'}
if((Acceptance-ExitCode @{phase='passed-local-session-interruption';fixture_task_removed=$true}) -ne 0){throw 'Completed acceptance returned failure'}
@{failure_reported=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["failure_reported"])

    def test_turn_store_orphans_and_old_progress_do_not_become_current_runs(self):
        source=self.desktop_functions()+r"""
$state=[pscustomobject]@{turns=@();turnHistory=@{kind='canonical';history=@{
 islands=@(@{entries=@(@{value='old'},@{value='latest'})})
 entitiesByKey=[pscustomobject]@{orphan=@{turnId='orphan';status='inProgress'};old=@{turnId='old';status='inProgress'};latest=@{turnId='latest';status='interrupted'}}
}}}
$turns=@(Visible-Turns $state)
if($turns.Count -ne 2 -or $turns[-1].turnId -ne 'latest' -or $turns[-1].status -ne 'interrupted'){throw 'Unreferenced or older turn selected'}
$state.turns=@([pscustomobject]@{turnId='new';status='inProgress'})
$turns=@(Visible-Turns $state)
if($turns[-1].turnId -ne 'new'){throw 'New legacy overlay omitted'}
@{canonical_membership=$true;latest_only=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["canonical_membership"])

    def test_fresh_native_interruption_is_confirmation_even_without_response(self):
        source=self.desktop_functions()+r"""
$sessions=@{fixture=@{ready=$true;observedAt=200;statuses=@{current='interrupted'};goal='paused';lastResponse='failed'}}
$target=@{thread='fixture';turn='current'}
if(-not (Confirmed-Target $target 100)){throw 'Native completion required a lost response'}
if(Confirmed-Target $target 201){throw 'Stale snapshot confirmed interruption'}
$sessions.fixture.statuses.current='inProgress'
if(Confirmed-Target $target 100){throw 'Running turn confirmed'}
@{fresh_completion=$true;stale_rejected=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["stale_rejected"])

    def offline_controller(self) -> str:
        return self.desktop_functions()+r"""
$clock=[Diagnostics.Stopwatch]::StartNew();$sessions=@{};$stop=$null
$actions=[Collections.Generic.List[object]]::new()
function Send($value){$actions.Add(@{kind='follow';id=$value.params.conversationId;following=$value.params.following})}
function Request([string]$method,$params,[int]$version,[string]$target,[string]$thread){
 if($method -ne 'thread-follower-interrupt-turn' -or $params.mode -ne 'user-stop'){throw 'Unexpected control operation'}
 $actions.Add(@{kind='interrupt';id=$thread})
}
$one='01234567-0123-0123-0123-0123456789ab'
$two='01234567-0123-0123-0123-0123456789ac'
$child='01234567-0123-0123-0123-0123456789ad'
"""

    def test_tracking_primitive_has_no_history_subscriptions_or_stop_requests(self):
        source=self.offline_controller()+r"""
Track $one;Track $two;Track $one
1..20|ForEach-Object {Advance-Stop}
if($actions.Count -ne 0 -or $sessions.Count -ne 2){throw 'Normal monitoring contacted thread history'}
if($sessions[$one].ready -or $sessions[$one].subscribed){throw 'Unobserved activity was presented as observed'}
@{metadata_only=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["metadata_only"])

    def test_background_snapshot_identifies_active_turn_before_any_stop_event(self):
        source=self.offline_controller()+r"""
Track $one
$sessions[$one].subscribed=$true
$snapshot=@{hostId='local';conversationId=$one;change=@{type='snapshot';conversationState=@{
 id=$one;hostId='local';turns=@(@{turnId='still-reasoning';status='inProgress'});
 threadRuntimeStatus=@{type='running'};threadGoal=@{status='inactive'}
}}}
Snapshot $snapshot
if(-not $sessions[$one].ready -or $sessions[$one].active.Count -ne 1 -or $sessions[$one].active[0] -ne 'still-reasoning'){throw 'Running activity before a stop was ignored'}
if($null -ne $stop -or $actions.Count -ne 0){throw 'Background activity caused an unsolicited interrupt'}
@{active_before_stop=$true;no_interrupt=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["active_before_stop"])

    def test_all_stop_requests_precede_snapshots_without_waiting_for_activity(self):
        source=self.offline_controller()+r"""
Track $one;Track $two;Track $child
$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
Advance-Stop
if($actions.Count -ne 3){throw 'History could delay the first interruption replies'}
foreach($action in $actions.GetRange(0,3)){if($action.kind -ne 'interrupt'){throw 'History read delayed another stop request'}}
foreach($session in $sessions.Values){$session.lastResponse='accepted'}
Advance-Stop
if($actions.Count -ne 4 -or $actions[3].kind -ne 'follow'){throw 'Verification did not start after replies'}
1..10|ForEach-Object {Advance-Stop}
if($actions.Count -ne 4){throw 'Multiple large snapshots were requested together'}
if($stop.targeted.Count -ne 0){throw 'Idle sessions were counted as interrupted work'}
@{requests_first=$true;no_fake_targets=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["requests_first"])

    def test_response_target_is_bound_to_incident_and_requires_fresh_completion(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
$response=@{resultType='success';result=@{ok=$true;interruptedTurnId='actual'}}
Interrupt-Response @{thread=$one;incident='stale'} $response
if($stop.targeted.Count -ne 0){throw 'Stale incident response changed new state'}
Interrupt-Response @{thread=$one;incident='fixture'} $response
if($stop.targeted.Count -ne 1){throw 'Actual interrupted turn was omitted'}
$target=$stop.targeted[$one+':actual']
if(Confirmed-Target $target $stop.began){throw 'Response alone was treated as completion'}
$sessions[$one].ready=$true;$sessions[$one].observedAt=$clock.ElapsedMilliseconds
$sessions[$one].statuses=@{actual='interrupted'};$sessions[$one].goal='paused'
if(-not (Confirmed-Target $target $stop.began)){throw 'Fresh native completion was not accepted'}
Release-Subscriptions;$stop=$null
Interrupt-Response @{thread=$one;incident='fixture'} $response
if($null -ne $stop){throw 'A delayed response recreated a cleared incident'}
@{incident_bound=$true;completion_required=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["completion_required"])

    def test_verification_unsubscribes_at_deadline_and_does_not_loop_history(self):
        source=self.offline_controller()+r"""
Track $one;Track $two;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900;Advance-Stop
$first=@($sessions.Values|Where-Object subscribed -eq $true)[0]
if($null -eq $first){throw 'Missing bounded first snapshot'}
$first.ready=$true;Advance-Stop
$initial=@($actions|Where-Object {$_.kind -eq 'follow' -and $_.following}).Count
1..10|ForEach-Object {$sessions[$one].dirty=$true;Advance-Stop}
if(@($actions|Where-Object {$_.kind -eq 'follow' -and $_.following}).Count -ne $initial){throw 'Unbounded history refresh'}
$stop.verifyUntil=$clock.ElapsedMilliseconds-1;Advance-Stop
if($sessions[$one].subscribed -or $sessions[$two].subscribed){throw 'Deadline left subscriptions open'}
if(@($actions|Where-Object {$_.kind -eq 'follow' -and -not $_.following}).Count -ne 2){throw 'Missing unsubscribe'}
$count=$actions.Count
1..10|ForEach-Object {Advance-Stop}
if($actions.Count -ne $count){throw 'Expired verification restarted by itself'}
$sessions[$one].ready=$false
$snapshot=@{hostId='local';conversationId=$one;change=@{type='snapshot';conversationState=@{id='wrong'}}}
Snapshot $snapshot
if($sessions[$one].ready){throw 'Late unrequested snapshot changed verification'}
@{deadline_closed=$true;no_history_loop=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["deadline_closed"])

    def test_pending_large_snapshot_is_not_requested_again_or_fanned_out(self):
        source=self.offline_controller()+r"""
Track $one;Track $two;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900;Advance-Stop
$first=@($sessions.Values|Where-Object subscribed -eq $true)[0]
$first.dirty=$true;$first.snapshotAt=-2000
$before=$actions.Count
1..10|ForEach-Object {Advance-Stop}
if($actions.Count -ne $before){throw 'Unread snapshot was repeated or fanned out'}
$first.ready=$true;$first.dirty=$false;Advance-Stop
if(@($sessions.Values|Where-Object subscribed -eq $true).Count -ne 2){throw 'Next snapshot never started'}
@{bounded_large_reads=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["bounded_large_reads"])

    def test_goal_pause_failure_preserves_the_actual_interrupted_turn(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
Interrupt-Response @{thread=$one;incident='fixture'} @{resultType='success';result=@{ok=$true;interruptedTurnId='actual';goalPauseError='fixture'}}
if($stop.targeted.Count -ne 1 -or $sessions[$one].lastResponse -ne 'failed'){throw 'Actual interruption was lost with goal failure'}
$sessions[$one].ready=$true;$sessions[$one].observedAt=$clock.ElapsedMilliseconds
$sessions[$one].statuses=@{actual='interrupted'};$sessions[$one].goal='active'
if(Confirmed-Target $stop.targeted[$one+':actual'] $stop.began){throw 'Active goal was accepted as stopped'}
@{actual_target_kept=$true;active_goal_rejected=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["actual_target_kept"])

    def test_idle_and_duplicate_stop_replies_do_not_invalidate_confirmed_state(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900
$request=@{thread=$one;incident='fixture'}
$actual=@{resultType='success';result=@{ok=$true;interruptedTurnId='actual'}}
Interrupt-Response $request $actual
$sessions[$one].ready=$true;$sessions[$one].dirty=$false
$sessions[$one].observedAt=$clock.ElapsedMilliseconds;$sessions[$one].goal='paused';$sessions[$one].statuses=@{actual='interrupted'}
1..10|ForEach-Object {
 Interrupt-Response $request @{resultType='success';result=@{ok=$true;interruptedTurnId=$null}}
 Interrupt-Response $request $actual
}
if($sessions[$one].dirty -or $stop.targeted.Count -ne 1){throw 'No-op replies invalidated an observed stop'}
@{no_op_preserved=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["no_op_preserved"])

    def test_unread_session_is_checked_before_refreshing_an_earlier_session(self):
        source=self.offline_controller()+r"""
Track $one;Track $two;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds());$stop.age_at_begin=1900;Advance-Stop
$first=@($sessions.Values|Where-Object subscribed -eq $true)[0]
$second=@($sessions.Values|Where-Object subscribed -eq $false)[0]
$first.ready=$true;$first.dirty=$true;$first.snapshotAt=-2000
Advance-Stop
if(-not $second.subscribed -or -not $first.ready){throw 'Repeated first read starved an unread session'}
@{unread_not_starved=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["unread_not_starved"])

    def test_user_resumed_run_is_not_interrupted_again_for_the_same_event(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
Advance-Stop
$sessions[$one].lastStop=-5000
Interrupt-Response @{thread=$one;incident='fixture';at=0} @{resultType='success';result=@{ok=$true;interruptedTurnId='first'}}
$sessions[$one].active=@('user-resumed');$sessions[$one].lastResponse='accepted'
1..20|ForEach-Object {Advance-Stop}
if(@($actions|Where-Object kind -eq 'interrupt').Count -ne 1){throw 'User-resumed work was interrupted again'}
@{single_stop=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["single_stop"])

    def test_new_chat_after_detection_is_not_in_the_stop_scope(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
Advance-Stop;Track $two;Advance-Stop
if(@($actions|Where-Object {$_.kind -eq 'interrupt' -and $_.id -eq $two}).Count){throw 'New chat was stopped by an old event'}
@{new_chat_allowed=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["new_chat_allowed"])

    def test_threshold_scope_does_not_interrupt_or_wait_for_other_conversations(self):
        source=self.offline_controller()+r"""
Track $one;Track $two
$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()) @((Session-Digest $one))
Advance-Stop
if(@($actions|Where-Object kind -eq 'interrupt').Count -ne 1){throw 'Scoped threshold interrupted another conversation'}
if(@($actions|Where-Object {$_.kind -eq 'interrupt' -and $_.id -eq $two}).Count){throw 'Other project received interruption'}
$stop.age_at_begin=1900;$sessions[$one].lastResponse='accepted';$sessions[$two].lastResponse='none'
Advance-Stop
if(-not $sessions[$one].subscribed){throw 'Other conversation delayed scoped confirmation'}
@{scope_preserved=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["scope_preserved"])

    def test_owner_known_at_detection_is_stopped_when_discovery_finishes(self):
        source=self.offline_controller()+r"""
Track $one;$catalogSeen=@{};$catalogSeen[$two]=$true
$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
Advance-Stop;Track $two;Advance-Stop
if(@($actions|Where-Object kind -eq 'interrupt').Count -ne 2){throw 'Known pending owner was missed'}
@{known_owner_stopped=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["known_owner_stopped"])

    def test_expired_event_never_extends_its_deadline_or_requests_new_stops(self):
        source=self.offline_controller()+r"""
Track $one;$stop=New-Stop 'fixture' ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
Advance-Stop;$stop.verifyUntil=$clock.ElapsedMilliseconds-1;$until=$stop.verifyUntil
Register-Target $one 'late-response' 0
Advance-Stop;$count=$actions.Count
1..20|ForEach-Object {Advance-Stop}
if($stop.verifyUntil -ne $until -or $actions.Count -ne $count -or -not (Stop-Finished)){throw 'Expired event reopened control'}
@{deadline_final=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["deadline_final"])

    def test_invalid_qualification_scope_is_rejected_before_native_pipe_code(self):
        path=str(ROOT / 'crates/hive-cli/src/usage_monitor/desktop.ps1').replace("'","''")
        source="$ErrorActionPreference='Stop';$blocked=$false;try {& '"+path+"' -QualificationThread 'invalid'}catch{if($_.Exception.Message -match 'Invalid qualification scope'){$blocked=$true}else{throw}};"
        source+="if(-not $blocked -or ('HiveDesktopPipe' -as [type])){throw 'Desktop transport was loaded'};@{blocked_before_native_code=$true}|ConvertTo-Json -Compress"
        self.assertTrue(self.powershell(source)["blocked_before_native_code"])

    def test_desktop_version_and_qualification_scope_are_exact(self):
        source=self.offline_controller()+r"""
Assert-DesktopVersion '26.1013.4894.0'
Assert-DesktopVersion '26.1016.6573.0'
foreach($version in @('26.1007.9446.0','26.1013.4894.1','26.1016.6573.1','')) {
 $rejected=$false
 try{Assert-DesktopVersion $version}catch{$rejected=$true}
 if(-not $rejected){throw 'Untested desktop version accepted'}
}
$QualificationThread=$one
Track $one;Track $two
if($sessions.Count -ne 1 -or -not $sessions.ContainsKey($one)){throw 'Qualification scope escaped'}
@{version_exact=$true;scope_exact=$true}|ConvertTo-Json -Compress
"""
        self.assertTrue(self.powershell(source)["scope_exact"])


if __name__ == "__main__":
    unittest.main()
