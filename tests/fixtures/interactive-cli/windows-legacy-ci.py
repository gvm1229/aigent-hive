"""Frozen Windows direct updater, private console and fixed synthetic installer/host."""
import ast
import base64
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run

if not (os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true"
        and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"):
    raise SystemExit("requires an isolated GitHub-hosted Windows VM")

(ROOT / "tests/work").mkdir(exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="windows-legacy-ci-", dir=ROOT / "tests/work")).resolve()
user, prefix, shim = work / "user", work / "prefix", work / "shim"
for path in [user, prefix / "bin", prefix / "share/aigent-hive", shim, work / "incoming"]:
    path.mkdir(parents=True, exist_ok=True)
candidate = ROOT / "target/debug/hive.exe"
diagnostic = sys.argv[1:] == ["--diagnostic"]
assert not sys.argv[1:] or diagnostic
diagnostic_sources = []
if diagnostic:
    diagnostic_source = ROOT / "crates/hive-cli/src/main.rs"
    original_source = diagnostic_source.read_bytes()
    needle = b"fn emit_action_result(result: &ActionResult) -> ExitCode {"
    assert original_source.count(needle) == 1
    replacement = needle + b'''
    if let Some(root) = std::env::args().collect::<Vec<_>>().windows(2)
        .find(|pair| pair[0] == "--user-root")
        .map(|pair| std::path::PathBuf::from(&pair[1])) {
        let _ = std::fs::write(root.join(".hive/runtime/update/bridge-diagnostic.txt"),
            format!("{}: {}", result.code, result.message));
    }
'''
    patched_source = original_source.replace(needle, replacement)
    diagnostic_sources.append((diagnostic_source, original_source, patched_source))
    for relative, before, after in [
        ("crates/hive-cli/src/update_bridge/windows.rs",
         b'"legacy updater parent could not be verified; settings preserved".into()',
         b'format!("legacy diagnostic reply: {reply}")'),
        ("crates/hive-cli/src/update_bridge/windows.ps1",
         b"reason='parent-or-console-unavailable'", b"reason=$_.Exception.Message"),
    ]:
        path = ROOT / relative
        original_bytes = path.read_bytes()
        assert original_bytes.count(before) == 1
        diagnostic_sources.append((path, original_bytes, original_bytes.replace(before, after)))
    for path, _, patched_bytes in diagnostic_sources:
        path.write_bytes(patched_bytes)
    build_command = [shutil.which("cargo"), "build", "-p", "hive-cli", "--locked"]
    build = Run("격리 CI 전용 반환 오류 진단 빌드", build_command, paths=["target/debug"])
    build_code = 1
    try:
        build_code = build.execute(build_command, env=dict(os.environ, CARGO_INCREMENTAL="0"))
    finally:
        build.finish(build_code, details={"scope": "diagnostic incoming variant only; copied result message, unchanged authorization/return; not product acceptance"})
        if build_code:
            for path, original_bytes, patched_bytes in diagnostic_sources:
                assert path.read_bytes() == patched_bytes
                path.write_bytes(original_bytes)
    if build_code:
        raise SystemExit(build_code)
original = prefix / "bin/hive.exe"
original_hash = "e34e5b7486feb6adc52f5b0ca59c01da303a14af2ecd397b1c8e60c9ecab7824"
sha = lambda data: hashlib.sha256(data).hexdigest()
run = Run("격리 Windows의 공개0.13.1 직접 갱신·질문·완료",
          [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
          paths=[work.relative_to(ROOT).as_posix()], binary=candidate)
code = 1
try:
    with urllib.request.urlopen("https://registry.npmjs.org/@aigent-hive/win32-x64/-/win32-x64-0.13.1.tgz", timeout=30) as response:
        archive = response.read(128 * 1024 * 1024 + 1)
    assert len(archive) <= 128 * 1024 * 1024
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        data = package.extractfile("package/bin/hive.exe").read()
    assert sha(data) == original_hash
    original.write_bytes(data)
    receipt = prefix / "share/aigent-hive/install-receipt.json"
    receipt.write_text(json.dumps({"schema_version": 1, "owner": "direct", "product": "aigent-hive",
        "version": "0.13.1", "package_version": "0.13.1", "artifact_sha256": "sha256:" + original_hash}))
    shutil.copyfile(candidate, work / "incoming/hive.exe")
    (work / "fixture-only").write_text("0.14.0-direct-fixture")
    (user / "README.txt").write_bytes(b"Preserve this synthetic user note.\r\n")

    installer = r'''
using System; using System.IO; using System.Reflection; using System.Security.Cryptography; using System.Diagnostics;
public static class FixtureInstaller {
    static string Plain(string value) { return Path.GetFullPath(value.StartsWith(@"\\?\") ? value.Substring(4) : value); }
    public static int Main(string[] args) {
        string root=Directory.GetParent(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)).FullName;
        if(!Path.GetFileName(root).StartsWith("windows-legacy-ci-") || File.ReadAllText(Path.Combine(root,"fixture-only"))!="0.14.0-direct-fixture") return 20;
        int p=Array.IndexOf(args,"-Prefix"), v=Array.IndexOf(args,"-Version"), n=Array.IndexOf(args,"-PackageVersion");
        if(p<0 || v<0 || n<0 || p+1>=args.Length || v+1>=args.Length || n+1>=args.Length) return 21;
        string prefix=Path.Combine(root,"prefix");
        if(!String.Equals(Plain(args[p+1]),Plain(prefix),StringComparison.OrdinalIgnoreCase) || args[v+1]!="0.13.2" || args[n+1]!="0.13.2-test.2") return 22;
        string target=Path.Combine(prefix,"bin/hive.exe");
        File.Copy(target,Path.Combine(root,"original-preserved.exe"));
        File.Copy(Path.Combine(root,"incoming/hive.exe"),target,true);
        string hash; using(var file=File.OpenRead(target)) using(var digest=SHA256.Create()) hash=BitConverter.ToString(digest.ComputeHash(file)).Replace("-","").ToLowerInvariant();
        File.WriteAllText(Path.Combine(prefix,"share/aigent-hive/install-receipt.json"),"{\"schema_version\":1,\"owner\":\"direct\",\"product\":\"aigent-hive\",\"version\":\"0.13.2\",\"package_version\":\"0.13.2-test.2\",\"artifact_sha256\":\"sha256:"+hash+"\"}");
        File.WriteAllText(Path.Combine(root,"install-count"),"1"); return 0;
    }
}
'''
    (work / "installer.cs").write_text(installer)
    registry = json.dumps(json.loads((ROOT / "harness/setup/legacy-update-images.json").read_bytes()),
                          separators=(",", ":"), sort_keys=True)
    bridge = (ROOT / "crates/hive-cli/src/update_bridge/windows.ps1").read_text()
    bridge = bridge.replace("{{REGISTRY_SOURCE}}", registry.replace("'", "''"))
    bridge = bridge.replace("{{REGISTRY_DIGEST}}", "sha256:" + sha(registry.encode()))
    for placeholder, filename in [("{{PARENT_SOURCE}}", "windows-parent.cs"), ("{{CONSOLE_SOURCE}}", "windows-console.cs")]:
        bridge = bridge.replace(placeholder, base64.b64encode((ROOT / "crates/hive-cli/src/update_bridge" / filename).read_bytes()).decode())
    # Only this test copy returns a bounded exception message for the synthetic parent.
    bridge = bridge.replace("reason='parent-or-console-unavailable'", "reason=$_.Exception.Message")
    bridge = "$probeStarted=[DateTime]::UtcNow\n" + bridge.replace(
        "function Reply($value) {", "function Reply($value) { $jsonStarted=[DateTime]::UtcNow; [void]($value | ConvertTo-Json -Compress -Depth 12); $value.json_prepare_ms=([DateTime]::UtcNow-$jsonStarted).TotalMilliseconds; $value.elapsed_ms=([DateTime]::UtcNow-$probeStarted).TotalMilliseconds;")
    for statement, stage in [
        ("    $request = Request", "request-read"),
        ('    $self = Get-CimInstance Win32_Process -Filter "ProcessId = $PID"', "self-found"),
        ('    $parent = Get-CimInstance Win32_Process -Filter ("ProcessId = " + [int]$caller.ParentProcessId)', "parent-found"),
        ("    $image = [HiveParentImage]::new([int]$parent.ProcessId)", "image-opened"),
        ("    $owner = Invoke-CimMethod -InputObject $parent -MethodName GetOwnerSid", "owner-read"),
        ("    $verified = @()", "image-check-started"),
        ("    if ($verified.Count -ne 1)", "image-check-ended"),
    ]:
        trace = "Reply @{event='probe-stage'; stage='" + stage + "'}"
        if stage == "image-check-ended":
            bridge = bridge.replace(statement, "    " + trace + "\n" + statement, 1)
        else:
            bridge = bridge.replace(statement, statement + "; " + trace, 1)
    bridge = bridge.replace("try {\n    $request = Request", "try {\n    Reply @{event='probe-stage'; stage='script-started'}\n    $request = Request", 1)
    (work / "probe.ps1").write_text(bridge, encoding="utf-8")
    (work / "probe-request.json").write_text(json.dumps({"operation": "inspect", "caller_pid": "__CALLER__"}))
    installer = installer.replace('File.WriteAllText(Path.Combine(root,"install-count"),"1"); return 0;', r'''
        File.WriteAllText(Path.Combine(root,"install-count"),"1");
        var probe=new ProcessStartInfo(Path.Combine(Environment.GetEnvironmentVariable("SystemRoot"),"System32/WindowsPowerShell/v1.0/powershell.exe"),"-NoProfile -NonInteractive -WindowStyle Hidden -File \""+Path.Combine(root,"probe.ps1")+"\"");
        probe.UseShellExecute=false; probe.CreateNoWindow=true; probe.RedirectStandardInput=true; probe.RedirectStandardOutput=true; probe.RedirectStandardError=true;
        try { using(var child=Process.Start(probe)) {
            var output=child.StandardOutput.ReadToEndAsync(); var error=child.StandardError.ReadToEndAsync(); string inputError="";
            try {
                child.StandardInput.WriteLine(File.ReadAllText(Path.Combine(root,"probe-request.json")).Replace("\"__CALLER__\"",Process.GetCurrentProcess().Id.ToString()));
                child.StandardInput.WriteLine("{\"operation\":\"open\"}");
                child.StandardInput.WriteLine("{\"operation\":\"close\"}");
            } catch(Exception e) { inputError=e.GetType().Name+":"+e.Message; }
            try { child.StandardInput.Close(); } catch(IOException) { }
            if(!child.WaitForExit(25000)) { child.Kill(); child.WaitForExit(); inputError+="; probe timeout"; }
            File.WriteAllText(Path.Combine(root,"bridge-probe.txt"),output.Result+error.Result+inputError);
        } } catch(Exception e) { File.WriteAllText(Path.Combine(root,"bridge-probe.txt"),e.ToString()); }
        return 0;''')
    (work / "installer.cs").write_text(installer)
    tree = ast.parse((ROOT / "tests/conformance/contracts/test_setup_console_interrupt.py").read_text())
    console_source = next(ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign)
                          and any(isinstance(target, ast.Name) and target.id == "FIXTURE" for target in node.targets))
    (work / "console.cs").write_text(console_source)
    compiler = Path(os.environ["SystemRoot"]) / "Microsoft.NET/Framework64/v4.0.30319/csc.exe"
    for source, executable in [(work / "installer.cs", shim / "powershell.exe"), (work / "console.cs", work / "console.exe")]:
        subprocess.run([str(compiler), "/nologo", "/target:exe", "/out:" + str(executable), str(source)], check=True, capture_output=True)

    (work / "agy.py").write_text('''import json,os,shutil,sys
from pathlib import Path
user=Path(os.environ['HIVE_TEST_USER_ROOT']).resolve();stage=user/'.gemini/config/plugins/aigent-hive';args=sys.argv[1:]
assert stage.resolve().is_relative_to(user)
if args==['--version']:print('1.1.7');raise SystemExit(0)
if args[:2]==['plugin','install']:
 source=Path(args[2]).resolve();assert source.is_relative_to(user)
 if stage.exists():shutil.rmtree(stage)
 shutil.copytree(source,stage)
if args==['plugin','list']:
 print(json.dumps({'imports':[{'name':'aigent-hive','source':'antigravity','importedAt':'2026-10-28T00:00:00Z','components':['skills']}]} if stage.exists() else {'imports':[]}))
else:print('{}')
''')
    (shim / "agy.cmd").write_text('@echo off\r\n"' + sys.executable + '" "' + str(work / "agy.py") + '" %*\r\n')
    environment = {"PATH": str(shim), "HIVE_TEST_USER_ROOT": str(user), "PYTHONUTF8": "1",
                   "PSModuleAnalysisCachePath": str(work / "powershell-module-cache")}
    for key in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT"]:
        if key in os.environ:
            environment[key] = os.environ[key]
    for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "CLAUDE_CONFIG_DIR",
                "GEMINI_CLI_HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "TEMP", "TMP", "NPM_CONFIG_PREFIX", "NPM_CONFIG_CACHE"]:
        directory = work / key
        directory.mkdir()
        environment[key] = str(directory)
    result = work / "console-result.json"
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    process = subprocess.run([str(work / "console.exe"), str(original), str(user), str(result), "legacy-direct"],
        env=environment, capture_output=True, timeout=150, creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup)
    error = Path(str(result) + ".error.txt")
    assert process.returncode == 0, (process.returncode,
        result.read_text(encoding="utf-8", errors="replace") if result.exists() else process.stderr.decode(errors="replace"),
        error.read_text(encoding="utf-8", errors="replace") if error.exists() else "no fixture exception",
        (user / ".hive/runtime/update/bridge-diagnostic.txt").read_text(encoding="utf-8", errors="replace") if (user / ".hive/runtime/update/bridge-diagnostic.txt").exists() else "no incoming diagnostic",
        (work / "bridge-probe.txt").read_text(encoding="utf-8", errors="replace")[-8000:] if (work / "bridge-probe.txt").exists() else "no parent probe")
    report = json.loads(result.read_text())
    assert report["helper_exit_code"] == 0 and report["confirmed"] and report["questions"] >= 2
    shown = subprocess.run([str(original), "setup", "--scope", "user", "--user-root", str(user), "--show", "--output", "json"],
                           env=environment, capture_output=True, timeout=30)
    assert shown.returncode == 0, shown.stderr
    saved = json.loads(shown.stdout)["data"]["stored"]
    assert saved["schema_version"] == 2 and saved["interface_language"] == "ko"
    assert saved["selected_hosts"] == ["antigravity"] and not saved["usage_guard"]["enforcement_enabled"]
    assert sha(original.read_bytes()) == sha(candidate.read_bytes())
    assert sha((work / "original-preserved.exe").read_bytes()) == original_hash
    assert (work / "install-count").read_text() == "1"
    assert (user / "README.txt").read_bytes() == b"Preserve this synthetic user note.\r\n"
    code = 1 if diagnostic else 0
finally:
    try:
        run.finish(code, details={"original_sha256": original_hash, "candidate_sha256": sha(candidate.read_bytes()),
            "diagnostic_incoming_variant": diagnostic,
            "incoming_diagnostic": (user / ".hive/runtime/update/bridge-diagnostic.txt").read_text(encoding="utf-8", errors="replace") if (user / ".hive/runtime/update/bridge-diagnostic.txt").exists() else None,
            "untracked_paths": subprocess.run(["git", "ls-files", "--others", "--exclude-standard"],
                                              cwd=ROOT, capture_output=True, text=True, check=True).stdout.splitlines(),
            "scope": "isolated Windows VM; frozen public13.1 direct helper and private native console, synthetic installer/Antigravity; diagnostic variants never acceptance; not public14 installer, real app/monitor control or personal installation"})
    finally:
        if diagnostic:
            for path, original_bytes, patched_bytes in diagnostic_sources:
                assert path.read_bytes() == patched_bytes
                path.write_bytes(original_bytes)
raise SystemExit(code)
