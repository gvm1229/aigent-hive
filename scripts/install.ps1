param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Version,
    [string]$Prefix = "$env:LOCALAPPDATA\AigentHive"
)

$ErrorActionPreference = "Stop"
if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Aigent Hive supports Windows x86_64 only"
}

$triple = "x86_64-pc-windows-msvc"
$archive = "aigent-hive-$Version-$triple.zip"
$base = "https://github.com/gvm1229/aigent-hive/releases/download/v$Version"
$work = Join-Path ([IO.Path]::GetTempPath()) ("aigent-hive-install-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $archivePath = Join-Path $work $archive
    $checksumPath = "$archivePath.sha256"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile $archivePath
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile $checksumPath
    $expected = ((Get-Content -LiteralPath $checksumPath -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected.Length -ne 64 -or $expected -ne $actual) {
        throw "release archive SHA-256 verification failed"
    }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($archivePath)
    try {
        $entries = @($zip.Entries | ForEach-Object { $_.FullName })
        $expectedEntries = @("hive.exe", "LICENSE")
        if (
            $entries.Count -ne $expectedEntries.Count -or
            (Compare-Object -ReferenceObject $expectedEntries -DifferenceObject $entries)
        ) {
            throw "release archive contains an unexpected path"
        }
        if ($zip.Entries | Where-Object {
            $_.FullName.Contains("\") -or
            $_.FullName.StartsWith("/") -or
            $_.FullName.Split("/") -contains ".." -or
            $_.FullName.EndsWith("/")
        }) {
            throw "release archive contains an unsafe or nonregular entry"
        }
    }
    finally {
        $zip.Dispose()
    }
    Expand-Archive -LiteralPath $archivePath -DestinationPath $work
    $binary = Join-Path $work "hive.exe"
    $signature = Get-AuthenticodeSignature -LiteralPath $binary
    if ($signature.Status -ne [Management.Automation.SignatureStatus]::Valid) {
        throw "Authenticode verification failed: $($signature.Status)"
    }
    if ((& $binary --version) -ne "hive $Version") {
        throw "signed binary version differs from requested release"
    }

    $binDirectory = Join-Path $Prefix "bin"
    $shareDirectory = Join-Path $Prefix "share\aigent-hive"
    New-Item -ItemType Directory -Force -Path $binDirectory, $shareDirectory | Out-Null
    $destination = Join-Path $binDirectory "hive.exe"
    $receiptPath = Join-Path $shareDirectory "install-receipt.json"
    if (Test-Path -LiteralPath $destination) {
        if (
            -not (Test-Path -LiteralPath $destination -PathType Leaf) -or
            -not (Test-Path -LiteralPath $receiptPath -PathType Leaf) -or
            ((Get-Item -LiteralPath $destination).Attributes -band [IO.FileAttributes]::ReparsePoint) -or
            ((Get-Item -LiteralPath $receiptPath).Attributes -band [IO.FileAttributes]::ReparsePoint)
        ) {
            throw "existing hive binary is not owned by the direct installer"
        }
        $priorReceipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
        $expectedProperties = @(
            "artifact_sha256",
            "owner",
            "product",
            "schema_version",
            "version"
        )
        $actualProperties = @($priorReceipt.PSObject.Properties.Name | Sort-Object)
        $propertyDifference = Compare-Object $expectedProperties $actualProperties
        $priorDigest = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
        $priorVersionOutput = & $destination --version
        if (
            $propertyDifference -or
            $priorReceipt.schema_version -ne 1 -or
            $priorReceipt.owner -ne "direct" -or
            $priorReceipt.product -ne "aigent-hive" -or
            $priorVersionOutput -ne "hive $($priorReceipt.version)" -or
            $priorReceipt.artifact_sha256 -ne "sha256:$priorDigest"
        ) {
            throw "existing hive binary is not owned by the direct installer"
        }
    }
    $stagedBinary = Join-Path $binDirectory (".hive-install-" + [Guid]::NewGuid())
    $stagedReceipt = Join-Path $shareDirectory (".install-receipt-" + [Guid]::NewGuid())
    Copy-Item -LiteralPath $binary -Destination $stagedBinary
    $binaryDigest = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
    @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = $Version
        artifact_sha256 = "sha256:$binaryDigest"
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $stagedReceipt -Encoding utf8NoBOM
    Move-Item -LiteralPath $stagedBinary -Destination $destination -Force
    Move-Item -LiteralPath $stagedReceipt -Destination $receiptPath -Force
    Write-Output "installed hive $Version to $binDirectory\hive.exe"
}
finally {
    if (Test-Path -LiteralPath $work) {
        Remove-Item -LiteralPath $work -Recurse -Force
    }
}
