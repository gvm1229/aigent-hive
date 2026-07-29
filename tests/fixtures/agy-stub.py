#!/usr/bin/env python3
"""Stateful Antigravity CLI fixture for native release qualification."""

from __future__ import annotations

import json
import os
import shutil
import sys
from pathlib import Path


def read_state(path: Path) -> dict[str, object]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {"plugin": False}


def write_state(path: Path, state: dict[str, object]) -> None:
    path.write_text(json.dumps(state, sort_keys=True), encoding="utf-8")


def main() -> int:
    arguments = sys.argv[1:]
    if arguments == ["--version"]:
        print("1.1.7")
        return 0

    state_path = Path(f"{os.environ['HIVE_TEST_HOST_LOG']}.antigravity.state")
    state = read_state(state_path)
    command = " ".join(arguments)

    if command.startswith("plugin install "):
        source = Path(arguments[2]).resolve()
        hive_root = next(parent for parent in source.parents if parent.name == ".hive")
        stage = hive_root.parent / ".gemini/config/plugins/aigent-hive"
        if stage.exists():
            shutil.rmtree(stage)
        shutil.copytree(source, stage)
        state["plugin"] = True
        state["stage"] = str(stage)
        write_state(state_path, state)
        print("{}")
        return 0

    if command == "plugin uninstall aigent-hive":
        stage = Path(str(state.get("stage", "")))
        if state.get("stage") and stage.exists():
            shutil.rmtree(stage)
        state["plugin"] = False
        write_state(state_path, state)
        print("{}")
        return 0

    if command == "plugin list":
        if state.get("plugin"):
            print(
                json.dumps(
                    {
                        "imports": [
                            {
                                "name": "aigent-hive",
                                "source": "antigravity",
                                "importedAt": "2026-07-27T00:00:00Z",
                                "components": ["skills"],
                            }
                        ]
                    }
                )
            )
        else:
            print("No imported plugins.")
        return 0

    print("{}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
