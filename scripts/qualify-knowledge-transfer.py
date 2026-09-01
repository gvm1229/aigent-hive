#!/usr/bin/env python3
"""Produce or consume the SAME synthetic knowledge archive across CI operating systems."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import platform
import sys
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("produce", "consume"))
    parser.add_argument("--hive", required=True, type=Path)
    parser.add_argument("--work", required=True, type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--files", type=int, choices=(100, 1000, 5000), default=100)
    parser.add_argument("--legacy-cli", action="store_true", help="measure the pre-transfer export/import surface")
    parser.add_argument("--multi", action="store_true", help="qualify a two-bundle merge with exact overlap")
    parser.add_argument("--require-cross-os", action="store_true", help="refuse same-OS evidence in a cross-platform CI acceptance")
    args = parser.parse_args()
    work = args.work.resolve()
    if not work.is_relative_to(ROOT / "tests/work") or work == ROOT / "tests/work":
        parser.error("--work must name an isolated directory below tests/work")
    work.mkdir(parents=True, exist_ok=False)
    binary = args.hive.resolve(strict=True)
    report = {"platform": platform.platform(), "machine": platform.machine(),
              "binary_sha256": digest(binary.read_bytes()), "steps": [], "status": "running"}

    def call(*argv: str) -> dict:
        start = time.perf_counter()
        process = subprocess.Popen([str(binary), *argv, "--output", "json"], stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, encoding="utf-8")
        try:
            stdout, _stderr = process.communicate(timeout=600)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            process.kill()
            process.communicate(timeout=10)
            raise
        value = json.loads(stdout)
        peak = None
        if sys.platform == "win32":
            import ctypes
            from ctypes import wintypes
            class Counters(ctypes.Structure):
                _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [(name, ctypes.c_size_t) for name in
                    ("peak_working", "working", "peak_paged", "paged", "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile")]
            counters = Counters(); counters.cb = ctypes.sizeof(counters)
            api = ctypes.WinDLL("kernel32", use_last_error=True)
            peak = counters.peak_working if api.K32GetProcessMemoryInfo(wintypes.HANDLE(process._handle), ctypes.byref(counters), counters.cb) else None
        else:
            import resource
            usage = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
            peak = int(usage if sys.platform == "darwin" else usage * 1024)
        report["steps"].append({"command": list(argv), "seconds": time.perf_counter() - start,
                                "exit_code": process.returncode, "peak_working_set_bytes": peak})
        if process.returncode:
            raise RuntimeError(f"{argv[:3]}: {value.get('message')}")
        return value["data"]

    def save() -> None:
        (work / "receipt.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def initialize_user(path: Path) -> Path:
        config = path / ".hive/config/user-setup.yml"
        config.parent.mkdir(parents=True)
        config.write_text(json.dumps({
            "schema_version": 1, "interface_language": "en",
            "wiki": {"enabled": True, "language": "both"},
            "profile": {"contexts": ["web-developer"]}, "persona": {"id": "balanced"},
            "selected_hosts": ["codex", "claude", "antigravity"], "skills": {"mode": "all"},
            "usage_guard": {"enabled": True, "stop_remaining_percent": 20,
                            "codexbar_fallback_enabled": True, "discord": {"enabled": False}},
        }), encoding="utf-8", newline="\n")
        wiki = path / ".hive/knowledge/Wiki"
        wiki.mkdir(parents=True)
        return wiki

    def write_pages(wiki: Path, indices: range) -> dict[str, str]:
        for index in indices:
            page_id = f"transfer-{index:05d}"
            text = (f"---\nschema_version: 1\nid: {page_id}\nkind: concept\nsummary: 이동 지식 {index}\n"
                    "tags: [transfer]\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\n"
                    "created_at: '2026-08-01T00:00:00Z'\nupdated_at: '2026-08-01T00:00:00Z'\n---\n\n"
                    f"Portable knowledge anchor{index:05d}. 한글 원문·수치 {index} 보존.\n")
            (wiki / f"{page_id}.md").write_text(text, encoding="utf-8", newline="\n")
        return {p.name: digest(p.read_bytes()) for p in sorted(wiki.glob("*.md"))}

    try:
        if args.mode == "produce":
            user = work / "사용자 지식 with spaces"
            wiki = initialize_user(user)
            if args.multi:
                first_user = work / "computer-a knowledge"
                second_user = work / "computer-b knowledge"
                first_wiki = initialize_user(first_user)
                second_wiki = initialize_user(second_user)
                overlap = max(1, args.files // 10)
                first_inventory = write_pages(first_wiki, range(0, args.files // 2 + overlap))
                second_inventory = write_pages(second_wiki, range(args.files // 2, args.files))
                call("knowledge", "refresh", "--user-root", str(first_user))
                call("knowledge", "refresh", "--user-root", str(second_user))
                command = ("knowledge", "export") if args.legacy_cli else ("knowledge", "transfer", "export", "--apply")
                first_archive = work / "portable-a.hivekb"
                second_archive = work / "portable-b.hivekb"
                first_export = call(*command, "--user-root", str(first_user), "--scope", "all-portable", "--bundle", str(first_archive))
                second_export = call(*command, "--user-root", str(second_user), "--scope", "all-portable", "--bundle", str(second_archive))
                expected = {"producer_os": platform.system(),
                            "files": {**first_inventory, **second_inventory},
                            "archives": {"portable-a.hivekb": first_export["archive_sha256"],
                                         "portable-b.hivekb": second_export["archive_sha256"]}}
                report["export"] = {"first": first_export, "second": second_export}
            else:
                inventory = write_pages(wiki, range(args.files))
                call("knowledge", "refresh", "--user-root", str(user))
                archive = work / "portable.hivekb"
                command = ("knowledge", "export") if args.legacy_cli else ("knowledge", "transfer", "export", "--apply")
                exported = call(*command, "--user-root", str(user), "--scope", "all-portable", "--bundle", str(archive))
                expected = {"producer_os": platform.system(), "files": inventory,
                            "archive_sha256": exported["archive_sha256"]}
                report["export"] = exported
            (work / "expected.json").write_text(json.dumps(expected, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        else:
            if not args.input:
                parser.error("consume requires --input containing portable.hivekb and expected.json")
            expected = json.loads((args.input / "expected.json").read_text(encoding="utf-8"))
            if args.require_cross_os and expected["producer_os"] == platform.system():
                raise RuntimeError("cross-OS acceptance cannot use a same-OS producer")
            user = work / "사용자 지식 with spaces"
            wiki = initialize_user(user)
            if args.multi:
                archives = []
                for name, expected_digest in expected["archives"].items():
                    archive = (args.input / name).resolve(strict=True)
                    if digest(archive.read_bytes()) != expected_digest:
                        raise RuntimeError("archive changed during machine-to-machine handoff")
                    archives.append(archive)
                preview = call("knowledge", "transfer", "merge", "preview", "--user-root", str(user),
                               "--bundle", str(archives[0]), "--bundle", str(archives[1]))
                if preview["merge"]["conflict_paths"] or preview["merge"]["semantic_candidates"]:
                    raise RuntimeError("synthetic multi-bundle fixture unexpectedly needs review")
                imported = call("knowledge", "transfer", "merge", "apply", "--user-root", str(user),
                                "--bundle", str(archives[1]), "--bundle", str(archives[0]),
                                "--preview-digest", preview["merge_preview_digest"])
                if not imported["import"]["transfer"]["complete"]:
                    raise RuntimeError("FTS not ready after merged import")
            else:
                archive = (args.input / "portable.hivekb").resolve(strict=True)
                if digest(archive.read_bytes()) != expected["archive_sha256"]:
                    raise RuntimeError("archive changed during machine-to-machine handoff")
                preview = call("knowledge", "transfer", "import", "--preview", "--user-root", str(user), "--bundle", str(archive),
                               "--expected-sha256", expected["archive_sha256"])
                imported = call("knowledge", "transfer", "import", "--apply", "--user-root", str(user), "--bundle", str(archive),
                                "--expected-sha256", expected["archive_sha256"], "--preview-digest", preview["transfer_preview_digest"])
                if not imported["transfer"]["complete"]:
                    raise RuntimeError("FTS not ready after import")
            inventory = {p.name: digest(p.read_bytes()) for p in sorted(wiki.glob("*.md"))}
            if inventory != expected["files"]:
                raise RuntimeError("canonical Markdown changed or disappeared during transfer")
            hits = call("knowledge", "retrieve", "--user-root", str(user), "--target", str(user), "--query", "anchor00000")
            if not hits.get("hits"):
                raise RuntimeError("representative restored knowledge is not searchable")
            report.update(producer_os=expected["producer_os"], consumer_os=platform.system(),
                          cross_os=expected["producer_os"] != platform.system(), imported=imported,
                          restored_files=len(inventory), canonical_bytes_equal=True, fts_query_verified=True)
        report["legacy_cli"] = args.legacy_cli
        report["peak_working_set_bytes"] = max((step["peak_working_set_bytes"] or 0) for step in report["steps"])
        report["status"] = "passed"
    except Exception as error:
        report.update(status="failed", error=str(error))
        raise
    finally:
        save()
    print(json.dumps({"status": report["status"], "receipt": str(work / "receipt.json")}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
