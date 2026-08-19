"""Project-to-user-root knowledge promotion conformance."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shlex
import stat
import subprocess
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Callable

from jsonschema import Draft202012Validator, FormatChecker
import yaml

from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA,
    FIXTURE_ROOT,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    read_yaml,
    snapshot_tree,
    write_yaml,
)

QUERY_RESULT_SCHEMA = json.loads(
    (
        REPOSITORY_ROOT / "schemas/knowledge-query-result.schema.json"
    ).read_text(encoding="utf-8")
)


class RootKnowledgePromotionConformance(Phase1CliTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.fake_bin = self.work_root / "fake-bin"
        self.fake_bin.mkdir()
        self._write_fake_antigravity()
        self.user_root = self.setup_user_root
        installed, installed_result = self.invoke(
            "install",
            "--scope",
            "user",
            "--host",
            "antigravity",
            "--user-root",
            str(self.user_root),
            "--apply",
        )
        self.assertEqual(installed.returncode, 0, installed.stderr)
        self.assertEqual(installed_result["code"], "hive.user-install-complete")
        self.project = self.setup_project("project-a", "stable-project-a")
        self.ingest_alpha(self.project)

    def _write_fake_antigravity(self) -> None:
        program = """\
import json
import os
import shutil
import sys
from pathlib import Path

arguments = sys.argv[1:]
stage = Path(os.environ["HIVE_TEST_USER_ROOT"]) / ".gemini/config/plugins/aigent-hive"
if arguments == ["--version"]:
    print("1.1.7")
elif arguments == ["plugin", "list"]:
    if stage.exists():
        print(json.dumps({"imports": [{
            "name": "aigent-hive",
            "source": "antigravity",
            "importedAt": "2026-07-27T00:00:00Z",
            "components": ["skills"],
        }]}))
    else:
        print("No imported plugins.")
elif len(arguments) == 3 and arguments[:2] == ["plugin", "install"]:
    if stage.exists():
        shutil.rmtree(stage)
    shutil.copytree(arguments[2], stage)
    print("{}")
elif arguments == ["plugin", "uninstall", "aigent-hive"]:
    if stage.exists():
        shutil.rmtree(stage)
    print("{}")
else:
    print("{}")
"""
        if os.name == "nt":
            python_path = self.fake_bin / "agy.py"
            python_path.write_text(program, encoding="utf-8")
            (self.fake_bin / "agy.cmd").write_text(
                f'@"{os.sys.executable}" "%~dp0\\agy.py" %*\r\n',
                encoding="utf-8",
            )
            return
        executable = self.fake_bin / "agy"
        executable.write_text(
            f"#!{os.sys.executable}\n{program}",
            encoding="utf-8",
        )
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)

    def invoke(
        self,
        *arguments: str,
        environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        command_arguments = list(arguments)
        if command_arguments[:1] == ["knowledge"] and "--user-root" not in command_arguments:
            command_arguments.extend(["--user-root", str(self.user_root)])
        process = subprocess.run(
            [str(self.hive_binary), *command_arguments, "--output", "json"],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
            env=(
                {
                    **os.environ,
                    "PATH": str(self.fake_bin)
                    + os.pathsep
                    + os.environ.get("PATH", ""),
                    "HIVE_TEST_USER_ROOT": str(self.user_root),
                    **(environment or {}),
                }
            ),
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
        if result.get("code") == "hive.knowledge-query-complete":
            Draft202012Validator(
                QUERY_RESULT_SCHEMA,
                format_checker=FormatChecker(),
            ).validate(result["data"])
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def setup_project(self, name: str, identity: str) -> Path:
        target = self.work_root / name
        target.mkdir()
        answers = read_yaml(FIXTURE_ROOT / "answers-base.yml")
        answers["project_name"] = name
        answers["project_identity"] = identity
        answers["root_knowledge_promotion_categories"] = [
            "fact",
            "preference",
            "workflow",
        ]
        answers["confidential_knowledge_categories"] = []
        answers["user_store_binding"] = "sha256:" + hashlib.sha256(
            str(self.user_root.resolve()).encode()
        ).hexdigest()
        answer_path = self.work_root / f"{name}-answers.yml"
        write_yaml(answer_path, answers)
        process, result = self.invoke_setup(target, answers=answer_path)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.setup-complete")
        return target

    def ingest_alpha(self, project: Path) -> None:
        process, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(project),
            "--source",
            str(REPOSITORY_ROOT / "tests/fixtures/knowledge/raw/alpha.md"),
            "--wiki",
            str(REPOSITORY_ROOT / "tests/fixtures/knowledge/wiki/alpha.md"),
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["code"], "hive.knowledge-ingested")

    def promote(
        self,
        project: Path,
        *,
        page_id: str = "alpha",
        category: str = "fact",
        mode: str = "--apply",
        environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        return self.invoke(
            "knowledge",
            "promote",
            "--target",
            str(project),
            "--user-root",
            str(self.user_root),
            "--page-id",
            page_id,
            "--category",
            category,
            mode,
            environment=environment,
        )

    def raced_promote(
        self,
        phase: str,
        mutate: Callable[[], None],
        *,
        project: Path | None = None,
        fail_after_writes: bool = False,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        race_directory = self.work_root / f"race-{phase}"
        race_directory.mkdir()
        environment = {
            **os.environ,
            "PATH": str(self.fake_bin)
            + os.pathsep
            + os.environ.get("PATH", ""),
            "HIVE_WIKI_TEST_RACE_PHASE": phase,
            "HIVE_WIKI_TEST_RACE_DIR": str(race_directory),
        }
        if fail_after_writes:
            environment["HIVE_WIKI_TEST_FAIL_AFTER_CANONICAL_WRITES"] = "1"
        command = [
            str(self.hive_binary),
            "knowledge",
            "promote",
            "--target",
            str(project or self.project),
            "--user-root",
            str(self.user_root),
            "--page-id",
            "alpha",
            "--category",
            "fact",
            "--apply",
            "--output",
            "json",
        ]
        process = subprocess.Popen(
            command,
            cwd=REPOSITORY_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=environment,
        )
        deadline = time.monotonic() + 5
        while not (race_directory / "ready").exists():
            if process.poll() is not None:
                stdout, stderr = process.communicate()
                self.fail(
                    f"promotion exited before race hook\nstdout={stdout!r}\n"
                    f"stderr={stderr!r}"
                )
            if time.monotonic() >= deadline:
                process.kill()
                self.fail(f"promotion did not reach race hook {phase}")
            time.sleep(0.01)
        mutate()
        (race_directory / "continue").write_text("continue\n", encoding="utf-8")
        stdout, stderr = process.communicate(timeout=10)
        completed = subprocess.CompletedProcess(
            command,
            process.returncode,
            stdout,
            stderr,
        )
        result = json.loads(stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(completed.returncode, result["exit_code"], stderr)
        return completed, result

    def invoke_skill_command(
        self,
        skill: Path,
        selector: str,
        replacements: dict[str, str],
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        text = skill.read_text(encoding="utf-8")
        commands = re.findall(r"`(hive [^`\n]+)`", text)
        commands.extend(
            re.findall(r"```text\s*\n\s*(hive [^\n]+)\n\s*```", text)
        )
        command = next(candidate for candidate in commands if selector in candidate)
        arguments = [
            replacements.get(argument, argument)
            for argument in shlex.split(command)[1:]
        ]
        process = subprocess.run(
            [str(self.hive_binary), *arguments],
            cwd=REPOSITORY_ROOT,
            check=False,
            text=True,
            capture_output=True,
            env=os.environ,
        )
        result = json.loads(process.stdout)
        Draft202012Validator(
            ACTION_RESULT_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def test_dry_run_apply_combined_query_and_rebuild_equivalence(self) -> None:
        before = snapshot_tree(self.user_root)
        dry, dry_result = self.promote(self.project, mode="--dry-run")
        self.assertEqual(dry.returncode, 0, dry.stderr)
        self.assertEqual(
            dry_result["code"], "hive.knowledge-promotion-planned"
        )
        self.assertFalse(dry_result["data"]["applied"])
        self.assertEqual(snapshot_tree(self.user_root), before)

        applied, applied_result = self.promote(self.project)
        self.assertEqual(applied.returncode, 0, applied.stderr)
        self.assertEqual(applied_result["code"], "hive.knowledge-promoted")
        self.assertTrue(applied_result["data"]["applied"])
        root_page = (
            self.user_root
            / ".hive/knowledge/Wiki"
            / f"{applied_result['data']['page_id']}.md"
        )
        page_text = root_page.read_text(encoding="utf-8")
        self.assertNotIn("stable-project-a", page_text)
        self.assertNotIn(str(self.project), page_text)

        queried, query_result = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.project),
            "--user-root",
            str(self.user_root),
            "--text",
            "deterministic local knowledge",
        )
        self.assertEqual(queried.returncode, 0, queried.stderr)
        self.assertEqual(
            query_result["data"]["precedence"],
            "own-project,user-root,shared",
        )
        self.assertEqual(
            query_result["data"]["hits"][0]["visibility"],
            "project-private",
        )
        self.assertTrue(
            any(
                hit["source_project"] == "user-root"
                and hit["visibility"] == "shared"
                for hit in query_result["data"]["hits"]
            )
        )

        root_query_before = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.user_root),
            "--user-root",
            str(self.user_root),
            "--text",
            "deterministic local knowledge",
        )[1]["data"]
        database = self.user_root / ".hive/index/hive.sqlite3"
        database.write_bytes(b"corrupt disposable index")
        rebuilt, rebuild_result = self.invoke(
            "index",
            "rebuild",
            "--user-root",
            str(self.user_root),
        )
        self.assertEqual(rebuilt.returncode, 0, rebuilt.stderr)
        self.assertEqual(rebuild_result["code"], "hive.index-rebuilt")
        root_query_after = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.user_root),
            "--user-root",
            str(self.user_root),
            "--text",
            "deterministic local knowledge",
        )[1]["data"]
        self.assertEqual(root_query_after, root_query_before)

    def test_post_write_failure_restores_exact_root_snapshot(self) -> None:
        before = snapshot_tree(self.user_root)
        failed, result = self.promote(
            self.project,
            environment={"HIVE_WIKI_TEST_FAIL_AFTER_CANONICAL_WRITES": "1"},
        )
        self.assertEqual(failed.returncode, 10, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-io-error")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.user_root), before)

    def test_external_edit_between_plan_and_claim_is_preserved(self) -> None:
        planned = self.promote(self.project, mode="--dry-run")[1]
        wiki_path = (
            self.user_root
            / ".hive/knowledge/Wiki"
            / f"{planned['data']['page_id']}.md"
        )
        external = b"external edit after promotion planning\n"

        failed, result = self.raced_promote(
            "after-plan-before-claim",
            lambda: wiki_path.write_bytes(external),
        )

        self.assertEqual(failed.returncode, 3, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-conflict")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(wiki_path.read_bytes(), external)
        self.assertFalse(
            any(
                (
                    self.user_root
                    / ".hive/knowledge/Raw/promoted-knowledge"
                ).glob("*.json")
            )
        )

    def test_destination_reoccupation_preserves_live_and_claimed_prior(self) -> None:
        first = self.promote(self.project)[1]
        wiki_path = (
            self.user_root
            / ".hive/knowledge/Wiki"
            / f"{first['data']['page_id']}.md"
        )
        prior = wiki_path.read_bytes()
        second_project = self.setup_project("project-b", "stable-project-b")
        self.ingest_alpha(second_project)
        racer = b"destination reoccupied by an external writer\n"

        failed, result = self.raced_promote(
            "after-claim-before-install",
            lambda: wiki_path.write_bytes(racer),
            project=second_project,
        )

        self.assertEqual(failed.returncode, 3, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-conflict")
        self.assertEqual(wiki_path.read_bytes(), racer)
        claims = list(wiki_path.parent.glob(".hive-wiki-claim-*"))
        self.assertEqual(len(claims), 1)
        self.assertEqual(claims[0].read_bytes(), prior)

    @unittest.skipIf(os.name == "nt", "POSIX mode race")
    def test_mode_change_between_plan_and_claim_is_preserved(self) -> None:
        first = self.promote(self.project)[1]
        wiki_path = (
            self.user_root
            / ".hive/knowledge/Wiki"
            / f"{first['data']['page_id']}.md"
        )
        prior = wiki_path.read_bytes()
        prior_mode = stat.S_IMODE(wiki_path.stat().st_mode)
        changed_mode = prior_mode ^ stat.S_IRGRP
        second_project = self.setup_project("project-b", "stable-project-b")
        self.ingest_alpha(second_project)

        failed, result = self.raced_promote(
            "after-plan-before-claim",
            lambda: wiki_path.chmod(changed_mode),
            project=second_project,
        )

        self.assertEqual(failed.returncode, 3, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-conflict")
        self.assertEqual(wiki_path.read_bytes(), prior)
        self.assertEqual(stat.S_IMODE(wiki_path.stat().st_mode), changed_mode)

    def test_rollback_preserves_external_edit_over_transaction_install(self) -> None:
        planned = self.promote(self.project, mode="--dry-run")[1]
        wiki_path = (
            self.user_root
            / ".hive/knowledge/Wiki"
            / f"{planned['data']['page_id']}.md"
        )
        racer = b"external edit racing transaction rollback\n"

        failed, result = self.raced_promote(
            "before-rollback",
            lambda: wiki_path.write_bytes(racer),
            fail_after_writes=True,
        )

        self.assertEqual(failed.returncode, 10, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-io-error")
        self.assertIn("rollback failed", result["message"])
        self.assertEqual(wiki_path.read_bytes(), racer)
        self.assertFalse(
            any(
                (
                    self.user_root
                    / ".hive/knowledge/Raw/promoted-knowledge"
                ).glob("*.json")
            )
        )

    def test_mismatched_user_store_binding_fails_without_root_mutation(self) -> None:
        answers_path = self.project / ".hive/setup-answers.yml"
        answers = read_yaml(answers_path)
        answers["user_store_binding"] = "sha256:" + ("0" * 64)
        write_yaml(answers_path, answers)

        before = snapshot_tree(self.user_root)
        failed, result = self.promote(self.project)
        self.assertEqual(failed.returncode, 3, failed.stderr)
        self.assertEqual(result["code"], "hive.knowledge-conflict")
        self.assertIn("user-store binding", result["message"])
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.user_root), before)

    def test_secret_confidential_and_contradiction_fail_without_root_mutation(
        self,
    ) -> None:
        page = self.project / ".hive/knowledge/Wiki/alpha.md"
        safe_page = page.read_bytes()
        page.write_bytes(
            safe_page
            + b"\nCredential ghp_abcdefghijklmnopqrstuvwxyz012345 must never promote.\n"
        )
        before_secret = snapshot_tree(self.user_root)
        secret, secret_result = self.promote(self.project)
        self.assertEqual(secret.returncode, 5, secret.stderr)
        self.assertEqual(secret_result["status"], "verification-failed")
        self.assertEqual(snapshot_tree(self.user_root), before_secret)
        page.write_bytes(safe_page)

        answers_path = self.project / ".hive/setup-answers.yml"
        answers = read_yaml(answers_path)
        answers["confidential_knowledge_categories"] = ["fact"]
        write_yaml(answers_path, answers)
        before_confidential = snapshot_tree(self.user_root)
        confidential, confidential_result = self.promote(self.project)
        self.assertEqual(confidential.returncode, 3, confidential.stderr)
        self.assertEqual(confidential_result["status"], "conflict")
        self.assertEqual(snapshot_tree(self.user_root), before_confidential)
        answers["confidential_knowledge_categories"] = []
        write_yaml(answers_path, answers)

        first, first_result = self.promote(self.project)
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(first_result["code"], "hive.knowledge-promoted")
        second_project = self.setup_project("project-b", "stable-project-b")
        self.ingest_alpha(second_project)
        second_page = second_project / ".hive/knowledge/Wiki/alpha.md"
        second_page.write_text(
            second_page.read_text(encoding="utf-8").replace(
                "The local index is rebuilt from canonical Markdown.",
                "The local index uses a contradictory noncanonical source.",
            ),
            encoding="utf-8",
        )
        before_contradiction = snapshot_tree(self.user_root / ".hive/knowledge")
        contradiction, contradiction_result = self.promote(second_project)
        self.assertEqual(contradiction.returncode, 3, contradiction.stderr)
        self.assertEqual(contradiction_result["status"], "conflict")
        self.assertEqual(
            snapshot_tree(self.user_root / ".hive/knowledge"),
            before_contradiction,
        )

    def test_concurrent_duplicate_promotion_is_serialized_and_idempotent(self) -> None:
        second_project = self.setup_project("project-b", "stable-project-b")
        self.ingest_alpha(second_project)
        with ThreadPoolExecutor(max_workers=2) as executor:
            results = list(
                executor.map(
                    self.promote,
                    [self.project, second_project],
                )
            )
        for process, result in results:
            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(result["code"], "hive.knowledge-promoted")
        root_wiki = self.user_root / ".hive/knowledge/Wiki"
        promoted = list(root_wiki.glob("shared-fact-*.md"))
        self.assertEqual(len(promoted), 1)
        frontmatter = yaml.safe_load(
            promoted[0].read_text(encoding="utf-8").split("---", 2)[1]
        )
        self.assertEqual(frontmatter["sources"], sorted(frontmatter["sources"]))
        self.assertEqual(len(frontmatter["sources"]), 2)
        expected_sources = frontmatter["sources"]
        raw_objects = sorted(
            (
                path,
                json.loads(path.read_text(encoding="utf-8")),
            )
            for path in (
                self.user_root / ".hive/knowledge/Raw/promoted-knowledge"
            ).glob("*.json")
        )
        self.assertEqual(len(raw_objects), 2)
        self.assertEqual(
            {
                raw["project_pseudonym"]
                for _, raw in raw_objects
            },
            {
                "sha256:" + hashlib.sha256(b"stable-project-a").hexdigest(),
                "sha256:" + hashlib.sha256(b"stable-project-b").hexdigest(),
            },
        )
        before_repeat = snapshot_tree(self.user_root)
        for project in (self.project, second_project):
            repeated, repeated_result = self.promote(project)
            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertEqual(repeated_result["code"], "hive.knowledge-promoted")
        self.assertEqual(snapshot_tree(self.user_root), before_repeat)
        queried = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.project),
            "--user-root",
            str(self.user_root),
            "--text",
            "deterministic local knowledge",
        )[1]
        self.assertEqual(
            queried["data"]["precedence"],
            "own-project,user-root,shared",
        )
        self.assertEqual(queried["data"]["project_hit_count"], 1)
        self.assertEqual(queried["data"]["root_hit_count"], 1)
        self.assertEqual(
            [
                (hit["source_project"] == "user-root", hit["visibility"])
                for hit in queried["data"]["hits"]
            ],
            [
                (False, "project-private"),
                (True, "shared"),
            ],
        )
        root_hit = next(
            hit
            for hit in queried["data"]["hits"]
            if hit["source_project"] == "user-root"
        )
        self.assertEqual(root_hit["sources"], expected_sources)
        self.assertEqual(root_hit["sources"], sorted(set(root_hit["sources"])))
        for locator in root_hit["sources"]:
            self.assertNotIn("stable-project-a", locator)
            self.assertNotIn("stable-project-b", locator)
            self.assertNotIn(str(self.project), locator)

    def test_copy_ready_skill_commands_execute_with_bound_user_root(self) -> None:
        promote_skill = (
            REPOSITORY_ROOT
            / "harness/skills/knowledge-promote/SKILL.md"
        )
        replacements = {
            "<project-root>": str(self.project),
            "<user-root>": str(self.user_root),
            "<id>": "alpha",
            "<category>": "fact",
        }
        dry, dry_result = self.invoke_skill_command(
            promote_skill,
            "--dry-run",
            replacements,
        )
        self.assertEqual(dry.returncode, 0, dry.stderr)
        self.assertEqual(dry_result["code"], "hive.knowledge-promotion-planned")
        applied, applied_result = self.invoke_skill_command(
            promote_skill,
            "--apply",
            replacements,
        )
        self.assertEqual(applied.returncode, 0, applied.stderr)
        self.assertEqual(applied_result["code"], "hive.knowledge-promoted")

        query, query_result = self.invoke_skill_command(
            REPOSITORY_ROOT / "harness/skills/knowledge-recall/SKILL.md",
            "--scope auto --query <query>",
            {
                "<current-project-root>": str(self.project),
                "<user-root>": str(self.user_root),
                "<query>": "deterministic",
            },
        )
        self.assertEqual(query.returncode, 0, query.stderr)
        self.assertCountEqual(
            [
                (hit["collection_id"] == "user-root", hit["visibility"])
                for hit in query_result["data"]["hits"]
            ],
            [
                (False, "project-private"),
                (True, "shared"),
            ],
        )
        skill_root_hit = next(
            hit
            for hit in query_result["data"]["hits"]
            if hit["collection_id"] == "user-root"
        )
        self.assertEqual(
            skill_root_hit["sources"],
            sorted(set(skill_root_hit["sources"])),
        )
