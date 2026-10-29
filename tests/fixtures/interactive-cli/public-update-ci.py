"""Original public updater to exact numbered test, only in disposable hosted OSes."""
import argparse
import base64
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import time
import urllib.request

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run


def public_image_hash(platform_name, version):
    with urllib.request.urlopen(f"https://registry.npmjs.org/@aigent-hive/{platform_name}/{version}", timeout=30) as response:
        metadata = json.load(response)
    assert metadata["version"] == version
    url = metadata["dist"]["tarball"]
    assert url.startswith("https://registry.npmjs.org/@aigent-hive/")
    with urllib.request.urlopen(url, timeout=60) as response:
        data = response.read(128 * 1024 * 1024 + 1)
    assert len(data) <= 128 * 1024 * 1024
    assert "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode() == metadata["dist"]["integrity"]
    name = "hive.exe" if platform_name.startswith("win32") else "hive"
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        member = archive.getmember("package/bin/" + name)
        assert member.isfile() and member.size <= 128 * 1024 * 1024
        image = archive.extractfile(member).read()
    return hashlib.sha256(image).hexdigest(), metadata["dist"]["integrity"]


def original_identity(platform_name, version):
    windows = platform_name == "win32-x64"
    registry = ROOT / "harness/setup" / ("legacy-update-images.json" if windows else "legacy-update-posix-images.json")
    entries = json.loads(registry.read_text(encoding="utf-8"))["images"]
    entry = next(row for row in entries if row["version"] == version
                 and (windows or row["platform"] == platform_name))
    return entry["binary_sha256"], entry["package_integrity"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--previous-version", choices=["0.13.1", "0.13.2-test.2"], required=True)
    parser.add_argument("--owner", choices=["npm", "direct"], required=True)
    parser.add_argument("--redirected", action="store_true")
    args = parser.parse_args()
    if not (os.environ.get("GITHUB_ACTIONS") == "true" and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
        parser.error("public installation/update acceptance requires an isolated hosted OS")
    assert re.fullmatch(r"0\.14\.0-test\.[1-9][0-9]*", args.package_version)
    work = args.fixture.resolve()
    assert work.is_relative_to((ROOT / "tests/work").resolve()) and work != ROOT / "tests/work" and not work.exists()
    windows = os.name == "nt"
    architecture = "arm64" if platform.machine().lower() in {"arm64", "aarch64"} else "x64"
    platform_name = ("win32" if windows else sys.platform) + "-" + architecture
    executable = "hive.exe" if windows else "hive"
    node = Path(shutil.which("node")).resolve()
    npm = Path(shutil.which("npm.cmd" if windows else "npm")).absolute()
    npm_js = npm.parent / "node_modules/npm/bin/npm-cli.js" if windows else npm.resolve()
    assert node.is_file() and npm_js.is_file()
    record = Run("공개 원본의 실제 갱신·보존 제거: " + args.previous_version + " " + args.owner,
        [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]], paths=[work.relative_to(ROOT).as_posix()])
    events, details, code = [], {"previous_version": args.previous_version, "package_version": args.package_version,
        "owner": args.owner, "redirected": args.redirected, "platform_package": platform_name}, 1
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        with urllib.request.urlopen("https://registry.npmjs.org/-/package/aigent-hive/dist-tags", timeout=30) as response:
            tags = json.load(response)
        assert tags["test"] == args.package_version, "requested version is not the current public test"
        stable = json.loads((ROOT / "docs/public-stable-release.json").read_text(encoding="utf-8"))["stable_version"]
        assert tags["latest"] == stable, "public test must not change latest"
        wanted_hash, wanted_integrity = public_image_hash(platform_name, args.package_version)
        original_hash, original_integrity = public_image_hash(platform_name, args.previous_version)
        assert (original_hash, original_integrity) == original_identity(platform_name, args.previous_version)
        details.update(original_image_sha256=original_hash, original_integrity=original_integrity,
                       target_image_sha256=wanted_hash, target_integrity=wanted_integrity, registry_tags=tags)
        work.mkdir(parents=True)
        user, project, fake_bin, prefix = work / "user", work / "project", work / "fake-bin", work / "npm-prefix"
        for path in [user, project, fake_bin, prefix]:
            path.mkdir()
        env = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "PYTHONUTF8": "1", "PYTHONNOUSERSITE": "1",
               "CARGO_INCREMENTAL": "0", "NPM_CONFIG_AUDIT": "false", "NPM_CONFIG_FUND": "false",
               "NPM_CONFIG_REGISTRY": "https://registry.npmjs.org", "HIVE_TEST_HOST_LOG": str(work / "host.jsonl"),
               "HIVE_TEST_USER_ROOT": str(user), "HIVE_PUBLIC_FIXTURE_ROOT": str(work)}
        for key, relative in {"HOME": "home", "USERPROFILE": "home", "APPDATA": "home/roaming",
            "LOCALAPPDATA": "home/local", "CODEX_HOME": "home/codex", "CLAUDE_CONFIG_DIR": "home/claude",
            "GEMINI_CLI_HOME": "home/gemini", "XDG_CONFIG_HOME": "home/config", "XDG_DATA_HOME": "home/data",
            "XDG_CACHE_HOME": "home/cache", "TEMP": "tmp", "TMP": "tmp", "NPM_CONFIG_PREFIX": "npm-prefix",
            "NPM_CONFIG_CACHE": "npm-cache"}.items():
            path = work / relative
            path.mkdir(parents=True, exist_ok=True)
            env[key] = str(path)
        for key, name in {"NPM_CONFIG_USERCONFIG": "user.npmrc", "NPM_CONFIG_GLOBALCONFIG": "global.npmrc"}.items():
            path = work / name
            path.write_text("", encoding="utf-8")
            env[key] = str(path)
        system_paths = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
        if windows:
            env.update({k: os.environ[k] for k in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"] if k in os.environ})
            system_paths = [str(Path(os.environ["SystemRoot"]) / relative)
                            for relative in ["System32", "System32/WindowsPowerShell/v1.0"]]
        env["PATH"] = os.pathsep.join(dict.fromkeys([str(fake_bin), str(node.parent), str(npm.parent), *system_paths]))
        stub = ROOT / "tests/fixtures/native-orchestration/agy-stub.py"
        wrapper = r"""import os,runpy,shutil
from pathlib import Path
def normalized(path):
    resolved=Path(path).resolve()
    return Path(str(resolved).removeprefix("\\\\?\\")) if os.name=="nt" else resolved
root=normalized(os.environ['HIVE_PUBLIC_FIXTURE_ROOT'])
remove,copy=shutil.rmtree,shutil.copytree
def confined(path):
    resolved=normalized(path)
    assert resolved.is_relative_to(root) and resolved!=root
def safe_remove(path,*args,**kwargs):
    confined(path);return remove(path,*args,**kwargs)
def safe_copy(source,target,*args,**kwargs):
    confined(source);confined(target);return copy(source,target,*args,**kwargs)
shutil.rmtree,shutil.copytree=safe_remove,safe_copy
runpy.run_path(STUB,run_name='__main__')
""".replace("STUB", repr(str(stub)))
        (fake_bin / "agy.py").write_text(wrapper, encoding="utf-8")
        if windows:
            (fake_bin / "agy.cmd").write_text(f'@"{sys.executable}" "%~dp0agy.py" %*\r\n', encoding="utf-8")
        else:
            launcher = fake_bin / "agy"
            launcher.write_text(f"#!{sys.executable}\n" + wrapper, encoding="utf-8")
            launcher.chmod(0o755)

        def invoke(command, *, timeout=180, expected=(0,), environment=None):
            started = time.monotonic()
            result = subprocess.run(list(map(str, command)), env=environment or env, cwd=work,
                input=b"", capture_output=True, timeout=timeout)
            events.append({"command": list(map(str, command)), "exit_code": result.returncode,
                "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(), "stderr_sha256": hashlib.sha256(result.stderr).hexdigest(),
                "elapsed_seconds": time.monotonic() - started})
            assert result.returncode in expected, (command, result.returncode, result.stdout[-3000:], result.stderr[-3000:])
            return result

        def npm_call(*flags):
            return invoke([node, npm_js, *flags])

        npm_call("install", "--global", "--prefix", prefix, "aigent-hive@" + args.previous_version)
        npm_root = Path(npm_call("root", "--global").stdout.decode().strip()).resolve()
        assert npm_root in {prefix / "node_modules", prefix / "lib/node_modules"}
        package = npm_root / "aigent-hive"
        old = package / "node_modules/@aigent-hive" / platform_name / "bin" / executable
        assert old.is_file()
        assert sha(old) == original_hash

        def direct_install(version, installer_package):
            product = version.split("-", 1)[0]
            direct = work / "direct"
            if windows:
                powershell = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
                invoke([powershell, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File",
                    installer_package / "install.ps1", "-Version", product, "-PackageVersion", version, "-Prefix", direct])
            else:
                invoke(["/bin/sh", installer_package / "install.sh"], environment={**env,
                    "AIGENT_HIVE_VERSION": product, "AIGENT_HIVE_PACKAGE_VERSION": version, "AIGENT_HIVE_PREFIX": str(direct)})
            return direct / "bin" / executable

        if args.owner == "direct":
            old = direct_install(args.previous_version, package)
            assert sha(old) == original_hash
        entry = [node, package / "bin/hive.cjs"] if args.owner == "npm" else [old]
        details["initial_version"] = invoke([*entry, "--version"]).stdout.decode().strip()
        choices = {"schema_version": 1, "interface_language": "en", "selected_hosts": ["antigravity"],
            "wiki": {"enabled": True, "language": "en", "backend": "markdown"},
            "profile": {"contexts": ["non-developer"]}, "persona": {"id": "balanced"},
            "skills": {"mode": "all"}, "update_check": {"enabled": False}, "judge_invocation": "explicit",
            "usage_guard": {"enabled": False, "stop_remaining_percent": 20,
                "codexbar_fallback_enabled": False, "discord": {"enabled": False}}}
        answers = work / "old-user.json"
        answers.write_text(json.dumps(choices), encoding="utf-8")
        invoke([*entry, "setup", "--scope", "user", "--user-root", user, "--quick-answers", answers, "--apply", "--output", "json"])
        project_answers = json.loads((ROOT / "tests/fixtures/interactive-cli/model-fixed-choices.json").read_text())["project-quick"]
        project_answers.update(schema_version=1, primary_host="antigravity")
        answers.write_text(json.dumps(project_answers), encoding="utf-8")
        invoke([*entry, "setup", "--target", project, "--user-root", user, "--answers", answers,
            "--capabilities", ROOT / "tests/fixtures/setup/capabilities-antigravity-absent.json", "--apply", "--output", "json"])
        runtime_note = user / ".hive/runtime/usage-monitor/incidents/acceptance.json"
        history_note = user / ".hive/history/usage-monitor/acceptance.json"
        sentinels = {user / "README.txt": b"Synthetic user bytes.\r\n",
            user / ".hive/knowledge/acceptance.md": b"# Synthetic acceptance knowledge\n",
            runtime_note: b'{"synthetic_preservation_fixture":true}\n',
            history_note: b'{"synthetic_durable_history_fixture":true}\n',
            project / ".hive/team/roles/user-note.md": b"# Synthetic user role note\nPreserve this body.\n"}
        for path, data in sentinels.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        project_before = {p.relative_to(project).as_posix(): sha(p) for p in project.rglob("*") if p.is_file()}
        if sys.platform == "darwin" and not args.redirected:
            probe = work / "mac-parent-probe"
            invoke(["/usr/bin/swiftc", "-module-cache-path", work / "swift-cache",
                ROOT / "tests/fixtures/interactive-cli/macos-parent-probe.swift", "-o", probe])
            registry = json.loads((ROOT / "harness/setup/legacy-update-posix-images.json").read_text(encoding="utf-8"))
            original = next(row for row in registry["images"] if row["platform"] == platform_name and row["version"] == args.previous_version)
            details["terminal_trace"] = {"mac_probe": str(probe), "original_cdhash": original["code_directory_hash"], "probe_sha256": sha(probe)}
        update = [*entry, "update", "--channel", "test", "--user-root", user]
        if args.redirected:
            update.append("--confirm")
            invoke(update, timeout=610)
        else:
            spec = importlib.util.spec_from_file_location("terminal_setup", ROOT / "tests/fixtures/interactive-cli/terminal-setup-ci.py")
            terminal = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(terminal)
            if windows:
                from tests.conformance.contracts.test_setup_console_interrupt import compile_console_fixture
                helper = compile_console_fixture(work)
                result = work / "update-console.json"
                mode = "public-update-npm" if args.owner == "npm" else "public-update"
                command = [helper, entry[0], user, result, mode, *([entry[1]] if len(entry) == 2 else [])]
                startup = subprocess.STARTUPINFO()
                startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                startup.wShowWindow = subprocess.SW_HIDE
                process = subprocess.run(list(map(str, command)), env=env, cwd=work, capture_output=True, timeout=610,
                    creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup)
                assert process.returncode == 0, (process.returncode, process.stderr[-3000:])
                outcome = json.loads(result.read_text())
                assert outcome["confirmed"] and outcome["questions"] >= 2 and outcome["exit_code"] == 0, outcome
                events.append({"command": list(map(str, update)), "terminal": "private Windows console", **outcome})
            else:
                outcome = terminal.posix_terminal(Path(entry[0]), list(map(str, update[1:])), env, user,
                    preserve_legacy=True, timeout_seconds=610, language="en", trace=details.setdefault("terminal_trace", {}))
                assert any(s["step"] == "usage-enforcement" for s in outcome["steps"]), outcome
                events.append({"command": list(map(str, update)), **outcome})
        current = old
        assert current.is_file() and sha(current) == wanted_hash
        details.update(final_image_sha256=sha(current), target_integrity=wanted_integrity,
                       final_version=invoke([*entry, "--version"]).stdout.decode().strip())
        product, revision = args.package_version.split("-test.")
        assert details["final_version"].startswith(f"AIgent Hive v{product}-test #{revision} · developer test build (released ")
        shown = json.loads(invoke([current, "setup", "--scope", "user", "--user-root", user, "--show", "--output", "json"]).stdout)
        settings = shown["data"]["stored"]
        for key in ["interface_language", "selected_hosts", "wiki", "profile", "persona", "skills", "update_check", "judge_invocation"]:
            assert settings[key] == choices[key], key
        guard = settings["usage_guard"]
        assert guard["stop_remaining_percent"] == 20 and not guard["codexbar_fallback_enabled"]
        if settings["schema_version"] == 2:
            assert guard["preflight"]["zero_quota_enabled"] and not guard["preflight"]["threshold_enabled"]
            assert not guard["preflight"]["reset_enabled"] and not guard["midway"]["enabled"]
        else:
            assert args.redirected and not guard["enabled"]
        if not args.redirected:
            progress = yaml.safe_load((user / ".hive/config/setup-progress/user.yml").read_text(encoding="utf-8"))
            assert progress["phase"] == "complete"
        assert {p.relative_to(project).as_posix(): sha(p) for p in project.rglob("*") if p.is_file()} == project_before
        for path, data in sentinels.items():
            assert path.read_bytes() == data
        details["runtime_preserved_through_update"] = True
        config = user / ".hive/config/user-setup.yml"
        sentinels[config] = config.read_bytes()
        preview = json.loads(invoke([*entry, "uninstall", "--user-root", user, "--dry-run", "--output", "json"]).stdout)
        assert preview["data"]["preview"]["core"]["owner"] == args.owner
        entries = preview["data"]["preview"]["entries"]
        assert entries[runtime_note.relative_to(user).as_posix()]["kind"] == "file"
        assert history_note.relative_to(user).as_posix() not in entries
        assert current.is_file() and sha(current) == wanted_hash
        refused = invoke([*entry, "uninstall", "--user-root", user, "--output", "json"], expected=(2,))
        assert json.loads(refused.stdout)["code"] == "hive.uninstall-needs-input"
        assert current.is_file() and sha(current) == wanted_hash
        for path, data in sentinels.items():
            assert path.read_bytes() == data
        # Observation runtime is displayed for removal; durable history is retained.
        sentinels.pop(runtime_note)
        removed = json.loads(invoke([*entry, "uninstall", "--user-root", user, "--confirm", "--output", "json"], expected=(0, 2)).stdout)
        receipt = (user / removed["data"]["receipt"]).resolve()
        assert receipt.is_relative_to(user / ".hive/history/lifecycle")
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            completion = json.loads(receipt.read_text())
            if completion["phase"] == "complete":
                break
            time.sleep(0.1)
        assert completion["phase"] == "complete" and not current.exists(), completion
        assert not runtime_note.exists()
        details["runtime_removed_only_after_confirmation"] = True
        for path, data in sentinels.items():
            assert path.read_bytes() == data
        assert {p.relative_to(project).as_posix(): sha(p) for p in project.rglob("*") if p.is_file()} == project_before
        npm_call("install", "--global", "--prefix", prefix, "aigent-hive@" + args.package_version)
        if args.owner == "direct":
            current = direct_install(args.package_version, package)
        assert sha(current) == wanted_hash
        reinstall = [*entry, "install", "--scope", "user", "--host", "antigravity", "--user-root", user, "--apply", "--output", "json"]
        restored = invoke(reinstall, expected=(0, 5))
        if restored.returncode == 5:
            refusal = json.loads(restored.stdout)
            assert refusal["code"] == "hive.user-install-verification-failed"
            assert "index recovery needs an explicit hive index rebuild --user-root action" in refusal["message"]
            assert all(path.read_bytes() == data for path, data in sentinels.items())
            assert {p.relative_to(project).as_posix(): sha(p) for p in project.rglob("*") if p.is_file()} == project_before
            # This fixture explicitly authorizes its own registered synthetic user/project store.
            rebuilt = json.loads(invoke([current, "index", "rebuild", "--user-root", user, "--output", "json"]).stdout)
            assert rebuilt["code"] == "hive.index-rebuilt"
            details["reinstall_index_recovery"] = {"initial_refusal": refusal["code"], "explicit_user_root_rebuild": True}
            invoke(reinstall)
        assert json.loads((work / "host.jsonl.antigravity.state").read_text(encoding="utf-8"))["plugin"]
        for path, data in sentinels.items():
            assert path.read_bytes() == data
        assert {p.relative_to(project).as_posix(): sha(p) for p in project.rglob("*") if p.is_file()} == project_before
        assert not (user / ".hive/runtime/usage-monitor/config.json").exists()
        details["preserved_sha256"] = {p.relative_to(work).as_posix(): hashlib.sha256(data).hexdigest() for p, data in sentinels.items()}
        details["project_tree_unchanged"] = True
        code = 0
    except Exception as error:
        details["error"] = str(error)[:5000]
        if "old" in locals() and old.is_file():
            details["image_after_failure_sha256"] = sha(old)
            if sha(old) == wanted_hash:
                try:
                    state = subprocess.run([str(old), "setup", "--scope", "user", "--user-root", str(user), "--show", "--summary", "--output", "json"],
                        env=env, cwd=work, input=b"", capture_output=True, timeout=15)
                    details["state_after_failure"] = {"exit_code": state.returncode, "stdout": state.stdout.decode(errors="replace")[:5000]}
                except Exception as diagnostic_error:
                    details["state_diagnostic_error"] = str(diagnostic_error)[:1000]
        details["npm_moves"] = []
        for log in sorted((work / "npm-cache/_logs").glob("*.log"), key=lambda p: p.stat().st_mtime)[-2:]:
            if not log.resolve().is_relative_to(work) or log.is_symlink():
                continue
            lines = log.read_text(encoding="utf-8", errors="replace").splitlines()
            selected = set()
            for index, line in enumerate(lines):
                if any(word in line.lower() for word in ["reify moves", "retired", "rename", "error"]):
                    selected.update(range(index, min(index + 10, len(lines))))
            details["npm_moves"].append({"file": log.name, "lines": [lines[n][:500] for n in sorted(selected)[:70]]})
        details["console_errors"] = {p.name: p.read_text(encoding="utf-8", errors="replace")[-4000:]
                                     for p in work.glob("*.error.txt") if p.is_file() and not p.is_symlink()}
        raise
    finally:
        details.update(events=events, scope="Real public binaries, package managers and original updater in isolated OS. Antigravity is a confined stand-in; history/role notes are synthetic preservation inputs, not live interruption evidence. No model, private user installation or stable publication.")
        record.finish(code, details=details)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
