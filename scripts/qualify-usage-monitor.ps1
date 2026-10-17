# Nonshipping Windows acceptance collector. The execute mode interrupts sessions and keeps Codex open.
param(
    [Parameter(Mandatory=$true)][string]$SourceRoot,
    [Parameter(Mandatory=$true)][string]$Hive,
    [Parameter(Mandatory=$true)][string]$FixtureRoot,
    [switch]$Execute,
    [switch]$ConfirmStopSessions,
    [string]$CollectorTask,
    [string]$InstallRequest,
    [int]$ExpectedAppPid,
    [long]$ExpectedAppTicks,
    [switch]$WaitForTrigger,
    [string]$PythonExe='python'
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$utf8 = New-Object Text.UTF8Encoding($false)
[Console]::OutputEncoding=$utf8
[Console]::InputEncoding=$utf8
$source = [IO.Path]::GetFullPath($SourceRoot).TrimEnd('\')
$fixture = [IO.Path]::GetFullPath($FixtureRoot).TrimEnd('\')
$allowed = Join-Path $source 'tests\work\usage-monitor-acceptance'
if (-not $fixture.StartsWith($allowed + '\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Acceptance fixture outside tests/work ownership' }
foreach ($candidate in @($source,$fixture)) {
    $path = $candidate
    while ($path) {
        if ((Test-Path -LiteralPath $path) -and ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Linked acceptance path refused' }
        $parent = [IO.Path]::GetDirectoryName($path)
        if ($parent -eq $path) { break }; $path = $parent
    }
}
$hivePath = [IO.Path]::GetFullPath($Hive)
if (-not $hivePath.StartsWith($source + '\target\debug\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Expected source-built acceptance binary' }
$reportPath = Join-Path $fixture 'result.json'
if (-not $Execute) {
    if ($ConfirmStopSessions) { throw 'No stop confirmation for prepare mode' }
    $configDir = Join-Path $fixture '.hive\config'
    $installDir = Join-Path $fixture '.hive\install'
    $null = New-Item -ItemType Directory -Path $configDir,$installDir -Force
    [IO.File]::WriteAllText((Join-Path $configDir 'user-setup.yml'),"schema_version: 1`n",$utf8)
    [IO.File]::WriteAllText((Join-Path $installDir 'codex.json'),'{"product_version":"0.12.1","acceptance_fixture":true}',$utf8)
    $stream = [IO.File]::OpenRead($hivePath)
    try { $hash = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLower() }
    finally { $stream.Dispose() }
    @{phase='prepared';scope='isolated-user-root,synthetic-reset,real-local-session-interruption,app-preserved';binary_sha256=$hash;report=$reportPath;installed_to_real_user_root=$false} | ConvertTo-Json -Compress
    exit 0
}
if (-not $ConfirmStopSessions) { throw 'Actual session interruption requires -ConfirmStopSessions' }
if ($CollectorTask -notmatch '^AigentHive-UsageAcceptance-[a-f0-9]{16}$') { throw 'Execute only through the independent acceptance collector task' }
$monitorEnabled = $false
$result = @{schema_version=1;phase='started';host='Windows Codex';synthetic_reset=$true;real_provider_reset=$false;installed_to_real_user_root=$false;started_at=[DateTime]::UtcNow.ToString('o')}
function Acceptance-ExitCode($record) {
    if($record.phase -eq 'passed-local-session-interruption' -and $record.fixture_task_removed){return 0}
    return 1
}
function Record { [IO.File]::WriteAllText($reportPath,($result | ConvertTo-Json -Depth 15),$utf8) }
function Read-State {
    for($attempt=0;$attempt -lt 6;$attempt++) {
        try {
            $stream=[IO.File]::Open($statePath,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
            try {
                if($stream.Length -gt 65536){throw 'Monitor state exceeds its limit'}
                $reader=[IO.StreamReader]::new($stream,[Text.Encoding]::UTF8,$true)
                try {return ($reader.ReadToEnd() | ConvertFrom-Json)} finally {$reader.Dispose()}
            } finally {$stream.Dispose()}
        }
        catch {if($attempt -eq 5){throw};Start-Sleep -Milliseconds 20}
    }
}

if ($ExpectedAppPid -lt 1 -or $ExpectedAppTicks -lt 1) {throw 'Exact original app identity is required'}
$appPin=[Diagnostics.Process]::GetProcessById($ExpectedAppPid)
$null=$appPin.Handle
$package=@(Get-AppxPackage -Name OpenAI.Codex | Where-Object PublisherId -eq '2p2nqsd0c76g0')
if ($package.Count -ne 1 -or $appPin.StartTime.ToUniversalTime().Ticks -ne $ExpectedAppTicks -or
    -not $appPin.MainModule.FileName.StartsWith(([string]$package[0].InstallLocation+'\'),[StringComparison]::OrdinalIgnoreCase)) {throw 'Original app identity differs'}
$result.desktop_version=[string]$package[0].Version
$stream=[IO.File]::OpenRead($hivePath)
try {$result.binary_sha256=[BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLowerInvariant()}
finally {$stream.Dispose()}
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class HiveAcceptanceWindow {
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
}
"@
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
function Find-StopReason([string]$phrase='사용량 초기화 감지',[bool]$confirmed=$true) {
    try {
        $condition=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,'Aigent Hive — 사용량 보호 상태')
        $windows=[System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children,$condition)
        $monitorConfig=[IO.File]::ReadAllText((Join-Path $fixture '.hive/runtime/usage-monitor/config.json')) | ConvertFrom-Json
        $monitorExe=[IO.Path]::GetFullPath([string]$monitorConfig.executable)
        $monitorIds=@(Get-CimInstance Win32_Process | Where-Object {$_.ExecutablePath -and [IO.Path]::GetFullPath($_.ExecutablePath) -eq $monitorExe} | ForEach-Object {[int]$_.ProcessId})
        foreach ($window in $windows) {
            $panel=Get-CimInstance Win32_Process -Filter ('ProcessId='+$window.Current.ProcessId)
            if ($panel.ParentProcessId -notin $monitorIds -or $window.Current.IsOffscreen) {continue}
            $controls=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
            foreach ($control in $controls) {
                $text=$control.Current.Name
                if ($text -and $text.Contains($phrase) -and (-not $confirmed -or $text.Contains('작업 중단 확인 — Codex 앱 유지')) -and -not $control.Current.IsOffscreen) {return $true}
            }
        }
    } catch { }
    return $false
}
function Verify-NativeSubjects([string]$baseline,[string]$output) {
    $raw=& $PythonExe (Join-Path $source 'scripts/qualify-session-events.py') verify --subjects $baseline --output $output
    return ($raw | ConvertFrom-Json)
}
function Acknowledge-TestIncident {
    $pending=Read-State
    if(-not $pending.incident){throw 'Expected exact disposable test incident'}
    $raw=& $hivePath usage monitor acknowledge --user-root $fixture --incident $pending.incident.digest --output json
    if($LASTEXITCODE -ne 0){throw 'Exact test acknowledgement with fresh usage failed'}
    $receipt=$raw | ConvertFrom-Json
    if(-not $receipt.data.acknowledged -or (Read-State).incident){throw 'Test latch did not clear'}
    if($receipt.data.PSObject.Properties['controller_release_verified'] -and -not $receipt.data.controller_release_verified){throw 'Test controller did not release its stop requests'}
    return $receipt.data
}
function Test-ObservationGap {
    $configPath=Join-Path $fixture '.hive/runtime/usage-monitor/config.json'
    $approvedConfig=[IO.File]::ReadAllBytes($configPath)
    $config=[Text.Encoding]::UTF8.GetString($approvedConfig) | ConvertFrom-Json
    $null=& $hivePath usage monitor disable --user-root $fixture --output json
    if($LASTEXITCODE -ne 0){throw 'Cannot stop the owned fixture controller for recovery test'}
    $deadline=[DateTime]::UtcNow.AddSeconds(5)
    do {
        $old=@(Get-CimInstance Win32_Process | Where-Object {$_.ExecutablePath -eq $config.executable})
        if($old.Count -gt 0){Start-Sleep -Milliseconds 100}
    } while($old.Count -gt 0 -and [DateTime]::UtcNow -lt $deadline)
    if($old.Count -gt 0){throw 'Previous fixture controller did not release its lease'}
    # Restore the same approved fixture configuration to exercise controller restart.
    # The real user root is never changed; only this child gets the failed proxy.
    [IO.File]::WriteAllBytes($configPath,$approvedConfig)
    $runner=Join-Path $fixture 'blocked-query-runner.ps1'
    $body=@'
$ErrorActionPreference='Stop'
$env:HTTPS_PROXY='http://127.0.0.1:1'
$env:HTTP_PROXY='http://127.0.0.1:1'
$env:ALL_PROXY='http://127.0.0.1:1'
$env:NO_PROXY=''
& 'BINARY' usage monitor run --user-root 'ROOT' --output json
exit $LASTEXITCODE
'@
    $body=$body.Replace('BINARY',([string]$config.executable).Replace("'","''")).Replace('ROOT',$fixture.Replace("'","''"))
    [IO.File]::WriteAllText($runner,$body,$utf8)
    $powershell=Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $gap=Start-Process -FilePath $powershell -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$runner+'"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $fixture 'gap-stdout.log') -RedirectStandardError (Join-Path $fixture 'gap-stderr.log')
    try {
        $result.phase='sensor-gap-running';Record
        $deadline=[DateTime]::UtcNow.AddSeconds(75)
        do {
            Start-Sleep -Milliseconds 100
            $current=Read-State
            if($gap.HasExited){throw 'Observation-gap controller exited unexpectedly'}
        } while(-not $current.incident -and [DateTime]::UtcNow -lt $deadline)
        if($current.incident.reason -ne 'sensor-unresponsive'){throw 'Actual usage query failure did not produce the protective stop'}
        $seconds=[long]$current.incident.detected_at-[long]$current.last_success
        if($seconds -lt 60 -or $seconds -gt 70){throw 'Observation-gap protective deadline was outside 60 to 70 seconds'}
        $deadline=[DateTime]::UtcNow.AddSeconds(8)
        do {$visible=Find-StopReason '60초 동안 정상 사용량 조회 없음' $false;if(-not $visible){Start-Sleep -Milliseconds 100}} while(-not $visible -and [DateTime]::UtcNow -lt $deadline)
        if(-not $visible){throw 'Actual observation-gap reason was not visible'}
        $result.sensor_gap=@{reason=$current.incident.reason;gap_seconds=$seconds;reason_visible=$visible;query_error=$current.last_query_error;active_turns=$current.active_turns;scope='real failed native reads while test models already stopped; running-work interruption proven by reset trial'}
        Record
    } finally {
        $null=& $hivePath usage monitor disable --user-root $fixture --output json
        if($LASTEXITCODE -ne 0){throw 'Observation-gap fixture cleanup failed'}
        if(-not $gap.WaitForExit(5000)){throw 'Observation-gap controller did not stop after disable'}
    }
    $result.sensor_gap_acknowledgement=Acknowledge-TestIncident
}
try {
    Record
    $preview = & $hivePath usage monitor preview --user-root $fixture --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Monitor preview failed' }
    $enabled = & $hivePath usage monitor enable --user-root $fixture --confirm $preview.data.preview_digest --confirm-session-interruption --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw ('Isolated monitor enable failed: '+($enabled | ConvertTo-Json -Compress -Depth 6)) }
    $monitorEnabled = $true
    $sampleAfter = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $statePath = Join-Path $fixture '.hive\runtime\usage-monitor\state.json'
    $deadline = [DateTime]::UtcNow.AddSeconds(90)
    do {
        Start-Sleep -Milliseconds 250
        if (Test-Path -LiteralPath $statePath) { $state = Read-State }
    } while ((-not $state.last_success -or $state.last_success -lt $sampleAfter -or $state.observation_count -lt 4 -or -not $state.control_connected -or $state.unknown_sessions -gt 0) -and [DateTime]::UtcNow -lt $deadline)
    if (-not $state.last_success -or $state.last_success -lt $sampleAfter -or $state.incident) { throw 'Monitor did not establish a valid initial baseline' }
    if ($state.observation_count -lt 4 -or -not $state.control_connected -or $state.known_sessions -lt 1 -or $state.unknown_sessions -gt 0) {throw 'Four fresh observations and a fully known desktop scope are required'}
    $result.phase='armed';$result.first_observation=$state.last_success;$result.observation_count=$state.observation_count;Record
    if ($WaitForTrigger) {
        $deadline=[DateTime]::UtcNow.AddMinutes(3)
        while (-not (Test-Path -LiteralPath (Join-Path $fixture 'trigger.json')) -and [DateTime]::UtcNow -lt $deadline) {Start-Sleep -Milliseconds 100}
        if (-not (Test-Path -LiteralPath (Join-Path $fixture 'trigger.json'))) {throw 'Session-stop trigger deadline elapsed'}
    }

    $subjects=Join-Path $fixture 'subjects.json'
    $subjectBaseline=Join-Path $fixture 'subject-baseline.json'
    $deadline=[DateTime]::UtcNow.AddSeconds(5)
    do {
        $beforeRaw=& $PythonExe (Join-Path $source 'scripts/qualify-session-events.py') capture --subjects $subjects --output $subjectBaseline
        $before=$beforeRaw | ConvertFrom-Json
        if(-not $before.verified){Start-Sleep -Milliseconds 100}
    } while(-not $before.verified -and [DateTime]::UtcNow -lt $deadline)
    if(-not $before.verified -or @($before.subjects).Count -lt 3 -or @($before.subjects | Where-Object tool_free_required -eq $true).Count -lt 1){throw 'Parent, long reasoning and long shell subjects must all be running'}
    $result.subjects_before=$before;Record


    $shellReceipt=Join-Path $fixture 'long-shell.json'
    if(-not (Test-Path -LiteralPath $shellReceipt)){throw 'Live long shell receipt is required'}
    $shell=[IO.File]::ReadAllText($shellReceipt) | ConvertFrom-Json
    $shellPin=[Diagnostics.Process]::GetProcessById([int]$shell.pid)
    $null=$shellPin.Handle
    if($shellPin.StartTime.ToUniversalTime().Ticks -ne [long]$shell.created_ticks -or $shellPin.HasExited -or
       [DateTime]::UtcNow.AddSeconds(90) -ge [DateTime]::Parse([string]$shell.natural_end_at).ToUniversalTime()) {throw 'Long shell is absent, reused, or too near natural completion'}
    $result.long_shell_alive_before_reset=$true;Record

    # Alter only historical evidence inside this disposable fixture. The next measurement is
    # obtained through the real native sensor and evaluated by the unmodified shipping engine.
    $lockPath = Join-Path $fixture '.hive\runtime\usage-monitor\state.lock'
    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    $held = $null
    do {
        try {
            $candidate=[IO.File]::Open($lockPath,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
            try {$candidate.Lock(0,[long]::MaxValue);$held=$candidate} catch {$candidate.Dispose();throw}
        }
        catch { Start-Sleep -Milliseconds 25 }
    } while (-not $held -and [DateTime]::UtcNow -lt $deadline)
    if (-not $held) { throw 'Cannot pin fixture state for synthetic reset' }
    try {
        $state = Read-State
        if ($state.incident) { throw 'An independent incident is already pending' }
        $epoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
        foreach ($snapshot in $state.baseline) {
            $snapshot.remaining_percent = 0
            $snapshot.measured_at_unix_seconds = $epoch - 2
            $snapshot.resets_at_unix_seconds = $epoch - 1
        }
        $temporary = Join-Path $fixture 'synthetic-state.json'
        [IO.File]::WriteAllText($temporary,($state | ConvertTo-Json -Depth 15),$utf8)
        # Windows PowerShell can coerce a null string argument to an illegal empty backup path.
        [IO.File]::Replace($temporary,$statePath,(Join-Path $fixture 'state-before-injection.json'))
    } finally { $held.Dispose() }
    $result.phase='reset-injected';$result.injected_at=[DateTime]::UtcNow.ToString('o');Record
    $deadline = [DateTime]::UtcNow.AddSeconds(35)
    do {
        Start-Sleep -Milliseconds 100
        $state = Read-State
        if($state.incident -and -not $result.subjects_interrupted.verified) {
            $native=Verify-NativeSubjects $subjectBaseline (Join-Path $fixture 'subject-interruption.json')
            if($native.verified){$result.subjects_interrupted=$native;Record}
        }
    } while ((-not $state.incident.confirmed_stop) -and [DateTime]::UtcNow -lt $deadline)
    $result.incident=$state.incident
    $result.latest_control=@{known=$state.known_sessions;unknown=$state.unknown_sessions;active=$state.active_turns;connected=$state.control_connected;app_alive=$state.app_alive};Record
    if ($state.incident.reason -ne 'usage-reset' -or -not $state.incident.confirmed_stop -or
        $state.incident.confirmed_stop.targeted_turns -lt 1) { throw 'Actual session interruption not verified' }
    $result.incident=$state.incident
    $deadlineMs=[long]$state.incident.detected_at*1000+5000
    $native=$result.subjects_interrupted
    while(-not $native.verified -and [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -lt $deadlineMs) {
        $native=Verify-NativeSubjects $subjectBaseline (Join-Path $fixture 'subject-interruption.json')
        if(-not $native.verified){Start-Sleep -Milliseconds 50}
    }
    if(-not $native.verified){throw 'Native interruption of every exact test turn was not observed'}
    foreach($subject in $native.subjects) {
        $at=[DateTimeOffset]::Parse([string]$subject.native_marker_at).ToUnixTimeMilliseconds()
        if($at -lt [long]$state.incident.detected_at*1000 -or $at -gt $deadlineMs){throw 'A native test turn missed the 5 second interruption deadline'}
    }
    $result.subjects_interrupted=$native;Record
    $result.app_preserved=(-not $appPin.HasExited -and $appPin.StartTime.ToUniversalTime().Ticks -eq $ExpectedAppTicks -and [HiveAcceptanceWindow]::IsWindowVisible($appPin.MainWindowHandle))
    if (-not $result.app_preserved) {throw 'Original Codex app window did not survive'}
    $deadline=[DateTime]::UtcNow.AddSeconds(8)
    do {$result.stop_reason_visible=Find-StopReason; if(-not $result.stop_reason_visible){Start-Sleep -Milliseconds 200}} while(-not $result.stop_reason_visible -and [DateTime]::UtcNow -lt $deadline)
    if (-not $result.stop_reason_visible) {throw 'Persistent reason and confirmed status were not visible'}
    $shellPin.Refresh()
    $result.long_shell_stopped=$shellPin.HasExited
    $result.long_shell_before_natural_end=([DateTime]::UtcNow -lt [DateTime]::Parse([string]$shell.natural_end_at).ToUniversalTime())
    if(-not $result.long_shell_stopped -or -not $result.long_shell_before_natural_end){throw 'Long shell was not interrupted before its natural completion'}
    $result.long_shell_exit_ms=[DateTimeOffset]::new($shellPin.ExitTime.ToUniversalTime()).ToUnixTimeMilliseconds()-[long]$state.incident.detected_at*1000
    if($result.long_shell_exit_ms -gt 5000){throw 'Long shell missed the 5 second local stop deadline'}
    $result.message='사용량 초기화 감지로 실행 중인 작업 중단 확인. Codex 앱 유지와 중단 이유·상태 표시 확인.'
    $result.phase='passed-local-session-interruption'
    $result.completed_at=[DateTime]::UtcNow.ToString('o')
    $result.scope='synthetic historical reset, real native sensor, actual desktop session interruption and same app window survival; provider refill and server billing cutoff unproven'
    Record
    # Preserve the latch long enough to observe that the monitor does not resume Codex itself.
    Start-Sleep -Seconds 5
    $after=Read-State
    if (-not $after.incident -or $after.active_turns -gt 0) {throw 'Session resumed while protection was latched'}
    if(-not (Verify-NativeSubjects $subjectBaseline (Join-Path $fixture 'subject-latched.json')).verified){throw 'A native test subject resumed while latched'}
    $result.no_automatic_resume=$true;Record
    $result.reset_acknowledgement=Acknowledge-TestIncident
    Start-Sleep -Seconds 2
    if(-not (Verify-NativeSubjects $subjectBaseline (Join-Path $fixture 'subject-after-ack.json')).verified){throw 'Acknowledgement automatically resumed model work'}
    $result.acknowledgement_does_not_resume=$true;Record
    Test-ObservationGap
    if($appPin.HasExited){throw 'Original app did not survive observation-gap test'}
    $result.phase='passed-local-session-interruption';Record
} catch {
    $result.phase='failed';$result.error=$_.Exception.Message;$result.completed_at=[DateTime]::UtcNow.ToString('o');Record
} finally {
    if ($monitorEnabled) {
        $disabled = & $hivePath usage monitor disable --user-root $fixture --output json | ConvertFrom-Json
        $result.fixture_task_removed=($LASTEXITCODE -eq 0)
        Record
    }
}
try {
    if ($InstallRequest -and $result.phase -eq 'passed-local-session-interruption' -and $result.fixture_task_removed) {
        $result.installation_phase='installing';Record
        & (Join-Path $source 'scripts\complete-usage-monitor-install.ps1') -Request $InstallRequest -AcceptanceResult $reportPath
        if($LASTEXITCODE -ne 0){throw 'Conditional always-on installation failed; inspect its receipt'}
        if($appPin.HasExited){throw 'Original app did not survive installation and monitoring'}
        $result.installation_phase='passed-installed-always-on';$result.installed_to_real_user_root=$true;Record
    }
} catch {
    $result.phase='failed';$result.installation_phase='failed';$result.error=$_.Exception.Message;Record
} finally {
    if ($CollectorTask -match '^AigentHive-UsageAcceptance-[a-f0-9]{16}$') {
        $task=Get-ScheduledTask -TaskName $CollectorTask -ErrorAction SilentlyContinue
        if ($task -and $task.Description -eq 'Aigent Hive disposable usage acceptance collector') {
            Unregister-ScheduledTask -TaskName $CollectorTask -Confirm:$false
        }
    }
}

exit (Acceptance-ExitCode $result)
