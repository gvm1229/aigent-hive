"""Source-only synthetic investigation; never select an existing user root.

Run from the source root with --work tests/work/<new-name>. No install command,
host command, network access, real preferences, or raw conversation input.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import struct
import subprocess
import sys
import threading
import time

import yaml


ROOT = Path(__file__).resolve().parents[3]


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def snapshot(root):
    return {p.relative_to(root).as_posix(): (digest(p.read_bytes()), p.stat().st_mtime_ns)
            for p in root.rglob("*") if p.is_file()}


class Watch:
    """Observe even transient lock/journal writes; no filesystem payload capture."""
    def __init__(self, root):
        self.events = set()
        self.error = None
        self.ready = threading.Event()
        self.stop = threading.Event()
        self.root = root

    def __enter__(self):
        if os.name != "nt":
            raise RuntimeError("This investigation watcher requires Windows")
        self.k = ctypes.WinDLL("kernel32", use_last_error=True)
        self.k.CreateFileW.restype = wintypes.HANDLE
        self.k.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                                      wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
        self.k.ReadDirectoryChangesW.argtypes = [wintypes.HANDLE, wintypes.LPVOID, wintypes.DWORD,
                                                wintypes.BOOL, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD),
                                                wintypes.LPVOID, wintypes.LPVOID]
        self.k.CancelIoEx.argtypes = [wintypes.HANDLE, wintypes.LPVOID]
        self.k.CloseHandle.argtypes = [wintypes.HANDLE]
        self.handle = self.k.CreateFileW(str(self.root), 1, 7, None, 3, 0x02000000, None)
        if self.handle == wintypes.HANDLE(-1).value:
            raise ctypes.WinError(ctypes.get_last_error())
        self.thread = threading.Thread(target=self.read, daemon=True)
        self.thread.start()
        self.ready.wait(5)
        time.sleep(0.03)
        return self

    def read(self):
        buf = ctypes.create_string_buffer(65536)
        while not self.stop.is_set():
            count = wintypes.DWORD()
            self.ready.set()
            ok = self.k.ReadDirectoryChangesW(self.handle, buf, len(buf), True, 0x1F,
                                              ctypes.byref(count), None, None)
            if not ok:
                code = ctypes.get_last_error()
                if not self.stop.is_set():
                    self.error = code
                return
            if not count.value:
                self.error = "buffer-overflow"
                return
            offset = 0
            while True:
                next_offset, action, length = struct.unpack_from("III", buf.raw, offset)
                name = buf.raw[offset + 12:offset + 12 + length].decode("utf-16-le").replace("\\", "/")
                if ".hive-knowledge-lock-" in name:
                    name = name[:name.index(".hive-knowledge-lock-")] + "<temporary-lock>"
                self.events.add((action, name))
                if not next_offset:
                    break
                offset += next_offset

    def __exit__(self, *_):
        time.sleep(0.05)
        self.stop.set()
        self.k.CancelIoEx(self.handle, None)
        self.k.CloseHandle(self.handle)
        self.thread.join(2)
        if self.thread.is_alive():
            raise RuntimeError("watcher did not stop")
        if self.error:
            raise RuntimeError(f"watcher error: {self.error}")


class Investigation:
    def __init__(self, binary, work):
        self.binary, self.work = binary, work
        self.user = work / "user"
        self.a, self.b = work / "project-a", work / "project-b"
        self.empty = work / "empty-root"
        for p in (self.user, self.a, self.b, self.empty, work / "fake-bin"):
            p.mkdir(parents=True)
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(("HIVE_", "CODEX_"))}
        self.env.update({k: str(work / "environment" / k.lower()) for k in
                         ("HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME",
                          "HIVE_INSTALL_ROOT", "HIVE_CACHE_ROOT", "TEMP", "TMP")})
        for k in ("HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME",
                  "HIVE_INSTALL_ROOT", "HIVE_CACHE_ROOT", "TEMP", "TMP"):
            Path(self.env[k]).mkdir(parents=True)
        # Capability receipt is an existing synthetic fixture; no host discovery or launch.
        self.env["PATH"] = str(work / "fake-bin")
        self.rows = []
        config = self.user / ".hive/config"
        config.mkdir(parents=True)
        # Existing conformance fixture shape; no second global installation.
        (config / "user-setup.yml").write_text(yaml.safe_dump({
            "schema_version": 1, "interface_language": "ko",
            "wiki": {"enabled": True, "language": "ko"},
            "profile": {"contexts": ["web-developer"]}, "persona": {"id": "balanced"},
            "selected_hosts": ["codex"], "skills": {"mode": "all"},
            "usage_guard": {"enabled": False, "stop_remaining_percent": 20,
                            "codexbar_fallback_enabled": False}}, allow_unicode=True), encoding="utf-8")
        answers = yaml.safe_load((ROOT / "tests/fixtures/setup/answers-base.yml").read_text("utf-8"))
        answers.update(project_name="synthetic-boundary", setup_mode="custom", interface_language="ko",
                       wiki={"enabled": True, "language": "ko"}, persona={"id": "balanced"},
                       skills={"mode": "individual", "selected": ["knowledge-capture", "knowledge-recall", "knowledge-maintain", "knowledge-promote", "knowledge-scan"]},
                       root_knowledge_promotion_categories=[], persistent_roles=[])
        self.answers = work / "answers.yml"
        self.answers.write_text(yaml.safe_dump(answers, allow_unicode=True), encoding="utf-8")
        self.caps = ROOT / "tests/fixtures/setup/capabilities-codex-host-native.json"

    def clean(self, value):
        if isinstance(value, dict):
            return {k: self.clean(v) for k, v in value.items()}
        if isinstance(value, list):
            return [self.clean(v) for v in value]
        if isinstance(value, str):
            return value.replace(str(self.work), "{WORK}").replace(str(ROOT), "{SOURCE}")
        return value

    def raw(self, args):
        p = subprocess.run([str(self.binary), *map(str, args), "--output", "json"],
                           cwd=self.a, env=self.env, capture_output=True, text=True,
                           encoding="utf-8", timeout=60)
        value = json.loads(p.stdout)
        assert p.returncode == value["exit_code"], value["code"]
        return value

    def state(self):
        path = self.user / ".hive/config/collections.yml"
        if not path.exists():
            return {}
        registry = yaml.safe_load(path.read_text("utf-8"))
        return {c["collection_id"]: c["state"] for c in registry["collections"]}

    def run(self, label, args, expected=None):
        before = snapshot(self.user)
        states = self.state()
        with Watch(self.user) as watch:
            value = self.raw(args)
        after = snapshot(self.user)
        changed = sorted(k for k in before.keys() | after.keys() if before.get(k) != after.get(k))
        command = self.clean(list(map(str, args)))
        for i, argument in enumerate(command[:-1]):
            if argument in ("--authorization-id", "--authorization-token"):
                command[i + 1] = "<one-time-credential>"
        row = {"case": label, "command": command,
               "code": value["code"], "exit": value["exit_code"],
               "message": self.clean(value["message"]), "persistent_changes": changed,
               "hashes": {k: [before.get(k, [None])[0], after.get(k, [None])[0]] for k in changed},
               "filesystem_events": sorted(watch.events), "states_before": states,
               "states_after": self.state()}
        data = value.get("data") or {}
        if "hits" in data:
            row["hit_count"] = len(data["hits"])
            row["hit_collections"] = sorted({h.get("collection_id", h.get("source_project")) for h in data["hits"]})
        if "plan" in data:
            row["disposition"] = data["plan"]["disposition"]
        self.rows.append(row)
        print(label, value["exit_code"], value["code"], "changed", len(changed), "events", len(watch.events), flush=True)
        self.save()
        if expected is not None:
            assert value["exit_code"] == expected, (label, value["message"])
        return value

    def save(self):
        report = {"platform": sys.platform, "binary_sha256": digest(self.binary.read_bytes()),
                  "watcher": "ReadDirectoryChangesW; 64KiB; recursive; overflow/error fatal",
                  "global_setup": "synthetic operational configuration fixture, not user installation",
                  "cases": self.rows}
        (self.work / "observations.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    def setup_args(self, target, mode="--apply"):
        return ["setup", "--target", target, "--user-root", self.user, "--answers", self.answers,
                "--capabilities", self.caps, mode]

    def collection_a(self):
        registry = yaml.safe_load((self.user / ".hive/config/collections.yml").read_text("utf-8"))
        return next(c["collection_id"] for c in registry["collections"]
                    if c.get("local_locator") == str(self.a) or c.get("collection_id") == getattr(self, "ca", None))

    def detach(self):
        if self.state()[self.ca] == "detached":
            return
        v = self.run("authorize-detach", ["knowledge", "authorize-collection", "--user-root", self.user,
            "--operation", "detach", "--collection", self.ca, "--expires-at", str(int(time.time()) + 45),
            "--nonce", "synthetic-detach-" + str(len(self.rows)), "--confirm-current-action"], 0)
        self.run("detach", ["knowledge", "collection", "detach", "--user-root", self.user,
            "--collection", self.ca, "--authorization-id", v["data"]["authorization_id"],
            "--authorization-token", v["data"]["authorization_token"]], 0)
        # Do not archive one-time authorization material, even synthetic.
        self.rows[-1]["command"][-3] = "<one-time-id>"
        self.rows[-1]["command"][-1] = "<one-time-token>"
        self.save()

    def request(self, name, fact, **extra):
        r = {"collection_id": self.ca, "claim_key": "synthetic.output", "locator": "auto",
             "kind": "preference", "status": "user-stated", "visibility": "project-private",
             "normalized_fact": fact, "provenance": {"source_kind": "reviewed-artifact",
             "summary": "Synthetic authorized conversation review", "locator": "fixture:review-a",
             "digest": digest(b"synthetic-review-a")}, "sources": ["fixture:review-a"], **extra}
        path = self.work / (name + ".json")
        path.write_text(json.dumps(r, ensure_ascii=False), encoding="utf-8")
        return path

    def execute(self):
        self.run("initialize-synthetic-global", ["knowledge", "remember", "--user-root", self.user,
            "--user-statement", "가짜 전역 규칙: globalbeacon 사용", "--claim-key", "synthetic.global", "--kind", "convention"], 0)
        self.run("fresh-project-a", self.setup_args(self.a), 0)
        self.run("fresh-project-b", self.setup_args(self.b), 0)
        self.ca = self.collection_a()
        self.run("no-user-root", ["knowledge", "query", "--target", self.a, "--text", "globalbeacon"])
        self.run("empty-user-root", ["knowledge", "query", "--target", self.a, "--user-root", self.empty, "--text", "globalbeacon"])
        self.run("global-read", ["knowledge", "retrieve", "--target", self.a, "--user-root", self.user, "--query", "globalbeacon"], 0)
        session = ["--target", self.a, "--host", "codex", "--session-id", "synthetic-session"]
        paths = ["--process-id", str(os.getpid()), "--path", "docs/example.md"]
        probes = [
            ("session-begin", ["session", "begin", *session, *paths]),
            ("session-check", ["session", "check", *session, *paths]),
            ("session-update", ["session", "update", *session, *paths]),
            ("session-close", ["session", "close", *session]),
            ("setup-status", ["setup", "--target", self.a, "--user-root", self.user, "--show", "--summary"]),
            ("setup-validate", self.setup_args(self.a, "--validate")),
            ("query-detached", ["knowledge", "query", "--target", self.a, "--user-root", self.user, "--text", "globalbeacon"]),
            ("retrieve-detached", ["knowledge", "retrieve", "--target", self.a, "--user-root", self.user, "--query", "globalbeacon"]),
            ("lint-detached", ["knowledge", "lint", "--target", self.a, "--user-root", self.user]),
            ("graph-status", ["knowledge", "graph", "status", "--target", self.a]),
            ("restart-fresh-process", ["knowledge", "retrieve", "--target", self.a, "--user-root", self.user, "--query", "globalbeacon"]),
            ("index-rebuild", ["index", "rebuild", "--user-root", self.user]),
            ("refresh", ["knowledge", "refresh", "--user-root", self.user]),
            ("setup-reapply", self.setup_args(self.a)),
            ("remember-existing-global-noop", ["knowledge", "remember", "--user-root", self.user,
              "--user-statement", "가짜 전역 규칙: globalbeacon 사용", "--claim-key", "synthetic.global", "--kind", "convention"]),
        ]
        for name, args in probes:
            self.detach()
            self.run(name, args)
        self.detach()
        third = self.work / "project-c"
        third.mkdir()
        self.run("setup-other-new-project", self.setup_args(third), 0)
        self.detach()
        invalid_source, invalid_draft = self.work / "valid-source.txt", self.work / "invalid-draft.md"
        invalid_source.write_text("synthetic invalid ingest source", encoding="utf-8")
        invalid_draft.write_text("invalid frontmatter", encoding="utf-8")
        self.run("failed-ingest-detached", ["knowledge", "ingest", "--target", self.a,
                 "--user-root", self.user, "--source", invalid_source, "--wiki", invalid_draft])
        # Synthetic project-private capture: physical location and visibility are separate.
        self.run("reattach-before-capture", ["index", "rebuild", "--user-root", self.user], 0)
        req = self.request("review", "가짜 프로젝트 규칙: localbeacon 짧은 설명 선택")
        first = self.run("project-remember", ["knowledge", "remember", "--user-root", self.user, "--request", req], 0)
        self.run("duplicate-review", ["knowledge", "remember", "--user-root", self.user, "--request", req], 0)
        claim = first["data"]["plan"]["new_claim"]
        correction = self.request("correction", "가짜 프로젝트 규칙: localbeacon 자세한 설명 선택",
            supersedes=[claim["claim_id"]], expected_active_digest=claim["digest"])
        self.run("correct-reviewed-preference", ["knowledge", "remember", "--user-root", self.user, "--request", correction], 0)
        for name, project in (("own-retrieval", self.a), ("unrelated-retrieval", self.b)):
            self.run(name, ["knowledge", "retrieve", "--target", project, "--user-root", self.user, "--query", "localbeacon"], 0)
        source, draft = self.work / "source.txt", self.work / "draft.md"
        source.write_text("가짜 자료 ingestbeacon", encoding="utf-8")
        draft.write_text("---\nschema_version: 1\nid: synthetic-ingest\nkind: concept\nsummary: 가짜 지식\ntags: [synthetic]\naliases: []\nsources: [raw:self]\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: 2026-11-04T00:00:00Z\nupdated_at: 2026-11-04T00:00:00Z\n---\n\n가짜 자료 ingestbeacon\n", encoding="utf-8")
        self.detach()
        self.run("ingest-detached", ["knowledge", "ingest", "--target", self.a, "--user-root", self.user,
                 "--source", source, "--wiki", draft], 0)
        # A controlled concurrency batch: every worker receives a distinct complete command.
        for iteration in range(5):
            self.detach()
            before = self.state()
            a = ["index", "rebuild", "--user-root", self.user]
            b = ["knowledge", "retrieve", "--target", self.b, "--user-root", self.user, "--query", "globalbeacon"]
            with concurrent.futures.ThreadPoolExecutor(2) as pool:
                futures = [pool.submit(self.raw, args) for args in (a, b)]
                results = [f.result() for f in futures]
            self.rows.append({"case": f"concurrent-rebuild-read-{iteration}", "commands": self.clean([list(map(str, a)), list(map(str, b))]),
                              "codes": [r["code"] for r in results], "states_before": before, "states_after": self.state()})
            self.save()
        # No-op status/session races, with no reconciling command, test a competing cause.
        self.detach()
        for iteration in range(5):
            with concurrent.futures.ThreadPoolExecutor(2) as pool:
                results = list(pool.map(self.raw, [probes[4][1], probes[7][1]]))
            self.rows.append({"case": f"concurrent-status-read-{iteration}", "codes": [r["code"] for r in results], "states_after": self.state()})
        self.save()
        # Authorization-vs-registration races: no barrier inside Hive, so retain order uncertainty.
        for iteration in range(5):
            self.run("prepare-race-attached", ["index", "rebuild", "--user-root", self.user], 0)
            auth = self.raw(["knowledge", "authorize-collection", "--user-root", self.user,
                "--operation", "detach", "--collection", self.ca, "--expires-at", str(int(time.time()) + 45),
                "--nonce", "synthetic-race-" + str(iteration), "--confirm-current-action"])
            assert auth["exit_code"] == 0
            new_project = self.work / ("concurrent-project-" + str(iteration))
            new_project.mkdir()
            detach = ["knowledge", "collection", "detach", "--user-root", self.user, "--collection", self.ca,
                      "--authorization-id", auth["data"]["authorization_id"],
                      "--authorization-token", auth["data"]["authorization_token"]]
            with concurrent.futures.ThreadPoolExecutor(2) as pool:
                results = list(pool.map(self.raw, [detach, self.setup_args(new_project)]))
            self.rows.append({"case": f"concurrent-detach-register-{iteration}",
                              "codes": [r["code"] for r in results], "states_after": self.state()})
            self.save()
        # Separate failure fixtures: preserve exact pre-command bytes and restore explicitly.
        for name, rel, replacement in (("missing-global-index", ".hive/index/hive.sqlite3", None),
                                       ("corrupt-global-index", ".hive/index/hive.sqlite3", b"invalid sqlite"),
                                       ("dirty-global", ".hive/index/rag-dirty.json", b"{}")):
            path = self.user / rel
            saved = path.read_bytes() if path.exists() else None
            if replacement is None:
                path.unlink()
            else:
                path.write_bytes(replacement)
            self.run(name, ["knowledge", "retrieve", "--target", self.b, "--user-root", self.user, "--query", "globalbeacon"])
            if saved is None:
                path.unlink(missing_ok=True)
            else:
                path.write_bytes(saved)
        self.run("final-global-read", ["knowledge", "retrieve", "--target", self.b, "--user-root", self.user, "--query", "globalbeacon"], 0)
        connection = sqlite3.connect(":memory:")
        connection.deserialize((self.user / ".hive/index/hive.sqlite3").read_bytes())
        counts = {}
        for table in ("documents", "claims", "chunks"):
            counts[table] = connection.execute(f"SELECT count(*) FROM {table} WHERE collection_id=?", (self.ca,)).fetchone()[0]
        connection.close()
        self.rows.append({"case": "physical-location-audit", "project_local_index_exists": (self.a / ".hive/index/hive.sqlite3").exists(),
            "global_sqlite_project_rows": counts, "global_canonical_project_claims": len(list((self.user / ".hive/knowledge/Claims" / self.ca).glob("*.md"))),
            "local_canonical_wiki": (self.a / ".hive/knowledge/Wiki/synthetic-ingest.md").exists()})
        self.save()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--nested-user-root", action="store_true")
    args = parser.parse_args()
    work = args.work.absolute()
    allowed = (ROOT / "tests/work").resolve()
    if not work.resolve().is_relative_to(allowed) or work.exists():
        raise SystemExit("choose a new tests/work child, never an existing directory")
    for parent in work.parents:
        if parent.exists() and (parent.is_symlink() or parent.is_junction()):
            raise SystemExit("linked work ancestor refused")
        if parent == ROOT:
            break
    run = Investigation(args.binary.resolve(strict=True), work)
    if args.nested_user_root:
        nested = run.a / "isolated-root"
        shutil.copytree(run.user, nested)
        run.user = nested
        run.run("nested-initialized-root", ["knowledge", "remember", "--user-root", nested,
                "--user-statement", "가짜 중첩 저장소 nestedbeacon", "--claim-key", "synthetic.nested"], 0)
        run.run("nested-modern-project-setup", run.setup_args(run.a), 0)
        run.run("nested-modern-project-query", ["knowledge", "retrieve", "--user-root", nested,
                "--target", run.a, "--query", "nestedbeacon"], 0)
    else:
        run.execute()


if __name__ == "__main__":
    main()
