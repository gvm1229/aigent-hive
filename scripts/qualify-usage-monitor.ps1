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
    [switch]$WaitForTrigger
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
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class HiveAcceptanceWindow {
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
}
"@
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
function Find-StopReason {
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
                if ($text -and $text.Contains('사용량 초기화 감지') -and $text.Contains('작업 중단 확인 — Codex 앱 유지') -and -not $control.Current.IsOffscreen) {return $true}
            }
        }
    } catch { }
    return $false
}
try {
    Record
    $preview = & $hivePath usage monitor preview --user-root $fixture --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Monitor preview failed' }
    $enabled = & $hivePath usage monitor enable --user-root $fixture --confirm $preview.data.preview_digest --confirm-session-interruption --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Isolated monitor enable failed' }
    $monitorEnabled = $true
    $sampleAfter = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $statePath = Join-Path $fixture '.hive\runtime\usage-monitor\state.json'
    $deadline = [DateTime]::UtcNow.AddSeconds(75)
    do {
        Start-Sleep -Milliseconds 250
        if (Test-Path -LiteralPath $statePath) { $state = Read-State }
    } while ((-not $state.last_success -or $state.last_success -lt $sampleAfter -or $state.observation_count -lt 4 -or -not $state.control_connected -or $state.unknown_sessions -gt 0) -and [DateTime]::UtcNow -lt $deadline)
    if (-not $state.last_success -or $state.last_success -lt $sampleAfter -or $state.incident) { throw 'Monitor did not establish a valid initial baseline' }
    if ($state.observation_count -lt 4 -or -not $state.control_connected -or $state.known_sessions -lt 1) {throw 'Four fresh observations and a live desktop session are required'}
    $result.phase='armed';$result.first_observation=$state.last_success;$result.observation_count=$state.observation_count;Record
    if ($WaitForTrigger) {
        $deadline=[DateTime]::UtcNow.AddMinutes(3)
        while (-not (Test-Path -LiteralPath (Join-Path $fixture 'trigger.json')) -and [DateTime]::UtcNow -lt $deadline) {Start-Sleep -Milliseconds 100}
        if (-not (Test-Path -LiteralPath (Join-Path $fixture 'trigger.json'))) {throw 'Session-stop trigger deadline elapsed'}
    }


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
    } while ((-not $state.incident.confirmed_stop) -and [DateTime]::UtcNow -lt $deadline)
    if ($state.incident.reason -ne 'usage-reset' -or -not $state.incident.confirmed_stop -or
        $state.incident.confirmed_stop.targeted_turns -lt 1) { throw 'Actual session interruption not verified' }
    $result.incident=$state.incident
    $result.app_preserved=(-not $appPin.HasExited -and $appPin.StartTime.ToUniversalTime().Ticks -eq $ExpectedAppTicks -and [HiveAcceptanceWindow]::IsWindowVisible($appPin.MainWindowHandle))
    if (-not $result.app_preserved) {throw 'Original Codex app window did not survive'}
    $deadline=[DateTime]::UtcNow.AddSeconds(8)
    do {$result.stop_reason_visible=Find-StopReason; if(-not $result.stop_reason_visible){Start-Sleep -Milliseconds 200}} while(-not $result.stop_reason_visible -and [DateTime]::UtcNow -lt $deadline)
    if (-not $result.stop_reason_visible) {throw 'Persistent reason and confirmed status were not visible'}
    $shellReceipt=Join-Path $fixture 'long-shell.json'
    if(Test-Path -LiteralPath $shellReceipt) {
        $shell=[IO.File]::ReadAllText($shellReceipt) | ConvertFrom-Json
        try {$p=[Diagnostics.Process]::GetProcessById([int]$shell.pid);$null=$p.Handle;$result.long_shell_stopped=($p.StartTime.ToUniversalTime().Ticks -ne [long]$shell.created_ticks -or $p.HasExited);$p.Dispose()}
        catch [ArgumentException] {$result.long_shell_stopped=$true}
        if(-not $result.long_shell_stopped){throw 'Long shell work survived the session interruption'}
    }
    $result.message='사용량 초기화 감지로 실행 중인 작업 중단 확인. Codex 앱 유지와 중단 이유·상태 표시 확인.'
    $result.phase='passed-local-session-interruption'
    $result.completed_at=[DateTime]::UtcNow.ToString('o')
    $result.scope='synthetic historical reset, real native sensor, actual desktop session interruption and same app window survival; provider refill and server billing cutoff unproven'
    Record
    # Preserve the latch long enough to observe that the monitor does not resume Codex itself.
    Start-Sleep -Seconds 5
    $after=Read-State
    if (-not $after.incident -or $after.active_turns -gt 0) {throw 'Session resumed while protection was latched'}
    $result.no_automatic_resume=$true;Record
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
        & (Join-Path $source 'scripts\complete-usage-monitor-install.ps1') -Request $InstallRequest -AcceptanceResult $reportPath
        if($LASTEXITCODE -ne 0){throw 'Conditional always-on installation failed; inspect its receipt'}
    }
} finally {
    if ($CollectorTask -match '^AigentHive-UsageAcceptance-[a-f0-9]{16}$') {
        $task=Get-ScheduledTask -TaskName $CollectorTask -ErrorAction SilentlyContinue
        if ($task -and $task.Description -eq 'Aigent Hive disposable usage acceptance collector') {
            Unregister-ScheduledTask -TaskName $CollectorTask -Confirm:$false
        }
    }
}
