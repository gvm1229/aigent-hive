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


def private_environment():
    env = {"PATH": str(ROOT / "closed-bin"), "PATHEXT": ".COM;.EXE;.BAT;.CMD", "COMSPEC": os.environ.get("COMSPEC", ""),
        "SystemRoot": str(ROOT / "disabled-system"), "WINDIR": str(ROOT / "disabled-system"),
        "HIVE_TEST_HOST_LOG": str(ROOT / "host-commands.jsonl"), "HIVE_TEST_USER_ROOT": str(ROOT / "user"),
        "HIVE_TEST_HOST_STATE": str(ROOT / "host-state.json"),
        "PYTHONUTF8": "1", "PYTHONNOUSERSITE": "1", "CARGO_INCREMENTAL": "0",
        "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": str(ROOT / "empty-git-config")}
    for key, relative in {"HOME": "profile", "USERPROFILE": "profile", "CODEX_HOME": "profile/codex",
        "CLAUDE_CONFIG_DIR": "profile/claude", "GEMINI_CLI_HOME": "profile/gemini", "APPDATA": "profile/roaming",
        "LOCALAPPDATA": "profile/local", "XDG_CONFIG_HOME": "profile/xdg", "NPM_CONFIG_PREFIX": "prefix",
        "NPM_CONFIG_CACHE": "npm-cache", "TEMP": "tmp", "TMP": "tmp"}.items():
        env[key] = str(ROOT / relative)
    return env


def comparison_arguments(args):
    assert CONFIG.get("comparison") is True
    if args in (["--version"], ["setup", "--help"]):
        return
    assert args and args[0] == "setup"
    flags = {"--scope", "--user-root", "--target", "--answers", "--quick-answers", "--capabilities",
             "--output", "--input-channel", "--section", "--question", "--progress", "--step", "--id"}
    switches = {"--help", "--describe", "--show", "--summary", "--dry-run", "--apply", "--validate"}
    start = 1
    feature = args[1:2] == ["feature"]
    if feature:
        assert CONFIG["setup_scope"] == "user" and args[2:3] in (["status"], ["claim"])
        start = 3
    seen = set()
    index = start
    while index < len(args):
        flag = args[index]
        assert flag not in seen and flag in flags | switches
        seen.add(flag)
        index += 1
        if flag in flags:
            assert index < len(args) and len(args[index]) <= 2048
            index += 1
    readonly = "--help" in seen or "--describe" in seen
    assert not readonly or not seen & {"--apply", "--dry-run", "--validate", "--progress"}
    if not readonly:
        assert args.count("--user-root") == 1
    if "--user-root" in seen:
        actual = Path(args[args.index("--user-root") + 1]).resolve()
        if actual != ROOT / "user":
            raise ValueError(f"--user-root scope mismatch: expected exactly {ROOT / 'user'}; received {actual}")
    if CONFIG["setup_scope"] == "user":
        assert "--target" not in seen
        if not feature:
            assert args.count("--scope") == 1 and args[args.index("--scope") + 1] == "user"
    else:
        assert "--scope" not in seen
        if not readonly:
            assert args.count("--target") == 1
        if "--target" in seen:
            actual = Path(args[args.index("--target") + 1]).resolve()
            if actual != ROOT / "project":
                raise ValueError(f"--target scope mismatch: expected exactly {ROOT / 'project'}; received {actual}")
    if "--id" in seen:
        assert feature and args[args.index("--id") + 1] == "vector-search"
    if "--progress" in seen:
        assert CONFIG["setup_scope"] == "user" and args[args.index("--progress") + 1] in {"status", "save", "clear"}
    if "--input-channel" in seen:
        assert args[args.index("--input-channel") + 1] == "agent"
    if "--output" in seen:
        assert args[args.index("--output") + 1] == "json"


operation = sys.argv[1] if len(sys.argv) > 1 else "missing"
try:
    if operation == "read-skill":
        assert len(sys.argv) in (2, 3)
        path = (ROOT / "skill" / (sys.argv[2] if len(sys.argv) == 3 else "SKILL.md")).resolve()
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
    elif operation == "read-input" and CONFIG.get("comparison"):
        assert len(sys.argv) == 3 and sys.argv[2] in CONFIG["readable_paths"]
        requested = ROOT / sys.argv[2]
        assert not requested.is_symlink()
        path = requested.resolve()
        assert path.is_relative_to(ROOT)
        content = path.read_bytes() if path.is_file() else b"File absent in the disposable fixture.\n"
        assert len(content) <= 65536
        emit({"operation": operation, "path": sys.argv[2], "sha256": digest(content)}, content)
    elif operation == "project-info" and CONFIG.get("comparison"):
        assert CONFIG["setup_scope"] == "project" and len(sys.argv) == 2
        git = Path(CONFIG["git_binary"])
        assert digest(git.read_bytes()) == CONFIG["git_sha256"]
        prefix = [str(git), "--no-optional-locks", "-c", "core.fsmonitor=false", "-C", str(ROOT / "project")]
        top = subprocess.run([*prefix, "rev-parse", "--show-toplevel"], env=private_environment(), capture_output=True, timeout=10)
        assert top.returncode == 0 and Path(top.stdout.decode().strip()).resolve() == ROOT / "project"
        result = subprocess.run([*prefix, "status", "--porcelain=v1"], env=private_environment(), capture_output=True, timeout=10)
        emit({"operation": operation, "exit_code": result.returncode}, result.stdout, result.stderr)
        raise SystemExit(result.returncode)
    elif operation == "discard-answers" and CONFIG.get("comparison"):
        assert len(sys.argv) == 2
        path = ROOT / "answers.json"
        assert path.is_file() and not path.is_symlink()
        content = path.read_bytes()
        archive = ROOT / "answer-history"
        archive.mkdir(exist_ok=True)
        (archive / (digest(content) + ".json")).write_bytes(content)
        path.unlink()
        emit({"operation": operation, "answer_sha256": digest(content)}, b"Temporary answer file removed; exact bytes retained for evaluation.\n")
    elif operation == "hive":
        args = sys.argv[2:]
        if CONFIG.get("comparison"):
            comparison_arguments(args)
            previous = (ROOT / "events.jsonl").read_text().splitlines() if (ROOT / "events.jsonl").exists() else []
            assert sum(json.loads(line).get("operation") == "hive" for line in previous) < 12
        else:
            assert args and args[0] == "setup"
            assert args.count("--user-root") == 1 and Path(args[args.index("--user-root") + 1]).resolve() == ROOT / "user"
            if CONFIG["setup_scope"] == "user":
                assert args.count("--scope") == 1 and args[args.index("--scope") + 1] == "user" and "--target" not in args
            else:
                assert "--scope" not in args and args.count("--target") == 1
                assert Path(args[args.index("--target") + 1]).resolve() == ROOT / "project"
        answer_digest = None
        for flag in ["--answers", "--quick-answers", "--capabilities"]:
            if flag in args:
                path = Path(args[args.index(flag) + 1]).resolve()
                assert path.is_relative_to(ROOT) and path.is_file()
                assert flag != "--capabilities" or path == ROOT / "capabilities.json"
                if flag != "--capabilities":
                    answer_digest = digest(path.read_bytes())
        binary = Path(CONFIG["binary"])
        assert digest(binary.read_bytes()) == CONFIG["binary_sha256"]
        started = time.monotonic()
        result = subprocess.run([str(binary), *args], env=private_environment(), cwd=ROOT, input=b"", capture_output=True, timeout=40)
        try:
            response = json.loads(result.stdout)
            diagnostic = {k: response.get(k) for k in ("action", "status", "code", "message")}
        except ValueError:
            diagnostic = {"protocol_error": "non-JSON response"}
        emit({"operation": operation, "arguments": args, "exit_code": result.returncode, "result": diagnostic, "answer_sha256": answer_digest,
            "stdout_sha256": digest(result.stdout), "stderr_sha256": digest(result.stderr),
            "elapsed_seconds": time.monotonic() - started}, result.stdout, result.stderr)
        raise SystemExit(result.returncode)
    else:
        raise ValueError("unsupported fixed-trial operation")
except Exception as failure:
    hint = {
        "read-skill": "Use read-skill alone for SKILL.md, or read-skill references/<file>.md relative to the fixture skill folder.",
        "write-answers": "Pass the complete JSON document as one argument or on stdin; this helper does not add an answer envelope.",
        "hive": "Use setup with the exact fixture --user-root and --target (or --scope user) paths.",
        "read-input": "Read only a listed non-secret fixture-relative path.",
        "project-info": "Use project-info alone; it reads only the private project Git status.",
        "discard-answers": "Use discard-answers alone after verification; the exact temporary answer file is archived.",
    }.get(operation, "Supported operations: read-skill, write-answers, hive.")
    detail = str(failure) if CONFIG.get("comparison") and isinstance(failure, ValueError) else ""
    message = ("Trial input rejected: " + type(failure).__name__ + ". " + detail + " " + hint + "\n").encode("utf-8")
    event = {"operation": operation, "exit_code": 2, "failure_kind": "fixture-input"}
    input_bytes = json.dumps(sys.argv[1:], ensure_ascii=False).encode("utf-8")
    if operation == "write-answers" and len(sys.argv) != 3:
        input_bytes += globals().get("content", b"")
    event["input_sha256"] = digest(input_bytes)
    if operation == "hive":
        event["arguments"] = [arg[:2048] for arg in sys.argv[2:]][:32]
    emit(event, error=message)
    raise SystemExit(2)
