#!/usr/bin/env python3
"""Produce or consume the SAME synthetic knowledge archive across CI operating systems."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import platform
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
        result = subprocess.run([str(binary), *argv, "--output", "json"],
                                capture_output=True, encoding="utf-8", timeout=600, check=False)
        value = json.loads(result.stdout)
        report["steps"].append({"command": list(argv), "seconds": time.perf_counter() - start,
                                "exit_code": result.returncode})
        if result.returncode:
            raise RuntimeError(f"{argv[:3]}: {value.get('message')}")
        return value["data"]

    def save() -> None:
        (work / "receipt.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    try:
        user = work / "사용자 지식 with spaces"
        config = user / ".hive/config/user-setup.yml"
        config.parent.mkdir(parents=True)
        config.write_text(json.dumps({
            "schema_version": 1, "interface_language": "en",
            "wiki": {"enabled": True, "language": "both"},
            "profile": {"contexts": ["web-developer"]}, "persona": {"id": "balanced"},
            "selected_hosts": ["codex", "claude", "antigravity"], "skills": {"mode": "all"},
            "usage_guard": {"enabled": True, "stop_remaining_percent": 20,
                            "codexbar_fallback_enabled": True, "discord": {"enabled": False}},
        }), encoding="utf-8", newline="\n")
        wiki = user / ".hive/knowledge/Wiki"
        wiki.mkdir(parents=True)
        if args.mode == "produce":
            for index in range(args.files):
                page_id = f"transfer-{index:05d}"
                text = (f"---\nschema_version: 1\nid: {page_id}\nkind: concept\nsummary: 이동 지식 {index}\n"
                        "tags: [transfer]\naliases: []\nsources: []\nlinks: []\ncontradictions: []\nstatus: active\n"
                        "created_at: '2026-08-01T00:00:00Z'\nupdated_at: '2026-08-01T00:00:00Z'\n---\n\n"
                        f"Portable knowledge anchor{index:05d}. 한글 원문·수치 {index} 보존.\n")
                (wiki / f"{page_id}.md").write_text(text, encoding="utf-8", newline="\n")
            call("knowledge", "refresh", "--user-root", str(user))
            archive = work / "portable.hivekb"
            exported = call("knowledge", "transfer", "export", "--apply", "--user-root", str(user),
                            "--scope", "all-portable", "--bundle", str(archive))
            inventory = {p.name: digest(p.read_bytes()) for p in sorted(wiki.glob("*.md"))}
            expected = {"producer_os": platform.system(), "files": inventory,
                        "archive_sha256": exported["archive_sha256"]}
            (work / "expected.json").write_text(json.dumps(expected, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            report["export"] = exported
        else:
            if not args.input:
                parser.error("consume requires --input containing portable.hivekb and expected.json")
            expected = json.loads((args.input / "expected.json").read_text(encoding="utf-8"))
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
