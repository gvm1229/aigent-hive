#!/usr/bin/env python3
"""Build a canonical Graphify wheelhouse digest lock without copying wheels."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wheelhouse", type=Path, required=True)
    parser.add_argument("--platform", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    files = []
    for path in sorted(args.wheelhouse.glob("*.whl"), key=lambda item: item.name.lower()):
        content = path.read_bytes()
        files.append(
            {
                "filename": path.name,
                "sha256": hashlib.sha256(content).hexdigest(),
                "size": len(content),
            }
        )
    if len(files) != 30 or not any(item["filename"].startswith("graphifyy-0.9.47-") for item in files):
        raise SystemExit("wheelhouse must contain the exact 30-package Graphify 0.9.47 closure")
    payload = {
        "schema_version": 1,
        "package": "graphifyy==0.9.47",
        "platform": args.platform,
        "python": "3.12",
        "files": files,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":"), sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
