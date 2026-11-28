"""Project sync from the global Hive: range, forward-only, lock and no-touch rules (OV170-029–032)."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tomllib
from pathlib import Path
from zipfile import ZipFile

from jsonschema import Draft202012Validator, FormatChecker

from tests.conformance.support.harness import (
    ACTION_RESULT_SCHEMA,
    Phase1CliTestCase,
    REPOSITORY_ROOT,
    snapshot_tree,
)

PRODUCT_VERSION = tomllib.loads(
    (REPOSITORY_ROOT / "Cargo.toml").read_text(encoding="utf-8")
)["workspace"]["package"]["version"]
PREDECESSOR = REPOSITORY_ROOT / "tests/fixtures/project-predecessors/0.16.1/public-stable"
GIT_ENV = {"GIT_AUTHOR_NAME": "Fixture", "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
           "GIT_COMMITTER_NAME": "Fixture", "GIT_COMMITTER_EMAIL": "fixture@example.invalid"}


class ProjectSyncConformance(Phase1CliTestCase):
    def run_hive(self, *arguments: str, machine: bool = True) -> subprocess.CompletedProcess[str]:
        command = [str(self.hive_binary), *arguments]
        if machine:
            command += ["--output", "json"]
        return subprocess.run(command, cwd=self.work_root, check=False, text=True,
                              capture_output=True, env={**os.environ, "HIVE_PROJECT_SYNC": ""})

    def sync(self, target: Path, *flags: str) -> tuple[int, dict]:
        process = self.run_hive("project", "sync", "--target", str(target), *flags)
        result = json.loads(process.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA, format_checker=FormatChecker()).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process.returncode, result

    def git(self, target: Path, *arguments: str) -> str:
        return subprocess.run(["git", *arguments], cwd=target, check=True, text=True,
                              capture_output=True, env={**os.environ, **GIT_ENV}).stdout

    def predecessor(self, name: str) -> Path:
        metadata = json.loads((PREDECESSOR / "manifest.json").read_bytes())
        archive = PREDECESSOR / metadata["archive"]
        self.assertEqual(hashlib.sha256(archive.read_bytes()).hexdigest(), metadata["sha256"])
        target = self.work_root / name
        with ZipFile(archive) as package:
            package.extractall(target)
        self.git(target, "init", "--quiet", "--initial-branch=main")
        self.commit(target, "fixture")
        return target

    def commit(self, target: Path, message: str) -> None:
        self.git(target, "add", "-A")
        self.git(target, "commit", "--quiet", "--allow-empty", "-m", message)

    def accept_range(self, target: Path, value: str) -> None:
        (target / ".hive/project.toml").write_text(
            f'schema_version = 1\n\n[hive]\nrange = "{value}"\n', encoding="utf-8")
        self.commit(target, f"range {value}")

    def test_out_of_range_then_upgrade_records_lock_and_check_detects_drift(self) -> None:
        target = self.predecessor("ranged")
        before = snapshot_tree(target)
        code, result = self.sync(target)
        self.assertEqual((code, result["code"]), (0, "hive.project-sync-out-of-range"))
        self.assertEqual(result["data"]["range"], "0.16")
        self.assertEqual(snapshot_tree(target), before, "outside the range nothing changes")

        self.accept_range(target, "0.17")
        code, result = self.sync(target)
        self.assertEqual((code, result["code"]), (0, "hive.project-synced"), result)
        self.assertEqual((result["data"]["from_version"], result["data"]["to_version"]),
                         ("0.16.1", PRODUCT_VERSION))
        self.assertEqual(result["actor"], "Eileithyia")
        self.assertIn(".hive/project.lock", result["changed_paths"])
        self.assertEqual(len(result["items"]), len(result["changed_paths"]))
        self.assertTrue(result["notice"].startswith(
            f"Eileithyia called · project Hive 0.16.1 → {PRODUCT_VERSION}, files: "))
        lock = tomllib.loads((target / ".hive/project.lock").read_text(encoding="utf-8"))
        self.assertEqual((lock["hive_version"], lock["range"]), (PRODUCT_VERSION, "0.17"))
        base = json.loads((target / ".hive/config/project-base.json").read_bytes())
        self.assertEqual(lock["base_ledger_digest"], base["ledger_digest"])
        self.assertEqual([entry["path"] for entry in lock["files"]],
                         [entry["path"] for entry in base["files"]])
        self.assertEqual(self.run_hive("project", "upgrade", "--target", str(target),
                                       "--validate").returncode, 0)
        self.assertEqual(self.sync(target, "--check")[0], 0)
        self.commit(target, "sync")

        code, again = self.sync(target)
        self.assertEqual((code, again["code"], again["changed_paths"]),
                         (0, "hive.project-sync-current", []))

        lock_path = target / ".hive/project.lock"
        good = lock_path.read_text(encoding="utf-8")
        lock_path.write_text(good.replace(base["ledger_digest"], "sha256:" + "0" * 64), encoding="utf-8")
        code, drift = self.sync(target, "--check")
        self.assertEqual(code, 5)
        self.assertIn("different commits", drift["message"])
        lock_path.write_text("<<<<<<< ours\n" + good + "=======\n" + good + ">>>>>>> theirs\n",
                             encoding="utf-8")
        code, conflicted = self.sync(target, "--check")
        self.assertEqual(code, 5)
        self.assertIn("conflict markers", conflicted["message"])
        self.assertIn("<<<<<<<", lock_path.read_text(encoding="utf-8"), "check never writes")
        self.git(target, "add", "-A")
        self.git(target, "commit", "--quiet", "-m", "conflicted lock")
        code, recomputed = self.sync(target)
        self.assertEqual(code, 0, recomputed)
        self.assertTrue(recomputed["data"]["lock_recomputed"])
        self.assertEqual(lock_path.read_text(encoding="utf-8"), good)
        self.assertEqual(self.sync(target, "--check")[0], 0)

        # Migration of a current project: Hive writes the settings once, then only people edit it.
        (target / ".hive/project.toml").unlink()
        self.commit(target, "settings removed")
        code, migrated = self.sync(target)
        self.assertEqual((code, migrated["code"], migrated["changed_paths"]),
                         (0, "hive.project-sync-recorded", [".hive/project.toml"]))
        settings = tomllib.loads((target / ".hive/project.toml").read_text(encoding="utf-8"))
        self.assertEqual(settings["hive"]["range"], ".".join(PRODUCT_VERSION.split(".")[:2]))
        self.assertEqual(settings["sections"]["hooks"]["protection"], "user-consent-protected")
        for name, document in (("project-settings", settings),
                               ("project-lock", tomllib.loads(lock_path.read_text(encoding="utf-8")))):
            schema = json.loads((REPOSITORY_ROOT / f"schemas/{name}.schema.json").read_text(encoding="utf-8"))
            Draft202012Validator(schema, format_checker=FormatChecker()).validate(document)

    def test_older_global_hive_never_rewrites_a_newer_project(self) -> None:
        target = self.predecessor("forward-only")
        self.accept_range(target, "0.17")
        self.assertEqual(self.sync(target)[1]["code"], "hive.project-synced")
        lock_path = target / ".hive/project.lock"
        lock_path.write_text(lock_path.read_text(encoding="utf-8").replace(
            f'hive_version = "{PRODUCT_VERSION}"', 'hive_version = "0.17.99"'), encoding="utf-8")
        self.commit(target, "teammate on a newer Hive")
        before = snapshot_tree(target)
        code, result = self.sync(target)
        self.assertEqual((code, result["code"]), (0, "hive.project-sync-older-hive"))
        self.assertEqual(result["data"]["from_version"], "0.17.99")
        self.assertEqual(snapshot_tree(target), before)

    def test_sync_waits_during_merge_and_with_uncommitted_hive_edits(self) -> None:
        target = self.predecessor("waiting")
        self.accept_range(target, "0.17")
        merge_head = target / ".git/MERGE_HEAD"
        merge_head.write_text(self.git(target, "rev-parse", "HEAD"), encoding="utf-8")
        before = snapshot_tree(target)
        code, result = self.sync(target)
        self.assertEqual((code, result["code"], result["data"]["reason"]),
                         (0, "hive.project-sync-skipped", "rebase-or-merge"))
        self.assertEqual(snapshot_tree(target), before)
        merge_head.unlink()
        agents = target / "AGENTS.md"
        agents.write_text(agents.read_text(encoding="utf-8") + "\nLocal note.\n", encoding="utf-8")
        before = snapshot_tree(target)
        code, result = self.sync(target)
        self.assertEqual((result["code"], result["data"]["reason"]),
                         ("hive.project-sync-skipped", "uncommitted-hive-changes"))
        self.assertEqual(snapshot_tree(target), before)

    def test_overlapping_user_edit_is_left_for_review(self) -> None:
        target = self.predecessor("overlap")
        self.accept_range(target, "0.17")
        process = self.run_hive("project", "upgrade", "--target", str(target), "--dry-run")
        preview = json.loads(process.stdout)
        self.assertEqual(process.returncode, 0, preview)
        changed = next(report["path"] for report in preview["data"]["reports"]
                       if report["disposition"] == "incoming-replace"
                       and report["path"].endswith(".md") and "/skills/" in report["path"])
        path = target / changed
        path.write_text("\n".join("local rule" for _ in path.read_text(encoding="utf-8").splitlines()) + "\n",
                        encoding="utf-8")
        self.commit(target, "local rewrite")
        before = snapshot_tree(target)
        code, result = self.sync(target)
        self.assertEqual((code, result["code"]), (0, "hive.project-sync-needs-review"), result)
        self.assertIn(changed, result["data"]["conflicts"])
        self.assertIn("--dry-run", result["next_action"])
        self.assertEqual(snapshot_tree(target), before)

    def test_folder_without_hive_is_never_written_prompted_or_offered_setup(self) -> None:
        plain = self.work_root / "plain"
        plain.mkdir()
        (plain / "README.md").write_text("team project\n", encoding="utf-8")
        removed = self.predecessor("removed-hive")
        subprocess.run(["rm", "-rf", str(removed / ".hive")], check=True)
        for folder in (plain, removed):
            before = snapshot_tree(folder)
            hook = subprocess.run([str(self.hive_binary), "project", "sync", "--hook", "claude",
                                   "--stdin-json"], cwd=self.work_root, check=False, text=True,
                                  capture_output=True, input=json.dumps({"cwd": str(folder)}))
            self.assertEqual((hook.returncode, hook.stdout), (0, ""))
            code, result = self.sync(folder)
            self.assertEqual((code, result["code"], result["changed_paths"]),
                             (0, "hive.project-sync-no-hive", []))
            for offer in ("setup", "install", "설치"):
                self.assertNotIn(offer, result["message"])
            self.assertEqual(snapshot_tree(folder), before)
            self.assertFalse((folder / ".hive").exists())

    def test_claude_hook_prints_one_line_and_can_be_turned_off(self) -> None:
        target = self.predecessor("hooked")
        self.accept_range(target, "0.17")
        off = subprocess.run([str(self.hive_binary), "project", "sync", "--hook", "claude",
                              "--target", str(target)], check=False, text=True, capture_output=True,
                             env={**os.environ, "HIVE_PROJECT_SYNC": "off"})
        self.assertEqual((off.returncode, off.stdout), (0, ""))
        self.assertFalse((target / ".hive/project.lock").exists())
        hook = subprocess.run([str(self.hive_binary), "project", "sync", "--hook", "claude",
                               "--target", str(target)], check=False, text=True, capture_output=True,
                              env={**os.environ, "HIVE_PROJECT_SYNC": ""})
        self.assertEqual(hook.returncode, 0, hook.stderr)
        payload = json.loads(hook.stdout)
        line = payload["systemMessage"]
        self.assertEqual(payload["hookSpecificOutput"],
                         {"hookEventName": "SessionStart", "additionalContext": line})
        self.assertNotIn("\n", line)
        self.assertIn("Eileithyia", line)
        self.assertIn(f"0.16.1 → {PRODUCT_VERSION}", line)
        self.assertEqual(self.git(target, "log", "--format=%s", "-1").strip(), "range 0.17",
                         "sync never commits")
