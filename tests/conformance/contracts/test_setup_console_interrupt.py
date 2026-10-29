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
using System.Text;
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
    static extern uint GetConsoleProcessList([Out] uint[] processes, uint count);
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
    [StructLayout(LayoutKind.Sequential)] struct Point { public short X,Y; }
    [StructLayout(LayoutKind.Sequential)] struct Rectangle { public short Left,Top,Right,Bottom; }
    [StructLayout(LayoutKind.Sequential)] struct ScreenInfo { public Point Size,Cursor; public short Attributes; public Rectangle Window; public Point Maximum; }
    [DllImport("kernel32.dll")] static extern bool GetConsoleScreenBufferInfo(SafeFileHandle output,out ScreenInfo info);
    [DllImport("kernel32.dll",EntryPoint="ReadConsoleOutputCharacterW",CharSet=CharSet.Unicode)]
    static extern bool ReadConsoleOutputCharacter(SafeFileHandle output,StringBuilder text,uint length,Point start,out uint read);
    static string OwnConsoleTail() {
        using(var output=CreateFile("CONOUT$",0x80000000,3,IntPtr.Zero,3,0,IntPtr.Zero)) {
            ScreenInfo info; if(!GetConsoleScreenBufferInfo(output,out info)) return "console unavailable";
            int end=info.Cursor.Y*info.Size.X+info.Cursor.X, count=Math.Min(end,12000), offset=end-count;
            var start=new Point {X=(short)(offset%info.Size.X),Y=(short)(offset/info.Size.X)};
            var text=new StringBuilder(count); uint read;
            return ReadConsoleOutputCharacter(output,text,(uint)count,start,out read) ? text.ToString() : "console unreadable";
        }
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
    static int ReviewAll(Process child, SafeFileHandle input, string[] args, bool resumeUpdate=false, string quickLanguage=null, bool publicUpdate=false) {
        string path=Path.Combine(args[1],".hive/config/setup-progress/user.yml");
        string last=""; bool confirmed=false; int answered=0;
        var deadline=Stopwatch.StartNew();
        if(publicUpdate) Send(input,"yes\r");
        while(!child.HasExited && deadline.ElapsedMilliseconds<(publicUpdate ? 600000 : 30000)) {
            try {
                string state=File.ReadAllText(path);
                if(state.Contains("phase: previewed") && !confirmed) { Send(input,"yes\r"); confirmed=true; }
                else if(state.Contains("phase: awaiting-answers")) {
                    var step=Regex.Match(state,@"(?m)^step: ([a-z0-9-]+)");
                    if(step.Success && step.Groups[1].Value!=last) {
                        last=step.Groups[1].Value;
                        if(resumeUpdate && last=="user-language") throw new InvalidOperationException("saved language asked twice");
                        Send(input,publicUpdate && last=="usage-enforcement" ? "yes\r" : publicUpdate && last=="usage-warnings" ? "no\r" : quickLanguage!=null ? (last=="user-language" ? (quickLanguage=="ko" ? "1\r" : "2\r") : "\r") : resumeUpdate ? "\r" : last=="user-section" ? "1\r" : last=="user-mode" ? "2\r" : "\r");
                        answered++;
                    }
                }
            } catch(IOException) { }
            Thread.Sleep(25);
        }
        if(!child.HasExited) throw new InvalidOperationException("review timeout at "+last);
        File.WriteAllText(args[2],"{\"exit_code\":"+child.ExitCode+",\"questions\":"+answered+",\"confirmed\":"+(confirmed?"true":"false")+"}");
        if(publicUpdate || child.ExitCode!=0) File.WriteAllText(args[2]+".error.txt",OwnConsoleTail(),new UTF8Encoding(false,false));
        return child.ExitCode;
    }
    static int LegacyDirect(Process child, SafeFileHandle input, string[] args, bool publicUpdate=false) {
        string progress=Path.Combine(args[1],".hive/config/setup-progress/user.yml");
        string expected=Path.GetFullPath(Path.Combine(args[1],".hive/runtime/update/hive-direct-update-helper.exe"));
        Process helper=null; string last=""; bool confirmed=false; int answered=0;
        bool stop=!publicUpdate && args[3]!="legacy-direct", hold=args[3]=="legacy-deadline", stopped=false;
        bool parentExit=args[3]=="legacy-parent-exit", parentApply=args[3]=="legacy-parent-apply";
        bool late=args[3]=="legacy-late-slow", slow=args[3]=="legacy-slow" || late || parentApply;
        Process host=null; long currentStart=-1;
        long firstQuestion=-1;
        var deadline=Stopwatch.StartNew();
        // The two frozen 0.13.1 bootstrap questions precede the new question engine.
        Send(input,publicUpdate ? "yes\r" : "yes\rantigravity\r");
        try {
            while(deadline.ElapsedMilliseconds<(publicUpdate?600000:late?580000:hold?470000:150000)) {
                if(helper==null) foreach(var candidate in Process.GetProcessesByName("hive-direct-update-helper")) {
                    try {
                        if(candidate.Handle==IntPtr.Zero) { candidate.Dispose(); continue; }
                        if(String.Equals(Path.GetFullPath(candidate.MainModule.FileName),expected,StringComparison.OrdinalIgnoreCase)) {
                            if(helper!=null) throw new InvalidOperationException("multiple owned helpers");
                            helper=candidate;
                        } else candidate.Dispose();
                    } catch(System.ComponentModel.Win32Exception) { candidate.Dispose(); }
                }
                try {
                    string state=File.ReadAllText(progress);
                    if(currentStart<0 && late && state.Contains("phase: awaiting-answers")) {
                        foreach(var candidate in Process.GetProcessesByName("hive")) {
                            try {
                                if(!candidate.HasExited && String.Equals(PlainPath(candidate.MainModule.FileName),PlainPath(args[0]),StringComparison.OrdinalIgnoreCase))
                                    currentStart=deadline.ElapsedMilliseconds-(long)(DateTime.UtcNow-candidate.StartTime.ToUniversalTime()).TotalMilliseconds;
                            } finally { candidate.Dispose(); }
                        }
                        if(currentStart<0) throw new InvalidOperationException("owned new updater start unavailable");
                    }
                    if(slow && host==null && File.Exists(Path.Combine(args[1],"slow-pid"))) {
                        int hostId;
                        if(Int32.TryParse(File.ReadAllText(Path.Combine(args[1],"slow-pid")),out hostId)) {
                            host=Process.GetProcessById(hostId);
                            if(host.Handle==IntPtr.Zero || !String.Equals(PlainPath(host.MainModule.FileName),PlainPath(Environment.GetEnvironmentVariable("HIVE_TEST_PYTHON")),StringComparison.OrdinalIgnoreCase))
                                throw new InvalidOperationException("test host identity differs");
                        }
                    }
                    if(parentApply && host!=null && !stopped) { helper.Kill(); stopped=true; }
                    if(state.Contains("phase: previewed") && !confirmed && (!stop || slow)
                        && (!late || deadline.ElapsedMilliseconds-currentStart>=417000)) { Send(input,"yes\r"); confirmed=true; }
                    else if(state.Contains("phase: awaiting-answers")) {
                        var step=Regex.Match(state,@"(?m)^step: ([a-z0-9-]+)");
                        if(step.Success && step.Groups[1].Value!=last && !stopped) {
                            last=step.Groups[1].Value;
                            if(publicUpdate && last=="user-language") throw new InvalidOperationException("saved language asked twice");
                            if(firstQuestion<0) firstQuestion=deadline.ElapsedMilliseconds;
                            if(stop && !slow && last!="user-language") {
                                if(parentExit) { helper.Kill(); }
                                else if(!hold) Send(input,new string((char)26,1)+"\r");
                                stopped=true;
                            } else { Send(input,publicUpdate && last=="usage-enforcement" ? "yes\r" : publicUpdate && last=="usage-warnings" ? "no\r" : last=="user-language" ? "1\r" : "\r"); answered++; }
                        }
                    }
                    if(helper!=null && helper.HasExited) {
                        if(stop) {
                            if((!stopped && !slow) || helper.ExitCode==0 || state.Contains("phase: complete") || confirmed!=slow) return 32;
                            if(!parentExit && !parentApply && helper.ExitCode!=2) return 32;
                            WaitPrivateExit(args[0],host);
                            File.WriteAllText(args[2],"{\"helper_exit_code\":"+helper.ExitCode+",\"questions\":"+answered+",\"confirmed\":"+(confirmed?"true":"false")+",\"question_wait_ms\":"+(deadline.ElapsedMilliseconds-firstQuestion)+",\"updater_elapsed_ms\":"+(currentStart<0?-1:deadline.ElapsedMilliseconds-currentStart)+",\"host_observed\":"+(host!=null?"true":"false")+",\"owned_processes_ended\":true}");
                            return 0;
                        }
                        if(helper.ExitCode!=0 || !state.Contains("phase: complete") || !confirmed) return 31;
                        File.WriteAllText(args[2],"{"+(publicUpdate?"\"exit_code\":0,":"")+"\"helper_exit_code\":0,\"questions\":"+answered+",\"confirmed\":true}");
                        return 0;
                    }
                } catch(IOException) { }
                if(helper!=null && helper.HasExited) throw new InvalidOperationException("legacy helper ended before setup completion: "+helper.ExitCode);
                Thread.Sleep(50);
            }
            throw new InvalidOperationException("legacy direct flow timeout at "+last);
        } finally {
            if(publicUpdate) File.WriteAllText(args[2]+".error.txt",OwnConsoleTail(),new UTF8Encoding(false,false));
            if(!File.Exists(args[2])) File.WriteAllText(args[2],"step="+last+"; confirmed="+confirmed+"; helper="+(helper==null?"absent":helper.HasExited?helper.ExitCode.ToString():"running")+"\n"+OwnConsoleTail(),new UTF8Encoding(false,false));
            if(helper!=null) { if(!helper.HasExited) helper.Kill(); helper.Dispose(); }
            if(host!=null) host.Dispose();
        }
    }
    static string PlainPath(string value) { return Path.GetFullPath(value.StartsWith(@"\\?\")?value.Substring(4):value); }
    static void WaitPrivateExit(string executable, Process host) {
        var waiting=Stopwatch.StartNew(); var pids=new uint[128];
        while(waiting.ElapsedMilliseconds<10000) {
            bool unlocked=false;
            try { using(File.Open(executable,FileMode.Open,FileAccess.Write,FileShare.ReadWrite|FileShare.Delete)) { unlocked=true; } } catch(IOException) { } catch(UnauthorizedAccessException) { }
            uint count=GetConsoleProcessList(pids,(uint)pids.Length);
            if(unlocked && count==1 && pids[0]==Process.GetCurrentProcess().Id && (host==null || host.HasExited)) return;
            Thread.Sleep(50);
        }
        throw new InvalidOperationException("owned updater or host remained after interruption");
    }
    public static int Main(string[] args) {
        Process child=null;
        try {
            bool project=args.Length==7 && (args[3]=="connection-loss" || args[3]=="project-quick");
            bool review=args.Length==4 && args[3]=="review-all";
            bool quick=args.Length==4 && (args[3]=="user-quick-ko" || args[3]=="user-quick-en");
            bool resumeUser=args.Length==4 && args[3]=="user-resume";
            bool publicUpdate=(args.Length==4 && args[3]=="public-update") || (args.Length==5 && args[3]=="public-update-npm");
            bool legacy=args.Length==4 && (args[3]=="legacy-direct" || args[3]=="legacy-cancel" || args[3]=="legacy-deadline"
                || args[3]=="legacy-parent-exit" || args[3]=="legacy-parent-apply" || args[3]=="legacy-slow" || args[3]=="legacy-late-slow");
            bool resumeUpdate=args.Length==4 && args[3]=="update-resume";
            if(args.Length!=4 && !project && !publicUpdate) return 20;
            Console.CancelKeyPress += delegate(object sender, ConsoleCancelEventArgs e) { e.Cancel=true; };
            using(var input=CreateFile("CONIN$",0xC0000000,3,IntPtr.Zero,3,0,IntPtr.Zero))
            using(var output=CreateFile("CONOUT$",0xC0000000,3,IntPtr.Zero,3,0,IntPtr.Zero)) {
                if(input.IsInvalid || output.IsInvalid || !SetHandleInformation(input,1,1) || !SetHandleInformation(output,1,1)
                    || !SetStdHandle(-10,input) || !SetStdHandle(-12,output)) return 21;
                string command=publicUpdate ? (args.Length==5 ? "\""+args[4]+"\" " : "")+"update --channel test --user-root \""+args[1]+"\"" : legacy ? "update --channel test --user-root \""+args[1]+"\"" : resumeUpdate
                    ? "update --interactive --user-root \""+args[1]+"\"" : project
                    ? "setup --target \""+args[4]+"\""+(args[5]=="-" ? "" : " --capabilities \""+args[5]+"\"")+" --interactive --user-root \""+args[1]+"\""
                    : "setup --scope user "+(resumeUser ? "--resume " : review || quick ? "" : "--section usage ")+"--interactive --user-root \""+args[1]+"\"";
                var start=new ProcessStartInfo(args[0],command);
                start.UseShellExecute=false;
                if(!review && !legacy && !resumeUpdate && !quick && !resumeUser && !publicUpdate) start.EnvironmentVariables["PATH"]=Path.Combine(args[1],"no-external-tools");
                start.EnvironmentVariables["CODEX_HOME"]=Path.Combine(args[1],"codex-home");
                child=Process.Start(start);
                if(legacy) return LegacyDirect(child,input,args);
                if(publicUpdate) return args.Length==4 ? LegacyDirect(child,input,args,true) : ReviewAll(child,input,args,true,null,true);
                if(resumeUpdate) return ReviewAll(child,input,args,true);
                if(project) return ProjectFlow(child,input,args);
                if(quick) return ReviewAll(child,input,args,false,args[3].EndsWith("-ko") ? "ko" : "en");
                if(resumeUser) return ReviewAll(child,input,args,true);
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
        } catch(Exception e) {
            if(args.Length>=3) File.AppendAllText(args[2]+".error.txt","\n"+e.ToString(),new UTF8Encoding(false,false));
            Console.Error.WriteLine(e.Message); return 25;
        }
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
