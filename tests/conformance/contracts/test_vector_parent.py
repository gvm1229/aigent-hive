"""Parent-death cancellation without models, downloads, or provider processes."""
from __future__ import annotations
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import tarfile
import time
import tomllib
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "crates/hive-cli/src/vector_parent.py"
spec = importlib.util.spec_from_file_location("vector_parent", SOURCE)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


class VectorParentLifetime(unittest.TestCase):
    def setUp(self):
        (ROOT / "tests/work").mkdir(parents=True, exist_ok=True)

    def test_windows_job_patch_preserves_the_exact_original_package_except_one_flag(self):
        provenance = json.loads((ROOT / "vendor/process-wrap-provenance.json").read_bytes())
        archive = ROOT / provenance["archive_path"]
        self.assertEqual(hashlib.sha256(archive.read_bytes()).hexdigest(), "2e842efad9119158434d193c6682e2ebee4b44d6ad801d7b349623b3f57cdf55")
        vendor = ROOT / provenance["root"]
        originals = {}
        with tarfile.open(archive, "r:gz") as package:
            for member in package.getmembers():
                self.assertTrue(member.isfile())
                self.assertTrue(member.name.startswith("process-wrap-9.1.0/"))
                relative = member.name.removeprefix("process-wrap-9.1.0/")
                self.assertNotIn("..", Path(relative).parts)
                originals[relative] = package.extractfile(member).read()
        self.assertEqual(len(originals), 84)
        self.assertEqual({name:hashlib.sha256(content).hexdigest() for name,content in originals.items()}, provenance["original_files"])
        self.assertEqual({path.relative_to(vendor).as_posix() for path in vendor.rglob("*") if path.is_file()}, set(originals))
        for name, original in originals.items():
            expected = original
            if name == provenance["patch"]["path"]:
                before, after = (provenance["patch"][key].encode() for key in ("before", "after"))
                self.assertEqual(original.count(before), 1)
                expected = original.replace(before, after)
                self.assertEqual(hashlib.sha256(expected).hexdigest(), provenance["patch"]["patched_sha256"])
            with self.subTest(path=name):
                self.assertEqual((vendor / name).read_bytes(), expected)
        for name in ("LICENSE-MIT", "LICENSE-APACHE", "COPYRIGHT"):
            self.assertIn(name, originals)

    def test_cargo_uses_the_pinned_job_patch_without_adding_it_to_workspace_members(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text("utf-8"))
        self.assertEqual(manifest["patch"]["crates-io"]["process-wrap"], {"path":"vendor/process-wrap-9.1.0"})
        self.assertIn("vendor/process-wrap-9.1.0", manifest["workspace"]["exclude"])
        self.assertEqual(manifest["workspace"]["dependencies"]["process-wrap"]["version"], "=9.1.0")
        package = next(item for item in tomllib.loads((ROOT / "Cargo.lock").read_text("utf-8"))["package"] if item["name"] == "process-wrap")
        self.assertEqual(package["version"], "9.1.0")
        self.assertNotIn("source", package)

    def test_fresh_checkout_creates_its_own_temporary_work_parent(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as directory:
            fresh = Path(directory) / "fresh-source"
            with patch.dict(globals(), {"ROOT": fresh}):
                self.setUp()
            self.assertTrue((fresh / "tests/work").is_dir())

    def test_guarded_bootstrap_runs_with_bounded_command_line_and_no_download(self):
        source = (ROOT / "crates/hive-cli/src/vector_runtime.py").read_text("utf-8")
        wrapped = (SOURCE.read_text("utf-8") + "\n_hive_bind_parent(" + str(os.getpid()) +
                   ");\nexec(compile(" + json.dumps(source, ensure_ascii=False) + ", '<hive-vector>', 'exec'))")
        arguments = [sys.executable, "-I", "-S", "-B", "-c", wrapped]
        self.assertLess(len(subprocess.list2cmdline(arguments).encode("utf-16-le")) // 2, 30000)
        result = subprocess.run(arguments, input=json.dumps({"action": "unsupported-test-action"}),
                                capture_output=True, text=True, encoding="utf-8", timeout=10)
        self.assertEqual(result.returncode, 10, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"status": "error", "error_type": "ValueError"})

    def test_worker_executes_only_the_exact_bounded_bytes_and_preserves_future_imports(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            source = Path(work) / "worker.py"
            marker = Path(work) / "executed"
            code = ("from __future__ import annotations\nfrom pathlib import Path\nPath(" + repr(str(marker)) + ").write_text('safe')").encode()
            source.write_bytes(code)
            with self.assertRaises(ValueError):
                guard._hive_run_verified_file(str(source), "0" * 64)
            self.assertFalse(marker.exists())
            guard._hive_run_verified_file(str(source), hashlib.sha256(code).hexdigest())
            self.assertEqual(marker.read_text(), "safe")
            source.write_bytes(b" " * (1024 * 1024 + 1))
            with self.assertRaises(ValueError):
                guard._hive_run_verified_file(str(source), hashlib.sha256(source.read_bytes()).hexdigest())

    def test_invalid_parent_is_refused_before_starting_a_thread(self):
        with patch.object(guard._hive_parent_threading, "Thread") as thread:
            for identity in (None, True, 0, 1, -1, "2"):
                with self.assertRaises(ValueError):
                    guard._hive_bind_parent(identity)
            thread.assert_not_called()

    def test_unix_watch_exits_after_reparenting(self):
        with patch.object(guard._hive_parent_os, "getppid", side_effect=[42, 42, 1]), \
             patch.object(guard._hive_parent_time, "sleep") as sleep, \
             patch.object(guard._hive_parent_os, "_exit", side_effect=SystemExit(130)) as stop:
            with self.assertRaises(SystemExit) as error:
                guard._hive_watch_parent(42)
            self.assertEqual(error.exception.code, 130)
            self.assertEqual(sleep.call_count, 2)
            stop.assert_called_once_with(130)

    def test_unix_parent_already_gone_never_starts_work(self):
        with patch.object(guard._hive_parent_os, "name", "posix"), \
             patch.object(guard._hive_parent_os, "getppid", return_value=1), \
             patch.object(guard._hive_parent_os, "_exit", side_effect=SystemExit(130)), \
             patch.object(guard._hive_parent_threading, "Thread") as thread:
            with self.assertRaises(SystemExit):
                guard._hive_bind_parent(42)
            thread.assert_not_called()

    def test_windows_keeps_job_object_containment_without_a_false_reparenting_watch(self):
        with patch.object(guard._hive_parent_os, "name", "nt"), \
             patch.object(guard._hive_parent_threading, "Thread") as thread:
            guard._hive_bind_parent(42)
            thread.assert_not_called()

    @unittest.skipUnless(os.name == "posix", "real Unix reparenting; Windows uses native Job Objects")
    def test_actual_unix_helper_exits_after_owning_parent_is_killed(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "tests/work") as work:
            ready = Path(work) / "ready.json"
            source = SOURCE.read_text("utf-8")
            body = "\nimport json, pathlib, os, time\npathlib.Path(" + repr(str(ready)) + ").write_text(json.dumps({'pid':os.getpid()}))\ntime.sleep(60)"
            parent_code = ("import subprocess,sys,os,time\ncode=" + repr(source) +
                           "+'\\n_hive_bind_parent('+str(os.getpid())+')\\n'+" + repr(body) +
                           "\nsubprocess.Popen([sys.executable,'-I','-S','-B','-c',code],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)\ntime.sleep(60)")
            parent = subprocess.Popen([sys.executable, "-I", "-S", "-B", "-c", parent_code])
            child_pid = None
            def running(pid):
                state = subprocess.run(["ps", "-p", str(pid), "-o", "stat="], capture_output=True, text=True, timeout=5)
                return bool(state.stdout.strip()) and not state.stdout.strip().startswith("Z")
            try:
                deadline = time.monotonic() + 10
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(.05)
                self.assertTrue(ready.exists(), "real child did not start")
                child_pid = json.loads(ready.read_bytes())["pid"]
                parent.kill()
                parent.wait(timeout=5)
                deadline = time.monotonic() + 5
                while running(child_pid) and time.monotonic() < deadline:
                    time.sleep(.05)
                self.assertFalse(running(child_pid), "orphan helper remained running")
            finally:
                if parent.poll() is None:
                    parent.kill()
                parent.wait(timeout=5)
                if child_pid and running(child_pid):
                    os.kill(child_pid, signal.SIGKILL)


if __name__ == "__main__":
    unittest.main()
