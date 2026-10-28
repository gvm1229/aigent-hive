"""Record fixed M19 setup cases, including observations from failed setup."""
from pathlib import Path
import json
import hashlib
import sys
import time

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run


def case_identity(config):
    """Transient trial paths and documentation commits do not create a new test."""
    inputs = config.get("input_sha256")
    if not isinstance(inputs, dict) or not inputs or not config.get("binary_sha256"):
        return None
    parameters = {key: config.get(key) for key in (
        "scenario", "setup_scope", "comparison_version", "model", "reasoning_effort",
        "requested_speed_mode", "binary_sha256", "expected_values", "user_before",
    )}
    parameters["input_sha256"] = inputs
    return hashlib.sha256(json.dumps(parameters, sort_keys=True, ensure_ascii=False).encode("utf-8")).hexdigest()


def prior_case_runs(config, reports):
    identity = case_identity(config)
    if identity is None:
        raise ValueError("model trial needs pinned binary and input hashes before dispatch")
    matches = []
    for path in sorted(reports.glob("*.md")):
        text = path.read_text(encoding="utf-8")
        if '"host:collaboration.spawn_agent"' not in text:
            continue
        record = json.loads(text.split("```json\n", 1)[1].split("\n```", 1)[0])
        if record.get("command", [])[:1] == ["host:collaboration.spawn_agent"] and case_identity(record.get("details", {})) == identity:
            matches.append(path.name)
    if len(matches) >= 5:
        raise ValueError("same-parameter model trial already has five runs; use existing evidence or obtain an explicit exception")
    return identity, matches


def pointer(value, path):
    for key in path.removeprefix("/").split("/"):
        key = key.replace("~1", "/").replace("~0", "~")
        value = value[int(key)] if isinstance(value, list) else value[key]
    return value


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
        "expected_apply_count": sum(e.get("operation") == "hive" and "--apply" in e.get("arguments", []) for e in events) == config.get("expected_apply_count", 1),
        "no_failed_cli_retry": all(e.get("exit_code", 0) == 0 for e in events),
        "bounded_responses": all(e["hive_input_bytes"] <= (4096 if "--show" in e.get("arguments", []) else 8192)
            for e in events if e.get("operation") == "hive"
            and "--dry-run" not in e.get("arguments", [])
            and not {"--describe", "--section"}.issubset(e.get("arguments", []))),
    }
    comparison = config.get("comparison") is True
    if comparison:
        failures = [e for e in events if e.get("exit_code", 0) != 0]
        fingerprints = [e.get("input_sha256") or json.dumps([e.get("operation"),
            e.get("arguments"), e.get("failure_kind"), e.get("result")], sort_keys=True) for e in failures]
        del checks["no_failed_cli_retry"]
        checks["no_repeated_failed_input"] = len(fingerprints) == len(set(fingerprints))
        checks["bounded_failure_corrections"] = len(failures) <= 2
        descriptions = [e for e in events if e.get("operation") == "hive"
            and e.get("exit_code", 0) == 0 and "--describe" in e.get("arguments", [])
            and not {"--section", "--question"}.intersection(e.get("arguments", []))]
        checks["no_repeated_full_schema"] = len(descriptions) <= 1
        if config["comparison_version"] == "0.13.1":
            del checks["bounded_responses"]
    scope = config.get("setup_scope", "project")
    settings = trial / ("user/.hive/config/user-setup.yml" if scope == "user" else "project/.hive/setup-answers.yml")
    checks["final_settings_present"] = settings.is_file()
    try:
        if settings.is_file():
            answers = yaml.safe_load(settings.read_text(encoding="utf-8"))
            expected = config.get("expected_values", {
                "/project_name": "luna-quick", "/project_kind": "general",
                "/setup_mode": "expedited", "/primary_host": "codex",
                "/persistent_roles": [], "/approved_optional_skills": [], "/approved_fallback_hooks": [],
            })
            checks["requested_values"] = all(pointer(answers, path) == value for path, value in expected.items())
            if not comparison or config["comparison_version"] != "0.13.1":
                choices = answers["preference_decisions"]
                checks["agent_not_human"] = all(choices[k]["input_channel"] == "agent"
                    and choices[k]["decided_at"] is None for k in config.get("agent_fields", ["/project_name", "/project_kind", "/primary_host"]))
        checks["foreign_bytes_preserved"] = (trial / scope / "README.txt").read_bytes() == b"Synthetic user note. Preserve exactly.\n"
        checks["global_config_preserved"] = all(hashlib.sha256((trial / "user" / path).read_bytes()).hexdigest() == digest
            for path, digest in config["user_before"].items())
        for relative, digest in config.get("preserved_files", {}).items():
            path = (trial / relative).resolve()
            if not path.is_relative_to(trial.resolve()):
                raise ValueError("preservation path leaves the trial")
            checks["preserved:" + relative] = hashlib.sha256(path.read_bytes()).hexdigest() == digest
        for relative, content in config.get("preserved_fragments", {}).items():
            path = (trial / relative).resolve()
            if not path.is_relative_to(trial.resolve()):
                raise ValueError("preservation fragment leaves the trial")
            checks["preserved_fragment:" + relative] = content.encode("utf-8") in path.read_bytes()
        if config.get("progress_relative"):
            path = (trial / config["progress_relative"]).resolve()
            if not path.is_relative_to(trial.resolve()):
                raise ValueError("progress path leaves the trial")
            progress = yaml.safe_load(path.read_text(encoding="utf-8"))
            checks["same_operation_complete"] = progress["operation_id"] == config["operation_id"] and progress["phase"] == "complete"
    except (OSError, KeyError, IndexError, ValueError, TypeError, yaml.YAMLError) as error:
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
    identity, earlier = prior_case_runs(config, ROOT / "tests/results/runs")
    run = Run("M20 실제 작은 모델의 버전별 고정 설정 비교" if config.get("comparison") else "실제 작은 모델의 고정 설정 사례 평가",
        ["host:collaboration.spawn_agent", config["model"], config["reasoning_effort"], config["scenario"]],
        paths=[trial.relative_to(ROOT).as_posix()], binary=Path(config["binary"]))
    run.data["details"] = {**config, "case_identity": identity,
        "same_parameter_iteration": len(earlier) + 1, "prior_case_reports": earlier,
        "maximum_same_parameter_iterations": 5, "host_state": "reserved-before-dispatch"}
    run.write()
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
        "case_identity": identity, "same_parameter_iteration": len(earlier) + 1,
        "prior_case_reports": earlier, "maximum_same_parameter_iterations": 5,
        "elapsed_seconds": time.monotonic() - started,
        "scope": "Actual host trial with synthetic inputs and closed CLI environment; no real app/service/install control. Missing host tokens or speed remain unknown. No M19/M20 aggregate acceptance implied."})
    print(json.dumps({"report": run.relative, "exit_code": code, "checks": result["checks"],
        "hive_input_bytes": result["hive_input_bytes"], "errors": result["errors"]}), flush=True)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
