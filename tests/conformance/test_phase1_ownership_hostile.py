#!/usr/bin/env python3
"""Cross-platform path, namespace, manifest, and symlink hostile tests."""

from __future__ import annotations

from pathlib import Path

from tests.conformance.phase1_support import (
    Phase1CliTestCase,
    snapshot_tree,
    write_yaml,
)


class Phase1OwnershipHostile(Phase1CliTestCase):
    def assert_hook_path_rejected_without_namespace_changes(
        self,
        hostile_path: str,
    ) -> None:
        target = self.work_root / "consumer"
        outside = self.work_root / "outside"
        target.mkdir()
        outside.mkdir()
        (outside / "sentinel.txt").write_bytes(b"outside bytes\n")
        answer_path, answers = self.copied_answers("answers-partial-hooks.yml")
        hooks = answers["approved_fallback_hooks"]
        self.assertIsInstance(hooks, list)
        hooks[0]["path"] = hostile_path
        write_yaml(answer_path, answers)
        target_before = snapshot_tree(target)
        namespace_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities="capabilities-absent.json",
        )

        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(self.work_root), namespace_before)

    def test_absolute_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            str(self.work_root / "outside/escaped-hook")
        )

    def test_posix_parent_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            "../outside/escaped-hook"
        )

    def test_backslash_parent_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            r"..\outside\escaped-hook"
        )

    def test_windows_drive_relative_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            r"C:outside\escaped-hook"
        )

    def test_windows_drive_absolute_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            r"C:\outside\escaped-hook"
        )

    def test_windows_unc_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            r"\\server\share\escaped-hook"
        )

    def test_windows_rooted_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            r"\outside\escaped-hook"
        )

    def test_omx_namespace_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".omx/hooks/escaped-hook"
        )

    def test_omc_namespace_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".omc/hooks/escaped-hook"
        )

    def test_codex_host_configuration_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".codex/settings.json"
        )

    def test_claude_host_configuration_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".claude/settings.json"
        )

    def test_agents_host_configuration_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".agents/hooks/escaped-hook"
        )

    def test_manifest_outside_hook_path_is_rejected(self) -> None:
        self.assert_hook_path_rejected_without_namespace_changes(
            ".hive/rogue-hook"
        )

    def create_symlink_or_skip(
        self,
        source: Path,
        destination: Path,
        *,
        directory: bool,
    ) -> None:
        try:
            destination.symlink_to(source, target_is_directory=directory)
        except (OSError, NotImplementedError) as error:
            self.skipTest(f"symlink creation is unavailable on this host: {error}")

    def test_generated_directory_symlink_escape_rolls_back_every_namespace(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        outside = self.work_root / "outside"
        target.mkdir()
        outside.mkdir()
        (target / ".hive").mkdir()
        self.create_symlink_or_skip(
            outside,
            target / ".hive/config",
            directory=True,
        )
        target_before = snapshot_tree(target)
        namespace_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(self.work_root), namespace_before)

    def test_generated_file_symlink_escape_rolls_back_every_namespace(
        self,
    ) -> None:
        target = self.work_root / "consumer"
        outside = self.work_root / "outside"
        target.mkdir()
        outside.mkdir()
        (target / ".hive/config").mkdir(parents=True)
        outside_file = outside / "harness.toml"
        outside_file.write_bytes(b"outside config bytes\n")
        self.create_symlink_or_skip(
            outside_file,
            target / ".hive/config/harness.toml",
            directory=False,
        )
        target_before = snapshot_tree(target)
        namespace_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(self.work_root), namespace_before)

    def test_shared_agents_symlink_escape_rolls_back_every_namespace(self) -> None:
        target = self.work_root / "consumer"
        outside = self.work_root / "outside"
        target.mkdir()
        outside.mkdir()
        outside_file = outside / "AGENTS.md"
        outside_file.write_bytes(b"outside shared bytes\n")
        self.create_symlink_or_skip(
            outside_file,
            target / "AGENTS.md",
            directory=False,
        )
        target_before = snapshot_tree(target)
        namespace_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(target)

        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(self.work_root), namespace_before)

    def test_foreign_runtime_namespaces_remain_byte_identical(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        for namespace in (".omx", ".omc"):
            path = target / namespace
            path.mkdir()
            (path / "sentinel.bin").write_bytes(
                f"{namespace} foreign bytes\n".encode()
            )
        before = {
            namespace: snapshot_tree(target / namespace)
            for namespace in (".omx", ".omc")
        }

        process, _ = self.invoke_setup(target)

        self.assertEqual(process.returncode, 0)
        self.assertEqual(
            {
                namespace: snapshot_tree(target / namespace)
                for namespace in (".omx", ".omc")
            },
            before,
        )
