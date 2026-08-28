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
    def test_shared_window_results_cannot_repeat_one_scope_or_misattribute_a_change(self):
        scopes = {"user-root":"a", "other":"b"}
        data = {"complete":True, "failed":False, "scopes":[{
            "scope_id":scope, "selector":{"kind":"collection", "partition":{"collection_id":identity,"visibility":"shared"}},
            "state":"complete", "result":{"complete":True,"chunks":8,"embedded":int(identity == "user-root")},
        } for identity, scope in scopes.items()]}
        expected = {"user-root":1,"other":0}
        runner.validate_shared_window(data, scopes, expected)
        for mutation in ("duplicate", "scope", "visibility", "incomplete", "misattributed"):
            changed = json.loads(json.dumps(data))
            if mutation == "duplicate": changed["scopes"][1] = changed["scopes"][0]
            elif mutation == "scope": changed["scopes"][1]["scope_id"] = "a"
            elif mutation == "visibility": changed["scopes"][1]["selector"]["partition"]["visibility"] = "confidential"
            elif mutation == "incomplete": changed["scopes"][1]["state"] = "checkpoint"
            else:
                changed["scopes"][0]["result"]["embedded"] = 0
                changed["scopes"][1]["result"]["embedded"] = 1
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                runner.validate_shared_window(changed, scopes, expected)

    def test_shared_window_fixture_has_three_separate_shared_roots(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            work = Path(directory)
            user, identities, protected = runner.shared_batch_fixture(work)
            registry = json.loads((user / ".hive/config/collections.yml").read_bytes())
            self.assertEqual(len(identities), 3)
            self.assertEqual(len(set(identities)), 3)
            roots = [Path(row["local_locator"]) for row in registry["collections"]]
            self.assertEqual(len(set(roots)), 3)
            self.assertTrue(all(root.parent == work for root in roots))
            self.assertTrue(all(row["default_visibility"] == "shared" for row in registry["collections"]))
            self.assertEqual(sum(len(list(root.glob(".hive/knowledge/Wiki/*.md"))) for root in roots), 24)
            self.assertEqual(len(protected), 28)

    def test_source_resume_uses_fresh_only_on_the_first_bounded_slice(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            qualification = runner.Qualification(Path(sys.executable), Path(directory))
            complete = {"complete":True, "chunks":81}
            with patch.object(qualification, "call", side_effect=[{"complete":False}, {"checkpoint_available":True}, complete]) as call:
                result, observed = qualification.source_rebuild(["--target", "synthetic-source"], fresh=True)
            self.assertEqual(result, complete)
            self.assertTrue(observed)
            first, status, last = [entry.args for entry in call.call_args_list]
            self.assertIn("fresh", first)
            self.assertEqual(first[first.index("--max-seconds")+1], "1")
            self.assertEqual(status[2], "status")
            self.assertNotIn("--rebuild-mode", last)
            self.assertEqual(last[last.index("--max-seconds")+1], "10")

    def test_source_resume_reports_no_observation_and_never_retries_a_failed_mutation(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            qualification = runner.Qualification(Path(sys.executable), Path(directory))
            with patch.object(qualification, "call", return_value={"complete":True}) as call:
                _, observed = qualification.source_rebuild([])
                self.assertFalse(observed)
                call.assert_called_once()
            with patch.object(qualification, "call", side_effect=RuntimeError("failed rebuild")) as call:
                with self.assertRaisesRegex(RuntimeError, "failed rebuild"):
                    qualification.source_rebuild([])
                call.assert_called_once()
            with patch.object(qualification, "call", side_effect=lambda *args: {"complete":False} if args[2] == "rebuild" else {"checkpoint_available":True}) as call:
                with self.assertRaisesRegex(RuntimeError, "eight bounded"):
                    qualification.source_rebuild([])
                self.assertEqual(sum(entry.args[2] == "rebuild" for entry in call.call_args_list), 8)

    def test_source_fixture_loading_never_creates_bytecode_without_B(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            work = Path(directory)
            source = work / "scripts/qualify-source-graph.py"
            source.parent.mkdir()
            source.write_text("def frozen_source(repository, target): pass\n", encoding="utf-8")
            qualification = runner.Qualification(Path(sys.executable), work)
            with patch.object(runner, "ROOT", work), patch.object(sys, "dont_write_bytecode", False), \
                 patch.object(qualification, "call", side_effect=RuntimeError("stop before CLI")):
                with self.assertRaisesRegex(RuntimeError, "stop before CLI"):
                    qualification.source_vectors()
            self.assertEqual(list(source.parent.iterdir()), [source])

    def test_consumer_and_source_install_share_the_download_timeout(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            qualification = runner.Qualification(Path(sys.executable), Path(directory))
            for prefix in ("knowledge", "source-wiki"):
                process = Mock(returncode=0)
                process.communicate.return_value = ('{"data":{}}', "")
                with self.subTest(prefix=prefix), patch.object(runner.subprocess, "Popen", return_value=process):
                    qualification.call(prefix, "vector", "enable")
                process.communicate.assert_called_once_with(timeout=1000)

    def test_portable_fixture_inventory_rejects_any_derived_or_duplicate_entry(self):
        names = ["manifest.json", "manifest-sha256.txt", "data/.hive/portable/collections.json",
                 "data/.hive/portable/collections/user-root/suppression.yml",
                 *(f"data/.hive/portable/collections/user-root/Wiki/vector-example-{number}.md" for number in range(8))]
        runner.validate_fixture_bundle_entries(names)
        for extra in ("data/.hive/index/vector/index.sqlite3", "data/.hive/config/vector-state/receipt.json",
                      "data/.agents/work/vector/model.onnx", "unrelated.txt", names[0]):
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                runner.validate_fixture_bundle_entries([*names, extra])
        with self.assertRaises(ValueError):
            runner.validate_fixture_bundle_entries(names[:-1])

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
        selectors = []
        for steps in (job["steps"], public["jobs"]["korean-public-test"]["steps"]):
            commands = [step["run"] for step in steps if "run" in step]
            self.assertTrue(any("qualify-vector-runtime.py" in command and "--authorize-install" in command for command in commands))
            selection = [step for step in steps if step.get("name") == "Select SQLite-capable Python for vector acceptance"]
            self.assertEqual(len(selection), 1)
            self.assertEqual(selection[0]["if"], "runner.os == 'macOS'")
            self.assertIn('if python -I -S -c "$probe"', selection[0]["run"])
            self.assertIn("enable_load_extension(True)", selection[0]["run"])
            self.assertIn("brew install python@3.13", selection[0]["run"])
            self.assertTrue(any("VECTOR_PYTHON:-python" in command and "qualify-vector-runtime.py" in command for command in commands))
            selectors.append(selection[0]["run"])
        self.assertEqual(selectors[0], selectors[1])


if __name__ == "__main__":
    unittest.main()
