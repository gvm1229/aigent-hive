#!/usr/bin/env python3
"""Phase 5 hostile judge package and deterministic quorum conformance."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from typing import Any

import yaml
from jsonschema import Draft202012Validator, FormatChecker, ValidationError


ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "tests/fixtures/phase5/judge"
SCHEMAS = ROOT / "schemas"
SKILL = ROOT / "harness/skills/hive-judge-package/SKILL.md"
PROJECTED_SKILL = (
    ROOT
    / "harness/template/.agents/skills/hive-judge-package/SKILL.md"
)
CATALOG = ROOT / "harness/skills/catalog.yml"
ACTIVE_SKILLS = ROOT / "harness/template/.hive/config/active-skills.yml"
FORBIDDEN_CONTEXT_FIELDS = (
    "chain_of_thought",
    "reasoning",
    "self_score",
    "self_praise",
    "desired_verdict",
    "other_judge_verdicts",
)
TARGET_FILE_BYTES = {
    "artifact/patch.diff": b"diff --git a/src b/src\n",
    "evidence/tests.json": b'{"passed":18,"failed":0}\n',
    "artifact/activation.diff": b"security activation patch\n",
    "evidence/hostile-tests.json": b'{"passed":9,"failed":0}\n',
}


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object: {path}")
    return value


def read_yaml(path: Path) -> dict[str, Any]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def package_digest(package: dict[str, Any]) -> str:
    payload = {key: value for key, value in package.items() if key != "package_digest"}
    encoded = json.dumps(
        payload,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest_bytes(encoded)


def artifact_digest(artifact: dict[str, Any], digest_field: str) -> str:
    payload = {key: value for key, value in artifact.items() if key != digest_field}
    encoded = json.dumps(
        payload,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest_bytes(encoded)


def trust_root_digest(trust_root: dict[str, Any]) -> str:
    payload = {
        key: value for key, value in trust_root.items() if key != "root_digest"
    }
    encoded = json.dumps(
        payload,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest_bytes(encoded)


def snapshot_tree(root: Path) -> dict[str, tuple[str, bytes | str]]:
    snapshot: dict[str, tuple[str, bytes | str]] = {}
    for current, directories, files in os.walk(root, followlinks=False):
        directories.sort()
        files.sort()
        current_path = Path(current)
        for name in [*directories, *files]:
            path = current_path / name
            relative = path.relative_to(root).as_posix()
            if path.is_symlink():
                snapshot[relative] = ("symlink", os.readlink(path))
            elif path.is_dir():
                snapshot[relative] = ("directory", "")
            else:
                snapshot[relative] = ("file", path.read_bytes())
    return snapshot


class Phase5JudgeStaticContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.package_schema = read_json(SCHEMAS / "judge-package.schema.json")
        cls.verdict_schema = read_json(SCHEMAS / "judge-verdict.schema.json")
        cls.assignment_schema = read_json(SCHEMAS / "judge-assignment.schema.json")
        cls.approval_schema = read_json(SCHEMAS / "judge-approval.schema.json")
        cls.attestation_schema = read_json(
            SCHEMAS / "judge-attestation.schema.json"
        )
        cls.trust_root_schema = read_json(
            SCHEMAS / "judge-trust-root.schema.json"
        )
        cls.package_request_schema = read_json(
            SCHEMAS / "judge-package-request.schema.json"
        )
        cls.quorum_request_schema = read_json(
            SCHEMAS / "judge-quorum-request.schema.json"
        )
        cls.package_validator = Draft202012Validator(
            cls.package_schema,
            format_checker=FormatChecker(),
        )
        cls.verdict_validator = Draft202012Validator(
            cls.verdict_schema,
            format_checker=FormatChecker(),
        )
        cls.assignment_validator = Draft202012Validator(
            cls.assignment_schema,
            format_checker=FormatChecker(),
        )
        cls.approval_validator = Draft202012Validator(
            cls.approval_schema,
            format_checker=FormatChecker(),
        )
        cls.attestation_validator = Draft202012Validator(cls.attestation_schema)
        cls.trust_root_validator = Draft202012Validator(
            cls.trust_root_schema,
            format_checker=FormatChecker(),
        )
        cls.package_request_validator = Draft202012Validator(
            cls.package_request_schema,
            format_checker=FormatChecker(),
        )
        cls.quorum_request_validator = Draft202012Validator(
            cls.quorum_request_schema,
            format_checker=FormatChecker(),
        )

    def test_package_request_fixtures_conform_to_request_schema(self) -> None:
        valid_requests = (
            "package-request-clean.json",
            "package-request-contaminated-values.json",
            "package-request-artifact-digest-mismatch.json",
            "package-request-missing-artifact.json",
        )
        for name in valid_requests:
            with self.subTest(name=name):
                self.package_request_validator.validate(read_json(FIXTURES / name))

    def test_unknown_clean_context_fields_are_invalid_request_shape(self) -> None:
        hostile = read_json(FIXTURES / "package-request-hostile.json")
        with self.assertRaises(ValidationError):
            self.package_request_validator.validate(hostile)

    def test_quorum_request_fixtures_conform_to_request_schema(self) -> None:
        for request_path in sorted(FIXTURES.glob("quorum-*.json")):
            with self.subTest(name=request_path.name):
                self.quorum_request_validator.validate(read_json(request_path))

    def test_valid_packages_conform_to_judge_package_schema(self) -> None:
        for name in ("package-elevated.json", "package-critical.json"):
            with self.subTest(name=name):
                package = read_json(FIXTURES / name)
                self.package_validator.validate(package)

    def test_valid_packages_match_their_exact_canonical_digest(self) -> None:
        for name in ("package-elevated.json", "package-critical.json"):
            with self.subTest(name=name):
                package = read_json(FIXTURES / name)
                self.assertEqual(package["package_digest"], package_digest(package))

    def test_package_references_match_exact_target_file_digests(self) -> None:
        for name in ("package-elevated.json", "package-critical.json"):
            package = read_json(FIXTURES / name)
            for locator in [*package["artifact_refs"], *package["evidence_refs"]]:
                relative, separator, expected_digest = locator.partition("#")
                with self.subTest(package=name, relative=relative):
                    self.assertEqual(separator, "#")
                    self.assertIn(relative, TARGET_FILE_BYTES)
                    self.assertEqual(
                        expected_digest,
                        digest_bytes(TARGET_FILE_BYTES[relative]),
                    )

    def test_package_schema_rejects_task_agent_chain_of_thought(self) -> None:
        self.assert_package_rejects_hostile_field("chain_of_thought")

    def test_package_schema_rejects_task_agent_reasoning(self) -> None:
        self.assert_package_rejects_hostile_field("reasoning")

    def test_package_schema_rejects_task_agent_self_score(self) -> None:
        self.assert_package_rejects_hostile_field("self_score")

    def test_package_schema_rejects_task_agent_self_praise(self) -> None:
        self.assert_package_rejects_hostile_field("self_praise")

    def test_package_schema_rejects_desired_verdict(self) -> None:
        self.assert_package_rejects_hostile_field("desired_verdict")

    def test_package_schema_rejects_other_judge_verdicts(self) -> None:
        self.assert_package_rejects_hostile_field("other_judge_verdicts")

    def assert_package_rejects_hostile_field(self, field: str) -> None:
        clean = read_json(FIXTURES / "package-elevated.json")
        hostile = read_json(FIXTURES / "package-request-hostile.json")
        contaminated = copy.deepcopy(clean)
        contaminated[field] = hostile[field]
        with self.assertRaises(ValidationError):
            self.package_validator.validate(contaminated)

    def test_assignment_roster_has_distinct_non_self_reviewing_slots(self) -> None:
        assignment = read_json(FIXTURES / "assignment-elevated.json")
        self.assignment_validator.validate(assignment)
        slots = assignment["slots"]
        self.assertEqual(len({slot["slot_id"] for slot in slots}), 3)
        self.assertEqual(len({slot["judge_instance_id"] for slot in slots}), 3)
        excluded = {assignment["requester_id"], assignment["task_agent_id"]}
        self.assertTrue(
            all(slot["judge_instance_id"] not in excluded for slot in slots)
        )

    def test_human_approval_is_separate_digest_bound_artifact(self) -> None:
        approval = read_json(FIXTURES / "approval-critical.json")
        self.approval_validator.validate(approval)
        self.assertEqual(approval["decision"], "APPROVE")
        self.assertNotIn("human_approved", approval)

    def test_external_trust_root_contains_public_keys_only(self) -> None:
        trust_root = tomllib.loads(
            (FIXTURES / "trust-root.toml").read_text(encoding="utf-8")
        )
        self.trust_root_validator.validate(trust_root)
        encoded = json.dumps(trust_root, sort_keys=True).casefold()
        for prohibited in ("private_key", "secret_key", "seed", "pkcs8", "pem"):
            with self.subTest(prohibited=prohibited):
                self.assertNotIn(prohibited, encoded)
        public_keys = [key["public_key"] for key in trust_root["keys"]]
        self.assertEqual(len(public_keys), len(set(public_keys)))
        self.assertEqual(trust_root["root_digest"], trust_root_digest(trust_root))

    def test_trust_root_rejects_impossible_calendar_timestamp(self) -> None:
        trust_root = tomllib.loads(
            (FIXTURES / "trust-root.toml").read_text(encoding="utf-8")
        )
        trust_root["keys"][0]["valid_until"] = "2026-99-31T23:59:59Z"
        with self.assertRaises(ValidationError):
            self.trust_root_validator.validate(trust_root)

    def test_detached_attestations_use_closed_ed25519_contract(self) -> None:
        paths = sorted(FIXTURES.glob("*-attestation.json"))
        self.assertEqual(len(paths), 5)
        for path in paths:
            with self.subTest(name=path.name):
                attestation = read_json(path)
                self.attestation_validator.validate(attestation)
                self.assertNotIn("public_key", attestation)

    def test_pass_with_missing_evidence_is_schema_invalid(self) -> None:
        invalid = read_json(
            FIXTURES / "verdict-invalid-pass-with-missing-evidence.json"
        )
        with self.assertRaises(ValidationError):
            self.verdict_validator.validate(invalid)

    def test_indeterminate_with_missing_evidence_is_schema_valid(self) -> None:
        verdict = read_json(FIXTURES / "verdict-elevated-indeterminate.json")
        self.verdict_validator.validate(verdict)
        self.assertEqual(verdict["verdict"], "INDETERMINATE")

    def test_all_quorum_verdict_fixtures_conform_to_verdict_schema(self) -> None:
        for request_path in sorted(FIXTURES.glob("quorum-*.json")):
            request = read_json(request_path)
            for relative in request["verdicts"]:
                verdict = read_json(FIXTURES / Path(relative).name)
                self.verdict_validator.validate(verdict)

    def test_all_quorum_verdict_fixtures_match_their_subject(self) -> None:
        for request_path in sorted(FIXTURES.glob("quorum-*.json")):
            request = read_json(request_path)
            package = read_json(FIXTURES / Path(request["package"]).name)
            for relative in request["verdicts"]:
                verdict = read_json(FIXTURES / Path(relative).name)
                self.assertEqual(verdict["subject_id"], package["subject_id"])

    def test_judge_skill_projection_is_byte_identical(self) -> None:
        self.assertEqual(PROJECTED_SKILL.read_bytes(), SKILL.read_bytes())

    def test_judge_skill_is_implemented_read_only_builtin(self) -> None:
        catalog = read_yaml(CATALOG)
        entry = next(
            item for item in catalog["skills"] if item["name"] == "hive-judge-package"
        )
        self.assertEqual(entry["availability"], "implemented")
        self.assertEqual(entry["side_effect_class"], "read-only")
        self.assertNotIn("filesystem-write", entry["capabilities"])
        self.assertNotIn("subagents", entry["capabilities"])

    def test_active_skill_digest_matches_canonical_source_bytes(self) -> None:
        ledger = read_yaml(ACTIVE_SKILLS)
        entry = next(
            item for item in ledger["skills"] if item["name"] == "hive-judge-package"
        )
        self.assertEqual(entry["content_digest"], digest_bytes(SKILL.read_bytes()))
        self.assertEqual(entry["source_type"], "built-in")
        self.assertIsNone(entry["consent_digest"])

    def test_judge_skill_preserves_simple_question_and_external_owner_precedence(
        self,
    ) -> None:
        text = SKILL.read_text(encoding="utf-8").casefold()
        self.assertIn("simple-question gate first", text)
        self.assertIn("preferred orchestration owner", text)
        command_lines = tuple(
            line.strip()
            for line in text.splitlines()
            if line.strip().startswith(("omx ", "omc "))
        )
        self.assertEqual(command_lines, ())

    def test_judge_skill_has_no_execution_or_completion_authority(self) -> None:
        text = SKILL.read_text(encoding="utf-8").casefold()
        required_boundaries = (
            "does not invoke a judge",
            "never call a model",
            "spawn a subagent",
            "aggregate verdicts",
            "authorize completion",
            "task agent judge or approve its own result",
        )
        for boundary in required_boundaries:
            with self.subTest(boundary=boundary):
                self.assertIn(boundary, text)


class Phase5JudgeCliContracts(unittest.TestCase):
    """Executable expectations; intentionally RED until the Phase 5 CLI lands."""

    @classmethod
    def setUpClass(cls) -> None:
        configured = os.environ.get("HIVE_BIN")
        cls.binary = (
            Path(configured).resolve()
            if configured
            else (ROOT / "target/debug/hive").resolve()
        )
        cls.action_validator = Draft202012Validator(
            read_json(SCHEMAS / "action-result.schema.json"),
            format_checker=FormatChecker(),
        )
        cls.package_validator = Draft202012Validator(
            read_json(SCHEMAS / "judge-package.schema.json"),
            format_checker=FormatChecker(),
        )

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(
            prefix="hive-phase5-judge-",
            dir=str(ROOT / "tests"),
        )
        self.work = Path(self.temporary.name).resolve()
        self.target = self.work / "consumer"
        self.target.mkdir()
        (self.target / "foreign.txt").write_bytes(b"foreign bytes must remain unchanged\n")
        for relative, content in TARGET_FILE_BYTES.items():
            path = self.target / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        judge_dir = self.target / "judge"
        judge_dir.mkdir()
        for fixture in FIXTURES.glob("*.json"):
            (judge_dir / fixture.name).write_bytes(fixture.read_bytes())
        self.before = snapshot_tree(self.target)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def run_judge(
        self,
        operation: str,
        request: Path,
        trust_root: Path | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        command = [
            str(self.binary),
            "judge",
            operation,
            "--target",
            str(self.target),
            "--request",
            (
                request.as_posix()
                if not request.is_absolute()
                else f"judge/{request.name}"
            ),
        ]
        if trust_root is not None:
            command.extend(("--trust-root", str(trust_root)))
        command.extend(("--output", "json"))
        process = subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            text=True,
            capture_output=True,
            timeout=15.0,
        )
        try:
            payload = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"judge CLI did not return one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.action_validator.validate(payload)
        self.assertEqual(payload["action"], "VerifyWork")
        self.assertEqual(payload["exit_code"], process.returncode)
        self.assertEqual(payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.target), self.before)
        return process, payload

    def write_judge_json(self, name: str, value: dict[str, Any]) -> Path:
        relative = Path("judge") / name
        (self.target / relative).write_text(
            json.dumps(value, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )
        self.before = snapshot_tree(self.target)
        return relative

    def test_package_returns_exact_digest_bound_clean_envelope(self) -> None:
        process, payload = self.run_judge(
            "package",
            FIXTURES / "package-request-clean.json",
        )
        self.assertEqual(process.returncode, 0, payload)
        package = payload["data"]["package"]
        self.package_validator.validate(package)
        self.assertEqual(package, read_json(FIXTURES / "package-elevated.json"))
        self.assertEqual(package["package_digest"], package_digest(package))

    def test_package_rejects_forbidden_context_without_writing(self) -> None:
        process, payload = self.run_judge(
            "package",
            FIXTURES / "package-request-hostile.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotEqual(payload["status"], "success")

    def test_package_rejects_contamination_hidden_in_allowed_values(self) -> None:
        process, payload = self.run_judge(
            "package",
            FIXTURES / "package-request-contaminated-values.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotEqual(payload["status"], "success")

    def test_package_rejects_target_artifact_digest_mismatch(self) -> None:
        process, payload = self.run_judge(
            "package",
            FIXTURES / "package-request-artifact-digest-mismatch.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotEqual(payload["status"], "success")

    def test_package_rejects_missing_target_artifact(self) -> None:
        process, payload = self.run_judge(
            "package",
            FIXTURES / "package-request-missing-artifact.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotEqual(payload["status"], "success")

    def test_unsigned_elevated_quorum_has_no_completion_authority(self) -> None:
        process, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-elevated-two-of-three.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        quorum = payload["data"]["quorum"]
        self.assertEqual(quorum["result"], "INDETERMINATE")
        self.assertFalse(quorum["authenticated"])
        self.assertEqual(quorum["authentication"], "none")

    def test_quorum_output_does_not_leak_individual_verdicts(self) -> None:
        _, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-elevated-two-of-three.json",
        )
        encoded = json.dumps(payload, sort_keys=True)
        for prohibited in (
            "judge-instance",
            "slot-",
            "findings",
            "verdicts",
            "sha256:",
            "APPROVE",
        ):
            with self.subTest(prohibited=prohibited):
                self.assertNotIn(prohibited, encoded)

    def test_duplicate_judge_identity_does_not_satisfy_elevated_quorum(self) -> None:
        _, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-elevated-duplicate-judge.json",
        )
        quorum = payload["data"]["quorum"]
        self.assertNotEqual(quorum["result"], "PASS")
        self.assertLess(quorum["eligible_count"], 3)

    def test_digest_mismatched_verdict_is_excluded_from_quorum(self) -> None:
        _, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-elevated-digest-mismatch.json",
        )
        quorum = payload["data"]["quorum"]
        self.assertNotEqual(quorum["result"], "PASS")
        self.assertEqual(quorum["excluded_count"], 1)

    def test_missing_evidence_cannot_be_counted_as_pass(self) -> None:
        _, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-elevated-missing-evidence.json",
        )
        quorum = payload["data"]["quorum"]
        self.assertNotEqual(quorum["result"], "PASS")
        self.assertEqual(quorum["pass_count"], 1)
        self.assertEqual(quorum["indeterminate_count"], 1)

    def test_critical_unanimous_judges_without_human_approval_do_not_pass(
        self,
    ) -> None:
        _, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-critical-no-human.json",
        )
        quorum = payload["data"]["quorum"]
        self.assertNotEqual(quorum["result"], "PASS")
        self.assertFalse(quorum["approval_valid"])

    def test_unsigned_critical_human_approval_has_no_completion_authority(self) -> None:
        process, payload = self.run_judge(
            "quorum",
            FIXTURES / "quorum-critical-with-human.json",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        quorum = payload["data"]["quorum"]
        self.assertEqual(quorum["result"], "INDETERMINATE")
        self.assertFalse(quorum["authenticated"])
        self.assertFalse(quorum["approval_valid"])

    def test_arbitrary_identities_and_bare_true_cannot_authorize_pass(self) -> None:
        request = read_json(FIXTURES / "quorum-critical-no-human.json")
        request["human_approved"] = True
        request_path = self.write_judge_json("quorum-bare-true.json", request)
        process, payload = self.run_judge("quorum", request_path)
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotIn("data", payload)

    def test_self_authored_identity_forgery_cannot_authorize_critical_pass(
        self,
    ) -> None:
        assignment = read_json(FIXTURES / "assignment-critical.json")
        assignment["requester_id"] = "forged-requester"
        assignment["task_agent_id"] = "decoy-task-agent"
        assignment["resolved_owner_id"] = "forged-owner"
        assignment["owner_provenance"]["authentication_evidence_digest"] = (
            "sha256:" + ("f" * 64)
        )
        for index, slot in enumerate(assignment["slots"], start=1):
            slot["judge_instance_id"] = f"forged-judge-{index}"
            slot["eligibility_evidence_digest"] = "sha256:" + (str(index) * 64)
        assignment["assignment_digest"] = artifact_digest(
            assignment, "assignment_digest"
        )
        self.write_judge_json("assignment-self-authored.json", assignment)

        verdict_paths: list[str] = []
        for index, fixture_name in enumerate(
            (
                "verdict-critical-pass-a.json",
                "verdict-critical-pass-b.json",
                "verdict-critical-pass-c.json",
            ),
            start=1,
        ):
            verdict = read_json(FIXTURES / fixture_name)
            slot = assignment["slots"][index - 1]
            verdict["assignment_digest"] = assignment["assignment_digest"]
            verdict["slot_id"] = slot["slot_id"]
            verdict["judge_instance_id"] = slot["judge_instance_id"]
            verdict["eligibility_evidence_digest"] = slot[
                "eligibility_evidence_digest"
            ]
            verdict_name = f"verdict-self-authored-{index}.json"
            self.write_judge_json(verdict_name, verdict)
            verdict_paths.append(f"judge/{verdict_name}")

        approval = read_json(FIXTURES / "approval-critical.json")
        approval["assignment_digest"] = assignment["assignment_digest"]
        approval["approver_id"] = "forged-human"
        approval["approval_digest"] = artifact_digest(approval, "approval_digest")
        self.write_judge_json("approval-self-authored.json", approval)

        request = {
            "schema_version": 2,
            "package": "judge/package-critical.json",
            "assignment": "judge/assignment-self-authored.json",
            "assignment_attestation": "judge/assignment-critical-attestation.json",
            "verdicts": verdict_paths,
            "verdict_attestations": [
                "judge/verdict-critical-pass-a-attestation.json",
                "judge/verdict-critical-pass-b-attestation.json",
                "judge/verdict-critical-pass-c-attestation.json",
            ],
            "approval": "judge/approval-self-authored.json",
            "approval_attestation": "judge/approval-critical-attestation.json",
        }
        request_path = self.write_judge_json("quorum-self-authored.json", request)
        process, payload = self.run_judge(
            "quorum",
            request_path,
            trust_root=FIXTURES / "trust-root.toml",
        )
        self.assertNotEqual(process.returncode, 0, payload)
        self.assertNotEqual(payload.get("data", {}).get("quorum", {}).get("result"), "PASS")

    def test_target_or_caller_writable_trust_root_is_blocked(self) -> None:
        request = FIXTURES / "quorum-critical-authenticated.json"
        target_root = self.target / "judge/trust-root.toml"
        target_root.write_bytes((FIXTURES / "trust-root.toml").read_bytes())
        self.before = snapshot_tree(self.target)
        for trust_root in (target_root, FIXTURES / "trust-root.toml"):
            with self.subTest(trust_root=trust_root):
                process, payload = self.run_judge(
                    "quorum",
                    request,
                    trust_root=trust_root,
                )
                self.assertEqual(process.returncode, 3, payload)
                self.assertEqual(payload["status"], "blocked")
                self.assertNotIn("data", payload)

    def test_assignment_and_approval_tamper_fail_closed(self) -> None:
        assignment = read_json(FIXTURES / "assignment-critical.json")
        assignment["resolved_owner_id"] = "tampered-owner"
        self.write_judge_json("assignment-tampered.json", assignment)
        request = read_json(FIXTURES / "quorum-critical-with-human.json")
        request["assignment"] = "judge/assignment-tampered.json"
        request_path = self.write_judge_json("quorum-assignment-tampered.json", request)
        process, payload = self.run_judge("quorum", request_path)
        self.assertNotEqual(process.returncode, 0, payload)

        approval = read_json(FIXTURES / "approval-critical.json")
        approval["approver_id"] = "different-human"
        self.write_judge_json("approval-tampered.json", approval)
        request["assignment"] = "judge/assignment-critical.json"
        request["approval"] = "judge/approval-tampered.json"
        request_path = self.write_judge_json("quorum-approval-tampered.json", request)
        _, payload = self.run_judge("quorum", request_path)
        self.assertFalse(payload["data"]["quorum"]["approval_valid"])

    def test_roster_excludes_requester_and_task_agent(self) -> None:
        assignment = read_json(FIXTURES / "assignment-elevated.json")
        assignment["slots"][0]["judge_instance_id"] = assignment["task_agent_id"]
        assignment["assignment_digest"] = artifact_digest(
            assignment, "assignment_digest"
        )
        self.write_judge_json("assignment-self-review.json", assignment)
        request = read_json(FIXTURES / "quorum-elevated-two-of-three.json")
        request["assignment"] = "judge/assignment-self-review.json"
        request_path = self.write_judge_json("quorum-self-review.json", request)
        process, payload = self.run_judge("quorum", request_path)
        self.assertNotEqual(process.returncode, 0, payload)

    def test_missing_authenticated_owner_provenance_is_indeterminate(self) -> None:
        assignment = read_json(FIXTURES / "assignment-elevated.json")
        del assignment["owner_provenance"]
        self.write_judge_json("assignment-no-provenance.json", assignment)
        request = read_json(FIXTURES / "quorum-elevated-two-of-three.json")
        request["assignment"] = "judge/assignment-no-provenance.json"
        request_path = self.write_judge_json("quorum-no-provenance.json", request)
        _, payload = self.run_judge("quorum", request_path)
        self.assertEqual(payload["data"]["quorum"]["result"], "INDETERMINATE")

    def test_wrong_slot_identity_evidence_and_timestamp_are_excluded(self) -> None:
        mutations = {
            "slot": ("slot_id", "unknown-slot"),
            "identity": ("judge_instance_id", "arbitrary-instance"),
            "evidence": (
                "eligibility_evidence_digest",
                "sha256:" + ("f" * 64),
            ),
            "timestamp": ("created_at", "2026-07-24T00:59:00Z"),
            "assignment": ("assignment_digest", "sha256:" + ("e" * 64)),
        }
        for label, (field, value) in mutations.items():
            with self.subTest(label=label):
                verdict = read_json(FIXTURES / "verdict-elevated-pass-b.json")
                verdict[field] = value
                verdict_name = f"verdict-wrong-{label}.json"
                self.write_judge_json(verdict_name, verdict)
                request = read_json(FIXTURES / "quorum-elevated-two-of-three.json")
                request["verdicts"][1] = f"judge/{verdict_name}"
                request_path = self.write_judge_json(
                    f"quorum-wrong-{label}.json", request
                )
                _, payload = self.run_judge("quorum", request_path)
                quorum = payload["data"]["quorum"]
                self.assertNotEqual(quorum["result"], "PASS")
                self.assertEqual(quorum["excluded_count"], 1)

    def test_quorum_rejects_traversal_absolute_symlink_and_oversize_paths(
        self,
    ) -> None:
        request = read_json(FIXTURES / "quorum-elevated-two-of-three.json")
        attacks = ("../package.json", "/tmp/package.json")
        for index, attack in enumerate(attacks):
            with self.subTest(attack=attack):
                hostile = copy.deepcopy(request)
                hostile["package"] = attack
                request_path = self.write_judge_json(
                    f"quorum-path-{index}.json", hostile
                )
                process, _ = self.run_judge("quorum", request_path)
                self.assertNotEqual(process.returncode, 0)

        outside = self.work / "outside.json"
        outside.write_bytes((FIXTURES / "package-elevated.json").read_bytes())
        symlink = self.target / "judge/package-link.json"
        try:
            symlink.symlink_to(outside)
        except OSError:
            pass
        else:
            hostile = copy.deepcopy(request)
            hostile["package"] = "judge/package-link.json"
            request_path = self.write_judge_json("quorum-symlink.json", hostile)
            process, _ = self.run_judge("quorum", request_path)
            self.assertNotEqual(process.returncode, 0)

        oversized = self.target / "judge/package-oversized.json"
        oversized.write_bytes(b" " * (256 * 1024 + 1))
        hostile["package"] = "judge/package-oversized.json"
        request_path = self.write_judge_json("quorum-oversized.json", hostile)
        process, _ = self.run_judge("quorum", request_path)
        self.assertNotEqual(process.returncode, 0)


if __name__ == "__main__":
    unittest.main()
