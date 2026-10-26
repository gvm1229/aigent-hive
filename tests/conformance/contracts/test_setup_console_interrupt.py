"""Real console cancellation uses a hidden, private console and a disposable user root."""
import json
import os
from pathlib import Path
import subprocess
import unittest

from tests.conformance.support.harness import Phase1CliTestCase, read_yaml


FIXTURE = r'''
using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Text.RegularExpressions;
using Microsoft.Win32.SafeHandles;
public static class SetupConsoleFixture {
    [StructLayout(LayoutKind.Explicit, Size=20)]
    public struct Record {
        [FieldOffset(0)] public ushort Type;
        [FieldOffset(4)] public int Down;
        [FieldOffset(8)] public ushort Repeat;
        [FieldOffset(10)] public ushort Key;
        [FieldOffset(14)] public ushort Character;
    }
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern SafeFileHandle CreateFile(string name, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool SetHandleInformation(SafeFileHandle handle, uint mask, uint flags);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool SetStdHandle(int kind, SafeFileHandle handle);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GenerateConsoleCtrlEvent(uint kind, uint group);
    [DllImport("kernel32.dll", EntryPoint="WriteConsoleInputW", SetLastError=true)]
    static extern bool WriteConsoleInput(SafeFileHandle handle, [In] Record[] records, uint count, out uint written);
    static void Send(SafeFileHandle input, string text) {
        var records = new Record[text.Length];
        for(int i=0;i<text.Length;i++) records[i]=new Record { Type=1, Down=1, Repeat=1,
            Key=(ushort)(text[i]=='\r'?13:0), Character=text[i] };
        uint written;
        if(!WriteConsoleInput(input,records,(uint)records.Length,out written) || written!=records.Length)
            throw new InvalidOperationException("private input failed");
    }
    static void WaitStep(Process child, string path, string step) {
        var deadline=Stopwatch.StartNew();
        while(deadline.ElapsedMilliseconds<15000 && !child.HasExited) {
            try { if(File.Exists(path) && File.ReadAllText(path).Contains("step: "+step)) return; }
            catch(IOException) { }
            Thread.Sleep(25);
        }
        throw new InvalidOperationException("CLI did not reach the expected private question");
    }
    static string WaitProjectStep(Process child, string root, string step) {
        string directory=Path.Combine(root,".hive/config/setup-progress/projects");
        var deadline=Stopwatch.StartNew();
        while(deadline.ElapsedMilliseconds<15000 && !child.HasExited) {
            if(Directory.Exists(directory)) foreach(string path in Directory.GetFiles(directory,"*.yml")) {
                try { if(File.ReadAllText(path).Contains("step: "+step)) return path; }
                catch(IOException) { }
            }
            Thread.Sleep(25);
        }
        throw new InvalidOperationException("project CLI did not reach the expected private question");
    }
    static int ProjectFlow(Process child, SafeFileHandle input, string[] args) {
        WaitProjectStep(child,args[1],"project-mode"); Send(input,"\r");
        string progress=WaitProjectStep(child,args[1],"project-name");
        if(args[3]=="project-quick") {
            Send(input,args[6]+"\r\r\ryes\r");
            if(!child.WaitForExit(30000)) return 28;
            File.WriteAllText(args[2],"{\"exit_code\":"+child.ExitCode+"}");
            return child.ExitCode;
        }
        File.WriteAllText(Path.Combine(args[6],"question.ready"),progress);
        var deadline=Stopwatch.StartNew();
        while(!File.Exists(Path.Combine(args[6],"continue-user"))) {
            if(child.HasExited || deadline.ElapsedMilliseconds>20000) return 27;
            Thread.Sleep(25);
        }
        Send(input,"Connection loss fixture\r\r\ryes\r");
        if(!child.WaitForExit(30000)) return 28;
        File.WriteAllText(args[2],"{\"exit_code\":"+child.ExitCode+"}");
        return child.ExitCode;
    }
    static int ReviewAll(Process child, SafeFileHandle input, string[] args) {
        string path=Path.Combine(args[1],".hive/config/setup-progress/user.yml");
        string last=""; bool confirmed=false; int answered=0;
        var deadline=Stopwatch.StartNew();
        while(!child.HasExited && deadline.ElapsedMilliseconds<30000) {
            try {
                string state=File.ReadAllText(path);
                if(state.Contains("phase: previewed") && !confirmed) { Send(input,"yes\r"); confirmed=true; }
                else if(state.Contains("phase: awaiting-answers")) {
                    var step=Regex.Match(state,@"(?m)^step: ([a-z0-9-]+)");
                    if(step.Success && step.Groups[1].Value!=last) {
                        last=step.Groups[1].Value;
                        Send(input,last=="user-section" ? "1\r" : last=="user-mode" ? "2\r" : "\r");
                        answered++;
                    }
                }
            } catch(IOException) { }
            Thread.Sleep(25);
        }
        if(!child.HasExited) throw new InvalidOperationException("review timeout at "+last);
        File.WriteAllText(args[2],"{\"exit_code\":"+child.ExitCode+",\"questions\":"+answered+",\"confirmed\":"+(confirmed?"true":"false")+"}");
        return child.ExitCode;
    }
    public static int Main(string[] args) {
        Process child=null;
        try {
            bool project=args.Length==7 && (args[3]=="connection-loss" || args[3]=="project-quick");
            bool review=args.Length==4 && args[3]=="review-all";
            if(args.Length!=4 && !project) return 20;
            Console.CancelKeyPress += delegate(object sender, ConsoleCancelEventArgs e) { e.Cancel=true; };
            using(var input=CreateFile("CONIN$",0xC0000000,3,IntPtr.Zero,3,0,IntPtr.Zero))
            using(var output=CreateFile("CONOUT$",0xC0000000,3,IntPtr.Zero,3,0,IntPtr.Zero)) {
                if(input.IsInvalid || output.IsInvalid || !SetHandleInformation(input,1,1) || !SetHandleInformation(output,1,1)
                    || !SetStdHandle(-10,input) || !SetStdHandle(-12,output)) return 21;
                string command=project
                    ? "setup --target \""+args[4]+"\""+(args[5]=="-" ? "" : " --capabilities \""+args[5]+"\"")+" --interactive --user-root \""+args[1]+"\""
                    : "setup --scope user "+(review ? "" : "--section usage ")+"--interactive --user-root \""+args[1]+"\"";
                var start=new ProcessStartInfo(args[0],command);
                start.UseShellExecute=false;
                if(!review) start.EnvironmentVariables["PATH"]=Path.Combine(args[1],"no-external-tools");
                start.EnvironmentVariables["CODEX_HOME"]=Path.Combine(args[1],"codex-home");
                child=Process.Start(start);
                if(project) return ProjectFlow(child,input,args);
                if(review) return ReviewAll(child,input,args);
                string progress=Path.Combine(args[1],".hive/config/setup-progress/user.yml");
                WaitStep(child,progress,"user-language"); Send(input,"1\r");
                WaitStep(child,progress,"usage-enforcement"); Thread.Sleep(100);
                if(args[3]=="ctrl-c") { if(!GenerateConsoleCtrlEvent(0,0)) return 22; }
                else if(args[3]=="eof") { Send(input,new string((char)26,1)+"\r"); }
                else return 26;
                if(!child.WaitForExit(10000)) return 23;
                File.WriteAllText(args[2],"{\"exit_code\":"+child.ExitCode+"}");
                return child.ExitCode==0 ? 24 : 0;
            }
        } catch(Exception e) { Console.Error.WriteLine(e.Message); return 25; }
        finally { if(child!=null && !child.HasExited) { child.Kill(); child.WaitForExit(); } }
    }
}
'''


def compile_console_fixture(work_root):
    """Only the fixed compiler process sees its real runtime directory, never Hive."""
    source = work_root / "console.cs"
    helper = work_root / "console.exe"
    source.write_text(FIXTURE, encoding="utf-8")
    system = Path(os.environ.get("HIVE_TEST_CONSOLE_SYSTEM_ROOT", os.environ["SystemRoot"]))
    framework = system / "Microsoft.NET/Framework64/v4.0.30319"
    compiler = framework / "csc.exe"
    result = subprocess.run(
        [str(compiler), "/nologo", "/noconfig", "/target:exe", f"/out:{helper}",
         f"/reference:{framework / 'System.dll'}", str(source)],
        env={**os.environ, "SystemRoot":str(system), "WINDIR":str(system)},
        capture_output=True, text=True, timeout=30, creationflags=subprocess.CREATE_NO_WINDOW,
    )
    if result.returncode:
        raise AssertionError((result.stdout, result.stderr))
    return helper


@unittest.skipUnless(os.name == "nt", "native Ctrl-C regression requires a private Windows console")
class SetupConsoleInterruptConformance(Phase1CliTestCase):
    def test_multiple_manifests_and_absent_host_complete_without_claiming_runtime_support(self):
        helper = compile_console_fixture(self.work_root)
        target = self.work_root / "mixed-project"
        target.mkdir()
        originals = {"package.json": b'{"name":"synthetic-js"}\n',
            "Cargo.toml": b'[package]\nname="synthetic-rust"\nversion="1.0.0"\n',
            "README.txt": b'Preserve the original note.\r\n'}
        for name, content in originals.items():
            (target / name).write_bytes(content)
        result_path = self.work_root / "project-console.json"
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        result = subprocess.run([str(helper), str(self.hive_binary), str(self.setup_user_root), str(result_path),
            "project-quick", str(target), "-", "multi-manifest"], capture_output=True, text=True, timeout=45,
            creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result_path.read_text())["exit_code"], 0)
        answers = read_yaml(target / ".hive/setup-answers.yml")
        self.assertEqual(answers["project_name"], "multi-manifest")
        self.assertEqual(answers["project_kind"], "general")
        self.assertEqual(answers["persistent_roles"], [])
        self.assertEqual(answers["approved_fallback_hooks"], [])
        capabilities = read_yaml(target / ".hive/config/capability-resolution.yml")
        self.assertEqual(capabilities["host_version"], "unverified")
        self.assertTrue(all(value=="unverified" for value in capabilities["capabilities"].values()))
        for name, content in originals.items():
            self.assertEqual((target / name).read_bytes(), content)
        capability_input = self.work_root / "stored-capabilities.json"
        capability_input.write_text(json.dumps(capabilities), encoding="utf-8")
        checked, outcome = self.invoke_setup(target, answers=target / ".hive/setup-answers.yml",
            capabilities=capability_input, mode="--validate")
        self.assertEqual(checked.returncode, 0, outcome)

    def test_full_review_in_both_languages_keeps_choices_and_validates_file_results(self):
        from tests.conformance.integration.test_project_lifecycle import ProjectLifecycleConformance
        helper = compile_console_fixture(self.work_root)
        for language in ("ko", "en"):
            with self.subTest(language=language):
                area = self.work_root / language
                area.mkdir()
                root = area / "user"
                root.mkdir()
                (root / "README.txt").write_bytes(b"Keep the user note.\r\n")
                fake = ProjectLifecycleConformance()
                fake.work_root = area
                fake.fake_bin = area / "fake-bin"
                fake.fake_bin.mkdir()
                fake.host_log = area / "host-calls.jsonl"
                fake._write_fake_host("codex")
                environment = {**os.environ, "PATH":str(fake.fake_bin), "HIVE_TEST_HOST_LOG":str(fake.host_log), "HIVE_TEST_USER_ROOT":str(root)}
                answers = area / "answers.json"
                answers.write_text(json.dumps({"schema_version":1,"kind":"hive.setup-answer-patch",
                    "answers":{"interface_language":language,"selected_hosts":["codex"]}}), encoding="utf-8")
                command = [str(self.hive_binary), "setup", "--scope", "user", "--user-root", str(root),
                    "--answers", str(answers), "--input-channel", "agent", "--apply", "--output", "json"]
                seeded = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=40)
                self.assertEqual(seeded.returncode, 0, seeded.stdout)
                config = root / ".hive/config/user-setup.yml"
                before = read_yaml(config)
                result_path = area / "review.json"
                startup = subprocess.STARTUPINFO()
                startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                startup.wShowWindow = subprocess.SW_HIDE
                reviewed = subprocess.run([str(helper),str(self.hive_binary),str(root),str(result_path),"review-all"],
                    env=environment, capture_output=True, text=True, timeout=45,
                    creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup)
                self.assertEqual(reviewed.returncode, 0, reviewed.stderr)
                receipt = json.loads(result_path.read_text())
                self.assertTrue(receipt["confirmed"])
                self.assertGreater(receipt["questions"], 6)
                after = read_yaml(config)
                self.assertEqual({k:v for k,v in after.items() if k!="preference_decisions"},
                    {k:v for k,v in before.items() if k!="preference_decisions"})
                self.assertEqual((root / "README.txt").read_bytes(), b"Keep the user note.\r\n")
                saved = config.read_bytes()
                command[command.index("--apply")] = "--validate"
                validated = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=40)
                self.assertEqual(validated.returncode, 0, validated.stdout)
                self.assertEqual(config.read_bytes(), saved)

    def test_ctrl_c_is_not_a_negative_answer_and_saved_choices_resume(self):
        self.check_interruption("ctrl-c")

    def test_eof_is_not_a_negative_answer_and_saved_choices_resume(self):
        self.check_interruption("eof")

    def test_same_file_after_ctrl_c_keeps_the_recorded_terminal_language(self):
        from tests.conformance.integration.test_project_lifecycle import ProjectLifecycleConformance
        root, progress_path = self.check_interruption("ctrl-c")
        progress = read_yaml(progress_path)
        original = progress["answers"]["preference_decisions"]["/interface_language"]
        self.assertEqual(original["input_channel"], "terminal")
        answers = self.work_root / "same-draft.json"
        answers.write_text(json.dumps(progress["answers"]), encoding="utf-8")
        fake = ProjectLifecycleConformance()
        fake.work_root = self.work_root
        fake.fake_bin = self.work_root / "fake-codex"
        fake.fake_bin.mkdir()
        fake.host_log = self.work_root / "fake-codex-calls.jsonl"
        fake._write_fake_host("codex")
        result = subprocess.run([str(self.hive_binary), "setup", "--scope", "user", "--user-root", str(root),
            "--answers", str(answers), "--input-channel", "agent", "--apply", "--summary", "--output", "json"],
            input="", capture_output=True, text=True, encoding="utf-8", timeout=40,
            env={**os.environ, "PATH": str(fake.fake_bin), "HIVE_TEST_HOST_LOG": str(fake.host_log), "HIVE_TEST_USER_ROOT": str(root)})
        self.assertEqual(result.returncode, 0, (result.stdout, result.stderr))
        self.assertTrue(json.loads(result.stdout)["data"]["completion_verified"])
        saved = read_yaml(root / ".hive/config/user-setup.yml")
        self.assertEqual(saved["preference_decisions"]["/interface_language"], original)
        self.assertEqual(read_yaml(progress_path)["operation_id"], progress["operation_id"])

    def check_interruption(self, kind):
        helper = compile_console_fixture(self.work_root)
        root = self.work_root / "signal-user"
        root.mkdir()
        result_path = self.work_root / "result.json"
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        result = subprocess.run(
            [str(helper), str(self.hive_binary), str(root), str(result_path), kind],
            capture_output=True, text=True, timeout=45,
            creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotEqual(json.loads(result_path.read_text())["exit_code"], 0)
        if kind == "eof":
            self.assertEqual(json.loads(result_path.read_text())["exit_code"], 2)
        progress_path = root / ".hive/config/setup-progress/user.yml"
        progress = read_yaml(progress_path)
        self.assertEqual(progress["answers"]["interface_language"], "ko")
        self.assertNotIn("usage-enforcement", progress["answered_questions"])
        self.assertFalse((root / ".hive/config/user-setup.yml").exists())
        resumed = subprocess.run(
            [str(self.hive_binary), "setup", "--scope", "user", "--resume", "--non-interactive",
             "--user-root", str(root), "--output", "json"],
            stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8", timeout=15,
            env={**os.environ, "PATH":str(root / "no-external-tools")},
        )
        self.assertEqual(resumed.returncode, 2, resumed.stderr)
        self.assertEqual(json.loads(resumed.stdout)["status"], "needs-input")
        self.assertEqual(read_yaml(progress_path)["answers"]["interface_language"], "ko")
        self.assertNotIn("usage-enforcement", read_yaml(progress_path)["answered_questions"])
        return root, progress_path


if __name__ == "__main__":
    unittest.main()
