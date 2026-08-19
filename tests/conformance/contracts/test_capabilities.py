#!/usr/bin/env python3
"""Host capability evidence and automatic-owner hostile conformance."""

from __future__ import annotations

import json

from tests.conformance.support.harness import (
    FIXTURE_ROOT,
    Phase1CliTestCase,
    snapshot_tree,
    write_yaml,
)


class Phase1CapabilityConformance(Phase1CliTestCase):
    def assert_owner(
        self,
        result: dict[str, object],
        owner: str,
    ) -> None:
        evidence = result["evidence"]
        self.assertIsInstance(evidence, list)
        self.assertIn(
            f"orchestration-owner:{owner}",
            {
                item["locator"]
                for item in evidence
                if isinstance(item, dict) and item.get("kind") == "report"
            },
        )

    def answers_for_host(self, host: str) -> object:
        path, answers = self.copied_answers("answers-no-role-no-hook.yml")
        answers["primary_host"] = host
        write_yaml(path, answers)
        return path

    def test_codex_catalog_compatibility_keeps_host_native_default(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities="capabilities-codex-host-native.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner(result, "host-native")

    def test_explicit_codex_external_selection_resolves_omx(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("codex"),
            capabilities="capabilities-codex-omx-executable.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner(result, "omx")

    def test_claude_catalog_compatibility_keeps_host_native_default(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("claude"),
            capabilities="capabilities-claude-host-native.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner(result, "host-native")

    def test_explicit_claude_external_selection_resolves_omc(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("claude"),
            capabilities="capabilities-claude-omc-executable.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner(result, "omc")

    def test_antigravity_absence_resolves_host_native_without_hooks(self) -> None:
        target = self.work_root / "consumer"
        target.mkdir()

        process, result = self.invoke_setup(
            target,
            answers=self.answers_for_host("antigravity"),
            capabilities="capabilities-antigravity-absent.json",
        )

        self.assertEqual(process.returncode, 0)
        self.assert_owner(result, "host-native")
        self.assertFalse((target / ".hive/config/approved-hooks.yml").exists())
        self.assertFalse((target / ".hive/hooks").exists())

    def assert_hostile_matrix_rejected(
        self,
        case_name: str,
        matrix: dict[str, object],
    ) -> None:
        target = self.work_root / "consumer"
        target.mkdir()
        capability_path = self.write_capability_case(case_name, matrix)
        answer_path = self.answers_for_host(str(matrix["host"]))
        target_before = snapshot_tree(target)
        outside_before = snapshot_tree(self.work_root)

        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities=capability_path,
        )

        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), target_before)
        self.assertEqual(snapshot_tree(self.work_root), outside_before)


HOSTILE_CAPABILITY_CASES = json.loads(
    (FIXTURE_ROOT / "capabilities-hostile.json").read_text(encoding="utf-8")
)


def make_hostile_capability_test(case: dict[str, object]):
    def test(self: Phase1CapabilityConformance) -> None:
        self.assert_hostile_matrix_rejected(
            str(case["name"]),
            case["matrix"],
        )

    test.__name__ = f"test_{str(case['name']).replace('-', '_')}_is_rejected"
    return test


for hostile_case in HOSTILE_CAPABILITY_CASES:
    test_name = f"test_{str(hostile_case['name']).replace('-', '_')}_is_rejected"
    setattr(
        Phase1CapabilityConformance,
        test_name,
        make_hostile_capability_test(hostile_case),
    )
