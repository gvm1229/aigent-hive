#!/bin/sh
# Build a local developer Hive binary without changing canonical user data.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
mode=sandbox
state_dir="${HOME:?HOME is required}/.hive/dev-install"

usage() {
  cat <<'EOF'
Usage: scripts/dev-install.sh [--sandbox|--global|--rollback] [--state-dir DIR]

  --sandbox   Build a local developer binary only (default).
  --global    Replace the active Hive executable after backing it up.
  --rollback  Restore the executable saved by the last --global activation.

Developer builds report `AIgent Hive vX.Y.Z-dev · local developer build`, never a
public `-test` release identity.

This script never initializes, deletes, or migrates ~/.hive/config, ~/.hive/knowledge,
~/.hive/index, project .hive directories, or user-managed directives and Skills.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --sandbox) mode=sandbox ;;
    --global) mode=global ;;
    --rollback) mode=rollback ;;
    --state-dir)
      shift
      [ "$#" -gt 0 ] || { echo "--state-dir requires a directory" >&2; exit 2; }
      state_dir=$1
      ;;
    --help|-h) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

case "$state_dir" in
  /*) ;;
  *) echo "--state-dir must be absolute" >&2; exit 2 ;;
esac

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

canonical_path() {
  python3 - "$1" <<'PY'
import os
import sys
print(os.path.realpath(sys.argv[1]))
PY
}

product_version=$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")
case "$product_version" in
  ''|*[!0-9.]*|*.*.*.*) echo "Cargo.toml does not contain an exact product version" >&2; exit 2 ;;
esac
package_version="${AIGENT_HIVE_PACKAGE_VERSION:-$product_version-dev}"
release_date="${AIGENT_HIVE_PACKAGE_RELEASE_DATE:-$(date +%F)}"
binary="$root/target/release/hive"

resolve_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    command -v cargo
    return 0
  fi
  cargo_fallback="${HOME:?HOME is required}/.cargo/bin/cargo"
  if [ -x "$cargo_fallback" ]; then
    printf '%s\n' "$cargo_fallback"
    return 0
  fi
  for cargo_fallback in "${HOME:?HOME is required}"/.rustup/toolchains/stable-*/bin/cargo; do
    if [ -x "$cargo_fallback" ]; then
      printf '%s\n' "$cargo_fallback"
      return 0
    fi
  done
  echo "cargo was not found; install Rust with rustup before running dev-install" >&2
  exit 4
}

build() {
  cargo_command=$(resolve_cargo)
  (
    cd "$root"
    PATH="$(dirname "$cargo_command"):$PATH"
    export PATH
    AIGENT_HIVE_PACKAGE_VERSION="$package_version" \
      AIGENT_HIVE_PACKAGE_RELEASE_DATE="$release_date" \
      "$cargo_command" build --locked --release -p hive-cli
  )
  [ -f "$binary" ] && [ ! -L "$binary" ] || {
    echo "developer build did not produce a regular hive binary" >&2
    exit 5
  }
}

rollback() {
  target_file="$state_dir/target-path"
  original_file="$state_dir/original"
  digest_file="$state_dir/developer-sha256"
  [ -f "$target_file" ] && [ -f "$original_file" ] && [ -f "$digest_file" ] || {
    echo "no recoverable developer activation exists at $state_dir" >&2
    exit 3
  }
  target=$(cat "$target_file")
  expected=$(cat "$digest_file")
  [ -f "$target" ] && [ ! -L "$target" ] || {
    echo "active Hive target is missing or no longer a regular file: $target" >&2
    exit 3
  }
  [ "$(sha256_file "$target")" = "$expected" ] || {
    echo "active Hive target changed after developer activation; refusing rollback" >&2
    exit 3
  }
  staged="$target.hive-rollback-$$"
  trap 'rm -f "$staged"' EXIT HUP INT TERM
  cp -p "$original_file" "$staged"
  mv -f "$staged" "$target"
  rm -f "$target_file" "$original_file" "$digest_file"
  rmdir "$state_dir" 2>/dev/null || true
  trap - EXIT HUP INT TERM
  echo "restored Hive executable: $target"
}

case "$mode" in
  rollback)
    rollback
    exit 0
    ;;
  sandbox)
    build
    echo "developer Hive binary: $binary"
    echo "canonical user data unchanged"
    exit 0
    ;;
  global) ;;
esac

build
active_command=$(command -v hive || true)
[ -n "$active_command" ] || {
  echo "no active hive command found; install a stable or test release before --global" >&2
  exit 3
}
target=$(canonical_path "$active_command")
[ -f "$target" ] && [ ! -L "$target" ] || {
  echo "active hive command does not resolve to a regular file: $target" >&2
  exit 3
}
"$active_command" --version | grep -q '^AIgent Hive v' || {
  echo "active command is not an Aigent Hive executable: $active_command" >&2
  exit 3
}
[ ! -e "$state_dir" ] || {
  echo "a developer activation already exists at $state_dir; use --rollback first" >&2
  exit 3
}
mkdir -p "$state_dir"
[ ! -L "$state_dir" ] || { echo "developer state directory must not be a symlink" >&2; exit 3; }
cp -p "$target" "$state_dir/original"
printf '%s\n' "$target" >"$state_dir/target-path"
developer_digest=$(sha256_file "$binary")
printf '%s\n' "$developer_digest" >"$state_dir/developer-sha256"
staged="$target.hive-dev-$$"
trap 'rm -f "$staged"' EXIT HUP INT TERM
cp -p "$binary" "$staged"
mv -f "$staged" "$target"
trap - EXIT HUP INT TERM
"$active_command" --version
echo "developer build activated: $target"
echo "canonical user data unchanged; use hive install separately to preview any projection update"
