#!/usr/bin/env python3
"""V9-021 capability inventory and clean-room runtime gates."""

from __future__ import annotations

import re
import unittest
from collections import Counter
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
INVENTORY_PATH = (
    REPOSITORY_ROOT / "docs/research/v0.9-omx-omc-capability-inventory.md"
)

OMX_SKILLS = frozenset(
    {
        "ai-slop-cleaner",
        "analyze",
        "ask",
        "ask-claude",
        "ask-gemini",
        "autopilot",
        "autoresearch",
        "autoresearch-goal",
        "best-practice-research",
        "build-fix",
        "cancel",
        "code-review",
        "configure-notifications",
        "deep-interview",
        "deepsearch",
        "design",
        "doctor",
        "ecomode",
        "frontend-ui-ux",
        "git-master",
        "help",
        "hud",
        "note",
        "omx-setup",
        "performance-goal",
        "pipeline",
        "plan",
        "prometheus-strict",
        "ralph",
        "ralph-init",
        "ralplan",
        "review",
        "security-review",
        "skill",
        "swarm",
        "tdd",
        "team",
        "trace",
        "ultragoal",
        "ultraqa",
        "ultrawork",
        "visual-ralph",
        "visual-verdict",
        "web-clone",
        "wiki",
        "worker",
    }
)
OMX_ADOPT = frozenset({"ai-slop-cleaner", "best-practice-research"})
OMX_MERGE = frozenset(
    {
        "autoresearch",
        "autoresearch-goal",
        "performance-goal",
        "pipeline",
        "ralph",
        "ultragoal",
        "ultraqa",
        "wiki",
    }
)

OMC_SKILLS = frozenset(
    {
        "ai-slop-cleaner",
        "ask",
        "autopilot",
        "autoresearch",
        "cancel",
        "ccg",
        "configure-notifications",
        "debug",
        "deep-dive",
        "deep-interview",
        "deepinit",
        "external-context",
        "hud",
        "learner",
        "mcp-setup",
        "omc-doctor",
        "omc-reference",
        "omc-setup",
        "omc-teams",
        "plan",
        "project-session-manager",
        "ralph",
        "ralplan",
        "release",
        "remember",
        "sciomc",
        "self-improve",
        "setup",
        "skill",
        "skillify",
        "team",
        "trace",
        "ultraqa",
        "ultrawork",
        "verify",
        "visual-verdict",
        "wiki",
        "writer-memory",
    }
)
OMC_ADOPT = frozenset({"ai-slop-cleaner"})
OMC_MERGE = frozenset(
    {
        "autoresearch",
        "external-context",
        "ralph",
        "remember",
        "ultraqa",
        "verify",
        "wiki",
    }
)

CURRENT_SOURCE_SKILLS = frozenset(
    {
        "ai-slop-cleaner",
        "auto-setup-harness",
        "best-practice-research",
        "hive-commit",
        "hive-directive-amend",
        "hive-editless-question",
        "hive-knowledge-capture",
        "hive-knowledge-query",
        "hive-knowledge-scan",
        "hive-loop-engineering",
        "hive-prompt-refine",
        "hive-simple-question",
        "hive-source-wiki",
        "hive-usage-guard",
        "hive-wiki",
    }
)
CURRENT_CONSUMER_SKILLS = frozenset(
    {
        "ai-slop-cleaner",
        "auto-setup-harness",
        "best-practice-research",
        "hive-judge-package",
        "hive-knowledge-capture",
        "hive-knowledge-maintenance",
        "hive-knowledge-promote",
        "hive-knowledge-query",
        "hive-knowledge-scan",
        "hive-loop-engineering",
        "hive-migrate",
        "hive-project-upgrade",
        "hive-prompt-refine",
        "hive-role-handoff",
        "hive-run-checkpoint",
        "hive-run-resume",
        "hive-simple-question",
        "hive-update",
        "hive-usage-guard",
        "hive-wiki",
        "setup-harness",
        "setup-hive",
    }
)
CURRENT_SHARED_SKILLS = CURRENT_SOURCE_SKILLS & CURRENT_CONSUMER_SKILLS

V09_SKILL_NAMES = (
    "ai-slop-cleaner",
    "best-practice-research",
    "hive-knowledge-scan",
    "hive-loop-engineering",
    "hive-wiki",
)
V09_RUNTIME_FILES = (
    "Cargo.toml",
    "crates/hive-cli/Cargo.toml",
    "crates/hive-cli/src/knowledge.rs",
    "crates/hive-cli/src/knowledge_scan.rs",
    "crates/hive-cli/src/loop_engineering.rs",
    "crates/hive-core/Cargo.toml",
    "crates/hive-core/src/loop_graph.rs",
    "crates/hive-wiki/Cargo.toml",
    "crates/hive-wiki/src/bundle_io.rs",
    "crates/hive-wiki/src/collection.rs",
    "crates/hive-wiki/src/lib.rs",
    "crates/hive-wiki/src/portable.rs",
    "crates/hive-wiki/src/rag.rs",
    "crates/hive-wiki/src/scan.rs",
    "crates/hive-wiki/src/shared.rs",
    "crates/hive-wiki/src/store.rs",
    "harness/skills/catalog.yml",
    "schemas/host-capability.schema.json",
    "schemas/loop-dispatch.schema.json",
    "schemas/loop-graph.schema.json",
)
V09_DIRECTIVE_FILES = (
    ".agents/directives/00-editing-discipline.md",
    ".agents/directives/01-behavior.md",
    ".agents/directives/02-architecture.md",
    ".agents/directives/03-workflow.md",
    ".agents/directives/05-security-safety.md",
)
V09_SKILL_GLOBS = (
    "ai-slop-cleaner",
    "best-practice-research",
    "hive-knowledge-*",
    "hive-loop-*",
    "hive-source-wiki",
    "hive-wiki",
)
TEXT_SUFFIXES = {
    ".json",
    ".md",
    ".py",
    ".ps1",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".yaml",
    ".yml",
}

PROTECTIVE_LINE = re.compile(
    r"(?i)(?:\bnever\b|\bmust not\b|\bdo not\b|\bforbid(?:den)?\b|"
    r"\bprohibit(?:ed)?\b|\breject(?:ed)?\b|\bden(?:y|ied)\b|"
    r"\bblock(?:ed)?\b|\bexclude(?:d)?\b|\bunsupported\b|"
    r"금지|차단|제외|거부|미지원|의존\s*(?:0|없)|실행하지|호출하지|"
    r"읽지|쓰지|생성하지|수집하지|0\s*(?:건|bytes?))"
)
EXPLICIT_COMPATIBILITY = re.compile(
    r"(?i)(?:user[- ]selected|explicit(?:ly)?[- ]selected|"
    r"explicit external compatibility|사용자.{0,20}명시.{0,20}선택|"
    r"명시.{0,20}선택.{0,20}외부\s*호환)"
)
COMMAND_CALL = re.compile(
    r"(?i)(?:Command::new|subprocess\.(?:run|call|Popen)|"
    r"(?:child_process\.)?(?:exec|execFile|spawn)|os\.system)\s*\(\s*"
    r"[rubf]*[\"'](?:omx|omc|tmux|psmux)[\"']"
)
CONFIGURED_COMMAND = re.compile(
    r"(?i)\b(?:command|executable|program|binary)\b\s*[:=]\s*"
    r"[\"']?(?:omx|omc|tmux|psmux)\b"
)
SHELL_COMMAND = re.compile(r"(?i)^\s*(?:[$>]\s*)?(?:omx|omc|tmux|psmux)\b")
IMPERATIVE_COMMAND = re.compile(
    r"(?i)(?:\b(?:run(?!\s*(?:owned\b|,\s*including\b))|execute|invoke|launch|spawn|call)\b.{0,60}"
    r"\b(?:omx|omc|tmux|psmux)\b|\b(?:omx|omc|tmux|psmux)\b"
    r".{0,60}(?:실행|호출|시작|기동))"
)
NAMESPACE_IO = re.compile(
    r"(?i)(?:Path(?:::new|Buf::from)|\.join|fs::(?:read|write|read_dir|"
    r"create_dir|remove_dir)|File::(?:open|create)|open|read_text|write_text|"
    r"mkdir|rmtree)\s*\([^\n)]{0,200}(?:\.omx|\.omc|omx_wiki)"
)
IMPERATIVE_NAMESPACE = re.compile(
    r"(?i)(?:\b(?:read|write|load|store|create|delete|remove|scan|open|"
    r"migrate|update)\b.{0,80}(?:\.omx|\.omc|omx_wiki)|"
    r"(?:\.omx|\.omc|omx_wiki).{0,80}(?:읽기|쓰기|생성|삭제|스캔|갱신))"
)
PROVIDER_SDK = re.compile(
    r"(?i)(?:^\s*(?:use|import|from|require|extern\s+crate)\s+"
    r"(?:async_openai|openai|anthropic|anthropic_ai|google_generative_ai|genai)\b|"
    r"^\s*(?:async[-_]openai|openai|anthropic|anthropic[-_]ai|"
    r"google[-_]generative[-_]ai)\s*=|@anthropic-ai/(?:sdk|claude-agent-sdk))"
)
EXTERNAL_RUNTIME_DEPENDENCY = re.compile(
    r"(?i)^\s*(?:omx|omc|tmux|psmux)\s*[:=]"
)
CREDENTIAL_ACCESS = re.compile(
    r"(?i)(?:(?:Path(?:::new|Buf::from)|\.join)\s*\([^\n)]{0,160}"
    r"(?:auth\.json|credentials\.json|api[_-]?key|access[_-]?token|"
    r"refresh[_-]?token)|(?:fs::(?:read|write)|read_to_string|File::(?:open|create)|"
    r"open|read_text|write_text|load|save|keychain)\s*\([^\n)]{0,240}"
    r"(?:auth\.json|credentials\.json|api[_-]?key|access[_-]?token|"
    r"refresh[_-]?token)|(?:env::var|std::env::var|os\.environ|process\.env)"
    r".{0,100}(?:OPENAI_API_KEY|ANTHROPIC_API_KEY|GOOGLE_API_KEY|"
    r"GEMINI_API_KEY))"
)
KEYWORD_ACTIVATION = re.compile(
    r"(?i)(?:keyword[-_ ]?detector|skill[-_ ]?injector|prompt[-_ ]?classifier|"
    r"keyword.{0,40}(?:skill[-_ ]?inject|prompt[-_ ]?classif|"
    r"prompt[-_ ]?detect)|(?:skill[-_ ]?inject|prompt[-_ ]?classif|"
    r"prompt[-_ ]?detect).{0,40}keyword)"
)
STOP_CONTINUATION = re.compile(
    r"(?i)(?:[\"'`]Stop[\"'`]|Stop[-_ ]hook).{0,80}"
    r"(?:continu|resume|retry|restart|reinvoke)|(?:continu|resume|reinvoke)"
    r".{0,80}Stop[-_ ]hook"
)


def section(text: str, heading: str, next_heading: str) -> str:
    match = re.search(
        rf"(?ms)^{re.escape(heading)}\s*$\n(.*?)(?=^{re.escape(next_heading)}\s*$)",
        text,
    )
    if match is None:
        raise AssertionError(f"missing section boundary: {heading!r} -> {next_heading!r}")
    return match.group(1)


def skill_rows(text: str, heading: str, next_heading: str) -> list[tuple[str, str, str, str]]:
    body = section(text, heading, next_heading)
    return re.findall(
        r"(?m)^\| `([^`]+)` \| `(adopt|merge|exclude)` \| (.+?) \| (.+?) \|$",
        body,
    )


def expected_decisions(
    names: frozenset[str], adopt: frozenset[str], merge: frozenset[str]
) -> dict[str, str]:
    decisions = {name: "exclude" for name in names}
    decisions.update({name: "adopt" for name in adopt})
    decisions.update({name: "merge" for name in merge})
    return decisions


def v09_runtime_paths() -> list[Path]:
    paths = {
        REPOSITORY_ROOT / relative
        for relative in (*V09_RUNTIME_FILES, *V09_DIRECTIVE_FILES)
        if (REPOSITORY_ROOT / relative).is_file()
    }
    for root_name in (".agents/skills", "harness/skills"):
        skill_parent = REPOSITORY_ROOT / root_name
        skill_roots = {
            skill_parent / skill_name for skill_name in V09_SKILL_NAMES
        }
        for pattern in V09_SKILL_GLOBS:
            skill_roots.update(skill_parent.glob(pattern))
        for skill_root in skill_roots:
            if skill_root.is_dir():
                paths.update(
                    path
                    for path in skill_root.rglob("*")
                    if path.is_file() and path.suffix.lower() in TEXT_SUFFIXES
                )
    for pattern in (
        "crates/hive-cli/src/knowledge*.rs",
        "crates/hive-cli/src/loop_*.rs",
        "crates/hive-core/src/host_capability*.rs",
        "crates/hive-core/src/loop_*.rs",
        "crates/hive-wiki/src/bundle*.rs",
        "crates/hive-wiki/src/collection*.rs",
        "crates/hive-wiki/src/portable*.rs",
        "crates/hive-wiki/src/rag*.rs",
        "crates/hive-wiki/src/scan*.rs",
        "crates/hive-wiki/src/store*.rs",
        "schemas/host-capability*.schema.json",
        "schemas/knowledge-*.schema.json",
        "schemas/loop-*.schema.json",
    ):
        paths.update(path for path in REPOSITORY_ROOT.glob(pattern) if path.is_file())
    return sorted(paths)


def scan_runtime_text(path: Path, text: str) -> list[tuple[int, str, str]]:
    findings: list[tuple[int, str, str]] = []
    markdown_fence = False
    for line_number, line in enumerate(text.splitlines(), start=1):
        stripped = line.strip()
        if path.suffix.lower() == ".md" and stripped.startswith("```"):
            markdown_fence = not markdown_fence
            continue
        selection_contract = EXPLICIT_COMPATIBILITY.search(line) and not any(
            pattern.search(line)
            for pattern in (
                COMMAND_CALL,
                CONFIGURED_COMMAND,
                SHELL_COMMAND,
                IMPERATIVE_COMMAND,
                NAMESPACE_IO,
                IMPERATIVE_NAMESPACE,
            )
        )
        prose_boundary = path.suffix.lower() == ".md" and (
            PROTECTIVE_LINE.search(line) or selection_contract
        )
        comment_boundary = stripped.startswith(("#", "//")) and PROTECTIVE_LINE.search(
            line
        )
        if prose_boundary or comment_boundary:
            continue

        checks = (
            ("external-command", COMMAND_CALL),
            ("configured-command", CONFIGURED_COMMAND),
            ("external-runtime-dependency", EXTERNAL_RUNTIME_DEPENDENCY),
            ("foreign-namespace-io", NAMESPACE_IO),
            ("provider-sdk", PROVIDER_SDK),
            ("credential-access", CREDENTIAL_ACCESS),
            ("keyword-activation", KEYWORD_ACTIVATION),
            ("stop-continuation", STOP_CONTINUATION),
        )
        for rule, pattern in checks:
            if pattern.search(line):
                findings.append((line_number, rule, stripped))

        if path.suffix.lower() == ".md":
            if markdown_fence and SHELL_COMMAND.search(line):
                findings.append((line_number, "external-shell-command", stripped))
            elif IMPERATIVE_COMMAND.search(line):
                findings.append((line_number, "external-command-instruction", stripped))
            if IMPERATIVE_NAMESPACE.search(line):
                findings.append((line_number, "foreign-namespace-instruction", stripped))
        if not stripped.startswith(("#", "//")) and re.search(
            r"(?i)UserPromptSubmit", line
        ):
            findings.append((line_number, "prompt-submit-hook", stripped))
    return findings


class V09SkillInventoryDocumentContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = INVENTORY_PATH.read_text(encoding="utf-8")

    def assert_inventory(
        self,
        heading: str,
        next_heading: str,
        names: frozenset[str],
        adopt: frozenset[str],
        merge: frozenset[str],
    ) -> None:
        rows = skill_rows(self.text, heading, next_heading)
        row_names = [name for name, _, _, _ in rows]
        self.assertEqual(len(rows), len(names))
        self.assertEqual(len(set(row_names)), len(names))
        self.assertEqual(set(row_names), names)
        actual = {name: decision for name, decision, _, _ in rows}
        self.assertEqual(actual, expected_decisions(names, adopt, merge))
        counts = Counter(decision for _, decision, _, _ in rows)
        self.assertEqual(
            counts,
            Counter(expected_decisions(names, adopt, merge).values()),
        )
        for name, _, owner, rationale in rows:
            with self.subTest(skill=name):
                self.assertTrue(owner.strip())
                self.assertTrue(rationale.strip())

    def test_omx_inventory_is_exact_and_unique(self) -> None:
        self.assertEqual(len(OMX_SKILLS), 46)
        self.assert_inventory(
            "## OMX Skill 46/46",
            "## OMC Skill 38/38",
            OMX_SKILLS,
            OMX_ADOPT,
            OMX_MERGE,
        )

    def test_omc_inventory_is_exact_and_unique(self) -> None:
        self.assertEqual(len(OMC_SKILLS), 38)
        self.assert_inventory(
            "## OMC Skill 38/38",
            "## Agent·tool·adapter inventory",
            OMC_SKILLS,
            OMC_ADOPT,
            OMC_MERGE,
        )

    def test_hive_current_inventory_matches_exact_skill_directories(self) -> None:
        actual_source = frozenset(
            path.parent.name
            for path in (REPOSITORY_ROOT / ".agents/skills").glob("*/SKILL.md")
        )
        actual_consumer = frozenset(
            path.parent.name
            for path in (REPOSITORY_ROOT / "harness/skills").glob("*/SKILL.md")
        )
        self.assertEqual(actual_source, CURRENT_SOURCE_SKILLS)
        self.assertEqual(actual_consumer, CURRENT_CONSUMER_SKILLS)
        self.assertEqual(len(CURRENT_SOURCE_SKILLS), 15)
        self.assertEqual(len(CURRENT_CONSUMER_SKILLS), 22)
        self.assertEqual(len(CURRENT_SHARED_SKILLS), 11)

        sections = (
            (
                "### 현재 source Skill 15/15",
                "### 현재 consumer Skill 22/22",
                CURRENT_SOURCE_SKILLS,
            ),
            (
                "### 현재 source↔consumer 교집합 11/11",
                "### 게시된 0.8 기준선",
                CURRENT_SHARED_SKILLS,
            ),
        )
        for heading, next_heading, expected in sections:
            values = re.findall(
                r"`([a-z][a-z0-9-]+)`",
                section(self.text, heading, next_heading),
            )
            with self.subTest(heading=heading):
                self.assertEqual(len(values), len(expected))
                self.assertEqual(set(values), expected)

        consumer_values = re.findall(
            r"`([a-z][a-z0-9-]+)`",
            section(
                self.text,
                "### 현재 consumer Skill 22/22",
                "### 현재 source↔consumer 교집합 11/11",
            ),
        )
        self.assertEqual(len(consumer_values), len(CURRENT_CONSUMER_SKILLS))
        self.assertEqual(set(consumer_values), CURRENT_CONSUMER_SKILLS)
        for historical in (
            "source Skill `7`",
            "consumer Skill `17`",
            "source↔consumer 교집합 `3`",
        ):
            self.assertIn(historical, self.text)

    def test_source_provenance_and_license_states_are_digest_bound(self) -> None:
        required = (
            "| Source | Version·revision | Inventory evidence | License state |",
            "`0.20.4`, `57f8e682af899b5d0e28d05b238c903c2fdeb913`",
            "e18a0a9c7a3362acd2d144c780950be949a3ab2878e0039011e7c139c56a224b",
            "278fff4222c019e02ef6ca2c22c3454fc32d39191b9f1188b2abcf8a204dd7ad",
            "`4.13.4`, `c1cd5d08b5279c82d2d9057fb61473ceacead4`",
            "1d2b966f93feaa8928a57a69868388ee9016c86a341299fc5261ded44249af79",
            "e4be54c4f141be7573a8bd8d9e2b8c1ab5e6a9f9a0e52c30584067e2e7a973f2",
            "OMX: MIT metadata 선언, license 전문 file 부재",
            "OMC: MIT 전문과 revision 확인",
            "Hive 결과물: Apache-2.0 독립 저작",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, self.text)
        self.assertRegex(self.text, r"OMX .*MIT 선언.*root `LICENSE` 파일 부재")
        self.assertRegex(self.text, r"OMC .*MIT 전문 확인.*`LICENSE` SHA-256")
        self.assertRegex(self.text, r"Hive .*Apache-2\.0.*root `LICENSE`")

    def test_clean_room_and_copied_byte_assertions_are_explicit(self) -> None:
        for required in (
            "concept-only clean-room 분류",
            "외부 원문·코드·prompt 복사 `0 bytes`",
            "외부 copied bytes: `0`",
            "외부 prompt·Skill paragraph의 번역·축약 복사: `0`",
            "외부 code·schema·test fixture import: `0`",
        ):
            with self.subTest(required=required):
                self.assertIn(required, self.text)
        self.assertNotIn("```", self.text)

    def test_required_evidence_surfaces_and_static_gates_are_complete(self) -> None:
        for heading in (
            "## Hive current Skill inventory",
            "## Agent·tool·adapter inventory",
            "### OMX agent·model instruction 39",
            "### OMC agent 19",
            "### OMX configured tool 45",
            "### OMC MCP·bridge surface",
            "### Runtime adapter classification",
            "## Security·implicit activation gate",
            "### Credential·provider boundary",
            "### Implicit activation boundary",
            "## License·clean-room boundary",
            "## Static conformance gates",
            "## V9-021 acceptance evidence",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, self.text)

        gate_body = section(
            self.text,
            "## Static conformance gates",
            "## V9-021 acceptance evidence",
        )
        gates = {
            int(number): label
            for number, label in re.findall(
                r"(?m)^(\d+)\. ([^:]+):", gate_body
            )
        }
        self.assertEqual(
            gates,
            {
                1: "Inventory cardinality",
                2: "Classification uniqueness",
                3: "Source provenance",
                4: "Copied-byte assertion",
                5: "Namespace scan",
                6: "Command scan",
                7: "Runtime scan",
                8: "Provider scan",
                9: "Capture scan",
                10: "Activation scan",
                11: "Owner scan",
                12: "Projection parity",
                13: "Consent gate",
                14: "Host gap truth",
                15: "Regression evidence",
            },
        )
        for marker in (
            "UserPromptSubmit",
            "Stop continuation",
            "provider SDK",
            "credential read/write",
            "keyword detector",
            "Skill injector",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.text)


class V09RuntimeBoundaryContract(unittest.TestCase):
    def test_scan_scope_contains_only_v09_runtime_surfaces(self) -> None:
        paths = v09_runtime_paths()
        self.assertTrue(paths)
        relatives = {
            path.relative_to(REPOSITORY_ROOT).as_posix() for path in paths
        }
        required_runtime = {
            "crates/hive-cli/src/knowledge.rs",
            "crates/hive-cli/src/knowledge_scan.rs",
            "crates/hive-cli/src/loop_engineering.rs",
            "crates/hive-core/src/loop_graph.rs",
            "crates/hive-wiki/src/bundle_io.rs",
            "crates/hive-wiki/src/portable.rs",
            "crates/hive-wiki/src/rag.rs",
            "crates/hive-wiki/src/scan.rs",
            "crates/hive-wiki/src/store.rs",
            "harness/skills/catalog.yml",
            "harness/skills/hive-knowledge-capture/SKILL.md",
            "harness/skills/hive-knowledge-maintenance/SKILL.md",
            "harness/skills/hive-knowledge-promote/SKILL.md",
            "harness/skills/hive-knowledge-query/SKILL.md",
            "schemas/knowledge-bundle-manifest.schema.json",
            "schemas/knowledge-scan-result.schema.json",
            "schemas/loop-dispatch.schema.json",
            "schemas/loop-graph.schema.json",
            *V09_DIRECTIVE_FILES,
        }
        self.assertTrue(
            required_runtime.issubset(relatives),
            sorted(required_runtime - relatives),
        )
        optional_bundle_store = REPOSITORY_ROOT / "crates/hive-wiki/src/bundle_store.rs"
        if optional_bundle_store.is_file():
            self.assertIn("crates/hive-wiki/src/bundle_store.rs", relatives)
        forbidden_parts = {"docs", "fixtures", "project-bases", "user-bases"}
        for path in paths:
            relative = path.relative_to(REPOSITORY_ROOT)
            with self.subTest(path=relative.as_posix()):
                self.assertTrue(forbidden_parts.isdisjoint(relative.parts))

    def test_v09_runtime_has_no_foreign_execution_or_activation_dependency(
        self,
    ) -> None:
        findings: list[str] = []
        for path in v09_runtime_paths():
            relative = path.relative_to(REPOSITORY_ROOT).as_posix()
            text = path.read_text(encoding="utf-8")
            findings.extend(
                f"{relative}:{line}: {rule}: {source}"
                for line, rule, source in scan_runtime_text(path, text)
            )
        self.assertEqual(findings, [], "\n".join(findings))

    def test_scanner_allows_protective_and_selection_contracts(self) -> None:
        protective = (
            "Never run `omx` or read `.omx/`; tmux dependency is prohibited.\n"
            "외부 copied bytes: `0`; UserPromptSubmit injector 실행 금지.\n"
        )
        selection = (
            "The user-selected external compatibility owner may be OMX or OMC.\n"
            "사용자가 명시적으로 선택한 외부 호환 계층만 기록.\n"
        )
        protected_names = (
            'const PROTECTED: &[&str] = &["auth.json", "credentials.json"];\n'
        )
        pinned_owner = (
            "Preserve the pinned owner of every existing run, including a 0.8.x run "
            "owned by OMX or OMC; migration requires an explicit user action.\n"
        )
        self.assertEqual(scan_runtime_text(Path("SKILL.md"), protective), [])
        self.assertEqual(scan_runtime_text(Path("SKILL.md"), selection), [])
        self.assertEqual(scan_runtime_text(Path("scan.rs"), protected_names), [])
        self.assertEqual(scan_runtime_text(Path("directive.md"), pinned_owner), [])

    def test_scanner_detects_executable_and_activation_dependencies(self) -> None:
        hostile = "\n".join(
            (
                'let _ = Command::new("omx").status();',
                'let _ = Command::new("omc").status();',
                'let _ = Command::new("tmux").status();',
                'let _ = Command::new("psmux").status();',
                'tmux = "0.1"',
                'let cache = Path::new(".omx").join("state");',
                'let state = Path::new(".omc").join("state");',
                'let wiki = Path::new("omx_wiki").join("pages");',
                'anthropic = "0.1"',
                'let token = std::env::var("OPENAI_API_KEY")?;',
                'let event = "UserPromptSubmit";',
                'let injector = "keyword Skill injector";',
                'let path = home.join("credentials.json");',
                'let hook = "Stop"; let action = "continue";',
            )
        )
        rules = {
            rule
            for _, rule, _ in scan_runtime_text(Path("runtime.rs"), hostile)
        }
        self.assertEqual(
            rules,
            {
                "credential-access",
                "external-command",
                "external-runtime-dependency",
                "foreign-namespace-io",
                "keyword-activation",
                "prompt-submit-hook",
                "provider-sdk",
                "stop-continuation",
            },
        )
        selected_command = (
            "For user-selected external compatibility, run `omx start`."
        )
        self.assertEqual(
            [
                rule
                for _, rule, _ in scan_runtime_text(
                    Path("SKILL.md"), selected_command
                )
            ],
            ["external-command-instruction"],
        )


if __name__ == "__main__":
    unittest.main()
