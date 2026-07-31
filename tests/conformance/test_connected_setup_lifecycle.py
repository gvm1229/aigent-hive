"""Connected global-user to project setup lifecycle conformance."""

from __future__ import annotations

import json
import os
import sqlite3
import stat
import subprocess
import tomllib
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    FIXTURE_ROOT,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    read_yaml,
    write_yaml,
)


PROJECT_REGISTRY_SCHEMA = json.loads(
    (REPOSITORY_ROOT / "schemas/project-registry.schema.json").read_text(
        encoding="utf-8"
    )
)


class ConnectedSetupLifecycleConformance(Phase1CliTestCase):
    host = "codex"

    def setUp(self) -> None:
        super().setUp()
        self.user_root = self.work_root / "connected-user-root"
        self.user_root.mkdir()
        self.fake_bin = self.work_root / "fake-bin"
        self.fake_bin.mkdir()
        self.host_state = self.work_root / f"{self.host}-host-state.json"
        self._write_fake_host()
        self.environment = {
            "PATH": str(self.fake_bin)
            + os.pathsep
            + os.environ.get("PATH", ""),
            "HIVE_TEST_USER_ROOT": str(self.user_root),
            "HIVE_TEST_HOST_STATE": str(self.host_state),
        }

        _, install = self.invoke(
            "install",
            "--scope",
            "user",
            "--host",
            self.host,
            "--user-root",
            str(self.user_root),
            "--apply",
        )
        self.assertEqual(install["code"], "hive.user-install-complete")
        if self.host == "antigravity":
            self.assertEqual(install["data"]["qualified_host_version"], "1.1.7")
            self.assertEqual(
                install["data"]["host_version_range"],
                ">=1.1.7 <1.2.0",
            )

        global_answers = self.work_root / "global-answers.yml"
        write_yaml(
            global_answers,
            {
                "schema_version": 1,
                "interface_language": "ko",
                "wiki": {"enabled": True, "language": "both"},
                "profile": {"id": "web-developer"},
                "persona": {"id": "friendly"},
                "selected_hosts": [self.host],
                "skills": {
                    "mode": "individual",
                    "selected": [
                        "setup-hive",
                        "setup-harness",
                        "hive-prompt-refine",
                        "hive-knowledge-query",
                    ],
                },
                "usage_guard": {
                    "enabled": True,
                    "stop_remaining_percent": 17,
                    "codexbar_fallback_enabled": False,
                },
            },
        )
        _, setup = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(global_answers),
            "--user-root",
            str(self.user_root),
            "--apply",
        )
        self.assertEqual(setup["code"], "hive.user-setup-complete")
        self.assertEqual(setup["data"]["setup_state"], "operational")
        installed_config = read_yaml(
            self.user_root / ".hive/config/user-setup.yml"
        )
        self.assertEqual(installed_config["selected_hosts"], [self.host])
        guidance_path = {
            "codex": ".codex/AGENTS.md",
            "antigravity": ".gemini/GEMINI.md",
        }[self.host]
        guidance = (self.user_root / guidance_path).read_text(encoding="utf-8")
        self.assertIn("상태: `operational`", guidance)

    def _write_fake_host(self) -> None:
        program = """\
import json
import os
import shutil
import sys
from pathlib import Path

host = __HIVE_TEST_HOST__
arguments = sys.argv[1:]
if arguments == ["--version"]:
    print("codex-cli 0.145.0" if host == "codex" else "1.1.7")
    raise SystemExit(0)

state_path = Path(os.environ["HIVE_TEST_HOST_STATE"])
try:
    state = json.loads(state_path.read_text(encoding="utf-8"))
except FileNotFoundError:
    state = {"marketplace": False, "plugin": False}

user_root = Path(os.environ["HIVE_TEST_USER_ROOT"])
command = " ".join(arguments)
if host == "antigravity" and command.startswith("plugin install "):
    source = Path(arguments[2])
    stage = user_root / ".gemini/config/plugins/aigent-hive"
    if stage.exists():
        shutil.rmtree(stage)
    shutil.copytree(source, stage)
    state["plugin"] = True
elif host == "antigravity" and command == "plugin uninstall aigent-hive":
    stage = user_root / ".gemini/config/plugins/aigent-hive"
    if stage.exists():
        shutil.rmtree(stage)
    state["plugin"] = False
elif command.startswith("plugin marketplace add "):
    state["marketplace"] = True
elif command == "plugin add aigent-hive@aigent-hive --json":
    state["plugin"] = True
elif command == "plugin remove aigent-hive@aigent-hive --json":
    state["plugin"] = False
elif command == "plugin marketplace remove aigent-hive --json":
    state["marketplace"] = False
state_path.write_text(json.dumps(state), encoding="utf-8")

marketplace = user_root / ".hive/marketplaces/codex"
if host == "antigravity" and command == "plugin list":
    if state["plugin"]:
        print(json.dumps({"imports": [{
            "name": "aigent-hive",
            "source": "antigravity",
            "importedAt": "2026-07-27T00:00:00Z",
            "components": ["skills"],
        }]}))
    else:
        print("No imported plugins.")
elif command == "plugin marketplace list --json":
    entries = [{"name": "aigent-hive", "root": str(marketplace)}] if state["marketplace"] else []
    print(json.dumps({"marketplaces": entries}))
elif command == "plugin list --json":
    installed = []
    if state["plugin"]:
        installed.append({
            "pluginId": "aigent-hive@aigent-hive",
            "version": "0.8.0",
            "enabled": True,
            "source": {"path": str(marketplace / "plugins/aigent-hive")},
            "marketplaceSource": {"source": str(marketplace)},
        })
    print(json.dumps({"installed": installed, "available": []}))
else:
    print("{}")
""".replace("__HIVE_TEST_HOST__", repr(self.host))
        if os.name == "nt":
            executable_name = "agy" if self.host == "antigravity" else "codex"
            python_path = self.fake_bin / f"{executable_name}.py"
            python_path.write_text(program, encoding="utf-8")
            (self.fake_bin / f"{executable_name}.cmd").write_text(
                f'@"{os.sys.executable}" "%~dp0\\{executable_name}.py" %*\r\n',
                encoding="utf-8",
            )
            return
        executable = self.fake_bin / (
            "agy" if self.host == "antigravity" else "codex"
        )
        executable.write_text(
            f"#!{os.sys.executable}\n{program}",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)

    def invoke(self, *arguments: str) -> tuple[subprocess.CompletedProcess[str], dict]:
        process = subprocess.run(
            [str(self.hive_binary), *arguments, "--output", "json"],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
            env={**os.environ, **self.environment},
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"stdout must be one JSON object: {error}\n"
                f"stdout={process.stdout!r}\nstderr={process.stderr!r}"
            )
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        self.assertEqual(process.returncode, 0, process.stderr)
        return process, result

    def setup_project(self, name: str, answers: dict) -> Path:
        target = self.work_root / name
        target.mkdir()
        answer_path = self.work_root / f"{name}-answers.yml"
        write_yaml(answer_path, answers)
        process, result = self.invoke_setup(
            target,
            answers=answer_path,
            capabilities={
                "codex": "capabilities-codex-omx.json",
                "antigravity": "capabilities-antigravity-absent.json",
            }[self.host],
            extra_arguments=("--user-root", str(self.user_root)),
            environment=self.environment,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-complete")
        return target

    def project_answers(self, name: str) -> dict:
        answers = read_yaml(FIXTURE_ROOT / "answers-base.yml")
        answers["project_name"] = name
        answers["project_identity"] = name
        answers["primary_host"] = self.host
        return answers

    def assert_shared_user_store(self, target: Path) -> None:
        registry_path = self.user_root / ".hive/config/projects.yml"
        registry = read_yaml(registry_path)
        Draft202012Validator(PROJECT_REGISTRY_SCHEMA).validate(registry)
        self.assertEqual(registry["schema_version"], 1)
        self.assertEqual(len(registry["projects"]), 1)
        project = registry["projects"][0]
        self.assertEqual(
            set(project),
            {"id", "root", "enabled", "language", "visibility"},
        )
        self.assertEqual(Path(project["root"]), target.resolve())
        self.assertTrue(project["enabled"])
        self.assertEqual(project["visibility"], "project-private")

        index_path = self.user_root / ".hive/index/hive.sqlite3"
        self.assertEqual(
            sorted(self.user_root.rglob("*.sqlite*")),
            [index_path],
        )
        self.assertEqual(list(target.rglob("*.sqlite*")), [])
        connection = sqlite3.connect(index_path)
        try:
            self.assertEqual(
                connection.execute("PRAGMA integrity_check").fetchone(),
                ("ok",),
            )
            metadata = dict(connection.execute("SELECT key, value FROM meta"))
            self.assertEqual(
                set(metadata),
                {
                    "schema_version",
                    "logical_digest",
                    "page_count",
                    "project_count",
                },
            )
            self.assertEqual(metadata["schema_version"], "2")
            self.assertEqual(metadata["project_count"], "1")
            self.assertRegex(metadata["logical_digest"], r"^sha256:[0-9a-f]{64}$")
        finally:
            connection.close()

    def test_expedited_project_inherits_operational_global_preferences(self) -> None:
        target = self.setup_project(
            "expedited-project",
            self.project_answers("expedited-project"),
        )

        with (target / ".hive/config/harness.toml").open("rb") as stream:
            harness = tomllib.load(stream)
        self.assertEqual(harness["primary_host"], self.host)
        self.assertEqual(harness["setup_mode"], "expedited")
        self.assertEqual(harness["preference_provenance"], "global-inherited")
        self.assertEqual(harness["interface_language"], "ko")
        self.assertTrue(harness["wiki_enabled"])
        self.assertEqual(harness["wiki_language"], "both")
        self.assertEqual(harness["persona_id"], "friendly")
        self.assertTrue(harness["usage_guard_enabled"])
        self.assertEqual(harness["usage_stop_remaining_percent"], 17)
        self.assertEqual(
            harness["selected_project_skills"],
            [
                "hive-knowledge-query",
                "hive-prompt-refine",
                "hive-usage-guard",
                "setup-harness",
            ],
        )
        self.assert_shared_user_store(target)

    def test_custom_project_preserves_explicit_project_preferences(self) -> None:
        answers = self.project_answers("custom-project")
        answers.update(
            {
                "setup_mode": "custom",
                "interface_language": "en",
                "wiki": {"enabled": True, "language": "en"},
                "persona": {"id": "strict"},
                "skills": {
                    "mode": "individual",
                    "selected": ["setup-harness"],
                },
            }
        )
        target = self.setup_project("custom-project", answers)

        with (target / ".hive/config/harness.toml").open("rb") as stream:
            harness = tomllib.load(stream)
        self.assertEqual(harness["primary_host"], self.host)
        self.assertEqual(harness["setup_mode"], "custom")
        self.assertEqual(harness["preference_provenance"], "project-custom")
        self.assertEqual(harness["interface_language"], "en")
        self.assertTrue(harness["wiki_enabled"])
        self.assertEqual(harness["wiki_language"], "en")
        self.assertEqual(harness["persona_id"], "strict")
        self.assertTrue(harness["usage_guard_enabled"])
        self.assertEqual(harness["usage_stop_remaining_percent"], 17)
        self.assertEqual(harness["selected_project_skills"], ["setup-harness"])
        self.assert_shared_user_store(target)


class AntigravityConnectedSetupLifecycleConformance(
    ConnectedSetupLifecycleConformance
):
    host = "antigravity"
