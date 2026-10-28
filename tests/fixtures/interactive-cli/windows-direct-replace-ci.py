"""Actual Windows update entry and file replacement with private, unpublished inputs."""
import ast
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

if not (os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires isolated GitHub-hosted Windows")
assert sys.argv[1:] in ([], ["npm"])

(ROOT / "tests/work").mkdir(exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="windows-direct-replace-", dir=ROOT / "tests/work")).resolve()
sha = lambda data: hashlib.sha256(data).hexdigest()
build_inputs = []


def build_variant(package, name):
    command = [shutil.which("cargo"), "build", "-p", "hive-cli", "--locked"]
    record = Run("직접 갱신의 비공개 패키지 표시 빌드: " + package, command, paths=["target/debug"])
    code = 1
    try:
        code = record.execute(command, env={**os.environ, "CARGO_INCREMENTAL": "0", "AIGENT_HIVE_PACKAGE_VERSION": package})
    finally:
        record.finish(code, details={"package_version": package, "scope": "private qualification binding, not a public tag or package"})
    assert code == 0
    target = work / name
    shutil.copyfile(ROOT / "target/debug/hive.exe", target)
    build_inputs.append({"package": package, "sha256": sha(target.read_bytes())})
    return target


baseline = build_variant("0.14.0-test", "baseline.exe")
incoming = build_variant("0.14.0-test.1", "incoming.exe")
fixture_tree = ast.parse((ROOT / "tests/conformance/integration/test_connected_setup_lifecycle.py").read_text(encoding="utf-8"))
fixture_class = next(n for n in fixture_tree.body if isinstance(n, ast.ClassDef) and n.name == "ConnectedSetupLifecycleConformance")
method = next(n for n in fixture_class.body if isinstance(n, ast.FunctionDef) and n.name == "_write_fake_host")
fake_host = next(n.value for n in ast.walk(method) if isinstance(n, ast.Constant)
                 and isinstance(n.value, str) and n.value.startswith("import json"))
fake_host = fake_host.replace("__HIVE_TEST_HOST__", repr("codex")).replace("__HIVE_TEST_PRODUCT_VERSION__", repr("0.14.0"))

RELAY = r'''
using System; using System.IO; using System.Diagnostics; using System.Reflection; using System.Text; using System.Security.Cryptography;
public static class DirectFixtureRelay {
    static string Plain(string value) { return Path.GetFullPath(value.StartsWith(@"\\?\") ? value.Substring(4) : value); }
    static string Quote(string value) {
        var text=new StringBuilder("\""); int slashes=0;
        foreach(char c in value) {
            if(c=='\\'){slashes++;continue;}
            if(c=='"'){text.Append('\\',slashes*2+1).Append(c);}else{text.Append('\\',slashes).Append(c);}
            slashes=0;
        }
        return text.Append('\\',slashes*2).Append('"').ToString();
    }
    static string Hash(string path) { using(var s=File.OpenRead(path))using(var h=SHA256.Create())return BitConverter.ToString(h.ComputeHash(s)); }
    public static int Main(string[] args) {
        string root=Directory.GetParent(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)).FullName;
        if(File.ReadAllText(Path.Combine(root,"fixture-only"))!="0.14.0-direct-fixture")return 20;
        if(args.Length!=14 || args[0]!="-NoLogo" || args[1]!="-NoProfile" || args[2]!="-NonInteractive" ||
            args[3]!="-ExecutionPolicy" || args[4]!="Bypass" || args[5]!="-File" || args[7]!="-Version" || args[8]!="0.14.0" ||
            args[9]!="-PackageVersion" || args[10]!="0.14.0-test.1" || args[11]!="-Prefix" || args[13]!="-DeferLockedBackup")return 21;
        if(!String.Equals(Plain(args[12]),Path.Combine(root,"prefix with spaces"),StringComparison.OrdinalIgnoreCase))return 22;
        string script=Plain(args[6]);
        if(!script.StartsWith(root+Path.DirectorySeparatorChar,StringComparison.OrdinalIgnoreCase) || Hash(script)!=Hash(Path.Combine(root,"installer.ps1")))return 23;
        string count=Path.Combine(root,"install-count");File.WriteAllText(count,(File.Exists(count)?Int32.Parse(File.ReadAllText(count))+1:1).ToString());
        string executable=Path.Combine(Environment.GetEnvironmentVariable("SystemRoot"),@"System32\WindowsPowerShell\v1.0\powershell.exe");
        var quoted=new string[args.Length];for(int i=0;i<args.Length;i++)quoted[i]=Quote(args[i]);
        var start=new ProcessStartInfo(executable,String.Join(" ",quoted)){UseShellExecute=false,CreateNoWindow=true};
        using(var process=Process.Start(start)){process.WaitForExit();return process.ExitCode;}
    }
}
'''


def run_case(name, interrupt):
    case = work / name
    user, prefix, shim = case / "user", case / "prefix with spaces", case / "fake-bin"
    for path in [user, prefix / "bin", prefix / "share/aigent-hive", shim]:
        path.mkdir(parents=True, exist_ok=True)
    binary = prefix / "bin/hive.exe"
    shutil.copyfile(baseline, binary)
    target = case / "incoming.exe"
    shutil.copyfile(incoming, target)
    (case / "fixture-only").write_text("0.14.0-direct-fixture")
    receipt = prefix / "share/aigent-hive/install-receipt.json"
    receipt.write_text(json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
        "version": "0.14.0", "package_version": "0.14.0-test", "artifact_sha256": "sha256:" + sha(binary.read_bytes())}))
    environment = {"PATH": str(shim), "HIVE_TEST_USER_ROOT": str(user), "HIVE_TEST_HOST_STATE": str(case / "host-state.json")}
    for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "CLAUDE_CONFIG_DIR", "GEMINI_CLI_HOME",
                "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "TEMP", "TMP", "NPM_CONFIG_PREFIX", "NPM_CONFIG_CACHE"]:
        path = case / key
        path.mkdir()
        environment[key] = str(path)
    for key in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"]:
        if key in os.environ:
            environment[key] = os.environ[key]
    environment["PSModuleAnalysisCachePath"] = str(case / "module-cache")
    (shim / "codex.py").write_text(fake_host, encoding="utf-8")
    (shim / "codex.cmd").write_text(f'@"{sys.executable}" "%~dp0codex.py" %*\r\n')
    (case / "relay.cs").write_text(RELAY)
    compiler = Path(os.environ["SystemRoot"]) / "Microsoft.NET/Framework64/v4.0.30319/csc.exe"
    compiled = subprocess.run([str(compiler), "/nologo", "/target:exe", "/out:" + str(shim / "powershell.exe"), str(case / "relay.cs")],
        env=environment, capture_output=True, timeout=30)
    assert compiled.returncode == 0, compiled.stdout + compiled.stderr
    shutil.copyfile(ROOT / "scripts/install.ps1", case / "publisher-source.ps1")
    installer = r'''param([string]$Version = "0.14.0",[string]$PackageVersion = "0.14.0-test.1",[string]$Prefix,[switch]$DeferLockedBackup)
$ErrorActionPreference='Stop'
Import-Module -Name @(($PSHOME+'\Modules\Microsoft.PowerShell.Utility\Microsoft.PowerShell.Utility.psd1'),($PSHOME+'\Modules\Microsoft.PowerShell.Management\Microsoft.PowerShell.Management.psd1'))
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$fixture=$env:HIVE_TEST_UPDATE_FIXTURE
$tokens=$null;$errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile([IO.Path]::Combine($fixture,'publisher-source.ps1'),[ref]$tokens,[ref]$errors)
if($errors){throw 'Publisher source is invalid'}
foreach($fn in $tree.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)){Invoke-Expression $fn.Extent.Text}
$candidate=[IO.Path]::Combine($fixture,'incoming.exe')
if((Get-FileSha256 -Path $candidate) -ne '__INCOMING_SHA__'){throw 'Fixture candidate changed'}
if(-not (Test-HiveVersionOutput -Output (& $candidate --version) -ExpectedVersion $Version -ExpectedPackageVersion $PackageVersion)){throw 'Fixture candidate version differs'}
Publish-DirectInstall -Binary $candidate -Prefix $Prefix -Version $Version -PackageVersion $PackageVersion -DeferLockedBackup:$DeferLockedBackup
'''.replace("__INCOMING_SHA__", sha(target.read_bytes()))
    (case / "installer.ps1").write_text(installer, encoding="utf-8")
    (case / "registry.json").write_text(json.dumps({"dist-tags": {"latest": "0.13.1", "test": "0.14.0-test.1"},
        "versions": {"0.13.1": {"aigentHive": {"productVersion": "0.13.1"}},
                     "0.14.0-test.1": {"aigentHive": {"productVersion": "0.14.0"}}}}))
    events = []
    record = Run("격리 Windows 실제 직접 교체와 종료 후 정리: " + name,
        [sys.executable, str(Path(__file__).resolve()), name], paths=[case.relative_to(ROOT).as_posix()], binary=binary)
    code = 1

    def invoke(*args, expected=0, extra=None):
        result = subprocess.run([str(binary), *args, "--output", "json"], env={**environment, **(extra or {})},
            input=b"", capture_output=True, timeout=180)
        response = json.loads(result.stdout) if result.stdout else {}
        events.append({"arguments": list(args), "exit_code": result.returncode, "result": response,
                       "stderr_sha256": sha(result.stderr)})
        assert result.returncode == expected, (args, response, result.stderr[-1200:])
        return response

    try:
        settings = invoke("setup", "--scope", "user", "--describe")["data"]["expedited_defaults"]
        settings.update(interface_language="ko", selected_hosts=["codex"])
        settings["wiki"]["enabled"] = False
        answers = case / "answers.json"
        answers.write_text(json.dumps(settings))
        invoke("install", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
        invoke("setup", "--scope", "user", "--user-root", str(user), "--answers", str(answers), "--apply")
        config = user / ".hive/config/user-setup.yml"
        before_config = config.read_bytes()
        neighbor = prefix / "neighbor.txt"
        neighbor.write_bytes(b"Preserve unrelated prefix content.\n")
        environment["HIVE_TEST_UPDATE_FIXTURE"] = str(case)
        if interrupt:
            environment["HIVE_TEST_DIRECT_REPLACEMENT_EXIT_AFTER"] = "backup"
        result = invoke("update", "--channel", "test", "--confirm", "--user-root", str(user))
        assert result["code"] == "hive.update-complete", result
        assert result["data"]["steps"]["core"] == "verified" and result["data"]["steps"]["harness"] == "verified", result
        cleanup = result["data"]["cleanup"]
        assert cleanup["state"] == "after-process-exit", cleanup
        assert result["data"]["from_version"] == "0.14.0-test" and result["data"]["to_version"] == "0.14.0-test.1"
        backup = prefix / cleanup["backup"]
        pending = prefix / cleanup["record"]
        lease = pending.parent / "replacement-operation.lock"
        assert backup.parent.samefile(prefix / "bin") and pending.parent.samefile(prefix / "share/aigent-hive")
        until = time.monotonic() + 40
        while (backup.exists() or lease.exists()) and time.monotonic() < until:
            time.sleep(.1)
        assert not backup.exists(), "old image cleanup did not finish"
        assert not lease.exists(), "cleanup worker still holds its operation lease"
        if interrupt:
            assert pending.exists(), "interrupted cleanup lost its recovery record"
            resumed = invoke("update", "--user-root", str(user))
            assert resumed["data"]["recovered"] and resumed["data"]["cleanup"]["state"] == "complete", resumed
        else:
            while pending.exists() and time.monotonic() < until:
                time.sleep(.1)
        assert not pending.exists()
        assert not lease.exists()
        assert (case / "install-count").read_text() == "1"
        assert sha(binary.read_bytes()) == sha(target.read_bytes())
        assert config.read_bytes() == before_config and neighbor.read_bytes() == b"Preserve unrelated prefix content.\n"
        assert json.loads(receipt.read_text())["package_version"] == "0.14.0-test.1"
        code = 0
    except BaseException as error:
        events.append({"failure": str(error)[:2500]})
        raise
    finally:
        record.finish(code, details={"build_inputs": build_inputs, "events": events,
            "input_sha256": {p.name: sha(p.read_bytes()) for p in [case / "registry.json", case / "installer.ps1", case / "publisher-source.ps1", target]},
            "scope": "GitHub-hosted Windows, actual public update command, actual frozen source-debug images and publisher/cleanup, fixed metadata and transport plus Codex plugin stand-in. Private14-test baseline and planned14-test.1 target, no publication. Not public signing/registry, desktop control or model acceptance."})


if not sys.argv[1:]:
    run_case("normal", False)
    run_case("interrupted", True)


def run_npm_case():
    case = work / "npm"
    user, prefix, shim = case / "user", case / "prefix with spaces", case / "fake-bin"
    for path in [user, prefix, shim]:
        path.mkdir(parents=True)
    node = Path(shutil.which("node")).resolve()
    npm_cmd = Path(shutil.which("npm.cmd")).absolute()
    npm_js = npm_cmd.parent / "node_modules/npm/bin/npm-cli.js"
    assert node.is_file() and npm_js.is_file()
    environment = {"PATH": str(shim), "HIVE_TEST_USER_ROOT": str(user),
                   "HIVE_TEST_HOST_STATE": str(case / "host-state.json")}
    for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "CLAUDE_CONFIG_DIR",
                "GEMINI_CLI_HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "TEMP", "TMP",
                "NPM_CONFIG_CACHE"]:
        path = case / key
        path.mkdir()
        environment[key] = str(path)
    for key in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"]:
        if key in os.environ:
            environment[key] = os.environ[key]
    environment.update(NPM_CONFIG_PREFIX=str(prefix), NPM_CONFIG_USERCONFIG=str(case / "user.npmrc"),
        NPM_CONFIG_GLOBALCONFIG=str(case / "global.npmrc"), NPM_CONFIG_OFFLINE="true",
        NPM_CONFIG_AUDIT="false", NPM_CONFIG_FUND="false", NPM_CONFIG_IGNORE_SCRIPTS="true")
    (shim / "codex.py").write_text(fake_host, encoding="utf-8")
    (shim / "codex.cmd").write_text(f'@"{sys.executable}" "%~dp0codex.py" %*\r\n')
    (case / "fixture-only").write_text("0.14.0-direct-fixture")
    (case / "registry.json").write_bytes((work / "normal/registry.json").read_bytes())
    events, packages = [], []
    record = Run("격리 Windows 실제 npm 갱신과 같은 버전 보존", [sys.executable, str(Path(__file__).resolve()), "npm"],
        paths=[case.relative_to(ROOT).as_posix()], binary=baseline)
    code = 1

    def npm_call(*args, cwd=case):
        result = subprocess.run([str(node), str(npm_js), *args], env=environment, cwd=cwd,
            capture_output=True, text=True, encoding="utf-8", timeout=90)
        assert result.returncode == 0, (args, result.stdout[-1500:], result.stderr[-1500:])
        return result.stdout.strip()

    def pack(image, version):
        staging = case / version
        native = staging / "node_modules/@aigent-hive/win32-x64"
        (native / "bin").mkdir(parents=True)
        (staging / "bin").mkdir()
        shutil.copyfile(image, native / "bin/hive.exe")
        (native / "package.json").write_text(json.dumps({"name": "@aigent-hive/win32-x64", "version": version,
            "aigentHive": {"productVersion": "0.14.0"}}))
        shutil.copyfile(ROOT / "packaging/npm/bin/hive.cjs", staging / "bin/hive.cjs")
        (staging / "package.json").write_text(json.dumps({"name": "aigent-hive", "version": version,
            "aigentHive": {"productVersion": "0.14.0"}, "bin": {"hive": "bin/hive.cjs"},
            "dependencies": {"@aigent-hive/win32-x64": version}, "bundleDependencies": ["@aigent-hive/win32-x64"]}))
        archive = case / json.loads(npm_call("pack", "--json", "--pack-destination", str(case), cwd=staging))[0]["filename"]
        packages.append({"version": version, "archive_sha256": sha(archive.read_bytes()), "image_sha256": sha(image.read_bytes())})
        return archive

    def invoke(*args):
        result = subprocess.run([str(node), str(prefix / "node_modules/aigent-hive/bin/hive.cjs"), *args, "--output", "json"],
            env=environment, input=b"", capture_output=True, timeout=180)
        response = json.loads(result.stdout) if result.stdout else {}
        events.append({"arguments": list(args), "exit_code": result.returncode, "result": response,
                       "stderr_sha256": sha(result.stderr)})
        assert result.returncode == 0, (args, response, result.stderr[-1500:])
        return response

    try:
        packages.append({"npm_version": npm_call("--version"), "node_sha256": sha(node.read_bytes()),
                         "npm_entry_sha256": sha(npm_js.read_bytes())})
        first, target = pack(baseline, "0.14.0-test"), pack(incoming, "0.14.0-test.1")
        npm_call("install", "--global", "--prefix", str(prefix), str(first))
        assert Path(npm_call("prefix", "--global")).samefile(prefix)
        assert Path(npm_call("root", "--global")).samefile(prefix / "node_modules")
        native = prefix / "node_modules/aigent-hive/node_modules/@aigent-hive/win32-x64/bin/hive.exe"
        assert sha(native.read_bytes()) == sha(baseline.read_bytes())
        foreign = prefix / "node_modules/hive-test-foreign"
        foreign.mkdir()
        (foreign / "package.json").write_text('{"name":"hive-test-foreign","version":"1.0.0"}')
        (foreign / "README.md").write_bytes(b"Preserve unrelated npm bytes.\n")
        preserved = {path: path.read_bytes() for path in foreign.iterdir()}
        settings = invoke("setup", "--scope", "user", "--describe")["data"]["expedited_defaults"]
        settings.update(interface_language="ko", selected_hosts=["codex"])
        settings["wiki"]["enabled"] = False
        answers = case / "answers.json"
        answers.write_text(json.dumps(settings))
        invoke("install", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply")
        invoke("setup", "--scope", "user", "--user-root", str(user), "--answers", str(answers), "--apply")
        config = user / ".hive/config/user-setup.yml"
        preserved[config] = config.read_bytes()
        relay = '''import hashlib,os,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parents[1]
assert sys.argv[1:]==['install','-g','aigent-hive@0.14.0-test.1']
assert Path(os.environ['NPM_CONFIG_PREFIX']).samefile(root/'prefix with spaces')
archive=root/'aigent-hive-0.14.0-test.1.tgz'
assert hashlib.sha256(archive.read_bytes()).hexdigest()==__HASH__
count=root/'install-count'
count.write_text(str(int(count.read_text())+1 if count.exists() else 1))
raise SystemExit(subprocess.run([__NODE__,__NPM__,'install','-g',str(archive)],env=os.environ).returncode)
'''.replace("__HASH__", repr(sha(target.read_bytes()))).replace("__NODE__", repr(str(node))).replace("__NPM__", repr(str(npm_js)))
        ast.parse(relay)
        (shim / "npm.py").write_text(relay, encoding="utf-8")
        (shim / "npm.cmd").write_text(f'@"{sys.executable}" "%~dp0npm.py" %*\r\n')
        environment["HIVE_TEST_UPDATE_FIXTURE"] = str(case)
        result = invoke("update", "--channel", "test", "--confirm", "--user-root", str(user))
        assert result["code"] == "hive.update-complete", result
        assert result["data"]["steps"]["core"] == "verified" and result["data"]["steps"]["harness"] == "verified", result
        assert result["data"]["from_version"] == "0.14.0-test" and result["data"]["to_version"] == "0.14.0-test.1"
        assert sha(native.read_bytes()) == sha(incoming.read_bytes())
        assert json.loads((prefix / "node_modules/aigent-hive/package.json").read_text())["version"] == "0.14.0-test.1"
        repeat = invoke("update", "--channel", "test", "--confirm", "--user-root", str(user))
        assert repeat["code"] == "hive.update-complete" and repeat["data"]["steps"]["core"] == "unchanged", repeat
        assert (case / "install-count").read_text() == "1"
        assert all(path.read_bytes() == value for path, value in preserved.items())
        code = 0
    except BaseException as error:
        events.append({"failure": str(error)[:2500]})
        raise
    finally:
        record.finish(code, details={"events": events, "packages": packages, "build_inputs": build_inputs,
            "scope": "GitHub-hosted Windows, actual Node/npm local package installation and public hive.cjs/update entry; private cache/prefix and exact local archive transport, Codex stand-in. No public registry/signing or actual app/model acceptance."})


if sys.argv[1:] == ["npm"]:
    run_npm_case()
