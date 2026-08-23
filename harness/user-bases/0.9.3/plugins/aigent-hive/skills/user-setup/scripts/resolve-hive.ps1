$ErrorActionPreference = 'Stop'

function Test-HiveCandidate([string]$Candidate) {
    if ([string]::IsNullOrWhiteSpace($Candidate) -or -not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        return $null
    }
    try {
        $version = & $Candidate --version 2>$null
        if ($LASTEXITCODE -eq 0 -and $version -match '^AIgent Hive v') {
            return [pscustomobject]@{ path = (Resolve-Path -LiteralPath $Candidate).Path; version = ($version -join "`n") }
        }
    } catch { }
    return $null
}

$candidates = New-Object System.Collections.Generic.List[string]
$command = Get-Command hive -ErrorAction SilentlyContinue
if ($command -and $command.Source) { [void]$candidates.Add($command.Source) }
if ($env:OS -eq 'Windows_NT') {
    foreach ($entry in @(where.exe hive 2>$null)) {
        if ($entry) { [void]$candidates.Add($entry.Trim()) }
    }
}
try {
    $prefix = (npm prefix -g 2>$null | Select-Object -First 1).Trim()
    if ($prefix) { [void]$candidates.Add((Join-Path $prefix 'hive.cmd')) }
} catch { }

foreach ($candidate in $candidates | Select-Object -Unique) {
    $verified = Test-HiveCandidate $candidate
    if ($verified) {
        $verified | ConvertTo-Json -Compress
        exit 0
    }
}

Write-Error 'A signed Aigent Hive CLI was not found. Run npm install -g aigent-hive, then retry global setup.'
exit 1
