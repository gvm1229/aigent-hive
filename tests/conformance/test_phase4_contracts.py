#!/usr/bin/env python3
"""Phase 4 black-box role, handoff, checkpoint, and resume conformance."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from typing import Any

import yaml
from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[2]
SCHEMAS = ROOT / "schemas"
PHASE1 = ROOT / "tests/fixtures/phase1"
FIXTURES = ROOT / "tests/fixtures/phase4"
CAPABILITIES = {
    "codex-omx": PHASE1 / "capabilities-codex-omx.json",
    "claude-omc": PHASE1 / "capabilities-claude-omc.json",
    "absent": PHASE1 / "capabilities-absent.json",
    "incompatible": PHASE1 / "capabilities-incompatible.json",
    "unknown": PHASE1 / "capabilities-unknown.json",
}
SCHEMA_FILES = {
    "action": "action-result.schema.json",
    "role": "role-profile.schema.json",
    "handoff_request": "role-handoff-request.schema.json",
    "checkpoint_request": "run-checkpoint-request.schema.json",
    "status": "run-status.schema.json",
    "brief": "dispatch-brief.schema.json",
    "capability": "capability-matrix.schema.json",
}
OWNER_KEYS = (
    "host",
    "host_version",
    "surface",
    "external_runtime",
    "resolved_owner",
    "resolution_evidence_digest",
    "subagent_support",
)
DATA_SKILLS = {
    "hive-role-handoff",
    "hive-run-checkpoint",
    "hive-run-resume",
}
RAW_USAGE_ACCOUNT = "usage-guard@example.invalid"
USAGE_ACCOUNT_DIGEST = (
    "sha256:" + hashlib.sha256(RAW_USAGE_ACCOUNT.encode()).hexdigest()
)
CODEXBAR_FIXTURE = (
    ROOT / "tests/fixtures/phase5/usage/codexbar_fixture.py"
).read_text(encoding="utf-8")


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def canonical_digest(value: dict[str, Any]) -> str:
    payload = {key: item for key, item in value.items() if key != "evidence_digest"}
    encoded = json.dumps(
        payload,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest_bytes(encoded)


def parse_document(path: Path) -> tuple[dict[str, Any], bytes]:
    return parse_document_bytes(path.read_bytes())


def parse_document_bytes(value: bytes) -> tuple[dict[str, Any], bytes]:
    if value.startswith(b"---\r\n"):
        prefix = b"---\r\n"
        delimiter = b"\r\n---\r\n"
    else:
        prefix = b"---\n"
        delimiter = b"\n---\n"
    if not value.startswith(prefix):
        raise AssertionError("document frontmatter start is missing")
    frontmatter, marker, body = value[len(prefix) :].partition(delimiter)
    if marker != delimiter:
        raise AssertionError("document frontmatter end is missing")
    return json.loads(frontmatter), body


def snapshot_tree(root: Path) -> dict[str, tuple[Any, ...]]:
    """Capture bytes and file kinds without following links or reading special files."""
    if not root.exists() and not root.is_symlink():
        return {}
    snapshot: dict[str, tuple[Any, ...]] = {}
    for current, directories, files in os.walk(root, followlinks=False):
        directories.sort()
        files.sort()
        current_path = Path(current)
        names = [*directories, *files]
        for name in names:
            path = current_path / name
            relative = path.relative_to(root).as_posix()
            metadata = path.lstat()
            mode = metadata.st_mode
            if stat.S_ISLNK(mode):
                snapshot[relative] = ("symlink", os.readlink(path))
            elif stat.S_ISDIR(mode):
                snapshot[relative] = ("directory", stat.S_IMODE(mode))
            elif stat.S_ISREG(mode):
                try:
                    content: Any = path.read_bytes()
                except PermissionError:
                    content = "<unreadable>"
                snapshot[relative] = ("file", stat.S_IMODE(mode), content)
            else:
                snapshot[relative] = ("special", stat.S_IFMT(mode), stat.S_IMODE(mode))
    return snapshot


class Phase4Contracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        configured = os.environ.get("HIVE_BIN")
        cls.binary = (
            Path(configured).resolve()
            if configured
            else (ROOT / "target/debug/hive").resolve()
        )
        cls.schemas = {
            name: json.loads((SCHEMAS / file_name).read_text(encoding="utf-8"))
            for name, file_name in SCHEMA_FILES.items()
        }
        cls.validators = {
            name: Draft202012Validator(schema, format_checker=FormatChecker())
            for name, schema in cls.schemas.items()
        }

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(
            prefix="hive-phase4-",
            dir=str(ROOT / "tests"),
        )
        self.work = Path(self.temporary.name).resolve()
        self.input_root = self.work / "inputs"
        self.input_root.mkdir()
        self.capability_copy_index = 0
        self.target = self.fresh_target("consumer")
        self.fake_home = self.work / "home"
        for runtime in (".omx", ".omc"):
            sentinel = self.fake_home / runtime / "foreign.txt"
            sentinel.parent.mkdir(parents=True)
            sentinel.write_bytes(f"home {runtime} foreign bytes\n".encode())
        self.home_snapshot = snapshot_tree(self.fake_home)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def fresh_target(self, name: str) -> Path:
        target = self.work / name
        shutil.copytree(FIXTURES / "valid-consumer", target)
        config = target / ".hive/config"
        config.mkdir()
        (config / "harness.toml").write_text(
            "schema_version = 1\nusage_stop_remaining_percent = 10\n",
            encoding="utf-8",
        )
        return target

    def write_json(self, name: str, value: dict[str, Any]) -> Path:
        path = self.input_root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
            + "\n",
            encoding="utf-8",
        )
        return path

    def fresh_capability_input(self, source: Path, label: str) -> Path:
        self.capability_copy_index += 1
        destination = (
            self.input_root
            / "fresh-capabilities"
            / f"{self.capability_copy_index:03d}-{label}-{source.name}"
        )
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())
        return destination

    def run_cli(
        self,
        *arguments: str | Path,
        timeout: float = 15.0,
        extra_environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        environment = os.environ.copy()
        environment["HOME"] = str(self.fake_home)
        if extra_environment:
            environment.update(extra_environment)
        process = subprocess.run(
            [str(self.binary), *(str(argument) for argument in arguments)],
            cwd=ROOT,
            env=environment,
            check=False,
            text=True,
            capture_output=True,
            timeout=timeout,
        )
        try:
            payload = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"CLI did not return one JSON result: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.validators["action"].validate(payload)
        self.assertEqual(payload["exit_code"], process.returncode, payload)
        self.assertEqual(snapshot_tree(self.fake_home), self.home_snapshot)
        return process, payload

    def assert_success(
        self,
        process: subprocess.CompletedProcess[str],
        payload: dict[str, Any],
    ) -> None:
        self.assertEqual(process.returncode, 0, payload)
        self.assertEqual(payload["status"], "success")

    def assert_failure(
        self,
        process: subprocess.CompletedProcess[str],
        payload: dict[str, Any],
        exit_codes: tuple[int, ...] = (2, 3, 4, 5),
    ) -> None:
        self.assertIn(process.returncode, exit_codes, payload)
        self.assertEqual(payload["changed_paths"], [])

    def assert_project_sentinels(self, target: Path) -> None:
        self.assertEqual(
            (target / ".omx/foreign.txt").read_bytes(),
            b"foreign namespace bytes\n",
        )
        self.assertEqual(
            (target / ".omc/foreign.txt").read_bytes(),
            b"foreign OMC namespace bytes\n",
        )

    def ensure_handoff(self, target: Path) -> dict[str, Any]:
        process, payload = self.run_cli(
            "role",
            "handoff",
            "--target",
            target,
            "--request",
            FIXTURES / "role-handoff-request.json",
            "--output",
            "json",
        )
        self.assert_success(process, payload)
        return payload

    def checkpoint_request(
        self,
        *,
        state: str = "executing",
        expected_revision: int = 0,
        passed: list[str] | None = None,
        failed: list[str] | None = None,
        active_roles: list[str] | None = None,
        next_action: str | None = "run build",
        latest_evidence: list[str] | None = None,
        blocker: str | None = None,
        resume_note: str | None = None,
        criterion_evidence: dict[str, list[str]] | None = None,
        updated_at: str = "2026-07-24T00:00:00Z",
    ) -> dict[str, Any]:
        return {
            "schema_version": 1,
            "run_id": "demo",
            "expected_revision": expected_revision,
            "state": state,
            "passed_criteria": passed or [],
            "failed_criteria": failed or [],
            "active_roles": active_roles or ["reviewer"],
            "next_action": next_action,
            "latest_evidence": latest_evidence or [],
            "blocker": blocker,
            "resume_note": resume_note,
            "criterion_evidence": criterion_evidence or {},
            "updated_at": updated_at,
        }

    def checkpoint(
        self,
        target: Path,
        request: dict[str, Any] | Path,
        capability: Path,
        name: str,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        request_path = (
            request
            if isinstance(request, Path)
            else self.write_json(f"{name}-checkpoint.json", request)
        )
        fresh_capability = self.fresh_capability_input(capability, f"{name}-checkpoint")
        return self.run_cli(
            "run",
            "checkpoint",
            "--target",
            target,
            "--request",
            request_path,
            "--capabilities",
            fresh_capability,
            "--output",
            "json",
        )

    def resume(
        self,
        target: Path,
        capability: Path,
        *,
        dispatch_intent: str | None = None,
        account_digest: str | None = None,
        role_id: str | None = None,
        threshold: int | None = None,
        extra_environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        fresh_capability = self.fresh_capability_input(capability, "resume")
        arguments: list[str | Path] = [
            "run",
            "resume",
            "--target",
            target,
            "--run",
            "demo",
            "--capabilities",
            fresh_capability,
            "--output",
            "json",
        ]
        if dispatch_intent is not None:
            arguments.extend(["--dispatch-intent", dispatch_intent])
        if account_digest is not None:
            arguments.extend(["--account-digest", account_digest])
        if role_id is not None:
            arguments.extend(["--role", role_id])
        elif dispatch_intent == "automatic":
            arguments.extend(["--role", "reviewer"])
        if threshold is not None:
            arguments.extend(["--threshold", str(threshold)])
        return self.run_cli(
            *arguments,
            extra_environment=extra_environment,
        )

    def fake_codexbar_environment(
        self,
        case: str,
        *,
        empty: bool = False,
        log: Path | None = None,
    ) -> dict[str, str]:
        fake_bin = self.work / f"fake-codexbar-{case}"
        fake_bin.mkdir(exist_ok=True)
        if not empty:
            if os.name == "nt":
                python = subprocess.list2cmdline([os.sys.executable])
                (fake_bin / "codexbar.cmd").write_text(
                    f"@{python} \"%~dp0\\codexbar.py\" %*\r\n",
                    encoding="utf-8",
                )
                (fake_bin / "codexbar.py").write_text(
                    CODEXBAR_FIXTURE,
                    encoding="utf-8",
                )
            else:
                executable = fake_bin / "codexbar"
                executable.write_text(
                    f"#!{os.sys.executable}\n{CODEXBAR_FIXTURE}",
                    encoding="utf-8",
                )
                executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        environment = {
            "PATH": str(fake_bin),
            "FAKE_CODEXBAR_CASE": case,
        }
        if log is not None:
            environment["FAKE_CODEXBAR_LOG"] = str(log)
        return environment

    def read_status(self, target: Path) -> tuple[dict[str, Any], bytes]:
        status, body = parse_document(target / ".hive/runs/demo/STATUS.md")
        self.validators["status"].validate(status)
        self.assertEqual(body, b"# Run status\n")
        return status, body

    def capability_variant(
        self,
        name: str,
        base: Path,
        *,
        host: str | None = None,
        detection: str | None = None,
        host_version: str | None = None,
        subagents: str | None = None,
        evidence_drift: bool = False,
        invalid_digest: bool = False,
    ) -> Path:
        value = json.loads(base.read_text(encoding="utf-8"))
        if host is not None:
            value["host"] = host
        if host_version is not None:
            value["host_version"] = host_version
        if subagents is not None:
            value["capabilities"]["subagents"] = subagents
        if detection is not None:
            if detection == "absent":
                value["detection"] = "absent"
                value["external_runtime"] = None
                value["resolved_owner"] = "host-native"
                value["evidence"] = [
                    {
                        "source": "host-catalog",
                        "locator": f"fixture:{value['host']}-catalog-empty",
                        "outcome": "absent",
                        "digest": "sha256:" + "a3" * 32,
                    },
                    {
                        "source": "public-executable",
                        "locator": f"fixture:{value['host']}-runtime-missing",
                        "outcome": "absent",
                        "digest": "sha256:" + "a4" * 32,
                    },
                ]
            elif detection == "incompatible":
                runtime = "omx" if value["host"] == "codex" else "omc"
                value["detection"] = "incompatible"
                value["external_runtime"] = runtime
                value["resolved_owner"] = "host-native"
                value["evidence"] = [
                    {
                        "source": "public-executable",
                        "locator": f"fixture:{runtime}-incompatible",
                        "outcome": "incompatible",
                        "digest": "sha256:" + "a5" * 32,
                    }
                ]
            elif detection == "unknown":
                value["detection"] = "unknown"
                value["external_runtime"] = None
                value["resolved_owner"] = "host-native"
                value["evidence"] = [
                    {
                        "source": "host-catalog",
                        "locator": f"fixture:{value['host']}-catalog-unavailable",
                        "outcome": "unavailable",
                        "digest": "sha256:" + "a6" * 32,
                    }
                ]
            else:
                raise AssertionError(f"unsupported test detection: {detection}")
        if evidence_drift:
            value["evidence"][0]["locator"] += "-drift"
        value["evidence_digest"] = canonical_digest(value)
        if invalid_digest:
            value["evidence_digest"] = "sha256:" + "0" * 64
        path = self.write_json(f"{name}-capability.json", value)
        if not invalid_digest:
            self.validators["capability"].validate(value)
            self.assertEqual(value["evidence_digest"], canonical_digest(value))
        return path

    def make_evidence(self, target: Path, name: str, content: bytes) -> str:
        relative = Path(".hive/runs/demo/evidence") / name
        path = target / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return f"{relative.as_posix()}#{digest_bytes(content)}"

    def assert_resume_payload(
        self,
        payload: dict[str, Any],
        *,
        state: str,
        brief_count: int,
        recovery_only: bool,
    ) -> None:
        data = payload["data"]
        self.assertEqual(data["state"], state)
        self.assertIs(data["spawned"], False)
        self.assertIs(data["recovery_only"], recovery_only)
        self.assertEqual(len(data["dispatch_briefs"]), brief_count)
        self.assertIn("usage_guard", data)
        self.assertNotIn("account", data["usage_guard"])
        self.assertNotIn("account_digest", data["usage_guard"])
        self.validators["status"].validate(data["status"])
        for role in data["roles"]:
            self.validators["role"].validate(role["profile"])
        for brief in data["dispatch_briefs"]:
            self.validators["brief"].validate(brief)
            self.assertIs(brief["prepared_only"], True)

    def test_draft_202012_format_validation_covers_all_phase4_documents(self) -> None:
        for name, schema in self.schemas.items():
            with self.subTest(schema=name):
                Draft202012Validator.check_schema(schema)

        checkpoint = json.loads(
            (FIXTURES / "checkpoint-request.json").read_text(encoding="utf-8")
        )
        handoff = json.loads(
            (FIXTURES / "role-handoff-request.json").read_text(encoding="utf-8")
        )
        self.validators["checkpoint_request"].validate(checkpoint)
        self.validators["handoff_request"].validate(handoff)
        for role_path in sorted(
            (FIXTURES / "valid-consumer/.hive/team/roles").glob("*.md")
        ):
            with self.subTest(role=role_path.name):
                role, _ = parse_document(role_path)
                self.validators["role"].validate(role)
        for name, capability_path in CAPABILITIES.items():
            with self.subTest(capability=name):
                self.validators["capability"].validate(
                    json.loads(capability_path.read_text(encoding="utf-8"))
                )

    def test_all_phase1_capability_matrices_parse_and_bind_exact_full_jcs(self) -> None:
        expected = {
            "codex-omx": ("codex", "omx", "omx"),
            "claude-omc": ("claude", "omc", "omc"),
            "absent": ("codex", None, "host-native"),
            "incompatible": ("codex", "omx", "host-native"),
            "unknown": ("codex", None, "host-native"),
        }
        for name, capability_path in CAPABILITIES.items():
            with self.subTest(capability=name):
                capability = json.loads(capability_path.read_text(encoding="utf-8"))
                self.assertEqual(capability["evidence_digest"], canonical_digest(capability))
                target = self.fresh_target(f"capability-{name}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(),
                    capability_path,
                    f"capability-{name}",
                )
                self.assert_success(process, payload)
                status, _ = self.read_status(target)
                self.assertEqual(
                    (status["host"], status["external_runtime"], status["resolved_owner"]),
                    expected[name],
                )
                self.assertEqual(
                    status["resolution_evidence_digest"],
                    capability["evidence_digest"],
                )
                self.assert_project_sentinels(target)

    def test_role_validate_is_read_only_and_preserves_entire_tree(self) -> None:
        before = snapshot_tree(self.target)
        process, payload = self.run_cli(
            "role",
            "validate",
            "--target",
            self.target,
            "--role",
            "reviewer",
            "--output",
            "json",
        )
        self.assert_success(process, payload)
        self.validators["role"].validate(payload["data"]["profile"])
        self.assertEqual(payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.target), before)
        self.assert_project_sentinels(self.target)

    def test_role_validate_rejects_malformed_mismatch_traversal_and_nonregular(self) -> None:
        role_root = self.target / ".hive/team/roles"
        (role_root / "malformed.md").write_bytes(b"---\n{bad}\n---\nbody\n")
        reviewer = (role_root / "reviewer.md").read_bytes()
        (role_root / "mismatch.md").write_bytes(reviewer)
        (role_root / "directory.md").mkdir()
        cases = ["malformed", "mismatch", "../reviewer", "directory", "missing"]
        if os.name != "nt":
            (role_root / "link.md").symlink_to("reviewer.md")
            cases.append("link")
        fifo = role_root / "fifo.md"
        if hasattr(os, "mkfifo"):
            os.mkfifo(fifo)
            cases.append("fifo")
        before = snapshot_tree(self.target)
        for role_id in cases:
            with self.subTest(role=role_id):
                process, payload = self.run_cli(
                    "role",
                    "validate",
                    "--target",
                    self.target,
                    "--role",
                    role_id,
                    "--output",
                    "json",
                    timeout=3.0,
                )
                self.assert_failure(process, payload)
                self.assertEqual(snapshot_tree(self.target), before)
        self.assert_project_sentinels(self.target)

    def test_role_unreadable_file_fails_bounded_without_mutation(self) -> None:
        if os.name == "nt":
            self.skipTest("portable unreadable mode check is POSIX-specific")
        path = self.target / ".hive/team/roles/unreadable.md"
        original = b"unreadable role bytes\n"
        path.write_bytes(original)
        path.chmod(0)
        try:
            process, payload = self.run_cli(
                "role",
                "validate",
                "--target",
                self.target,
                "--role",
                "unreadable",
                "--output",
                "json",
                timeout=3.0,
            )
            self.assert_failure(process, payload)
            self.assertEqual(stat.S_IMODE(path.lstat().st_mode), 0)
        finally:
            path.chmod(0o600)
        self.assertEqual(path.read_bytes(), original)
        self.assert_project_sentinels(self.target)

    def test_handoff_success_retry_body_and_shared_entries_are_exact(self) -> None:
        reviewer = self.target / ".hive/team/roles/reviewer.md"
        writer = self.target / ".hive/team/roles/writer.md"
        reviewer_before = reviewer.read_bytes()
        _, writer_body_before = parse_document(writer)
        process, payload = self.run_cli(
            "role",
            "handoff",
            "--target",
            self.target,
            "--request",
            FIXTURES / "role-handoff-request.json",
            "--output",
            "json",
        )
        self.assert_success(process, payload)
        self.assertEqual(payload["changed_paths"], [".hive/runs/demo/HANDOFF.md"])
        self.assertEqual(reviewer.read_bytes(), reviewer_before)
        handoff_path = self.target / ".hive/runs/demo/HANDOFF.md"
        first_bytes = handoff_path.read_bytes()
        envelope, body = parse_document_bytes(first_bytes)
        self.assertEqual(body, b"# Role handoffs\n")
        self.assertEqual(
            envelope["handoffs"]["reviewer"],
            {
                "markdown": "Next: verify build.\n",
                "updated_at": "2026-07-24T00:00:00Z",
            },
        )

        before_retry = snapshot_tree(self.target)
        retry, retry_payload = self.run_cli(
            "role",
            "handoff",
            "--target",
            self.target,
            "--request",
            FIXTURES / "role-handoff-request.json",
            "--output",
            "json",
        )
        self.assert_success(retry, retry_payload)
        self.assertEqual(retry_payload["code"], "hive.role-handoff-idempotent")
        self.assertEqual(retry_payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.target), before_retry)

        writer_request = {
            "schema_version": 1,
            "role_id": "writer",
            "run_id": "demo",
            "expected_current_assignment": None,
            "expected_handoff_path": None,
            "expected_handoff_digest": digest_bytes(first_bytes),
            "handoff_markdown": "Next: document verified evidence.\n",
            "updated_at": "2026-07-24T00:01:00Z",
        }
        writer_request_path = self.write_json("writer-handoff.json", writer_request)
        writer_process, writer_payload = self.run_cli(
            "role",
            "handoff",
            "--target",
            self.target,
            "--request",
            writer_request_path,
            "--output",
            "json",
        )
        self.assert_success(writer_process, writer_payload)
        self.assertEqual(
            writer_payload["changed_paths"],
            [
                ".hive/runs/demo/HANDOFF.md",
                ".hive/team/roles/writer.md",
            ],
        )
        current_envelope, current_body = parse_document(handoff_path)
        self.assertEqual(current_body, b"# Role handoffs\n")
        self.assertEqual(set(current_envelope["handoffs"]), {"reviewer", "writer"})
        self.assertEqual(
            current_envelope["handoffs"]["reviewer"],
            envelope["handoffs"]["reviewer"],
        )
        writer_profile, writer_body_after = parse_document(writer)
        self.validators["role"].validate(writer_profile)
        self.assertEqual(writer_body_after, writer_body_before)
        self.assertEqual(writer_profile["current_assignment"], "demo")
        self.assertEqual(writer_profile["handoff_path"], ".hive/runs/demo/HANDOFF.md")
        self.assertEqual(reviewer.read_bytes(), reviewer_before)
        self.assert_project_sentinels(self.target)

    def test_handoff_stale_assignment_path_digest_and_failure_are_write_zero(self) -> None:
        self.ensure_handoff(self.target)
        handoff = self.target / ".hive/runs/demo/HANDOFF.md"
        current_digest = digest_bytes(handoff.read_bytes())
        stale_cases = {
            "assignment": {
                "expected_current_assignment": "other-run",
                "expected_handoff_path": ".hive/runs/demo/HANDOFF.md",
                "expected_handoff_digest": current_digest,
            },
            "path": {
                "expected_current_assignment": "demo",
                "expected_handoff_path": None,
                "expected_handoff_digest": current_digest,
            },
            "digest": {
                "expected_current_assignment": "demo",
                "expected_handoff_path": ".hive/runs/demo/HANDOFF.md",
                "expected_handoff_digest": "sha256:" + "0" * 64,
            },
        }
        for name, observed in stale_cases.items():
            with self.subTest(stale=name):
                request = {
                    "schema_version": 1,
                    "role_id": "reviewer",
                    "run_id": "demo",
                    **observed,
                    "handoff_markdown": f"changed {name}\n",
                    "updated_at": "2026-07-24T00:02:00Z",
                }
                path = self.write_json(f"stale-{name}.json", request)
                before = snapshot_tree(self.target)
                process, payload = self.run_cli(
                    "role",
                    "handoff",
                    "--target",
                    self.target,
                    "--request",
                    path,
                    "--output",
                    "json",
                )
                self.assertEqual(process.returncode, 3, payload)
                self.assertEqual(payload["status"], "conflict")
                self.assertEqual(payload["changed_paths"], [])
                self.assertEqual(snapshot_tree(self.target), before)
        self.assert_project_sentinels(self.target)

    def test_handoff_and_checkpoint_reject_impossible_rfc3339_timestamps_write_zero(
        self,
    ) -> None:
        invalid = {
            "month": "2026-13-01T00:00:00Z",
            "day": "2026-02-30T00:00:00Z",
            "time": "2026-07-24T24:00:00Z",
            "offset": "2026-07-24T00:00:00+24:00",
        }
        base_request = json.loads(
            (FIXTURES / "role-handoff-request.json").read_text(encoding="utf-8")
        )
        for name, timestamp in invalid.items():
            with self.subTest(handoff_request=name):
                target = self.fresh_target(f"invalid-request-{name}")
                request = {**base_request, "updated_at": timestamp}
                request_path = self.write_json(f"invalid-handoff-{name}.json", request)
                before = snapshot_tree(target)
                process, payload = self.run_cli(
                    "role",
                    "handoff",
                    "--target",
                    target,
                    "--request",
                    request_path,
                    "--output",
                    "json",
                )
                self.assert_failure(process, payload, (2,))
                self.assertNotIn("data", payload)
                self.assertEqual(snapshot_tree(target), before)
                self.assertFalse((target / ".hive/runs/demo/HANDOFF.md").exists())

        for location in ("envelope", "entry"):
            for name, timestamp in invalid.items():
                with self.subTest(existing_handoff=location, timestamp=name):
                    target = self.fresh_target(f"invalid-existing-{location}-{name}")
                    self.ensure_handoff(target)
                    handoff_path = target / ".hive/runs/demo/HANDOFF.md"
                    envelope, body = parse_document(handoff_path)
                    if location == "envelope":
                        envelope["updated_at"] = timestamp
                    else:
                        envelope["handoffs"]["reviewer"]["updated_at"] = timestamp
                    frontmatter = json.dumps(
                        envelope,
                        ensure_ascii=False,
                        separators=(",", ":"),
                        sort_keys=True,
                    ).encode("utf-8")
                    handoff_path.write_bytes(b"---\n" + frontmatter + b"\n---\n" + body)
                    before = snapshot_tree(target)
                    process, payload = self.checkpoint(
                        target,
                        self.checkpoint_request(),
                        CAPABILITIES["codex-omx"],
                        f"invalid-existing-{location}-{name}",
                    )
                    self.assert_failure(process, payload, (5,))
                    self.assertNotIn("data", payload)
                    self.assertEqual(snapshot_tree(target), before)
                    self.assertFalse((target / ".hive/runs/demo/STATUS.md").exists())

    def test_checkpoint_create_retry_lost_revision_derivation_and_pin_immutability(self) -> None:
        self.ensure_handoff(self.target)
        request = self.checkpoint_request()
        process, payload = self.checkpoint(
            self.target,
            request,
            CAPABILITIES["codex-omx"],
            "lifecycle",
        )
        self.assert_success(process, payload)
        self.assertEqual(payload["changed_paths"], [".hive/runs/demo/STATUS.md"])
        status, _ = self.read_status(self.target)
        self.assertEqual(status["required_criteria"], ["build", "tests"])
        self.assertEqual(status["revision"], 1)
        pin = {key: status[key] for key in OWNER_KEYS}
        status_path = self.target / ".hive/runs/demo/STATUS.md"
        first_bytes = status_path.read_bytes()

        retry, retry_payload = self.checkpoint(
            self.target,
            request,
            CAPABILITIES["codex-omx"],
            "lifecycle",
        )
        self.assert_success(retry, retry_payload)
        self.assertEqual(retry_payload["code"], "hive.run-checkpoint-idempotent")
        self.assertEqual(retry_payload["changed_paths"], [])
        self.assertEqual(status_path.read_bytes(), first_bytes)

        stale = {**request, "updated_at": "2026-07-24T00:00:01Z"}
        stale_process, stale_payload = self.checkpoint(
            self.target,
            stale,
            CAPABILITIES["codex-omx"],
            "lost-revision",
        )
        self.assertEqual(stale_process.returncode, 3, stale_payload)
        self.assertEqual(stale_payload["status"], "conflict")
        self.assertEqual(stale_payload["changed_paths"], [])
        self.assertEqual(status_path.read_bytes(), first_bytes)

        transition = self.checkpoint_request(
            state="verifying",
            expected_revision=1,
            next_action="verify build",
            updated_at="2026-07-24T00:01:00Z",
        )
        next_process, next_payload = self.checkpoint(
            self.target,
            transition,
            CAPABILITIES["codex-omx"],
            "pin-transition",
        )
        self.assert_success(next_process, next_payload)
        next_status, _ = self.read_status(self.target)
        self.assertEqual(next_status["revision"], 2)
        self.assertEqual({key: next_status[key] for key in OWNER_KEYS}, pin)
        self.assert_project_sentinels(self.target)

    def test_checkpoint_and_resume_reject_stale_or_future_capability_before_parse(
        self,
    ) -> None:
        cases = {
            "stale": (time.time() - 120, "older than 60 seconds"),
            "future": (time.time() + 120, "timestamp is in the future"),
        }
        for name, (modified, diagnostic) in cases.items():
            with self.subTest(checkpoint=name):
                target = self.fresh_target(f"freshness-checkpoint-{name}")
                self.ensure_handoff(target)
                capability = self.input_root / f"{name}-invalid-capability.json"
                capability.write_bytes(b"not valid capability JSON\n")
                os.utime(capability, (modified, modified))
                request = self.write_json(
                    f"freshness-{name}-checkpoint.json",
                    self.checkpoint_request(),
                )
                before = snapshot_tree(target)
                process, payload = self.run_cli(
                    "run",
                    "checkpoint",
                    "--target",
                    target,
                    "--request",
                    request,
                    "--capabilities",
                    capability,
                    "--output",
                    "json",
                )
                self.assert_failure(process, payload, (2,))
                self.assertIn(diagnostic, payload["message"])
                self.assertNotIn("data", payload)
                self.assertEqual(snapshot_tree(target), before)
                self.assertFalse((target / ".hive/runs/demo/STATUS.md").exists())

            with self.subTest(resume=name):
                target = self.fresh_target(f"freshness-resume-{name}")
                self.ensure_handoff(target)
                checkpoint, checkpoint_payload = self.checkpoint(
                    target,
                    self.checkpoint_request(),
                    CAPABILITIES["codex-omx"],
                    f"freshness-resume-setup-{name}",
                )
                self.assert_success(checkpoint, checkpoint_payload)
                capability = self.input_root / f"{name}-resume-invalid-capability.json"
                capability.write_bytes(b"not valid capability JSON\n")
                os.utime(capability, (modified, modified))
                before = snapshot_tree(target)
                process, payload = self.run_cli(
                    "run",
                    "resume",
                    "--target",
                    target,
                    "--run",
                    "demo",
                    "--capabilities",
                    capability,
                    "--output",
                    "json",
                )
                self.assert_failure(process, payload, (2,))
                self.assertIn(diagnostic, payload["message"])
                self.assertNotIn("data", payload)
                self.assertNotIn("dispatch_briefs", process.stdout)
                self.assertNotIn('"spawned"', process.stdout)
                self.assertEqual(snapshot_tree(target), before)

    def test_checkpoint_rejects_owner_selection_and_ninety_nine_percent_success(self) -> None:
        for name, mode in (("request-owner", "request"), ("capability-owner", "capability")):
            with self.subTest(owner_selection=name):
                target = self.fresh_target(name)
                self.ensure_handoff(target)
                request = self.checkpoint_request()
                capability = CAPABILITIES["codex-omx"]
                if mode == "request":
                    request["resolved_owner"] = "host-native"
                else:
                    selected = json.loads(capability.read_text(encoding="utf-8"))
                    selected["resolved_owner"] = "host-native"
                    selected["evidence_digest"] = canonical_digest(selected)
                    capability = self.write_json("selected-owner-capability.json", selected)
                before = snapshot_tree(target)
                process, payload = self.checkpoint(
                    target,
                    request,
                    capability,
                    name,
                )
                self.assert_failure(process, payload, (2, 5))
                self.assertEqual(snapshot_tree(target), before)

        target = self.fresh_target("ninety-nine-percent")
        criteria = [f"c{index:02d}" for index in range(100)]
        plan = "# 99 percent is not success\n\n" + "".join(
            f"- [ ] [{criterion}] required\n" for criterion in criteria
        )
        (target / ".hive/runs/demo/PLAN.md").write_text(plan, encoding="utf-8")
        self.ensure_handoff(target)
        locator = self.make_evidence(target, "shared.txt", b"verified bytes\n")
        request = self.checkpoint_request(
            state="succeeded",
            passed=criteria[:99],
            next_action=None,
            criterion_evidence={criterion: [locator] for criterion in criteria[:99]},
        )
        before = snapshot_tree(target)
        process, payload = self.checkpoint(
            target,
            request,
            CAPABILITIES["codex-omx"],
            "ninety-nine-percent",
        )
        self.assertEqual(process.returncode, 5, payload)
        self.assertEqual(payload["status"], "verification-failed")
        self.assertEqual(payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before)
        self.assertFalse((target / ".hive/runs/demo/STATUS.md").exists())

    def test_checkpoint_evidence_is_exact_and_tamper_blocks_resume_without_write(self) -> None:
        self.ensure_handoff(self.target)
        locator = self.make_evidence(self.target, "build.json", b'{"build":"passed"}\n')
        request = self.checkpoint_request(
            state="verifying",
            passed=["build"],
            next_action="verify tests",
            latest_evidence=[locator],
            criterion_evidence={"build": [locator]},
        )
        process, payload = self.checkpoint(
            self.target,
            request,
            CAPABILITIES["codex-omx"],
            "evidence",
        )
        self.assert_success(process, payload)
        status, _ = self.read_status(self.target)
        self.assertEqual(status["latest_evidence"], [locator])
        self.assertEqual(status["criterion_evidence"], {"build": [locator]})
        actual_digest = locator.rsplit("#", 1)[1]
        evidence_item = next(
            item for item in payload["evidence"] if item["locator"] == locator
        )
        self.assertEqual(evidence_item["digest"], actual_digest)

        status_path = self.target / ".hive/runs/demo/STATUS.md"
        status_before = status_path.read_bytes()
        (self.target / ".hive/runs/demo/evidence/build.json").write_bytes(b"tampered\n")
        before_resume = snapshot_tree(self.target)
        resumed, resume_payload = self.resume(self.target, CAPABILITIES["codex-omx"])
        self.assertEqual(resumed.returncode, 5, resume_payload)
        self.assertEqual(resume_payload["status"], "verification-failed")
        self.assertEqual(resume_payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.target), before_resume)
        self.assertEqual(status_path.read_bytes(), status_before)

    def test_resume_executing_and_verifying_prepares_only_without_spawning_or_writes(self) -> None:
        for state in ("executing", "verifying"):
            with self.subTest(state=state):
                target = self.fresh_target(f"resume-{state}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(
                        state=state,
                        next_action=f"{state} next action",
                    ),
                    CAPABILITIES["codex-omx"],
                    f"resume-{state}",
                )
                self.assert_success(process, payload)
                before = snapshot_tree(target)
                empty_path = self.work / f"manual-no-sensor-{state}"
                empty_path.mkdir()
                resumed, resume_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    extra_environment={"PATH": str(empty_path)},
                )
                self.assert_success(resumed, resume_payload)
                self.assertEqual(resume_payload["code"], "hive.run-resume-prepared")
                self.assert_resume_payload(
                    resume_payload,
                    state=state,
                    brief_count=1,
                    recovery_only=False,
                )
                self.assertEqual(
                    resume_payload["data"]["usage_guard"],
                    {
                        "dispatch_intent": "manual",
                        "enforced": False,
                        "outcome": "not_requested",
                        "evidence_digest": None,
                        "window": None,
                    },
                )
                self.assertNotIn("usage-snapshots:", resumed.stdout)
                self.assertEqual(snapshot_tree(target), before)
                self.assert_project_sentinels(target)

    def test_automatic_resume_allows_once_with_sanitized_usage_evidence(self) -> None:
        target = self.fresh_target("automatic-allow")
        self.ensure_handoff(target)
        process, payload = self.checkpoint(
            target,
            self.checkpoint_request(state="executing"),
            CAPABILITIES["codex-omx"],
            "automatic-allow",
        )
        self.assert_success(process, payload)
        sensor_log = self.work / "automatic-allow.log"
        before = snapshot_tree(target)

        resumed, resume_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            extra_environment=self.fake_codexbar_environment(
                "allow",
                log=sensor_log,
            ),
        )

        self.assert_success(resumed, resume_payload)
        self.assertEqual(resume_payload["code"], "hive.run-resume-prepared")
        self.assert_resume_payload(
            resume_payload,
            state="executing",
            brief_count=1,
            recovery_only=False,
        )
        usage_guard = resume_payload["data"]["usage_guard"]
        self.assertEqual(usage_guard["dispatch_intent"], "automatic")
        self.assertIs(usage_guard["enforced"], True)
        self.assertEqual(usage_guard["outcome"], "authorized")
        self.assertEqual(usage_guard["window"], "session")
        self.assertRegex(usage_guard["evidence_digest"], r"^sha256:[0-9a-f]{64}$")
        self.assertRegex(usage_guard["authorization_id"], r"^sha256:[0-9a-f]{64}$")
        self.assertEqual(usage_guard["role_id"], "reviewer")
        self.assertEqual(usage_guard["history"], "absent")
        self.assertNotIn(RAW_USAGE_ACCOUNT, resumed.stdout)
        self.assertEqual(len(resume_payload["changed_paths"]), 2)
        for changed in resume_payload["changed_paths"]:
            self.assertTrue(changed.startswith(".hive/runtime/"), changed)
            self.assertNotIn(
                RAW_USAGE_ACCOUNT,
                (target / changed).read_text(encoding="utf-8"),
            )
        after = snapshot_tree(target)
        for path, value in before.items():
            self.assertEqual(after[path], value)
        invocations = [
            json.loads(line)
            for line in sensor_log.read_text(encoding="utf-8").splitlines()
        ]
        self.assertEqual(
            invocations,
            [
                ["--version"],
                [
                    "usage",
                    "--provider",
                    "codex",
                    "--all-accounts",
                    "--source",
                    "cli",
                    "--format",
                    "json",
                    "--json-only",
                ],
            ],
        )

    def test_automatic_resume_blocks_at_ten_percent_or_unknown_sensor(self) -> None:
        for case, sensor_case, threshold, expected_code, expected_outcome in (
            ("default-threshold", "threshold", None, "hive.usage-limited", "limited"),
            ("configured-threshold", "allow", 60, "hive.usage-limited", "limited"),
            ("missing", "missing", None, "hive.usage-unknown", "unknown"),
        ):
            with self.subTest(case=case):
                target = self.fresh_target(f"automatic-{case}")
                if threshold is not None:
                    (target / ".hive/config/harness.toml").write_text(
                        "schema_version = 1\n"
                        f"usage_stop_remaining_percent = {threshold}\n",
                        encoding="utf-8",
                    )
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(state="executing"),
                    CAPABILITIES["codex-omx"],
                    f"automatic-{case}",
                )
                self.assert_success(process, payload)
                before = snapshot_tree(target)

                resumed, resume_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    dispatch_intent="automatic",
                    account_digest=USAGE_ACCOUNT_DIGEST,
                    threshold=threshold,
                    extra_environment=self.fake_codexbar_environment(
                        sensor_case,
                        empty=case == "missing",
                    ),
                )

                self.assertEqual(resumed.returncode, 3, resume_payload)
                self.assertEqual(resume_payload["code"], expected_code)
                self.assert_resume_payload(
                    resume_payload,
                    state="executing",
                    brief_count=0,
                    recovery_only=True,
                )
                usage_guard = resume_payload["data"]["usage_guard"]
                self.assertIs(usage_guard["enforced"], False)
                self.assertEqual(usage_guard["outcome"], expected_outcome)
                self.assertNotIn(RAW_USAGE_ACCOUNT, resumed.stdout)
                after = snapshot_tree(target)
                if case == "missing":
                    self.assertEqual(after, before)
                else:
                    self.assertEqual(
                        resume_payload["changed_paths"],
                        [
                            next(
                                path
                                for path in after
                                if path.startswith(".hive/runtime/usage-history/")
                            )
                        ],
                    )
                    for path, value in before.items():
                        self.assertEqual(after[path], value)

    def test_automatic_resume_session_precedes_low_weekly(self) -> None:
        target = self.fresh_target("automatic-session-precedence")
        self.ensure_handoff(target)
        process, payload = self.checkpoint(
            target,
            self.checkpoint_request(state="executing"),
            CAPABILITIES["codex-omx"],
            "automatic-session-precedence",
        )
        self.assert_success(process, payload)

        resumed, resume_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            extra_environment=self.fake_codexbar_environment(
                "weekly-low-session-high"
            ),
        )

        self.assert_success(resumed, resume_payload)
        self.assert_resume_payload(
            resume_payload,
            state="executing",
            brief_count=1,
            recovery_only=False,
        )
        self.assertEqual(
            resume_payload["data"]["usage_guard"]["window"],
            "session",
        )

    def test_automatic_threshold_is_bound_to_installed_config(self) -> None:
        target = self.fresh_target("automatic-threshold-binding")
        (target / ".hive/config/harness.toml").write_text(
            "schema_version = 1\nusage_stop_remaining_percent = 20\n",
            encoding="utf-8",
        )
        self.ensure_handoff(target)
        process, payload = self.checkpoint(
            target,
            self.checkpoint_request(state="executing"),
            CAPABILITIES["codex-omx"],
            "automatic-threshold-binding",
        )
        self.assert_success(process, payload)
        sensor_log = self.work / "threshold-binding.log"
        before = snapshot_tree(target)

        resumed, resume_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            threshold=10,
            extra_environment=self.fake_codexbar_environment(
                "allow",
                log=sensor_log,
            ),
        )

        self.assertEqual(resumed.returncode, 2, resume_payload)
        self.assertEqual(resume_payload["code"], "hive.invalid-input")
        self.assertIn("installed usage_stop_remaining_percent (20)", resumed.stdout)
        self.assertFalse(sensor_log.exists())
        self.assertEqual(snapshot_tree(target), before)

    def test_automatic_config_missing_malformed_or_symlink_fails_closed(self) -> None:
        cases = ("missing", "malformed", "symlink")
        for case in cases:
            if case == "symlink" and os.name == "nt":
                continue
            with self.subTest(case=case):
                target = self.fresh_target(f"automatic-config-{case}")
                config = target / ".hive/config/harness.toml"
                if case == "missing":
                    config.unlink()
                elif case == "malformed":
                    config.write_text(
                        "usage_stop_remaining_percent = 10\n"
                        "usage_stop_remaining_percent = 9\n",
                        encoding="utf-8",
                    )
                else:
                    external = self.work / "foreign-harness.toml"
                    external.write_text(
                        "usage_stop_remaining_percent = 10\n",
                        encoding="utf-8",
                    )
                    config.unlink()
                    config.symlink_to(external)
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(state="executing"),
                    CAPABILITIES["codex-omx"],
                    f"automatic-config-{case}",
                )
                self.assert_success(process, payload)
                before = snapshot_tree(target)
                resumed, resume_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    dispatch_intent="automatic",
                    account_digest=USAGE_ACCOUNT_DIGEST,
                    extra_environment=self.fake_codexbar_environment("allow"),
                )
                self.assertEqual(resumed.returncode, 3, resume_payload)
                self.assertEqual(resume_payload["code"], "hive.run-blocked")
                self.assertEqual(resume_payload["changed_paths"], [])
                self.assertEqual(snapshot_tree(target), before)

    def test_automatic_history_rejects_regressions_and_tampering(self) -> None:
        reset_at = "2026-07-25T00:00:00Z"
        for case in ("remaining-increase", "measurement-regression"):
            with self.subTest(case=case):
                target = self.fresh_target(f"automatic-history-{case}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(state="executing"),
                    CAPABILITIES["codex-omx"],
                    f"automatic-history-first-{case}",
                )
                self.assert_success(process, payload)
                first, first_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    dispatch_intent="automatic",
                    account_digest=USAGE_ACCOUNT_DIGEST,
                    extra_environment={
                        **self.fake_codexbar_environment("allow"),
                        "FAKE_CODEXBAR_RESET_AT": reset_at,
                    },
                )
                self.assert_success(first, first_payload)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(
                        state="executing",
                        expected_revision=1,
                        updated_at="2026-07-24T00:01:00Z",
                    ),
                    CAPABILITIES["codex-omx"],
                    f"automatic-history-second-{case}",
                )
                self.assert_success(process, payload)
                second, second_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    dispatch_intent="automatic",
                    account_digest=USAGE_ACCOUNT_DIGEST,
                    extra_environment={
                        **self.fake_codexbar_environment(case),
                        "FAKE_CODEXBAR_RESET_AT": reset_at,
                    },
                )
                self.assertEqual(second.returncode, 3, second_payload)
                self.assertEqual(second_payload["code"], "hive.usage-unknown")
                self.assertEqual(
                    second_payload["data"]["usage_guard"]["history"],
                    "available",
                )
                self.assertEqual(second_payload["data"]["dispatch_briefs"], [])

        for corruption in ("tampered", "symlink"):
            if corruption == "symlink" and os.name == "nt":
                continue
            with self.subTest(corruption=corruption):
                target = self.fresh_target(f"automatic-history-{corruption}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(state="executing"),
                    CAPABILITIES["codex-omx"],
                    f"automatic-history-{corruption}",
                )
                self.assert_success(process, payload)
                history = target / ".hive/runtime/usage-history"
                history.mkdir(parents=True)
                key = hashlib.sha256(USAGE_ACCOUNT_DIGEST.encode()).hexdigest()
                record = history / f"{key}.json"
                if corruption == "tampered":
                    record.write_text('{"schema_version":1}\n', encoding="utf-8")
                else:
                    external = self.work / "foreign-usage-history.json"
                    external.write_text('{"schema_version":1}\n', encoding="utf-8")
                    record.symlink_to(external)
                before = snapshot_tree(target)
                resumed, resume_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                    dispatch_intent="automatic",
                    account_digest=USAGE_ACCOUNT_DIGEST,
                    extra_environment=self.fake_codexbar_environment("allow"),
                )
                self.assertEqual(resumed.returncode, 3, resume_payload)
                self.assertEqual(resume_payload["code"], "hive.run-blocked")
                self.assertEqual(snapshot_tree(target), before)

    def test_automatic_authorization_is_one_role_one_brief_and_not_reissued(self) -> None:
        target = self.fresh_target("automatic-one-brief")
        self.ensure_handoff(target)
        writer_request = {
            "schema_version": 1,
            "role_id": "writer",
            "run_id": "demo",
            "expected_current_assignment": None,
            "expected_handoff_path": None,
            "expected_handoff_digest": digest_bytes(
                (target / ".hive/runs/demo/HANDOFF.md").read_bytes()
            ),
            "handoff_markdown": "Next: write docs.\n",
            "updated_at": "2026-07-24T00:00:01Z",
        }
        writer_request_path = self.write_json(
            "automatic-one-brief-writer.json",
            writer_request,
        )
        writer_process, writer_payload = self.run_cli(
            "role",
            "handoff",
            "--target",
            target,
            "--request",
            writer_request_path,
            "--output",
            "json",
        )
        self.assert_success(writer_process, writer_payload)
        process, payload = self.checkpoint(
            target,
            self.checkpoint_request(
                state="executing",
                active_roles=["reviewer", "writer"],
            ),
            CAPABILITIES["codex-omx"],
            "automatic-one-brief",
        )
        self.assert_success(process, payload)
        sensor_log = self.work / "automatic-one-brief.log"
        first, first_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            role_id="writer",
            extra_environment=self.fake_codexbar_environment(
                "allow",
                log=sensor_log,
            ),
        )
        self.assert_success(first, first_payload)
        briefs = first_payload["data"]["dispatch_briefs"]
        self.assertEqual(len(briefs), 1)
        self.assertEqual(briefs[0]["role_id"], "writer")
        authorization_id = first_payload["data"]["usage_guard"]["authorization_id"]
        invocation_count = len(sensor_log.read_text(encoding="utf-8").splitlines())

        replay, replay_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            role_id="writer",
            extra_environment=self.fake_codexbar_environment(
                "allow",
                log=sensor_log,
            ),
        )
        self.assertEqual(replay.returncode, 3, replay_payload)
        self.assertEqual(replay_payload["code"], "hive.usage-unknown")
        self.assertEqual(replay_payload["data"]["dispatch_briefs"], [])
        self.assertEqual(
            replay_payload["data"]["usage_guard"]["outcome"],
            "already_issued",
        )
        self.assertEqual(
            replay_payload["data"]["usage_guard"]["authorization_id"],
            authorization_id,
        )
        self.assertEqual(
            len(sensor_log.read_text(encoding="utf-8").splitlines()),
            invocation_count,
        )
        authorization_file = next(
            (target / ".hive/runtime/dispatch-authorizations").glob("*.json")
        )
        authorization_file.write_text('{"schema_version":1}\n', encoding="utf-8")
        before_tampered_retry = snapshot_tree(target)
        tampered, tampered_payload = self.resume(
            target,
            CAPABILITIES["codex-omx"],
            dispatch_intent="automatic",
            account_digest=USAGE_ACCOUNT_DIGEST,
            role_id="writer",
            extra_environment=self.fake_codexbar_environment(
                "allow",
                log=sensor_log,
            ),
        )
        self.assertEqual(tampered.returncode, 3, tampered_payload)
        self.assertEqual(tampered_payload["code"], "hive.run-blocked")
        self.assertEqual(tampered_payload["changed_paths"], [])
        self.assertEqual(snapshot_tree(target), before_tampered_retry)
        self.assertEqual(
            len(sensor_log.read_text(encoding="utf-8").splitlines()),
            invocation_count,
        )

    def test_resume_ready_is_recovery_only_and_makes_no_hidden_transition(self) -> None:
        self.ensure_handoff(self.target)
        process, payload = self.checkpoint(
            self.target,
            self.checkpoint_request(
                state="resume-ready",
                next_action="resume explicit work",
                resume_note="fresh session may resume",
            ),
            CAPABILITIES["codex-omx"],
            "resume-ready",
        )
        self.assert_success(process, payload)
        before = snapshot_tree(self.target)
        resumed, resume_payload = self.resume(self.target, CAPABILITIES["codex-omx"])
        self.assert_success(resumed, resume_payload)
        self.assertEqual(resume_payload["code"], "hive.run-recovery-loaded")
        self.assert_resume_payload(
            resume_payload,
            state="resume-ready",
            brief_count=0,
            recovery_only=True,
        )
        self.assertEqual(
            resume_payload["data"]["status"]["resume_note"],
            "fresh session may resume",
        )
        self.assertEqual(snapshot_tree(self.target), before)

    def test_blocked_and_usage_limited_resume_exit_three_without_brief_or_spawn(self) -> None:
        for state in ("blocked", "usage-limited"):
            with self.subTest(state=state):
                target = self.fresh_target(f"blocked-{state}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(
                        state=state,
                        next_action="wait for explicit recovery",
                        blocker=f"{state} condition",
                    ),
                    CAPABILITIES["codex-omx"],
                    f"blocked-{state}",
                )
                self.assert_success(process, payload)
                before = snapshot_tree(target)
                resumed, resume_payload = self.resume(
                    target,
                    CAPABILITIES["codex-omx"],
                )
                self.assertEqual(resumed.returncode, 3, resume_payload)
                self.assertEqual(resume_payload["status"], "blocked")
                self.assert_resume_payload(
                    resume_payload,
                    state=state,
                    brief_count=0,
                    recovery_only=True,
                )
                self.assertEqual(snapshot_tree(target), before)

    def test_unsupported_and_unverified_resume_exit_four_without_dispatch_data(self) -> None:
        for support in ("unsupported", "unverified"):
            with self.subTest(support=support):
                target = self.fresh_target(f"support-{support}")
                self.ensure_handoff(target)
                capability = self.capability_variant(
                    f"support-{support}",
                    CAPABILITIES["codex-omx"],
                    subagents=support,
                )
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(),
                    capability,
                    f"support-{support}",
                )
                self.assert_success(process, payload)
                before = snapshot_tree(target)
                resumed, resume_payload = self.resume(target, capability)
                self.assertEqual(resumed.returncode, 4, resume_payload)
                self.assertEqual(resume_payload["status"], "unsupported")
                self.assertEqual(resume_payload["code"], "hive.run-unsupported")
                self.assertNotIn("data", resume_payload)
                self.assertNotIn("dispatch_briefs", resumed.stdout)
                self.assertNotIn('"spawned"', resumed.stdout)
                self.assertEqual(snapshot_tree(target), before)

    def test_succeeded_resume_is_terminal_recovery_without_continuation(self) -> None:
        self.ensure_handoff(self.target)
        build = self.make_evidence(self.target, "build.txt", b"build passed\n")
        tests = self.make_evidence(self.target, "tests.txt", b"tests passed\n")
        process, payload = self.checkpoint(
            self.target,
            self.checkpoint_request(
                state="succeeded",
                passed=["build", "tests"],
                next_action=None,
                latest_evidence=[build, tests],
                criterion_evidence={"build": [build], "tests": [tests]},
            ),
            CAPABILITIES["codex-omx"],
            "succeeded",
        )
        self.assert_success(process, payload)
        before = snapshot_tree(self.target)
        resumed, resume_payload = self.resume(self.target, CAPABILITIES["codex-omx"])
        self.assert_success(resumed, resume_payload)
        self.assert_resume_payload(
            resume_payload,
            state="succeeded",
            brief_count=0,
            recovery_only=True,
        )
        self.assertIsNone(resume_payload["next_action"])
        self.assertIsNone(resume_payload["data"]["next_action"])
        self.assertEqual(snapshot_tree(self.target), before)

    def test_omx_and_omc_owner_drift_never_switches_or_writes(self) -> None:
        for owner_name, base in (
            ("omx", CAPABILITIES["codex-omx"]),
            ("omc", CAPABILITIES["claude-omc"]),
        ):
            with self.subTest(owner=owner_name):
                target = self.fresh_target(f"drift-{owner_name}")
                self.ensure_handoff(target)
                process, payload = self.checkpoint(
                    target,
                    self.checkpoint_request(),
                    base,
                    f"drift-{owner_name}",
                )
                self.assert_success(process, payload)
                host = "codex" if owner_name == "omx" else "claude"
                variants = {
                    "missing": (
                        self.capability_variant(
                            f"{owner_name}-missing",
                            base,
                            host=host,
                            detection="absent",
                        ),
                        3,
                    ),
                    "incompatible": (
                        self.capability_variant(
                            f"{owner_name}-incompatible",
                            base,
                            host=host,
                            detection="incompatible",
                        ),
                        4,
                    ),
                    "version": (
                        self.capability_variant(
                            f"{owner_name}-version",
                            base,
                            host_version="fixture-drift",
                        ),
                        3,
                    ),
                    "evidence": (
                        self.capability_variant(
                            f"{owner_name}-evidence",
                            base,
                            evidence_drift=True,
                        ),
                        3,
                    ),
                    "invalid-digest": (
                        self.capability_variant(
                            f"{owner_name}-invalid-digest",
                            base,
                            invalid_digest=True,
                        ),
                        5,
                    ),
                }
                for drift_name, (variant, expected_exit) in variants.items():
                    with self.subTest(owner=owner_name, drift=drift_name):
                        before = snapshot_tree(target)
                        resumed, resume_payload = self.resume(target, variant)
                        self.assertEqual(resumed.returncode, expected_exit, resume_payload)
                        self.assertEqual(resume_payload["changed_paths"], [])
                        self.assertNotIn("data", resume_payload)
                        self.assertEqual(snapshot_tree(target), before)
                        status, _ = self.read_status(target)
                        self.assertEqual(status["resolved_owner"], owner_name)
                self.assert_project_sentinels(target)

    def test_compatible_external_runtime_projects_data_skills_but_no_duplicate_orchestration(self) -> None:
        for host, capability, discovery in (
            ("codex", CAPABILITIES["codex-omx"], ".agents/skills"),
            ("claude", CAPABILITIES["claude-omc"], ".claude/skills"),
        ):
            with self.subTest(host=host):
                target = self.work / f"projection-{host}"
                target.mkdir()
                for runtime, content in (
                    (".omx", b"project OMX sentinel\n"),
                    (".omc", b"project OMC sentinel\n"),
                ):
                    sentinel = target / runtime / "foreign.txt"
                    sentinel.parent.mkdir()
                    sentinel.write_bytes(content)
                answers = yaml.safe_load(
                    (PHASE1 / "answers-no-role-no-hook.yml").read_text(encoding="utf-8")
                )
                answers["primary_host"] = host
                answers_path = self.input_root / f"projection-{host}-answers.yml"
                answers_path.write_text(
                    yaml.safe_dump(answers, sort_keys=False),
                    encoding="utf-8",
                )
                process, payload = self.run_cli(
                    "setup",
                    "--target",
                    target,
                    "--answers",
                    answers_path,
                    "--capabilities",
                    capability,
                    "--apply",
                    "--output",
                    "json",
                    timeout=30.0,
                )
                self.assert_success(process, payload)
                root = target / discovery
                projected = {
                    path.parent.name
                    for path in root.glob("*/SKILL.md")
                    if path.is_file()
                }
                self.assertTrue(DATA_SKILLS.issubset(projected))
                self.assertTrue(
                    {"plan", "ralph", "team", "loop", "autopilot"}.isdisjoint(projected)
                )
                self.assertFalse((target / ".hive/hooks").exists())
                self.assertFalse(
                    (target / ".hive/config/approved-hooks.yml").exists()
                )
                for skill_path in root.glob("*/SKILL.md"):
                    for line in skill_path.read_text(encoding="utf-8").splitlines():
                        command = line.strip().casefold()
                        self.assertFalse(command.startswith(("omx ", "omc ")), line)
                        self.assertFalse(
                            command.startswith(
                                (
                                    "hive plan ",
                                    "hive team ",
                                    "hive ralph ",
                                    "hive loop ",
                                )
                            ),
                            line,
                        )
                self.assertEqual(
                    (target / ".omx/foreign.txt").read_bytes(),
                    b"project OMX sentinel\n",
                )
                self.assertEqual(
                    (target / ".omc/foreign.txt").read_bytes(),
                    b"project OMC sentinel\n",
                )

    def test_target_source_traversal_symlink_and_non_directory_are_rejected(self) -> None:
        source_marker = ROOT / "hive-source.json"
        marker_before = source_marker.read_bytes()
        cases: list[tuple[str, Path]] = [
            ("source-root", ROOT),
            ("traversal", self.target / ".." / self.target.name),
        ]
        target_file = self.work / "not-a-target"
        target_file.write_bytes(b"not a directory\n")
        cases.append(("regular-file", target_file))
        if os.name != "nt":
            link = self.work / "target-link"
            link.symlink_to(self.target, target_is_directory=True)
            cases.append(("symlink", link))
        target_before = snapshot_tree(self.target)
        for name, target in cases:
            with self.subTest(target=name):
                process, payload = self.run_cli(
                    "role",
                    "validate",
                    "--target",
                    target,
                    "--role",
                    "reviewer",
                    "--output",
                    "json",
                )
                self.assert_failure(process, payload, (3,))
                self.assertEqual(snapshot_tree(self.target), target_before)
                self.assertEqual(source_marker.read_bytes(), marker_before)

    def test_explicit_request_and_capability_inputs_reject_traversal_symlink_and_nonregular(self) -> None:
        valid_request = self.write_json(
            "explicit-valid-request.json",
            json.loads(
                (FIXTURES / "role-handoff-request.json").read_text(encoding="utf-8")
            ),
        )
        nested = self.input_root / "nested"
        nested.mkdir()
        request_cases: list[tuple[str, Path]] = [
            ("traversal", nested / ".." / valid_request.name),
        ]
        request_directory = self.input_root / "request-directory"
        request_directory.mkdir()
        request_cases.append(("directory", request_directory))
        if os.name != "nt":
            request_link = self.input_root / "request-link.json"
            request_link.symlink_to(valid_request)
            request_cases.append(("symlink", request_link))
        request_fifo = self.input_root / "request.fifo"
        if hasattr(os, "mkfifo"):
            os.mkfifo(request_fifo)
            request_cases.append(("fifo", request_fifo))
        for name, request_path in request_cases:
            with self.subTest(request=name):
                before = snapshot_tree(self.target)
                process, payload = self.run_cli(
                    "role",
                    "handoff",
                    "--target",
                    self.target,
                    "--request",
                    request_path,
                    "--output",
                    "json",
                    timeout=3.0,
                )
                self.assert_failure(process, payload, (2, 3))
                self.assertEqual(snapshot_tree(self.target), before)

        capability_directory = self.input_root / "capability-directory"
        capability_directory.mkdir()
        capability_cases: list[tuple[str, Path]] = [
            (
                "traversal",
                CAPABILITIES["codex-omx"].parent
                / ".."
                / "phase1"
                / CAPABILITIES["codex-omx"].name,
            ),
            ("directory", capability_directory),
        ]
        if os.name != "nt":
            capability_link = self.input_root / "capability-link.json"
            capability_link.symlink_to(CAPABILITIES["codex-omx"])
            capability_cases.append(("symlink", capability_link))
        capability_fifo = self.input_root / "capability.fifo"
        if hasattr(os, "mkfifo"):
            os.mkfifo(capability_fifo)
            capability_cases.append(("fifo", capability_fifo))
        for name, capability_path in capability_cases:
            with self.subTest(capability=name):
                before = snapshot_tree(self.target)
                process, payload = self.run_cli(
                    "run",
                    "checkpoint",
                    "--target",
                    self.target,
                    "--request",
                    FIXTURES / "checkpoint-request.json",
                    "--capabilities",
                    capability_path,
                    "--output",
                    "json",
                    timeout=3.0,
                )
                self.assert_failure(process, payload, (2, 3))
                self.assertEqual(snapshot_tree(self.target), before)
        self.assert_project_sentinels(self.target)

    def test_unreadable_explicit_input_fails_without_read_or_write(self) -> None:
        if os.name == "nt":
            self.skipTest("portable unreadable mode check is POSIX-specific")
        request = self.write_json(
            "unreadable-request.json",
            json.loads(
                (FIXTURES / "role-handoff-request.json").read_text(encoding="utf-8")
            ),
        )
        original = request.read_bytes()
        request.chmod(0)
        before = snapshot_tree(self.target)
        try:
            process, payload = self.run_cli(
                "role",
                "handoff",
                "--target",
                self.target,
                "--request",
                request,
                "--output",
                "json",
                timeout=3.0,
            )
            self.assert_failure(process, payload, (2,))
            self.assertEqual(snapshot_tree(self.target), before)
        finally:
            request.chmod(0o600)
        self.assertEqual(request.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
