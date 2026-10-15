# Nonshipping Windows acceptance collector. The execute mode intentionally closes Codex.
param(
    [Parameter(Mandatory=$true)][string]$SourceRoot,
    [Parameter(Mandatory=$true)][string]$Hive,
    [Parameter(Mandatory=$true)][string]$FixtureRoot,
    [switch]$Execute,
    [switch]$ConfirmStopCodex,
    [string]$CollectorTask
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$utf8 = New-Object Text.UTF8Encoding($false)
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
    if ($ConfirmStopCodex) { throw 'No stop confirmation for prepare mode' }
    $configDir = Join-Path $fixture '.hive\config'
    $installDir = Join-Path $fixture '.hive\install'
    $null = New-Item -ItemType Directory -Path $configDir,$installDir -Force
    [IO.File]::WriteAllText((Join-Path $configDir 'user-setup.yml'),"schema_version: 1`n",$utf8)
    [IO.File]::WriteAllText((Join-Path $installDir 'codex.json'),'{"product_version":"0.12.1","acceptance_fixture":true}',$utf8)
    $stream = [IO.File]::OpenRead($hivePath)
    try { $hash = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLower() }
    finally { $stream.Dispose() }
    @{phase='prepared';scope='isolated-user-root,synthetic-reset,real-local-Codex-stop';binary_sha256=$hash;report=$reportPath;installed_to_real_user_root=$false} | ConvertTo-Json -Compress
    exit 0
}
if (-not $ConfirmStopCodex) { throw 'Actual app termination requires -ConfirmStopCodex' }
if ($CollectorTask -notmatch '^AigentHive-UsageAcceptance-[a-f0-9]{16}$') { throw 'Execute only through the independent acceptance collector task' }
$monitorEnabled = $false
$result = @{schema_version=1;phase='started';host='Windows Codex';synthetic_reset=$true;real_provider_reset=$false;installed_to_real_user_root=$false;started_at=[DateTime]::UtcNow.ToString('o')}
function Record { [IO.File]::WriteAllText($reportPath,($result | ConvertTo-Json -Depth 15),$utf8) }
try {
    Record
    $preview = & $hivePath usage monitor preview --user-root $fixture --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Monitor preview failed' }
    $enabled = & $hivePath usage monitor enable --user-root $fixture --confirm $preview.data.preview_digest --confirm-app-termination --output json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Isolated monitor enable failed' }
    $monitorEnabled = $true
    $sampleAfter = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $statePath = Join-Path $fixture '.hive\runtime\usage-monitor\state.json'
    $deadline = [DateTime]::UtcNow.AddSeconds(35)
    do {
        Start-Sleep -Milliseconds 250
        if (Test-Path -LiteralPath $statePath) { $state = [IO.File]::ReadAllText($statePath) | ConvertFrom-Json }
    } while ((-not $state.last_success -or $state.last_success -lt $sampleAfter) -and [DateTime]::UtcNow -lt $deadline)
    if (-not $state.last_success -or $state.last_success -lt $sampleAfter -or $state.incident) { throw 'Monitor did not establish a valid initial baseline' }
    $result.phase='armed'; $result.first_observation=$state.last_success; Record

    # Alter only historical evidence inside this disposable fixture. The next measurement is
    # obtained through the real native sensor and evaluated by the unmodified shipping engine.
    $lockPath = Join-Path $fixture '.hive\runtime\usage-monitor\state.lock'
    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    $held = $null
    do {
        try { $held = [IO.File]::Open($lockPath,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None) }
        catch { Start-Sleep -Milliseconds 25 }
    } while (-not $held -and [DateTime]::UtcNow -lt $deadline)
    if (-not $held) { throw 'Cannot pin fixture state for synthetic reset' }
    try {
        $state = [IO.File]::ReadAllText($statePath) | ConvertFrom-Json
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
        $state = [IO.File]::ReadAllText($statePath) | ConvertFrom-Json
    } while ((-not $state.incident.termination_verified) -and [DateTime]::UtcNow -lt $deadline)
    if ($state.incident.reason -ne 'usage-reset' -or -not $state.incident.termination_verified) { throw 'Actual local termination not verified' }
    $result.incident = $state.incident
    $result.phase='passed-local-app-termination'
    $result.completed_at=[DateTime]::UtcNow.ToString('o')
    $result.scope='synthetic historical reset, real sensor, actual app process tree; remote billing and all model-turn states unproven'
    Record
    # Preserve the latch long enough to observe that the monitor does not resume Codex itself.
    Start-Sleep -Seconds 3
} catch {
    $result.phase='failed';$result.error=$_.Exception.Message;$result.completed_at=[DateTime]::UtcNow.ToString('o');Record
} finally {
    if ($monitorEnabled) {
        $disabled = & $hivePath usage monitor disable --user-root $fixture --output json | ConvertFrom-Json
        $result.fixture_task_removed=($LASTEXITCODE -eq 0)
        Record
    }
    if ($CollectorTask -match '^AigentHive-UsageAcceptance-[a-f0-9]{16}$') {
        $task=Get-ScheduledTask -TaskName $CollectorTask -ErrorAction SilentlyContinue
        if ($task -and $task.Description -eq 'Aigent Hive disposable usage acceptance collector') {
            Unregister-ScheduledTask -TaskName $CollectorTask -Confirm:$false
        }
    }
}
