"""Small isolated checks; no use of the shared source tests/work or build tree."""
from datetime import timedelta
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from scripts import test_artifacts as a


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="hive-artifact-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        (self.root / "hive-source.json").write_text('{}', encoding="utf-8")
        (self.root / "tests/work/old").mkdir(parents=True)
        (self.root / "tests/work/old/data").write_text("synthetic", encoding="utf-8")
        (self.root / "tests/results").mkdir()
        (self.root / "tests/results/evidence.md").write_text("# Synthetic evidence\n", encoding="utf-8")
        self.git("init", "-q")
        self.git("add", "hive-source.json", "tests/results/evidence.md")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture")
        self.manager = a.Manager(self.root)
        self.snapshot = [{"pid": os.getpid(), "parent": 0, "name": "python", "start": "own-start", "command": "test_artifacts_checks", "image": ""}]
        self.process_patch = patch.object(a, "processes", side_effect=lambda: self.snapshot)
        self.process_patch.start()
        self.addCleanup(self.process_patch.stop)

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, capture_output=True, check=True)

    def review(self, **changes):
        item = {"path": "tests/work/old", "state": "completed", "owner": "fixture", "reason": "verified finished", "report": "tests/results/evidence.md"}
        item.update(changes)
        self.manager.review([item])

    def scan(self):
        return self.manager.scan(selected=["tests/work/old"])[0]

    def test_unknown_is_not_a_delete_candidate(self):
        self.assertEqual("review", self.scan()["status"])
        self.assertEqual("unowned-artifact", self.scan()["reason"])

    def test_completed_committed_evidence_is_eligible_preview_does_not_delete(self):
        self.review()
        rows = self.manager.cleanup()
        self.assertEqual("eligible", rows[0]["status"])
        self.assertTrue((self.root / "tests/work/old/data").exists())

    def test_uncommitted_or_changed_evidence_prevents_deletion(self):
        self.review()
        (self.root / "tests/results/evidence.md").write_text("changed", encoding="utf-8")
        self.assertEqual("review", self.scan()["status"])
        with self.assertRaises(a.ArtifactError):
            self.review()

    def test_missing_evidence_prevents_deletion(self):
        with self.assertRaises(a.ArtifactError):
            self.review(report="tests/results/missing.md")

    def test_plain_markdown_with_json_array_is_valid_reviewed_evidence(self):
        report = self.root / "tests/results/evidence.md"
        report.write_bytes(b"# Evidence\n\n```json\n[]\n```\n")
        self.git("add", "tests/results/evidence.md")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "review")
        self.review()
        self.assertEqual("eligible", self.scan()["status"])

    def test_live_process_preserved(self):
        self.review()
        self.snapshot.append({"pid": 999, "parent": 0, "name": "hive.exe", "start": "abc", "command": "hive --target " + str(self.root / "tests/work/old"), "image": "hive"})
        self.assertEqual("active", self.scan()["status"])

    def test_unavailable_relevant_process_identity_preserved(self):
        self.review()
        self.snapshot.append({"pid": 999, "parent": 0, "name": "hive.exe", "start": "abc", "command": "", "image": ""})
        self.assertEqual("process-identity-unavailable", self.scan()["reason"])

    def test_process_snapshot_failure_aborts(self):
        self.review()
        with patch.object(a, "processes", side_effect=OSError("unavailable")):
            with self.assertRaises(OSError):
                self.manager.cleanup(apply=True)

    def test_reuse_requires_concrete_task_and_within_72_hours(self):
        with self.assertRaises(a.ArtifactError):
            self.review(state="retained", review_at=(a.now() + timedelta(hours=73)).isoformat(), task="rerun")
        with self.assertRaises(a.ArtifactError):
            self.review(state="retained", review_at=(a.now() + timedelta(hours=2)).isoformat())
        self.review(state="retained", review_at=(a.now() + timedelta(hours=2)).isoformat(), task="rerun")
        self.assertEqual("retained", self.scan()["status"])

    def test_expired_review_is_not_deletion_permission(self):
        self.review(state="review", review_at=(a.now() + timedelta(hours=2)).isoformat())
        record = self.manager.records()[0]
        record["review_at"] = (a.now() - timedelta(seconds=1)).isoformat()
        self.manager.save(record)
        self.assertEqual("expired-review-required", self.scan()["reason"])

    def test_parent_reservation_preserves_children(self):
        self.review()
        self.review(path="tests/work", state="retained", task="concurrent acceptance", review_at=(a.now() + timedelta(hours=2)).isoformat())
        self.assertEqual("retained", self.scan()["status"])

    def test_explicit_release_removes_parent_hold_not_child_evidence_gate(self):
        self.review(path="tests/work", state="retained", task="acceptance", review_at=(a.now() + timedelta(hours=2)).isoformat())
        self.review(path="tests/work", state="released")
        self.assertEqual("review", self.scan()["status"])
        self.review()
        self.assertEqual("eligible", self.scan()["status"])

    def test_paths_outside_owned_roots_and_root_deletion_refused(self):
        for path in ("tests/work", "tests/work/../fixtures", "target", "target/release", "C:/tmp", "/tmp", "tests/work//old"):
            with self.subTest(path=path), self.assertRaises(a.ArtifactError):
                self.manager.target(path)

    def test_symlink_escape_refused(self):
        alias = self.root / "tests/work/old/alias"
        try:
            alias.symlink_to(self.root / "hive-source.json")
        except OSError as error:
            if os.name == "nt" and error.winerror == 1314:
                self.skipTest("Windows symlink privilege unavailable; junction checked separately")
            raise
        with self.assertRaises(a.ArtifactError):
            self.review()

    @unittest.skipUnless(os.name == "nt", "Windows junction contract")
    def test_junction_descendant_refused(self):
        alias = self.root / "tests/work/old/alias"
        result = subprocess.run(["pwsh", "-NoProfile", "-Command", "New-Item -ItemType Junction -Path $env:FIXTURE_ALIAS -Target $env:FIXTURE_ROOT | Out-Null"], env={**os.environ, "FIXTURE_ALIAS": str(alias), "FIXTURE_ROOT": str(self.root)}, capture_output=True)
        self.assertEqual(0, result.returncode, result.stderr)
        try:
            with self.assertRaises(a.ArtifactError):
                self.review()
        finally:
            alias.unlink() if alias.is_symlink() else os.rmdir(alias)

    def test_tracked_content_refused(self):
        self.review()
        self.git("add", "tests/work/old/data")
        self.assertEqual("tracked content refused", self.scan()["reason"])

    def test_concurrent_lock_refused(self):
        with self.manager.lock():
            with self.assertRaises(a.ArtifactError):
                with self.manager.lock():
                    self.fail("acquired twice")

    def test_fresh_state_change_prevents_delete(self):
        self.review()
        original = self.manager.scan
        calls = 0
        def changing(**kwargs):
            nonlocal calls
            calls += 1
            rows = original(**kwargs)
            if calls == 2:
                rows[0]["fingerprint"] = "different"
            return rows
        with patch.object(self.manager, "scan", side_effect=changing), patch.object(self.manager, "remove") as remove:
            rows = self.manager.cleanup(apply=True)
            remove.assert_not_called()
        self.assertEqual("cleanup-failed", rows[0]["status"])

    def test_partial_delete_failure_is_not_success(self):
        self.review()
        def partial(_):
            (self.root / "tests/work/old/data").unlink()
            raise PermissionError("locked remainder")
        with patch.object(self.manager, "remove", side_effect=partial):
            self.assertEqual("cleanup-failed", self.manager.cleanup(apply=True)[0]["status"])

    def test_actual_bounded_cleanup_and_external_sentinel(self):
        self.review()
        sentinel = self.root / "keep"
        sentinel.write_text("unchanged", encoding="utf-8")
        self.assertEqual("removed", self.manager.cleanup(apply=True)[0]["status"])
        self.assertFalse((self.root / "tests/work/old").exists())
        self.assertEqual("unchanged", sentinel.read_text(encoding="utf-8"))

    def test_shared_active_owner_prevents_completed_owner_cleanup(self):
        run = a.Run("fixture", ["synthetic"], root=self.root, paths=["tests/work/old"])
        self.review()
        self.assertEqual("active", self.scan()["status"])
        run.finish(1)
        self.assertEqual("review", self.scan()["status"])

    def test_pid_reuse_or_lost_process_never_becomes_completed(self):
        a.Run("fixture", ["synthetic"], root=self.root, paths=["tests/work/old"])
        self.snapshot[0]["start"] = "reused"
        self.assertEqual("stale-or-reused-process-identity", self.scan()["reason"])

    def test_report_failure_leaves_active_reservation(self):
        run = a.Run("fixture", ["synthetic"], root=self.root, paths=["tests/work/old"])
        with patch.object(run, "write", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                run.finish(0)
        self.assertEqual("active", self.manager.records()[0]["state"])

    def test_report_success_failure_cancelled_and_command_exit_code(self):
        for code, status in ((0, "passed"), (1, "failed"), (130, "cancelled")):
            run = a.Run("fixture", ["synthetic"], root=self.root)
            actual = run.execute([sys.executable, "-c", f"print('Ran 3 tests'); print('OK (skipped=1)'); raise SystemExit({code})"])
            self.assertEqual(code, actual)
            report = run.finish(code, status=status)
            text = (self.root / report).read_text(encoding="utf-8")
            self.assertIn(f'"status": "{status}"', text)
            self.assertIn("skipped=1", text)
            self.assertNotIn(str(self.root), text)

    def test_attachment_shape_preserved_and_private_values_redacted(self):
        run = a.Run("fixture", ["synthetic"], root=self.root)
        source = self.root / "tests/work/old/receipt.json"
        source.write_text(json.dumps({"status": "passed", "calls": 3, "token": "never-export", "path": str(self.root / "tests/work/old")}), encoding="utf-8")
        target = run.archive_json(source)
        result = json.loads(target.read_text(encoding="utf-8"))
        self.assertEqual("passed", result["status"])
        self.assertEqual(3, result["calls"])
        self.assertNotIn("never-export", target.read_text(encoding="utf-8"))
        self.assertNotIn(str(self.root), target.read_text(encoding="utf-8"))

    def test_cli_check_flags_unknown_artifacts(self):
        self.assertEqual(1, a.cli(["check", "--root", str(self.root)]))

    def test_new_bytes_after_completion_need_new_review(self):
        self.review()
        (self.root / "tests/work/old/new").write_text("new owner", encoding="utf-8")
        self.assertEqual("artifact changed since completion review", self.scan()["reason"])

    def test_missing_or_changed_attachment_blocks_cleanup(self):
        run = a.Run("fixture", ["synthetic"], root=self.root)
        receipt = self.root / "tests/work/old/receipt.json"
        receipt.write_text('{"status":"passed"}', encoding="utf-8")
        attachment = run.archive_json(receipt)
        report = run.finish(0)
        self.git("add", "tests/results")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "evidence")
        self.review(report=report)
        attachment.write_text('{"status":"failed"}', encoding="utf-8")
        self.assertEqual("review", self.scan()["status"])

    def test_operator_resolves_terminal_run_but_not_live_owner(self):
        run = a.Run("fixture", ["synthetic"], root=self.root, paths=["tests/work/old"])
        run.finish(1)
        self.review()
        self.assertEqual("eligible", self.scan()["status"])

    def test_shared_build_is_not_automatically_released(self):
        (self.root / "target/debug").mkdir(parents=True)
        run = a.Run("fixture", ["synthetic"], root=self.root, paths=["target/debug"])
        run.finish(0)
        self.assertEqual("review", self.manager.scan(selected=["target/debug"])[0]["status"])

    def test_legacy_archive_does_not_invent_success_and_keeps_original(self):
        source = self.root / "tests/work/old/result.json"
        source.write_text('{"elapsed":1.5}', encoding="utf-8")
        report = a.archive_legacy(self.root, "tests/work/old/result.json", purpose="synthetic", limit="Original measurement only")
        text = (self.root / report).read_text(encoding="utf-8")
        self.assertIn("not specified in original", text)
        self.assertTrue(source.is_file())
        self.assertIn(a.sha(source), text)

    def test_escaped_paths_and_embedded_credentials_redacted(self):
        raw = repr(str(self.root / "tests/work/old")) + ' {"token": "not-for-export"} Bearer another-secret'
        result = a.clean_text(raw, self.root)
        self.assertNotIn("not-for-export", result)
        self.assertNotIn("another-secret", result)
        self.assertNotIn(str(self.root).replace("\\", "\\\\"), result)
        self.assertEqual(["--authorization-token", "<redacted>"], a.scrub(["--authorization-token", "never-keep"]))
        self.assertNotIn("header-secret", a.clean_text("Authorization: Bearer header-secret"))

    def test_lock_excludes_another_process(self):
        with self.manager.lock():
            command = [sys.executable, "-B", str(a.ROOT / "scripts/test-artifacts.py"), "cleanup", "--root", str(self.root), "--apply"]
            result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(2, result.returncode)
        self.assertIn("already in progress", result.stderr)
        self.assertTrue((self.root / "tests/work/old/data").exists())

    @unittest.skipUnless(os.name == "nt", "Windows file sharing contract")
    def test_real_locked_file_retained_and_retry_possible(self):
        import ctypes
        from ctypes import wintypes
        api = ctypes.WinDLL("kernel32", use_last_error=True)
        api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
        api.CreateFileW.restype = wintypes.HANDLE
        api.CloseHandle.argtypes = [wintypes.HANDLE]
        api.CloseHandle.restype = wintypes.BOOL
        self.review()
        handle = api.CreateFileW(str(self.root / "tests/work/old/data"), 0x80000000, 1, None, 3, 0, None)
        self.assertNotEqual(ctypes.c_void_p(-1).value, handle)
        try:
            rows = self.manager.cleanup(apply=True)
            self.assertEqual("cleanup-failed", rows[0]["status"])
            self.assertTrue((self.root / "tests/work/old/data").exists())
        finally:
            api.CloseHandle(handle)
        # A fresh explicit review is necessary if a partial deletion changed metadata.
        self.review()
        self.assertEqual("removed", self.manager.cleanup(apply=True)[0]["status"])


if __name__ == "__main__":
    unittest.main()
