# Nonshipping, explicitly approved local deployment after actual app-stop acceptance.
param([Parameter(Mandatory=$true)][string]$Request,[Parameter(Mandatory=$true)][string]$AcceptanceResult)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$utf8=New-Object Text.UTF8Encoding($false)
[Console]::OutputEncoding=$utf8
[Console]::InputEncoding=$utf8
$approval=[IO.File]::ReadAllText($Request) | ConvertFrom-Json
$acceptance=[IO.File]::ReadAllText($AcceptanceResult) | ConvertFrom-Json
if ($approval.product_version -ne '0.12.1' -or -not $approval.install_after_pass -or -not $approval.always_on) { throw 'Missing exact conditional installation authority' }
if ($acceptance.phase -ne 'passed-local-session-interruption' -or -not $acceptance.incident.confirmed_stop -or
    $acceptance.incident.confirmed_stop.targeted_turns -lt 1 -or -not $acceptance.app_preserved -or -not $acceptance.stop_reason_visible -or -not $acceptance.fixture_task_removed) { throw 'App-stop acceptance and fixture cleanup must pass before installation' }
$source=[IO.Path]::GetFullPath($approval.source_root).TrimEnd('\')
$userRoot=[IO.Path]::GetFullPath($approval.user_root).TrimEnd('\')
$binary=[IO.Path]::GetFullPath($approval.binary)
$native=[IO.Path]::GetFullPath($approval.native_binary)
$report=[IO.Path]::GetFullPath($approval.report)
if (-not $binary.StartsWith($source+'\target\debug\',[StringComparison]::OrdinalIgnoreCase) -or
    -not $report.StartsWith($source+'\.agents\work\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Deployment paths differ from approved source ownership' }
function Hash-File([string]$path) {
    $stream=[IO.File]::OpenRead($path)
    try { return [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLower() }
    finally { $stream.Dispose() }
}
function Preserved-Digests {
    $result=@{}
    foreach($relative in @('.hive\config\user-setup.yml','.hive\config\user-feature-answers.yml','.hive\config\user-preferences.json')) {
        $path=Join-Path $userRoot $relative
        if(Test-Path -LiteralPath $path){$result[$relative]=Hash-File $path}
    }
    $knowledge=Join-Path $userRoot '.hive\knowledge'
    if(Test-Path -LiteralPath $knowledge) {
        $pending=New-Object 'System.Collections.Generic.Stack[string]'
        $pending.Push($knowledge)
        while($pending.Count -gt 0) {
            foreach($file in Get-ChildItem -LiteralPath $pending.Pop() -Force) {
                if($file.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Linked canonical knowledge refused'}
                if($file.PSIsContainer){$pending.Push($file.FullName)}
                else{$result[$file.FullName.Substring($userRoot.Length+1)]=Hash-File $file.FullName}
            }
        }
    }
    return $result
}
function Run-Hive([string[]]$arguments) {
    $raw=& $native @arguments
    if($LASTEXITCODE -ne 0){throw ('Hive step failed: '+$arguments[0]+' '+$arguments[1])}
    return ($raw | ConvertFrom-Json)
}
$result=@{schema_version=1;phase='started';product_version='0.12.1';package_identity='0.12.1-dev';installed_to_real_user_root=$true;started_at=[DateTime]::UtcNow.ToString('o')}
function Record { [IO.File]::WriteAllText($report,($result|ConvertTo-Json -Depth 12),$utf8) }
try {
    Record
    if((Hash-File $binary) -ne $approval.binary_sha256 -or (Hash-File $native) -ne $approval.native_sha256) { throw 'Approved source or active native binary changed' }
    $protected=Preserved-Digests
    $result.protected_file_count=$protected.Count
    $result.before_version=(& $native --version | Out-String).Trim()
    $result.phase='activating-local-binary';Record
    $previousDevBinary=$env:AIGENT_HIVE_DEV_BINARY
    try {
        $env:AIGENT_HIVE_DEV_BINARY=$binary
        $activationState=Join-Path $userRoot '.hive\dev-install'
        if(Test-Path -LiteralPath $activationState) {
            $activeDigest=([IO.File]::ReadAllText((Join-Path $activationState 'developer-sha256'))).Trim()
            if((Hash-File $native) -ne $activeDigest){throw 'Existing developer activation changed; no replacement'}
            & $approval.git_bash --noprofile --norc (Join-Path $source 'scripts\invoke-dev-install.sh') rollback (Split-Path $native) (Join-Path $source 'scripts\dev-install.sh') $activationState $approval.python_directory
            if($LASTEXITCODE -ne 0){throw 'Managed prior developer activation rollback failed'}
        }
        & $approval.git_bash --noprofile --norc (Join-Path $source 'scripts\invoke-dev-install.sh') global (Split-Path $native) (Join-Path $source 'scripts\dev-install.sh') (Join-Path $userRoot '.hive\dev-install') $approval.python_directory
        if($LASTEXITCODE -ne 0){throw 'Managed developer binary activation failed'}
    } finally {
        if($null -eq $previousDevBinary){Remove-Item Env:AIGENT_HIVE_DEV_BINARY -ErrorAction SilentlyContinue}else{$env:AIGENT_HIVE_DEV_BINARY=$previousDevBinary}
    }
    if((Hash-File $native) -ne $approval.binary_sha256){throw 'Activated native binary differs from accepted binary'}
    $result.after_version=(& $native --version | Out-String).Trim()
    if($result.after_version -notmatch '^AIgent Hive v0\.12\.1-dev '){throw 'Developer product identity differs'}
    $result.phase='installing-user-projection';Record
    $preview=Run-Hive -arguments @('install','--scope','user','--host','codex','--dry-run','--user-root',$userRoot,'--output','json')
    $result.install_preview_code=$preview.code
    $installed=Run-Hive -arguments @('install','--scope','user','--host','codex','--apply','--user-root',$userRoot,'--output','json')
    $result.install_code=$installed.code
    $validation=Run-Hive -arguments @('install','--scope','user','--host','codex','--validate','--user-root',$userRoot,'--output','json')
    $result.install_validation_code=$validation.code
    $after=Preserved-Digests
    foreach($key in $protected.Keys){if(-not $after.ContainsKey($key) -or $after[$key] -ne $protected[$key]){throw 'Canonical knowledge or saved preference changed'}}
    $result.canonical_preservation_verified=$true
    $result.phase='enabling-always-on';Record
    $preview=Run-Hive -arguments @('usage','monitor','preview','--user-root',$userRoot,'--output','json')
    $enabled=Run-Hive -arguments @('usage','monitor','enable','--user-root',$userRoot,'--confirm',$preview.data.preview_digest,'--confirm-session-interruption','--output','json')
    $result.monitor_task=$enabled.data.task_name
    $deadline=[DateTime]::UtcNow.AddSeconds(75)
    do {
        Start-Sleep -Seconds 1
        $status=Run-Hive -arguments @('usage','monitor','status','--user-root',$userRoot,'--output','json')
        $heartbeat=[long]$status.data.state.heartbeat
        if($status.data.state.incident){throw 'Always-on protection halted during readiness verification'}
    } while((-not $status.data.monitoring_verified -or $status.data.state.observation_count -lt 4) -and [DateTime]::UtcNow -lt $deadline)
    if(-not $status.data.configuration.enabled -or -not $status.data.monitoring_verified -or $status.data.state.observation_count -lt 4){throw 'Four real observations and desktop control did not become ready'}
    $task=Get-ScheduledTask -TaskName $result.monitor_task
    if($task.State -ne 'Running' -or @($task.Triggers).Count -lt 1){throw 'Always-on task is not running with a login trigger'}
    $probe=Run-Hive -arguments @('usage','probe-native','--host','codex','--output','json')
    $result.native_sensor_code=$probe.code
    $result.always_on_enabled=$true
    $result.controller_heartbeat=$heartbeat
    $result.monitor_mode=if($status.data.monitoring_verified){'monitoring'}else{'armed-idle-app-closed'}
    $result.backup_expires_at=[DateTime]::UtcNow.AddDays(7).ToString('o')
    $result.phase='passed-installed-always-on'
    $result.completed_at=[DateTime]::UtcNow.ToString('o');Record
    try {
        $notice=New-Object -ComObject WScript.Shell
        $null=$notice.Popup('앱 종료 검증과 0.12.1 로컬 설치, 상시 감시 등록이 끝났습니다. Codex 앱을 유지한 작업 중단을 확인했습니다.',10,'Aigent Hive',64)
    } catch { } # The persisted receipt remains authoritative if desktop notifications are unavailable.
} catch {
    $result.error=$_.Exception.Message;$result.phase='failed';$result.completed_at=[DateTime]::UtcNow.ToString('o');Record
    try {
        $notice=New-Object -ComObject WScript.Shell
        $null=$notice.Popup(('앱 종료 검증 뒤 설치를 완료하지 못했습니다. 결과 확인: '+$report),10,'Aigent Hive',48)
    } catch { }
    exit 1
}
