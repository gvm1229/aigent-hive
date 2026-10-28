"""Exercise actual setup terminals only in disposable hosted operating systems."""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import time

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run
from tests.conformance.support.harness import write_operational_user_setup


def hosted():
    return (os.environ.get("GITHUB_ACTIONS") == "true"
            and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"
            and sys.platform in {"win32", "linux", "darwin"})


def fake_codex(directory, environment):
    tree = ast.parse((ROOT / "tests/conformance/integration/test_connected_setup_lifecycle.py").read_text(encoding="utf-8"))
    klass = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "ConnectedSetupLifecycleConformance")
    method = next(n for n in klass.body if isinstance(n, ast.FunctionDef) and n.name == "_write_fake_host")
    body = next(n.value for n in ast.walk(method) if isinstance(n, ast.Constant)
                and isinstance(n.value, str) and n.value.startswith("import json"))
    program = directory / "codex.py"
    program.write_text(body.replace("__HIVE_TEST_HOST__", repr("codex"))
                       .replace("__HIVE_TEST_PRODUCT_VERSION__", repr("0.14.0")), encoding="utf-8")
    if os.name == "nt":
        (directory / "codex.cmd").write_text(f'@"{sys.executable}" "%~dp0codex.py" %*\r\n', encoding="utf-8")
    else:
        launcher = directory / "codex"
        launcher.write_text(f'#!{sys.executable}\nexec(compile(open({str(program)!r}, encoding="utf-8").read(), {str(program)!r}, "exec"))\n', encoding="utf-8")
        launcher.chmod(0o755)
    environment["HIVE_TEST_HOST_STATE"] = str(directory.parent / "host-state.json")
    environment["HIVE_TEST_HOST_LOG"] = str(directory.parent / "host-calls.jsonl")


def journal(user, project):
    paths = sorted((user / ".hive/config/setup-progress/projects").glob("*.yml")) if project else [user / ".hive/config/setup-progress/user.yml"]
    for path in paths:
        try:
            data = yaml.safe_load(path.read_text(encoding="utf-8"))
            if isinstance(data, dict):
                return data
        except (OSError, yaml.YAMLError):
            pass
    return {}


def posix_terminal(binary, arguments, environment, user, project=False, cancel=False, language="ko"):
    import pty
    import select
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execve(str(binary), [str(binary), *arguments], environment)
    output = bytearray()
    steps = []
    status = None
    replied = set()
    began = time.monotonic()
    try:
        while time.monotonic() - began < 120:
            if select.select([descriptor], [], [], 0.1)[0]:
                try:
                    output.extend(os.read(descriptor, 65536))
                except OSError:
                    pass
                if len(output) > 512 * 1024:
                    raise AssertionError("unexpected unbounded terminal output")
            state = journal(user, project)
            phase, step = state.get("phase"), state.get("step")
            key = (phase, step)
            if key not in replied and phase in {"awaiting-answers", "previewed"}:
                replied.add(key)
                steps.append({"phase": phase, "step": step})
                if cancel and step != "user-language" and phase == "awaiting-answers":
                    os.write(descriptor, b"\x04")
                else:
                    answer = ("yes" if phase == "previewed" else ("1" if language == "ko" else "2")
                              if step == "user-language" else "terminal-project" if step == "project-name" else "")
                    os.write(descriptor, (answer + "\n").encode())
            ended, raw = os.waitpid(pid, os.WNOHANG)
            if ended:
                status = os.waitstatus_to_exitcode(raw)
                break
        assert status is not None, "private terminal timed out"
        assert (status != 0 if cancel else status == 0), (status, output[-2000:].decode(errors="replace"))
    finally:
        if status is None:
            try:
                os.killpg(pid, 9)
                os.waitpid(pid, 0)
            except ProcessLookupError:
                pass
        os.close(descriptor)
    return {"terminal": "PTY", "exit_code": status, "steps": steps,
            "output_bytes": len(output), "output_sha256": hashlib.sha256(output).hexdigest()}


def windows_terminal(binary, user, project, work, environment, mode, label):
    from tests.conformance.contracts.test_setup_console_interrupt import compile_console_fixture
    helper = work / "console.exe"
    if not helper.exists():
        helper = compile_console_fixture(work)
    result = work / (label + ".json")
    args = [str(binary), str(user), str(result), mode]
    if mode == "project-quick":
        args += [str(project), "-", "terminal-project"]
    quote = lambda text: "'" + text.replace("'", "''") + "'"
    script = work / (label + ".ps1")
    script.write_text("$ErrorActionPreference='Stop'\n& " + quote(str(helper)) + " "
                      + " ".join(quote(a) for a in args) + "\nexit $LASTEXITCODE\n", encoding="utf-8")
    powershell = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    process = subprocess.run([str(powershell), "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", str(script)],
        env=environment, capture_output=True, timeout=150,
        creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup)
    assert process.returncode == 0, (process.returncode, process.stdout[-2000:], process.stderr[-2000:])
    return {"terminal": "PowerShell private console", **json.loads(result.read_text(encoding="utf-8"))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--user-root", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    parser.add_argument("--scenario", choices=["setup", "project", "cancel-resume", "unattended"], required=True)
    parser.add_argument("--language", choices=["ko", "en"], default="ko")
    args = parser.parse_args()
    if not hosted():
        parser.error("mutating qualification requires an isolated GitHub-hosted OS")
    contract = runpy.run_path(str(ROOT / "scripts/qualify-interactive-cli.py"))
    inspection = contract["Inspection"](args.fixture, args.binary)
    work, binary = inspection.fixture, inspection.binary
    user = contract["beneath"](args.user_root, work)
    result_path = contract["beneath"](args.result, work)
    assert not work.exists(), "never replace an earlier fixture"
    record = Run("격리 운영체제의 실제 CLI 흐름: " + args.scenario,
        [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
        paths=[work.relative_to(ROOT).as_posix()], binary=binary)
    code, events = 1, []
    details = {"scenario": args.scenario, "user_root": str(user), "language": args.language,
        "scope": "Actual terminal/CLI in isolated OS; fixed answers and fake Codex, no model or personal installation. Windows Terminal app qualification is separate."}
    if os.name == "nt" and args.scenario == "project":
        terminal_path = shutil.which("wt.exe")
        details["windows_terminal"] = {"executable": terminal_path, "available": terminal_path is not None,
                                       "execution": "not-run", "scope": "availability only; no Windows Terminal acceptance"}
    try:
        environment = inspection.environment()
        environment.update({"PYTHONUTF8": "1", "PYTHONNOUSERSITE": "1", "CARGO_INCREMENTAL": "0",
                            "HIVE_TEST_USER_ROOT": str(user), "PATHEXT": ".COM;.EXE;.BAT;.CMD"})
        if os.name == "nt":
            environment["COMSPEC"] = os.environ["COMSPEC"]
            # Native console infrastructure is available only after the isolated-OS gate.
            environment["SystemRoot"] = os.environ["SystemRoot"]
            environment["WINDIR"] = os.environ["SystemRoot"]
        user.mkdir(parents=True, exist_ok=True)
        fake_codex(inspection.fake_bin, environment)
        note = user / "README.txt"
        note.write_bytes(b"Synthetic private user note.\r\n")
        project = work / "project"
        project.mkdir()
        originals = {"AGENTS.md": b"Synthetic project instruction.\r\n", ".gitignore": b"foreign-cache/\r\n"}
        for name, data in originals.items():
            (project / name).write_bytes(data)
        base = ["setup", "--scope", "user", "--user-root", str(user)]

        def terminal(command, label, mode, *, is_project=False, cancel=False):
            event = (windows_terminal(binary, user, project, work, environment, mode, label) if os.name == "nt"
                     else posix_terminal(binary, command, environment, user, is_project, cancel, args.language))
            events.append({"command": [str(binary), *command], **event})

        if args.scenario == "project":
            write_operational_user_setup(user)
            settings_path = user / ".hive/config/user-setup.yml"
            values = yaml.safe_load(settings_path.read_text(encoding="utf-8"))
            values["interface_language"] = args.language
            values["usage_guard"]["enabled"] = False
            values["usage_guard"]["codexbar_fallback_enabled"] = False
            settings_path.write_text(yaml.safe_dump(values), encoding="utf-8")
            before = settings_path.read_bytes()
            terminal(["setup", "--target", str(project), "--user-root", str(user), "--interactive"],
                     "project", "project-quick", is_project=True)
            actual = yaml.safe_load((project / ".hive/setup-answers.yml").read_text(encoding="utf-8"))
            assert actual["project_name"] == "terminal-project"
            assert not actual["persistent_roles"] and not actual["approved_fallback_hooks"]
            assert settings_path.read_bytes() == before
            for name, data in originals.items():
                assert data in (project / name).read_bytes(), name
        elif args.scenario == "unattended":
            for command in [base + ["--non-interactive", "--output", "json"],
                            ["update", "--user-root", str(user), "--non-interactive", "--output", "json"]]:
                process = subprocess.run([str(binary), *command], env=environment, input=b"yes\nyes\n", capture_output=True, timeout=30)
                response = json.loads(process.stdout)
                assert process.returncode == 2 and response["status"] == "needs-input", response
                assert not (user / ".hive/config/user-setup.yml").exists()
                events.append({"command": [str(binary), *command], "exit_code": process.returncode, "code": response["code"]})
        else:
            previous = None
            if args.scenario == "cancel-resume":
                terminal(base + ["--section", "usage", "--interactive"], "cancel", "ctrl-c", cancel=True)
                previous = journal(user, False)
                assert previous["phase"] != "complete" and "user-language" in previous["answered_questions"]
                assert not (user / ".hive/config/user-setup.yml").exists()
            terminal(base + (["--resume"] if previous else []) + ["--interactive"], "complete",
                     "user-resume" if previous else "user-quick-" + args.language)
            actual = yaml.safe_load((user / ".hive/config/user-setup.yml").read_text(encoding="utf-8"))
            assert not actual["usage_guard"]["enforcement_enabled"]
            final = journal(user, False)
            assert final["phase"] == "complete"
            assert actual["interface_language"] == (previous["answers"]["interface_language"] if previous else args.language)
            if previous:
                assert final["operation_id"] == previous["operation_id"]
        assert note.read_bytes() == b"Synthetic private user note.\r\n"
        code = 0
    except Exception as error:
        details["error"] = str(error)[:4000]
        details["console_errors"] = {p.name: p.read_text(encoding="utf-8", errors="replace")[:4000]
            for p in work.glob("*.error.txt") if p.is_file() and not p.is_symlink()}
        raise
    finally:
        details["events"] = events
        result_path.parent.mkdir(parents=True, exist_ok=True)
        result_path.write_text(json.dumps({"exit_code": code, **details}, ensure_ascii=False, indent=2), encoding="utf-8")
        record.finish(code, details=details)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
