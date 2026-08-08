from __future__ import annotations

import hashlib
import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]

CURRENT_REQUIRED = {
    ".agents/directives/02-architecture.md": (
        "Implement Hive-native iterative planning",
        "Do not select, invoke, install, or configure OMX/OMC for a new workflow",
        "creates a new Hive-native run identity",
        "dispatch-uncertain",
    ),
    "harness/directives/00-project-harness.md": (
        "host-native capabilities by default for new v0.9 runs",
        "only after explicit user selection",
        "including a 0.8.x external owner",
    ),
    "harness/template/AGENTS.md.jinja": (
        "Start every new v0.9 run with verified host-native capabilities",
        "only when the user explicitly selects that external compatibility layer",
        "including a 0.8.x OMX/OMC owner",
    ),
    "harness/skills/setup-harness/SKILL.md": (
        "Default a new run to the active host's verified native capabilities",
        "Treat OMX and OMC as external compatibility options only",
        "including a 0.8.x OMX/OMC owner",
        "host_capability_unsupported",
    ),
    "harness/skills/hive-judge-package/SKILL.md": (
        "host-native owner by default",
        "explicitly selected external compatibility owner",
        "legacy 0.8.x owner",
    ),
}

CURRENT_ROOTS = (
    ".agents/directives",
    "harness/directives",
    "harness/template",
    "harness/skills",
)

AUTOMATIC_EXTERNAL_PRIORITY = (
    re.compile(r"(?i)\bprefer(?:red|s)?\b[^\n]{0,120}\b(?:omx|omc)\b"),
    re.compile(r"(?i)\b(?:omx|omc)\b[^\n]{0,120}\bprefer(?:red|s)?\b"),
    re.compile(
        r"(?i)\b(?:omx|omc)\b[^\n]{0,120}\bbefore\b[^\n]{0,60}\bhost-native\b"
    ),
)

FROZEN_080 = {
    "harness/project-bases/0.8.0/AGENTS.md.template": (
        "e9545c960f609ad7369e2d5e0cc9f48f79fdc7cd20836cf6199f19eb4ca4f301",
        "Resolve compatible OMX on Codex and compatible OMC on Claude before host-native capability",
    ),
    "harness/project-bases/0.8.0/skills/setup-harness/SKILL.md": (
        "3a97227617ad0115ed59288d3ea4e909bd4a6167255d3ff433f6d765ba319790",
        "On Codex, prefer compatible OMX. On Claude Code, prefer compatible OMC.",
    ),
    "harness/user-bases/0.8.0/plugins/aigent-hive/skills/setup-harness/SKILL.md": (
        "3a97227617ad0115ed59288d3ea4e909bd4a6167255d3ff433f6d765ba319790",
        "On Codex, prefer compatible OMX. On Claude Code, prefer compatible OMC.",
    ),
}


class V09HostNativeContractTests(unittest.TestCase):
    def test_current_contracts_default_host_native_and_bound_external_selection(self) -> None:
        for relative, required in CURRENT_REQUIRED.items():
            text = (REPOSITORY_ROOT / relative).read_text(encoding="utf-8")
            with self.subTest(path=relative):
                for phrase in required:
                    self.assertIn(phrase, text)

    def test_current_contracts_have_no_omx_omc_automatic_priority(self) -> None:
        findings: list[str] = []
        for root in CURRENT_ROOTS:
            for path in sorted((REPOSITORY_ROOT / root).rglob("*")):
                if not path.is_file() or path.suffix not in {".jinja", ".md", ".yaml"}:
                    continue
                text = path.read_text(encoding="utf-8")
                for pattern in AUTOMATIC_EXTERNAL_PRIORITY:
                    if match := pattern.search(text):
                        relative = path.relative_to(REPOSITORY_ROOT).as_posix()
                        findings.append(f"{relative}: {match.group(0)}")
        self.assertEqual(findings, [], "\n".join(findings))

    def test_source_contract_allows_native_orchestration_without_provider_runtime(self) -> None:
        text = (REPOSITORY_ROOT / ".agents/directives/02-architecture.md").read_text(
            encoding="utf-8"
        )
        for required in (
            "logical scheduling",
            "team coordination",
            "multi-goal execution",
            "declarative execution envelopes",
            "authenticated single-action authority",
        ):
            self.assertIn(required, text)
        for forbidden in (
            "model-provider API",
            "launch a model/subagent process",
        ):
            self.assertIn(forbidden, text)

    def test_run_data_skills_preserve_pinned_owner(self) -> None:
        for relative in (
            "harness/skills/hive-run-checkpoint/SKILL.md",
            "harness/skills/hive-run-resume/SKILL.md",
            "harness/skills/hive-role-handoff/SKILL.md",
            "harness/skills/hive-project-upgrade/SKILL.md",
        ):
            text = (REPOSITORY_ROOT / relative).read_text(encoding="utf-8")
            with self.subTest(path=relative):
                self.assertRegex(text, r"(?i)(?:preserv|remain).{0,120}0\.8\.x")
                self.assertRegex(text, r"(?i)(?:pinned owner|owner pins?)")

    def test_frozen_080_owner_contract_remains_byte_exact(self) -> None:
        for relative, (expected_digest, legacy_phrase) in FROZEN_080.items():
            content = (REPOSITORY_ROOT / relative).read_bytes()
            with self.subTest(path=relative):
                self.assertEqual(hashlib.sha256(content).hexdigest(), expected_digest)
                self.assertIn(legacy_phrase, content.decode("utf-8"))


if __name__ == "__main__":
    unittest.main()
