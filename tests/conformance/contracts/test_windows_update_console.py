"""ReadConsole must not include unwritten buffer contents in setup answers."""
import base64
import json
import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[3]


@unittest.skipUnless(os.name == "nt", "Windows console API regression requires Windows")
class WindowsUpdateConsoleTests(unittest.TestCase):
    def test_short_empty_and_korean_answers_use_only_the_returned_character_count(self):
        script = r'''
$ErrorActionPreference = 'Stop'
Add-Type -Path $env:HIVE_CONSOLE_TEST_SOURCE
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class FixtureConsoleKeys {
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
    [DllImport("kernel32.dll", EntryPoint="WriteConsoleInputW", SetLastError=true)]
    static extern bool WriteConsoleInput(SafeFileHandle handle, [In] Record[] records, uint count, out uint written);
    public static void Send(string text) {
        using (var input = CreateFile("CONIN$", 0xC0000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero)) {
            var records = new Record[text.Length];
            for(int i=0; i<text.Length; i++) records[i] = new Record {
                Type=1, Down=1, Repeat=1, Key=(ushort)(text[i]=='\r' ? 13 : 0), Character=text[i]
            };
            uint written;
            if (!WriteConsoleInput(input, records, (uint)records.Length, out written) || written!=records.Length)
                throw new InvalidOperationException("fixture input injection failed");
        }
    }
}
'@
$self = [Diagnostics.Process]::GetCurrentProcess()
$created = $self.StartTime.ToUniversalTime().ToFileTimeUtc()
$console = [HiveUpdateConsole]::new($self.Id, $self.Id, $created, $created)
try {
    $expected = @("yes`r`n", "`r`n", ([string][char]0xC608 + "`r`n"), "no`r`n")
    $lengths = @()
    foreach($answer in $expected) {
        [FixtureConsoleKeys]::Send($answer.Replace("`n", ""))
        $actual = $console.ReadLine(5000)
        if ($actual -cne $answer) { throw 'console returned bytes outside the typed answer' }
        $lengths += $actual.Length
    }
    [Console]::WriteLine(($lengths | ConvertTo-Json -Compress))
} finally { $console.Dispose() }
'''
        # A hidden, separate console owns these synthetic keys. Never write into the user's console.
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        shell = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        result = subprocess.run(
            [str(shell), "-NoProfile", "-NonInteractive", "-EncodedCommand",
             base64.b64encode(script.encode("utf-16le")).decode("ascii")],
            env={**os.environ, "HIVE_CONSOLE_TEST_SOURCE": str(ROOT / "crates/hive-cli/src/update_bridge/windows-console.cs")},
            stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=45,
            creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), [5, 2, 3, 4])


if __name__ == "__main__":
    unittest.main()
