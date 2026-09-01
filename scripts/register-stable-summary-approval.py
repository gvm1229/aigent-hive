#!/usr/bin/env python3
"""Register an already approved subscriber digest without publishing or sending anything."""

from __future__ import annotations

import argparse
import json
import runpy
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NOTIFIER = runpy.run_path(str(ROOT / "scripts/publish-stable-discord-update.py"))
NotificationError = NOTIFIER["NotificationError"]
REPOSITORY = "gvm1229/aigent-hive"
ENVIRONMENT = "release-publication"
SECRET_NAME = "AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST"


def register_approval(version: str, approved_digest: str, root: Path = ROOT) -> dict[str, str]:
    # The caller supplies the digest from explicit wording approval, never from a changed draft.
    if NOTIFIER["STABLE_VERSION"].fullmatch(version) is None:
        raise NotificationError("approval registration requires a stable X.Y.Z version")
    if NOTIFIER["SUMMARY_DIGEST"].fullmatch(approved_digest) is None:
        raise NotificationError("an explicitly approved sha256 digest is required")
    summary = root / "docs/releases" / f"{version}.subscriber.ko.md"
    approval = summary.with_suffix(".sha256")
    NOTIFIER["read_summary"](summary, version, approval, approved_digest)
    try:
        result = subprocess.run(
            ["gh", "secret", "set", SECRET_NAME, "--repo", REPOSITORY, "--env", ENVIRONMENT],
            input=approved_digest,
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        raise NotificationError(
            "approval registration could not be confirmed; retry the same approved digest"
        ) from None
    if result.returncode != 0:
        raise NotificationError(
            "approval registration failed; check gh authentication and environment secret access"
        )
    return {
        "status": "registered",
        "product_version": version,
        "approved_digest": approved_digest,
        "repository": REPOSITORY,
        "environment": ENVIRONMENT,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--approved-digest", required=True)
    arguments = parser.parse_args()
    try:
        receipt = register_approval(arguments.product_version, arguments.approved_digest)
    except NotificationError as error:
        print(f"subscriber approval registration error: {error}", file=sys.stderr)
        return 1
    print(json.dumps(receipt))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
