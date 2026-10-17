#!/bin/sh
# File-based Windows/MSYS adapter: do not pass nested shell code through PowerShell -c.
set -eu
[ "$#" -eq 5 ] || { echo "mode, native directory, installer, state and Python directory required" >&2; exit 2; }
mode=$1
case "$mode" in sandbox|global|rollback) ;; *) echo "unsupported developer install mode" >&2; exit 2 ;; esac
native_dir=$(cygpath -u "$2")
installer=$(cygpath -u "$3")
state_dir=$(cygpath -u "$4")
python_dir=$(cygpath -u "$5")
PATH="$native_dir:$python_dir:$PATH"
export PATH
exec sh "$installer" "--$mode" --state-dir "$state_dir"
