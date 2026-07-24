#!/bin/sh
set -eu

version=${AIGENT_HIVE_VERSION:-}
prefix=${AIGENT_HIVE_PREFIX:-/usr/local}
authorized_team_id='__AIGENT_HIVE_APPLE_TEAM_ID__'

if [ -z "$version" ]; then
  echo "set AIGENT_HIVE_VERSION to an exact released X.Y.Z" >&2
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
if [ "$(uname -s)" != "Darwin" ]; then
  echo "this bootstrap supports macOS only; use install.ps1 on Windows" >&2
  exit 4
fi
case "$authorized_team_id" in
  __AIGENT_HIVE_*|*[!A-Z0-9]*|'')
    echo "installer does not contain an authorized macOS signer identity" >&2
    exit 5
    ;;
esac
if [ "${#authorized_team_id}" -ne 10 ]; then
  echo "installer does not contain an authorized macOS signer identity" >&2
  exit 5
fi
case "$(uname -m)" in
  arm64) triple=aarch64-apple-darwin ;;
  x86_64) triple=x86_64-apple-darwin ;;
  *)
    echo "unsupported macOS architecture" >&2
    exit 4
    ;;
esac

work=$(mktemp -d "${TMPDIR:-/tmp}/aigent-hive-install.XXXXXX")
staged_binary=
staged_receipt=
trap 'rm -rf "$work"; test -z "$staged_binary" || rm -f "$staged_binary"; test -z "$staged_receipt" || rm -f "$staged_receipt"' EXIT HUP INT TERM

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
  owned_digest=$(shasum -a 256 "$owned_binary" | awk '{ print $1 }')
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
      if [ "$(stat -f '%Lp' "$next_path")" != 755 ]; then
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
curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$work/$archive.sha256" "$base/$archive.sha256"

expected=$(awk 'NR == 1 { print $1 }' "$work/$archive.sha256")
actual=$(shasum -a 256 "$work/$archive" | awk '{ print $1 }')
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
codesign --verify --strict --verbose=2 "$binary"
spctl --assess --type execute --verbose=4 "$binary"
actual_team_id=$(codesign -dv --verbose=4 "$binary" 2>&1 \
  | awk -F= '$1 == "TeamIdentifier" { print $2 }')
if [ "$actual_team_id" != "$authorized_team_id" ]; then
  echo "signed binary signer differs from the authorized release identity" >&2
  exit 5
fi
if [ "$("$binary" --version)" != "hive $version" ]; then
  echo "signed binary version differs from requested release" >&2
  exit 5
fi
binary_digest=$(shasum -a 256 "$binary" | awk '{ print $1 }')

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
    && [ "$(shasum -a 256 "$prefix/bin/hive" | awk '{ print $1 }')" = "$pending_digest" ]; then
    if [ -e "$receipt" ] || [ -L "$receipt" ]; then
      [ -f "$receipt" ] && [ ! -L "$receipt" ] && parse_receipt "$receipt" || {
        echo "existing hive binary is not owned by the direct installer" >&2
        exit 3
      }
    fi
    ensure_safe_directory_chain "$prefix/bin"
    ensure_safe_directory_chain "$prefix/share/aigent-hive"
    mv -fh "$pending_receipt" "$receipt"
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
  if [ "$("$prefix/bin/hive" --version 2>/dev/null || true)" != "hive $prior_version" ]; then
    echo "existing hive binary is not owned by the direct installer" >&2
    exit 3
  fi
fi

ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
staged_binary=$(mktemp "$prefix/bin/.hive-install.XXXXXX")
staged_receipt=$(mktemp "$prefix/share/aigent-hive/.install-receipt.XXXXXX")
mv -fh "$binary" "$staged_binary"
chmod -h 0755 "$staged_binary"
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
chmod -h 0644 "$staged_receipt"
if [ ! -f "$staged_receipt" ] || [ -L "$staged_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
if [ "$(stat -f '%Lp' "$staged_receipt")" != 644 ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
if [ -e "$pending_receipt" ] || [ -L "$pending_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
mv -fh "$staged_receipt" "$pending_receipt"
staged_receipt=
if [ ! -f "$pending_receipt" ] || [ -L "$pending_receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
mv -fh "$staged_binary" "$prefix/bin/hive"
staged_binary=
if [ ! -f "$prefix/bin/hive" ] || [ -L "$prefix/bin/hive" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
ensure_safe_directory_chain "$prefix/bin"
ensure_safe_directory_chain "$prefix/share/aigent-hive"
mv -fh "$pending_receipt" "$receipt"
if [ ! -f "$receipt" ] || [ -L "$receipt" ]; then
  echo "existing hive install transaction is not recoverable" >&2
  exit 3
fi
echo "installed hive $version to $prefix/bin/hive"
