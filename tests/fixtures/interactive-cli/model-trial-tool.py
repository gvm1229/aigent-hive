"""Fixed model-trial transport; only synthetic setup roots and pinned bytes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
CONFIG = json.loads((ROOT / "config.json").read_text(encoding="utf-8"))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def emit(event, output=b"", error=b""):
    event["hive_input_bytes"] = len(output) + len(error)
    with (ROOT / "events.jsonl").open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(event, ensure_ascii=False) + "\n")
    sys.stdout.buffer.write(output)
    sys.stderr.buffer.write(error)


operation = sys.argv[1] if len(sys.argv) > 1 else "missing"
try:
    if operation == "read-skill":
        path = (ROOT / "skill" / sys.argv[2]).resolve()
        assert path.is_relative_to(ROOT / "skill") and path.is_file()
        content = path.read_bytes()
        assert digest(content) == CONFIG["input_sha256"][path.relative_to(ROOT).as_posix()]
        emit({"operation": operation, "path": path.relative_to(ROOT / "skill").as_posix(), "sha256": digest(content)}, content)
    elif operation == "write-answers":
        content = sys.argv[2].encode("utf-8") if len(sys.argv) == 3 else sys.stdin.buffer.read(65537)
        assert len(content) <= 65536
        json.loads(content)
        (ROOT / "answers.json").write_bytes(content)
        output = b"answers.json saved\n"
        emit({"operation": operation, "answer_sha256": digest(content), "answer_bytes": len(content)}, output)
    elif operation == "hive":
        args = sys.argv[2:]
        assert args and args[0] == "setup"
        for flag, path in [("--user-root", ROOT / "user")]:
            assert args.count(flag) == 1 and Path(args[args.index(flag) + 1]).resolve() == path
        if CONFIG["setup_scope"] == "user":
            assert args.count("--scope") == 1 and args[args.index("--scope") + 1] == "user" and "--target" not in args
        else:
            assert "--scope" not in args and args.count("--target") == 1
            assert Path(args[args.index("--target") + 1]).resolve() == ROOT / "project"
        for flag in ["--answers", "--quick-answers", "--capabilities"]:
            if flag in args:
                path = Path(args[args.index(flag) + 1]).resolve()
                assert path.is_relative_to(ROOT) and path.is_file()
                assert flag != "--capabilities" or path == ROOT / "capabilities.json"
        binary = Path(CONFIG["binary"])
        assert digest(binary.read_bytes()) == CONFIG["binary_sha256"]
        env = {"PATH": str(ROOT / "closed-bin"), "PATHEXT": ".COM;.EXE;.BAT;.CMD", "COMSPEC": os.environ["COMSPEC"],
            "SystemRoot": str(ROOT / "disabled-system"), "WINDIR": str(ROOT / "disabled-system"),
            "HIVE_TEST_HOST_LOG": str(ROOT / "host-commands.jsonl"), "HIVE_TEST_USER_ROOT": str(ROOT / "user"),
            "PYTHONUTF8": "1", "PYTHONNOUSERSITE": "1", "CARGO_INCREMENTAL": "0"}
        for key, relative in {"HOME": "profile", "USERPROFILE": "profile", "CODEX_HOME": "profile/codex",
            "CLAUDE_CONFIG_DIR": "profile/claude", "GEMINI_CLI_HOME": "profile/gemini", "APPDATA": "profile/roaming",
            "LOCALAPPDATA": "profile/local", "XDG_CONFIG_HOME": "profile/xdg", "NPM_CONFIG_PREFIX": "prefix",
            "NPM_CONFIG_CACHE": "npm-cache", "TEMP": "tmp", "TMP": "tmp"}.items():
            env[key] = str(ROOT / relative)
        started = time.monotonic()
        result = subprocess.run([str(binary), *args], env=env, cwd=ROOT, input=b"", capture_output=True, timeout=40)
        try:
            response = json.loads(result.stdout)
            diagnostic = {k: response.get(k) for k in ("action", "status", "code", "message")}
        except ValueError:
            diagnostic = {"protocol_error": "non-JSON response"}
        emit({"operation": operation, "arguments": args, "exit_code": result.returncode, "result": diagnostic,
            "stdout_sha256": digest(result.stdout), "stderr_sha256": digest(result.stderr),
            "elapsed_seconds": time.monotonic() - started}, result.stdout, result.stderr)
        raise SystemExit(result.returncode)
    else:
        raise ValueError("unsupported fixed-trial operation")
except Exception as failure:
    message = ("Trial input rejected: " + type(failure).__name__ + ". For write-answers, pass JSON as one argument or on stdin.\n").encode("utf-8")
    emit({"operation": operation, "exit_code": 2, "failure_kind": "fixture-input"}, error=message)
    raise SystemExit(2)
