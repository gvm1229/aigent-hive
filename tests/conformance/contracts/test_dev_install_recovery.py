"""No real installation or host control: authenticated recovery file boundaries."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("dev_recovery", ROOT / "scripts/recover-dev-install.py")
recovery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recovery)


class DeveloperRecovery(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="hive-dev-recovery-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name).resolve()
        self.root = self.base / "user"
        self.backup = self.base / "backup"
        self.generated = recovery.PACKAGE + "skills/example/SKILL.md"
        self.binary = self.base / "bin/hive.exe"
        self.put(self.binary, b"developer executable")
        self.put(self.root / self.generated, b"generated")
        self.put(self.root / recovery.METADATA, b"metadata")
        self.protected = [".hive/knowledge/note.md", ".hive/config/user-setup.yml",
                          ".hive/runtime/usage-monitor/config.json", ".codex/AGENTS.md",
                          recovery.PACKAGE + "foreign-file.txt"]
        for name in self.protected:
            self.put(self.root / name, b"saved user data")
        entries = [self.entry(self.generated, "immutable-plugin-package"),
                   self.entry(recovery.METADATA, "host-adapter-metadata"),
                   self.entry(".codex/AGENTS.md", "shared-marker"),
                   self.entry(".hive/knowledge/note.md", "canonical-data-protected")]
        self.manifest = {"schema_version": 1, "host": "codex", "entries": entries}
        self.put(self.root / recovery.MANIFEST, recovery.encode(self.manifest))
        self.put(self.root / ".hive/dev-install/target-path", str(self.binary).encode())
        self.put(self.root / ".hive/dev-install/developer-sha256",
                 recovery.digest(self.binary.read_bytes()).removeprefix("sha256:").encode())
        valid = {"code": "hive.user-install-valid", "status": "success", "changed_paths": []}
        self.runner = patch.object(recovery.subprocess, "run", return_value=
                                  subprocess.CompletedProcess([], 0, json.dumps(valid).encode(), b""))
        self.run_mock = self.runner.start()
        self.addCleanup(self.runner.stop)

    @staticmethod
    def put(path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def entry(self, name, ownership):
        return {"path": name, "ownership": ownership,
                "digest": recovery.digest((self.root / name).read_bytes())}

    def plan(self):
        return recovery.make_plan(self.root, self.binary)

    def test_stage_and_restore_preserve_all_user_data_and_foreign_files(self):
        plan = self.plan()
        recovery.stage(plan, self.backup)
        for entry in plan["entries"]:
            self.assertFalse((self.root / entry["path"]).exists())
            self.assertTrue((self.backup / "files" / entry["path"]).is_file())
        for name in self.protected:
            self.assertEqual((self.root / name).read_bytes(), b"saved user data")
        recovery.restore(self.backup)
        recovery.restore(self.backup)  # Idempotent, with backup retained.
        self.assertEqual(self.plan(), plan)

    def test_cli_validation_failure_never_creates_a_backup(self):
        self.run_mock.return_value = subprocess.CompletedProcess([], 3, b'{}', b'')
        with self.assertRaises(ValueError):
            self.plan()
        self.assertFalse(self.backup.exists())

    def test_modified_developer_executable_rejected_before_execution(self):
        self.binary.write_bytes(b"different binary")
        with self.assertRaises(ValueError):
            self.plan()
        self.run_mock.assert_not_called()

    def test_modified_generated_file_is_rejected(self):
        (self.root / self.generated).write_bytes(b"user edit")
        with self.assertRaises(ValueError):
            self.plan()

    def test_mutation_after_plan_is_rejected_before_any_move(self):
        plan = self.plan()
        (self.root / recovery.METADATA).write_bytes(b"changed")
        with self.assertRaises(ValueError):
            recovery.stage(plan, self.backup)
        self.assertTrue((self.root / self.generated).exists())
        self.assertFalse(self.backup.exists())

    def test_partial_failure_restores_generated_files(self):
        plan = self.plan()
        real_write = recovery.write_new
        def fail(path, data):
            if path == self.backup / "files" / recovery.METADATA:
                raise OSError("simulated disk failure")
            return real_write(path, data)
        with patch.object(recovery, "write_new", side_effect=fail):
            with self.assertRaises(OSError):
                recovery.stage(plan, self.backup)
        for entry in plan["entries"]:
            self.assertEqual(recovery.digest((self.root / entry["path"]).read_bytes()), entry["digest"])

    def test_restore_never_overwrites_new_files(self):
        recovery.stage(self.plan(), self.backup)
        self.put(self.root / recovery.METADATA, b"new occupant")
        with self.assertRaises(ValueError):
            recovery.restore(self.backup)
        self.assertFalse((self.root / self.generated).exists())
        self.assertEqual((self.root / recovery.METADATA).read_bytes(), b"new occupant")

    def test_hostile_paths_duplicates_and_protected_namespace_rejected(self):
        for path in ("../escape", "/absolute", "C:/escape", "a\\b", "a/../b",
                     ".hive/knowledge/user.md", ".hive/config/user-setup.yml"):
            manifest = {**self.manifest, "entries": [{"path": path,
                        "ownership": "immutable-plugin-package", "digest": "sha256:" + "a" * 64}]}
            with self.subTest(path=path), self.assertRaises(ValueError):
                recovery.selected_paths(manifest)
        duplicate = {**self.manifest, "entries": self.manifest["entries"] * 2}
        with self.assertRaises(ValueError):
            recovery.selected_paths(duplicate)

    def test_backup_inside_runtime_is_rejected(self):
        with self.assertRaises(ValueError):
            recovery.stage(self.plan(), self.root / ".hive/runtime/backup")

    def test_symlink_is_rejected(self):
        link = self.base / "linked"
        try:
            link.symlink_to(self.binary)
        except OSError:
            self.skipTest("symlink privilege unavailable")
        with self.assertRaises(ValueError):
            recovery.regular(link)

    def test_windows_reparse_point_is_rejected_without_following_it(self):
        from types import SimpleNamespace
        real_stat = Path.lstat
        def attributes(path):
            result = real_stat(path)
            if path == self.binary.parent:
                return SimpleNamespace(st_mode=result.st_mode, st_file_attributes=0x400)
            return result
        with patch.object(Path, "lstat", attributes), self.assertRaises(ValueError):
            recovery.regular(self.binary)


if __name__ == "__main__":
    unittest.main()
