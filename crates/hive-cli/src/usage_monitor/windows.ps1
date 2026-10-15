# Embedded, fixed Windows adapter. Requests are JSON on stdin, never shell fragments.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$request = [Console]::In.ReadToEnd() | ConvertFrom-Json
function Emit($value) { ConvertTo-Json -InputObject $value -Depth 12 -Compress }
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
    return @{ roots = @($roots); rows = @($rows) }
}
function Pin-Processes($tree) {
    $selected = @{}
    foreach ($root in $tree.roots) { $selected[[int]$root.ProcessId] = $root }
    do {
        $changed = $false
        foreach ($row in $tree.rows) {
            if (-not $selected.ContainsKey([int]$row.ProcessId) -and $selected.ContainsKey([int]$row.ParentProcessId)) {
                $parent = $selected[[int]$row.ParentProcessId]
                if ($row.CreationDate -ge $parent.CreationDate) { $selected[[int]$row.ProcessId] = $row; $changed = $true }
            }
        }
    } while ($changed)
    $pins = @()
    foreach ($row in $selected.Values) {
        try {
            $process = [Diagnostics.Process]::GetProcessById([int]$row.ProcessId)
            # Opening a handle pins the process object; PID reuse cannot change the target.
            $null = $process.Handle
            if ([Math]::Abs(($process.StartTime.ToUniversalTime() - $row.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 1) {
                $process.Dispose(); throw 'Process identity changed'
            }
            $pins += $process
        } catch [ArgumentException] { } # Already exited before pinning.
    }
    return ,$pins
}
try {
    switch ($request.operation) {
        'inspect' {
            $tree = App-Roots
            $roots = @($tree.roots | ForEach-Object { @{ pid = [int]$_.ProcessId; created_ticks = $_.CreationDate.ToUniversalTime().Ticks; executable = $_.ExecutablePath } })
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
            Emit @{ available = $true; app_count = $tree.roots.Count; roots = $roots; caller_within_app = $callerWithinApp; native_control = 'unavailable'; identity = 'current-user-OpenAI.Codex-package' }
        }
        'halt' {
            $started = [Diagnostics.Stopwatch]::StartNew()
            # Discovery was performed before the incident. Revalidate cached identities using
            # held handles; package/owner discovery must not delay the emergency stop.
            $rows = @(Get-CimInstance Win32_Process)
            $roots = @()
            foreach ($cached in @($request.roots)) {
                if (-not $cached) { continue }
                try {
                    $process = [Diagnostics.Process]::GetProcessById([int]$cached.pid)
                    $null = $process.Handle
                    if ($process.StartTime.ToUniversalTime().Ticks -ne [long]$cached.created_ticks -or
                        $process.MainModule.FileName -ne [string]$cached.executable) { throw 'Cached process identity changed' }
                    $row = @($rows | Where-Object { [int]$_.ProcessId -eq [int]$cached.pid })
                    if ($row.Count -eq 1) { $roots += $row[0] }
                    $process.Dispose()
                } catch [ArgumentException] { }
            }
            $tree = @{ roots = $roots; rows = $rows }
            $pins = Pin-Processes $tree
            $failures = @()
            # Stop roots first to prevent respawning workers, then every pinned descendant.
            $rootIds = @($tree.roots | ForEach-Object { [int]$_.ProcessId })
            $ordered = @($pins | Sort-Object @{ Expression = { if ($rootIds -contains $_.Id) { 0 } else { 1 } } })
            foreach ($process in $ordered) {
                try { if (-not $process.HasExited) { $process.Kill() } }
                catch { $failures += [int]$process.Id }
            }
            foreach ($process in $ordered) {
                $left = [Math]::Max(0, 4500 - [int]$started.ElapsedMilliseconds)
                try { if (-not $process.WaitForExit($left)) { $failures += [int]$process.Id } }
                catch { $failures += [int]$process.Id }
            }
            $survivors = @($pins | Where-Object { -not $_.HasExited })
            $result = @{ verified = ($pins.Count -gt 0 -and $survivors.Count -eq 0 -and $failures.Count -eq 0); targeted = $pins.Count;
                survivor_count = $survivors.Count; failure_count = $failures.Count; method = 'windows-emergency'; elapsed_ms = $started.ElapsedMilliseconds }
            foreach ($process in $pins) { $process.Dispose() }
            Emit $result
        }
        'notify' {
            # A separate notification process survives the terminated desktop application.
            $message = [string]$request.message
            try {
                $null = [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
                $null = [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime]
                $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
                $escaped = [Security.SecurityElement]::Escape($message)
                $xml.LoadXml('<toast><visual><binding template="ToastGeneric"><text>Aigent Hive</text><text>' + $escaped + '</text></binding></visual></toast>')
                $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
                [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Aigent Hive').Show($toast)
            } catch {
                Add-Type -AssemblyName System.Windows.Forms
                $null = [System.Windows.Forms.MessageBox]::Show($message, 'Aigent Hive', 'OK', 'Warning')
            }
            Emit @{ notified = $true }
        }
        'register' {
            $taskName = [string]$request.task_name
            if ($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$') { throw 'Invalid owned task name' }
            $user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
            $exe = [string]$request.executable
            $args = [string]$request.arguments
            $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if ($existing) { throw 'Task already exists; disable the owned monitor before replacing it' }
            $action = New-ScheduledTaskAction -Execute $exe -Argument $args
            $trigger = New-ScheduledTaskTrigger -AtLogOn -User $user
            $principal = New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Limited
            $settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
            $null = Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Description 'Aigent Hive version-bound Codex usage protection'
            Emit @{ registered = $true }
        }
        'start' {
            Start-ScheduledTask -TaskName ([string]$request.task_name)
            Emit @{ started = $true }
        }
        'unregister' {
            $taskName = [string]$request.task_name
            if ($taskName -notmatch '^AigentHive-UsageMonitor-[a-f0-9]{16}$') { throw 'Invalid owned task name' }
            $task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            if ($task) {
                if ($task.Description -ne 'Aigent Hive version-bound Codex usage protection' -or @($task.Actions).Count -ne 1 -or
                    $task.Actions[0].Execute -ne $request.executable -or $task.Actions[0].Arguments -ne $request.arguments) {
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
