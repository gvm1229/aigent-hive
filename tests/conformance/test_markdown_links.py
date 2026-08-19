from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/check-markdown-links.py"
SPEC = importlib.util.spec_from_file_location("markdown_links", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class MarkdownLinkTest(unittest.TestCase):
    def test_inventory_excludes_frozen_archive_but_keeps_navigation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            (root / "docs/archive/state").mkdir(parents=True)
            (root / "docs/archive/README.md").write_text("# Archive\n", encoding="utf-8")
            (root / "docs/archive/MANIFEST.md").write_text("# Manifest\n", encoding="utf-8")
            (root / "docs/archive/state/old.md").write_text(
                "[missing](gone.md)\n", encoding="utf-8"
            )
            subprocess.run(["git", "add", "."], cwd=root, check=True)

            report = MODULE.scan(root)

            self.assertEqual(report["checked_files"], 2)
            self.assertEqual(report["failure_count"], 0)

    def make_repository(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        return temporary, root

    def test_valid_relative_links_anchors_and_duplicate_headings(self) -> None:
        temporary, root = self.make_repository()
        with temporary:
            (root / "docs").mkdir()
            (root / "README.md").write_text(
                "[문서](docs/guide.md#한국어-heading) "
                "[중복](docs/guide.md#repeat-1)\n",
                encoding="utf-8",
            )
            (root / "docs/guide.md").write_text(
                "# 한국어 Heading\n\n## Repeat\n\n## Repeat\n",
                encoding="utf-8",
            )
            report = MODULE.scan(root)
            self.assertEqual(report["checked_files"], 2)
            self.assertEqual(report["checked_links"], 2)
            self.assertEqual(report["failures"], [])

    def test_reports_missing_file_and_heading_anchor(self) -> None:
        temporary, root = self.make_repository()
        with temporary:
            (root / "README.md").write_text(
                "[missing](absent.md)\n[anchor](guide.md#absent)\n",
                encoding="utf-8",
            )
            (root / "guide.md").write_text("# Present\n", encoding="utf-8")
            report = MODULE.scan(root)
            self.assertEqual(
                [(item["target"], item["reason"]) for item in report["failures"]],
                [
                    ("absent.md", "missing-target"),
                    ("guide.md#absent", "missing-anchor"),
                ],
            )

    def test_ignores_external_inline_code_comments_and_fenced_examples(self) -> None:
        temporary, root = self.make_repository()
        with temporary:
            (root / "README.md").write_text(
                "[external](https://example.invalid/missing)\n"
                "`[inline](missing-inline.md)`\n"
                "<!-- [comment](missing-comment.md) -->\n"
                "```md\n[fenced](missing-fenced.md)\n```\n",
                encoding="utf-8",
            )
            report = MODULE.scan(root)
            self.assertEqual(report["checked_links"], 0)
            self.assertEqual(report["failures"], [])

    def test_inventory_includes_tracked_and_untracked_nonignored_markdown(self) -> None:
        temporary, root = self.make_repository()
        with temporary:
            (root / ".gitignore").write_text("ignored/\n", encoding="utf-8")
            (root / "tracked.md").write_text("# Tracked\n", encoding="utf-8")
            subprocess.run(["git", "add", ".gitignore", "tracked.md"], cwd=root, check=True)
            (root / "untracked.md.jinja").write_text("# Template\n", encoding="utf-8")
            (root / "deleted.md").write_text("# Deleted\n", encoding="utf-8")
            subprocess.run(["git", "add", "deleted.md"], cwd=root, check=True)
            (root / "deleted.md").unlink()
            (root / "ignored").mkdir()
            (root / "ignored/ignored.md").write_text("# Ignored\n", encoding="utf-8")
            self.assertEqual(
                [path.relative_to(root).as_posix() for path in MODULE.inventory(root)],
                ["tracked.md", "untracked.md.jinja"],
            )


if __name__ == "__main__":
    unittest.main()
