"""Deterministic Codex CLI fixture that forces the CodexBar fallback path."""

from __future__ import annotations

import sys


if sys.argv[1:] == ["--version"]:
    print("codex-cli 0.144.5")
    raise SystemExit(0)

raise SystemExit(64)
