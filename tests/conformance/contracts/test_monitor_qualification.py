"""Offline installation evidence gates. Never installs or sends desktop control."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]


@unittest.skipUnless(os.name == "nt", "Windows PowerShell installation evidence gate")
class QualificationEvidence(unittest.TestCase):
    def evaluate(self, modification: str = "") -> bool:
        helper = str(ROOT / "scripts/complete-usage-monitor-install.ps1").replace("'", "''")
        report = str(ROOT / "tests/results/session-stop-component-qualification-0.12.1.json").replace("'", "''")
        source = r"""
$ErrorActionPreference='Stop'
$tokens=$null;$errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile('HELPER',[ref]$tokens,[ref]$errors)
if($errors){throw 'Installation helper parse failed'}
$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)|ForEach-Object {Invoke-Expression $_.Extent.Text}
$record=[IO.File]::ReadAllText('REPORT')|ConvertFrom-Json
$authority=[pscustomobject]@{source_root='ROOT';binary_sha256=$record.binary_sha256}
MODIFICATION
@{accepted=[bool](Accepts-StopEvidence $record $authority)}|ConvertTo-Json -Compress
""".replace("'HELPER'", "'" + helper + "'").replace("'REPORT'", "'" + report + "'").replace("'ROOT'", "'" + str(ROOT).replace("'", "''") + "'").replace("MODIFICATION", modification)
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        with tempfile.TemporaryDirectory(prefix="hive-stop-proof-") as directory:
            path = Path(directory) / "proof.ps1"
            path.write_text(source, encoding="utf-8-sig")
            result = subprocess.run([str(executable), "-NoProfile", "-NonInteractive", "-File", str(path)], capture_output=True, timeout=20)
        self.assertEqual(result.returncode, 0, result.stderr.decode("utf-8", "replace")[-2000:])
        return json.loads(result.stdout.decode("utf-8-sig"))["accepted"]

    def test_complete_bound_component_evidence_is_accepted(self):
        self.assertTrue(self.evaluate())

    def test_evidence_file_digest_cannot_be_replaced(self):
        self.assertFalse(self.evaluate("$record.evidence[0].sha256='0'*64"))

    def test_a_missing_fourth_confirmation_cannot_be_hidden(self):
        self.assertFalse(self.evaluate("$record.readonly_confirmation_count=3"))

    def test_source_change_requires_requalification(self):
        self.assertFalse(self.evaluate("$record.source_hashes.'crates/hive-cli/src/usage_monitor/desktop.ps1'='0'*64"))

    def test_live_scope_failure_is_not_relabelled_as_passed(self):
        self.assertFalse(self.evaluate("$record.original_live_aggregate_passed=$true"))

    def test_unknown_qualification_method_is_rejected(self):
        self.assertFalse(self.evaluate("$record.qualification_method='unchecked'"))


if __name__ == "__main__":
    unittest.main()
