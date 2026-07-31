"""Shared black-box support for Phase 1 conformance tests."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_ROOT = REPOSITORY_ROOT / "tests/fixtures/phase1"
EXPECTED_ROOT = FIXTURE_ROOT / "expected"
ACTION_RESULT_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8")
)
FRESH_HOOK_CAPABILITIES_PATH = Path(
    ".hive/runtime/current-capability-resolution.json"
)


def read_yaml(path: Path) -> dict[str, object]:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected YAML object: {path}")
    return value


def write_yaml(path: Path, value: object) -> None:
    path.write_text(
        yaml.safe_dump(value, allow_unicode=True, sort_keys=False),
        encoding="utf-8",
    )


def write_operational_user_setup(root: Path) -> None:
    config = root / ".hive/config"
    config.mkdir(parents=True, exist_ok=True)
    write_yaml(
        config / "user-setup.yml",
        {
            "schema_version": 1,
            "interface_language": "en",
            "wiki": {"enabled": True, "language": "both"},
            "profile": {"id": "web-developer"},
            "persona": {"id": "balanced"},
            "selected_hosts": ["codex", "claude", "antigravity"],
            "skills": {
                "mode": "individual",
                "selected": [
                    "auto-setup-harness",
                    "hive-judge-package",
                    "hive-knowledge-capture",
                    "hive-knowledge-maintenance",
                    "hive-knowledge-promote",
                    "hive-knowledge-query",
                    "hive-migrate",
                    "hive-project-upgrade",
                    "hive-prompt-refine",
                    "hive-role-handoff",
                    "hive-run-checkpoint",
                    "hive-run-resume",
                    "hive-simple-question",
                    "hive-update",
                    "hive-usage-guard",
                    "setup-harness",
                    "setup-hive",
                ],
            },
            "usage_guard": {
                "enabled": True,
                "stop_remaining_percent": 20,
                "codexbar_fallback_enabled": True,
            },
        },
    )


def snapshot_tree(root: Path) -> dict[str, tuple[str, bytes | str]]:
    snapshot: dict[str, tuple[str, bytes | str]] = {}
    if not root.exists():
        return snapshot
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            snapshot[relative] = ("symlink", os.readlink(path))
        elif path.is_file():
            snapshot[relative] = ("file", path.read_bytes())
        elif path.is_dir():
            snapshot[relative] = ("directory", "")
    return snapshot


def recursive_strings(value: object) -> list[str]:
    strings: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            strings.append(str(key))
            strings.extend(recursive_strings(child))
    elif isinstance(value, list):
        for child in value:
            strings.extend(recursive_strings(child))
    elif isinstance(value, str):
        strings.append(value)
    return strings


class Phase1CliTestCase(unittest.TestCase):
    """Build once and execute Hive only inside disposable consumer projects."""

    @classmethod
    def setUpClass(cls) -> None:
        configured_binary = os.environ.get("HIVE_BIN")
        if configured_binary:
            cls.hive_binary = Path(configured_binary).resolve()
            return
        subprocess.run(
            ["cargo", "build", "--quiet", "--bin", "hive"],
            cwd=REPOSITORY_ROOT,
            check=True,
        )
        executable = "hive.exe" if os.name == "nt" else "hive"
        cls.hive_binary = REPOSITORY_ROOT / "target/debug" / executable

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory(
            prefix="aigent-hive-phase1-expanded-"
        )
        self.work_root = Path(self.temporary_directory.name).resolve()
        self.setup_user_root = self.work_root / "user-root"
        write_operational_user_setup(self.setup_user_root)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def invoke_setup(
        self,
        target: Path,
        *,
        answers: Path | None = None,
        capabilities: Path | str = "capabilities-codex-omx.json",
        mode: str = "--apply",
        reconfigure_roles: tuple[str, ...] = (),
        extra_arguments: tuple[str, ...] = (),
        timeout: float | None = None,
        environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        capability_path = (
            capabilities
            if isinstance(capabilities, Path)
            else FIXTURE_ROOT / capabilities
        )
        command = [
            str(self.hive_binary),
            "setup",
            "--target",
            str(target),
            "--answers",
            str(answers or FIXTURE_ROOT / "answers-base.yml"),
            "--capabilities",
            str(capability_path),
            "--user-root",
            str(self.setup_user_root),
            mode,
        ]
        for role_id in reconfigure_roles:
            command.extend(["--reconfigure-role", role_id])
        command.extend(extra_arguments)
        command.extend(["--output", "json"])
        process = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
            timeout=timeout,
            env=(
                {**os.environ, **environment}
                if environment is not None
                else None
            ),
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"stdout must be exactly one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.assertIsInstance(result, dict)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"])
        return process, result

    def invoke_hook(
        self,
        consumer: Path,
        *,
        capability: str,
        event: str,
        capabilities: Path | str | None = "capabilities-absent.json",
        input_path: Path | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command = [
            str(self.hive_binary),
            "hook",
            "--capability",
            capability,
            "--event",
            event,
        ]
        if capabilities is not None:
            capability_source = (
                capabilities
                if isinstance(capabilities, Path)
                else FIXTURE_ROOT / capabilities
            )
            capability_path = consumer / FRESH_HOOK_CAPABILITIES_PATH
            runtime_directory = capability_path.parent
            runtime_directory_preexisted = runtime_directory.exists()
            prior_capabilities = (
                capability_path.read_bytes()
                if capability_path.is_file()
                else None
            )
            runtime_directory.mkdir(parents=True, exist_ok=True)
            capability_path.write_bytes(capability_source.read_bytes())
            command.extend(
                ["--capabilities", FRESH_HOOK_CAPABILITIES_PATH.as_posix()]
            )
        else:
            capability_path = None
            runtime_directory = None
            runtime_directory_preexisted = False
            prior_capabilities = None
        command.extend(
            [
                "--input",
                str(input_path or FIXTURE_ROOT / "stop-input.json"),
                "--output",
                "json",
            ]
        )
        try:
            process = subprocess.run(
                command,
                cwd=consumer,
                check=False,
                text=True,
                capture_output=True,
                env={**os.environ, "PATH": ""},
            )
        finally:
            if capability_path is not None:
                if prior_capabilities is None:
                    capability_path.unlink(missing_ok=True)
                else:
                    capability_path.write_bytes(prior_capabilities)
                if (
                    runtime_directory is not None
                    and not runtime_directory_preexisted
                ):
                    runtime_directory.rmdir()
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"hook stdout must be exactly one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        self.assertIsInstance(result, dict)
        return process, result

    def assert_neutral_stop(
        self,
        process: subprocess.CompletedProcess[str],
        result: dict[str, object],
    ) -> None:
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result.get("decision"), "allow")
        for value in recursive_strings(result):
            lowered = value.lower()
            self.assertNotIn("block", lowered)
            self.assertNotIn("continue", lowered)
            self.assertNotIn("prompt", lowered)

    def copied_answers(
        self,
        fixture_name: str,
    ) -> tuple[Path, dict[str, object]]:
        value = read_yaml(FIXTURE_ROOT / fixture_name)
        destination = self.work_root / fixture_name
        write_yaml(destination, value)
        return destination, value

    def write_capability_case(self, name: str, value: object) -> Path:
        path = self.work_root / f"{name}.json"
        path.write_text(
            json.dumps(value, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        return path
