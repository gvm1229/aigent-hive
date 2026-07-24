#!/usr/bin/env python3
"""Phase 6 signed release, migration, recovery, and packaging conformance."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from typing import Any

import yaml
from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.exceptions import ValidationError


ROOT = Path(__file__).resolve().parents[2]
SCHEMAS = ROOT / "schemas"
RELEASE_FIXTURE = ROOT / "tests/fixtures/phase6/releases/valid-0.7.0"
WRONG_SIGNERS = ROOT / "tests/fixtures/phase6/platform-signers/wrong-valid-signers.json"
DIGEST = "sha256:" + "0" * 64


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object: {path}")
    return value


def validate(schema_name: str, value: object) -> None:
    schema = read_json(SCHEMAS / schema_name)
    Draft202012Validator(
        schema,
        format_checker=FormatChecker(),
    ).validate(value)


class Phase6StaticContracts(unittest.TestCase):
    def test_every_new_schema_is_valid_and_representative_instances_pass(self) -> None:
        names = (
            "backup-manifest.schema.json",
            "historical-builtins.schema.json",
            "historical-surfaces.schema.json",
            "major-release-confirmation.schema.json",
            "migration-table.schema.json",
            "platform-signing-evidence.schema.json",
            "release-bundle-manifest.schema.json",
            "release-surface-inventory.schema.json",
            "update-journal.schema.json",
            "update-state.schema.json",
        )
        for name in names:
            with self.subTest(name=name):
                Draft202012Validator.check_schema(read_json(SCHEMAS / name))

        validate(
            "historical-builtins.schema.json",
            yaml.safe_load(
                (
                    ROOT / "harness/skills/historical-builtins.yml"
                ).read_text(encoding="utf-8")
            ),
        )
        validate(
            "historical-surfaces.schema.json",
            yaml.safe_load(
                (
                    ROOT / "harness/release/historical-surfaces.yml"
                ).read_text(encoding="utf-8")
            ),
        )
        validate(
            "release-bundle-manifest.schema.json",
            read_json(RELEASE_FIXTURE / "targets/bundle-manifest.json"),
        )
        validate(
            "migration-table.schema.json",
            read_json(RELEASE_FIXTURE / "targets/migration-table.json"),
        )
        validate(
            "release-surface-inventory.schema.json",
            read_json(
                RELEASE_FIXTURE / "targets/release-surface-inventory.json"
            ),
        )
        validate(
            "platform-signing-evidence.schema.json",
            read_json(
                RELEASE_FIXTURE / "targets/platform-signing-evidence.json"
            ),
        )
        wrong_signer = read_json(
            RELEASE_FIXTURE / "targets/platform-signing-evidence.json"
        )
        wrong_signer["evidence"][0]["signer"] = {
            "kind": "authenticode-certificate-thumbprint",
            "value": "A" * 40,
        }
        with self.assertRaises(ValidationError):
            validate("platform-signing-evidence.schema.json", wrong_signer)
        validate(
            "backup-manifest.schema.json",
            {
                "schema_version": 1,
                "transaction_id": "txn-" + "a" * 24,
                "source_version": "0.6.0",
                "target_version": "0.7.0",
                "created_at_unix": 1,
                "expires_at_unix": 604801,
                "tree_digest": DIGEST,
                "entries": [
                    {
                        "path": ".hive/config/harness.toml",
                        "ownership": "canonical-protected",
                        "prior_digest": DIGEST,
                        "prior_length": 1,
                        "backup_path": "files/.hive/config/harness.toml",
                    }
                ],
                "manifest_digest": DIGEST,
            },
        )
        validate(
            "update-journal.schema.json",
            {
                "schema_version": 1,
                "transaction_id": "txn-" + "a" * 24,
                "state": "prepared",
                "source_version": "0.6.0",
                "target_version": "0.7.0",
                "release_manifest_digest": DIGEST,
                "plan_digest": DIGEST,
                "backup_manifest_path": (
                    ".hive/backups/txn-" + "a" * 24 + "/backup-manifest.json"
                ),
                "changes": [
                    {
                        "path": ".hive/config/harness.toml",
                        "before_digest": DIGEST,
                        "after_digest": DIGEST,
                        "backup_path": "files/.hive/config/harness.toml",
                    }
                ],
                "next_state": {
                    "schema_version": 1,
                    "product_version": "0.7.0",
                    "release_manifest_digest": DIGEST,
                    "rollback": {
                        "root_version": 1,
                        "timestamp_version": 1,
                        "snapshot_version": 1,
                        "targets_version": 1,
                        "release_sequence": 7,
                        "manifest_digest": DIGEST,
                    },
                },
                "journal_digest": DIGEST,
            },
        )
        validate(
            "update-state.schema.json",
            {
                "schema_version": 1,
                "product_version": "0.7.0",
                "release_manifest_digest": DIGEST,
                "rollback": {
                    "root_version": 1,
                    "timestamp_version": 1,
                    "snapshot_version": 1,
                    "targets_version": 1,
                    "release_sequence": 7,
                    "manifest_digest": DIGEST,
                },
            },
        )

    def test_release_fixture_contains_public_material_only(self) -> None:
        for fixture_root in (RELEASE_FIXTURE, WRONG_SIGNERS.parent):
            for path in fixture_root.rglob("*"):
                if not path.is_file():
                    continue
                text = path.read_text(encoding="utf-8").casefold()
                self.assertNotIn("private_key", text, path)
                self.assertNotIn("secret_key", text, path)
                self.assertNotIn("signing_seed", text, path)
                self.assertNotIn("begin private key", text, path)
        wrong_signers = read_json(WRONG_SIGNERS)
        self.assertRegex(wrong_signers["macos"]["team_id"], r"^[A-Z0-9]{10}$")
        self.assertRegex(
            wrong_signers["windows"]["certificate_thumbprint"],
            r"^[0-9A-F]{40}$",
        )
        cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('ed25519-dalek = { version = "=3.0.0", default-features = false }', cargo)
        update_source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted((ROOT / "crates/hive-update/src").glob("*.rs"))
        )
        self.assertNotIn("SigningKey", update_source)

    def test_signed_release_workflows_separate_build_authority_and_publication(self) -> None:
        candidate = (ROOT / ".github/workflows/release.yml").read_text(
            encoding="utf-8"
        )
        publication = (
            ROOT / ".github/workflows/release-publish.yml"
        ).read_text(encoding="utf-8")
        yaml.safe_load(candidate)
        yaml.safe_load(publication)
        for required in (
            "macos-15",
            "macos-15-intel",
            "windows-2025",
            "codesign --verify --strict",
            "notarytool submit",
            "azure/artifact-signing-action@",
            "ARTIFACT_SIGNING_CERTIFICATE_THUMBPRINT",
            "apple-team-id",
            "authenticode-certificate-thumbprint",
            "actions/attest@",
            "release-signing",
        ):
            self.assertIn(required, candidate)
        for required in (
            "signed_tuf_repository_url",
            "HIVE_RELEASE_ROOT_JSON_BASE64",
            "hive release verify",
            "CANDIDATE_SHA",
            "authorized_sha",
            'test "$authorized_sha" = "$CANDIDATE_SHA"',
            "gh attestation verify",
            "platform-signing-evidence.canonical.json",
            "__AIGENT_HIVE_APPLE_TEAM_ID__",
            "__AIGENT_HIVE_WINDOWS_CERTIFICATE_THUMBPRINT__",
            "dist/install.sh",
            "dist/install.ps1",
            'cmp "$artifact"',
            "release-publication",
            "gh release create",
        ):
            self.assertIn(required, publication)
        self.assertNotIn("gh release create", candidate)
        self.assertNotIn("eval ", candidate + publication)

    def test_dispatch_inputs_are_never_interpolated_into_run_scripts(self) -> None:
        def run_scripts(value: object) -> list[str]:
            if isinstance(value, dict):
                scripts = [
                    child
                    for key, child in value.items()
                    if key == "run" and isinstance(child, str)
                ]
                return scripts + [
                    script
                    for child in value.values()
                    for script in run_scripts(child)
                ]
            if isinstance(value, list):
                return [
                    script
                    for child in value
                    for script in run_scripts(child)
                ]
            return []

        for name in ("release.yml", "release-publish.yml"):
            text = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
            workflow = yaml.safe_load(text)
            scripts = run_scripts(workflow)
            self.assertTrue(scripts)
            for script in scripts:
                self.assertNotIn("${{ inputs.", script, name)

    def test_direct_homebrew_and_winget_paths_preserve_binary_ownership(self) -> None:
        shell = (ROOT / "scripts/install.sh").read_text(encoding="utf-8")
        powershell = (ROOT / "scripts/install.ps1").read_text(encoding="utf-8")
        formula = (
            ROOT / "packaging/homebrew/aigent-hive.rb.in"
        ).read_text(encoding="utf-8")
        winget = (
            ROOT / "packaging/winget/Gvm1229.AigentHive.installer.yaml.in"
        ).read_text(encoding="utf-8")
        self.assertIn("codesign --verify --strict", shell)
        self.assertIn("spctl --assess --type execute", shell)
        self.assertIn("release archive contains an unexpected path", shell)
        self.assertIn("existing hive binary is not owned by the direct installer", shell)
        self.assertIn("Get-AuthenticodeSignature", powershell)
        self.assertIn(
            '__AIGENT_HIVE_WINDOWS_CERTIFICATE_THUMBPRINT__',
            powershell,
        )
        self.assertIn("__AIGENT_HIVE_APPLE_TEAM_ID__", shell)
        self.assertNotIn("AIGENT_HIVE_MACOS_TEAM_ID", shell)
        self.assertIn("TeamIdentifier", shell)
        self.assertIn("SignerCertificate.Thumbprint", powershell)
        parameter_block = re.search(
            r"(?s)^param\((.*?)\)\n\n\$ErrorActionPreference",
            powershell,
        )
        self.assertIsNotNone(parameter_block)
        self.assertNotIn("AuthorizedSigner", parameter_block.group(1))
        self.assertIn("[IO.Compression.ZipFile]::OpenRead", powershell)
        self.assertIn(
            "existing hive binary is not owned by the direct installer",
            powershell,
        )
        self.assertIn('owner":"direct"', shell)
        self.assertIn('owner = "direct"', powershell)
        self.assertNotIn("grep -q", shell)
        self.assertIn('owned_digest=$(shasum -a 256 "$owned_binary"', shell)
        self.assertIn('[ "$parsed_digest" = "$owned_digest" ]', shell)
        self.assertIn('verify_owned_pair "$prefix/bin/hive" "$receipt"', shell)
        self.assertIn("Compare-Object $expectedProperties $actualProperties", powershell)
        self.assertIn(
            "if ($null -eq $destinationItem -or $null -eq $receiptItem)",
            powershell,
        )
        self.assertIn("$destinationItem.PSIsContainer", powershell)
        self.assertIn("$receiptItem.PSIsContainer", powershell)
        self.assertIn("[IO.FileAttributes]::ReparsePoint", powershell)
        self.assertIn("Get-FileHash -LiteralPath $Destination", powershell)
        self.assertIn(
            '$priorReceipt.artifact_sha256 -ne "sha256:$priorDigest"',
            powershell,
        )
        self.assertNotIn("AIGENT_HIVE_RELEASE_BASE", shell)
        self.assertNotIn("ReleaseBase", powershell)
        self.assertIn("on_arm do", formula)
        self.assertIn("on_intel do", formula)
        self.assertIn("PortableCommandAlias: hive", winget)
        for skill in ("hive-update", "hive-migrate"):
            text = (ROOT / f"harness/skills/{skill}/SKILL.md").read_text(
                encoding="utf-8"
            )
            self.assertIn("Homebrew", text)
            self.assertIn("WinGet", text)
            self.assertNotIn("curl ", text)

    def test_macos_installer_rejects_valid_signature_from_wrong_team(self) -> None:
        wrong_team = read_json(WRONG_SIGNERS)["macos"]["team_id"]
        self.assertRegex(wrong_team, r"^[A-Z0-9]{10}$")
        source = (ROOT / "scripts/install.sh").read_text(encoding="utf-8")
        rendered = source.replace(
            "__AIGENT_HIVE_APPLE_TEAM_ID__",
            "FIXTURE123",
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            installer = root / "install.sh"
            installer.write_text(rendered, encoding="utf-8")
            installer.chmod(0o755)
            commands = root / "commands"
            commands.mkdir()
            mocks = {
                "uname": """#!/bin/sh
if [ "$1" = "-s" ]; then printf 'Darwin\\n'; else printf 'arm64\\n'; fi
""",
                "curl": """#!/bin/sh
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output" ]; then shift; output=$1; fi
  shift
done
case "$output" in
  *.sha256) printf '%064d  archive\\n' 0 >"$output" ;;
  *) : >"$output" ;;
esac
""",
                "shasum": """#!/bin/sh
printf '%064d  artifact\\n' 0
""",
                "tar": """#!/bin/sh
case "$1" in
  -tzf) printf 'aigent-hive-0.7.0-aarch64-apple-darwin/hive\\naigent-hive-0.7.0-aarch64-apple-darwin/LICENSE\\n' ;;
  -tvzf) printf '%s\\n%s\\n' '-rwxr-xr-x 0/0 1 1980-01-01 00:00 aigent-hive-0.7.0-aarch64-apple-darwin/hive' '-rw-r--r-- 0/0 1 1980-01-01 00:00 aigent-hive-0.7.0-aarch64-apple-darwin/LICENSE' ;;
  -xzf)
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-C" ]; then shift; destination=$1; fi
      shift
    done
    package="$destination/aigent-hive-0.7.0-aarch64-apple-darwin"
    mkdir -p "$package"
    printf '%s\\n' '#!/bin/sh' 'printf \"hive 0.7.0\\\\n\"' >"$package/hive"
    chmod 0755 "$package/hive"
    : >"$package/LICENSE"
    ;;
esac
""",
                "codesign": f"""#!/bin/sh
case "$1" in
  -dv) printf 'TeamIdentifier={wrong_team}\\n' >&2 ;;
esac
""",
                "spctl": "#!/bin/sh\nexit 0\n",
            }
            for name, contents in mocks.items():
                path = commands / name
                path.write_text(contents, encoding="utf-8")
                path.chmod(0o755)
            environment = os.environ.copy()
            environment.update(
                {
                    "AIGENT_HIVE_VERSION": "0.7.0",
                    "AIGENT_HIVE_PREFIX": str(root / "prefix"),
                    "PATH": f"{commands}{os.pathsep}{environment['PATH']}",
                }
            )
            result = subprocess.run(
                [str(installer)],
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
        self.assertEqual(result.returncode, 5, result)
        self.assertIn(
            "signed binary signer differs from the authorized release identity",
            result.stderr,
        )

    def test_windows_ownership_function_executes_when_pwsh_is_available(self) -> None:
        pwsh = shutil.which("pwsh")
        if pwsh is None or sys.platform != "win32":
            self.skipTest("Windows pwsh is unavailable on this host")
        command = r"""
$errors = $null
$tokens = $null
$ast = [Management.Automation.Language.Parser]::ParseFile(
    $env:HIVE_INSTALLER_PATH,
    [ref]$tokens,
    [ref]$errors
)
if ($errors.Count -ne 0) { throw ($errors | Out-String) }
foreach ($name in @(
    "Assert-ExistingDirectInstall",
    "Assert-AuthorizedAuthenticodeSignature"
)) {
    $function = $ast.Find(
        {
            param($node)
            $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -eq $name
        },
        $true
    )
    if ($null -eq $function) { throw "installer validation function missing: $name" }
    Invoke-Expression $function.Extent.Text
}
$wrongSignature = [pscustomobject]@{
    Status = "Valid"
    SignerCertificate = [pscustomobject]@{
        Thumbprint = $env:HIVE_WRONG_WINDOWS_THUMBPRINT
    }
}
try {
    Assert-AuthorizedAuthenticodeSignature `
        -Signature $wrongSignature `
        -AuthorizedThumbprint "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    throw "wrong Authenticode signer was accepted"
} catch {
    if ($_.Exception.Message -eq "wrong Authenticode signer was accepted") { throw }
}
$root = Join-Path ([IO.Path]::GetTempPath()) ("hive-owner-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $root | Out-Null
function Assert-Rejected {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Operation,
        [Parameter(Mandatory = $true)]
        [string]$Label
    )
    try {
        & $Operation
        throw "$Label was accepted"
    } catch {
        if ($_.Exception.Message -eq "$Label was accepted") { throw }
    }
}
try {
    $binary = Join-Path $root "hive.exe"
    $receipt = Join-Path $root "install-receipt.json"
    Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8NoBOM
    Assert-Rejected -Label "receipt-only state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-Item -LiteralPath $receipt
    Set-Content -LiteralPath $binary -Value "not an executable" -Encoding utf8NoBOM
    Assert-Rejected -Label "binary-only state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-Item -LiteralPath $binary
    New-Item -ItemType Directory -Path $binary, $receipt | Out-Null
    Assert-Rejected -Label "nonregular state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-Item -LiteralPath $binary, $receipt -Recurse -Force
    $executionMarker = Join-Path $root "ownership-probe-executed"
    $env:HIVE_EXECUTION_MARKER = $executionMarker
    Add-Type -TypeDefinition @"
using System;
using System.IO;
public static class HiveOwnershipProbe {
    public static int Main(string[] args) {
        File.WriteAllText(
            Environment.GetEnvironmentVariable("HIVE_EXECUTION_MARKER"),
            "executed"
        );
        Console.WriteLine("hive 0.7.0");
        return 0;
    }
}
"@ -OutputAssembly $binary -OutputType ConsoleApplication
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8NoBOM
    Assert-Rejected -Label "malformed receipt" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    if (Test-Path -LiteralPath $executionMarker) {
        throw "malformed receipt executed the unowned binary"
    }
    @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = "0.7.0"
        artifact_sha256 = "sha256:" + ("0" * 64)
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $receipt -Encoding utf8NoBOM
    Assert-Rejected -Label "mismatched receipt" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    if (Test-Path -LiteralPath $executionMarker) {
        throw "mismatched receipt executed the unowned binary"
    }
    Remove-Item -LiteralPath $binary, $receipt
    $targetDirectory = Join-Path $root "reparse-target"
    New-Item -ItemType Directory -Path $targetDirectory | Out-Null
    $binaryLink = New-Item `
        -ItemType Junction `
        -Path $binary `
        -Target $targetDirectory `
        -ErrorAction Stop
    if (-not ($binaryLink.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "binary junction is not a reparse point"
    }
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8NoBOM
    Assert-Rejected -Label "binary reparse-point state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-Item -LiteralPath $binary, $receipt -Force
    Set-Content -LiteralPath $binary -Value "not an executable" -Encoding utf8NoBOM
    $receiptLink = New-Item `
        -ItemType Junction `
        -Path $receipt `
        -Target $targetDirectory `
        -ErrorAction Stop
    if (-not ($receiptLink.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "receipt junction is not a reparse point"
    }
    Assert-Rejected -Label "receipt reparse-point state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
} finally {
    Remove-Item -LiteralPath $root -Recurse -Force
}
"""
        environment = os.environ.copy()
        environment["HIVE_INSTALLER_PATH"] = str(ROOT / "scripts/install.ps1")
        environment["HIVE_WRONG_WINDOWS_THUMBPRINT"] = read_json(
            WRONG_SIGNERS
        )["windows"]["certificate_thumbprint"]
        result = subprocess.run(
            [pwsh, "-NoProfile", "-NonInteractive", "-Command", command],
            cwd=ROOT,
            env=environment,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_release_shell_entrypoints_are_executable_and_reject_bad_versions(self) -> None:
        for relative in ("scripts/check-release-version.sh", "scripts/install.sh"):
            path = ROOT / relative
            self.assertTrue(os.access(path, os.X_OK), relative)
        release_gate = subprocess.run(
            [str(ROOT / "scripts/check-release-version.sh"), "01.0.0"],
            cwd=ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(release_gate.returncode, 2)
        self.assertIn("exact X.Y.Z", release_gate.stderr)
        environment = os.environ.copy()
        environment["AIGENT_HIVE_VERSION"] = "01.0.0"
        installer = subprocess.run(
            [str(ROOT / "scripts/install.sh")],
            cwd=ROOT,
            env=environment,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(installer.returncode, 2)
        self.assertIn("exact X.Y.Z", installer.stderr)

    def test_product_version_matches_signed_feature_fixture(self) -> None:
        cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        version = re.search(
            r"\[workspace\.package\]\s+version = \"([^\"]+)\"",
            cargo,
        )
        self.assertIsNotNone(version)
        manifest = read_json(RELEASE_FIXTURE / "targets/bundle-manifest.json")
        migration = read_json(RELEASE_FIXTURE / "targets/migration-table.json")
        self.assertEqual(version.group(1), "0.7.0")
        self.assertEqual(manifest["release_version"], version.group(1))
        self.assertEqual(migration["target_version"], version.group(1))
        harness = (
            ROOT / "harness/template/.hive/config/harness.toml.jinja"
        ).read_text(encoding="utf-8")
        self.assertIn(f'harness_version = "{version.group(1)}"', harness)
        self.assertIn(
            f'source_release_version = "{version.group(1)}"',
            harness,
        )
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn(
            f"version-{version.group(1)}-",
            readme,
        )
        self.assertRegex(
            readme,
            rf"(?m)^\| Product version \| `{re.escape(version.group(1))}` \|$",
        )
        self.assertIn(
            f"- product version: `{version.group(1)}`",
            (ROOT / "docs/state/CURRENT.md").read_text(encoding="utf-8"),
        )
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
        hive_packages = [
            package
            for package in lock["package"]
            if package["name"].startswith("hive-")
        ]
        self.assertTrue(hive_packages)
        self.assertTrue(
            all(package["version"] == version.group(1) for package in hive_packages)
        )


class Phase6CliContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        configured = os.environ.get("HIVE_BIN")
        if configured:
            cls.binary = Path(configured).resolve()
            return
        subprocess.run(
            ["cargo", "build", "--quiet", "--locked", "--bin", "hive"],
            cwd=ROOT,
            check=True,
        )
        cls.binary = ROOT / "target/debug" / (
            "hive.exe" if os.name == "nt" else "hive"
        )

    def invoke(self, *arguments: str) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        process = subprocess.run(
            [str(self.binary), *arguments],
            cwd=ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        value = json.loads(process.stdout)
        validate("action-result.schema.json", value)
        self.assertEqual(process.returncode, value["exit_code"])
        return process, value

    def test_release_verify_requires_an_external_protected_absolute_root(self) -> None:
        process, result = self.invoke(
            "release",
            "verify",
            "--bundle",
            str(RELEASE_FIXTURE),
            "--trust-root",
            "metadata/root.json",
            "--output",
            "json",
        )
        self.assertEqual(process.returncode, 3)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(result["changed_paths"], [])

    def test_update_parser_refuses_partial_and_conflicting_authority(self) -> None:
        process, result = self.invoke(
            "update",
            "--target",
            str(ROOT),
            "--dry-run",
            "--output",
            "json",
        )
        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["code"], "hive.update-invalid-input")
        process, result = self.invoke(
            "update",
            "--target",
            str(ROOT),
            "--recover",
            "--exact-major-target",
            "1.0.0",
            "--output",
            "json",
        )
        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["changed_paths"], [])


if __name__ == "__main__":
    unittest.main()
