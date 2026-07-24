param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Version,
    [string]$Prefix = "$env:LOCALAPPDATA\AigentHive"
)

$ErrorActionPreference = "Stop"
$AuthorizedSignerThumbprint = "__AIGENT_HIVE_WINDOWS_CERTIFICATE_THUMBPRINT__"
if ($AuthorizedSignerThumbprint -notmatch '^[0-9A-F]{40}$') {
    throw "installer does not contain an authorized Windows signer identity"
}
if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Aigent Hive supports Windows x86_64 only"
}

function Assert-ExistingDirectInstall {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Destination,
        [Parameter(Mandatory = $true)]
        [string]$ReceiptPath
    )

    $destinationItem = Get-Item -LiteralPath $Destination -Force -ErrorAction SilentlyContinue
    $receiptItem = Get-Item -LiteralPath $ReceiptPath -Force -ErrorAction SilentlyContinue
    if ($null -eq $destinationItem -and $null -eq $receiptItem) {
        return
    }
    if ($null -eq $destinationItem -or $null -eq $receiptItem) {
        throw "existing hive binary is not owned by the direct installer"
    }
    if (
        $destinationItem.PSIsContainer -or
        $receiptItem.PSIsContainer -or
        ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        ($receiptItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
    ) {
        throw "existing hive binary is not owned by the direct installer"
    }
    $priorReceipt = Get-Content -LiteralPath $ReceiptPath -Raw | ConvertFrom-Json
    $expectedProperties = @(
        "artifact_sha256",
        "owner",
        "product",
        "schema_version",
        "version"
    )
    $actualProperties = @($priorReceipt.PSObject.Properties.Name | Sort-Object)
    $propertyDifference = Compare-Object $expectedProperties $actualProperties
    $priorDigest = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash.ToLowerInvariant()
    if (
        $propertyDifference -or
        $priorReceipt.schema_version -ne 1 -or
        $priorReceipt.owner -ne "direct" -or
        $priorReceipt.product -ne "aigent-hive" -or
        $priorReceipt.version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' -or
        $priorReceipt.artifact_sha256 -notmatch '^sha256:[0-9a-f]{64}$' -or
        $priorReceipt.artifact_sha256 -ne "sha256:$priorDigest"
    ) {
        throw "existing hive binary is not owned by the direct installer"
    }
    $priorVersionOutput = & $Destination --version
    if ($priorVersionOutput -ne "hive $($priorReceipt.version)") {
        throw "existing hive binary is not owned by the direct installer"
    }
}

function Assert-AuthorizedAuthenticodeSignature {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Signature,
        [Parameter(Mandatory = $true)]
        [string]$AuthorizedThumbprint
    )

    if ("$($Signature.Status)" -ne "Valid") {
        throw "Authenticode verification failed: $($Signature.Status)"
    }
    if (
        $null -eq $Signature.SignerCertificate -or
        $Signature.SignerCertificate.Thumbprint.ToUpperInvariant() -ne $AuthorizedThumbprint
    ) {
        throw "signed binary signer differs from the authorized release identity"
    }
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
    Assert-AuthorizedAuthenticodeSignature `
        -Signature $signature `
        -AuthorizedThumbprint $AuthorizedSignerThumbprint
    if ((& $binary --version) -ne "hive $Version") {
        throw "signed binary version differs from requested release"
    }

    $binDirectory = Join-Path $Prefix "bin"
    $shareDirectory = Join-Path $Prefix "share\aigent-hive"
    New-Item -ItemType Directory -Force -Path $binDirectory, $shareDirectory | Out-Null
    $destination = Join-Path $binDirectory "hive.exe"
    $receiptPath = Join-Path $shareDirectory "install-receipt.json"
    Assert-ExistingDirectInstall -Destination $destination -ReceiptPath $receiptPath
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
