"""User install, project bootstrap, and local-priority upgrade conformance."""

from __future__ import annotations

import json
import os
import stat
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.phase1_support import (
    ACTION_RESULT_SCHEMA,
    FIXTURE_ROOT,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    read_yaml,
    snapshot_tree,
    write_yaml,
)


class ProjectLifecycleConformance(Phase1CliTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.fake_bin = self.work_root / "fake-bin"
        self.fake_bin.mkdir()
        self.host_log = self.work_root / "host-commands.jsonl"
        for host in ("codex", "claude", "antigravity"):
            self._write_fake_host(host)

    def _write_fake_host(self, host: str) -> None:
        program = f"""\
import json
import os
import shutil
import sys
from pathlib import Path

with open(os.environ["HIVE_TEST_HOST_LOG"], "a", encoding="utf-8") as stream:
    stream.write(json.dumps({{"program": os.path.basename(sys.argv[0]), "argv": sys.argv[1:]}}) + "\\n")
host = {host!r}
arguments = sys.argv[1:]
if arguments == ["--version"]:
    print({{"codex": "codex-cli 0.145.0", "claude": "2.1.0 (Claude Code)", "antigravity": "1.1.7"}}[host])
    raise SystemExit(0)

state_path = os.environ["HIVE_TEST_HOST_LOG"] + "." + host + ".state"
try:
    with open(state_path, encoding="utf-8") as stream:
        state = json.load(stream)
except FileNotFoundError:
    state = {{"marketplace": False, "plugin": False}}

command = " ".join(arguments)
if host == "antigravity" and command.startswith("plugin install "):
    source = Path(arguments[2])
    hive_root = next(parent for parent in source.parents if parent.name == ".hive")
    stage = hive_root.parent / ".gemini/config/plugins/aigent-hive"
    if stage.exists():
        shutil.rmtree(stage)
    shutil.copytree(source, stage)
    for candidate in sorted(stage.rglob("*"), reverse=True):
        if candidate.is_dir() and not any(candidate.iterdir()):
            candidate.rmdir()
    state["plugin"] = True
    state["stage"] = str(stage)
elif host == "antigravity" and command == "plugin uninstall aigent-hive":
    stage = Path(state.get("stage", ""))
    if state.get("stage") and stage.exists():
        shutil.rmtree(stage)
    state["plugin"] = False
elif command.startswith("plugin marketplace add "):
    state["marketplace"] = True
elif command in ("plugin add aigent-hive@aigent-hive --json", "plugin install aigent-hive@aigent-hive --scope user"):
    state["plugin"] = True
elif "plugin marketplace remove" in command:
    state["marketplace"] = False
elif command.startswith("plugin remove ") or command.startswith("plugin uninstall "):
    state["plugin"] = False

with open(state_path, "w", encoding="utf-8") as stream:
    json.dump(state, stream)

root = os.path.dirname(os.environ["HIVE_TEST_HOST_LOG"])
user_root = os.environ.get("HIVE_TEST_USER_ROOT")
if command == "plugin marketplace list --json":
    if host == "codex":
        marketplace = os.path.join(user_root, ".hive/marketplaces/codex") if user_root else os.path.join(root, "user-codex/.hive/marketplaces/codex")
        entries = [{{"name": "aigent-hive", "root": marketplace}}] if state["marketplace"] else []
        print(json.dumps({{"marketplaces": entries}}))
    else:
        marketplace = os.path.join(user_root, ".hive/marketplaces/claude") if user_root else os.path.join(root, "user-claude/.hive/marketplaces/claude")
        entries = [{{"name": "aigent-hive", "source": "directory", "path": marketplace}}] if state["marketplace"] else []
        print(json.dumps(entries))
elif host == "antigravity" and command == "plugin list":
    if state["plugin"]:
        print(json.dumps({{"imports": [{{
            "name": "aigent-hive",
            "source": "antigravity",
            "importedAt": "2026-07-27T00:00:00Z",
            "components": ["skills"],
        }}]}}))
    else:
        print("No imported plugins.")
elif command == "plugin list --json":
    if host == "codex":
        plugin_root = os.path.join(user_root, ".hive/marketplaces/codex") if user_root else os.path.join(root, "user-codex/.hive/marketplaces/codex")
        entries = [{{
            "pluginId": "aigent-hive@aigent-hive",
            "version": "0.9.0",
            "enabled": True,
            "source": {{"path": os.path.join(plugin_root, "plugins/aigent-hive")}},
            "marketplaceSource": {{"source": plugin_root}},
        }}] if state["plugin"] else []
        print(json.dumps({{"installed": entries, "available": []}}))
    else:
        entries = [{{"id": "aigent-hive@aigent-hive", "version": "0.9.0", "enabled": True, "scope": "user"}}] if state["plugin"] else []
        print(json.dumps(entries))
else:
    print("{{}}")
"""
        if os.name == "nt":
            executable_name = "agy" if host == "antigravity" else host
            python_path = self.fake_bin / f"{executable_name}.py"
            python_path.write_text(program, encoding="utf-8")
            command_path = self.fake_bin / f"{executable_name}.cmd"
            command_path.write_text(
                f'@"{os.sys.executable}" "%~dp0\\{executable_name}.py" %*\r\n',
                encoding="utf-8",
            )
            return
        executable_name = "agy" if host == "antigravity" else host
        command_path = self.fake_bin / executable_name
        command_path.write_text(
            f"#!{os.sys.executable}\n{program}",
            encoding="utf-8",
        )
        command_path.chmod(command_path.stat().st_mode | stat.S_IXUSR)

    def invoke(
        self,
        *arguments: str,
        environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [str(self.hive_binary), *arguments, "--output", "json"],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
            env={
                **os.environ,
                "PATH": str(self.fake_bin)
                + os.pathsep
                + os.environ.get("PATH", ""),
                "HIVE_TEST_HOST_LOG": str(self.host_log),
                **(environment or {}),
            },
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
        return process, result

    def setup_project(
        self,
        name: str,
        *,
        host: str = "codex",
        foreign_guidance: bool = True,
    ) -> Path:
        target = self.work_root / name
        target.mkdir()
        if foreign_guidance:
            (target / "AGENTS.md").write_text(
                "# Foreign project rules\n\n"
                "<!-- omx:generated:agents-md -->\n"
                "OMX bytes must survive exactly.\n",
                encoding="utf-8",
            )
            (target / "CLAUDE.md").write_text(
                "# Existing Claude adapter\n",
                encoding="utf-8",
            )
            (target / "GEMINI.md").write_text(
                "# Existing Gemini adapter\n",
                encoding="utf-8",
            )
        if host == "claude":
            answers = FIXTURE_ROOT / "answers-claude.yml"
            capabilities = "capabilities-claude-omc.json"
        elif host == "antigravity":
            value = read_yaml(FIXTURE_ROOT / "answers-base.yml")
            value["primary_host"] = "antigravity"
            answers = self.work_root / f"{name}-answers.yml"
            write_yaml(answers, value)
            capabilities = "capabilities-antigravity-absent.json"
        else:
            answers = FIXTURE_ROOT / "answers-base.yml"
            capabilities = "capabilities-codex-omx.json"
        process, result = self.invoke_setup(
            target,
            answers=answers,
            capabilities=capabilities,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-complete")
        return target

    def write_user_setup_answers(
        self,
        name: str,
        *,
        hosts: list[str],
        wiki_enabled: bool = True,
        skills_mode: str = "all",
        selected_skills: list[str] | None = None,
        usage_enabled: bool = False,
        threshold: int = 20,
    ) -> Path:
        path = self.work_root / f"{name}.yml"
        skills: dict[str, object]
        if skills_mode == "all":
            skills = {
                "mode": "all",
            }
        else:
            skills = {
                "mode": "individual",
                "selected": selected_skills or ["configure"],
            }
        write_yaml(
            path,
            {
                "schema_version": 1,
                "interface_language": "ko",
                "wiki": {"enabled": wiki_enabled, "language": "both"},
                "profile": {"id": "web-developer"},
                "persona": {"id": "balanced"},
                "selected_hosts": hosts,
                "skills": skills,
                "usage_guard": {
                    "enabled": usage_enabled,
                    "stop_remaining_percent": threshold,
                    "codexbar_fallback_enabled": False,
                },
            },
        )
        return path

    def test_user_install_preserves_foreign_guidance_for_all_hosts(self) -> None:
        cases = {
            "codex": (".codex/AGENTS.override.md", "OMX override bytes\n"),
            "claude": (".claude/CLAUDE.md", "OMC global bytes\n"),
            "antigravity": (".gemini/GEMINI.md", "Gemini global bytes\n"),
        }
        for host, (relative, foreign) in cases.items():
            with self.subTest(host=host):
                user_root = self.work_root / f"user-{host}"
                guidance = user_root / relative
                guidance.parent.mkdir(parents=True)
                guidance.write_text(foreign, encoding="utf-8")
                if host == "codex":
                    base = user_root / ".codex/AGENTS.md"
                    base.write_text("Base OMX bytes\n", encoding="utf-8")
                before = snapshot_tree(user_root)

                dry, dry_result = self.invoke(
                    "install",
                    "--scope",
                    "user",
                    "--host",
                    host,
                    "--user-root",
                    str(user_root),
                    "--dry-run",
                )
                self.assertEqual(dry.returncode, 0, dry.stderr)
                self.assertEqual(
                    dry_result["code"], "hive.user-install-dry-run-complete"
                )
                self.assertEqual(snapshot_tree(user_root), before)

                applied, applied_result = self.invoke(
                    "install",
                    "--scope",
                    "user",
                    "--host",
                    host,
                    "--user-root",
                    str(user_root),
                    "--apply",
                )
                self.assertEqual(applied.returncode, 0, applied.stderr)
                self.assertEqual(
                    applied_result["code"], "hive.user-install-complete"
                )
                if host == "antigravity":
                    self.assertEqual(
                        applied_result["data"]["qualified_host_version"],
                        "1.1.7",
                    )
                    self.assertEqual(
                        applied_result["data"]["host_version_range"],
                        ">=1.1.7 <1.2.0",
                    )
                    host_calls = [
                        json.loads(line)
                        for line in self.host_log.read_text(
                            encoding="utf-8"
                        ).splitlines()
                    ]
                    self.assertTrue(
                        any(
                            Path(call["program"]).stem == "agy"
                            for call in host_calls
                        )
                    )
                installed = guidance.read_text(encoding="utf-8")
                self.assertTrue(installed.startswith(foreign))
                self.assertEqual(
                    installed.count("<!-- AIGENT-HIVE:USER:START -->"), 1
                )
                self.assertEqual(
                    installed.count("<!-- AIGENT-HIVE:USER:END -->"), 1
                )
                manifest = json.loads(
                    (
                        user_root / f".hive/install/{host}.json"
                    ).read_text(encoding="utf-8")
                )
                self.assertEqual(manifest["host"], host)
                self.assertTrue(manifest["source_release_digest"].startswith("sha256:"))
                self.assertFalse((user_root / ".hive/knowledge").exists())
                self.assertIn("State / 상태: `setup-required`", installed)
                if host == "antigravity":
                    self.assertTrue(
                        (
                            user_root
                            / ".gemini/config/skills/configure/SKILL.md"
                        ).is_file()
                    )
                    self.assertEqual(
                        json.loads(
                            (
                                user_root
                                / ".gemini/config/plugins/aigent-hive/plugin.json"
                            ).read_text(encoding="utf-8")
                        ),
                        {"name": "aigent-hive"},
                    )
                    self.assertTrue(
                        (
                            user_root
                            / ".gemini/config/plugins/aigent-hive"
                            / "skills/configure/SKILL.md"
                        ).is_file()
                    )
                else:
                    self.assertTrue(
                        (
                            user_root
                            / f".hive/marketplaces/{host}/plugins/aigent-hive"
                            / "skills/configure/SKILL.md"
                        ).is_file()
                    )
                    self.assertFalse(
                        (
                            user_root
                            / f".hive/marketplaces/{host}/plugins/aigent-hive"
                            / "skills/refine-prompt/SKILL.md"
                        ).exists()
                    )
                    if host == "codex":
                        marketplace = (
                            user_root
                            / ".hive/marketplaces/codex"
                            / ".agents/plugins/marketplace.json"
                        )
                        self.assertEqual(
                            json.loads(marketplace.read_text(encoding="utf-8"))[
                                "plugins"
                            ][0]["source"]["path"],
                            "./plugins/aigent-hive",
                        )
                        self.assertFalse(
                            (
                                user_root
                                / ".hive/marketplaces/codex/marketplace.json"
                            ).exists()
                        )

                valid, valid_result = self.invoke(
                    "install",
                    "--scope",
                    "user",
                    "--host",
                    host,
                    "--user-root",
                    str(user_root),
                    "--validate",
                )
                self.assertEqual(valid.returncode, 0, valid.stderr)
                self.assertEqual(valid_result["code"], "hive.user-install-valid")
                if host == "codex":
                    self.assertEqual(
                        (user_root / ".codex/AGENTS.md").read_text(encoding="utf-8"),
                        "Base OMX bytes\n",
                    )

    def test_global_setup_projects_selected_skills_and_wiki_preferences(self) -> None:
        user_root = self.work_root / "user-codex"
        user_root.mkdir()
        environment = {"HIVE_TEST_USER_ROOT": str(user_root)}
        installed, install_result = self.invoke(
            "install",
            "--scope",
            "user",
            "--host",
            "codex",
            "--user-root",
            str(user_root),
            "--apply",
            environment=environment,
        )
        self.assertEqual(installed.returncode, 0, installed.stderr)
        self.assertEqual(install_result["code"], "hive.user-install-complete")

        all_built_ins = self.write_user_setup_answers(
            "all-built-ins-user-setup",
            hosts=["codex"],
        )
        before_preview = snapshot_tree(user_root)
        preview, preview_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(all_built_ins),
            "--user-root",
            str(user_root),
            "--dry-run",
            environment=environment,
        )
        self.assertEqual(preview.returncode, 0, preview.stderr)
        self.assertEqual(
            preview_result["code"], "hive.user-setup-dry-run-complete"
        )
        self.assertEqual(preview_result["data"]["setup_state"], "setup-required")
        self.assertEqual(snapshot_tree(user_root), before_preview)
        self.assertIn(
            "search-knowledge", preview_result["data"]["resolved_skills"]
        )
        self.assertIn(
            ".hive/config/user-setup.yml", preview_result["changed_paths"]
        )
        self.assertIn(
            ".agents/directives/00-hive-user.md",
            preview_result["changed_paths"],
        )
        self.assertIn(
            ".agents/skills/refine-prompt/SKILL.md",
            preview_result["changed_paths"],
        )
        self.assertIn(
            ".hive/marketplaces/codex/plugins/aigent-hive/skills/"
            "refine-prompt/SKILL.md",
            preview_result["changed_paths"],
        )

        applied, applied_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(all_built_ins),
            "--user-root",
            str(user_root),
            "--apply",
            environment=environment,
        )
        self.assertEqual(applied.returncode, 0, applied.stderr)
        self.assertEqual(applied_result["code"], "hive.user-setup-complete")
        self.assertEqual(applied_result["data"]["setup_state"], "operational")
        self.assertTrue((user_root / ".hive/knowledge/Wiki/index.md").is_file())
        self.assertTrue((user_root / ".hive/index/hive.sqlite3").is_file())
        self.assertTrue(
            (user_root / ".agents/skills/refine-prompt/SKILL.md").is_file()
        )
        self.assertTrue(
            (
                user_root
                / ".hive/marketplaces/codex/plugins/aigent-hive"
                / "skills/refine-prompt/SKILL.md"
            ).is_file()
        )
        self.assertTrue(
            (user_root / ".agents/skills/manage-usage/SKILL.md").is_file()
        )
        guidance = (user_root / ".codex/AGENTS.md").read_text(encoding="utf-8")
        self.assertIn("상태: `operational`", guidance)
        self.assertIn(
            "명시적 요청이 없는 한 모든 질문과 응답에 한국어 사용", guidance
        )
        self.assertIn(
            "다른 언어로 작성된 메시지만으로 이 선호를 변경하지 않음", guidance
        )

        reduced = self.write_user_setup_answers(
            "reduced-user-setup",
            hosts=["codex"],
            wiki_enabled=False,
            skills_mode="individual",
            selected_skills=["configure", "refine-prompt"],
            usage_enabled=True,
            threshold=17,
        )
        reapplied, reapplied_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(reduced),
            "--user-root",
            str(user_root),
            "--apply",
            environment=environment,
        )
        self.assertEqual(reapplied.returncode, 0, reapplied.stderr)
        self.assertEqual(reapplied_result["code"], "hive.user-setup-complete")
        self.assertFalse((user_root / ".hive/index/hive.sqlite3").exists())
        self.assertTrue((user_root / ".hive/knowledge/Wiki/index.md").is_file())
        self.assertTrue(
            (user_root / ".agents/skills/manage-usage/SKILL.md").is_file()
        )
        self.assertFalse(
            (user_root / ".agents/skills/search-knowledge/SKILL.md").exists()
        )
        self.assertFalse(
            (
                user_root
                / ".hive/marketplaces/codex/plugins/aigent-hive"
                / "skills/search-knowledge/SKILL.md"
            ).exists()
        )
        installed_config = read_yaml(user_root / ".hive/config/user-setup.yml")
        self.assertEqual(
            installed_config["usage_guard"]["stop_remaining_percent"], 17
        )

        valid, valid_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(reduced),
            "--user-root",
            str(user_root),
            "--validate",
            environment=environment,
        )
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.assertEqual(valid_result["code"], "hive.user-setup-valid")

        generic_directive = user_root / ".agents/directives/00-hive-user.md"
        generic_bytes = generic_directive.read_bytes()
        generic_directive.write_bytes(generic_bytes + b"\nlocal tamper\n")
        invalid_generic, invalid_generic_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(reduced),
            "--user-root",
            str(user_root),
            "--validate",
            environment=environment,
        )
        self.assertNotEqual(invalid_generic.returncode, 0)
        self.assertEqual(
            invalid_generic_result["code"], "hive.user-setup-conflict"
        )
        generic_directive.write_bytes(generic_bytes)

        host_skill = (
            user_root
            / ".hive/marketplaces/codex/plugins/aigent-hive"
            / "skills/refine-prompt/SKILL.md"
        )
        host_bytes = host_skill.read_bytes()
        host_skill.write_bytes(host_bytes + b"\nlocal tamper\n")
        invalid_host, invalid_host_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(reduced),
            "--user-root",
            str(user_root),
            "--validate",
            environment=environment,
        )
        self.assertNotEqual(invalid_host.returncode, 0)
        self.assertEqual(
            invalid_host_result["code"],
            "hive.user-setup-verification-failed",
        )
        host_skill.write_bytes(host_bytes)

        final_valid, final_valid_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(reduced),
            "--user-root",
            str(user_root),
            "--validate",
            environment=environment,
        )
        self.assertEqual(final_valid.returncode, 0, final_valid.stderr)
        self.assertEqual(final_valid_result["code"], "hive.user-setup-valid")

    def test_global_setup_preserves_all_host_guidance_markers(self) -> None:
        user_root = self.work_root / "user-multi-host"
        foreign = {
            ".codex/AGENTS.md": "Existing Codex root bytes\n",
            ".codex/AGENTS.override.md": "Existing Codex override bytes\n",
            ".claude/CLAUDE.md": "Existing Claude bytes\n",
            ".gemini/GEMINI.md": "Existing Antigravity bytes\n",
        }
        for relative, content in foreign.items():
            path = user_root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        environment = {"HIVE_TEST_USER_ROOT": str(user_root)}
        for host in ("codex", "claude", "antigravity"):
            installed, result = self.invoke(
                "install",
                "--scope",
                "user",
                "--host",
                host,
                "--user-root",
                str(user_root),
                "--apply",
                environment=environment,
            )
            self.assertEqual(installed.returncode, 0, installed.stderr)
            self.assertEqual(result["code"], "hive.user-install-complete")

        answers = self.write_user_setup_answers(
            "multi-host-user-setup",
            hosts=["codex", "claude", "antigravity"],
            skills_mode="individual",
            selected_skills=["record-knowledge"],
        )
        preview_before = snapshot_tree(user_root)
        preview, preview_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(answers),
            "--user-root",
            str(user_root),
            "--dry-run",
            environment=environment,
        )
        self.assertEqual(preview.returncode, 0, preview.stderr)
        self.assertEqual(snapshot_tree(user_root), preview_before)
        self.assertEqual(
            preview_result["data"]["resolved_skills"],
            ["configure", "record-knowledge", "search-knowledge"],
        )
        host_skill_paths = {
            "codex": ".hive/marketplaces/codex/plugins/aigent-hive/skills/"
            "search-knowledge/SKILL.md",
            "claude": ".hive/marketplaces/claude/plugins/aigent-hive/skills/"
            "search-knowledge/SKILL.md",
            "antigravity": ".gemini/config/skills/"
            "search-knowledge/SKILL.md",
        }
        for path in host_skill_paths.values():
            self.assertIn(
                path,
                preview_result["changed_paths"],
            )

        applied, applied_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(answers),
            "--user-root",
            str(user_root),
            "--apply",
            environment=environment,
        )
        self.assertEqual(applied.returncode, 0, applied.stderr)
        self.assertEqual(applied_result["code"], "hive.user-setup-complete")
        for relative, content in foreign.items():
            installed = (user_root / relative).read_text(encoding="utf-8")
            if relative == ".codex/AGENTS.md":
                self.assertEqual(installed, content)
            else:
                self.assertTrue(installed.startswith(content))
                self.assertEqual(
                    installed.count("<!-- AIGENT-HIVE:USER:START -->"), 1
                )
                self.assertEqual(
                    installed.count("<!-- AIGENT-HIVE:USER:END -->"), 1
                )

        valid, valid_result = self.invoke(
            "setup",
            "--scope",
            "user",
            "--answers",
            str(answers),
            "--user-root",
            str(user_root),
            "--validate",
            environment=environment,
        )
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.assertEqual(valid_result["code"], "hive.user-setup-valid")

    def test_project_setup_projects_portable_harness_for_every_host(self) -> None:
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                target = self.setup_project(f"consumer-{host}", host=host)
                agents = (target / "AGENTS.md").read_text(encoding="utf-8")
                self.assertTrue(agents.startswith("# Foreign project rules\n"))
                self.assertIn("OMX bytes must survive exactly.", agents)
                self.assertEqual(
                    agents.count("<!-- AIGENT-HIVE:START -->"), 1
                )
                for directive in (
                    "00-project-harness.md",
                    "01-project-knowledge.md",
                    "02-project-upgrade.md",
                ):
                    self.assertTrue(
                        (target / ".agents/directives" / directive).is_file()
                    )
                for skill in (
                    "refine-prompt",
                    "upgrade-project",
                    "share-knowledge",
                ):
                    self.assertTrue(
                        (target / ".agents/skills" / skill / "SKILL.md").is_file()
                    )
                self.assertTrue(
                    (target / ".hive/config/project-base.json").is_file()
                )
                self.assertFalse(
                    (target / ".hive/index/hive.sqlite3").exists()
                )
                self.assertIn(
                    "@AGENTS.md",
                    (target / "CLAUDE.md").read_text(encoding="utf-8"),
                )
                self.assertIn(
                    "@AGENTS.md",
                    (target / "GEMINI.md").read_text(encoding="utf-8"),
                )
                if host == "claude":
                    self.assertTrue(
                        (
                            target
                            / ".claude/skills/upgrade-project/SKILL.md"
                        ).is_file()
                    )
                else:
                    self.assertFalse((target / ".claude/skills").exists())

    def test_upgrade_preserves_local_skill_and_recovers_injected_failure(self) -> None:
        target = self.setup_project("upgrade-consumer")
        skill = target / ".agents/skills/answer/SKILL.md"
        user_suffix = b"\n<!-- user-local-preference -->\n"
        skill.write_bytes(skill.read_bytes() + user_suffix)
        generated_config = target / ".hive/config/harness.toml"
        generated_config.write_text(
            generated_config.read_text(encoding="utf-8")
            + 'unowned_override = "must be replaced"\n',
            encoding="utf-8",
        )

        scanned, scan_result = self.invoke(
            "project",
            "upgrade",
            "--target",
            str(target),
            "--scan",
        )
        self.assertEqual(scanned.returncode, 0, scanned.stderr)
        self.assertEqual(scan_result["code"], "hive.project-upgrade-available")
        reports = {
            report["path"]: report for report in scan_result["data"]["reports"]
        }
        self.assertEqual(
            reports[".agents/skills/answer/SKILL.md"]["disposition"],
            "local-preserved",
        )
        self.assertTrue(
            reports[".agents/skills/answer/SKILL.md"]["local_priority"]
        )

        active_before = {
            path: value
            for path, value in snapshot_tree(target).items()
            if not path.startswith((".hive/backups", ".hive/runtime"))
        }
        failed, failed_result = self.invoke(
            "project",
            "upgrade",
            "--target",
            str(target),
            "--apply",
            environment={"HIVE_PROJECT_UPGRADE_FAIL_AFTER": "1"},
        )
        self.assertEqual(failed.returncode, 10, failed.stderr)
        self.assertEqual(failed_result["code"], "hive.internal-error")
        active_after = {
            path: value
            for path, value in snapshot_tree(target).items()
            if not path.startswith((".hive/backups", ".hive/runtime"))
        }
        self.assertEqual(active_after, active_before)
        self.assertFalse(
            (target / ".hive/runtime/project-upgrade-journal.json").exists()
        )

        applied, applied_result = self.invoke(
            "project",
            "upgrade",
            "--target",
            str(target),
            "--apply",
        )
        self.assertEqual(applied.returncode, 0, applied.stderr)
        self.assertEqual(
            applied_result["code"], "hive.project-upgrade-complete"
        )
        self.assertTrue(skill.read_bytes().endswith(user_suffix))
        self.assertNotIn(
            "unowned_override",
            generated_config.read_text(encoding="utf-8"),
        )
        self.assertNotIn(b"<<<<<<<", skill.read_bytes())
        overrides = json.loads(
            (
                target / ".hive/config/project-overrides.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(
            [entry["path"] for entry in overrides["files"]],
            [".agents/skills/answer/SKILL.md"],
        )

        valid, valid_result = self.invoke(
            "project",
            "upgrade",
            "--target",
            str(target),
            "--validate",
        )
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.assertEqual(valid_result["code"], "hive.project-upgrade-current")

    def test_missing_or_tampered_base_and_source_root_fail_without_mutation(self) -> None:
        for case in ("missing", "tampered"):
            with self.subTest(case=case):
                target = self.setup_project(f"base-{case}")
                skill = target / ".agents/skills/answer/SKILL.md"
                skill.write_text(
                    skill.read_text(encoding="utf-8") + "\nUser change.\n",
                    encoding="utf-8",
                )
                base = target / ".hive/config/project-base.json"
                if case == "missing":
                    base.unlink()
                else:
                    payload = json.loads(base.read_text(encoding="utf-8"))
                    payload["files"][0]["content"] += "tampered"
                    base.write_text(
                        json.dumps(payload, separators=(",", ":"), sort_keys=True)
                        + "\n",
                        encoding="utf-8",
                    )
                before = snapshot_tree(target)
                process, result = self.invoke(
                    "project",
                    "upgrade",
                    "--target",
                    str(target),
                    "--scan",
                )
                self.assertNotEqual(process.returncode, 0, process.stderr)
                self.assertIn(result["status"], {"conflict", "verification-failed"})
                self.assertEqual(snapshot_tree(target), before)

        protected_source = [
            REPOSITORY_ROOT / "hive-source.json",
            REPOSITORY_ROOT / "AGENTS.md",
            REPOSITORY_ROOT / ".agents/directives/01-behavior.md",
        ]
        before_source = {
            path: path.read_bytes() for path in protected_source
        }
        process, result = self.invoke(
            "project",
            "upgrade",
            "--target",
            str(REPOSITORY_ROOT),
            "--scan",
        )
        self.assertNotEqual(process.returncode, 0, process.stderr)
        self.assertIn(result["status"], {"blocked", "error"})
        self.assertEqual(
            {path: path.read_bytes() for path in protected_source},
            before_source,
        )
