#!/usr/bin/env python3
"""Evaluate a pinned Graphify candidate without changing the product dependency."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import venv

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.9.76"
WHEEL = "graphifyy-0.9.76-py3-none-any.whl"
WHEEL_SHA = "68e09bc159628c4cc0d62ce76786993037266b74f3dbce3a40f498ed7cd9f129"


def run(argv: list[str], cwd: Path, env: dict[str, str], timeout: int = 120) -> dict:
    started = time.monotonic()
    try:
        result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, timeout=timeout)
        return {"exit_code": result.returncode, "elapsed_seconds": round(time.monotonic()-started, 3),
                "stdout": result.stdout, "stderr": result.stderr}
    except subprocess.TimeoutExpired:
        return {"exit_code": None, "elapsed_seconds": round(time.monotonic()-started, 3),
                "stdout": b"", "stderr": b"", "timeout": True}


def receipt(result: dict) -> dict:
    return {key: value for key, value in result.items() if key not in {"stdout", "stderr"}} | {
        "stdout_bytes": len(result["stdout"]), "stderr_bytes": len(result["stderr"]),
        "stdout_sha256": hashlib.sha256(result["stdout"]).hexdigest(),
        "stderr_sha256": hashlib.sha256(result["stderr"]).hexdigest()}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    owned = (ROOT / "tests/work").resolve()
    if work == owned or not work.is_relative_to(owned):
        raise SystemExit("candidate work must be an exact child of tests/work")
    work.mkdir(parents=True, exist_ok=True)
    home = work / "home"
    home.mkdir(exist_ok=True)
    wheels = work / "wheels"
    wheels.mkdir(exist_ok=True)
    env = {key: value for key, value in os.environ.items() if key in {"SYSTEMROOT", "WINDIR", "COMSPEC", "PATH"}}
    env.update({"USERPROFILE": str(home), "LOCALAPPDATA": str(home / "local"),
                "APPDATA": str(home / "roaming"), "PIP_CONFIG_FILE": os.devnull,
                "PIP_CACHE_DIR": str(work / "pip-cache"), "PIP_DISABLE_PIP_VERSION_CHECK": "1",
                "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1", "PYTHONHASHSEED": "0",
                "GRAPHIFY_QUERY_LOG_DISABLE": "1", "GIT_CONFIG_NOSYSTEM": "1",
                "GIT_CONFIG_GLOBAL": os.devnull, "GIT_TERMINAL_PROMPT": "0"})
    report = {"schema_version": 1, "candidate": VERSION, "host": sys.platform,
              "python": sys.version.split()[0], "product_dependency_changed": False,
              "decision": "retain-0.9.47", "steps": {}}

    def finish(reason: str, *, error: bool = False) -> int:
        report["reason"] = reason
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
        print(json.dumps({"decision": report["decision"], "reason": reason, "candidate": VERSION}))
        return int(error)

    download = run([sys.executable, "-m", "pip", "download", "--only-binary=:all:",
                    "--dest", str(wheels), f"graphifyy=={VERSION}"], work, env, 180)
    report["steps"]["download"] = receipt(download)
    if download["exit_code"] != 0:
        return finish("candidate dependency download not qualified", error=True)
    wheel = wheels / WHEEL
    if not wheel.is_file() or hashlib.sha256(wheel.read_bytes()).hexdigest() != WHEEL_SHA:
        return finish("candidate wheel identity mismatch", error=True)
    locked = [{"name": p.name, "sha256": hashlib.sha256(p.read_bytes()).hexdigest(), "bytes": p.stat().st_size}
              for p in sorted(wheels.glob("*.whl"))]
    report["wheel_lock"] = locked
    environment = work / "venv"
    if not environment.exists():
        venv.EnvBuilder(with_pip=True).create(environment)
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    install = run([str(python), "-m", "pip", "install", "--no-index", "--find-links", str(wheels),
                   f"graphifyy=={VERSION}"], work, env, 120)
    report["steps"]["isolated_install"] = receipt(install)
    if install["exit_code"] != 0:
        return finish("candidate isolated environment not qualified", error=True)

    corpus = work / "corpus"
    corpus.mkdir(exist_ok=True)
    (corpus / "main.py").write_text("from helper import value\nprint(value())\n", encoding="utf-8")
    (corpus / "helper.py").write_text("def value():\n    return 42\n", encoding="utf-8")
    (corpus / "README.md").write_text("# Synthetic fixture\nNo model or external data required.\n", encoding="utf-8")
    # Stop repository discovery at this disposable fixture, never at the source checkout.
    initialized = run(["git", "init", "--initial-branch=main", str(corpus)], work, env)
    if initialized["exit_code"] != 0:
        return finish("isolated repository boundary unavailable", error=True)

    runner = work / "offline_entry.py"
    runner.write_text('''import importlib.metadata, json, os, socket, subprocess, sys
blocked = {"network": 0, "external_process": 0}
def deny_network(*args, **kwargs):
    blocked["network"] += 1
    raise RuntimeError("network disabled during code-only qualification")
socket.socket.connect = deny_network
socket.socket.connect_ex = deny_network
socket.create_connection = deny_network
original = subprocess.Popen
def bounded_process(argv, *args, **kwargs):
    if not isinstance(argv, (list, tuple)) or kwargs.get("shell"):
        blocked["external_process"] += 1
        raise RuntimeError("unqualified process")
    name = os.path.basename(str(argv[0])).lower()
    safe = {"rev-parse", "status", "diff", "ls-files", "log", "show"}
    if name not in {"git", "git.exe"} or not any(value in safe for value in argv[1:]):
        blocked["external_process"] += 1
        raise RuntimeError("unqualified process")
    return original(argv, *args, **kwargs)
subprocess.Popen = bounded_process
sys.argv = ["graphify", *sys.argv[1:]]
code = 0
try:
    entry = next(e for e in importlib.metadata.distribution("graphifyy").entry_points if e.name == "graphify")
    result = entry.load()()
    code = result if isinstance(result, int) else 0
except SystemExit as error:
    code = error.code if isinstance(error.code, int) else 1
except Exception as error:
    print(json.dumps({"error_class": type(error).__name__}), file=sys.stderr)
    code = 1
finally:
    print("HIVE_QUALIFICATION=" + json.dumps(blocked), file=sys.stderr)
if any(blocked.values()): code = 1
raise SystemExit(code)
''', encoding="utf-8", newline="\n")

    def extract(label: str, *arguments: str) -> dict:
        result = run([str(python), "-s", str(runner), *arguments], corpus, env)
        details = receipt(result)
        for line in result["stderr"].decode("utf-8", errors="replace").splitlines():
            if line.startswith("HIVE_QUALIFICATION="):
                details["blocked_operations"] = json.loads(line.partition("=")[2])
        report["steps"][label] = details
        return result

    full = extract("initial_full", "extract", str(corpus), "--force", "--code-only", "--no-cluster")
    if full["exit_code"] != 0:
        return finish("candidate code-only extraction failed; existing adapter retained")
    graph_paths = [p for p in corpus.rglob("graph.json") if ".git" not in p.parts]
    if len(graph_paths) != 1:
        return finish("candidate graph output identity is ambiguous")
    graph_path = graph_paths[0]
    initial = json.loads(graph_path.read_text(encoding="utf-8"))
    report["initial_root_fields"] = sorted(initial)
    report["initial_nodes"] = len(initial.get("nodes", []))
    report["initial_edges"] = len(initial.get("edges", []))
    def structure(value: dict) -> str:
        selected = {key: sorted(value.get(key, []), key=lambda row: json.dumps(row, sort_keys=True))
                    for key in ("nodes", "edges", "hyperedges")}
        return hashlib.sha256(json.dumps(selected, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

    report["comparisons"] = {}
    for change in ("modify", "add", "rename", "delete"):
        if change == "modify":
            (corpus / "helper.py").write_text("def value():\n    return extra()\ndef extra():\n    return 43\n", encoding="utf-8")
        elif change == "add":
            (corpus / "added.py").write_text("def added():\n    return 99\n", encoding="utf-8")
        elif change == "rename":
            (corpus / "added.py").replace(corpus / "renamed.py")
        else:
            (corpus / "renamed.py").unlink()
        incremental = extract(change + "_update", "update", str(corpus), "--force", "--no-cluster")
        increment = json.loads(graph_path.read_text(encoding="utf-8")) if incremental["exit_code"] == 0 else None
        rebuilt = extract(change + "_full", "extract", str(corpus), "--force", "--code-only", "--no-cluster")
        if rebuilt["exit_code"] != 0:
            return finish("candidate full rebuild failed")
        fresh = json.loads(graph_path.read_text(encoding="utf-8"))
        report["comparisons"][change] = {
            "update_equal_full": increment is not None and structure(increment) == structure(fresh),
            "update_structure_sha256": structure(increment) if increment is not None else None,
            "full_structure_sha256": structure(fresh),
            "update_is_true_incremental": False,
        }
    report["incremental_enabled"] = False
    report["cross_platform_qualification"] = "not-run; Windows first, macOS last"
    if not all(row["update_equal_full"] for row in report["comparisons"].values()):
        return finish("Windows update/full equivalence failed; candidate and incremental activation excluded")
    return finish("candidate Windows evidence only; full three-platform and adapter-contract qualification still required")


if __name__ == "__main__":
    raise SystemExit(main())
