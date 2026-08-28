"""Offline checks for the separately consented real-package vector qualification."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/qualify-vector-runtime.py"
spec = importlib.util.spec_from_file_location("vector_qualification", SCRIPT)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class VectorRuntimeAcceptance(unittest.TestCase):
    def test_external_timeout_cleanup_closes_real_parent_and_descendant_pipes(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            ready = Path(work) / "child.json"
            code = ("import subprocess,sys,time,pathlib,json\n"
                    "child=subprocess.Popen([sys.executable,'-I','-S','-B','-c','import time; time.sleep(60)'])\n"
                    "pathlib.Path(" + repr(str(ready)) + ").write_text(json.dumps({'pid':child.pid}))\ntime.sleep(60)")
            parent = subprocess.Popen([sys.executable, "-I", "-S", "-B", "-c", code],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                deadline = time.monotonic() + 10
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(.05)
                self.assertTrue(ready.exists())
                runner.stop_cli_tree(parent)
                parent.communicate(timeout=5)
                self.assertIsNotNone(parent.returncode)
            finally:
                if parent.poll() is None:
                    runner.stop_cli_tree(parent)

    def test_optimized_python_cannot_report_acceptance(self):
        result = subprocess.run([sys.executable, "-O", str(SCRIPT), "--hive", sys.executable, "--authorize-install"],
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2)
        self.assertIn("optimized Python", result.stderr)

    def test_missing_consent_never_creates_a_fixture_or_runs_a_cli(self):
        with patch.object(sys, "argv", [str(SCRIPT), "--hive", "not-a-binary"]), \
             patch.object(runner.tempfile, "mkdtemp") as mkdir, \
             patch.object(runner.subprocess, "run") as execute:
            with self.assertRaises(SystemExit) as failure:
                runner.main()
            self.assertEqual(failure.exception.code, 2)
            mkdir.assert_not_called()
            execute.assert_not_called()

    def test_disposable_fixture_has_eight_real_canonical_pages(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            paths = runner.consumer_fixture(Path(work))
            self.assertEqual(len(paths), 9)
            pages = list((Path(work) / ".hive/knowledge/Wiki").glob("*.md"))
            self.assertEqual(len(pages), 8)
            ids = set()
            for page in pages:
                metadata = json.loads(page.read_text("utf-8").split("---\n")[1])
                ids.add(metadata["id"])
                self.assertEqual(metadata["status"], "active")
            self.assertEqual(len(ids), 8)

    def test_failed_command_retains_exact_result_and_elapsed_time(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            qualification = runner.Qualification(Path(sys.executable), Path(work))
            process = Mock(returncode=1)
            process.communicate.return_value = ('{"status":"error","message":"failure"}', "")
            with patch.object(runner.subprocess, "Popen", return_value=process):
                with self.assertRaises(RuntimeError):
                    qualification.call("knowledge", "vector", "status")
            saved = json.loads((Path(work) / "receipt.json").read_bytes())
            self.assertEqual(saved["calls"][0]["exit_code"], 1)
            self.assertGreaterEqual(saved["calls"][0]["elapsed_seconds"], 0)
            self.assertEqual(saved["calls"][0]["result"]["message"], "failure")

    def test_timeout_does_not_retry_a_mutation_or_claim_success(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            qualification = runner.Qualification(Path(sys.executable), Path(work))
            process = Mock()
            process.communicate.side_effect = [subprocess.TimeoutExpired("hive", 1000), ("", "")]
            with patch.object(runner.subprocess, "Popen", return_value=process) as execute, \
                 patch.object(runner, "stop_cli_tree") as cleanup:
                with self.assertRaises(RuntimeError):
                    qualification.call("knowledge", "vector", "enable")
                execute.assert_called_once()
                cleanup.assert_called_once_with(process)
            saved = json.loads((Path(work) / "receipt.json").read_bytes())
            self.assertNotEqual(saved["status"], "passed")
            self.assertIn("timeout", saved["calls"][0]["error"])

    def test_snapshot_detects_canonical_changes(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            page = Path(work) / "document.md"
            page.write_text("before", encoding="utf-8")
            before = runner.snapshot([page])
            page.write_text("after", encoding="utf-8")
            self.assertNotEqual(before, runner.snapshot([page]))

    def test_native_and_public_acceptance_use_the_real_opt_in_runner(self):
        import yaml
        native = yaml.safe_load((ROOT / ".github/workflows/release-runtime.yml").read_text("utf-8"))
        job = native["jobs"]["vector"]
        self.assertIn("inputs.vector_only", job["if"])
        self.assertEqual({item["target"] for item in job["strategy"]["matrix"]["include"]},
                         {"x86_64-pc-windows-msvc", "aarch64-apple-darwin", "x86_64-unknown-linux-musl"})
        public = yaml.safe_load((ROOT / ".github/workflows/public-test-acceptance.yml").read_text("utf-8"))
        for steps in (job["steps"], public["jobs"]["korean-public-test"]["steps"]):
            commands = [step["run"] for step in steps if "run" in step]
            self.assertTrue(any("qualify-vector-runtime.py" in command and "--authorize-install" in command for command in commands))


if __name__ == "__main__":
    unittest.main()
