"""Phase 2 canonical Markdown knowledge and disposable SQLite conformance."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
import unittest
from contextlib import closing
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker
from tests.conformance.phase1_support import (
    FIXTURE_ROOT as PHASE1_FIXTURES,
    write_operational_user_setup,
)


ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "tests/fixtures/phase2"
ACTION_SCHEMA = json.loads(
    (ROOT / "schemas/action-result.schema.json").read_text(encoding="utf-8")
)
QUERY_SCHEMA = json.loads(
    (ROOT / "schemas/knowledge-query-result.schema.json").read_text(encoding="utf-8")
)
LINT_SCHEMA = json.loads(
    (ROOT / "schemas/knowledge-lint-result.schema.json").read_text(encoding="utf-8")
)


def snapshot_tree(root: Path) -> dict[str, tuple[str, bytes | str]]:
    snapshot: dict[str, tuple[str, bytes | str]] = {}
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            snapshot[relative] = ("symlink", os.readlink(path))
        elif path.is_file():
            snapshot[relative] = ("file", path.read_bytes())
        elif path.is_dir():
            snapshot[relative] = ("directory", "")
    return snapshot


class Phase2WikiConformance(unittest.TestCase):
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
            cls.hive = ROOT / "target/debug" / ("hive.exe" if os.name == "nt" else "hive")

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="aigent-hive-phase2-")
        self.target = Path(self.temporary.name).resolve()
        self.user_root = self.target.parent / f"{self.target.name}-user-root"
        write_operational_user_setup(self.user_root)
        knowledge = self.target / ".hive/knowledge"
        shutil.copytree(
            ROOT / "harness/template/.hive/knowledge",
            knowledge,
        )
        setup = subprocess.run(
            [
                str(self.hive),
                "setup",
                "--target",
                str(self.target),
                "--answers",
                str(PHASE1_FIXTURES / "answers-base.yml"),
                "--capabilities",
                str(PHASE1_FIXTURES / "capabilities-codex-omx.json"),
                "--user-root",
                str(self.user_root),
                "--apply",
                "--output",
                "json",
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(setup.returncode, 0, setup.stdout + setup.stderr)

    def tearDown(self) -> None:
        shutil.rmtree(self.user_root, ignore_errors=True)
        self.temporary.cleanup()

    def invoke(
        self,
        *arguments: str,
        environment: dict[str, str] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [str(self.hive), *arguments, "--output", "json"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
            env={**os.environ, **environment} if environment is not None else None,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"invalid JSON: {error}\nstdout={process.stdout}\nstderr={process.stderr}")
        Draft202012Validator(ACTION_SCHEMA, format_checker=FormatChecker()).validate(result)
        self.assertEqual(process.returncode, result["exit_code"], process.stderr)
        return process, result

    def ingest(self, name: str) -> dict[str, object]:
        _, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(FIXTURES / f"raw/{name}.md"),
            "--wiki",
            str(FIXTURES / f"wiki/{name}.md"),
        )
        self.assertEqual(result["status"], "success", result)
        return result

    def query(self, text: str) -> dict[str, object]:
        _, result = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.target),
            "--text",
            text,
        )
        return result

    def test_ingest_query_and_full_projection(self) -> None:
        self.ingest("alpha")
        self.ingest("beta")
        result = self.query("deterministic knowledge")
        self.assertEqual(result["status"], "success")
        Draft202012Validator(QUERY_SCHEMA).validate(result["data"])
        self.assertEqual([hit["id"] for hit in result["data"]["hits"]], ["alpha"])

        tagged = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.target),
            "--tag",
            "integration",
        )[1]
        self.assertEqual([hit["id"] for hit in tagged["data"]["hits"]], ["beta"])
        self.assertEqual(
            [hit["id"] for hit in self.query("serial-curation")["data"]["hits"]],
            ["beta"],
        )

        with closing(
            sqlite3.connect(self.target / ".hive/index/hive.sqlite3")
        ) as database:
            self.assertEqual(database.execute("select count(*) from pages").fetchone()[0], 2)
            self.assertEqual(database.execute("select count(*) from tags").fetchone()[0], 4)
            self.assertEqual(
                database.execute("select count(*) from aliases").fetchone()[0], 2
            )
            self.assertEqual(database.execute("select count(*) from links").fetchone()[0], 2)
            self.assertEqual(
                database.execute("select count(*) from sources").fetchone()[0], 2
            )
            self.assertEqual(
                database.execute("select count(*) from raw_objects").fetchone()[0], 2
            )

    def test_deleted_database_rebuild_has_same_digest_and_query(self) -> None:
        self.ingest("alpha")
        self.ingest("beta")
        before_query = self.query("canonical Markdown")["data"]
        database = self.target / ".hive/index/hive.sqlite3"
        database.unlink()
        _, rebuilt = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(self.target),
        )
        self.assertEqual(rebuilt["status"], "success")
        after_query = self.query("canonical Markdown")["data"]
        self.assertEqual(before_query, after_query)
        second = self.invoke("index", "rebuild", "--target", str(self.target))[1]
        self.assertEqual(
            rebuilt["data"]["logical_digest"],
            second["data"]["logical_digest"],
        )

    def test_direct_markdown_change_is_stale_until_rebuild(self) -> None:
        self.ingest("alpha")
        page = self.target / ".hive/knowledge/Wiki/alpha.md"
        page.write_text(page.read_text(encoding="utf-8") + "\nChanged.\n", encoding="utf-8")
        result = self.query("Changed")
        self.assertEqual(result["status"], "verification-failed")
        self.assertIn("stale", result["message"])
        lint_result = self.invoke(
            "knowledge",
            "lint",
            "--target",
            str(self.target),
        )[1]
        codes = {issue["code"] for issue in lint_result["data"]["issues"]}
        self.assertIn("stale-index", codes)
        stale = self.target / ".hive/index/.stale"
        stale.write_text('{"schema_version":1,"stale":true}\n', encoding="utf-8")
        _, rebuilt = self.invoke("index", "rebuild", "--target", str(self.target))
        self.assertEqual(
            rebuilt["changed_paths"],
            [".hive/index/.stale", ".hive/index/hive.sqlite3"],
        )
        self.assertFalse(stale.exists())
        self.assertEqual(self.query("Changed")["status"], "success")

    def test_lint_detects_broken_link_orphan_missing_citation_and_contradiction(self) -> None:
        self.ingest("alpha")
        orphan = self.target / ".hive/knowledge/Wiki/orphan.md"
        orphan.write_text(
            """---
schema_version: 1
id: orphan
kind: concept
summary: Orphan without citation
tags: []
aliases: []
sources: []
links: []
contradictions: []
status: contradicted
created_at: 2026-07-24T00:00:00Z
updated_at: 2026-07-24T00:00:00Z
---

# Orphan

Uncited contradictory claim.
""",
            encoding="utf-8",
        )
        self.invoke("index", "rebuild", "--target", str(self.target))
        _, result = self.invoke(
            "knowledge",
            "lint",
            "--target",
            str(self.target),
        )
        self.assertEqual(result["status"], "verification-failed")
        Draft202012Validator(LINT_SCHEMA).validate(result["data"])
        codes = {issue["code"] for issue in result["data"]["issues"]}
        self.assertTrue(
            {"broken-link", "orphan", "missing-citation", "missing-contradiction"}
            <= codes
        )

    def test_deprecated_page_is_rejected_without_active_write(self) -> None:
        _, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(FIXTURES / "raw/alpha.md"),
            "--wiki",
            str(FIXTURES / "wiki/deprecated.md"),
        )
        self.assertEqual(result["status"], "error")
        self.assertFalse((self.target / ".hive/knowledge/Wiki/deprecated.md").exists())
        raw_files = list((self.target / ".hive/knowledge/Raw").rglob("*.md"))
        self.assertEqual(raw_files, [self.target / ".hive/knowledge/Raw/README.md"])

    def test_invalid_raw_locator_is_rejected_before_any_write(self) -> None:
        base = (FIXTURES / "wiki/alpha.md").read_text(encoding="utf-8")
        invalid_locators = (
            f"raw:.hive/knowledge/Raw/../../outside.txt#sha256:{'0' * 64}",
            (
                f"raw:.hive/knowledge/Raw/source/{'0' * 64}.txt"
                f"#sha256:{'1' * 64}"
            ),
        )
        for index, locator in enumerate(invalid_locators):
            with self.subTest(locator=locator):
                wiki = self.target / f"invalid-locator-{index}.md"
                wiki.write_text(
                    base.replace("sources: [raw:self]", f"sources: [{locator}]"),
                    encoding="utf-8",
                )
                before = snapshot_tree(self.target)
                _, result = self.invoke(
                    "knowledge",
                    "ingest",
                    "--target",
                    str(self.target),
                    "--source",
                    str(FIXTURES / "raw/alpha.md"),
                    "--wiki",
                    str(wiki),
                )
                self.assertEqual(result["status"], "error")
                self.assertIn("immutable Raw revision", result["message"])
                self.assertEqual(snapshot_tree(self.target), before)

    def test_delete_records_metadata_only_and_removes_search_result(self) -> None:
        source = self.target / "gamma.txt"
        source.write_text("Deleted body sentinel SECRET-DELETED-PROSE", encoding="utf-8")
        draft = self.target / "gamma.md"
        draft.write_text(
            """---
schema_version: 1
id: gamma
kind: source-summary
summary: Temporary source
tags: [temporary]
aliases: []
sources: [raw:self]
links: []
contradictions: []
status: active
created_at: 2026-07-24T00:00:00Z
updated_at: 2026-07-24T00:00:00Z
---

# Gamma

Deleted body sentinel SECRET-DELETED-PROSE.
""",
            encoding="utf-8",
        )
        self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(source),
            "--wiki",
            str(draft),
        )
        _, deleted = self.invoke(
            "knowledge",
            "delete",
            "--target",
            str(self.target),
            "--page-id",
            "gamma",
            "--reason",
            "obsolete",
            "--replacement",
            "wiki:alpha",
            "--timestamp",
            "2026-07-24T01:00:00Z",
        )
        self.assertEqual(deleted["status"], "success")
        self.assertEqual(self.query("SECRET-DELETED-PROSE")["data"]["hits"], [])
        suppression_path = self.target / ".hive/knowledge/suppression.yml"
        suppression_text = suppression_path.read_text(encoding="utf-8")
        self.assertNotIn("SECRET-DELETED-PROSE", suppression_text)
        self.assertFalse(any((self.target / ".hive/knowledge/Raw/gamma").glob("*")))
        ledger = yaml.safe_load(suppression_text)
        self.assertEqual(len(ledger["entries"]), 2)
        for entry in ledger["entries"]:
            self.assertEqual(
                set(entry),
                {"fingerprint", "source_locator", "reason", "replacement", "timestamp"},
            )

    def test_suppressed_raw_fingerprint_cannot_be_reingested(self) -> None:
        source = FIXTURES / "raw/alpha.md"
        fingerprint = f"sha256:{hashlib.sha256(source.read_bytes()).hexdigest()}"
        arguments = (
            "knowledge",
            "suppress",
            "--target",
            str(self.target),
            "--fingerprint",
            fingerprint,
            "--source-locator",
            "external:obsolete-alpha",
            "--reason",
            "obsolete",
            "--timestamp",
            "2026-07-24T01:00:00Z",
        )
        _, first = self.invoke(*arguments)
        self.assertEqual(first["status"], "success")
        suppression = self.target / ".hive/knowledge/suppression.yml"
        index = self.target / ".hive/index/hive.sqlite3"
        before = (suppression.read_bytes(), index.read_bytes())
        _, repeated = self.invoke(*arguments)
        self.assertEqual(repeated["status"], "success")
        self.assertEqual(repeated["changed_paths"], [])
        self.assertEqual((suppression.read_bytes(), index.read_bytes()), before)

        _, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(source),
            "--wiki",
            str(FIXTURES / "wiki/alpha.md"),
        )
        self.assertEqual(result["status"], "conflict")
        self.assertIn("suppressed", result["message"])

    def test_suppress_rejects_fingerprint_or_locator_still_active(self) -> None:
        ingested = self.ingest("alpha")
        raw_locator = ingested["data"]["source_locator"]
        raw_fingerprint = raw_locator.rsplit("#", 1)[1]
        wiki_path = self.target / ".hive/knowledge/Wiki/alpha.md"
        wiki_fingerprint = f"sha256:{hashlib.sha256(wiki_path.read_bytes()).hexdigest()}"
        before = snapshot_tree(self.target)

        for fingerprint, locator in (
            (raw_fingerprint, raw_locator),
            (wiki_fingerprint, "wiki:alpha"),
        ):
            with self.subTest(locator=locator):
                process, result = self.invoke(
                    "knowledge",
                    "suppress",
                    "--target",
                    str(self.target),
                    "--fingerprint",
                    fingerprint,
                    "--source-locator",
                    locator,
                    "--reason",
                    "obsolete",
                    "--timestamp",
                    "2026-07-24T01:00:00Z",
                )
                self.assertEqual(process.returncode, 3)
                self.assertEqual(result["status"], "conflict")
                self.assertEqual(result["changed_paths"], [])
                self.assertIn("active", result["message"])
                self.assertEqual(snapshot_tree(self.target), before)

    def test_suppression_reason_rejects_deleted_prose_without_mutation(self) -> None:
        self.ingest("alpha")
        before = snapshot_tree(self.target)
        prose = "alpha-deleted-body-sentence-must-never-persist"
        _, deleted = self.invoke(
            "knowledge",
            "delete",
            "--target",
            str(self.target),
            "--page-id",
            "alpha",
            "--reason",
            prose,
            "--timestamp",
            "2026-07-24T01:00:00Z",
        )
        self.assertEqual(deleted["status"], "error")
        self.assertIn("stable reason code", deleted["message"])
        self.assertEqual(snapshot_tree(self.target), before)

        _, suppressed = self.invoke(
            "knowledge",
            "suppress",
            "--target",
            str(self.target),
            "--fingerprint",
            f"sha256:{'0' * 64}",
            "--source-locator",
            "external:deleted",
            "--reason",
            prose,
            "--timestamp",
            "2026-07-24T01:00:00Z",
        )
        self.assertEqual(suppressed["status"], "error")
        self.assertIn("stable reason code", suppressed["message"])
        self.assertEqual(snapshot_tree(self.target), before)

        for option, locator in (
            ("--source-locator", "deleted body sentence"),
            ("--replacement", "deleted-body-sentence"),
        ):
            arguments = [
                "knowledge",
                "suppress",
                "--target",
                str(self.target),
                "--fingerprint",
                f"sha256:{'0' * 64}",
                "--source-locator",
                "external:deleted",
                "--reason",
                "obsolete",
                "--timestamp",
                "2026-07-24T01:00:00Z",
            ]
            if option == "--source-locator":
                arguments[arguments.index("--source-locator") + 1] = locator
            else:
                arguments.extend([option, locator])
            _, locator_result = self.invoke(*arguments)
            self.assertEqual(locator_result["status"], "error")
            self.assertIn("locator", locator_result["message"])
            self.assertEqual(snapshot_tree(self.target), before)

    def test_likely_credentials_are_rejected_before_raw_write(self) -> None:
        credentials = {
            "provider-key": 'OPENAI_API_KEY = "sk-live-obvious-secret"\n',
            "embedded-example": 'OPENAI_API_KEY = "example-live-secret-123"\n',
            "password": "password: correct-horse-battery-staple\n",
            "short-api-key": "api_key=qwerty\n",
            "json-password": '{"password":"correct-horse-battery-staple"}\n',
            "netrc-password": (
                "machine example.invalid login build password secret-value\n"
            ),
            "gitlab-token": "token = glpat-obvious-test-token\n",
            "npm-token": "//registry.npmjs.org/:_authToken=npm_obvious_test_token\n",
            "basic-auth": "Authorization: Basic dXNlcjpwYXNzd29yZA==\n",
            "docker-auth": '{"auth":"dXNlcjpwYXNzd29yZA=="}\n',
            "npm-basic-auth": "_auth=dXNlcjpwYXNzd29yZA==\n",
            "decoy-before-assignment": (
                "api_key_example=redacted "
                "api_key=correct-horse-battery-staple-123456\n"
            ),
            "ec-private-key": "-----BEGIN EC PRIVATE KEY-----\nnot-a-real-key\n",
            "pgp-private-key": (
                "-----BEGIN PGP PRIVATE KEY BLOCK-----\nnot-a-real-key\n"
            ),
            "putty-private-key": (
                "PuTTY-User-Key-File-2: ssh-rsa\n"
                "Private-Lines: 1\n"
                "not-a-real-key\n"
                "Private-MAC: deadbeef\n"
            ),
        }
        for source_name, credential in credentials.items():
            with self.subTest(source_name=source_name):
                source = self.target / f"{source_name}.txt"
                source.write_text(credential, encoding="utf-8")
                _, result = self.invoke(
                    "knowledge",
                    "ingest",
                    "--target",
                    str(self.target),
                    "--source",
                    str(source),
                    "--wiki",
                    str(FIXTURES / "wiki/alpha.md"),
                )
                self.assertEqual(result["status"], "error")
                self.assertIn("credential", result["message"])
                self.assertFalse(
                    (self.target / f".hive/knowledge/Raw/{source_name}").exists()
                )

    def test_oversized_raw_source_is_bounded_and_rejected_before_write(self) -> None:
        source = self.target / "oversized.bin"
        oversized = b"x" * (5 * 1024 * 1024 + 1)
        source.write_bytes(oversized)
        _, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(source),
            "--wiki",
            str(FIXTURES / "wiki/alpha.md"),
        )
        self.assertEqual(result["status"], "error")
        self.assertIn("5 MiB", result["message"])
        self.assertFalse((self.target / ".hive/knowledge/Raw/oversized").exists())

        digest = hashlib.sha256(oversized).hexdigest()
        manual = self.target / f".hive/knowledge/Raw/manual/{digest}.bin"
        manual.parent.mkdir()
        manual.write_bytes(oversized)
        _, rebuild = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(self.target),
        )
        self.assertEqual(rebuild["status"], "verification-failed")
        self.assertIn("5 MiB", rebuild["message"])
        self.assertFalse((self.target / ".hive/index/hive.sqlite3").exists())

    def test_credentials_are_rejected_from_wiki_and_manual_raw(self) -> None:
        wiki = self.target / "credential-wiki.md"
        wiki.write_text(
            (FIXTURES / "wiki/alpha.md").read_text(encoding="utf-8")
            + '\nOPENAI_API_KEY = "synthetic-sensitive-value"\n',
            encoding="utf-8",
        )
        _, wiki_result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(FIXTURES / "raw/alpha.md"),
            "--wiki",
            str(wiki),
        )
        self.assertEqual(wiki_result["status"], "error")
        self.assertIn("credential", wiki_result["message"])
        self.assertFalse((self.target / ".hive/knowledge/Raw/alpha").exists())

        manual_bytes = b'password = "synthetic-sensitive-value"\n'
        digest = hashlib.sha256(manual_bytes).hexdigest()
        manual = self.target / f".hive/knowledge/Raw/manual/{digest}.txt"
        manual.parent.mkdir()
        manual.write_bytes(manual_bytes)
        _, rebuild_result = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(self.target),
        )
        self.assertEqual(rebuild_result["status"], "verification-failed")
        self.assertIn("sensitive material", rebuild_result["message"])
        self.assertFalse((self.target / ".hive/index/hive.sqlite3").exists())

    def test_same_page_ingest_accumulates_every_raw_source(self) -> None:
        first = self.ingest("alpha")
        source = self.target / "alpha.md"
        source.write_text("A second Alpha source revision.", encoding="utf-8")
        _, second = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(source),
            "--wiki",
            str(FIXTURES / "wiki/alpha.md"),
        )
        self.assertEqual(second["status"], "success", second)
        page = self.target / ".hive/knowledge/Wiki/alpha.md"
        frontmatter = yaml.safe_load(page.read_text(encoding="utf-8").split("---", 2)[1])
        self.assertEqual(
            frontmatter["sources"],
            sorted(
                [
                    first["data"]["source_locator"],
                    second["data"]["source_locator"],
                ]
            ),
        )

    def test_parallel_same_page_ingest_has_no_lost_source(self) -> None:
        sources = []
        for name, body in (
            ("alpha-one", "First concurrent Alpha source."),
            ("alpha-two", "Second concurrent Alpha source."),
        ):
            source = self.target / f"{name}.md"
            source.write_text(body, encoding="utf-8")
            sources.append(source)

        def ingest_source(source: Path) -> dict[str, object]:
            return self.invoke(
                "knowledge",
                "ingest",
                "--target",
                str(self.target),
                "--source",
                str(source),
                "--wiki",
                str(FIXTURES / "wiki/alpha.md"),
            )[1]

        with ThreadPoolExecutor(max_workers=2) as executor:
            results = list(executor.map(ingest_source, sources))
        self.assertEqual(
            [result["status"] for result in results],
            ["success", "success"],
        )
        page = self.target / ".hive/knowledge/Wiki/alpha.md"
        frontmatter = yaml.safe_load(page.read_text(encoding="utf-8").split("---", 2)[1])
        self.assertEqual(
            frontmatter["sources"],
            sorted(result["data"]["source_locator"] for result in results),
        )

    def test_ingest_failure_after_canonical_writes_rolls_back_exact_tree(self) -> None:
        (self.target / ".hive/index").mkdir()
        before = snapshot_tree(self.target)
        _, result = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(FIXTURES / "raw/alpha.md"),
            "--wiki",
            str(FIXTURES / "wiki/alpha.md"),
            environment={"HIVE_WIKI_TEST_FAIL_AFTER_CANONICAL_WRITES": "1"},
        )
        self.assertEqual(result["status"], "error")
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(self.target), before)

    def test_parallel_extraction_serial_integration_has_no_lost_update(self) -> None:
        def command(name: str) -> dict[str, object]:
            return self.invoke(
                "knowledge",
                "ingest",
                "--target",
                str(self.target),
                "--source",
                str(FIXTURES / f"raw/{name}.md"),
                "--wiki",
                str(FIXTURES / f"wiki/{name}.md"),
            )[1]

        with ThreadPoolExecutor(max_workers=2) as executor:
            results = list(executor.map(command, ("alpha", "beta")))
        self.assertEqual([result["status"] for result in results], ["success", "success"])
        self.assertEqual(
            [hit["id"] for hit in self.query("knowledge")["data"]["hits"]],
            ["alpha", "beta"],
        )
        with closing(
            sqlite3.connect(self.target / ".hive/index/hive.sqlite3")
        ) as database:
            self.assertEqual(database.execute("select count(*) from pages").fetchone()[0], 2)
            self.assertEqual(database.execute("select count(*) from links").fetchone()[0], 2)

    def test_raw_revision_is_content_addressed_and_immutable(self) -> None:
        first = self.ingest("alpha")
        source_copy = self.target / "alpha.md"
        source_copy.write_text("A changed Alpha revision.", encoding="utf-8")
        _, second = self.invoke(
            "knowledge",
            "ingest",
            "--target",
            str(self.target),
            "--source",
            str(source_copy),
            "--wiki",
            str(FIXTURES / "wiki/alpha.md"),
        )
        self.assertEqual(second["status"], "success")
        self.assertNotEqual(
            first["data"]["source_locator"],
            second["data"]["source_locator"],
        )
        revisions = [
            path
            for path in (self.target / ".hive/knowledge/Raw/alpha").iterdir()
            if path.is_file()
        ]
        self.assertEqual(len(revisions), 2)

    def test_tampered_raw_revision_cannot_be_reindexed_under_old_locator(self) -> None:
        ingested = self.ingest("alpha")
        locator = ingested["data"]["source_locator"]
        raw_relative = locator.removeprefix("raw:").split("#", 1)[0]
        raw_path = self.target / raw_relative
        index_path = self.target / ".hive/index/hive.sqlite3"
        index_before = index_path.read_bytes()
        raw_path.write_text("tampered bytes", encoding="utf-8")

        process, result = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(self.target),
        )
        self.assertEqual(process.returncode, 5)
        self.assertEqual(result["status"], "verification-failed")
        self.assertIn("content digest", result["message"])
        self.assertEqual(index_path.read_bytes(), index_before)

    def test_index_symlink_is_never_followed_by_query_or_rebuild(self) -> None:
        self.ingest("alpha")
        index_path = self.target / ".hive/index/hive.sqlite3"
        outside = self.target.parent / f"{self.target.name}-outside.sqlite3"
        self.addCleanup(outside.unlink, missing_ok=True)
        index_path.replace(outside)
        try:
            index_path.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symlink creation is unavailable: {error}")
        outside_before = outside.read_bytes()

        query_process, query_result = self.invoke(
            "knowledge",
            "query",
            "--target",
            str(self.target),
            "--text",
            "knowledge",
        )
        self.assertNotEqual(query_process.returncode, 0)
        self.assertIn(query_result["status"], {"conflict", "verification-failed"})
        self.assertEqual(outside.read_bytes(), outside_before)

        rebuild_process, rebuild_result = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(self.target),
        )
        self.assertNotEqual(rebuild_process.returncode, 0)
        self.assertIn(rebuild_result["status"], {"conflict", "verification-failed"})
        self.assertTrue(index_path.is_symlink())
        self.assertEqual(outside.read_bytes(), outside_before)

    def test_stale_marker_symlink_is_never_followed_by_query_or_rebuild(self) -> None:
        self.ingest("alpha")
        stale_path = self.target / ".hive/index/.stale"
        outside = self.target.parent / f"{self.target.name}-outside-stale"
        self.addCleanup(outside.unlink, missing_ok=True)
        outside.write_bytes(b"user-owned-stale-target\n")
        try:
            stale_path.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symlink creation is unavailable: {error}")
        outside_before = outside.read_bytes()

        for arguments in (
            (
                "knowledge",
                "query",
                "--target",
                str(self.target),
                "--text",
                "knowledge",
            ),
            (
                "index",
                "rebuild",
                "--target",
                str(self.target),
            ),
        ):
            with self.subTest(action=arguments[:2]):
                process, result = self.invoke(*arguments)
                self.assertNotEqual(process.returncode, 0)
                self.assertIn(result["status"], {"conflict", "verification-failed"})
                self.assertTrue(stale_path.is_symlink())
                self.assertEqual(outside.read_bytes(), outside_before)

    def test_target_below_symlinked_parent_is_rejected_without_external_write(
        self,
    ) -> None:
        actual_parent = self.target / "actual"
        actual_target = actual_parent / "consumer"
        shutil.copytree(
            ROOT / "harness/template/.hive/knowledge",
            actual_target / ".hive/knowledge",
        )
        alias = self.target / "alias"
        try:
            alias.symlink_to(actual_parent, target_is_directory=True)
        except OSError as error:
            self.skipTest(f"symlink creation is unavailable: {error}")
        before = snapshot_tree(actual_target)

        process, result = self.invoke(
            "index",
            "rebuild",
            "--target",
            str(alias / "consumer"),
        )
        self.assertNotEqual(process.returncode, 0)
        self.assertIn(
            result["status"],
            {"error", "conflict", "verification-failed"},
        )
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual(snapshot_tree(actual_target), before)


if __name__ == "__main__":
    unittest.main()
