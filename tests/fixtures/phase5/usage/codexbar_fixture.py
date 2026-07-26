from __future__ import annotations

import copy
import json
import os
import stat
import sys
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

case = os.environ.get("FAKE_CODEXBAR_CASE", "allow")
requested_provider = os.environ.get("FAKE_CODEXBAR_PROVIDER", "codex")
now = datetime.now(timezone.utc)
expected_usage_argv = [
    "usage",
    "--provider",
    requested_provider,
    "--all-accounts",
    "--source",
    "cli",
    "--format",
    "json",
    "--json-only",
]
if log_path := os.environ.get("FAKE_CODEXBAR_LOG"):
    with Path(log_path).open("a", encoding="utf-8") as log_file:
        log_file.write(json.dumps(sys.argv[1:], separators=(",", ":")) + "\n")

if sys.argv[1:] == ["--version"]:
    if case == "timeout":
        time.sleep(1)
        raise SystemExit(0)
    if case == "oversized-version":
        sys.stdout.write("v" * (1024 * 1024 + 1))
        raise SystemExit(0)
    if case == "executable-change" and os.name != "nt":
        executable = Path(sys.argv[0])
        replacement = executable.with_name(executable.name + ".replacement")
        replacement.write_text("#!/bin/sh\nexit 9\n", encoding="utf-8")
        replacement.chmod(stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR)
        os.replace(replacement, executable)
    if case == "symlink-swap" and os.name != "nt":
        link = Path(os.environ["FAKE_CODEXBAR_LINK"])
        malicious = Path(os.environ["FAKE_CODEXBAR_MALICIOUS"])
        link.unlink()
        link.symlink_to(malicious)
    version = "999.0.0" if case == "unsupported-version" else "0.45.2"
    sys.stdout.write(f"CodexBar {version}\n")
    raise SystemExit(0)

if sys.argv[1:] != expected_usage_argv:
    sys.stderr.write("unexpected argv\n")
    raise SystemExit(64)
if case == "timeout":
    time.sleep(1)
    raise SystemExit(0)
if case == "malformed":
    sys.stdout.write("{not-json")
    raise SystemExit(0)
if case == "process-error":
    sys.stderr.write("fixture sensor failure\n")
    raise SystemExit(7)
if case == "oversized-usage":
    sys.stdout.write("x" * (1024 * 1024 + 1))
    raise SystemExit(0)
if case == "oversized-stderr":
    sys.stderr.write("x" * (1024 * 1024 + 1))
    raise SystemExit(0)

account = "usage-guard@example.invalid"
updated_at = now
primary_used = 43
secondary_used = 43
error = None
provider = requested_provider
source = f"{requested_provider}-cli"
identity: dict[str, str] | None = {"providerID": requested_provider}

if case == "threshold":
    primary_used = secondary_used = 90
elif case == "one-window-low":
    primary_used = 92
elif case == "weekly-only":
    pass
elif case == "weekly-only-threshold":
    secondary_used = 90
elif case == "weekly-low-session-high":
    secondary_used = 92
elif case == "remaining-increase":
    primary_used = 30
elif case == "session-low-weekly-malformed":
    primary_used = 92
elif case == "wrong-account":
    account = "wrong-account@example.invalid"
elif case == "stale":
    updated_at = now - timedelta(hours=1)
elif case == "future":
    updated_at = now + timedelta(minutes=2)
elif case == "sensor-error":
    error = "usage unavailable"
elif case == "wrong-provider":
    provider = "claude"
elif case == "wrong-source":
    source = "openai-web"
elif case == "missing-identity":
    identity = None
elif case == "measurement-regression":
    updated_at = now - timedelta(seconds=10)

reset_at = os.environ.get(
    "FAKE_CODEXBAR_RESET_AT",
    (now + timedelta(hours=2)).isoformat().replace("+00:00", "Z"),
)
usage: dict[str, object] = {
    "primary": {
        "usedPercent": primary_used,
        "windowMinutes": 300,
        "resetsAt": reset_at,
    },
    "secondary": {
        "usedPercent": secondary_used,
        "windowMinutes": 10080,
        "resetsAt": (now + timedelta(days=4)).isoformat().replace("+00:00", "Z"),
    },
    "updatedAt": updated_at.isoformat().replace("+00:00", "Z"),
    "identity": identity,
}
if case in ("weekly-only", "weekly-only-threshold", "weekly-duplicate"):
    usage["primary"] = None
if case == "missing-window":
    del usage["primary"]
    del usage["secondary"]
if case in ("weekly-malformed-session-high", "session-low-weekly-malformed"):
    usage["secondary"] = "malformed"

row = {
    "provider": provider,
    "account": account,
    "version": "0.45.2",
    "source": source,
    "error": error,
    "usage": usage,
}
rows = [row]
if case == "duplicate-account":
    rows.append(copy.deepcopy(row))

encoded = json.dumps(rows, separators=(",", ":"))
if case in ("weekly-duplicate-session-high", "weekly-duplicate"):
    secondary = json.dumps(usage["secondary"], separators=(",", ":"))
    encoded = encoded.replace(
        f'"secondary":{secondary}',
        f'"secondary":"duplicate","secondary":{secondary}',
        1,
    )
sys.stdout.write(encoded)
sys.stdout.write("\n")
