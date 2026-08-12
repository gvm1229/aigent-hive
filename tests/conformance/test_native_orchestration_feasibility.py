#!/usr/bin/env python3
"""Host feasibility snapshots stay explicit, schema-bound, and default-off."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = json.loads(
    (ROOT / "schemas/host-orchestration-capability.schema.json").read_text(
        encoding="utf-8"
    )
)
FIXTURES = ROOT / "tests/fixtures/native-orchestration"


class NativeOrchestrationFeasibilityContract(unittest.TestCase):
    def test_all_host_snapshots_validate(self) -> None:
        validator = Draft202012Validator(SCHEMA, format_checker=FormatChecker())
        hosts = set()
        for path in sorted(FIXTURES.glob("*-capability.json")):
            payload = json.loads(path.read_text(encoding="utf-8"))
            with self.subTest(path=path.name):
                validator.validate(payload)
                self.assertEqual(payload["activation"], "default-off")
                hosts.add(payload["host"])
        self.assertEqual(hosts, {"codex", "claude", "antigravity"})

    def test_no_host_claims_complete_lifecycle_or_attestation(self) -> None:
        for path in sorted(FIXTURES.glob("*-capability.json")):
            payload = json.loads(path.read_text(encoding="utf-8"))
            capabilities = payload["capabilities"]
            with self.subTest(path=path.name):
                self.assertNotEqual(capabilities["idempotency"], "supported")
                self.assertNotEqual(capabilities["runtime_attestation"], "supported")
                self.assertNotEqual(capabilities["fresh_session"], "supported")

    def test_snapshots_do_not_define_provider_credentials(self) -> None:
        forbidden = {"api_key", "credential", "token", "endpoint"}
        for path in sorted(FIXTURES.glob("*-capability.json")):
            payload = json.loads(path.read_text(encoding="utf-8"))
            keys = set()

            def collect(value: object) -> None:
                if isinstance(value, dict):
                    keys.update(value)
                    for child in value.values():
                        collect(child)
                elif isinstance(value, list):
                    for child in value:
                        collect(child)

            collect(payload)
            with self.subTest(path=path.name):
                self.assertTrue(forbidden.isdisjoint(keys))


if __name__ == "__main__":
    unittest.main()
