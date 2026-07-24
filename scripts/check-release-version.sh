#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: check-release-version.sh X.Y.Z" >&2
  exit 2
fi

requested=$1
if ! printf '%s\n' "$requested" | awk -F. '
  NF != 3 { exit 1 }
  {
    for (part = 1; part <= 3; part += 1) {
      if ($part !~ /^(0|[1-9][0-9]*)$/) {
        exit 1
      }
    }
  }
'; then
  echo "release version must be exact X.Y.Z" >&2
  exit 2
fi

workspace=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1)
if [ -z "$workspace" ] || [ "$workspace" != "$requested" ]; then
  echo "requested release $requested differs from workspace $workspace" >&2
  exit 3
fi

compiled=$(cargo run --quiet --locked -p hive-cli -- --version)
if [ "$compiled" != "hive $requested" ]; then
  echo "compiled CLI version differs: $compiled" >&2
  exit 3
fi

template=$(sed -n 's/^harness_version = "\([^"]*\)"$/\1/p' \
  harness/template/.hive/config/harness.toml.jinja)
source_release=$(sed -n 's/^source_release_version = "\([^"]*\)"$/\1/p' \
  harness/template/.hive/config/harness.toml.jinja)
if [ "$template" != "$requested" ] || [ "$source_release" != "$requested" ]; then
  echo "installed harness template version differs from $requested" >&2
  exit 3
fi

python_command=python3
if ! command -v "$python_command" >/dev/null 2>&1; then
  python_command=python
fi
"$python_command" - "$requested" <<'PY'
import json
import re
import sys
import tomllib
from pathlib import Path

requested = sys.argv[1]
root = Path(".")

lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
for package in lock["package"]:
    if package["name"].startswith("hive-") and package["version"] != requested:
        raise SystemExit(
            f"Cargo.lock package {package['name']} differs from {requested}"
        )

fixture = root / f"tests/fixtures/phase6/releases/valid-{requested}/targets"
manifest = json.loads(
    (fixture / "bundle-manifest.json").read_text(encoding="utf-8")
)
migration = json.loads(
    (fixture / "migration-table.json").read_text(encoding="utf-8")
)
if manifest["release_version"] != requested:
    raise SystemExit("release bundle manifest version differs")
if manifest["source"]["tag"] != f"v{requested}":
    raise SystemExit("release bundle source tag differs")
if migration["target_version"] != requested:
    raise SystemExit("migration table target version differs")

readme = (root / "README.md").read_text(encoding="utf-8")
readme_match = re.search(
    r"(?m)^\| Product version \| `([^`]+)` \|$",
    readme,
)
if readme_match is None or readme_match.group(1) != requested:
    raise SystemExit("README product version differs")

current = (root / "docs/state/CURRENT.md").read_text(encoding="utf-8")
match = re.search(r"(?m)^- product version: `([^`]+)`$", current)
if match is None or match.group(1) != requested:
    raise SystemExit("CURRENT.md product version differs")
PY
