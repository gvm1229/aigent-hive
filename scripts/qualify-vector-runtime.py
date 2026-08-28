#!/usr/bin/env python3
"""Explicitly consented, disposable native-vector acceptance; no provider calls.

This is not a performance gate. Each platform runs real pinned packages and embeddings.
The ordinary conformance suite tests this runner without downloading models.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValueError("qualification refuses a linked or non-regular evidence file")
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def snapshot(paths: list[Path]) -> dict[str, str]:
    return {str(path): digest(path) for path in paths}


def tree_snapshot(root: Path) -> dict[str, str]:
    return {str(path.relative_to(root)): digest(path) if path.is_file() else "directory"
            for path in root.rglob("*")}


def stop_cli_tree(process: subprocess.Popen) -> None:
    """Stop only this live qualification CLI and its observed descendants on timeout."""
    if process.poll() is not None:
        return
    try:
        if os.name == "nt":
            result = subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                    capture_output=True, timeout=15)
            if result.returncode and process.poll() is None:
                raise RuntimeError("cannot terminate owned CLI process tree")
        else:
            # Freeze this parent first so it cannot launch another helper during enumeration.
            # Model helpers use threads, not independent descendant-spawning services.
            os.kill(process.pid, signal.SIGSTOP)
            result = subprocess.run(["ps", "-axo", "pid=,ppid="], capture_output=True, text=True,
                                    timeout=10, check=True)
            children: dict[int, list[int]] = {}
            for line in result.stdout.splitlines():
                child, parent = map(int, line.split())
                children.setdefault(parent, []).append(child)
            owned = [process.pid]
            for parent in owned:
                owned.extend(child for child in children.get(parent, []) if child not in owned)
            for child in reversed(owned[1:]):
                try:
                    os.kill(child, signal.SIGKILL)
                except ProcessLookupError:
                    pass
    finally:
        if process.poll() is None:
            process.kill()
        process.wait(timeout=10)


def consumer_fixture(target: Path) -> list[Path]:
    config = target / ".hive/config/user-setup.yml"
    write_json(config, {
        "schema_version": 1, "interface_language": "en",
        "wiki": {"enabled": True, "language": "both"},
        "profile": {"id": "web-developer"}, "persona": {"id": "balanced"},
        "selected_hosts": ["codex"], "skills": {"mode": "individual", "selected": ["setup-hive"]},
        "usage_guard": {"enabled": False, "stop_remaining_percent": 20, "codexbar_fallback_enabled": False},
    })
    write_json(target / ".hive/knowledge/suppression.yml", {"schema_version": 1, "entries": []})
    raw = target / ".hive/knowledge/Raw"
    raw.mkdir(parents=True)
    wiki = target / ".hive/knowledge/Wiki"
    wiki.mkdir()
    protected = [config]
    for number in range(8):
        identity = f"vector-example-{number}"
        body = (f"Reference DOC-{number:03d}. A local search index can be rebuilt from original Markdown. "
                "Similar meanings can be found across languages. The original documents remain unchanged.\n")
        metadata = {
            "schema_version": 1, "id": identity, "kind": "concept",
            "summary": f"Disposable semantic search example {number}", "tags": ["search"], "aliases": [identity],
            "sources": ["raw:.hive/knowledge/Raw/source-id/" + "0" * 64 + ".md#sha256:" + "0" * 64],
            "links": [], "contradictions": [], "status": "active",
            "created_at": "2026-08-28T00:00:00Z", "updated_at": "2026-08-28T00:00:00Z",
        }
        page = wiki / f"{identity}.md"
        page.write_text("---\n" + json.dumps(metadata) + "\n---\n\n" + body, encoding="utf-8")
        protected.append(page)
    return protected


class Qualification:
    def __init__(self, binary: Path, work: Path):
        self.binary = binary
        self.work = work
        self.report = {
            "schema_version": 1, "platform": platform.platform(), "machine": platform.machine(),
            "python": sys.version, "binary_sha256": digest(binary),
            "purpose": "native functional acceptance, not scale or latency qualification",
            "status": "running", "calls": [],
        }

    def save(self) -> None:
        write_json(self.work / "receipt.json", self.report)

    def call(self, *args: str, expect_failure: bool = False) -> dict:
        started = time.perf_counter()
        record = {"arguments": list(args)}
        self.report["calls"].append(record)
        try:
            timeout = 1000 if args[:3] == ("knowledge", "vector", "enable") else 150
            process = subprocess.Popen([str(self.binary), *args, "--output", "json"],
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8")
            try:
                stdout, _stderr = process.communicate(timeout=timeout)
            except subprocess.TimeoutExpired:
                stop_cli_tree(process)
                process.communicate(timeout=10)
                raise
            record.update(exit_code=process.returncode, result=json.loads(stdout))
            if (process.returncode == 0) == expect_failure:
                raise RuntimeError(f"unexpected result for {args[:3]}: {record['result']}")
            return record["result"].get("data", {})
        except subprocess.TimeoutExpired as error:
            record["error"] = "CLI timeout; result is not acceptance evidence"
            raise RuntimeError(record["error"]) from error
        finally:
            record["elapsed_seconds"] = time.perf_counter() - started
            self.save()

    def run(self) -> None:
        target = self.work / "consumer"
        protected = consumer_fixture(target)
        self.call("knowledge", "refresh", "--user-root", str(target))
        # First FTS preparation may normalize registry representation. Vector preservation
        # starts after that ordinary preparation, not before its documented serialization.
        protected += [target / ".hive/config/collections.yml", target / ".hive/index/hive.sqlite3"]
        before = snapshot(protected)
        scope = ["--user-root", str(target), "--target", str(target),
                 "--collection", "user-root", "--visibility", "shared"]
        query = ["knowledge", "retrieve", "--user-root", str(target), "--target", str(target),
                 "--scope", "collection:user-root", "--query", "원본 문서를 바꾸지 않고 검색 색인을 다시 만들기",
                 "--top-k", "5", "--byte-budget", "16384"]
        plain = self.call(*query)
        absent = self.call(*query, "--mode", "semantic")
        assert absent["search"]["used"] == ["fts"]
        python = str(Path(sys.executable).resolve())
        before_preview = tree_snapshot(target)
        preview = self.call("knowledge", "vector", "preview", *scope, "--python", python)
        assert tree_snapshot(target) == before_preview
        before_consent = tree_snapshot(target)
        self.call("knowledge", "vector", "enable", *scope, "--python", python,
                  "--consent-digest", "sha256:" + "0" * 64, expect_failure=True)
        assert tree_snapshot(target) == before_consent
        self.call("knowledge", "vector", "enable", *scope, "--python", python,
                  "--consent-digest", preview["consent_digest"])
        assert not any(path.is_file() for path in (target / ".hive/index/vector/work").rglob("*"))
        for _ in range(10):
            built = self.call("knowledge", "vector", "rebuild", *scope,
                              "--max-seconds", "10", "--workers", "2")
            if built["complete"]:
                break
        else:
            raise RuntimeError("bounded fixture did not complete within ten rebuild calls")
        assert built["chunks"] == 8
        assert not built["cleanup_pending"]
        assert not list((target / ".hive/index/vector/scopes").rglob("staging.sqlite3"))
        ready = self.call("knowledge", "vector", "status", *scope)
        assert ready["runtime_verified"] and ready["index_ready"]
        found = self.call(*query, "--mode", "semantic")
        assert found["search"]["used"] == ["fts", "vector"] and found["hits"]
        assert all(hit["visibility"] == "shared" for hit in found["hits"])
        unchanged = self.call("knowledge", "vector", "rebuild", *scope, "--max-seconds", "10", "--workers", "2")
        assert unchanged["complete"] and unchanged["embedded"] == 0
        assert unchanged["database_digest"] == built["database_digest"]
        assert unchanged["snapshot_id"] != built["snapshot_id"]
        controls = list((target / ".hive/config/vector-state").glob("scope-*.json"))
        assert len(controls) == 1
        before_rollback = json.loads(controls[0].read_bytes())
        assert before_rollback["active"]["id"] == unchanged["snapshot_id"]
        assert before_rollback["previous"]["id"] == built["snapshot_id"]
        self.call("knowledge", "vector", "rollback", *scope)
        after_rollback = json.loads(controls[0].read_bytes())
        assert after_rollback["active"] == before_rollback["previous"]
        assert after_rollback["previous"] == before_rollback["active"]
        assert self.call(*query, "--mode", "semantic")["search"]["used"] == ["fts", "vector"]
        fresh = self.call("knowledge", "vector", "rebuild", *scope, "--rebuild-mode", "fresh", "--max-seconds", "10", "--workers", "2")
        assert fresh["complete"] and fresh["embedded"] == 8 and not fresh["cleanup_pending"]
        assert not list((target / ".hive/index/vector/scopes").rglob("staging.sqlite3"))
        assert not list((target / ".hive/index/vector/scopes").rglob("quarantine-*"))
        control = json.loads(controls[0].read_bytes())
        runtimes = target / ".hive/index/vector/runtimes"
        # Discover the worker only inside the exact approved runtime, never in another scope.
        workers = list(runtimes.rglob("vector_helper.py"))
        assert len(workers) == 1
        worker = workers[0]
        original = worker.read_bytes()
        try:
            worker.write_bytes(original + b"\n# deliberate corruption in disposable acceptance\n")
            assert self.call(*query, "--mode", "semantic")["search"]["used"] == ["fts"]
        finally:
            worker.write_bytes(original)
        assert self.call(*query, "--mode", "semantic")["search"]["used"] == ["fts", "vector"]
        self.call("knowledge", "vector", "disable", *scope)
        disabled = self.call(*query, "--mode", "semantic")
        assert disabled["search"]["used"] == ["fts"]
        assert self.call(*query) == plain
        assert snapshot(protected) == before
        assert digest(self.binary) == self.report["binary_sha256"]
        self.report.update(status="passed", protected_bytes_preserved=True, chunks=8,
                           runtime_id=control["runtime"]["id"],
                           proven=["missing fallback", "exact consent", "native embedding and sqlite-vec", "semantic citations",
                                   "unchanged reuse", "rollback", "corruption fallback", "disable", "FTS preservation"],
                           not_proven=["50k performance", "private/confidential isolation", "parent cancellation", "public package identity"])
        self.save()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hive", required=True, type=Path)
    parser.add_argument("--authorize-install", action="store_true",
                        help="approve exact previews only in a newly created disposable fixture")
    args = parser.parse_args()
    if sys.flags.optimize:
        parser.error("optimized Python disables acceptance assertions; run without -O or PYTHONOPTIMIZE")
    if not args.authorize_install:
        parser.error("explicit --authorize-install is required; no network or fixture mutation performed")
    binary = args.hive.resolve(strict=True)
    if not binary.is_file() or args.hive.is_symlink():
        parser.error("--hive must be an existing regular executable")
    work_root = ROOT / "tests/work"
    work_root.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="vector-native-", dir=work_root))
    qualification = Qualification(binary, work)
    qualification.report["source_commit"] = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()
    try:
        qualification.run()
    except Exception as error:
        qualification.report.update(status="failed", error=str(error))
        qualification.save()
        print(json.dumps({"status": "failed", "receipt": str(work / "receipt.json"), "error": str(error)}), flush=True)
        return 1
    print(json.dumps({"status": "passed", "receipt": str(work / "receipt.json")}), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
