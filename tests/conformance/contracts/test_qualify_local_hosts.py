"""The local host collector reports command inputs without assigning a verdict."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/qualify-local-hosts.py"


class QualifyLocalHostsTests(unittest.TestCase):
    def setUp(self):
        self.work = ROOT / "tests/work"
        self.work.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="qualify-local-hosts-test-", dir=self.work)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.target = self.root / "project"
        self.user = self.root / "user"
        self.target.mkdir()
        self.user.mkdir()

    def test_fake_hive_inputs_include_probe_results_without_verdict(self):
        skill = self.user / ".agents/skills/sample-skill/SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("name: sample-skill\n", encoding="utf-8")
        fake = self.root / "hive_fake.py"
        calls = self.root / "calls.jsonl"
        fake.write_text(
            "import json, os, sys\n"
            "with open(os.environ['FAKE_HIVE_CALLS'], 'a', encoding='utf-8') as f: "
            "f.write(json.dumps(sys.argv[1:]) + '\\n')\n"
            "print('AIgent Hive v0.16.0' if sys.argv[1:] == ['--version'] else "
            "json.dumps({'status':'observed','args':sys.argv[1:]}))\n",
            encoding="utf-8",
        )
        environment = os.environ.copy()
        environment["FAKE_HIVE_CALLS"] = str(calls)
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--target", str(self.target), "--user-root", str(self.user),
             "--host", "codex", "--hive", f'"{sys.executable}" "{fake}"'],
            capture_output=True, text=True, env=environment, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["hive_available"])
        self.assertNotIn("verdict", report)
        self.assertEqual(report["inputs"]["installed_version"]["stdout"], "AIgent Hive v0.16.0\n")
        self.assertEqual(report["inputs"]["user_install_validation"]["returncode"], 0)
        self.assertEqual(report["inputs"]["usage_enforcement"]["returncode"], 0)
        self.assertEqual(report["inputs"]["hook_preview"]["returncode"], 0)
        self.assertEqual(report["inputs"]["skill_projection_names"]["codex"], ["sample-skill"])
        self.assertFalse(report["inputs"]["flow_start_preview"]["supported"])
        observed = [json.loads(line) for line in calls.read_text(encoding="utf-8").splitlines()]
        self.assertEqual(observed[0], ["--version"])
        usage_call = next(call for call in observed if call[:2] == ["usage", "enforce"])
        self.assertIn(str(self.work.resolve()), usage_call[usage_call.index("--user-root") + 1])

    def test_missing_hive_is_an_input_and_not_a_failed_verdict(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--target", str(self.target), "--user-root", str(self.user),
             "--hive", str(self.root / "missing-hive")],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertFalse(report["hive_available"])
        self.assertNotIn("verdict", report)


if __name__ == "__main__":
    unittest.main()
