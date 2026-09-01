#!/usr/bin/env python3
"""Run the compiled Korean inspection gate over explicitly selected UTF-8 files."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hive-bin", type=Path, required=True)
    parser.add_argument("--profile", choices=("response", "release-note", "documentation", "technical", "verbatim"), required=True)
    parser.add_argument("--input", type=Path, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    files = []
    failed = False
    for path in sorted(args.input):
        process = subprocess.run(
            [
                str(args.hive_bin),
                "korean",
                "inspect",
                "--profile",
                args.profile,
                "--input",
                str(path),
                "--output",
                "json",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=20,
        )
        if process.returncode != 0:
            raise SystemExit(f"Korean inspection failed for {path}: {process.stderr.strip()}")
        result = json.loads(process.stdout)
        findings = result["data"]["findings"]
        severe = [finding["rule_id"] for finding in findings if finding["severity"] == "S1"]
        failed = failed or bool(severe)
        files.append(
            {
                "path": path.as_posix(),
                "digest": "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest(),
                "finding_ids": [finding["rule_id"] for finding in findings],
                "severe_ids": severe,
                "route_hint": result["data"]["route_hint"],
            }
        )
    receipt = {
        "schema_version": 1,
        "profile": args.profile,
        "status": "failed" if failed else "passed",
        "files": files,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(json.dumps(receipt, ensure_ascii=False, sort_keys=True))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
