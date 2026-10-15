# Embedded, fixed Windows adapter. Requests are JSON on stdin, never shell fragments.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = New-Object System.Text.UTF8Encoding($false)
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

try {
    switch ($request.operation) {
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
            Emit @{ available = $true; app_count = $tree.roots.Count; roots = $roots; caller_within_app = $callerWithinApp; native_control = 'unavailable'; identity = 'current-user-OpenAI.Codex-package' }
        }
        'notify' {
            Add-Type -AssemblyName System.Windows.Forms
            Add-Type -AssemblyName System.Drawing
            $form=[Windows.Forms.Form]::new()
            $form.Text='Aigent Hive — 사용량 보호 상태'
            $form.Width=680;$form.Height=360;$form.StartPosition='CenterScreen'
            $label=[Windows.Forms.Label]::new();$label.Dock='Fill';$label.Padding=[Windows.Forms.Padding]::new(20)
            $label.Font=[Drawing.Font]::new('Malgun Gothic',11)
            $baseMessage=[string]$request.message
            $label.Text=$baseMessage
            $form.Controls.Add($label)
            $button=[Windows.Forms.Button]::new();$button.Text='창 닫기 (작업 재개 없음)';$button.Dock='Bottom';$button.Height=38
            $button.Add_Click({$form.Close()});$form.Controls.Add($button)
            $statePath=Join-Path ([string]$request.user_root) '.hive/runtime/usage-monitor/state.json'
            $timer=[Windows.Forms.Timer]::new();$timer.Interval=1000
            $timer.Add_Tick({
                try {
                    $state=[IO.File]::ReadAllText($statePath) | ConvertFrom-Json
                    if ($null -eq $state.incident) {$label.Text="중단 기록 확인 완료. 작업은 자동으로 재개하지 않습니다.";return}
                    if ($state.incident.digest -ne $request.incident) {$label.Text='새 보호 사건이 발생했습니다. 최신 상태 창을 확인해 주세요.';return}
                    $reason=switch ($state.incident.reason) {'usage-reset' {'사용량 초기화 감지'} 'sensor-unresponsive' {'30초 동안 정상 사용량 조회 없음'} default {'감시 상태 오류'}}
                    $appText=if ($state.app_alive) {'Codex 앱 유지'} else {'앱 생존 확인 필요'}
                    $status=$(if ($state.incident.stop_confirmed) {'작업 중단 확인'} elseif ($state.incident.stop_error) {'작업 중단 확인 실패'} else {'작업 중단 확인 대기'})+' — '+$appText
                    $connection=if ($state.control_connected) {'연결됨'} else {'연결 확인 필요'}
                    $label.Text=$reason+"`r`n`r`n"+$status+"`r`n중단 대상: "+$state.incident.targeted_turns+" / 확인: "+$state.incident.confirmed_turns+"`r`n현재 실행 중: "+$state.active_turns+" / 제어 연결: "+$connection+"`r`n`r`n명시 확인 전 보호 잠금 유지. 창을 닫아도 작업 재개 없음."
                } catch {$label.Text=$baseMessage+"`r`n상태를 읽을 수 없습니다. 중단 성공 여부를 확인해 주세요."}
            })
            $form.Add_Shown({[Console]::Out.WriteLine('{"notified":true}');[Console]::Out.Flush();$timer.Start()})
            $null=$form.ShowDialog()
            $timer.Dispose();$form.Dispose()
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
