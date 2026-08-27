#!/usr/bin/env python3
"""Exact public action/result routing contract for v0.9 CLI verbs."""

from __future__ import annotations

import json
import os
import re
import subprocess
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[3]
SCHEMA = ROOT / "schemas" / "action-result.schema.json"
KNOWLEDGE_SOURCE = ROOT / "crates" / "hive-cli" / "src" / "knowledge.rs"
LOOP_SOURCE = ROOT / "crates" / "hive-cli" / "src" / "loop_engineering.rs"

EXPECTED_ACTIONS = {
    "UnknownAction",
    "SetupHarness",
    "SetupHiveUser",
    "DescribeHiveUserSetup",
    "InstallHiveUser",
    "UninstallHiveUser",
    "UpdateHiveUser",
    "ValidateHiveUser",
    "RecoverHiveUser",
    "CheckHiveUpdate",
    "ValidateRole",
    "RecordRoleHandoff",
    "RoleWork",
    "CheckpointRun",
    "CheckUsage",
    "ShowUsageStatus",
    "SetUsageThreshold",
    "ControlUsageSession",
    "CaptureUsage",
    "InstallUsageFallback",
    "AnswerSimpleQuestion",
    "RefinePrompt",
    "VerifySkillProjection",
    "RunWork",
    "ResumeWork",
    "CheckRunClosure",
    "CheckRunContinuation",
    "VerifyWork",
    "IngestKnowledge",
    "AddKnowledge",
    "QueryKnowledge",
    "VectorKnowledge",
    "ListKnowledge",
    "ReadKnowledge",
    "RememberKnowledge",
    "RetrieveKnowledge",
    "AuthorizeConfidentialKnowledge",
    "MapKnowledgeCollection",
    "RefreshKnowledge",
    "ScanKnowledge",
    "ExportKnowledge",
    "ImportKnowledge",
    "PromoteKnowledge",
    "NotionKnowledge",
    "SyncNotionKnowledge",
    "RetrieveNotionKnowledge",
    "WriteThroughNotionKnowledge",
    "ScanProjectUpgrade",
    "UpdateProjectHarness",
    "ValidateProjectUpgrade",
    "RecoverProjectUpgrade",
    "BeginSession",
    "CheckSession",
    "UpdateSession",
    "CloseSession",
    "RecoverSession",
    "CoordinateSession",
    "LintKnowledge",
    "DeleteKnowledge",
    "SuppressKnowledge",
    "RebuildKnowledgeIndex",
    "Loop",
    "InitializeLoop",
    "ValidateLoop",
    "CheckpointLoop",
    "SteerLoop",
    "PrepareLoopDispatch",
    "RecoverLoop",
    "LintSourceWiki",
    "QuerySourceWiki",
    "RebuildSourceWikiIndex",
    "UpdateHarness",
}

EXPECTED_KNOWLEDGE_ACTIONS = {action for action in EXPECTED_ACTIONS if "Knowledge" in action}
EXPECTED_LOOP_ACTIONS = {
    "Loop",
    "InitializeLoop",
    "ValidateLoop",
    "CheckpointLoop",
    "SteerLoop",
    "PrepareLoopDispatch",
    "RecoverLoop",
}
GENERIC_EVIDENCE_KINDS = {
    "command",
    "file",
    "test",
    "verdict",
    "report",
    "release",
    "user-setup-catalog",
    "session-manifest",
}
EXPECTED_LOOP_EVIDENCE_KINDS = {
    "loop-graph",
    "loop-evidence",
    "usage-authorization",
    "capability-resolution",
    "run-status",
    "prepared-dispatch",
}
LOOP_COMMAND_ACTIONS = {
    ("unknown",): "Loop",
    ("initialize",): "InitializeLoop",
    ("validate",): "ValidateLoop",
    ("checkpoint",): "CheckpointLoop",
    ("steer",): "SteerLoop",
    ("prepare",): "PrepareLoopDispatch",
    ("recover",): "RecoverLoop",
}


class ActionResultActionTests(unittest.TestCase):
    def test_schema_action_enum_is_the_exact_public_action_set(self) -> None:
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        actions = schema["properties"]["action"]["enum"]
        self.assertEqual(len(actions), len(set(actions)), "duplicate action enum")
        self.assertEqual(set(actions), EXPECTED_ACTIONS)

    def test_v09_source_actions_are_exactly_represented(self) -> None:
        knowledge = KNOWLEDGE_SOURCE.read_text(encoding="utf-8")
        loop = LOOP_SOURCE.read_text(encoding="utf-8")
        knowledge_actions = set(
            re.findall(r'"([A-Z][A-Za-z]*Knowledge[A-Za-z]*)"', knowledge)
        )
        loop_actions = set(
            re.findall(
                r'"(Loop|[A-Z][A-Za-z]+Loop|PrepareLoopDispatch)"',
                loop,
            )
        )
        self.assertEqual(knowledge_actions, EXPECTED_KNOWLEDGE_ACTIONS)
        self.assertEqual(loop_actions, EXPECTED_LOOP_ACTIONS)
        schema_actions = set(
            json.loads(SCHEMA.read_text(encoding="utf-8"))["properties"]["action"]["enum"]
        )
        self.assertTrue(knowledge_actions | loop_actions <= schema_actions)

    def test_new_knowledge_actions_validate_and_unknown_action_is_rejected(self) -> None:
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        validator = Draft202012Validator(schema, format_checker=FormatChecker())
        base = {
            "schema_version": 1,
            "status": "success",
            "exit_code": 0,
            "code": "hive.test",
            "message": "schema fixture",
            "changed_paths": [],
            "evidence": [],
            "next_action": None,
            "data": {},
        }
        for action in (
            "AuthorizeConfidentialKnowledge",
            "MapKnowledgeCollection",
            "NotionKnowledge",
            "SyncNotionKnowledge",
            "RetrieveNotionKnowledge",
            "WriteThroughNotionKnowledge",
        ):
            with self.subTest(action=action):
                validator.validate({**base, "action": action})
        self.assertTrue(
            list(validator.iter_errors({**base, "action": "ForgeKnowledgeAuthority"}))
        )

    def test_schema_covers_generic_and_loop_evidence_kinds(self) -> None:
        loop = LOOP_SOURCE.read_text(encoding="utf-8")
        emitted_kinds = set(
            re.findall(r'(?<![A-Za-z])Evidence\s*\{\s*kind:\s*"([a-z-]+)"', loop)
        )
        self.assertEqual(emitted_kinds, EXPECTED_LOOP_EVIDENCE_KINDS)

        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        schema_kinds = set(
            schema["properties"]["evidence"]["items"]["properties"]["kind"]["enum"]
        )
        self.assertEqual(
            schema_kinds,
            GENERIC_EVIDENCE_KINDS | EXPECTED_LOOP_EVIDENCE_KINDS,
        )


class LoopActionResultCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        configured_binary = os.environ.get("HIVE_BIN")
        if configured_binary:
            cls.hive_binary = Path(configured_binary).resolve()
        else:
            subprocess.run(
                ["cargo", "build", "--quiet", "--locked", "--bin", "hive"],
                cwd=ROOT,
                check=True,
            )
            executable = "hive.exe" if os.name == "nt" else "hive"
            cls.hive_binary = ROOT / "target" / "debug" / executable
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)
        cls.validator = Draft202012Validator(
            schema,
            format_checker=FormatChecker(),
        )

    def test_every_loop_action_emits_a_complete_schema_valid_result(self) -> None:
        for arguments, expected_action in LOOP_COMMAND_ACTIONS.items():
            with self.subTest(action=expected_action):
                command = [
                    str(self.hive_binary),
                    "loop",
                    *arguments,
                    "--output",
                    "json",
                ]
                process = subprocess.run(
                    command,
                    cwd=ROOT,
                    check=False,
                    capture_output=True,
                    text=True,
                )
                try:
                    result = json.loads(process.stdout)
                except json.JSONDecodeError as error:
                    self.fail(
                        f"stdout must be exactly one JSON object: {error}\n"
                        f"command={command!r}\nstdout={process.stdout!r}\n"
                        f"stderr={process.stderr!r}"
                    )

                self.validator.validate(result)
                self.assertEqual(result["action"], expected_action)
                self.assertEqual(result["status"], "error")
                self.assertEqual(result["exit_code"], 2)
                self.assertEqual(process.returncode, result["exit_code"])
                self.assertEqual(
                    result["data"],
                    {"prepared_only": True, "spawned": False},
                )


if __name__ == "__main__":
    unittest.main()
