#!/usr/bin/env python3
"""Record bounded, non-mutating CLI contracts and startup timings for a refactor."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
CASES = {
    "version": ["--version"],
    "help": ["--help"],
    "knowledge-help": ["knowledge", "--help"],
    "run-help": ["run", "--help"],
    "setup-help": ["setup", "--help"],
    "neutral-stop": ["hook", "--event", "Stop", "--output", "json"],
    "invalid-command": ["not-a-hive-command"],
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def work_path(path):
    resolved = path.resolve()
    if not resolved.is_relative_to((ROOT / "tests/work").resolve()):
        raise ValueError("measurement output must stay inside tests/work")
    return resolved


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = work_path(args.output)
    if output.exists():
        raise ValueError("measurement exists; preserve it and choose a new result path")
    if args.snapshot:
        snapshot = work_path(args.snapshot)
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        if snapshot.exists():
            raise ValueError("baseline snapshot already exists")
        shutil.copyfile(binary, snapshot)
        binary = snapshot
    rows = []
    for name, arguments in CASES.items():
        samples, outputs = [], []
        for iteration in range(35):
            started = time.perf_counter()
            result = subprocess.run([str(binary), *arguments], cwd=ROOT,
                                    capture_output=True, timeout=20)
            elapsed = (time.perf_counter() - started) * 1000
            if iteration >= 5:
                samples.append(elapsed)
            outputs.append((result.returncode, hashlib.sha256(result.stdout).hexdigest(),
                            hashlib.sha256(result.stderr).hexdigest()))
        if len(set(outputs)) != 1:
            raise ValueError(f"non-deterministic CLI output: {name}")
        rows.append({"case": name, "arguments": arguments, "exit_code": outputs[-1][0],
                     "stdout_sha256": outputs[-1][1], "stderr_sha256": outputs[-1][2],
                     "median_ms": statistics.median(samples),
                     "p95_ms": sorted(samples)[28], "samples_ms": samples})
        print(f"{name}: p95={rows[-1]['p95_ms']:.2f} ms", flush=True)
    report = {"schema_version": 1, "source_commit": args.source_commit,
              "binary_sha256": digest(binary), "platform": platform.platform(),
              "warmups": 5, "samples": 30, "commands": rows,
              "limit": "CLI contracts and startup only; not retrieval or live host acceptance",
              "directive_bytes": sum(p.stat().st_size for p in (ROOT / "harness/directives").glob("*.md"))}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(f"Recorded {output.relative_to(ROOT).as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
