#!/usr/bin/env python3
"""Inspect source test artifacts; cleanup is a preview unless --apply is set."""
from test_artifacts import cli

if __name__ == "__main__":
    raise SystemExit(cli())
