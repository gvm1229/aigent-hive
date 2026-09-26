#!/usr/bin/env python3
"""Compare representative setup file effects of two preserved binaries in disposable targets."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/setup"


def digest(payload):
    return hashlib.sha256(payload).hexdigest()


def snapshot(root):
    return {p.relative_to(root).as_posix(): digest(p.read_bytes())
            for p in sorted(root.rglob("*")) if p.is_file()}


def invoke(binary, target, user, mode, capabilities):
    result = subprocess.run([str(binary), "setup", "--target", str(target), "--user-root", str(user),
        "--answers", str(FIXTURES / "answers-base.yml"), "--capabilities", str(capabilities),
        mode, "--output", "json"], cwd=ROOT, capture_output=True, timeout=60)
    payload = json.loads(result.stdout)
    return result.returncode, payload


def run(binary, work):
    target, user = work / "project", work / "user"
    target.mkdir(parents=True)
    config = user / ".hive/config/user-setup.yml"
    config.parent.mkdir(parents=True)
    config.write_text(json.dumps({"schema_version": 1, "interface_language": "en",
        "wiki": {"enabled": True, "language": "both"}, "profile": {"contexts": ["web-developer"]},
        "persona": {"id": "balanced"}, "selected_hosts": ["codex"], "skills": {"mode": "all"},
        "usage_guard": {"enabled": True, "stop_remaining_percent": 20,
                        "codexbar_fallback_enabled": False, "discord": {"enabled": False}}}), encoding="utf-8")
    foreign = b"# User instructions\r\nPreserve this exact user-authored prefix.\r\n"
    (target / "AGENTS.md").write_bytes(foreign)
    (target / "foreign.txt").write_bytes(b"foreign bytes\x00\r\n")
    capability = FIXTURES / "capabilities-codex-host-native.json"
    initial = snapshot(work)
    code, preview = invoke(binary, target, user, "--dry-run", capability)
    if code or snapshot(work) != initial:
        raise ValueError(f"setup preview changed files or failed: {preview.get('code')}")
    code, applied = invoke(binary, target, user, "--apply", capability)
    if code:
        raise ValueError(f"setup failed: {applied.get('code')}")
    after = snapshot(work)
    if not (target / "AGENTS.md").read_bytes().startswith(foreign):
        raise ValueError("foreign instruction prefix changed")
    if (target / "foreign.txt").read_bytes() != b"foreign bytes\x00\r\n":
        raise ValueError("foreign file changed")
    code, repeated = invoke(binary, target, user, "--apply", capability)
    if code or snapshot(work) != after:
        raise ValueError(f"setup replay did not converge: {repeated.get('code')}")
    code, validated = invoke(binary, target, user, "--validate", capability)
    if code or snapshot(work) != after:
        raise ValueError(f"setup validation failed or wrote files: {validated.get('code')}")
    malformed = work / "invalid-capability.json"
    malformed.write_text('{"host":"unsupported-fixture-host"}', encoding="utf-8")
    before_rejection = snapshot(work)
    rejected_code, rejected = invoke(binary, target, user, "--apply", malformed)
    if not rejected_code or snapshot(work) != before_rejection:
        raise ValueError("invalid capability was accepted or caused mutation")
    result = {"binary_sha256": digest(binary.read_bytes()), "files_before": initial, "files_after": after,
        "preview_write_zero": True, "apply_replay_identical": True, "validation_write_zero": True,
        "rejection_write_zero": True, "foreign_bytes_preserved": True,
        "results": [{"action": value["action"], "code": value["code"], "exit_code": value["exit_code"]}
                    for value in [preview, applied, repeated, validated, rejected]]}
    print(f"{work.name}: preview, apply, replay, validation and invalid-input rejection passed", flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--after", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    if work.exists() or not work.is_relative_to((ROOT / "tests/work").resolve()):
        raise ValueError("choose a new exact work directory under tests/work")
    work.mkdir(parents=True)
    result = {"schema_version": 1, "platform": platform.platform(),
        "fixture_digests": {name: digest((FIXTURES / name).read_bytes()) for name in
                            ["answers-base.yml", "capabilities-codex-host-native.json"]},
        "limit": "synthetic project setup on this OS; no real host execution, global installation, or cross-OS proof"}
    for label, binary in [("before", args.before), ("after", args.after)]:
        result[label] = run(binary.resolve(), work / label)
        (work / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    left, right = result["before"]["files_after"], result["after"]["files_after"]
    result["file_comparison"] = {"added": sorted(right.keys() - left.keys()),
        "removed": sorted(left.keys() - right.keys()),
        "different": sorted(path for path in left.keys() & right.keys() if left[path] != right[path])}
    result["same_result_contracts"] = result["before"]["results"] == result["after"]["results"]
    (work / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    if not result["same_result_contracts"]:
        raise ValueError("setup result contracts changed")
    print("Setup action/code/exit contracts match; exact changed file maps retained for review.")


if __name__ == "__main__":
    main()
