[CmdletBinding()]
param(
    [switch]$Apply,
    [switch]$ConfirmInstall,
    [ValidateSet("user", "machine")]
    [string]$Scope = "user"
)

$ErrorActionPreference = "Stop"
$PackageId = "Microsoft.PowerShell"
$PackageVersion = "7.6.4.0"
$RequiredVersion = [Version]"7.6.4"

function Get-QualifiedPowerShellVersion {
    $command = Get-Command pwsh -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        return $null
    }
    $reported = & $command.Source -NoProfile -NonInteractive -Command `
        '$PSVersionTable.PSVersion.ToString()'
    if ($LASTEXITCODE -ne 0) {
        return $null
    }
    $version = $null
    if (-not [Version]::TryParse("$reported".Trim(), [ref]$version)) {
        return $null
    }
    if (
        $version.Major -ne $RequiredVersion.Major -or
        $version.Minor -ne $RequiredVersion.Minor -or
        $version -lt $RequiredVersion
    ) {
        return $null
    }
    return $version
}

function Write-DependencyResult {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Status,
        [object]$DetectedVersion,
        [Parameter(Mandatory = $true)]
        [bool]$Changed
    )

    [ordered]@{
        schema_version = 1
        status = $Status
        dependency = "PowerShell 7 LTS"
        package_id = $PackageId
        package_version = $PackageVersion
        scope = $Scope
        detected_version = if ($null -eq $DetectedVersion) {
            $null
        } else {
            "$DetectedVersion"
        }
        changed = $Changed
        command = "winget install --id $PackageId --exact --version $PackageVersion --source winget --scope $Scope --accept-source-agreements --accept-package-agreements --disable-interactivity"
    } | ConvertTo-Json -Compress
}

$detected = Get-QualifiedPowerShellVersion
if ($null -ne $detected) {
    Write-DependencyResult `
        -Status "already-satisfied" `
        -DetectedVersion $detected `
        -Changed $false
    exit 0
}

if (-not $Apply) {
    Write-DependencyResult `
        -Status "install-required" `
        -DetectedVersion $null `
        -Changed $false
    exit 0
}
if (-not $ConfirmInstall) {
    throw "PowerShell 7 LTS installation requires -ConfirmInstall"
}

$winget = Get-Command winget -ErrorAction SilentlyContinue
if ($null -eq $winget) {
    throw "Microsoft WinGet is unavailable"
}
$arguments = @(
    "install",
    "--id", $PackageId,
    "--exact",
    "--version", $PackageVersion,
    "--source", "winget",
    "--scope", $Scope,
    "--accept-source-agreements",
    "--accept-package-agreements",
    "--disable-interactivity"
)
& $winget.Source @arguments
if ($LASTEXITCODE -ne 0) {
    throw "Microsoft PowerShell installer failed with exit code $LASTEXITCODE"
}

$detected = Get-QualifiedPowerShellVersion
if ($null -eq $detected) {
    throw "PowerShell 7 LTS requalification failed after installation"
}
Write-DependencyResult `
    -Status "installed" `
    -DetectedVersion $detected `
    -Changed $true
