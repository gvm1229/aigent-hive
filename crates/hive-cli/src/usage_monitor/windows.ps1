# Embedded, fixed Windows adapter. Requests are JSON on stdin, never shell fragments.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = New-Object System.Text.UTF8Encoding($false)
$request = [Console]::In.ReadToEnd() | ConvertFrom-Json
function Emit($value) { ConvertTo-Json -InputObject $value -Depth 12 -Compress }
function Owned-Task($task,$expected) {
    if(-not $task -or $task.Description -ne 'Aigent Hive version-bound Codex usage protection' -or @($task.Actions).Count -ne 1 -or
        $task.Actions[0].Execute -ne $expected.executable -or $task.Actions[0].Arguments -ne $expected.arguments){return $false}
    $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
    if($task.Principal.LogonType -notin @('Interactive',3) -or $task.Principal.RunLevel -notin @('Limited',0)){return $false}
    try {
        $user=[string]$task.Principal.UserId
        $sid=if($user -match '^S-1-') {([Security.Principal.SecurityIdentifier]::new($user)).Value}
            else {([Security.Principal.NTAccount]::new($user)).Translate([Security.Principal.SecurityIdentifier]).Value}
        return $sid -eq $identity.User.Value
    } catch {return $false}
}
function App-Roots {
    $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    $packages = @(Get-AppxPackage -Name 'OpenAI.Codex' | Where-Object { $_.PublisherId -eq '2p2nqsd0c76g0' })
    $rows = @(Get-CimInstance Win32_Process)
    $byId = @{}
    foreach ($item in $rows) { $byId[[int]$item.ProcessId] = $item }
    $roots = @()
    foreach ($row in $rows) {
        if ($row.Name -ne 'ChatGPT.exe' -or -not $row.ExecutablePath) { continue }
        $parentRow = $byId[[int]$row.ParentProcessId]
        if ($parentRow -and $parentRow.Name -eq 'ChatGPT.exe' -and $parentRow.ExecutablePath -eq $row.ExecutablePath) { continue }
        $matched = @($packages | Where-Object {
            $root = [IO.Path]::GetFullPath($_.InstallLocation).TrimEnd('\') + '\'
            $row.ExecutablePath.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)
        })
        if ($matched.Count -ne 1) { continue }
        $owner = Invoke-CimMethod -InputObject $row -MethodName GetOwner
        if ($owner.ReturnValue -ne 0 -or ($owner.Domain + '\' + $owner.User) -ne $identity) { continue }
        $roots += $row
    }
    $version=if($packages.Count -eq 1){[string]$packages[0].Version}else{$null}
    return @{ roots = @($roots); rows = @($rows); desktop_package_version=$version }
}

try {
    switch ($request.operation) {
        'session-identity' {
            $row=Get-CimInstance Win32_Process -Filter ('ProcessId='+[int]$request.pid)
            if(-not $row -or -not $row.ExecutablePath -or -not $row.ExecutablePath.Equals([string]$request.executable,[StringComparison]::OrdinalIgnoreCase)){throw 'Session process unavailable'}
            $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwner
            if($owner.ReturnValue -ne 0 -or ($owner.Domain+'\'+$owner.User) -ne [Security.Principal.WindowsIdentity]::GetCurrent().Name){throw 'Session owner mismatch'}
            $pinned=[Diagnostics.Process]::GetProcessById([int]$row.ProcessId);$null=$pinned.Handle
            try {
                if($pinned.MainModule.FileName -ne $row.ExecutablePath){throw 'Session executable changed'}
                $identity=([string]$row.ProcessId)+'|'+$pinned.StartTime.ToUniversalTime().Ticks+'|'+$pinned.MainModule.FileName.ToLowerInvariant()
                $hash=[Security.Cryptography.SHA256]::Create()
                try {Emit @{digest='sha256:'+([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($identity)))).Replace('-','').ToLowerInvariant();uptime_ms=[long][Math]::Floor([Diagnostics.Stopwatch]::GetTimestamp()/([double][Diagnostics.Stopwatch]::Frequency/1000))}}finally{$hash.Dispose()}
            } finally {$pinned.Dispose()}
        }
        'inspect' {
            $tree = App-Roots
            $roots = @()
            foreach ($root in $tree.roots) {
                try {
                    $pinned = [Diagnostics.Process]::GetProcessById([int]$root.ProcessId)
                    $null = $pinned.Handle
                    if ([Math]::Abs(($pinned.StartTime.ToUniversalTime() - $root.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 1 -or
                        $pinned.MainModule.FileName -ne $root.ExecutablePath) { throw 'App identity changed during discovery' }
                    # CIM timestamps can lose sub-microsecond precision. Cache the handle's
                    # exact ticks so the later strict PID-reuse check compares like with like.
                    $roots += @{ pid = [int]$root.ProcessId; created_ticks = $pinned.StartTime.ToUniversalTime().Ticks; executable = $pinned.MainModule.FileName }
                    $pinned.Dispose()
                } catch [ArgumentException] { }
            }
            $callerWithinApp = $false
            if ($request.caller_pid) {
                $cursor = [int]$request.caller_pid
                for ($depth = 0; $depth -lt 64; $depth++) {
                    if (@($tree.roots | Where-Object { [int]$_.ProcessId -eq $cursor }).Count -gt 0) { $callerWithinApp = $true; break }
                    $row = @($tree.rows | Where-Object { [int]$_.ProcessId -eq $cursor })
                    if ($row.Count -ne 1 -or [int]$row[0].ParentProcessId -eq $cursor) { break }
                    $cursor = [int]$row[0].ParentProcessId
                }
            }
            $uptime=[long][Math]::Floor([Diagnostics.Stopwatch]::GetTimestamp()/([double][Diagnostics.Stopwatch]::Frequency/1000))
            Emit @{ available = $true; app_count = $tree.roots.Count; roots = $roots; caller_within_app = $callerWithinApp; desktop_package_version=$tree.desktop_package_version; uptime_ms=$uptime; identity = 'current-user-OpenAI.Codex-package' }
        }
        'register' {
            $taskName = [string]$request.task_name
            if ($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$') { throw 'Invalid owned task name' }
            $user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
            $exe = [string]$request.executable
            $args = [string]$request.arguments
            $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if ($existing) {
                if($request.allow_existing_exact -eq $true -and (Owned-Task $existing $request) -and $existing.Settings.Enabled){
                    Emit @{registered=$true;reused=$true};break
                }
                throw 'Task already exists; disable the owned monitor before replacing it'
            }
            $action = New-ScheduledTaskAction -Execute $exe -Argument $args
            $trigger = New-ScheduledTaskTrigger -AtLogOn -User $user
            $principal = New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Limited
            $settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
            $null = Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Description 'Aigent Hive version-bound Codex usage protection'
            Emit @{ registered = $true }
        }
        'start' {
            $taskName=[string]$request.task_name
            if($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$'){throw 'Invalid owned task name'}
            $task=Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if(-not (Owned-Task $task $request) -or -not $task.Settings.Enabled){throw 'Owned task is absent, disabled, or changed; no start'}
            if([string]$task.State -in @('Running','Queued')){Emit @{started=$true;reused=$true};break}
            Start-ScheduledTask -TaskName $taskName
            Emit @{ started = $true;reused=$false }
        }
        'inspect-task' {
            $taskName=[string]$request.task_name
            if($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$'){throw 'Invalid owned task name'}
            $task=Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if(-not $task){Emit @{present=$false};break}
            Emit @{present=$true;owned=(Owned-Task $task $request);enabled=[bool]$task.Settings.Enabled;state=[string]$task.State}
        }
        'unregister' {
            $taskName = [string]$request.task_name
            if ($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$') { throw 'Invalid owned task name' }
            $task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if ($task) {
                if (-not (Owned-Task $task $request)) {
                    throw 'Owned task identity differs; no removal'
                }
                Stop-ScheduledTask -TaskName $taskName
                Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
            }
            Emit @{ removed = $true }
        }
        default { throw 'Unsupported monitor operation' }
    }
} catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }
