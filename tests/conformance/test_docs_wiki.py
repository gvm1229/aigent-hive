"""Human-readable docs Wiki structure and README knowledge preservation."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
MARKDOWN_LINK = re.compile(r"\[[^\]]+\]\(([^)]+\.md(?:#[^)]+)?)\)")


class DocsWikiConformance(unittest.TestCase):
    def test_document_home_reaches_every_tracked_docs_markdown_file(self) -> None:
        pending = [DOCS / "00-home.md"]
        reached: set[Path] = set()

        while pending:
            current = pending.pop()
            current = current.resolve()
            if current in reached or not current.is_relative_to(DOCS.resolve()):
                continue
            self.assertTrue(current.is_file(), f"missing docs Wiki link target: {current}")
            reached.add(current)
            text = current.read_text(encoding="utf-8")
            for raw_target in MARKDOWN_LINK.findall(text):
                target = raw_target.split("#", 1)[0]
                if "://" in target:
                    continue
                resolved = (current.parent / target).resolve()
                if resolved.is_relative_to(DOCS.resolve()) and resolved not in reached:
                    pending.append(resolved)

        expected = {path.resolve() for path in DOCS.rglob("*.md")}
        missing = sorted(
            path.relative_to(DOCS.resolve()).as_posix() for path in expected - reached
        )
        self.assertEqual(missing, [])

    def test_streamlined_readmes_link_to_the_preserved_knowledge(self) -> None:
        english = (ROOT / "README.md").read_text(encoding="utf-8")
        korean = (DOCS / "readme/README.ko.md").read_text(encoding="utf-8")
        for readme in (english, korean):
            self.assertIn("docs/00-home.md", readme.replace("../", "docs/"))
            self.assertIn("docs/01-index.md", readme.replace("../", "docs/"))

    def test_old_readme_knowledge_has_current_topic_documents(self) -> None:
        product = (DOCS / "overview/product.md").read_text(encoding="utf-8")
        development = (DOCS / "guides/development.md").read_text(encoding="utf-8")
        index = (DOCS / "01-index.md").read_text(encoding="utf-8")

        for heading in ("## 지원 범위", "## 핵심 원칙", "## 주요 기능"):
            self.assertIn(heading, product)
        for heading in ("## 기술 stack", "## Rust dependency", "## 빠른 검증"):
            self.assertIn(heading, development)
        for topic in (
            "Architecture",
            "Decisions",
            "Guides",
            "Research",
            "Facts",
            "State",
            "Plans",
        ):
            self.assertIn(f"## {topic}", index)


if __name__ == "__main__":
    unittest.main()
