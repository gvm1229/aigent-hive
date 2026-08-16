"""Human-readable docs Wiki structure and README knowledge preservation."""

from __future__ import annotations

import hashlib
import re
import unittest
from pathlib import Path

import yaml


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

    def test_global_knowledge_bundle_guidance_keeps_shell_roots_separate(self) -> None:
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        install_guide = (DOCS / "hive-install-guide.ko.html").read_text(encoding="utf-8")

        for text in (readme, install_guide):
            self.assertIn("hive knowledge export --user-root", text)
            self.assertIn("hive knowledge import --user-root", text)
            self.assertIn("--scope global", text)
            self.assertIn("--dry-run --output json", text)
            self.assertIn("--apply --output json", text)
            self.assertIn("$HOME", text)
            self.assertIn("$env:USERPROFILE", text)

        self.assertIn('"$HOME" --scope global', readme)
        self.assertIn('$env:USERPROFILE --scope global', readme)
        self.assertIn("$HOME</code>과 Windows PowerShell", install_guide)

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

    def test_fact_pages_are_atomic_exact_bilingual_pairs(self) -> None:
        facts = DOCS / "facts"
        pages: dict[tuple[str, str], tuple[dict[str, object], str]] = {}
        for language in ("en", "ko"):
            for path in sorted((facts / language).glob("*.md")):
                text = path.read_text(encoding="utf-8")
                self.assertTrue(text.startswith("---\n"), path)
                frontmatter_text, body = text[4:].split("\n---\n", 1)
                frontmatter = yaml.safe_load(frontmatter_text)
                slug = path.stem
                self.assertEqual(frontmatter["topic_slug"], slug)
                self.assertEqual(frontmatter["pair_id"], slug)
                self.assertEqual(frontmatter["language"], language)
                other = "ko" if language == "en" else "en"
                self.assertEqual(frontmatter["counterpart"], f"../{other}/{slug}.md")
                self.assertEqual(body.count("\n# "), 1)
                self.assertNotIn("\n## ", body)
                self.assertLessEqual(len(body.encode("utf-8")), 800)
                for field in ("links", "sources", "tags"):
                    values = frontmatter[field]
                    self.assertEqual(values, sorted(set(values)))
                for source in frontmatter["sources"]:
                    match = re.fullmatch(
                        r"repo:([^#]+)#sha256:([0-9a-f]{64})", source
                    )
                    self.assertIsNotNone(match, source)
                    source_path = ROOT / match.group(1)
                    self.assertTrue(source_path.is_file(), source)
                    self.assertEqual(
                        hashlib.sha256(source_path.read_bytes()).hexdigest(),
                        match.group(2),
                    )
                pages[(language, slug)] = (frontmatter, body)

        slugs = {slug for _, slug in pages}
        self.assertGreaterEqual(len(slugs), 20)
        self.assertEqual(
            {slug for language, slug in pages if language == "en"},
            {slug for language, slug in pages if language == "ko"},
        )
        for (language, slug), (frontmatter, _) in pages.items():
            with self.subTest(language=language, slug=slug):
                for link in frontmatter["links"]:
                    self.assertIn(link, slugs)


if __name__ == "__main__":
    unittest.main()
