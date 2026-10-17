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
    @classmethod
    def setUpClass(cls):
        cls.fixture = tempfile.TemporaryDirectory(prefix="hive-historical-stop-proof-")
        cls.source = Path(cls.fixture.name)
        report = "tests/results/session-stop-one-shot-qualification-0.12.1.json"
        pending = [report]
        copied = set()
        while pending:
            relative = pending.pop()
            if relative in copied:
                continue
            if not relative.startswith("tests/results/") or ".." in Path(relative).parts:
                raise AssertionError("Unexpected historical evidence path")
            payload = (ROOT / relative).read_bytes()
            destination = cls.source / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(payload)
            copied.add(relative)
            record = json.loads(payload)
            pending.extend(item["path"] for item in record.get("evidence", []) if "path" in item)
        code = json.loads((ROOT / "tests/results/session-stop-one-shot-code-0.12.1.json").read_bytes())
        for relative in code["source_hashes"]:
            destination = cls.source / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(subprocess.check_output(
                ["git", "show", "da2797509ea7e0ef94783877e3c2cb69c6bfd490:" + relative], cwd=ROOT
            ))

    @classmethod
    def tearDownClass(cls):
        cls.fixture.cleanup()

    def evaluate(self, modification: str = "", *, current_source: bool = False) -> bool:
        helper = str(ROOT / "scripts/complete-usage-monitor-install.ps1").replace("'", "''")
        report = str(self.source / "tests/results/session-stop-one-shot-qualification-0.12.1.json").replace("'", "''")
        source_root = ROOT if current_source else self.source
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
""".replace("'HELPER'", "'" + helper + "'").replace("'REPORT'", "'" + report + "'").replace("'ROOT'", "'" + str(source_root).replace("'", "''") + "'").replace("MODIFICATION", modification)
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        with tempfile.TemporaryDirectory(prefix="hive-stop-proof-") as directory:
            path = Path(directory) / "proof.ps1"
            path.write_text(source, encoding="utf-8-sig")
            result = subprocess.run([str(executable), "-NoProfile", "-NonInteractive", "-File", str(path)], capture_output=True, timeout=20)
        self.assertEqual(result.returncode, 0, result.stderr.decode("utf-8", "replace")[-2000:])
        return json.loads(result.stdout.decode("utf-8-sig"))["accepted"]

    def test_complete_bound_component_evidence_is_accepted(self):
        self.assertTrue(self.evaluate())

    def test_historical_approval_does_not_cover_current_release_sources(self):
        self.assertFalse(self.evaluate(current_source=True))

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

    def test_a_resume_lock_cannot_be_reintroduced(self):
        self.assertFalse(self.evaluate("$record.resume_lock=$true"))

    def test_additional_notice_buttons_are_rejected(self):
        self.assertFalse(self.evaluate("$record.notice_button_count=2"))


if __name__ == "__main__":
    unittest.main()
