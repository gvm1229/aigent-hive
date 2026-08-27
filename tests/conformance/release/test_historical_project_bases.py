"""Exact source-byte checks for the full historical project-base registry."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
VERSIONS = ("0.9.1", "0.9.2", "0.9.3", "0.9.4")


def source_paths(version: str) -> dict[str, Path]:
    paths = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", f"v{version}", "harness/template"],
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
