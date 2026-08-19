#!/usr/bin/env python3
"""Validate deterministic Phase 1 fixture invariants independently of the CLI."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
FIXTURE_ROOT = REPOSITORY_ROOT / "tests/fixtures/setup"
SCHEMA_ROOT = REPOSITORY_ROOT / "schemas"
EXPECTED_ROOT = FIXTURE_ROOT / "expected"
VALID_CAPABILITY_DIGESTS = {
    "capabilities-codex-host-native-hooks.json": "sha256:8b260a7698068a4ec113bb1f60dfc84147647f8b2bae6a5de1fd852a79eea833",
    "capabilities-absent.json": "sha256:ff554b1b02ac93d7112f69022268466caed8d75229fc7bd0f672071f98c8376a",
    "capabilities-antigravity-absent.json": "sha256:f302144ed6a975b85a8a6c62ab39918070ad9168364ea570c1006a474664b7d8",
    "capabilities-claude-host-native.json": "sha256:6a9a4f9c08c86f15e64000541b38964717616da70994ecf69fed8c35bf6d9e80",
    "capabilities-claude-omc-executable.json": "sha256:b9a73552fe6b7eb91663b965d123ac71765e42309553f22352009f2df5de4ed8",
    "capabilities-claude-omc.json": "sha256:9519678e71925179bc93e4fabc7686d5595da3e456de6a850a9ac10179a76cb0",
    "capabilities-codex-host-native.json": "sha256:0218bcf5a147a6cbfe3f5fb2fd5bea3053db5f99c37abcaf833a27543546a4e2",
    "capabilities-codex-omx-executable.json": "sha256:4c4d9f4ea469ee3c16bac54555ac1bebfc7a01adc9764033db471cb5a6472656",
    "capabilities-codex-omx.json": "sha256:a4f187054f8152eabb31990e470fee0ec849bd3ddeac2dbd071a1c6e0eaf3d81",
    "capabilities-incompatible.json": "sha256:0f66adcb318fe2076c75931dfdef897a4f91ca11f9a555d96b0134f7056ea7c6",
    "capabilities-unknown.json": "sha256:4c5cb33a3f3e73ba3bc3b210e4be5a0c9ed8001e7a4caee69135f17e156d9eb4",
}


def read_yaml(path: Path) -> dict[str, object]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def sha256_digest(value: bytes) -> str:
    return f"sha256:{hashlib.sha256(value).hexdigest()}"


class Phase1FixtureConformance(unittest.TestCase):
    def test_current_setup_answer_fixtures_satisfy_the_machine_schema(self) -> None:
        schema = json.loads(
            (SCHEMA_ROOT / "setup-answers.schema.json").read_text(encoding="utf-8")
        )
        validator = Draft202012Validator(schema, format_checker=FormatChecker())

        for fixture_name in (
            "answers-base.yml",
            "answers-no-role-no-hook.yml",
            "answers-partial-hooks.yml",
            "answers-all-hooks.yml",
            "answers-approved-skill.yml",
        ):
            with self.subTest(fixture=fixture_name):
                validator.validate(read_yaml(FIXTURE_ROOT / fixture_name))

    def test_current_setup_answers_do_not_contain_orchestration_preference(self) -> None:
        current_fixtures = (
            "answers-base.yml",
            "answers-no-role-no-hook.yml",
            "answers-partial-hooks.yml",
            "answers-approved-skill.yml",
        )

        for fixture_name in current_fixtures:
            with self.subTest(fixture=fixture_name):
                answers = read_yaml(FIXTURE_ROOT / fixture_name)
                self.assertNotIn("orchestration_layer", answers)
                self.assertNotIn("capability_resolution", answers)

    def test_hook_consent_digests_bind_every_preview_field(self) -> None:
        answers = read_yaml(FIXTURE_ROOT / "answers-partial-hooks.yml")
        approvals = answers["approved_fallback_hooks"]
        self.assertIsInstance(approvals, list)
        literal_preimages = json.loads(
            (EXPECTED_ROOT / "hook-consent-preimages.json").read_text(
                encoding="utf-8"
            )
        )

        for approval in approvals:
            with self.subTest(capability=approval["capability"]):
                preimage = literal_preimages[approval["path"]].encode("utf-8")
                payload = json.loads(preimage)
                self.assertEqual(
                    payload,
                    {
                        key: value
                        for key, value in approval.items()
                        if key != "consent_digest"
                    },
                )
                self.assertEqual(approval["consent_digest"], sha256_digest(preimage))

    def test_optional_skill_consent_digest_binds_subset_approval(self) -> None:
        answers = read_yaml(FIXTURE_ROOT / "answers-approved-skill.yml")
        approvals = answers["approved_optional_skills"]
        self.assertIsInstance(approvals, list)
        approval = approvals[0]
        preimage = (
            EXPECTED_ROOT / "skill-consent-preimage.json"
        ).read_bytes().removesuffix(b"\n")
        expected_digest = (
            EXPECTED_ROOT / "skill-consent-digest.txt"
        ).read_text(encoding="utf-8").strip()

        self.assertEqual(
            json.loads(preimage),
            {
                key: value
                for key, value in approval.items()
                if key != "consent_digest"
            },
        )
        self.assertEqual(approval["consent_digest"], expected_digest)
        self.assertEqual(expected_digest, sha256_digest(preimage))
        self.assertTrue(
            set(approval["approved_capabilities"])
            <= set(approval["requested_capabilities"])
        )

    def test_hook_content_digests_match_literal_descriptor_bytes(self) -> None:
        answers = read_yaml(FIXTURE_ROOT / "answers-partial-hooks.yml")
        approvals = answers["approved_fallback_hooks"]
        expected_digests = json.loads(
            (EXPECTED_ROOT / "hook-descriptor-digests.json").read_text(
                encoding="utf-8"
            )
        )

        for approval in approvals:
            with self.subTest(capability=approval["capability"]):
                descriptor = EXPECTED_ROOT / (
                    "hook-checkpoint-reminder-stop.json"
                    if approval["capability"] == "checkpoint-reminder"
                    else "hook-protect-hive-owned-state.json"
                )
                self.assertEqual(
                    approval["content_digest"],
                    expected_digests[approval["path"]],
                )
                self.assertEqual(
                    approval["content_digest"],
                    sha256_digest(descriptor.read_bytes()),
                )

    def test_absent_evidence_proves_both_catalog_and_executable_absence(self) -> None:
        capability_input = json.loads(
            (FIXTURE_ROOT / "capabilities-absent.json").read_text(encoding="utf-8")
        )

        observed = {
            (item["source"], item["outcome"])
            for item in capability_input["evidence"]
        }
        self.assertEqual(
            observed,
            {
                ("host-catalog", "absent"),
                ("public-executable", "absent"),
            },
        )

    def test_available_evidence_supports_host_native_default_and_matching_external_selection(self) -> None:
        fixtures = (
            ("capabilities-codex-host-native.json", "codex", "omx", "host-native"),
            ("capabilities-codex-omx.json", "codex", "omx"),
            ("capabilities-claude-host-native.json", "claude", "omc", "host-native"),
            ("capabilities-claude-omc.json", "claude", "omc"),
        )

        for fixture in fixtures:
            fixture_name, expected_host, expected_runtime, *owner = fixture
            expected_owner = owner[0] if owner else expected_runtime
            with self.subTest(fixture=fixture_name):
                capability_input = json.loads(
                    (FIXTURE_ROOT / fixture_name).read_text(encoding="utf-8")
                )
                self.assertEqual(capability_input["detection"], "available")
                self.assertEqual(capability_input["host"], expected_host)
                self.assertEqual(
                    capability_input["external_runtime"],
                    expected_runtime,
                )
                self.assertEqual(
                    capability_input["resolved_owner"],
                    expected_owner,
                )

    def test_capability_evidence_digest_binds_the_full_resolution_object(
        self,
    ) -> None:
        for fixture_name, expected_digest in VALID_CAPABILITY_DIGESTS.items():
            with self.subTest(fixture=fixture_name):
                capability_input = json.loads(
                    (FIXTURE_ROOT / fixture_name).read_text(encoding="utf-8")
                )
                self.assertEqual(
                    capability_input["evidence_digest"],
                    expected_digest,
                )

    def test_capability_inputs_satisfy_the_machine_schema(self) -> None:
        schema = json.loads(
            (SCHEMA_ROOT / "capability-matrix.schema.json").read_text(
                encoding="utf-8"
            )
        )
        validator = Draft202012Validator(schema, format_checker=FormatChecker())

        for fixture_name in VALID_CAPABILITY_DIGESTS:
            with self.subTest(fixture=fixture_name):
                validator.validate(
                    json.loads(
                        (FIXTURE_ROOT / fixture_name).read_text(encoding="utf-8")
                    )
                )

    def test_hostile_capability_matrices_are_rejected_by_the_machine_schema(
        self,
    ) -> None:
        schema = json.loads(
            (SCHEMA_ROOT / "capability-matrix.schema.json").read_text(
                encoding="utf-8"
            )
        )
        validator = Draft202012Validator(schema, format_checker=FormatChecker())
        hostile_cases = json.loads(
            (FIXTURE_ROOT / "capabilities-hostile.json").read_text(encoding="utf-8")
        )

        for case in hostile_cases:
            with self.subTest(case=case["name"]):
                self.assertTrue(list(validator.iter_errors(case["matrix"])))

    def test_supported_host_native_hook_ledger_satisfies_the_machine_schema(self) -> None:
        schema = json.loads(
            (SCHEMA_ROOT / "hook-consent.schema.json").read_text(encoding="utf-8")
        )
        answers = read_yaml(FIXTURE_ROOT / "answers-partial-hooks.yml")
        capability_input = json.loads(
            (
                FIXTURE_ROOT / "capabilities-codex-host-native-hooks.json"
            ).read_text(encoding="utf-8")
        )
        ledger = {
            "schema_version": 1,
            "detection": capability_input["detection"],
            "resolution_evidence_digest": capability_input["evidence_digest"],
            "hooks": answers["approved_fallback_hooks"],
        }

        Draft202012Validator(
            schema,
            format_checker=FormatChecker(),
        ).validate(ledger)

    def test_non_absent_evidence_cannot_be_mistaken_for_absence(self) -> None:
        fixtures = (
            "capabilities-codex-omx.json",
            "capabilities-claude-omc.json",
            "capabilities-incompatible.json",
            "capabilities-unknown.json",
        )

        for fixture_name in fixtures:
            with self.subTest(fixture=fixture_name):
                capability_input = json.loads(
                    (FIXTURE_ROOT / fixture_name).read_text(encoding="utf-8")
                )
                outcomes = {
                    item["outcome"] for item in capability_input["evidence"]
                }
                self.assertFalse(
                    capability_input["detection"] == "absent"
                    or outcomes
                    == {
                        "absent",
                    }
                )

    def test_rfc8785_unicode_and_number_known_answer_uses_literal_bytes(
        self,
    ) -> None:
        fixture = json.loads(
            (FIXTURE_ROOT / "rfc8785-known-answer.json").read_text(encoding="utf-8")
        )
        canonical = bytes.fromhex(fixture["canonical_utf8_hex"])

        self.assertEqual(
            canonical,
            (
                b'{"literals":[null,true,false],"numbers":'
                b'[333333333.3333333,1e+30,4.5,0.002,1e-27],"string":"'
                + "€".encode()
                + b"$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}"
            ),
        )
        self.assertEqual(fixture["sha256"], sha256_digest(canonical))


if __name__ == "__main__":
    unittest.main()
