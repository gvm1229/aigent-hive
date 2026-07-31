param(
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Version = "__AIGENT_HIVE_PRODUCT_VERSION__",
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-test\.[1-9][0-9]*$')]
    [string]$PackageVersion = "__AIGENT_HIVE_PACKAGE_VERSION__",
    [string]$Prefix = "$env:LOCALAPPDATA\AigentHive"
)

$ErrorActionPreference = "Stop"
$AuthorizedSignerThumbprint = "__AIGENT_HIVE_WINDOWS_CERTIFICATE_THUMBPRINT__"
$ExpectedArchiveSha256 = "__AIGENT_HIVE_SHA256_X86_64_PC_WINDOWS_MSVC__"
if ($AuthorizedSignerThumbprint -like "__AIGENT_HIVE_*") {
    $AuthorizedSignerThumbprint = ""
}
if (
    $AuthorizedSignerThumbprint -ne "" -and
    $AuthorizedSignerThumbprint -notmatch '^[0-9A-F]{40}$'
) {
    throw "installer does not contain an authorized Windows signer identity"
}
if ($ExpectedArchiveSha256 -notmatch '^[0-9a-f]{64}$') {
    throw "installer does not contain the release archive SHA-256"
}
if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Aigent Hive supports Windows x86_64 only"
}

function Test-HiveVersionOutput {
    param(
        [AllowEmptyString()]
        [string]$Output,
        [Parameter(Mandatory = $true)]
        [string]$ExpectedVersion
    )

    $escapedVersion = [regex]::Escape($ExpectedVersion)
    return $Output -cmatch (
        "^hive $escapedVersion \(released [0-9]{4}-[0-9]{2}-[0-9]{2}\)$"
    )
}

function Assert-SafeDirectoryChain {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $fullPath = [IO.Path]::GetFullPath($Path)
    $pathRoot = [IO.Path]::GetPathRoot($fullPath)
    if ([string]::IsNullOrEmpty($pathRoot)) {
        throw "install path contains a reparse point or non-directory"
    }
    $currentPath = $pathRoot
    $relativePath = $fullPath.Substring($pathRoot.Length)
    $components = $relativePath.Split(
        [char[]]@([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar),
        [System.StringSplitOptions]::RemoveEmptyEntries
    )
    foreach ($component in $components) {
        if ($component -eq "." -or $component -eq "..") {
            throw "install path contains a reparse point or non-directory"
        }
        $currentPath = Join-Path $currentPath $component
        $item = Get-Item -LiteralPath $currentPath -Force -ErrorAction SilentlyContinue
        if ($null -eq $item) {
            New-Item -ItemType Directory -Path $currentPath -ErrorAction Stop | Out-Null
            $item = Get-Item -LiteralPath $currentPath -Force -ErrorAction Stop
        }
        if (
            -not $item.PSIsContainer -or
            ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)
        ) {
            throw "install path contains a reparse point or non-directory"
        }
    }
}

function Get-ValidatedDirectReceipt {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ReceiptPath
    )

    $receiptItem = Get-Item -LiteralPath $ReceiptPath -Force -ErrorAction SilentlyContinue
    if (
        $null -eq $receiptItem -or
        $receiptItem.PSIsContainer -or
        ($receiptItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
    ) {
        throw "existing hive binary is not owned by the direct installer"
    }
    $receipt = Get-Content -LiteralPath $ReceiptPath -Raw | ConvertFrom-Json
    $expectedProperties = @(
        "artifact_sha256",
        "owner",
        "package_version",
        "product",
        "schema_version",
        "version"
    )
    $actualProperties = @($receipt.PSObject.Properties.Name | Sort-Object)
    if (
        (Compare-Object $expectedProperties $actualProperties) -or
        $receipt.schema_version -ne 1 -or
        $receipt.owner -ne "direct" -or
        $receipt.product -ne "aigent-hive" -or
        $receipt.version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' -or
        $receipt.package_version -notmatch (
            '^' + [regex]::Escape($receipt.version) + '-test\.[1-9][0-9]*$'
        ) -or
        $receipt.artifact_sha256 -notmatch '^sha256:[0-9a-f]{64}$'
    ) {
        throw "existing hive binary is not owned by the direct installer"
    }
    return $receipt
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
    $priorReceipt = Get-ValidatedDirectReceipt -ReceiptPath $ReceiptPath
    $priorDigest = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash.ToLowerInvariant()
    if (
        $priorReceipt.artifact_sha256 -ne "sha256:$priorDigest"
    ) {
        throw "existing hive binary is not owned by the direct installer"
    }
    $priorVersionOutput = & $Destination --version
    if (-not (Test-HiveVersionOutput `
        -Output $priorVersionOutput `
        -ExpectedVersion $priorReceipt.version
    )) {
        throw "existing hive binary is not owned by the direct installer"
    }
}

function Move-InstallFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Source,
        [Parameter(Mandatory = $true)]
        [string]$Destination,
        [Parameter(Mandatory = $true)]
        [bool]$Replace
    )

    $sourceItem = Get-Item -LiteralPath $Source -Force -ErrorAction Stop
    $destinationItem = Get-Item `
        -LiteralPath $Destination `
        -Force `
        -ErrorAction SilentlyContinue
    if (
        $sourceItem.PSIsContainer -or
        ($sourceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        (
            $null -ne $destinationItem -and (
                $destinationItem.PSIsContainer -or
                ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
            )
        )
    ) {
        throw "install file target is not a regular leaf"
    }
    if ($Replace -and $null -ne $destinationItem) {
        $destinationDirectory = Split-Path -Parent $Destination
        $backup = Join-Path $destinationDirectory (
            ".hive-replace-backup-" + [Guid]::NewGuid().ToString("N")
        )
        [IO.File]::Replace($Source, $Destination, $backup)
        $backupItem = Get-Item -LiteralPath $backup -Force -ErrorAction Stop
        if (
            $backupItem.PSIsContainer -or
            ($backupItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
        ) {
            throw "install replacement backup is not a regular leaf"
        }
        Remove-Item -LiteralPath $backup -Force
    } else {
        [IO.File]::Move($Source, $Destination)
    }
}

function Repair-PendingDirectInstall {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Destination,
        [Parameter(Mandatory = $true)]
        [string]$ReceiptPath,
        [Parameter(Mandatory = $true)]
        [string]$PendingReceiptPath
    )

    $pendingItem = Get-Item -LiteralPath $PendingReceiptPath -Force -ErrorAction SilentlyContinue
    if ($null -eq $pendingItem) {
        return
    }
    $destinationDirectory = Split-Path -Parent $Destination
    $receiptDirectory = Split-Path -Parent $ReceiptPath
    Assert-SafeDirectoryChain -Path $destinationDirectory
    Assert-SafeDirectoryChain -Path $receiptDirectory
    $pendingReceipt = Get-ValidatedDirectReceipt -ReceiptPath $PendingReceiptPath
    $destinationItem = Get-Item -LiteralPath $Destination -Force -ErrorAction SilentlyContinue
    $receiptItem = Get-Item -LiteralPath $ReceiptPath -Force -ErrorAction SilentlyContinue
    if (
        $null -ne $destinationItem -and
        -not $destinationItem.PSIsContainer -and
        -not ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
    ) {
        $destinationDigest = (
            Get-FileHash -LiteralPath $Destination -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        if ($pendingReceipt.artifact_sha256 -eq "sha256:$destinationDigest") {
            if ($null -ne $receiptItem) {
                Get-ValidatedDirectReceipt -ReceiptPath $ReceiptPath | Out-Null
            }
            Assert-SafeDirectoryChain -Path $destinationDirectory
            Assert-SafeDirectoryChain -Path $receiptDirectory
            Move-InstallFile `
                -Source $PendingReceiptPath `
                -Destination $ReceiptPath `
                -Replace $true
            return
        }
    }
    if ($null -ne $destinationItem -and $null -ne $receiptItem) {
        $priorReceipt = Get-ValidatedDirectReceipt -ReceiptPath $ReceiptPath
        if (
            -not $destinationItem.PSIsContainer -and
            -not ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint)
        ) {
            $destinationDigest = (
                Get-FileHash -LiteralPath $Destination -Algorithm SHA256
            ).Hash.ToLowerInvariant()
            if ($priorReceipt.artifact_sha256 -eq "sha256:$destinationDigest") {
                Assert-SafeDirectoryChain -Path $destinationDirectory
                Assert-SafeDirectoryChain -Path $receiptDirectory
                Remove-Item -LiteralPath $PendingReceiptPath -Force
                return
            }
        }
    }
    if ($null -eq $destinationItem -and $null -eq $receiptItem) {
        Assert-SafeDirectoryChain -Path $destinationDirectory
        Assert-SafeDirectoryChain -Path $receiptDirectory
        Remove-Item -LiteralPath $PendingReceiptPath -Force
        return
    }
    throw "existing hive install transaction is not recoverable"
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

$npmPackage = "win32-x64"
$archive = "$npmPackage-$PackageVersion.tgz"
$base = "https://registry.npmjs.org/@aigent-hive/$npmPackage/-"
$work = Join-Path ([IO.Path]::GetTempPath()) ("aigent-hive-install-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
$stagedBinary = $null
$stagedReceipt = $null
try {
    $archivePath = Join-Path $work $archive
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile $archivePath
    $actual = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ExpectedArchiveSha256 -ne $actual) {
        throw "release archive SHA-256 verification failed"
    }
    $tarCommand = Get-Command tar.exe -ErrorAction SilentlyContinue
    if ($null -eq $tarCommand) {
        throw "Windows tar.exe is required to install the npm package"
    }
    $entries = @(& $tarCommand.Source -tzf $archivePath)
    if ($LASTEXITCODE -ne 0) {
        throw "release archive could not be listed"
    }
    $expectedEntries = @(
        "package/LICENSE",
        "package/README.md",
        "package/bin/hive.exe",
        "package/package.json"
    )
    if (
        $entries.Count -ne $expectedEntries.Count -or
        (Compare-Object `
            -ReferenceObject ($expectedEntries | Sort-Object) `
            -DifferenceObject ($entries | Sort-Object))
    ) {
        throw "release archive contains an unexpected path"
    }
    $verboseEntries = @(& $tarCommand.Source -tvzf $archivePath)
    if (
        $LASTEXITCODE -ne 0 -or
        ($verboseEntries | Where-Object { -not $_.StartsWith("-") })
    ) {
        throw "release archive contains an unsafe or nonregular entry"
    }
    & $tarCommand.Source -xzf $archivePath -C $work `
        "package/bin/hive.exe" "package/LICENSE"
    if ($LASTEXITCODE -ne 0) {
        throw "release archive could not be extracted"
    }
    $binary = Join-Path $work "package\bin\hive.exe"
    if ($AuthorizedSignerThumbprint -ne "") {
        $signature = Get-AuthenticodeSignature -LiteralPath $binary
        Assert-AuthorizedAuthenticodeSignature `
            -Signature $signature `
            -AuthorizedThumbprint $AuthorizedSignerThumbprint
    }
    if (-not (Test-HiveVersionOutput `
        -Output (& $binary --version) `
        -ExpectedVersion $Version
    )) {
        throw "signed binary version differs from requested release"
    }

    $binDirectory = Join-Path $Prefix "bin"
    $shareDirectory = Join-Path $Prefix "share\aigent-hive"
    Assert-SafeDirectoryChain -Path $Prefix
    Assert-SafeDirectoryChain -Path $binDirectory
    Assert-SafeDirectoryChain -Path $shareDirectory
    $destination = Join-Path $binDirectory "hive.exe"
    $receiptPath = Join-Path $shareDirectory "install-receipt.json"
    $pendingReceiptPath = Join-Path $shareDirectory "install-receipt.pending.json"
    Repair-PendingDirectInstall `
        -Destination $destination `
        -ReceiptPath $receiptPath `
        -PendingReceiptPath $pendingReceiptPath
    Assert-ExistingDirectInstall -Destination $destination -ReceiptPath $receiptPath
    $stagedBinary = Join-Path $binDirectory (".hive-install-" + [Guid]::NewGuid())
    $stagedReceipt = Join-Path $shareDirectory (".install-receipt-" + [Guid]::NewGuid())
    Assert-SafeDirectoryChain -Path $binDirectory
    Assert-SafeDirectoryChain -Path $shareDirectory
    [IO.File]::Copy($binary, $stagedBinary, $false)
    $binaryDigest = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
    $receiptJson = @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = $Version
        package_version = $PackageVersion
        artifact_sha256 = "sha256:$binaryDigest"
    } | ConvertTo-Json -Compress
    $receiptBytes = [Text.UTF8Encoding]::new($false).GetBytes($receiptJson)
    $receiptStream = [IO.File]::Open(
        $stagedReceipt,
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::None
    )
    try {
        $receiptStream.Write($receiptBytes, 0, $receiptBytes.Length)
    }
    finally {
        $receiptStream.Dispose()
    }
    Assert-SafeDirectoryChain -Path $binDirectory
    Assert-SafeDirectoryChain -Path $shareDirectory
    if ($null -ne (
        Get-Item -LiteralPath $pendingReceiptPath -Force -ErrorAction SilentlyContinue
    )) {
        throw "existing hive install transaction is not recoverable"
    }
    Move-InstallFile `
        -Source $stagedReceipt `
        -Destination $pendingReceiptPath `
        -Replace $false
    $stagedReceipt = $null
    Assert-SafeDirectoryChain -Path $binDirectory
    Assert-SafeDirectoryChain -Path $shareDirectory
    Move-InstallFile `
        -Source $stagedBinary `
        -Destination $destination `
        -Replace $true
    $stagedBinary = $null
    Assert-SafeDirectoryChain -Path $binDirectory
    Assert-SafeDirectoryChain -Path $shareDirectory
    Move-InstallFile `
        -Source $pendingReceiptPath `
        -Destination $receiptPath `
        -Replace $true
    Write-Output "installed hive $Version to $binDirectory\hive.exe"
}
finally {
    foreach ($stagedPath in @($stagedBinary, $stagedReceipt)) {
        if ($null -ne $stagedPath -and (Test-Path -LiteralPath $stagedPath)) {
            Remove-Item -LiteralPath $stagedPath -Force
        }
    }
    if (Test-Path -LiteralPath $work) {
        Remove-Item -LiteralPath $work -Recurse -Force
    }
}
