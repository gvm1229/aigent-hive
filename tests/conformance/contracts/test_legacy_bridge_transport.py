"""Windows legacy helper boundaries; no installed settings or model process is touched."""
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "crates/hive-cli/src/update_bridge"


def run_fixed_script(script, packet=None):
    encoded = base64.b64encode(script.encode()).decode()
    chunks = [encoded[offset:offset + 16000] for offset in range(0, len(encoded), 16000)]
    env = os.environ.copy()
    for index, chunk in enumerate(chunks):
        env[f"HIVE_BRIDGE_TEST_SOURCE_{index}"] = chunk
    bootstrap = "$s=[Text.StringBuilder]::new();for($i=0;$i -lt " + str(len(chunks)) + ";$i++){[void]$s.Append([Environment]::GetEnvironmentVariable('HIVE_BRIDGE_TEST_SOURCE_'+$i));[Environment]::SetEnvironmentVariable('HIVE_BRIDGE_TEST_SOURCE_'+$i,$null)};& ([scriptblock]::Create([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($s.ToString()))))"
    executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
    return subprocess.run([str(executable), "-NoLogo", "-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", bootstrap],
                          env=env, input=None if packet is None else json.dumps(packet) + "\n",
                          capture_output=True, text=True, encoding="utf-8", timeout=20, creationflags=0x08000000)


@unittest.skipUnless(os.name == "nt", "requires Windows PowerShell and native console APIs")
class LegacyBridgeTransportTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        raw = (ROOT / "harness/setup/legacy-update-images.json").read_bytes().decode()
        cls.registry = json.dumps(json.loads(raw), separators=(",", ":"), sort_keys=True)
        cls.script = (SOURCE / "windows.ps1").read_text(encoding="utf-8")
        cls.script = cls.script.replace("{{REGISTRY_SOURCE}}", cls.registry.replace("'", "''"))
        cls.script = cls.script.replace("{{REGISTRY_DIGEST}}", "sha256:" + hashlib.sha256(cls.registry.encode()).hexdigest())
        for placeholder, filename in [("{{PARENT_SOURCE}}", "windows-parent.cs"), ("{{CONSOLE_SOURCE}}", "windows-console.cs")]:
            cls.script = cls.script.replace(placeholder, base64.b64encode((SOURCE / filename).read_bytes()).decode())

    def test_non_hive_parent_does_not_open_a_console_or_wait_for_input(self):
        result = run_fixed_script(self.script, {"operation":"inspect", "caller_pid":os.getpid()})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"event":"not-legacy-parent"})
        self.assertEqual(result.stderr, "")

    def test_a_different_caller_cannot_select_an_arbitrary_parent(self):
        result = run_fixed_script(self.script, {"operation":"inspect", "caller_pid":1})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"event":"closed", "reason":"parent-or-console-unavailable"})

    def test_embedded_registry_keeps_its_exact_digest_and_rejects_changed_bytes(self):
        prefix = self.script.split("\ntry {\n", 1)[0]
        binding = "    $registrySource = " + self.script.split("    $registrySource = ", 1)[1].split("    Add-Type", 1)[0]
        script = prefix + "\n" + binding + "\nReply @{versions=@($registry.images.version)}\n"
        result = run_fixed_script(script, {})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"versions":["0.13.1","0.13.2-test.1","0.13.2-test.2"]})
        self.assertIn('"schema_version":1', script)
        changed = run_fixed_script(script.replace('"schema_version":1', '"schema_version":9', 1), {})
        self.assertNotEqual(changed.returncode, 0)
        self.assertIn("public registry differs", changed.stderr)

    def test_native_command_line_parser_preserves_quoted_paths_and_rejects_invalid_counts(self):
        source = base64.b64encode((SOURCE / "windows-parent.cs").read_bytes()).decode()
        script = "$ErrorActionPreference='Stop';Add-Type -TypeDefinition ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('" + source + "'))) -Language CSharp\n" + r'''
$parsed=[HiveParentImage]::Arguments('"C:\A B\hive.exe" update --user-root "C:\User A" --confirm')
if($parsed.Count -ne 5 -or $parsed[3] -ne 'C:\User A' -or $parsed[4] -ne '--confirm'){throw 'argument mismatch'}
$negative=0
foreach($line in @('','hive.exe',('hive.exe update '+('x '*40)))){try{[void][HiveParentImage]::Arguments($line)}catch{$negative++}}
if($negative -ne 3){throw 'argument bounds accepted'}
@{arguments=$parsed.Count;invalid_rejected=$negative}|ConvertTo-Json -Compress
'''
        script += "\n#" + self.registry
        result = run_fixed_script(script)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"arguments":5,"invalid_rejected":3})


if __name__ == "__main__":
    unittest.main()
