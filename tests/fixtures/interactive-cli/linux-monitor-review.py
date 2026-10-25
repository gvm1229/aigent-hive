"""M02-C2 flow, invoked by linux-legacy-update.py in the private Linux container."""
import copy
import hashlib
import json
import os
from pathlib import Path
import pty
import select
import subprocess
import time

assert os.environ.get("HIVE_QUALIFICATION_CONTAINER") == "1"
assert str(user).startswith("/tmp/hive-first-update/")
setup = [str(binary), "setup", "--scope", "user", "--user-root", str(user)]
progress_path = user / ".hive/config/setup-progress/user.yml"
history = user / ".hive/history/lifecycle"
config = user / ".hive/config/user-setup.yml"


def invoke(flags, expected=0, extra=None):
    result = subprocess.run([*setup, *flags, "--output", "json"],
                            env={**environment, **(extra or {})}, input="", capture_output=True,
                            text=True, timeout=30)
    assert (result.returncode != 0 if expected is None else result.returncode == expected), (flags, result.returncode, result.stdout, result.stderr)
    return json.loads(result.stdout) if result.stdout else None


def digest(value):
    return "sha256:" + hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True,
                                                separators=(",", ":")).encode()).hexdigest()


def private_write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False))
    path.chmod(0o600)


def prepare(language="ko"):
    # The activation itself is unsupported on Linux. Preserve a real CLI-created bound draft,
    # then seed only the unfinished activation/receipt as explicit synthetic input.
    terminal_run([*setup, "--section", "persona"], "decline-final")
    pending = invoke(["--show"])["data"]["pending"]
    pending["phase"] = "applying"
    pending["step"] = "user-monitor-activation"
    pending["answers"]["interface_language"] = language
    pending["answers"]["usage_guard"]["enforcement_enabled"] = True
    pending["answers"]["usage_guard"]["midway"]["enabled"] = True
    for key in ("/usage_guard/enforcement_enabled", "/usage_guard/midway/enabled"):
        pending["answers"]["preference_decisions"].pop(key, None)
    selections = copy.deepcopy(pending["answers"])
    selections.pop("preference_decisions", None)
    native = "sha256:" + "a" * 64
    pending["monitor_start"] = {"native_preview": native, "selection_digest": digest(selections),
        "confirmation_digest": digest({"monitor": native, "settings": digest(selections)}), "channel": "terminal"}
    intent = copy.deepcopy(pending)
    intent.pop("phase")
    receipt = {"schema_version": 1, "operation_id": pending["operation_id"], "operation": "setup",
        "product_version": "0.14.0", "binding": pending["binding"], "before_digest": pending["base_config_digest"],
        "before_mode": pending["base_config_mode"], "candidate_digest": digest(pending["answers"]),
        "intent_digest": digest(intent), "after_digest": None, "question_revision": pending["question_revision"],
        "catalog_digest": pending["catalog_digest"], "phase": "applying",
        "started_at_unix_nanos": time.time_ns(), "completed_at_unix_nanos": None}
    private_write(progress_path, pending)
    private_write(history / (pending["operation_id"] + ".json"), receipt)
    return pending


def answer(pending, channel="answers-file"):
    preview = invoke(["--resume", "--review", "--dry-run"])["data"]
    assert preview["operation_id"] == pending["operation_id"]
    path = root / "review-answers.json"
    path.write_text(json.dumps({"question_id": "user-monitor-withdrawal", "question_revision": 1,
        "answer": True, "confirmed": True, "preview_digest": preview["preview_digest"], "input_channel": channel}))
    return path


def verify(pending, channel):
    shown = invoke(["--show"])["data"]
    stored = copy.deepcopy(shown["stored"])
    assert stored["usage_guard"]["enforcement_enabled"]
    assert not stored["usage_guard"]["midway"]["enabled"]
    desired = copy.deepcopy(pending["answers"])
    desired["usage_guard"]["midway"]["enabled"] = False
    for value in (desired, stored):
        value.pop("preference_decisions", None)
    assert desired == stored, (desired, stored)
    record = json.loads((history / (pending["operation_id"] + ".json")).read_text())
    assert record["phase"] == "recoverable-failure" and record["cancelled_by_operation"] != pending["operation_id"]
    replacement = json.loads((history / (record["cancelled_by_operation"] + ".json")).read_text())
    assert replacement["phase"] == "complete"
    current = invoke(["--show"])["data"]["stored"]
    decision = current["preference_decisions"]["/usage_guard/midway/enabled"]
    assert decision["input_channel"] == channel and decision["question_id"] == "user-monitor-withdrawal"
    assert not (user / ".hive/config/setup-progress/monitor-handoff.json").exists()
    assert not (user / ".hive/runtime/usage-monitor/config.json").exists()
    return record["cancelled_by_operation"]


def review_terminal(replies, expected):
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execve(str(binary), [*setup, "--resume", "--review"], environment)
    output = b""
    pending = b""
    status = None
    queue = list(replies)
    began = time.monotonic()
    try:
        while time.monotonic() - began < 30:
            if select.select([descriptor], [], [], 0.1)[0]:
                try:
                    chunk = os.read(descriptor, 65536)
                except OSError:
                    chunk = b""
                output += chunk
                pending += chunk
                if pending.endswith(b"> ") and queue:
                    os.write(descriptor, queue.pop(0))
                    pending = b""
            done, raw = os.waitpid(pid, os.WNOHANG)
            if done:
                status = os.waitstatus_to_exitcode(raw)
                break
        assert status == expected, (status, output.decode(errors="replace"))
    finally:
        if status is None:
            os.killpg(pid, 9)
            os.waitpid(pid, 0)
        os.close(descriptor)
    return output.decode(errors="replace")


cases = []
pending = prepare()
review_terminal([b"\n"], 5)
old = json.loads((history / (pending["operation_id"] + ".json")).read_text())
assert old["phase"] == "recoverable-failure" and "cancelled_by_operation" not in old
assert not (user / ".hive/runtime/usage-monitor/disable.json").exists()
cases.append("default-no/resume-original/unsupported-Linux-activation")
before = (config.read_bytes(), progress_path.read_bytes())
path = answer(pending, "agent")
assert before == (config.read_bytes(), progress_path.read_bytes())
invoke(["--resume", "--review"], 2)
text = review_terminal([b"\x04"], 2)
assert "철회" in text and before == (config.read_bytes(), progress_path.read_bytes())
review_terminal([b"yes\n", b"no\n"], 2)
assert before == (config.read_bytes(), progress_path.read_bytes())
result = invoke(["--resume", "--review", "--review-answers", str(path)])
assert result["data"]["monitor_review"]["withdrawn"]
new_id = verify(pending, "agent")
assert invoke(["--resume", "--review"])["data"]["operation_id"] == new_id
cases.append("preview/no-input/EOF/final-no/agent-file/complete-repeat")

pending = prepare("en")
text = review_terminal([b"yes\n", b"yes\n"], 0)
assert "Withdraw" in text and "Other settings were applied and verified" in text
verify(pending, "terminal")
cases.append("terminal-yes")

for boundary in ("stopped", "settings", "handoff", "checkpoint"):
    pending = prepare()
    path = answer(pending)
    invoke(["--resume", "--review", "--review-answers", str(path)], 89,
           {"HIVE_TEST_SETUP_REVIEW_EXIT_AFTER": boundary})
    path.unlink()
    result = invoke(["--resume"])
    assert result["code"] == "hive.user-setup-complete"
    assert result["data"]["monitor_review"]["withdrawn"]
    verify(pending, "answers-file")
    cases.append("process-exit/recovery:" + boundary)

pending = prepare()
path = answer(pending)
invoke(["--resume", "--review", "--review-answers", str(path)], 89,
       {"HIVE_TEST_SETUP_REVIEW_EXIT_AFTER": "checkpoint"})
off = config.read_bytes()
assert not invoke(["--show"])["data"]["stored"]["usage_guard"]["midway"]["enabled"]
host_command = root / "shim/agy"
host_bytes = host_command.read_bytes()
host_command.write_text("#!/usr/bin/python3\nraise SystemExit(42)\n")
failure = invoke(["--resume"], None)
assert failure["status"] != "success" and config.read_bytes() == off
host_command.write_bytes(host_bytes)
result = invoke(["--resume", "--review"])
assert result["data"]["monitor_review"]["withdrawn"]
verify(pending, "answers-file")
cases.append("new-settings-failure/off-preserved/recovered")

Path("/results/monitor-review-cases.json").write_text(json.dumps({"cases": cases,
    "scope": "actual Linux CLI with fake app, synthetic unfinished consent, no configured native monitor"}, indent=2))
