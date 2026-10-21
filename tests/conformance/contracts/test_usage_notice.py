"""Notice copy and state tests. No monitor, app interruption, or installation."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
NOTICE = ROOT / "crates/hive-cli/src/usage_monitor/notice.ps1"


@unittest.skipUnless(os.name == "nt", "Windows PowerShell notice state")
class UsageNotice(unittest.TestCase):
    def view(self, case: str) -> dict:
        path = str(NOTICE).replace("'", "''")
        source = r"""
$ErrorActionPreference='Stop'
$tokens=$null;$errors=$null
$text=[IO.File]::ReadAllText('NOTICE',[Text.Encoding]::UTF8)
$tree=[Management.Automation.Language.Parser]::ParseInput($text,[ref]$tokens,[ref]$errors)
if($errors){throw 'Notice parser failed'}
$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)|ForEach-Object {Invoke-Expression $_.Extent.Text}
$context=@{incident='fixture';reason='usage-reset';detected_at=100L;is_test=$false;preview=$false}
$state=[pscustomobject]@{app_alive=$true;control_connected=$true;active_turns=0;unknown_sessions=0;incident=[pscustomobject]@{
 digest='fixture';reason='usage-reset';detected_at=100L;stop_confirmed=$true;stop_error=$null;targeted_turns=4;confirmed_turns=4
}}
CASE
""".replace("'NOTICE'", "'" + path + "'").replace("CASE", case)
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        with tempfile.TemporaryDirectory(prefix="hive-notice-test-") as directory:
            script = Path(directory) / "test.ps1"
            script.write_text(source, encoding="utf-8-sig")
            result = subprocess.run([str(executable), "-NoLogo", "-NoProfile", "-NonInteractive", "-Sta", "-File", str(script)],
                                    capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr.decode("utf-8", "replace")[-2000:])
        return json.loads(result.stdout.decode("utf-8-sig"))

    def test_production_reset_names_hive_and_omits_test_and_shell_details(self):
        view = self.view("Get-NoticeView $state $context $true | ConvertTo-Json -Compress")
        self.assertIn("Aigent Hive", view["Explanation"])
        self.assertIn("사용량 초기화", view["Explanation"])
        self.assertIn("멈췄습니다", view["Title"])
        self.assertEqual(view["Status"], "작업 중단 확인")
        self.assertEqual(view["Banner"], "")
        self.assertNotIn("hive usage", str(view))
        self.assertIn("자동으로", view["Automatic"])
        self.assertIn("일반 외부 명령은 계속 실행될 수 있습니다", view["Automatic"])

    def test_sensor_failure_is_not_presented_as_a_reset(self):
        view = self.view("$state.incident.reason='sensor-unresponsive';Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertIn("60초", view["Reason"])
        self.assertIn("60초", view["Explanation"])
        self.assertNotIn("초기화", view["Reason"])

    def test_threshold_notice_names_the_actual_floor_window_and_stop_confirmation(self):
        view = self.view("$state.incident.reason='usage-threshold';"
                         "$state.incident|Add-Member NoteProperty threshold_details @(@{remaining_percent=8.0;threshold_percent=10;window='weekly'});"
                         "Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertIn('8%', view['Explanation'])
        self.assertIn('10%', view['Explanation'])
        self.assertIn('주간', view['Explanation'])
        self.assertIn('Aigent Hive', view['Explanation'])
        self.assertEqual(view['Status'], '작업 중단 확인')
        self.assertNotIn('오류', view['Explanation'])

    def test_threshold_request_does_not_claim_an_unconfirmed_stop(self):
        view = self.view("$state.incident.reason='usage-threshold';$state.incident.stop_confirmed=$false;"
                         "$state.incident|Add-Member NoteProperty threshold_details @(@{remaining_percent=8.0;threshold_percent=10;window='session'});"
                         "Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertIn('요청했습니다', view['Title'])
        self.assertNotIn('확인했습니다', view['Explanation'])

    def test_missing_threshold_details_do_not_invent_zero_usage_or_floor(self):
        view = self.view("$context.reason='usage-threshold';Get-NoticeView $null $context $false $true|ConvertTo-Json -Compress")
        self.assertNotIn('0%', view['Explanation'])
        self.assertEqual(view['Status'], '상태를 확인할 수 없습니다')

    def test_windows_powershell_reads_verbatim_paths_with_atomic_replacement_sharing(self):
        result = self.view(r"""
$path=[IO.Path]::Combine($PSScriptRoot,'verbatim-state.json')
[IO.File]::WriteAllText($path,'{"value":99}',[Text.UTF8Encoding]::new($false))
$extended='\\?\'+[IO.Path]::GetFullPath($path)
$normal=Read-NoticeState $path
$verbatim=Read-NoticeState $extended
@{normal=$normal.value;verbatim=$verbatim.value}|ConvertTo-Json -Compress
""")
        self.assertEqual(result, {"normal": 99, "verbatim": 99})

    def test_sensor_notice_keeps_the_specific_original_failure_after_recovery(self):
        view = self.view("$state.incident.reason='sensor-unresponsive';"
                         "$state.incident|Add-Member NoteProperty query_error 'observation-timeout';"
                         "$null=Get-NoticeView $state $context $true;$state.incident=$null;"
                         "Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertIn('60초', view['Explanation'])
        self.assertIn('제한 시간', view['Explanation'])

    def test_official_logo_is_shared_by_the_title_and_body(self):
        path = str(ROOT / "harness/plugins/aigent-hive/assets/hive-logo-plugin.png").replace("'", "''")
        result = self.view("$logo=[Convert]::ToBase64String([IO.File]::ReadAllBytes('" + path + "'));"
                           "$window=New-NoticeWindow $context 'C:\\Users\\fixture' $logo;"
                           "@{same=[object]::ReferenceEquals($window.Icon,$window.FindName('Logo').Source);"
                           "width=$window.Icon.PixelWidth;height=$window.Icon.PixelHeight}|ConvertTo-Json -Compress")
        self.assertTrue(result["same"])
        self.assertEqual((result["width"], result["height"]), (512, 512))

    def test_partial_confirmation_is_not_success(self):
        view = self.view("$state.incident.stop_confirmed=$false;$state.incident.stop_error='session-scope-unverified';$state.incident.confirmed_turns=3;$state.unknown_sessions=1;Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertEqual(view["Tone"], "warning")
        self.assertIn("요청했습니다", view["Title"])
        self.assertIn("확인하지 못했습니다", view["StatusDetail"])
        self.assertIn("확인  3개", view["Details"])

    def test_missing_record_does_not_claim_unlock_or_change_the_original_cause(self):
        view = self.view("$null=Get-NoticeView $state $context $true;$state.incident=$null;Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertEqual(view["Reason"], "사용량 초기화 감지")
        self.assertEqual(view["Status"], "이전 중단 안내")
        self.assertIn("Codex", view["Next"])
        self.assertIn("자동으로", view["Automatic"])

    def test_read_failure_does_not_look_like_acknowledgement(self):
        view = self.view("Get-NoticeView $null $context $false $true|ConvertTo-Json -Compress")
        self.assertEqual(view["Status"], "상태를 확인할 수 없습니다")
        self.assertNotEqual(view["Tone"], "success")

    def test_new_incident_does_not_reuse_old_confirmation(self):
        view = self.view("$state.incident.digest='other';Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertEqual(view["Status"], "새 보호 사건 발생")
        self.assertNotEqual(view["Tone"], "success")

    def test_preview_and_synthetic_reset_are_distinct(self):
        values = self.view("$context.is_test=$true;$test=Get-NoticeView $state $context $true;$context.preview=$true;$preview=Get-NoticeView $state $context $true;@{test=$test.Banner;preview=$preview.Banner}|ConvertTo-Json -Compress")
        self.assertIn("격리 시험", values["test"])
        self.assertIn("작업 중단 없음", values["preview"])
        self.assertNotIn("격리 시험", values["preview"])

    def test_missing_app_never_claims_preservation_or_confirmed_stop(self):
        view = self.view("$state.app_alive=$false;Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertIn("확인", view["App"])
        self.assertNotEqual(view["Status"], "작업 중단 확인")

    def test_empty_work_is_distinct_from_confirmed_interruption(self):
        view = self.view("$state.incident.stop_confirmed=$false;$state.incident.targeted_turns=0;$state.incident.confirmed_turns=0;Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertEqual(view["Status"], "감지 당시 실행 중인 작업이 없었습니다")
        self.assertNotEqual(view["Tone"], "success")

    def test_other_guard_errors_do_not_invent_a_sixty_second_gap(self):
        view = self.view("$state.incident.reason='state-error';Get-NoticeView $state $context $true|ConvertTo-Json -Compress")
        self.assertNotIn("60초", view["Explanation"])
        self.assertNotIn("초기화", view["Explanation"])
        self.assertIn("보호 상태", view["Reason"])

    def test_separate_visible_controls_form_the_notice_confirmation(self):
        path = str(ROOT / "scripts/qualify-usage-monitor.ps1").replace("'", "''")
        case = r"""
$script=[IO.File]::ReadAllText('QUALIFIER',[Text.Encoding]::UTF8)
$tree=[Management.Automation.Language.Parser]::ParseInput($script,[ref]$tokens,[ref]$errors)
$node=$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Matches-StopReason'},$false)[0]
Invoke-Expression $node.Extent.Text
$visible=@('Aigent Hive가 사용량 초기화를 감지했습니다.','작업 중단 확인','Codex 앱은 열려 있습니다.')
$pass=Matches-StopReason $visible '사용량 초기화' $true
$partial=Matches-StopReason @('사용량 초기화','일부 중단 상태의 확인이 필요합니다','Codex 앱은 열려 있습니다.') '사용량 초기화' $true
@{separate_controls=$pass;partial_rejected=(-not $partial)}|ConvertTo-Json -Compress
""".replace("'QUALIFIER'", "'" + path + "'")
        result = self.view(case)
        self.assertTrue(result["separate_controls"])
        self.assertTrue(result["partial_rejected"])

    def test_notice_has_only_close_and_no_resume_mutation_or_command(self):
        result = self.view("$logo=[Convert]::ToBase64String([IO.File]::ReadAllBytes('" + str(ROOT / "harness/plugins/aigent-hive/assets/hive-logo-plugin.png").replace("'", "''") + "'));"
                           "$window=New-NoticeWindow $context 'C:\\Users\\fixture' $logo;"
                           "$nodes=[Collections.Generic.List[object]]::new();"
                           "function Visit($node){if($node -isnot [Windows.DependencyObject]){return};if($node -is [Windows.Controls.Button]){$nodes.Add($node.Content)};"
                           "foreach($child in [Windows.LogicalTreeHelper]::GetChildren($node)){Visit $child}};"
                           "$window.ApplyTemplate()|Out-Null;Visit $window;"
                           "@{buttons=@($nodes.ToArray());resize=$window.ResizeMode.ToString();next=(Get-NoticeView $state $context $true).Next}|ConvertTo-Json -Compress")
        self.assertEqual(result["buttons"], ["안내 닫기"])
        self.assertEqual(result["resize"], "NoResize")
        self.assertIn("재개하거나 새 메시지", result["next"])
        source = NOTICE.read_text(encoding="utf-8")
        for removed in ("Get-ResumeCommand", "Clipboard", "CanCopy", "보호 잠금", "Start-Process", "acknowledge"):
            self.assertNotIn(removed, source)


if __name__ == "__main__":
    unittest.main()
