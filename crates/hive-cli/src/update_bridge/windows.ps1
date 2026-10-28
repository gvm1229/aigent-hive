# This fixed helper exchanges JSON over saved pipes and uses only a verified parent's console.
$ErrorActionPreference = 'Stop'
# Resolve the fixed JSON/process cmdlets without scanning the host's module catalog on first input.
Import-Module -Name @(
    ($PSHOME + '\Modules\Microsoft.PowerShell.Utility\Microsoft.PowerShell.Utility.psd1'),
    ($PSHOME + '\Modules\CimCmdlets\CimCmdlets.psd1')
)
[Console]::InputEncoding = [Text.UTF8Encoding]::new($false,$true)
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$pipeIn = [IO.StreamReader]::new([Console]::OpenStandardInput(),[Text.UTF8Encoding]::new($false,$true))
$pipeOut = [IO.StreamWriter]::new([Console]::OpenStandardOutput(),[Text.UTF8Encoding]::new($false))
$pipeOut.AutoFlush = $true
$image = $null
$console = $null

function Reply($value) { $pipeOut.WriteLine(($value | ConvertTo-Json -Compress -Depth 12)) }
function Request {
    $reading = $pipeIn.ReadLineAsync()
    while (-not $reading.Wait(200)) { if ($null -ne $image) { $image.RequireAlive() } }
    $line = $reading.GetAwaiter().GetResult()
    if ($null -eq $line) { return $null }
    if ($line.Length -gt 1048576) { throw 'bridge input limit' }
    return ($line | ConvertFrom-Json)
}
function Sha256([string]$text) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return 'sha256:' + ([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($text))).Replace('-','').ToLowerInvariant()) }
    finally { $sha.Dispose() }
}

try {
    $request = Request
    if ($null -eq $request -or $request.operation -ne 'inspect') { throw 'bridge inspection required' }
    $self = Get-CimInstance Win32_Process -Filter "ProcessId = $PID"
    if ([int]$request.caller_pid -le 0 -or [int]$request.caller_pid -ne [int]$self.ParentProcessId) { throw 'bridge caller differs' }
    $caller = Get-CimInstance Win32_Process -Filter ("ProcessId = " + [int]$request.caller_pid)
    $parent = Get-CimInstance Win32_Process -Filter ("ProcessId = " + [int]$caller.ParentProcessId)
    if ($null -eq $parent -or -not $parent.ExecutablePath -or
        [IO.Path]::GetFileName([string]$parent.ExecutablePath) -notin @('hive.exe','hive-direct-update-helper.exe')) {
        Reply @{ event='not-legacy-parent' }; exit 0
    }
    $registrySource = '{{REGISTRY_SOURCE}}'
    if ((Sha256 $registrySource) -ne '{{REGISTRY_DIGEST}}') { throw 'public registry differs' }
    $registry = $registrySource | ConvertFrom-Json
    Add-Type -TypeDefinition ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{{PARENT_SOURCE}}'))) -Language CSharp
    $arguments = [HiveParentImage]::Arguments([string]$parent.CommandLine)
    if ($arguments[1] -ne 'update') { Reply @{ event='not-legacy-parent' }; exit 0 }
    $image = [HiveParentImage]::new([int]$parent.ProcessId)
    $created = ([DateTime]$parent.CreationDate).ToUniversalTime().ToFileTimeUtc()
    if ([Math]::Abs($image.CreationFileTime - $created) -gt 10 -or
        -not $image.ExecutablePath.Equals([string]$parent.ExecutablePath,[StringComparison]::OrdinalIgnoreCase)) { throw 'parent identity changed' }
    $owner = Invoke-CimMethod -InputObject $parent -MethodName GetOwnerSid
    if ($owner.ReturnValue -ne 0 -or $owner.Sid -ne [Security.Principal.WindowsIdentity]::GetCurrent().User.Value) { throw 'parent owner differs' }
    $verified = @()
    foreach ($candidate in $registry.images) {
        if (-not $image.HeaderMatches([int]$candidate.header_size,[uint64]$candidate.image_base,[int]$candidate.image_base_offset,[string]$candidate.header_sha256)) { continue }
        $relocations = [HiveParentImage]::DecodeRelocations([string]$candidate.relocations_delta_base64,[int]$candidate.relocation_count)
        $matches = $true
        foreach ($section in $candidate.sections) {
            $offset = 0; $size = 0
            if ($null -ne $section.import_address_mask) { $offset=[int]$section.import_address_mask.offset; $size=[int]$section.import_address_mask.size }
            if (-not $image.SectionMatches([uint64]$candidate.image_base,[uint32]$section.rva,[int]$section.size,[string]$section.sha256,$relocations,$offset,$size)) { $matches=$false; break }
        }
        if ($matches) { $verified += [string]$candidate.version }
    }
    if ($verified.Count -ne 1) { Reply @{ event='unverified-parent' }; exit 0 }
    $image.RequireAlive()
    Reply @{ event='verified-parent'; version=$verified[0]; process_id=[int]$parent.ProcessId;
        creation_filetime=$image.CreationFileTime; executable=$image.ExecutablePath; arguments=$arguments }
    $request = Request
    if ($null -eq $request -or $request.operation -notin @('open','watch')) { exit 0 }
    $image.RequireAlive()
    if ($request.operation -eq 'open') {
        Add-Type -TypeDefinition ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{{CONSOLE_SOURCE}}'))) -Language CSharp
        $callerCreated = ([DateTime]$caller.CreationDate).ToUniversalTime().ToFileTimeUtc()
        $console = [HiveUpdateConsole]::new([int]$parent.ProcessId,[int]$caller.ProcessId,$image.CreationFileTime,$callerCreated)
        Reply @{ event='opened' }
    } else { Reply @{ event='watching' } }
    while ($null -ne ($request = Request)) {
        $image.RequireAlive()
        switch ([string]$request.operation) {
            'write' { $console.Write([string]$request.text); Reply @{ event='written' } }
            'read' {
                $line = $console.ReadLine([int]$request.remaining_milliseconds)
                Reply @{ event='input'; line=$line }
                if ($null -eq $line) { exit 0 }
            }
            'close' { exit 0 }
            default { throw 'unknown bridge operation' }
        }
    }
} catch {
    # Do not forward command lines, native diagnostics, or memory contents as user-visible errors.
    Reply @{ event='closed'; reason='parent-or-console-unavailable' }
    exit 0
} finally {
    if ($null -ne $console) { $console.Dispose() }
    if ($null -ne $image) { $image.Dispose() }
}
