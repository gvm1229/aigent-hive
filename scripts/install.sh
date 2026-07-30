#!/bin/sh
set -eu

embedded_version='__AIGENT_HIVE_VERSION__'
version=${AIGENT_HIVE_VERSION:-$embedded_version}
prefix=${AIGENT_HIVE_PREFIX:-"${HOME:?HOME is required}/.local"}
authorized_team_id='__AIGENT_HIVE_APPLE_TEAM_ID__'
sha_aarch64_apple_darwin='__AIGENT_HIVE_SHA256_AARCH64_APPLE_DARWIN__'
sha_x86_64_apple_darwin='__AIGENT_HIVE_SHA256_X86_64_APPLE_DARWIN__'
sha_aarch64_linux_musl='__AIGENT_HIVE_SHA256_AARCH64_UNKNOWN_LINUX_MUSL__'
sha_x86_64_linux_musl='__AIGENT_HIVE_SHA256_X86_64_UNKNOWN_LINUX_MUSL__'

if [ -z "$version" ] || [ "$version" = "$embedded_version" ]; then
  echo "installer does not contain an exact released X.Y.Z version" >&2
  exit 2
fi
if ! printf '%s\n' "$version" | awk -F. '
  NF != 3 { exit 1 }
  {
    for (part = 1; part <= 3; part += 1) {
      if ($part !~ /^(0|[1-9][0-9]*)$/) {
        exit 1
      }
    }
  }
'; then
  echo "AIGENT_HIVE_VERSION must be exact X.Y.Z" >&2
  exit 2
fi
operating_system=$(uname -s)
case "$operating_system" in
  Darwin)
    case "$(uname -m)" in
      arm64)
        triple=aarch64-apple-darwin
        expected=$sha_aarch64_apple_darwin
        ;;
      x86_64)
        triple=x86_64-apple-darwin
        expected=$sha_x86_64_apple_darwin
        ;;
      *)
        echo "unsupported macOS architecture" >&2
        exit 4
        ;;
    esac
    ;;
  Linux)
    case "$(uname -m)" in
      arm64|aarch64)
        triple=aarch64-unknown-linux-musl
        expected=$sha_aarch64_linux_musl
        ;;
      x86_64|amd64)
        triple=x86_64-unknown-linux-musl
        expected=$sha_x86_64_linux_musl
        ;;
      *)
        echo "unsupported Linux architecture" >&2
        exit 4
        ;;
    esac
    ;;
  *)
    echo "this bootstrap supports macOS and Linux; use install.ps1 on Windows" >&2
    exit 4
    ;;
esac
case "$expected" in
  *[!0-9a-f]*|'')
    echo "installer does not contain the release archive SHA-256" >&2
    exit 5
    ;;
esac
if [ "${#expected}" -ne 64 ]; then
  echo "installer does not contain the release archive SHA-256" >&2
  exit 5
fi
case "$authorized_team_id" in
  __AIGENT_HIVE_*) authorized_team_id= ;;
  *[!A-Z0-9]*)
    echo "installer does not contain an authorized macOS signer identity" >&2
    exit 5
    ;;
esac
if [ -n "$authorized_team_id" ] && [ "${#authorized_team_id}" -ne 10 ]; then
  echo "installer does not contain an authorized macOS signer identity" >&2
  exit 5
fi

work=$(mktemp -d "${TMPDIR:-/tmp}/aigent-hive-install.XXXXXX")
staged_binary=
staged_receipt=
trap 'rm -rf "$work"; test -z "$staged_binary" || rm -f "$staged_binary"; test -z "$staged_receipt" || rm -f "$staged_receipt"' EXIT HUP INT TERM

matches_hive_version() {
  hive_version_output=$1
  hive_expected_version=$2
  hive_version_prefix="hive $hive_expected_version (released "
  case "$hive_version_output" in
    "$hive_version_prefix"????-??-??")") ;;
    *) return 1 ;;
  esac
  hive_release_date=${hive_version_output#"$hive_version_prefix"}
  hive_release_date=${hive_release_date%)}
  printf '%s\n' "$hive_release_date" | awk -F- '
    NF == 3 &&
    $1 ~ /^[0-9][0-9][0-9][0-9]$/ &&
    $2 ~ /^[0-9][0-9]$/ &&
    $3 ~ /^[0-9][0-9]$/ &&
    $2 >= 1 && $2 <= 12 &&
    $3 >= 1 && $3 <= 31 { exit 0 }
    { exit 1 }
  '
}

sha256_file() {
  if [ "$operating_system" = Darwin ]; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}

directory_mode() {
  if [ "$operating_system" = Darwin ]; then
    stat -f '%Lp' "$1"
  else
    stat -c '%a' "$1"
  fi
}

set_file_mode() {
  mode=$1
  target=$2
  [ -f "$target" ] && [ ! -L "$target" ] || return 1
  if [ "$operating_system" = Darwin ]; then
    chmod -h "$mode" "$target"
  else
    chmod "$mode" "$target"
  fi
  [ -f "$target" ] && [ ! -L "$target" ]
}

move_file() {
  source=$1
  destination=$2
  if [ "$operating_system" = Darwin ]; then
    mv -fh "$source" "$destination"
  else
    mv -fT -- "$source" "$destination"
  fi
}

parse_receipt() {
  receipt_path=$1
  receipt_json=$(cat "$receipt_path")
  receipt_prefix='{"schema_version":1,"owner":"direct","product":"aigent-hive","version":"'
  receipt_digest_marker='","artifact_sha256":"sha256:'
  case "$receipt_json" in
    "$receipt_prefix"*"$receipt_digest_marker"*'"}') ;;
    *) return 1 ;;
  esac
  parsed_version=${receipt_json#"$receipt_prefix"}
  parsed_version=${parsed_version%%"$receipt_digest_marker"*}
  parsed_digest=${receipt_json#*"$receipt_digest_marker"}
  parsed_digest=${parsed_digest%\}}
  parsed_digest=${parsed_digest%\"}
  if ! printf '%s\n' "$parsed_version" | awk -F. '
    NF != 3 { exit 1 }
    {
      for (part = 1; part <= 3; part += 1) {
        if ($part !~ /^(0|[1-9][0-9]*)$/) {
          exit 1
        }
      }
    }
  '; then
    return 1
  fi
  case "$parsed_digest" in
    *[!0-9a-f]*|'') return 1 ;;
  esac
  [ "${#parsed_digest}" -eq 64 ] || return 1
  expected_receipt=$(printf '{"schema_version":1,"owner":"direct","product":"aigent-hive","version":"%s","artifact_sha256":"sha256:%s"}' \
    "$parsed_version" "$parsed_digest")
  [ "$receipt_json" = "$expected_receipt" ]
}

verify_owned_pair() {
  owned_binary=$1
  owned_receipt=$2
  [ -f "$owned_binary" ] && [ ! -L "$owned_binary" ] \
    && [ -f "$owned_receipt" ] && [ ! -L "$owned_receipt" ] \
    || return 1
  parse_receipt "$owned_receipt" || return 1
  owned_digest=$(sha256_file "$owned_binary")
  [ "$parsed_digest" = "$owned_digest" ]
}

ensure_safe_directory_chain() {
  target_path=$1
  case "$target_path" in
    /*)
      current_path=/
      remaining_path=${target_path#/}
      ;;
    *)
      current_path=.
      remaining_path=$target_path
      ;;
  esac
  while [ -n "$remaining_path" ]; do
    path_component=${remaining_path%%/*}
    if [ "$remaining_path" = "$path_component" ]; then
      remaining_path=
    else
      remaining_path=${remaining_path#*/}
    fi
    case "$path_component" in
      ''|.) continue ;;
      ..)
        echo "install path contains a symlink or non-directory" >&2
        exit 3
        ;;
    esac
    if [ "$current_path" = / ]; then
      next_path="/$path_component"
    else
      next_path="$current_path/$path_component"
    fi
    if [ -L "$next_path" ]; then
      echo "install path contains a symlink or non-directory" >&2
      exit 3
    elif [ -e "$next_path" ]; then
      if [ ! -d "$next_path" ]; then
        echo "install path contains a symlink or non-directory" >&2
        exit 3
      fi
    else
      mkdir -m 0755 "$next_path"
      if [ "$(directory_mode "$next_path")" != 755 ]; then
        echo "install path contains a symlink or non-directory" >&2
        exit 3
      fi
    fi
    if [ ! -d "$next_path" ] || [ -L "$next_path" ]; then
      echo "install path contains a symlink or non-directory" >&2
      exit 3
    fi
    current_path=$next_path
  done
}

archive="aigent-hive-${version}-${triple}.tar.gz"
package="aigent-hive-${version}-${triple}"
base="https://github.com/gvm1229/aigent-hive/releases/download/v${version}"
curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$work/$archive" "$base/$archive"

actual=$(sha256_file "$work/$archive")
if [ "${#expected}" -ne 64 ] || [ "$expected" != "$actual" ]; then
  echo "release archive SHA-256 verification failed" >&2
  exit 5
fi
if [ "$(tar -tzf "$work/$archive")" != "$(printf '%s\n%s' "$package/hive" "$package/LICENSE")" ]; then
  echo "release archive contains an unexpected path" >&2
  exit 5
fi
if ! tar -tvzf "$work/$archive" | awk 'substr($1, 1, 1) != "-" { exit 1 }'; then
  echo "release archive contains a nonregular entry" >&2
  exit 5
fi
tar -xzf "$work/$archive" -C "$work"
binary="$work/$package/hive"
if [ "$operating_system" = Darwin ] && [ -n "$authorized_team_id" ]; then
  codesign --verify --strict --verbose=2 "$binary"
  spctl --assess --type execute --verbose=4 "$binary"
  actual_team_id=$(codesign -dv --verbose=4 "$binary" 2>&1 \
    | awk -F= '$1 == "TeamIdentifier" { print $2 }')
  if [ "$actual_team_id" != "$authorized_team_id" ]; then
    echo "signed binary signer differs from the authorized release identity" >&2
    exit 5
  fi
fi
if ! matches_hive_version "$("$binary" --version)" "$version"; then
  echo "signed binary version differs from requested release" >&2
  exit 5
fi
binary_digest=$(sha256_file "$binary")

ensure_safe_directory_chain "$prefix"
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
receipt="$prefix/share/aigent-hive/install-receipt.json"
pending_receipt="$prefix/share/aigent-hive/install-receipt.pending.json"
if [ -e "$pending_receipt" ] || [ -L "$pending_receipt" ]; then
  if [ ! -f "$pending_receipt" ] || [ -L "$pending_receipt" ] \
    || ! parse_receipt "$pending_receipt"; then
    echo "existing hive binary is not owned by the direct installer" >&2
    exit 3
  fi
  pending_digest=$parsed_digest
  if [ -f "$prefix/bin/hive" ] && [ ! -L "$prefix/bin/hive" ] \
    && [ "$(sha256_file "$prefix/bin/hive")" = "$pending_digest" ]; then
    if [ -e "$receipt" ] || [ -L "$receipt" ]; then
      [ -f "$receipt" ] && [ ! -L "$receipt" ] && parse_receipt "$receipt" || {
        echo "existing hive binary is not owned by the direct installer" >&2
        exit 3
      }
    fi
    ensure_safe_directory_chain "$prefix/bin"
    ensure_safe_directory_chain "$prefix/share/aigent-hive"
    move_file "$pending_receipt" "$receipt"
    if [ ! -f "$receipt" ] || [ -L "$receipt" ]; then
      echo "existing hive install transaction is not recoverable" >&2
      exit 3
    fi
  elif verify_owned_pair "$prefix/bin/hive" "$receipt"; then
    ensure_safe_directory_chain "$prefix/bin"
    ensure_safe_directory_chain "$prefix/share/aigent-hive"
    rm -f "$pending_receipt"
  elif [ ! -e "$prefix/bin/hive" ] && [ ! -L "$prefix/bin/hive" ] \
    && [ ! -e "$receipt" ] && [ ! -L "$receipt" ]; then
    ensure_safe_directory_chain "$prefix/bin"
    ensure_safe_directory_chain "$prefix/share/aigent-hive"
    rm -f "$pending_receipt"
  else
    echo "existing hive binary is not owned by the direct installer" >&2
    exit 3
  fi
fi

if [ -e "$prefix/bin/hive" ] || [ -L "$prefix/bin/hive" ] \
  || [ -e "$receipt" ] || [ -L "$receipt" ]; then
  if ! verify_owned_pair "$prefix/bin/hive" "$receipt"; then
    echo "existing hive binary is not owned by the direct installer" >&2
    exit 3
  fi
  prior_version=$parsed_version
  if ! matches_hive_version \
    "$("$prefix/bin/hive" --version 2>/dev/null || true)" \
    "$prior_version"; then
    echo "existing hive binary is not owned by the direct installer" >&2
    exit 3
  fi
fi

ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
staged_binary=$(mktemp "$prefix/bin/.hive-install.XXXXXX")
staged_receipt=$(mktemp "$prefix/share/aigent-hive/.install-receipt.XXXXXX")
move_file "$binary" "$staged_binary"
set_file_mode 0755 "$staged_binary"
if [ ! -f "$staged_binary" ] || [ -L "$staged_binary" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
rm -f "$staged_receipt"
prior_umask=$(umask)
umask 077
set -C
exec 3>"$staged_receipt"
set +C
umask "$prior_umask"
printf '{"schema_version":1,"owner":"direct","product":"aigent-hive","version":"%s","artifact_sha256":"sha256:%s"}\n' \
  "$version" "$binary_digest" >&3
exec 3>&-
set_file_mode 0644 "$staged_receipt"
if [ ! -f "$staged_receipt" ] || [ -L "$staged_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
if [ "$(directory_mode "$staged_receipt")" != 644 ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
if [ -e "$pending_receipt" ] || [ -L "$pending_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
move_file "$staged_receipt" "$pending_receipt"
staged_receipt=
if [ ! -f "$pending_receipt" ] || [ -L "$pending_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
move_file "$staged_binary" "$prefix/bin/hive"
staged_binary=
if [ ! -f "$prefix/bin/hive" ] || [ -L "$prefix/bin/hive" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
move_file "$pending_receipt" "$receipt"
if [ ! -f "$receipt" ] || [ -L "$receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
echo "installed hive $version to $prefix/bin/hive"
