"""Record the fixed M19 project trial, including observations from failed setup."""
from pathlib import Path
import json
import hashlib
import sys
import time

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run


def collect(trial, config, host):
    events = []
    errors = []
    event_path = trial / "events.jsonl"
    complete = event_path.is_file()
    if complete:
        for number, line in enumerate(event_path.read_text(encoding="utf-8").splitlines(), 1):
            try:
                event = json.loads(line)
                if type(event.get("hive_input_bytes")) is not int or event["hive_input_bytes"] < 0:
                    raise ValueError("missing or invalid byte count")
                events.append(event)
            except (ValueError, AttributeError) as error:
                errors.append(f"event {number}: {error}")
                complete = False
    else:
        errors.append("event log absent; byte count unknown")
    checks = {
        "event_log_complete": complete,
        "host_completed": host.get("status") == "completed",
        "one_apply": sum(e.get("operation") == "hive" and "--apply" in e.get("arguments", []) for e in events) == 1,
        "no_failed_cli_retry": all(e.get("exit_code", 0) == 0 for e in events),
    }
    settings = trial / "project/.hive/setup-answers.yml"
    checks["final_settings_present"] = settings.is_file()
    try:
        if settings.is_file():
            answers = yaml.safe_load(settings.read_text(encoding="utf-8"))
            checks["requested_values"] = all(answers[k] == v for k, v in {
                "project_name": "luna-quick", "project_kind": "general",
                "setup_mode": "expedited", "primary_host": "codex",
            }.items())
            checks["unrequested_roles_and_hooks_absent"] = all(answers[k] == [] for k in (
                "persistent_roles", "approved_optional_skills", "approved_fallback_hooks"))
            choices = answers["preference_decisions"]
            checks["agent_not_human"] = all(choices["/" + k]["input_channel"] == "agent"
                and choices["/" + k]["decided_at"] is None for k in ("project_name", "project_kind", "primary_host"))
        checks["foreign_bytes_preserved"] = (trial / "project/README.txt").read_bytes() == b"Synthetic user note. Preserve exactly.\n"
        checks["global_config_preserved"] = all(hashlib.sha256((trial / "user" / path).read_bytes()).hexdigest() == digest
            for path, digest in config["user_before"].items())
    except (OSError, KeyError, TypeError, yaml.YAMLError) as error:
        errors.append(f"final state: {error}")
        checks["final_state_readable"] = False
    return {
        "checks": checks, "events": events, "errors": errors,
        "hive_input_bytes": sum(e["hive_input_bytes"] for e in events) if complete else None,
        "observed_event_bytes": sum(e["hive_input_bytes"] for e in events),
    }


def main():
    trial = Path(sys.argv[1]).resolve()
    if not trial.is_relative_to((ROOT / "tests/work").resolve()):
        raise ValueError("trial must remain inside tests/work")
    config = json.loads((trial / "config.json").read_text(encoding="utf-8"))
    run = Run("Luna low 실제 최소 프로젝트 설정 재평가",
        ["host:collaboration.spawn_agent", config["model"], config["reasoning_effort"], "project-quick"],
        paths=[trial.relative_to(ROOT).as_posix()], binary=Path(config["binary"]))
    (trial / "ready.json").write_text(json.dumps({"report": run.relative}), encoding="utf-8")
    print("ready " + run.relative, flush=True)
    started = time.monotonic()
    host = {}
    error = None
    try:
        while not (trial / "done.json").exists() and time.monotonic() - started < 480:
            time.sleep(1)
        host = json.loads((trial / "done.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as failure:
        error = str(failure)
    result = collect(trial, config, host)
    if error:
        result["errors"].append(error)
    code = int(bool(result["errors"]) or not all(result["checks"].values()))
    run.finish(code, details={**config, **result, "host_receipt": host,
        "elapsed_seconds": time.monotonic() - started,
        "scope": "Actual host trial with synthetic inputs and closed CLI environment; no real app/service/install control. Missing host tokens or speed remain unknown. No M19/M20 aggregate acceptance implied."})
    print(json.dumps({"report": run.relative, "exit_code": code, **result}), flush=True)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
