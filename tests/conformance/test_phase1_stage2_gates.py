#!/usr/bin/env python3
"""Black-box Stage 2 namespace, consent, and Git visibility gates."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
from pathlib import Path

from tests.conformance.phase1_support import (
    FIXTURE_ROOT,
    Phase1CliTestCase,
    snapshot_tree,
)


CANONICAL_VISIBLE_PATHS = (
    ".hive/config/harness.toml",
    ".hive/config/approved-skills.yml",
    ".hive/config/capability-resolution.yml",
    ".hive/config/knowledge-scope.yml",
    ".hive/config/role-seeds.yml",
    ".hive/knowledge/Raw/README.md",
    ".hive/knowledge/Schema/schema.md",
    ".hive/knowledge/Wiki/index.md",
    ".hive/knowledge/Wiki/log.md",
    ".hive/knowledge/suppression.yml",
    ".hive/runs/README.md",
    ".hive/setup-answers.yml",
    ".hive/team/roles/README.md",
    ".hive/team/roles/reviewer.md",
)


def semantic_json_digest(raw: bytes) -> str:
    normalized = json.dumps(
        json.loads(raw),
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(normalized).hexdigest()


def special_tree_snapshot(root: Path) -> dict[str, tuple[object, ...]]:
    """Snapshot a tree without opening FIFOs or unreadable regular files."""

    snapshot: dict[str, tuple[object, ...]] = {}
    if not root.exists():
        return snapshot
    for directory, names, files in os.walk(root, followlinks=False):
        directory_path = Path(directory)
        for name in sorted(names + files):
            path = directory_path / name
            relative = path.relative_to(root).as_posix()
            metadata = path.lstat()
            mode = stat.S_IMODE(metadata.st_mode)
            if stat.S_ISDIR(metadata.st_mode):
                snapshot[relative] = ("directory", mode)
            elif stat.S_ISFIFO(metadata.st_mode):
                snapshot[relative] = ("fifo", mode)
            elif stat.S_ISLNK(metadata.st_mode):
                snapshot[relative] = ("symlink", mode, os.readlink(path))
            elif stat.S_ISREG(metadata.st_mode):
                snapshot[relative] = ("file", mode, metadata.st_size)
            else:
                snapshot[relative] = ("special", mode, metadata.st_size)
    return snapshot


class Phase1ForeignNamespaceReadWriteGate(Phase1CliTestCase):
    def test_setup_does_not_read_or_write_project_or_host_global_namespaces(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        fake_home = self.work_root / "isolated-home"
        target.mkdir()
        fake_home.mkdir()

        guarded_roots = [
            (target / ".omx", "state.json"),
            (target / ".omc", "state.json"),
            (target / ".codex", "config.toml"),
            (target / ".claude", "settings.json"),
            (target / ".agents", "config.yml"),
            (fake_home / ".codex", "config.toml"),
            (fake_home / ".claude", "settings.json"),
        ]
        unreadable_files: list[Path] = []
        for index, (root, known_config_name) in enumerate(guarded_roots):
            root.mkdir(parents=True)
            (root / "sentinel.bin").write_bytes(
                b"foreign namespace bytes\x00\xff:" + str(index).encode("ascii")
            )
            if os.name != "nt" and hasattr(os, "mkfifo"):
                os.mkfifo(root / known_config_name)
                unreadable = root / "must-not-read.bin"
                unreadable.write_bytes(b"unreadable foreign bytes\n")
                unreadable.chmod(0)
                unreadable_files.append(unreadable)
            else:
                (root / known_config_name).write_bytes(
                    b"foreign config bytes\x00\xff\n"
                )

        before = {
            str(root): special_tree_snapshot(root)
            for root, _ in guarded_roots
        }
        environment = {
            "HOME": str(fake_home),
            "USERPROFILE": str(fake_home),
            "XDG_CONFIG_HOME": str(fake_home / ".config"),
            "CODEX_HOME": str(fake_home / ".codex"),
            "CLAUDE_CONFIG_DIR": str(fake_home / ".claude"),
        }

        try:
            try:
                process, result = self.invoke_setup(
                    target,
                    timeout=2.0 if os.name != "nt" else 10.0,
                    environment=environment,
                )
            except subprocess.TimeoutExpired as error:
                self.fail(f"setup opened a guarded FIFO and blocked: {error}")
            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(result["status"], "success")
            self.assertEqual(result["code"], "hive.setup-complete")
            self.assertEqual(
                {
                    str(root): special_tree_snapshot(root)
                    for root, _ in guarded_roots
                },
                before,
            )
        finally:
            for unreadable in unreadable_files:
                unreadable.chmod(0o600)

        for index, (root, known_config_name) in enumerate(guarded_roots):
            self.assertEqual(
                (root / "sentinel.bin").read_bytes(),
                b"foreign namespace bytes\x00\xff:" + str(index).encode("ascii"),
            )
            if os.name == "nt" or not hasattr(os, "mkfifo"):
                self.assertEqual(
                    (root / known_config_name).read_bytes(),
                    b"foreign config bytes\x00\xff\n",
                )
        for unreadable in unreadable_files:
            self.assertEqual(
                unreadable.read_bytes(),
                b"unreadable foreign bytes\n",
            )


class Phase1HookForeignEntryGate(Phase1CliTestCase):
    def test_hook_approval_and_revocation_preserve_adjacent_foreign_entries(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        hook_directory = target / ".hive/hooks"
        hook_directory.mkdir(parents=True)
        foreign_entry = hook_directory / "foreign-runtime-entry.json"
        foreign_bytes = (
            b'{\n  "event": "ForeignEvent",\n'
            b'  "command": ["foreign", "--flag"],\n'
            b'  "enabled": true\n}\n'
        )
        foreign_entry.write_bytes(foreign_bytes)
        adjacent_bytes = hook_directory / "foreign-preamble.bin"
        adjacent_bytes.write_bytes(b"\x00foreign hook-adjacent bytes\xff\n")
        expected_semantic_digest = semantic_json_digest(foreign_bytes)

        setup_process, setup_result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(setup_process.returncode, 0, setup_process.stderr)
        self.assertEqual(setup_result["status"], "success")
        self.assertEqual(setup_result["code"], "hive.setup-complete")
        self.assertNotIn(
            ".hive/hooks/foreign-runtime-entry.json",
            setup_result["changed_paths"],
        )
        self.assertEqual(foreign_entry.read_bytes(), foreign_bytes)
        self.assertEqual(
            adjacent_bytes.read_bytes(),
            b"\x00foreign hook-adjacent bytes\xff\n",
        )
        self.assertEqual(
            semantic_json_digest(foreign_entry.read_bytes()),
            expected_semantic_digest,
        )

        revoke_process, revoke_result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(revoke_process.returncode, 0, revoke_process.stderr)
        self.assertEqual(revoke_result["status"], "success")
        self.assertEqual(revoke_result["code"], "hive.setup-complete")
        self.assertNotIn(
            ".hive/hooks/foreign-runtime-entry.json",
            revoke_result["changed_paths"],
        )
        self.assertEqual(foreign_entry.read_bytes(), foreign_bytes)
        self.assertEqual(
            adjacent_bytes.read_bytes(),
            b"\x00foreign hook-adjacent bytes\xff\n",
        )
        self.assertEqual(
            semantic_json_digest(foreign_entry.read_bytes()),
            expected_semantic_digest,
        )


class Phase1CanonicalGitVisibilityGate(Phase1CliTestCase):
    def require_git(self) -> str:
        git = shutil.which("git")
        if git is None:
            self.skipTest("git executable is unavailable")
        return git

    def initialize_consumer_repository(self, target: Path) -> str:
        git = self.require_git()
        process = subprocess.run(
            [git, "init", "--quiet"],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return git

    def assert_not_ignored(self, git: str, target: Path, relative: str) -> None:
        process = subprocess.run(
            [git, "check-ignore", "-v", "--", relative],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(
            process.returncode,
            1,
            f"{relative} was unexpectedly ignored by {process.stdout!r}",
        )
        self.assertEqual(process.stdout, "")

    def assert_ignored(self, git: str, target: Path, relative: str) -> None:
        process = subprocess.run(
            [git, "check-ignore", "-v", "--", relative],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(
            process.returncode,
            0,
            f"{relative} was unexpectedly Git-visible: {process.stderr!r}",
        )
        self.assertIn(relative, process.stdout)

    def install(self) -> tuple[Path, str]:
        target = self.work_root / "consumer"
        target.mkdir()
        git = self.initialize_consumer_repository(target)
        process, result = self.invoke_setup(target)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["code"], "hive.setup-complete")
        return target, git

    def test_canonical_markdown_yaml_toml_role_and_run_files_are_git_visible(
        self,
    ) -> None:
        target, git = self.install()

        status = subprocess.run(
            [git, "status", "--porcelain=v1", "--untracked-files=all"],
            cwd=target,
            check=False,
            text=True,
            capture_output=True,
        )

        self.assertEqual(status.returncode, 0, status.stderr)
        visible = {
            line[3:].replace("\\", "/")
            for line in status.stdout.splitlines()
            if len(line) >= 4
        }
        for relative in CANONICAL_VISIBLE_PATHS:
            with self.subTest(path=relative):
                self.assertTrue((target / relative).is_file())
                self.assertIn(relative, visible)
                self.assert_not_ignored(git, target, relative)

    def test_consumer_gitignore_excludes_only_indexes_and_backups(self) -> None:
        target, git = self.install()
        ignored = (
            ".hive/index/wiki.sqlite",
            ".hive/index/wiki.sqlite3",
            ".hive/index/wiki.sqlite-wal",
            ".hive/index/wiki.sqlite-shm",
            ".hive/index/wiki.sqlite3-wal",
            ".hive/index/wiki.sqlite3-shm",
            ".hive/backups/rollback/config.yml",
        )
        visible = (
            ".hive/index/.stale",
            ".hive/index/search.db",
            ".hive/config/local-visible.toml",
            ".hive/knowledge/Wiki/local-visible.md",
            ".hive/team/roles/local-visible.md",
            ".hive/runs/run-001/PLAN.md",
            ".hive/runs/run-001/STATUS.md",
        )
        for relative in ignored + visible:
            path = target / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"representative consumer bytes\n")

        for relative in ignored:
            with self.subTest(path=relative):
                self.assert_ignored(git, target, relative)
        for relative in visible:
            with self.subTest(path=relative):
                self.assert_not_ignored(git, target, relative)


class Phase1NoConsentProjectionGate(Phase1CliTestCase):
    def test_absent_consent_creates_no_project_host_projection(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-no-role-no-hook.yml",
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["code"], "hive.setup-complete")
        for namespace in (".codex", ".claude", ".agents"):
            with self.subTest(namespace=namespace):
                self.assertFalse((target / namespace).exists())
                self.assertFalse(
                    any(
                        path == namespace or path.startswith(namespace + "/")
                        for path in result["changed_paths"]
                    )
                )
