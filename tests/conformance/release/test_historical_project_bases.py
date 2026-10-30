"""Exact source-byte checks for the full historical project-base registry."""

from __future__ import annotations

import subprocess
import unittest
import json
import hashlib
import os
import shutil

import yaml
from pathlib import Path
from tests.conformance.support.harness import FIXTURE_ROOT, Phase1CliTestCase, snapshot_tree


ROOT = Path(__file__).resolve().parents[3]
VERSIONS = ("0.9.1", "0.9.2", "0.9.3", "0.9.4")


def source_paths(version: str, ref: str | None = None) -> dict[str, Path]:
    paths = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", ref or f"v{version}", "harness/template"],
        cwd=ROOT,
        text=True,
    ).splitlines()
    mapped: dict[str, Path] = {}
    for source in paths:
        suffix = source.removeprefix("harness/template/")
        if suffix == "AGENTS.md.jinja":
            mapped[source] = ROOT / "harness/project-bases" / version / "AGENTS.md.template"
        elif suffix.startswith(".agents/directives/"):
            mapped[source] = (
                ROOT
                / "harness/project-bases"
                / version
                / "directives"
                / suffix.removeprefix(".agents/directives/")
            )
        elif suffix.startswith(".agents/skills/"):
            mapped[source] = (
                ROOT
                / "harness/project-bases"
                / version
                / "skills"
                / suffix.removeprefix(".agents/skills/")
            )
    return mapped


class HistoricalProjectBaseContract(unittest.TestCase):
    def test_public_0132_test2_is_the_exact_source_of_its_compatibility_bases(self) -> None:
        ref = "v0.13.2-test.2"
        self.assertEqual(subprocess.check_output(["git", "rev-parse", ref + "^{commit}"],
            cwd=ROOT, text=True).strip(), "ed5a56d14564d2acaa778067f6712c9ef494fbc8")
        mapped = source_paths("0.13.2", ref)
        self.assertEqual(set(mapped.values()),
            {p for p in (ROOT / "harness/project-bases/0.13.2").rglob("*") if p.is_file()})
        user = ROOT / "harness/user-bases/0.13.2"
        for path in user.rglob("*"):
            if path.is_file():
                mapped["harness/" + path.relative_to(user).as_posix()] = path
        self.assertEqual(len(mapped), 148)
        for source, destination in mapped.items():
            self.assertEqual(destination.read_bytes(),
                subprocess.check_output(["git", "show", ref + ":" + source], cwd=ROOT), source)
        public = json.loads(subprocess.check_output(["git", "show",
            ref + ":harness/release/0.13.2/release-surface-inventory.json"], cwd=ROOT))
        registry = yaml.safe_load((ROOT / "harness/release/historical-surfaces.yml").read_text())
        self.assertEqual([r for r in registry["releases"] if r["product_version"] == "0.13.2"], [public])

    def test_latest_frozen_base_checkout_preserves_stored_bytes(self) -> None:
        registry = yaml.safe_load((ROOT / "harness/project-bases/registry.yml").read_text(encoding="utf-8"))
        version = registry["releases"][-1]["version"]
        for family in ("project-bases", "user-bases"):
            prefix = f"harness/{family}/{version}"
            paths = subprocess.check_output(["git", "ls-files", "--", prefix], cwd=ROOT, text=True).splitlines()
            self.assertTrue(paths, prefix)
            for path in paths:
                with self.subTest(path=path):
                    stored = subprocess.check_output(["git", "cat-file", "blob", f"HEAD:{path}"], cwd=ROOT)
                    checkout = subprocess.check_output(["git", "cat-file", "--filters", f"HEAD:{path}"], cwd=ROOT)
                    self.assertEqual(checkout, stored, "checkout filters must not rewrite frozen evidence")

    def test_test4_vector_skill_deltas_match_the_exact_published_source(self) -> None:
        for name in ("knowledge-recall", "knowledge-maintain"):
            actual = (ROOT / "harness/project-bases/0.10.0-test.4/skills" / name / "SKILL.md").read_bytes()
            expected = subprocess.check_output(
                ["git", "show", f"5cedbed4b27f2d6a224d1ea841d1670c7599a452:harness/skills/{name}/SKILL.md"], cwd=ROOT
            )
            self.assertEqual(actual, expected, name)

    def test_test2_deltas_match_the_exact_published_source(self) -> None:
        for source, destination in (
            ("harness/template/.agents/directives/04-korean-language.md", "directives/04-korean-language.md"),
            ("harness/template/.agents/skills/humanize-kor/SKILL.md", "skills/humanize-kor/SKILL.md"),
        ):
            actual = (ROOT / "harness/project-bases/0.10.0-test.2" / destination).read_bytes()
            expected = subprocess.check_output(["git", "show", f"4b9275ae90c08f31dce82085b9cda939a623a975:{source}"], cwd=ROOT)
            self.assertEqual(actual, expected, source)

    def test_full_historical_project_bases_match_their_release_templates_byte_for_byte(self) -> None:
        for version in VERSIONS:
            mapped = source_paths(version)
            self.assertTrue(mapped, version)
            expected = set(mapped.values())
            base = ROOT / "harness/project-bases" / version
            actual = {path for path in base.rglob("*") if path.is_file()}
            self.assertEqual(actual, expected)
            for source, destination in mapped.items():
                release_bytes = subprocess.check_output(
                    ["git", "show", f"v{version}:{source}"], cwd=ROOT
                )
                self.assertEqual(destination.read_bytes(), release_bytes, source)


@unittest.skipUnless(os.name == "nt" and os.environ.get("HIVE_PUBLIC_0132_TEST2"),
    "requires the explicitly selected, pinned public 0.13.2-test.2 Windows binary")
class Public132ProjectUpgradeConformance(Phase1CliTestCase):
    def test_public_test2_project_upgrades_and_recovers_without_losing_user_bytes(self):
        source = Path(os.environ["HIVE_PUBLIC_0132_TEST2"]).resolve()
        self.assertTrue(source.is_relative_to((ROOT / "tests/work").resolve()))
        expected = "e250fd16b530060f6067a1779f658c050cd1f4c6212b94fad742634ed7f72853"
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), expected)
        program = self.work_root / "public/hive.exe"
        program.parent.mkdir()
        shutil.copyfile(source, program)
        target = self.work_root / "consumer"
        target.mkdir()
        agent = target / "AGENTS.md"
        agent.write_bytes(b"User-owned preface.\r\n")
        settings = self.setup_user_root / ".hive/config/user-setup.yml"
        settings_before = settings.read_bytes()
        old = subprocess.run([str(program), "setup", "--target", str(target), "--user-root",
            str(self.setup_user_root), "--answers", str(FIXTURE_ROOT / "answers-no-role-no-hook.yml"),
            "--capabilities", str(FIXTURE_ROOT / "capabilities-codex-host-native.json"),
            "--apply", "--output", "json"], capture_output=True, text=True, encoding="utf-8", timeout=40)
        self.assertEqual(old.returncode, 0, (old.stdout, old.stderr))
        harness = target / ".hive/config/harness.toml"
        self.assertIn('harness_version = "0.13.2"', harness.read_text())
        suffix = b"\n<!-- User-owned suffix. -->\n"
        agent.write_bytes(agent.read_bytes() + suffix)
        note = target / "FOREIGN.md"
        note.write_bytes(b"Preserve these bytes.\r\n")
        command = [str(self.hive_binary), "project", "upgrade", "--target", str(target),
            "--user-root", str(self.setup_user_root), "--output", "json"]
        base_path = target / ".hive/config/project-base.json"
        original_base = base_path.read_bytes()
        unregistered = json.loads(original_base)
        self.assertEqual(unregistered["package_version"], "0.13.2-test.2")
        recorded_digest = unregistered.pop("ledger_digest")
        def digest(value):
            return "sha256:" + hashlib.sha256(json.dumps(value, ensure_ascii=False,
                sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()
        self.assertEqual(digest(unregistered), recorded_digest)
        unregistered["package_version"] = "0.13.2-test.999"
        unregistered["ledger_digest"] = digest(unregistered)
        base_path.write_bytes(json.dumps(unregistered, ensure_ascii=False,
            sort_keys=True, separators=(",", ":")).encode("utf-8") + b"\n")
        forged_tree = snapshot_tree(target)
        rejected = subprocess.run([*command, "--scan"], capture_output=True, text=True, encoding="utf-8", timeout=40)
        self.assertEqual(rejected.returncode, 4, (rejected.stdout, rejected.stderr))
        self.assertEqual(snapshot_tree(target), forged_tree)
        base_path.write_bytes(original_base)
        before = snapshot_tree(target)
        for mode in ("--scan", "--dry-run"):
            read = subprocess.run([*command, mode], capture_output=True, text=True, encoding="utf-8", timeout=40)
            self.assertEqual(read.returncode, 0, (read.stdout, read.stderr))
            self.assertEqual(snapshot_tree(target), before)
        failed = subprocess.run([*command, "--apply"], capture_output=True, text=True, encoding="utf-8", timeout=40,
            env={**os.environ, "HIVE_PROJECT_UPGRADE_FAIL_AFTER":"1"})
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn('harness_version = "0.13.2"', harness.read_text())
        for mode in ("--apply", "--validate"):
            current = subprocess.run([*command, mode], capture_output=True, text=True, encoding="utf-8", timeout=40)
            self.assertEqual(current.returncode, 0, (current.stdout, current.stderr))
            self.assertEqual(json.loads(current.stdout)["status"], "success")
            self.assertTrue(agent.read_bytes().startswith(b"User-owned preface.\r\n"))
            self.assertTrue(agent.read_bytes().endswith(suffix))
            self.assertEqual(note.read_bytes(), b"Preserve these bytes.\r\n")
            self.assertEqual(settings.read_bytes(), settings_before)
        self.assertIn('harness_version = "0.14.0"', harness.read_text())
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), expected)
