#!/usr/bin/env python3
"""Allow automatic numbered tests only for completed, new product bytes."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/public-test-product.json"
STABLE = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
PLAN_ID = re.compile(r"^[A-Z][A-Z0-9]*-[0-9]{3}$")
PRODUCT_PREFIXES = ("crates/", "harness/", "schemas/", "packaging/", "vendor/", "LICENSES/")
PRODUCT_FILES = {
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "copier.yml", "LICENSE", "REUSE.toml",
}
PRODUCT_SCRIPT_PREFIXES = (
    "scripts/install.", "scripts/package-npm.", "scripts/render-installers.",
)


class GateError(ValueError):
    pass


def git(*arguments: str) -> str:
    result = subprocess.run(
        ["git", *arguments], cwd=ROOT, check=False, capture_output=True, text=True, timeout=30
    )
    if result.returncode != 0:
        raise GateError("required Git evidence is unavailable")
    return result.stdout


def is_product_path(path: str) -> bool:
    return (
        path in PRODUCT_FILES
        or path.startswith(PRODUCT_PREFIXES)
        or path.startswith(PRODUCT_SCRIPT_PREFIXES)
    )


def product_entries(ref: str) -> list[str]:
    rows = []
    for line in git("ls-tree", "-r", ref).splitlines():
        metadata, path = line.split("\t", 1)
        if is_product_path(path):
            rows.append(f"{metadata} {path}")
    return sorted(rows)


def product_digest(ref: str) -> str:
    payload = "\n".join(product_entries(ref)).encode("utf-8")
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def changed_product_paths(base: str, head: str) -> list[str]:
    return sorted(
        path
        for path in git("diff", "--name-only", f"{base}..{head}").splitlines()
        if is_product_path(path)
    )


def checked_plan_ids() -> set[str]:
    checked: set[str] = set()
    for path in (ROOT / "docs/plans/active").glob("*.md"):
        checked.update(re.findall(r"^- \[x\] \[([A-Z][A-Z0-9]*-[0-9]{3})\]", path.read_text(encoding="utf-8"), re.MULTILINE))
    return checked


def read_registry() -> dict[str, object]:
    try:
        value = json.loads(REGISTRY.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise GateError("accepted public-test product registry is unavailable") from error
    required = {"schema_version", "product_version", "accepted_package_version", "accepted_source_commit", "product_tree_sha256"}
    if set(value) != required or value["schema_version"] != 1:
        raise GateError("accepted public-test product registry has an unsupported shape")
    return value


def verify(product_version: str, package_version: str, plan_ids: str, head: str) -> dict[str, object]:
    if STABLE.fullmatch(product_version) is None:
        raise GateError("product version must be stable X.Y.Z")
    if re.fullmatch(re.escape(product_version) + r"-test\.[1-9][0-9]*", package_version) is None:
        raise GateError("package version must be a uniquely numbered public test")
    requested = [item for item in plan_ids.split(",") if item]
    if not requested or len(requested) != len(set(requested)) or any(PLAN_ID.fullmatch(item) is None for item in requested):
        raise GateError("one or more unique implementation plan IDs are required")
    if any(item.startswith("REL") for item in requested):
        raise GateError("release mechanics cannot authorize product changes")
    missing = sorted(set(requested) - checked_plan_ids())
    if missing:
        raise GateError("product change plan IDs are absent or incomplete: " + ",".join(missing))
    registry = read_registry()
    if registry["product_version"] != product_version:
        raise GateError("accepted product registry belongs to another product version")
    base = str(registry["accepted_source_commit"])
    prior_digest = product_digest(base)
    if prior_digest != registry["product_tree_sha256"]:
        raise GateError("accepted product registry digest is stale")
    current_digest = product_digest(head)
    paths = changed_product_paths(base, head)
    if not paths or current_digest == prior_digest:
        raise GateError("numbered test refused because shipped product bytes did not change")
    return {
        "schema_version": 1,
        "status": "authorized",
        "product_version": product_version,
        "package_version": package_version,
        "base_package_version": registry["accepted_package_version"],
        "base_source_commit": base,
        "head": git("rev-parse", head).strip(),
        "prior_product_digest": prior_digest,
        "product_digest": current_digest,
        "product_paths": paths,
        "plan_ids": requested,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--plan-ids", required=True)
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--output", choices=("json",), default="json")
    args = parser.parse_args()
    try:
        result = verify(args.product_version, args.package_version, args.plan_ids, args.head)
    except GateError as error:
        print(json.dumps({"schema_version": 1, "status": "refused", "message": str(error)}))
        return 1
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
