"""Managed developer digest formatting. No installation or app control."""
import hashlib
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]


@unittest.skipUnless(os.name == 'nt', 'Windows native paths and PowerShell')
class DeveloperDigest(unittest.TestCase):
    def test_native_windows_path_digest_has_no_filename_escape_prefix(self):
        script = (ROOT / 'scripts/dev-install.sh').read_text(encoding='utf-8')
        function = re.search(r'(?ms)^sha256_file\(\) \{.*?^\}', script).group()
        with tempfile.TemporaryDirectory(prefix='hive digest ') as temporary:
            file = Path(temporary) / 'file with spaces'
            file.write_bytes(b'unchanged executable fixture')
            result = subprocess.run(['C:/Program Files/Git/bin/bash.exe', '--noprofile', '--norc', '-s', '--', str(file)],
                                    input=(function+'\nsha256_file "$1"\n').encode(), capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        self.assertEqual(result.stdout.decode().strip(), hashlib.sha256(b'unchanged executable fixture').hexdigest())

    def test_legacy_escape_is_accepted_but_damaged_records_are_refused(self):
        path = str(ROOT / 'scripts/complete-usage-monitor-install.ps1').replace("'", "''")
        script = r"""
$ErrorActionPreference='Stop';$tokens=$null;$errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile('PATH',[ref]$tokens,[ref]$errors)
$node=$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Parse-DeveloperDigest'},$false)[0]
Invoke-Expression $node.Extent.Text
$digest='a'*64
if((Parse-DeveloperDigest $digest) -ne $digest -or (Parse-DeveloperDigest ('\'+$digest)) -ne $digest){throw 'Valid digest rejected'}
foreach($invalid in @(('b'*63),'\bad',('\\'+$digest),('A'*64))){
 $rejected=$false;try{$null=Parse-DeveloperDigest $invalid}catch{$rejected=$true}
 if(-not $rejected){throw 'Damaged digest accepted'}
}
""".replace('PATH', path)
        exe = str(Path(os.environ['SystemRoot']) / 'System32/WindowsPowerShell/v1.0/powershell.exe')
        with tempfile.TemporaryDirectory(prefix='hive-digest-test-') as temporary:
            fixture = Path(temporary) / 'test.ps1'
            fixture.write_text(script, encoding='utf-8-sig')
            result = subprocess.run([exe, '-NoProfile', '-NonInteractive', '-File', str(fixture)], capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))

    def test_managed_rollback_accepts_an_exact_legacy_digest_without_editing_it(self):
        with tempfile.TemporaryDirectory(prefix='hive digest rollback ') as temporary:
            folder = Path(temporary)
            target = folder / 'fake hive.exe'
            target.write_bytes(b'developer fixture')
            state = folder / 'state'
            state.mkdir()
            (state / 'target-path').write_text(str(target)+'\n', encoding='utf-8')
            (state / 'original').write_bytes(b'original fixture')
            (state / 'developer-sha256').write_text('\\'+hashlib.sha256(target.read_bytes()).hexdigest()+'\n', encoding='utf-8')
            native = state.as_posix()
            unix = '/'+native[0].lower()+native[2:]
            result = subprocess.run(['C:/Program Files/Git/bin/bash.exe', '--noprofile', '--norc',
                                     str(ROOT / 'scripts/dev-install.sh'), '--rollback', '--state-dir', unix],
                                    capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
            self.assertEqual(target.read_bytes(), b'original fixture')
            self.assertFalse(state.exists())


if __name__ == '__main__':
    unittest.main()
