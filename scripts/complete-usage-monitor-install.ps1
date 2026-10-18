# Nonshipping, explicitly approved local deployment after actual session-stop acceptance.
param([Parameter(Mandatory=$true)][string]$Request,[Parameter(Mandatory=$true)][string]$AcceptanceResult)
throw 'Legacy global developer deployment is retired. Use a public numbered test package; existing development installations require the nondeleting recovery workflow.'
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$utf8=New-Object Text.UTF8Encoding($false)
[Console]::OutputEncoding=$utf8
[Console]::InputEncoding=$utf8
$approval=[IO.File]::ReadAllText($Request) | ConvertFrom-Json
$acceptance=[IO.File]::ReadAllText($AcceptanceResult) | ConvertFrom-Json
if ($approval.product_version -ne '0.12.1' -or -not $approval.install_after_pass -or -not $approval.always_on) { throw 'Missing exact conditional installation authority' }
function Hash-File([string]$path) {
    $stream=[IO.File]::OpenRead($path)
    try { return [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLower() }
    finally { $stream.Dispose() }
}
function Parse-DeveloperDigest([string]$text) {
    $value=$text.Trim()
    if($value -cmatch '^\\[a-f0-9]{64}$'){$value=$value.Substring(1)}
    if($value -cnotmatch '^[a-f0-9]{64}$'){throw 'Malformed developer activation digest'}
    return $value
}
function Accepts-OneShotEvidence($record,$authority) {
    if($record.desktop_version -ne '26.1013.4894.0' -or $record.product_version -ne '0.12.1' -or
       $record.binary_sha256 -ne $authority.binary_sha256 -or -not $record.app_preserved -or -not $record.fixture_task_removed -or
       $record.phase -ne 'passed-qualified-one-shot' -or $record.qualification_method -ne 'live-stop-primitive-plus-one-shot-regression' -or
       $record.resume_lock -ne $false -or $record.notice_button_count -ne 1 -or $record.new_live_stop_requests_sent -ne 0 -or
       $record.original_live_aggregate_passed -ne $false -or $record.readonly_confirmation_count -ne 4){return $false}
    $sourceRoot=[IO.Path]::GetFullPath([string]$authority.source_root)
    $proofs=@{}
    $names=@('session-stop-component-qualification','session-stop-one-shot-code','session-stop-one-shot-checks','usage-stop-notice-only-windows')
    if(@($record.evidence).Count -ne $names.Count){return $false}
    foreach($name in $names) {
        $relative='tests/results/'+$name+'-0.12.1.json'
        $items=@($record.evidence|Where-Object path -eq $relative)
        $path=Join-Path $sourceRoot $relative
        if($items.Count -ne 1 -or (Hash-File $path) -ne $items[0].sha256){return $false}
        $proofs[$name]=[IO.File]::ReadAllText($path)|ConvertFrom-Json
    }
    $parent=$proofs['session-stop-component-qualification']
    if($parent.phase -ne 'passed-qualified-components' -or $parent.live_native_turn_count -ne 4 -or
       $parent.readonly_confirmation_count -ne 4 -or $parent.original_live_aggregate_passed -ne $false){return $false}
    foreach($item in $parent.evidence){if((Hash-File (Join-Path $sourceRoot $item.path)) -ne $item.sha256){return $false}}
    $code=$proofs['session-stop-one-shot-code'];$checks=$proofs['session-stop-one-shot-checks'];$notice=$proofs['usage-stop-notice-only-windows']
    if($code.previous_binary_sha256 -ne $parent.binary_sha256 -or $code.candidate_binary_sha256 -ne $record.binary_sha256 -or
       -not $code.native_stop_primitive_unchanged -or -not $code.identity_unchanged -or -not $code.candidate_scripts_and_logo_embedded -or
       $code.native_request_sha256 -notmatch '^[a-f0-9]{64}$' -or $code.native_request_sha256 -eq ('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855')){return $false}
    $parts=@('crates/hive-cli/src/usage_monitor.rs','crates/hive-cli/src/usage_monitor/desktop.rs','crates/hive-cli/src/usage_monitor/state.rs',
        'crates/hive-cli/src/usage.rs','crates/hive-cli/src/usage/codex_native.rs','crates/hive-cli/src/usage_control.rs',
        'crates/hive-cli/src/usage_monitor/platform.rs','crates/hive-cli/src/usage_monitor/desktop.ps1','crates/hive-cli/src/usage_monitor/windows.ps1','crates/hive-cli/src/usage_monitor/notice.ps1',
        'harness/plugins/aigent-hive/assets/hive-logo-plugin.png','harness/skills/usage-guard/SKILL.md','harness/skills/usage-guard/references/control.md',
        'harness/plugins/aigent-hive/skills/usage-guard/SKILL.md','harness/plugins/aigent-hive/skills/usage-guard/references/control.md',
        'harness/template/.agents/skills/usage-guard/SKILL.md','harness/template/.agents/skills/usage-guard/references/control.md',
        'harness/template/.claude/skills/usage-guard/SKILL.md','harness/template/.claude/skills/usage-guard/references/control.md','harness/template/.hive/config/active-skills.yml.jinja')
    if(@($record.source_hashes.PSObject.Properties).Count -ne $parts.Count){return $false}
    foreach($part in $parts) {
        $path=Join-Path $sourceRoot $part
        if($part.EndsWith('.png')){$digest=Hash-File $path}else{
            $text=[IO.File]::ReadAllText($path,[Text.Encoding]::UTF8).Replace("`r`n","`n")
            $sha=[Security.Cryptography.SHA256]::Create()
            try{$digest=[BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($text))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
        }
        if($record.source_hashes.$part -ne $digest -or $code.source_hashes.$part -ne $digest){return $false}
    }
    $required=@('rustfmt','cli','clippy','build','windows','notice','preflight','display','delivery')
    foreach($name in $required){$found=@($checks.checks|Where-Object name -eq $name);if($found.Count -ne 1 -or $found[0].exit -ne 0){return $false}}
    if($notice.entrypoint.phase -ne 'passed-display-only-entrypoint' -or $notice.entrypoint.shown_ms -gt 5000 -or
       -not $notice.entrypoint.incident_unchanged -or -not $notice.entrypoint.original_codex_app_preserved -or
       -not $notice.entrypoint.closed_by_own_button -or $notice.entrypoint.close_only_window -ne $true -or $notice.button_count -ne 1 -or $notice.resume_lock -ne $false -or
       $notice.entrypoint.notice_sha256 -ne $record.source_hashes.'crates/hive-cli/src/usage_monitor/notice.ps1'){return $false}
    return $true
}

function Accepts-StopEvidence($record,$authority) {
    if($record.phase -eq 'passed-qualified-one-shot'){return (Accepts-OneShotEvidence $record $authority)}
    if($record.desktop_version -ne '26.1013.4894.0' -or $record.binary_sha256 -ne $authority.binary_sha256 -or
       -not $record.app_preserved -or -not $record.fixture_task_removed){return $false}
    if($record.phase -eq 'passed-local-session-interruption') {
        return ($record.incident.confirmed_stop -and $record.incident.confirmed_stop.targeted_turns -ge 1 -and
            $record.stop_reason_visible -and $record.subjects_interrupted.verified -and @($record.subjects_interrupted.subjects).Count -ge 3 -and
            @($record.subjects_interrupted.subjects|Where-Object tool_free_required -eq $true).Count -ge 1 -and
            $record.no_automatic_resume -and $record.acknowledgement_does_not_resume -and $record.sensor_gap.reason_visible -and
            $record.sensor_gap.gap_seconds -ge 60 -and $record.sensor_gap.gap_seconds -le 70 -and $record.sensor_gap_acknowledgement.acknowledged)
    }
    if($record.phase -ne 'passed-qualified-components' -or $record.qualification_method -ne 'live-native-stop-plus-current-readonly-confirmation' -or
       $record.product_version -ne '0.12.1' -or $record.original_live_aggregate_passed -ne $false -or
       $record.original_live_trial_phase -ne 'failed' -or $record.new_live_stop_requests_sent -ne 0){return $false}
    # Reuse live actuation only with exact source, binary and independent confirmation evidence.
    $sourceRoot=[IO.Path]::GetFullPath([string]$authority.source_root).TrimEnd('\')
    $names=@('session-stop-native-subjects','session-stop-readonly-reconciliation','session-stop-readonly-confirmation',
        'session-stop-component-code','session-stop-sensor-failure','session-stop-acknowledgement','usage-stop-notice-windows','session-adapter-latch-windows')
    if(@($record.evidence).Count -ne $names.Count){return $false}
    $data=@{}
    foreach($name in $names) {
        $relative='tests/results/'+$name+'-0.12.1.json'
        $items=@($record.evidence|Where-Object path -eq $relative)
        if($items.Count -ne 1){return $false}
        $path=Join-Path $sourceRoot $relative
        if((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint){return $false}
        if((Hash-File $path) -ne $items[0].sha256){return $false}
        $data[$name]=[IO.File]::ReadAllText($path)|ConvertFrom-Json
    }
    $subjects=$data['session-stop-native-subjects'];$native=$data['session-stop-readonly-reconciliation'];$read=$data['session-stop-readonly-confirmation']
    if(-not $subjects.before.verified -or -not $subjects.after.verified -or @($subjects.after.subjects).Count -lt 3 -or
       @($subjects.after.subjects|Where-Object {$_.role -eq 'reasoning' -and $_.tool_free_required -and $_.tool_free_after_capture}).Count -lt 1 -or
       -not $native.all_reconstructed_active_turns_interrupted_within_five_seconds -or $native.original_automatic_confirmation_passed -ne $false){return $false}
    $nativeKeys=@($native.subjects|ForEach-Object {$_.thread_digest+':'+$_.turn_digest}|Sort-Object)
    $readKeys=@($read.confirmed_targets|Where-Object {$_.fresh -and $_.status -eq 'interrupted'}|ForEach-Object {$_.thread_digest+':'+$_.turn_digest}|Sort-Object)
    if($nativeKeys.Count -lt 3 -or ($nativeKeys -join '|') -ne ($readKeys -join '|') -or $read.expected_targets -ne $nativeKeys.Count -or
       $read.expected_confirmed -ne $nativeKeys.Count -or $read.ready -ne $read.known -or $read.native_stop_requests_sent -ne 0 -or
       -not $read.read_only -or $read.elapsed_ms -lt 1 -or $read.elapsed_ms -gt 5000 -or
       @($native.subjects|Where-Object {$_.elapsed_from_detected_second_ms -lt 0 -or $_.elapsed_from_detected_second_ms -gt 2000}).Count -gt 0){return $false}
    $bound=($native.subjects|Measure-Object elapsed_from_detected_second_ms -Maximum).Maximum
    if($record.live_native_turn_count -ne $nativeKeys.Count -or $record.readonly_confirmation_count -ne $read.expected_confirmed -or
       $record.readonly_confirmation_ms -ne $read.elapsed_ms -or $record.live_interruption_upper_bound_ms -ne $bound -or
       $record.request_upper_bound_from_native_abort_ms -ne $bound){return $false}
    $code=$data['session-stop-component-code'];$empty='e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
    if(-not $code.native_stop_and_identity_unchanged -or -not $code.all_checked_parts_match -or
       $code.candidate_binary_sha256 -ne $record.binary_sha256 -or $code.old_desktop.stop_dispatch -eq $empty -or
       $code.old_desktop.stop_dispatch -ne $code.current_desktop.stop_dispatch -or
       $code.old_desktop.outside_verification -ne $code.current_desktop.outside_verification){return $false}
    $immutable=@('crates/hive-cli/src/usage_monitor.rs','crates/hive-cli/src/usage_monitor/desktop.rs','crates/hive-cli/src/usage_monitor/state.rs',
        'crates/hive-cli/src/usage.rs','crates/hive-cli/src/usage/codex_native.rs','platform::command','platform::call',
        'previous-desktop-embedded','candidate-desktop-embedded','candidate-notice-embedded')
    foreach($part in $immutable){if($code.immutable_rust_and_embedded_parts.$part -ne $true){return $false}}
    foreach($part in @('inspect','unregister','default','register','App-Roots','Emit','start')) {
        if($code.old_tasks.$part -notmatch '^[a-f0-9]{64}$' -or $code.old_tasks.$part -ne $code.current_tasks.$part){return $false}
    }
    $parts=@('usage_monitor.rs','usage_monitor/desktop.rs','usage_monitor/state.rs','usage.rs','usage/codex_native.rs',
        'usage_monitor/platform.rs','usage_monitor/desktop.ps1','usage_monitor/windows.ps1','usage_monitor/notice.ps1')
    if(@($record.source_hashes.PSObject.Properties).Count -ne $parts.Count){return $false}
    foreach($part in $parts) {
        $relative='crates/hive-cli/src/'+$part
        $text=[IO.File]::ReadAllText((Join-Path $sourceRoot $relative),[Text.Encoding]::UTF8).Replace("`r`n","`n")
        $sha=[Security.Cryptography.SHA256]::Create()
        try {$digest=[BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($text))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
        if($record.source_hashes.$relative -ne $digest -or $code.source_hashes.$relative -ne $digest){return $false}
    }
    $gap=$data['session-stop-sensor-failure'];$ack=$data['session-stop-acknowledgement'];$notice=$data['usage-stop-notice-windows'];$release=$data['session-adapter-latch-windows']
    if($gap.phase -ne 'passed-readonly-sensor-failure-and-recovery' -or $gap.gap_seconds -lt 60 -or $gap.failed_read_count -lt 3 -or
       $gap.initial_normal_code -ne 'hive.usage-native-available' -or $gap.recovery_code -ne 'hive.usage-native-available' -or
       @($gap.failed_reads|Where-Object exit -eq 0).Count -gt 0 -or $gap.session_interruption -ne $false -or
       -not $ack.wrong_incident_refused -or -not $ack.acknowledged -or $ack.auto_resume -ne $false -or
       -not $ack.fresh_usage_verified -or -not $ack.fixture_incident_cleared -or -not $ack.same_current_turns_and_statuses -or
       $notice.entrypoint.phase -ne 'passed-display-only-entrypoint' -or $notice.entrypoint.shown_ms -gt 5000 -or
       -not $notice.entrypoint.closed_by_own_button -or -not $notice.entrypoint.incident_unchanged -or
       -not $notice.entrypoint.original_codex_app_preserved -or $notice.entrypoint.notice_sha256 -ne $code.notice_source_sha256 -or
       -not $release.subscriptions_released -or -not $release.app_preserved){return $false}
    return $true
}
if(-not (Accepts-StopEvidence $acceptance $approval)){throw 'Session-stop acceptance and fixture cleanup must pass before installation'}
$source=[IO.Path]::GetFullPath($approval.source_root).TrimEnd('\')
$userRoot=[IO.Path]::GetFullPath($approval.user_root).TrimEnd('\')
$binary=[IO.Path]::GetFullPath($approval.binary)
$native=[IO.Path]::GetFullPath($approval.native_binary)
$report=[IO.Path]::GetFullPath($approval.report)
if (-not $binary.StartsWith($source+'\target\debug\',[StringComparison]::OrdinalIgnoreCase) -or
    -not $report.StartsWith($source+'\.agents\work\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Deployment paths differ from approved source ownership' }
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
    $raw=$null
    for($attempt=0;$attempt -lt 3;$attempt++) {
        $raw=& $native @arguments
        if($LASTEXITCODE -eq 0 -or $arguments[0] -ne 'usage' -or $arguments[2] -ne 'status'){break}
        Start-Sleep -Milliseconds 25
    }
    if($LASTEXITCODE -ne 0){throw ('Hive step failed: '+$arguments[0]+' '+$arguments[1])}
    return ($raw | ConvertFrom-Json)
}
$result=@{schema_version=1;phase='started';product_version='0.12.1';package_identity='0.12.1-dev';installed_to_real_user_root=$true;started_at=[DateTime]::UtcNow.ToString('o')}
$activated=$false
function Record { [IO.File]::WriteAllText($report,($result|ConvertTo-Json -Depth 12),$utf8) }
try {
    Record
    $candidateVersion=(& $binary --version | Out-String).Trim()
    if($LASTEXITCODE -ne 0 -or $candidateVersion -notlike 'AIgent Hive v0.12.1-dev*'){throw 'Candidate developer product identity differs; active installation unchanged'}
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
            $activeDigest=Parse-DeveloperDigest ([IO.File]::ReadAllText((Join-Path $activationState 'developer-sha256')))
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
    $activated=$true
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
        if($acceptance.phase -eq 'passed-qualified-one-shot' -and $status.data.resume_lock -ne $false){throw 'Resume lock is not removed'}
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
        $null=$notice.Popup('0.12.1 로컬 설치와 상시 감시 등록을 완료했습니다. 중단 후에는 Codex에서 직접 재개하거나 새 메시지를 보낼 수 있습니다.',10,'Aigent Hive',64)
    } catch { } # The persisted receipt remains authoritative if desktop notifications are unavailable.
} catch {
    $result.error=$_.Exception.Message;$result.phase='failed'
    if($activated) {
        try {
            $null=& $binary usage monitor disable --user-root $userRoot --output json
            if($LASTEXITCODE -ne 0){throw 'Owned monitor rollback failed'}
            & $approval.git_bash --noprofile --norc (Join-Path $source 'scripts\invoke-dev-install.sh') rollback (Split-Path $native) (Join-Path $source 'scripts\dev-install.sh') (Join-Path $userRoot '.hive\dev-install') $approval.python_directory
            if($LASTEXITCODE -ne 0 -or (Hash-File $native) -ne $approval.native_sha256){throw 'Managed native rollback did not restore the original binary'}
            $result.native_rollback_verified=$true
        } catch {$result.rollback_error=$_.Exception.Message}
    }
    $result.completed_at=[DateTime]::UtcNow.ToString('o');Record
    try {
        $notice=New-Object -ComObject WScript.Shell
        $null=$notice.Popup(('작업 중단 검증 뒤 설치를 완료하지 못했습니다. 결과 확인: '+$report),10,'Aigent Hive',48)
    } catch { }
    exit 1
}
