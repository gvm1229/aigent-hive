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
  --global    Refused: use a public numbered test package for a real installation.
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

if [ "$mode" = global ]; then
  echo "global developer replacement is disabled: it breaks package ownership and the stable upgrade path" >&2
  echo "use --sandbox with an isolated --user-root, or install a public numbered test package" >&2
  exit 3
fi

case "$state_dir" in
  /*) ;;
  *) echo "--state-dir must be absolute" >&2; exit 2 ;;
esac

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 < "$1" | awk '{print $1}'
  else
    sha256sum < "$1" | awk '{print $1}'
  fi
}

product_version=$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")
case "$product_version" in
  ''|*[!0-9.]*|*.*.*.*) echo "Cargo.toml does not contain an exact product version" >&2; exit 2 ;;
esac
package_version="${AIGENT_HIVE_PACKAGE_VERSION:-$product_version-dev}"
release_date="${AIGENT_HIVE_PACKAGE_RELEASE_DATE:-$(date +%F)}"
binary="${AIGENT_HIVE_DEV_BINARY:-$root/target/release/hive}"

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
  if [ -n "${AIGENT_HIVE_DEV_BINARY:-}" ]; then
    [ -f "$binary" ] && [ ! -L "$binary" ] && [ -x "$binary" ] || {
      echo "AIGENT_HIVE_DEV_BINARY must name an executable regular file" >&2
      exit 5
    }
    return 0
  fi
  cargo_command=$(resolve_cargo)
  (
    cd "$root"
    PATH="$(dirname "$cargo_command"):$PATH"
    export PATH
    AIGENT_HIVE_PACKAGE_VERSION="$package_version" \
      AIGENT_HIVE_PACKAGE_RELEASE_DATE="$release_date" \
      "$cargo_command" build --locked --release -p hive-cli
  )
  [ -f "$binary" ] && [ ! -L "$binary" ] && [ -x "$binary" ] || {
    echo "developer build did not produce an executable regular hive binary" >&2
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
  # Older GNU output escaped a native Windows filename with one leading backslash.
  case "$expected" in
    \\*) expected=${expected#?} ;;
  esac
  case "$expected" in
    ''|*[!0-9a-f]*) echo "invalid developer digest" >&2; exit 3 ;;
  esac
  [ "${#expected}" -eq 64 ] || { echo "invalid developer digest" >&2; exit 3; }
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
esac
