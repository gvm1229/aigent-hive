"""Provider-neutral bilingual source Wiki conformance."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[2]
ACTION_SCHEMA = json.loads(
    (ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8")
)
PAGE_SCHEMA = json.loads(
    (ROOT / "schemas/source-wiki-page.schema.json").read_text(encoding="utf-8")
)
REVISION = f"git:{'0' * 40}"
LOCK_RELATIVE = ".agents/work/source-wiki/.index.lock"
INDEX_RELATIVE = ".agents/work/source-wiki/index.sqlite3"
V1_LOCK_MARKER = b"schema_version=1\n"
V2_LOCK_MARKER = b"schema_version=2\n"
FIRST_INDEX_CHANGED_PATHS = [LOCK_RELATIVE, INDEX_RELATIVE]


class SourceWikiConformance(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        configured = os.environ.get("HIVE_BIN")
        if configured:
            cls.hive = Path(configured).resolve()
        else:
            subprocess.run(
                ["cargo", "build", "--quiet", "--bin", "hive"],
                cwd=ROOT,
                check=True,
            )
            cls.hive = ROOT / "target/debug" / (
                "hive.exe" if os.name == "nt" else "hive"
            )

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="hive-source-wiki-")
        self.target = Path(self.temporary.name).resolve()
        (self.target / "hive-source.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "kind": "aigent-hive-source-workspace",
                    "consumer_setup_allowed": False,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        (self.target / "AGENTS.md").write_text(
            "# Source contract\n\nProvider-neutral source authority.\n",
            encoding="utf-8",
        )
        (self.target / "docs/facts/en").mkdir(parents=True)
        (self.target / "docs/facts/ko").mkdir(parents=True)
        self.write_pair(
            "index",
            links=["boundaries"],
            tags=["index", "source"],
            en_title="Source Wiki index",
            en_summary="Entry point for provider-neutral source knowledge.",
            en_body="# Source Wiki index\n\nProvider-neutral source knowledge entry point.\n",
            ko_title="Source Wiki 색인",
            ko_summary="공급자 중립 source 지식 진입점",
            ko_body="# Source Wiki 색인\n\n공급자 중립 source 지식 진입점.\n",
        )
        self.write_pair(
            "boundaries",
            links=["index"],
            tags=["architecture", "source"],
            en_title="Source boundaries",
            en_summary="Canonical ownership and runtime boundaries.",
            en_body="# Source boundaries\n\nCanonical Markdown owns durable knowledge.\n",
            ko_title="Source 경계",
            ko_summary="정본 소유권과 runtime 경계",
            ko_body="# Source 경계\n\nCanonical Markdown 기반 durable knowledge 소유권.\n",
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def source_locator(self) -> str:
        digest = hashlib.sha256((self.target / "AGENTS.md").read_bytes()).hexdigest()
        return f"repo:AGENTS.md#sha256:{digest}"

    def page(
        self,
        *,
        language: str,
        slug: str,
        links: list[str],
        tags: list[str],
        title: str,
        summary: str,
        body: str,
        revision: str = REVISION,
    ) -> str:
        other = "ko" if language == "en" else "en"
        frontmatter = yaml.safe_dump(
            {
                "schema_version": 1,
                "pair_id": slug,
                "topic_slug": slug,
                "language": language,
                "counterpart": f"../{other}/{slug}.md",
                "title": title,
                "summary": summary,
                "tags": tags,
                "aliases": [],
                "sources": [self.source_locator()],
                "links": links,
                "reviewed_revision": revision,
                "status": "active",
            },
            allow_unicode=True,
            sort_keys=False,
        )
        return f"---\n{frontmatter}---\n\n{body}"

    @staticmethod
    def write_page(path: Path, text: str) -> None:
        path.write_text(text, encoding="utf-8", newline="\n")

    def write_pair(
        self,
        slug: str,
        *,
        links: list[str],
        tags: list[str],
        en_title: str,
        en_summary: str,
        en_body: str,
        ko_title: str,
        ko_summary: str,
        ko_body: str,
        ko_revision: str = REVISION,
    ) -> None:
        self.write_page(
            self.target / f"docs/facts/en/{slug}.md",
            self.page(
                language="en",
                slug=slug,
                links=links,
                tags=tags,
                title=en_title,
                summary=en_summary,
                body=en_body,
            ),
        )
        self.write_page(
            self.target / f"docs/facts/ko/{slug}.md",
            self.page(
                language="ko",
                slug=slug,
                links=links,
                tags=tags,
                title=ko_title,
                summary=ko_summary,
                body=ko_body,
                revision=ko_revision,
            ),
        )

    def read_frontmatter(self, path: Path) -> dict[str, object]:
        return yaml.safe_load(path.read_text(encoding="utf-8").split("---")[1])

    def replace_frontmatter(
        self,
        path: Path,
        frontmatter: dict[str, object],
    ) -> None:
        original = path.read_text(encoding="utf-8")
        body = original.split("---", 2)[2]
        rendered = yaml.safe_dump(
            frontmatter,
            allow_unicode=True,
            sort_keys=False,
        )
        self.write_page(path, f"---\n{rendered}---{body}")

    def set_paired_field(self, slug: str, field: str, value: object) -> None:
        for language in ("en", "ko"):
            path = self.target / f"docs/facts/{language}/{slug}.md"
            frontmatter = self.read_frontmatter(path)
            frontmatter[field] = value
            self.replace_frontmatter(path, frontmatter)

    def invoke(
        self,
        *arguments: str,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [str(self.hive), *arguments, "--output", "json"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                f"invalid JSON: {error}\nstdout={process.stdout}\nstderr={process.stderr}"
            )
        Draft202012Validator(
            ACTION_SCHEMA,
            format_checker=FormatChecker(),
        ).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def lint(self) -> dict[str, object]:
        return self.invoke(
            "source-wiki",
            "lint",
            "--target",
            str(self.target),
        )[1]

    def index(self) -> dict[str, object]:
        return self.invoke(
            "source-wiki",
            "index",
            "--target",
            str(self.target),
        )[1]

    def query(self, language: str, text: str) -> dict[str, object]:
        return self.invoke(
            "source-wiki",
            "query",
            "--target",
            str(self.target),
            "--language",
            language,
            "--text",
            text,
        )[1]

    def test_index_and_bilingual_query_are_reproducible(self) -> None:
        for page in sorted((self.target / "docs/facts").rglob("*.md")):
            frontmatter = yaml.safe_load(page.read_text(encoding="utf-8").split("---")[1])
            Draft202012Validator(PAGE_SCHEMA).validate(frontmatter)
        first = self.index()
        self.assertEqual(first["status"], "success", first)
        self.assertEqual(first["action"], "RebuildSourceWikiIndex")
        self.assertEqual(first["code"], "hive.source-wiki-index-rebuilt")
        self.assertEqual(
            first["changed_paths"],
            FIRST_INDEX_CHANGED_PATHS,
        )
        english = self.query("en", "Provider-neutral")
        korean = self.query("ko", "공급자")
        self.assertEqual(english["status"], "success", english)
        self.assertEqual(korean["status"], "success", korean)
        self.assertEqual(english["action"], "QuerySourceWiki")
        self.assertEqual(english["code"], "hive.source-wiki-query-complete")
        self.assertGreater(english["data"]["count"], 0)
        self.assertGreater(korean["data"]["count"], 0)
        self.assertEqual(english["data"]["hits"][0]["topic_slug"], "index")
        self.assertEqual(english["data"]["hits"][0]["language"], "en")
        self.assertEqual(english["data"]["hits"][0]["path"], "docs/facts/en/index.md")
        self.assertEqual(korean["data"]["hits"][0]["topic_slug"], "index")
        self.assertEqual(korean["data"]["hits"][0]["language"], "ko")
        tagged = self.invoke(
            "source-wiki",
            "query",
            "--target",
            str(self.target),
            "--language",
            "en",
            "--tag",
            "architecture",
        )[1]
        self.assertEqual(
            [hit["topic_slug"] for hit in tagged["data"]["hits"]],
            ["boundaries"],
        )

        database = self.target / ".agents/work/source-wiki/index.sqlite3"
        database.unlink()
        second = self.index()
        self.assertEqual(second["changed_paths"], [INDEX_RELATIVE])
        self.assertEqual(
            first["data"]["logical_digest"],
            second["data"]["logical_digest"],
        )
        self.assertEqual(english["data"], self.query("en", "Provider-neutral")["data"])
        self.assertEqual(korean["data"], self.query("ko", "공급자")["data"])

    def test_schema_and_cli_agree_on_slug_hostile_corpus(self) -> None:
        pages = [
            self.target / "docs/facts/en/boundaries.md",
            self.target / "docs/facts/ko/boundaries.md",
        ]
        originals = {path: path.read_text(encoding="utf-8") for path in pages}
        corpus = (
            ("a", True),
            ("a-b", True),
            ("a" * 96, True),
            ("", False),
            ("-a", False),
            ("a-", False),
            ("a--b", False),
            ("A", False),
            ("a_b", False),
            ("a" * 97, False),
        )

        for pair_id, expected_valid in corpus:
            with self.subTest(pair_id=pair_id):
                for path, original in originals.items():
                    self.write_page(path, original)
                self.set_paired_field("boundaries", "pair_id", pair_id)
                schema_valid = all(
                    Draft202012Validator(PAGE_SCHEMA).is_valid(
                        self.read_frontmatter(path)
                    )
                    for path in pages
                )
                cli_valid = self.index()["status"] == "success"

                self.assertEqual(schema_valid, expected_valid)
                self.assertEqual(cli_valid, expected_valid)

    def test_schema_and_cli_agree_on_source_locator_hostile_corpus(self) -> None:
        pages = [
            self.target / "docs/facts/en/boundaries.md",
            self.target / "docs/facts/ko/boundaries.md",
        ]
        originals = {path: path.read_text(encoding="utf-8") for path in pages}
        source = self.target / "docs/source.md"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("safe repository source\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        invalid_paths = (
            "../AGENTS.md",
            "/AGENTS.md",
            "docs//source.md",
            "docs/./source.md",
            "docs/../AGENTS.md",
            "docs\\source.md",
            "docs/source.md#fragment",
            ".git/config",
            "docs/.hive/state",
            ".omx/state",
            ".omc/state",
            ".codex/config.toml",
            ".claude/settings.json",
            "omx_wiki/page.md",
        )
        corpus = (("docs/source.md", True),) + tuple(
            (path, False) for path in invalid_paths
        )

        for relative, expected_valid in corpus:
            with self.subTest(relative=relative):
                for path, original in originals.items():
                    self.write_page(path, original)
                locator = f"repo:{relative}#sha256:{digest}"
                self.set_paired_field("boundaries", "sources", [locator])
                schema_valid = all(
                    Draft202012Validator(PAGE_SCHEMA).is_valid(
                        self.read_frontmatter(path)
                    )
                    for path in pages
                )
                cli_valid = self.index()["status"] == "success"

                self.assertEqual(schema_valid, expected_valid)
                self.assertEqual(cli_valid, expected_valid)

    def test_schema_and_cli_agree_on_expressible_trim_rules(self) -> None:
        pages = [
            self.target / "docs/facts/en/boundaries.md",
            self.target / "docs/facts/ko/boundaries.md",
        ]
        originals = {path: path.read_text(encoding="utf-8") for path in pages}
        corpus = (
            ("title", "   ", False),
            ("summary", "\t ", False),
            ("aliases", [" padded"], False),
            ("aliases", ["padded "], False),
            ("title", " padded title ", True),
            ("summary", " padded summary ", True),
            ("aliases", ["two words"], True),
        )

        for field, value, expected_valid in corpus:
            with self.subTest(field=field, value=value):
                for path, original in originals.items():
                    self.write_page(path, original)
                self.set_paired_field("boundaries", field, value)
                schema_valid = all(
                    Draft202012Validator(PAGE_SCHEMA).is_valid(
                        self.read_frontmatter(path)
                    )
                    for path in pages
                )
                cli_valid = self.index()["status"] == "success"

                self.assertEqual(schema_valid, expected_valid)
                self.assertEqual(cli_valid, expected_valid)

    def test_schema_documents_cli_only_lexicographic_sortedness(self) -> None:
        pages = [
            self.target / "docs/facts/en/boundaries.md",
            self.target / "docs/facts/ko/boundaries.md",
        ]
        originals = {path: path.read_text(encoding="utf-8") for path in pages}
        source = self.target / "docs/source.md"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("second canonical source\n", encoding="utf-8")
        source_digest = hashlib.sha256(source.read_bytes()).hexdigest()
        unsorted_values = {
            "tags": ["source", "architecture"],
            "aliases": ["zeta", "alpha"],
            "sources": sorted(
                (
                    self.source_locator(),
                    f"repo:docs/source.md#sha256:{source_digest}",
                ),
                reverse=True,
            ),
            "links": ["index", "boundaries"],
        }

        for field, values in unsorted_values.items():
            with self.subTest(field=field):
                for path, original in originals.items():
                    self.write_page(path, original)
                self.set_paired_field("boundaries", field, values)

                self.assertTrue(
                    all(
                        Draft202012Validator(PAGE_SCHEMA).is_valid(
                            self.read_frontmatter(path)
                        )
                        for path in pages
                    )
                )
                self.assertNotEqual(self.index()["status"], "success")
                description = PAGE_SCHEMA["properties"][field]["description"]
                self.assertIn("JSON Schema", description)
                self.assertIn("Hive CLI", description)

    def test_lint_reports_missing_pair_and_pair_mismatch(self) -> None:
        korean = self.target / "docs/facts/ko/boundaries.md"
        original = korean.read_text(encoding="utf-8")
        korean.unlink()

        missing = self.lint()

        self.assertEqual(missing["status"], "verification-failed", missing)
        self.assertIn(
            "missing-pair",
            {issue["code"] for issue in missing["data"]["issues"]},
        )
        self.write_page(
            korean,
            original.replace(REVISION, f"git:{'1' * 40}"),
        )

        mismatch = self.lint()

        self.assertEqual(mismatch["status"], "verification-failed", mismatch)
        self.assertIn(
            "pair-mismatch",
            {issue["code"] for issue in mismatch["data"]["issues"]},
        )

    def test_lint_rejects_pair_id_reused_by_another_topic(self) -> None:
        self.set_paired_field("boundaries", "pair_id", "index")

        duplicate = self.lint()

        self.assertEqual(duplicate["status"], "verification-failed", duplicate)
        self.assertIn(
            "duplicate-pair-id",
            {issue["code"] for issue in duplicate["data"]["issues"]},
        )

    def test_lint_reports_broken_link_and_source_digest_mismatch(self) -> None:
        for language in ("en", "ko"):
            page = self.target / f"docs/facts/{language}/boundaries.md"
            self.write_page(
                page,
                page.read_text(encoding="utf-8").replace(
                    "links:\n- index",
                    "links:\n- missing",
                ),
            )

        broken = self.lint()

        self.assertEqual(broken["status"], "verification-failed", broken)
        self.assertIn(
            "broken-link",
            {issue["code"] for issue in broken["data"]["issues"]},
        )
        for language in ("en", "ko"):
            page = self.target / f"docs/facts/{language}/boundaries.md"
            text = page.read_text(encoding="utf-8")
            self.write_page(
                page,
                text.replace("links:\n- missing", "links:\n- index"),
            )
        (self.target / "AGENTS.md").write_text("# changed\n", encoding="utf-8")

        stale_source = self.lint()

        self.assertEqual(stale_source["status"], "verification-failed", stale_source)
        self.assertIn(
            "source-digest-mismatch",
            {issue["code"] for issue in stale_source["data"]["issues"]},
        )

    def test_stale_and_corrupt_index_block_query_until_rebuild(self) -> None:
        self.assertEqual(self.index()["status"], "success")
        page = self.target / "docs/facts/en/index.md"
        self.write_page(
            page,
            page.read_text(encoding="utf-8") + "\nCanonical change.\n",
        )

        stale = self.query("en", "Canonical")

        self.assertEqual(stale["status"], "verification-failed", stale)
        self.assertIn("source-wiki index", stale["next_action"])
        self.assertEqual(self.index()["status"], "success")
        database = self.target / ".agents/work/source-wiki/index.sqlite3"
        database.write_bytes(b"not sqlite")

        corrupt = self.query("en", "Canonical")

        self.assertNotEqual(corrupt["status"], "success", corrupt)
        self.assertIn("source-wiki index", corrupt["next_action"])
        self.assertEqual(self.index()["status"], "success")

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_symlink_and_secret_candidates_are_rejected_without_escape(self) -> None:
        external = self.target / "external.md"
        external.write_text("external sentinel\n", encoding="utf-8")
        page = self.target / "docs/facts/en/boundaries.md"
        page.unlink()
        page.symlink_to(external)

        symlinked_page = self.lint()

        self.assertNotEqual(symlinked_page["status"], "success")
        self.assertEqual(external.read_text(encoding="utf-8"), "external sentinel\n")
        page.unlink()
        self.write_pair(
            "boundaries",
            links=["index"],
            tags=["architecture", "source"],
            en_title="Source boundaries",
            en_summary="Canonical ownership and runtime boundaries.",
            en_body=(
                "# Source boundaries\n\n"
                'api_key = "correct-horse-battery-staple-123456"\n'
            ),
            ko_title="Source 경계",
            ko_summary="정본 소유권과 runtime 경계",
            ko_body="# Source 경계\n\n민감 정보 후보.\n",
        )

        secret = self.lint()

        self.assertNotEqual(secret["status"], "success")
        self.assertIn(
            "secret-candidate",
            {issue["code"] for issue in secret["data"]["issues"]},
        )

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_source_locator_rejects_ancestor_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory(prefix="hive-source-wiki-external-") as raw:
            external = Path(raw).resolve()
            source = external / "source.md"
            source.write_text("external sentinel\n", encoding="utf-8")
            (self.target / "docs").symlink_to(external, target_is_directory=True)
            digest = hashlib.sha256(source.read_bytes()).hexdigest()
            self.set_paired_field(
                "boundaries",
                "sources",
                [f"repo:docs/source.md#sha256:{digest}"],
            )

            result = self.lint()

            self.assertEqual(result["status"], "verification-failed", result)
            self.assertIn(
                "invalid-source",
                {issue["code"] for issue in result["data"]["issues"]},
            )
            self.assertEqual(source.read_text(encoding="utf-8"), "external sentinel\n")

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_wiki_root_rejects_ancestor_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory(prefix="hive-source-wiki-external-") as raw:
            external = Path(raw).resolve()
            external_wiki = external / "docs/facts"
            shutil.copytree(self.target / "docs/facts", external_wiki)
            expected = {
                path.relative_to(external_wiki).as_posix(): path.read_bytes()
                for path in external_wiki.rglob("*.md")
            }
            shutil.rmtree(self.target / "docs/facts")
            (self.target / "docs/facts").symlink_to(
                external_wiki,
                target_is_directory=True,
            )

            result = self.lint()

            self.assertNotEqual(result["status"], "success", result)
            actual = {
                path.relative_to(external_wiki).as_posix(): path.read_bytes()
                for path in external_wiki.rglob("*.md")
            }
            self.assertEqual(actual, expected)

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_index_refuses_symlink_and_creates_no_foreign_runtime_state(self) -> None:
        index_directory = self.target / ".agents/work/source-wiki"
        index_directory.mkdir(parents=True)
        external = self.target / "external.sqlite3"
        external.write_bytes(b"external sentinel")
        database = index_directory / "index.sqlite3"
        database.symlink_to(external)

        result = self.index()

        self.assertNotEqual(result["status"], "success")
        self.assertEqual(external.read_bytes(), b"external sentinel")
        for forbidden in (".hive", ".omx", ".omc", "omx_wiki"):
            self.assertFalse((self.target / forbidden).exists(), forbidden)

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_index_rejects_runtime_ancestor_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory(prefix="hive-source-wiki-external-") as raw:
            external = Path(raw).resolve()
            sentinel = external / "sentinel"
            sentinel.write_text("external sentinel\n", encoding="utf-8")
            (self.target / ".agents").symlink_to(external, target_is_directory=True)

            result = self.index()

            self.assertNotEqual(result["status"], "success", result)
            self.assertEqual(
                sentinel.read_text(encoding="utf-8"),
                "external sentinel\n",
            )
            self.assertEqual(
                sorted(path.name for path in external.iterdir()),
                ["sentinel"],
            )

    def assert_lock_marker_rejected_and_preserved(self, marker: bytes) -> None:
        lock = self.target / LOCK_RELATIVE
        lock.parent.mkdir(parents=True)
        lock.write_bytes(marker)

        result = self.index()

        self.assertEqual(result["status"], "verification-failed", result)
        self.assertEqual(result["exit_code"], 5, result)
        self.assertEqual(result["code"], "hive.source-wiki-verification-failed")
        self.assertEqual(lock.read_bytes(), marker)
        self.assertFalse((self.target / INDEX_RELATIVE).exists())

    def test_index_migrates_exact_v1_marker_to_v2(self) -> None:
        lock = self.target / LOCK_RELATIVE
        lock.parent.mkdir(parents=True)
        lock.write_bytes(V1_LOCK_MARKER)

        result = self.index()

        self.assertEqual(result["status"], "success", result)
        self.assertEqual(result["changed_paths"], FIRST_INDEX_CHANGED_PATHS)
        self.assertEqual(lock.read_bytes(), V2_LOCK_MARKER)

    def test_invalid_pair_leaves_v1_marker_uncommitted(self) -> None:
        lock = self.target / LOCK_RELATIVE
        lock.parent.mkdir(parents=True)
        lock.write_bytes(V1_LOCK_MARKER)
        (self.target / "docs/facts/ko/boundaries.md").unlink()

        result = self.index()

        self.assertEqual(result["status"], "verification-failed", result)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(lock.read_bytes(), V1_LOCK_MARKER)
        self.assertFalse((self.target / INDEX_RELATIVE).exists())

    def test_index_rejects_unknown_lock_marker_without_mutation(self) -> None:
        self.assert_lock_marker_rejected_and_preserved(b"unrecognized lock marker\n")

    def test_index_rejects_sqlite_lock_marker_without_mutation(self) -> None:
        self.assert_lock_marker_rejected_and_preserved(
            b"SQLite format 3\x00" + (b"\x00" * 84)
        )

    def test_concurrent_v1_migration_and_rebuild_both_succeed(self) -> None:
        lock = self.target / LOCK_RELATIVE
        lock.parent.mkdir(parents=True)
        lock.write_bytes(V1_LOCK_MARKER)
        command = [
            str(self.hive),
            "source-wiki",
            "index",
            "--target",
            str(self.target),
            "--output",
            "json",
        ]
        processes = [
            subprocess.Popen(
                command,
                cwd=ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            for _ in range(2)
        ]

        outputs = []
        try:
            for process in processes:
                outputs.append(process.communicate(timeout=15))
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                    process.communicate()

        results = []
        for process, (stdout, stderr) in zip(processes, outputs, strict=True):
            result = json.loads(stdout)
            Draft202012Validator(
                ACTION_SCHEMA,
                format_checker=FormatChecker(),
            ).validate(result)
            self.assertEqual(process.returncode, result["exit_code"], stderr)
            results.append(result)

        self.assertEqual(
            [result["status"] for result in results],
            ["success", "success"],
            results,
        )
        self.assertEqual(
            sorted(tuple(result["changed_paths"]) for result in results),
            sorted(
                (
                    tuple(FIRST_INDEX_CHANGED_PATHS),
                    (INDEX_RELATIVE,),
                )
            ),
        )
        self.assertEqual(lock.read_bytes(), V2_LOCK_MARKER)

    def test_index_lock_is_released_when_prior_cli_process_exits(self) -> None:
        first = self.index()

        second = self.index()

        self.assertEqual(first["status"], "success", first)
        self.assertEqual(first["changed_paths"], FIRST_INDEX_CHANGED_PATHS)
        self.assertEqual(second["status"], "success", second)
        self.assertEqual(second["changed_paths"], [INDEX_RELATIVE])

    def test_index_recovers_valid_orphan_claim_when_live_index_is_missing(self) -> None:
        first = self.index()
        self.assertEqual(first["status"], "success", first)
        index = self.target / INDEX_RELATIVE
        claim = index.parent / ".index.sqlite3.claim-999999-0"
        index.replace(claim)

        recovered = self.index()

        self.assertEqual(recovered["status"], "success", recovered)
        self.assertEqual(recovered["changed_paths"], [INDEX_RELATIVE])
        self.assertTrue(index.is_file())
        self.assertFalse(claim.exists())

    def test_index_preserves_foreign_claim_name(self) -> None:
        first = self.index()
        self.assertEqual(first["status"], "success", first)
        foreign = (
            self.target
            / ".agents/work/source-wiki/.index.sqlite3.claim-foreign"
        )
        foreign.write_bytes(b"foreign sentinel")

        rebuilt = self.index()

        self.assertEqual(rebuilt["status"], "success", rebuilt)
        self.assertEqual(foreign.read_bytes(), b"foreign sentinel")

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_index_rejects_owned_claim_symlink_and_preserves_live_index(self) -> None:
        first = self.index()
        self.assertEqual(first["status"], "success", first)
        index = self.target / INDEX_RELATIVE
        original_index = index.read_bytes()
        external = self.target / "external-claim"
        external.write_bytes(b"external sentinel")
        claim = index.parent / ".index.sqlite3.claim-999999-1"
        claim.symlink_to(external)

        result = self.index()

        self.assertEqual(result["status"], "verification-failed", result)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(index.read_bytes(), original_index)
        self.assertEqual(external.read_bytes(), b"external sentinel")

    @unittest.skipIf(os.name == "nt", "symlink creation may require Windows privileges")
    def test_index_rejects_lock_symlink_without_touching_external_bytes(self) -> None:
        lock = self.target / LOCK_RELATIVE
        lock.parent.mkdir(parents=True)
        external = self.target / "external.lock"
        external.write_bytes(b"external sentinel")
        lock.symlink_to(external)

        result = self.index()

        self.assertNotEqual(result["status"], "success", result)
        self.assertEqual(external.read_bytes(), b"external sentinel")
        self.assertFalse((self.target / INDEX_RELATIVE).exists())

    def test_clean_source_copy_rebuilds_without_runtime_state(self) -> None:
        canonical_before = {
            path.relative_to(self.target).as_posix(): hashlib.sha256(
                path.read_bytes()
            ).hexdigest()
            for path in sorted((self.target / "docs/facts").rglob("*.md"))
        }
        first = self.index()
        with tempfile.TemporaryDirectory(prefix="hive-source-wiki-clean-") as raw:
            clean = Path(raw).resolve()
            shutil.copy2(self.target / "hive-source.json", clean / "hive-source.json")
            shutil.copy2(self.target / "AGENTS.md", clean / "AGENTS.md")
            shutil.copytree(self.target / "docs/facts", clean / "docs/facts")

            rebuilt = self.invoke(
                "source-wiki",
                "index",
                "--target",
                str(clean),
            )[1]
            queried = self.invoke(
                "source-wiki",
                "query",
                "--target",
                str(clean),
                "--language",
                "en",
                "--text",
                "Provider-neutral",
            )[1]

            self.assertEqual(rebuilt["status"], "success", rebuilt)
            self.assertEqual(
                first["data"]["logical_digest"],
                rebuilt["data"]["logical_digest"],
            )
            self.assertEqual(queried["status"], "success", queried)
            for forbidden in (".hive", ".omx", ".omc", "omx_wiki"):
                self.assertFalse((clean / forbidden).exists(), forbidden)
        canonical_after = {
            path.relative_to(self.target).as_posix(): hashlib.sha256(
                path.read_bytes()
            ).hexdigest()
            for path in sorted((self.target / "docs/facts").rglob("*.md"))
        }
        self.assertEqual(canonical_before, canonical_after)

    def test_source_identity_and_query_arguments_are_strict(self) -> None:
        marker = self.target / "hive-source.json"
        original = marker.read_text(encoding="utf-8")
        marker.unlink()
        missing = self.index()
        self.assertEqual(missing["status"], "verification-failed", missing)
        marker.write_text('{"schema_version":1,"kind":"wrong"}\n', encoding="utf-8")
        invalid = self.index()
        self.assertEqual(invalid["status"], "verification-failed", invalid)
        marker.write_text(original, encoding="utf-8")

        invalid_arguments = (
            ("--language", "fr", "--text", "source"),
            ("--language", "en"),
            ("--language", "en", "--text", "source", "--limit", "0"),
        )
        for arguments in invalid_arguments:
            with self.subTest(arguments=arguments):
                _, result = self.invoke(
                    "source-wiki",
                    "query",
                    "--target",
                    str(self.target),
                    *arguments,
                )
                self.assertEqual(result["status"], "error", result)
                self.assertEqual(result["exit_code"], 2)
                self.assertEqual(result["code"], "hive.source-wiki-invalid-input")

    def test_omx_wiki_exclusion_reason_is_durable(self) -> None:
        decision = (
            ROOT / "docs/decisions/ADR-0011-source-wiki-independence.md"
        ).read_text(encoding="utf-8")
        for requirement in (
            "제외 판단은 OMX Wiki의 품질·유용성과 무관",
            "고정 저장 경로 `omx_wiki/`",
            "`.omx-config.json` 기반 lifecycle·auto-capture 계약",
            "OMX/OMC retirement 시 source knowledge migration 0건",
        ):
            self.assertIn(requirement, decision)

    def test_material_source_task_autocapture_contract_is_durable(self) -> None:
        source_manifest = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        documentation_directive = (
            ROOT / ".agents/directives/04-documentation-state.md"
        ).read_text(encoding="utf-8")
        source_skill = (
            ROOT / ".agents/skills/hive-source-wiki/SKILL.md"
        ).read_text(encoding="utf-8")
        decision = (
            ROOT / "docs/decisions/ADR-0011-source-wiki-independence.md"
        ).read_text(encoding="utf-8")
        for surface in (
            source_manifest,
            documentation_directive,
            source_skill,
            decision,
        ):
            self.assertIn("agent-reviewed", surface.lower())
            self.assertIn("task fact", surface.lower().replace("-", " "))
            self.assertIn("raw transcript", surface.lower())
        self.assertIn("originating request", documentation_directive)
        self.assertIn("current authorized task", source_skill)
        self.assertIn("hook", decision.lower())

    def test_hive_marketing_deck_has_bilingual_resume_memory(self) -> None:
        english = (ROOT / "docs/facts/en/marketing-deck-record.md").read_text(
            encoding="utf-8"
        )
        korean = (ROOT / "docs/facts/ko/marketing-deck-record.md").read_text(
            encoding="utf-8"
        )
        task_record = (
            ROOT / "docs/state/artifacts/aigent-hive-marketing-deck.md"
        ).read_text(encoding="utf-8")
        for surface in (english, korean, task_record):
            self.assertIn("LumaDeck", surface)
            self.assertIn("aigent-hive-overview", surface)
            self.assertIn("What our Hive harness is about", surface)
            self.assertIn("optimization strategy", surface)
        self.assertIn("Initial request", task_record)
        self.assertIn("8", english)
        self.assertIn("8", korean)


if __name__ == "__main__":
    unittest.main()
