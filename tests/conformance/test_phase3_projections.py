#!/usr/bin/env python3
"""Black-box installed projection and hook-exclusion conformance."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from pathlib import Path

from tests.conformance.phase1_support import (
    FIXTURE_ROOT,
    Phase1CliTestCase,
    read_yaml,
    snapshot_tree,
    write_yaml,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
PHASE3_FIXTURES = REPOSITORY_ROOT / "tests/fixtures/phase3"
HOST_PATHS = json.loads(
    (PHASE3_FIXTURES / "host-paths.json").read_text(encoding="utf-8")
)
LOCAL_SKILL_SOURCE = (
    PHASE3_FIXTURES / "optional/local-inspect/SKILL.md"
)
PROJECTED_BUILTINS = (
    "setup-harness",
    "hive-simple-question",
    "hive-prompt-refine",
    "hive-knowledge-capture",
    "hive-knowledge-query",
    "hive-knowledge-maintenance",
    "hive-knowledge-promote",
    "hive-project-upgrade",
    "hive-role-handoff",
    "hive-run-checkpoint",
    "hive-run-resume",
    "hive-judge-package",
    "hive-update",
    "hive-usage-guard",
    "hive-migrate",
)
CATALOG_ONLY = ()


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def canonical_digest(value: object) -> str:
    encoded = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest_bytes(encoded)


class Phase3ProjectionTestCase(Phase1CliTestCase):
    def answers_for_host(self, host: str) -> Path:
        answers = read_yaml(FIXTURE_ROOT / "answers-no-role-no-hook.yml")
        answers["primary_host"] = host
        path = self.work_root / f"answers-{host}.yml"
        write_yaml(path, answers)
        return path

    def capability_for_host(self, host: str) -> str:
        return {
            "codex": "capabilities-codex-omx.json",
            "claude": "capabilities-claude-omc.json",
            "antigravity": "capabilities-antigravity-absent.json",
        }[host]

    def install_host(self, host: str) -> Path:
        target = self.work_root / f"consumer-{host}"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=self.answers_for_host(host),
            capabilities=self.capability_for_host(host),
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return target

    def discovery_root(self, target: Path, host: str) -> Path:
        return target / HOST_PATHS[host]


class Phase3HostProjection(Phase3ProjectionTestCase):
    def test_active_skill_ledger_contains_only_implemented_builtins(
        self,
    ) -> None:
        target = self.install_host("codex")
        ledger_path = target / ".hive/config/active-skills.yml"
        self.assertTrue(ledger_path.is_file())
        ledger = read_yaml(ledger_path)
        skills = ledger["skills"]
        self.assertIsInstance(skills, list)
        names = {entry["name"] for entry in skills}
        self.assertEqual(names, set(PROJECTED_BUILTINS))
        for entry in skills:
            self.assertEqual(entry["source_type"], "built-in")
            self.assertIsNone(entry["consent_digest"])

    def test_each_host_uses_portable_skills_and_only_its_required_adapter(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.install_host(host)
                expected = self.discovery_root(target, host)
                self.assertTrue(expected.is_dir())
                self.assertTrue((target / ".agents/skills").is_dir())
                self.assertEqual(
                    (target / ".claude/skills").is_dir(),
                    host == "claude",
                )

    def test_each_host_projects_every_implemented_builtin_skill(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.install_host(host)
                root = self.discovery_root(target, host)
                for skill in PROJECTED_BUILTINS:
                    self.assertTrue((root / skill / "SKILL.md").is_file())
                    self.assertTrue(
                        (
                            target
                            / ".agents/skills"
                            / skill
                            / "SKILL.md"
                        ).is_file()
                    )

    def test_each_host_projects_the_automatic_dispatch_usage_gate(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.install_host(host)
                agents = (target / "AGENTS.md").read_text(encoding="utf-8")
                skill = (
                    self.discovery_root(target, host)
                    / "hive-usage-guard/SKILL.md"
                ).read_text(encoding="utf-8")
                for surface in (agents, skill):
                    self.assertIn("hive usage enforce", surface)
                    self.assertIn(
                        "Immediately before each new automatic dispatch",
                        surface,
                    )
                    self.assertIn(
                        "hive run resume --dispatch-intent automatic",
                        surface,
                    )
                    self.assertIn("enforced=true", surface)
                    self.assertIn("outcome=authorized", surface)
                    self.assertIn("exactly one dispatch brief", surface)
                    self.assertIn("never authorizes dispatch", surface)
                    self.assertIn("ordinary", surface)
                    self.assertIn("manual", surface)
                    self.assertIn("non-dispatch", surface)
                    self.assertIn("non-codex", surface.lower())
                    self.assertIn("fails closed", surface.lower())
                    self.assertIn("bare continue", surface.lower())
                    self.assertNotIn("At every turn boundary", surface)
                    self.assertNotIn("start a watcher", agents)
                self.assertIn("finite phrase list", agents)
                self.assertIn(
                    "illustrative rather than a finite phrase",
                    skill,
                )

    def test_catalog_only_skills_are_not_discoverable(self) -> None:
        target = self.install_host("codex")
        root = self.discovery_root(target, "codex")
        for skill in CATALOG_ONLY:
            with self.subTest(skill=skill):
                self.assertFalse((root / skill / "SKILL.md").exists())

    def test_host_projection_preserves_foreign_discovery_bytes(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.work_root / f"foreign-{host}"
                root = self.discovery_root(target, host)
                root.mkdir(parents=True)
                foreign = root / "foreign-skill/SKILL.md"
                foreign.parent.mkdir()
                foreign_bytes = b"---\nname: foreign-skill\n---\nforeign bytes\x00\xff\n"
                foreign.write_bytes(foreign_bytes)

                process, _ = self.invoke_setup(
                    target,
                    answers=self.answers_for_host(host),
                    capabilities=self.capability_for_host(host),
                )

                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertEqual(foreign.read_bytes(), foreign_bytes)

    def test_host_aliases_do_not_enter_canonical_hive_config(self) -> None:
        aliases = tuple(HOST_PATHS.values())
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.install_host(host)
                canonical = b"\n".join(
                    path.read_bytes()
                    for path in sorted((target / ".hive/config").glob("*"))
                    if path.is_file()
                    and path.name
                    not in {"project-base.json", "project-overrides.json"}
                )
                for alias in aliases:
                    self.assertNotIn(alias.encode("utf-8"), canonical)

    def test_projection_reapply_is_byte_identical_for_each_host(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.install_host(host)
                before = snapshot_tree(target)
                process, result = self.invoke_setup(
                    target,
                    answers=self.answers_for_host(host),
                    capabilities=self.capability_for_host(host),
                )
                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertEqual(result["changed_paths"], [])
                self.assertEqual(snapshot_tree(target), before)

    def test_available_external_runtime_projects_no_fallback_hook(self) -> None:
        for host in ("codex", "claude"):
            with self.subTest(host=host):
                target = self.install_host(host)
                self.assertFalse(
                    (target / ".hive/config/approved-hooks.yml").exists()
                )
                self.assertFalse((target / ".hive/hooks").exists())
                discovery_bytes = b"\n".join(
                    path.read_bytes()
                    for path in self.discovery_root(target, host).rglob("*")
                    if path.is_file()
                )
                self.assertNotIn(b"UserPromptSubmit", discovery_bytes)
                for skill in (
                    "hive-run-checkpoint",
                    "hive-run-resume",
                    "hive-role-handoff",
                ):
                    self.assertTrue(
                        (
                            self.discovery_root(target, host)
                            / skill
                            / "SKILL.md"
                        ).is_file()
                    )
                for forbidden_command in (
                    b"\nomx ",
                    b"\nomc ",
                    b"\nhive plan ",
                    b"\nhive team ",
                ):
                    self.assertNotIn(forbidden_command, discovery_bytes)


class Phase3OptionalSkillProjection(Phase3ProjectionTestCase):
    def make_consumer_with_local_source(self, name: str) -> tuple[Path, Path]:
        target = self.work_root / name
        source = target / "vendor-skills/local-inspect/SKILL.md"
        source.parent.mkdir(parents=True)
        source.write_bytes(LOCAL_SKILL_SOURCE.read_bytes())
        return target, source

    def signed_answers(self, source: Path, host: str = "codex") -> Path:
        answers = read_yaml(FIXTURE_ROOT / "answers-no-role-no-hook.yml")
        answers["primary_host"] = host
        payload = {
            "consent_version": 1,
            "name": "local-inspect",
            "source": "path:vendor-skills/local-inspect/SKILL.md",
            "revision": digest_bytes(source.read_bytes()),
            "content_digest": digest_bytes(source.read_bytes()),
            "requested_capabilities": ["filesystem-read"],
            "approved_capabilities": ["filesystem-read"],
            "approved_at": "2026-07-24T00:00:00Z",
        }
        answers["approved_optional_skills"] = [
            {**payload, "consent_digest": canonical_digest(payload)}
        ]
        path = self.work_root / f"approved-{host}.yml"
        write_yaml(path, answers)
        return path

    def test_unapproved_local_optional_skill_is_not_projected(self) -> None:
        target, source = self.make_consumer_with_local_source("unapproved")
        source_bytes = source.read_bytes()

        process, _ = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        projected = (
            self.discovery_root(target, "codex")
            / "local-inspect/SKILL.md"
        )
        self.assertFalse(projected.exists())
        self.assertEqual(source.read_bytes(), source_bytes)

    def test_approved_local_optional_skill_projects_exact_source_bytes(
        self,
    ) -> None:
        target, source = self.make_consumer_with_local_source("approved")
        source_bytes = source.read_bytes()

        process, result = self.invoke_setup(
            target,
            answers=self.signed_answers(source),
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        projected = (
            self.discovery_root(target, "codex")
            / "local-inspect/SKILL.md"
        )
        self.assertTrue(projected.is_file())
        self.assertEqual(projected.read_bytes(), source_bytes)
        self.assertIn(
            projected.relative_to(target).as_posix(),
            result["changed_paths"],
        )
        self.assertEqual(source.read_bytes(), source_bytes)
        ledger = read_yaml(target / ".hive/config/active-skills.yml")
        projected_entry = next(
            entry
            for entry in ledger["skills"]
            if entry["name"] == "local-inspect"
        )
        self.assertEqual(projected_entry["source_type"], "approved-optional")
        self.assertEqual(
            projected_entry["consent_digest"],
            read_yaml(self.signed_answers(source))[
                "approved_optional_skills"
            ][0]["consent_digest"],
        )

    def test_tampered_local_optional_source_is_not_projected(self) -> None:
        target, source = self.make_consumer_with_local_source("tampered")
        answers = self.signed_answers(source)
        source.write_text(
            source.read_text(encoding="utf-8") + "\ntampered\n",
            encoding="utf-8",
        )
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=answers,
            capabilities=self.capability_for_host("codex"),
        )

        self.assertIn(process.returncode, (2, 3))
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)


class Phase3FallbackHookExclusions(Phase3ProjectionTestCase):
    def test_current_external_runtime_makes_installed_hook_inert_before_input(
        self,
    ) -> None:
        if os.name == "nt" or not hasattr(os, "mkfifo"):
            self.skipTest("FIFO no-read evidence is POSIX-specific")
        target = self.work_root / "external-now-available"
        target.mkdir()
        install, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(install.returncode, 0, install.stderr)
        guarded_input = self.work_root / "must-not-read.fifo"
        os.mkfifo(guarded_input)

        try:
            process = subprocess.run(
                [
                    str(self.hive_binary),
                    "hook",
                    "--capability",
                    "protect-hive-owned-state",
                    "--event",
                    "PreToolUse",
                    "--capabilities",
                    str(
                        PHASE3_FIXTURES
                        / "capabilities-codex-enriched.json"
                    ),
                    "--input",
                    str(guarded_input),
                    "--output",
                    "json",
                ],
                cwd=target,
                check=False,
                text=True,
                capture_output=True,
                timeout=2.0,
            )
        except subprocess.TimeoutExpired as error:
            self.fail(f"inert hook read current input FIFO: {error}")

        self.assertEqual(process.returncode, 0, process.stderr)
        result = json.loads(process.stdout)
        self.assertEqual(result["decision"], "allow")
        self.assertFalse(result["active"])

    def test_external_runtime_reconfigure_removes_only_hive_hook_artifacts(
        self,
    ) -> None:
        target = self.work_root / "external-reconfigure"
        foreign = target / ".hive/hooks/foreign-entry.json"
        foreign.parent.mkdir(parents=True)
        foreign_bytes = b'{"foreign":true}\n'
        foreign.write_bytes(foreign_bytes)
        install, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-partial-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(install.returncode, 0, install.stderr)

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertFalse(
            (target / ".hive/config/approved-hooks.yml").exists()
        )
        self.assertFalse(
            (
                target
                / ".hive/hooks/protect-hive-owned-state"
            ).exists()
        )
        self.assertEqual(foreign.read_bytes(), foreign_bytes)
        self.assertNotIn(
            ".hive/hooks/foreign-entry.json",
            result["changed_paths"],
        )

    def test_projected_hook_descriptors_use_only_allowed_events(self) -> None:
        target = self.work_root / "hooks"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-all-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)

        allowed = {"PreToolUse", "PostToolUse", "PreCompact", "Stop"}
        for path in sorted((target / ".hive/hooks").iterdir()):
            with self.subTest(path=path):
                descriptor = json.loads(path.read_text(encoding="utf-8"))
                self.assertIn(descriptor["event"], allowed)
                self.assertNotEqual(descriptor["event"], "UserPromptSubmit")

    def test_projected_hook_commands_exclude_semantic_orchestration_actions(
        self,
    ) -> None:
        target = self.work_root / "hook-commands"
        target.mkdir()
        process, _ = self.invoke_setup(
            target,
            answers=FIXTURE_ROOT / "answers-all-hooks.yml",
            capabilities="capabilities-absent.json",
        )
        self.assertEqual(process.returncode, 0, process.stderr)

        forbidden = (
            "prompt-refine",
            "skill activate",
            "knowledge ingest",
            "subagent",
            "orchestrat",
            "continue",
        )
        for path in sorted((target / ".hive/hooks").iterdir()):
            command = str(
                json.loads(path.read_text(encoding="utf-8"))["command"]
            ).casefold()
            for token in forbidden:
                with self.subTest(path=path, token=token):
                    self.assertNotIn(token, command)


class Phase3ProjectionHostile(Phase3ProjectionTestCase):
    def test_discovery_root_symlink_is_rejected_without_external_write(
        self,
    ) -> None:
        if os.name == "nt":
            self.skipTest("portable unprivileged directory symlink unavailable")
        target = self.work_root / "symlink-root"
        target.mkdir()
        outside = self.work_root / "outside-discovery"
        outside.mkdir()
        sentinel = outside / "sentinel.bin"
        sentinel.write_bytes(b"outside user bytes\x00\xff\n")
        discovery = self.discovery_root(target, "codex")
        discovery.parent.mkdir(parents=True)
        discovery.symlink_to(outside, target_is_directory=True)
        target_before = snapshot_tree(target)
        outside_before = snapshot_tree(outside)

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(outside), outside_before)
        self.assertEqual(sentinel.read_bytes(), b"outside user bytes\x00\xff\n")

    def test_existing_projected_builtin_conflicts_without_overwrite(self) -> None:
        target = self.work_root / "builtin-conflict"
        projected = (
            self.discovery_root(target, "codex")
            / "hive-simple-question/SKILL.md"
        )
        projected.parent.mkdir(parents=True)
        user_bytes = b"user-owned colliding Skill bytes\x00\xff\n"
        projected.write_bytes(user_bytes)
        before = snapshot_tree(target)

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 3, process.stderr)
        self.assertEqual(result["status"], "conflict")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)
        self.assertEqual(projected.read_bytes(), user_bytes)

    def test_projection_activation_failure_rolls_back_host_tree(self) -> None:
        target = self.install_host("codex")
        before = snapshot_tree(target)
        answers = read_yaml(FIXTURE_ROOT / "answers-no-role-no-hook.yml")
        answers["primary_host"] = "codex"
        answers["project_name"] = "phase3-rollback-injection"
        answers_path = self.work_root / "rollback-answers.yml"
        write_yaml(answers_path, answers)

        process, result = self.invoke_setup(
            target,
            answers=answers_path,
            capabilities=self.capability_for_host("codex"),
            environment={"HIVE_TEST_ACTIVATION_FAIL_AFTER": "2"},
        )

        self.assertEqual(process.returncode, 10, process.stderr)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)

    def test_unknown_traversal_skill_approval_remains_inert(self) -> None:
        target = self.work_root / "unsafe-name"
        target.mkdir()
        answers = read_yaml(FIXTURE_ROOT / "answers-no-role-no-hook.yml")
        payload = {
            "consent_version": 1,
            "name": "../../outside",
            "source": "fixture:unknown",
            "revision": "immutable-revision",
            "content_digest": "sha256:" + "11" * 32,
            "requested_capabilities": [],
            "approved_capabilities": [],
            "approved_at": "2026-07-24T00:00:00Z",
        }
        answers["approved_optional_skills"] = [
            {**payload, "consent_digest": canonical_digest(payload)}
        ]
        answers_path = self.work_root / "unsafe-name-answers.yml"
        write_yaml(answers_path, answers)
        outside_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(
            target,
            answers=answers_path,
            capabilities=self.capability_for_host("codex"),
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertFalse((self.work_root / "outside").exists())
        self.assertFalse(
            (
                self.discovery_root(target, "codex")
                / "../../outside/SKILL.md"
            ).resolve().exists()
        )
        changed_outside_target = {
            path: value
            for path, value in snapshot_tree(self.work_root).items()
            if not path.startswith("unsafe-name/")
            and not path.startswith("user-root/")
            and path != "unsafe-name-answers.yml"
        }
        expected_outside = {
            path: value
            for path, value in outside_before.items()
            if not path.startswith("unsafe-name/")
            and not path.startswith("user-root/")
            and path != "unsafe-name-answers.yml"
        }
        self.assertEqual(changed_outside_target, expected_outside)
        self.assertNotIn("../", "\n".join(result["changed_paths"]))

    def test_enriched_capability_matrix_is_preserved_in_installed_config(
        self,
    ) -> None:
        target = self.work_root / "enriched-capability"
        target.mkdir()
        answers = self.answers_for_host("codex")
        capabilities = PHASE3_FIXTURES / "capabilities-codex-enriched.json"

        process, _ = self.invoke_setup(
            target,
            answers=answers,
            capabilities=capabilities,
        )

        self.assertEqual(process.returncode, 0, process.stderr)
        installed = read_yaml(
            target / ".hive/config/capability-resolution.yml"
        )
        self.assertEqual(
            installed["capabilities"]["automatic-skill-routing"],
            "supported",
        )
        self.assertEqual(
            installed["capabilities"]["prompt-refine"],
            "supported",
        )
        self.assertIn("hook_events", installed)


if __name__ == "__main__":
    import unittest

    unittest.main()
