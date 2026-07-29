#!/usr/bin/env python3
"""Phase 6 signed release, migration, recovery, and packaging conformance."""

from __future__ import annotations

import json
import os
import re
import shutil
import stat
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


def macos_installer_fixture(root: Path, actual_team_id: str) -> tuple[Path, Path]:
    if sys.platform == "win32":
        raise unittest.SkipTest("macOS installer fixture is unavailable on Windows")
    source = (ROOT / "scripts/install.sh").read_text(encoding="utf-8")
    installer = root / "install.sh"
    installer.write_text(
        source.replace("__AIGENT_HIVE_APPLE_TEAM_ID__", "FIXTURE123"),
        encoding="utf-8",
    )
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
        "shasum": "#!/bin/sh\nprintf '%064d  artifact\\n' 0\n",
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
    printf '%s\\n' '#!/bin/sh' 'printf "hive 0.7.0\\\\n"' >"$package/hive"
    chmod 0755 "$package/hive"
    : >"$package/LICENSE"
    ;;
esac
""",
        "codesign": f"""#!/bin/sh
case "$1" in
  -dv) printf 'TeamIdentifier={actual_team_id}\\n' >&2 ;;
esac
""",
        "install": """#!/bin/sh
for destination do :; done
if [ -n "${HIVE_STAGE_BINARY_ATTACK:-}" ]; then
  rm -f "$destination"
  ln -s "$HIVE_STAGE_BINARY_OUTSIDE" "$destination"
fi
/usr/bin/install "$@"
if [ -n "${HIVE_STAGE_RECEIPT_ATTACK:-}" ]; then
  staged_receipt="$AIGENT_HIVE_PREFIX/share/aigent-hive/.install-receipt-$PPID"
  rm -f "$staged_receipt"
  ln -s "$HIVE_STAGE_RECEIPT_OUTSIDE" "$staged_receipt"
fi
""",
        "chmod": """#!/bin/sh
if [ "$1" != "-h" ]; then exec /bin/chmod "$@"; fi
if [ "$#" -ne 3 ]; then exit 64; fi
if [ -L "$3" ]; then exit 0; fi
exec /bin/chmod "$2" "$3"
""",
        "mktemp": """#!/bin/sh
created=$(/usr/bin/mktemp "$@") || exit
case "$created" in
  */.hive-install.*)
    if [ -n "${HIVE_STAGE_BINARY_ATTACK:-}" ]; then
      rm -f "$created"
      ln -s "$HIVE_STAGE_BINARY_OUTSIDE" "$created"
    fi
    ;;
  */.install-receipt.*)
    if [ -n "${HIVE_STAGE_RECEIPT_ATTACK:-}" ]; then
      rm -f "$created"
      ln -s "$HIVE_STAGE_RECEIPT_OUTSIDE" "$created"
    fi
    ;;
esac
printf '%s\\n' "$created"
""",
        "mv": """#!/bin/sh
if [ -n "${HIVE_MV_ATTACK_CALL:-}" ]; then
  count=0
  if [ -f "$HIVE_MV_COUNT_FILE" ]; then count=$(cat "$HIVE_MV_COUNT_FILE"); fi
  count=$((count + 1))
  printf '%s\\n' "$count" >"$HIVE_MV_COUNT_FILE"
  if [ "$count" -eq "$HIVE_MV_ATTACK_CALL" ]; then
    for destination do :; done
    rm -f "$destination"
    ln -s "$HIVE_MV_OUTSIDE" "$destination"
  fi
fi
if [ "$#" -ne 3 ] || [ "$1" != "-fh" ]; then exit 64; fi
source=$2
destination=$3
if [ -L "$destination" ]; then rm -f "$destination" || exit; fi
exec /bin/mv -f "$source" "$destination"
""",
        "spctl": "#!/bin/sh\nexit 0\n",
        "stat": """#!/bin/sh
if [ "$#" -ne 3 ] || [ "$1" != "-f" ] || [ "$2" != "%Lp" ]; then exit 64; fi
python - "$3" <<'PY'
import os
import stat
import sys

print(format(stat.S_IMODE(os.lstat(sys.argv[1]).st_mode), "o"))
PY
""",
    }
    for name, contents in mocks.items():
        path = commands / name
        path.write_text(contents, encoding="utf-8")
        path.chmod(0o755)
    return installer, commands


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

        for name in ("release.yml", "release-publish.yml", "release-runtime.yml"):
            text = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
            workflow = yaml.safe_load(text)
            scripts = run_scripts(workflow)
            self.assertTrue(scripts)
            for script in scripts:
                self.assertNotIn("${{ inputs.", script, name)

    def test_unsigned_native_release_runtime_qualification_contract(self) -> None:
        path = ROOT / ".github/workflows/release-runtime.yml"
        text = path.read_text(encoding="utf-8")
        workflow = yaml.safe_load(text)
        self.assertIsInstance(workflow, dict)
        triggers = workflow.get("on", workflow.get(True))
        self.assertIsInstance(triggers, dict)
        self.assertIn("workflow_dispatch", triggers)
        self.assertEqual(triggers["push"]["branches"], ["develop", "main"])
        self.assertIn(
            ".github/workflows/release-runtime.yml",
            triggers["push"]["paths"],
        )
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        self.assertEqual(set(workflow["jobs"]), {"macos", "windows"})
        macos_matrix = workflow["jobs"]["macos"]["strategy"]["matrix"]["include"]
        self.assertEqual(
            {(entry["runner"], entry["target"]) for entry in macos_matrix},
            {
                ("macos-15", "aarch64-apple-darwin"),
                ("macos-15-intel", "x86_64-apple-darwin"),
            },
        )
        self.assertEqual(workflow["jobs"]["windows"]["runs-on"], "windows-2025")
        for required in (
            "push:",
            "workflow_dispatch:",
            "develop",
            "main",
            "permissions:",
            "contents: read",
            "macos-15",
            "macos-15-intel",
            "windows-2025",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "x86_64-pc-windows-msvc",
            "cargo build --release --locked",
            "GITHUB_WORKFLOW_SHA",
            "git rev-parse HEAD",
            "lipo -archs",
            "0x8664",
            "expected-entries.txt",
            '"$package/LICENSE" "$package/hive" | sort',
            "Compare-Object $expected $entries",
            'Compress-Archive -Path "$package\\*"',
            '$expected = @("LICENSE", "hive.exe")',
            "if ($LASTEXITCODE -ne 0)",
            "BINARY_DIGEST",
            "ARCHIVE_DIGEST",
            "hive.user-install-dry-run-complete",
            "hive.user-install-complete",
            "hive.user-install-valid",
            "GITHUB_STEP_SUMMARY",
        ):
            self.assertIn(required, text)
        self.assertEqual(text.count("--host antigravity"), 6)
        self.assertEqual(text.count("--dry-run --output json"), 2)
        self.assertEqual(text.count("--apply --output json"), 2)
        self.assertEqual(text.count("--validate --output json"), 2)
        for forbidden in (
            "actions/upload-artifact",
            "actions/attest",
            "id-token: write",
            "release-signing",
            "release-publication",
            "codesign",
            "notarytool",
            "artifact-signing",
            "gh release",
            "scripts/install.sh",
            "scripts/install.ps1",
        ):
            self.assertNotIn(forbidden, text)

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
        self.assertIn("ensure_safe_directory_chain", shell)
        self.assertIn('ensure_safe_directory_chain "$prefix"', shell)
        self.assertIn("Compare-Object $expectedProperties $actualProperties", powershell)
        self.assertIn(
            "if ($null -eq $destinationItem -or $null -eq $receiptItem)",
            powershell,
        )
        self.assertIn("$destinationItem.PSIsContainer", powershell)
        self.assertIn("$receiptItem.PSIsContainer", powershell)
        self.assertIn("[IO.FileAttributes]::ReparsePoint", powershell)
        self.assertIn("Assert-SafeDirectoryChain", powershell)
        self.assertIn("Repair-PendingDirectInstall", powershell)
        self.assertIn("[IO.File]::Move", powershell)
        self.assertIn("[IO.File]::Replace", powershell)
        self.assertNotIn("[IO.File]::Move($Source, $Destination, $true)", powershell)
        self.assertNotIn("Move-Item", powershell)
        self.assertNotRegex(powershell, r"catch\s*\{\s*\}")
        self.assertIn("install-receipt.pending.json", powershell)
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
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            installer, commands = macos_installer_fixture(root, wrong_team)
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

    def test_macos_installer_rejects_symlinked_install_ancestors_without_external_writes(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            installer, commands = macos_installer_fixture(root, "FIXTURE123")

            for scenario in ("ancestor", "prefix", "bin", "share"):
                with self.subTest(scenario=scenario):
                    case_root = root / scenario
                    case_root.mkdir()
                    outside = case_root / "outside"
                    outside.mkdir()
                    sentinel = outside / "sentinel"
                    sentinel.write_text("preserve", encoding="utf-8")
                    prefix = case_root / "prefix"
                    if scenario == "ancestor":
                        ancestor = case_root / "ancestor"
                        ancestor.symlink_to(outside, target_is_directory=True)
                        prefix = ancestor / "prefix"
                    elif scenario == "prefix":
                        prefix.symlink_to(outside, target_is_directory=True)
                    else:
                        prefix.mkdir()
                        if scenario == "bin":
                            (prefix / "bin").symlink_to(
                                outside,
                                target_is_directory=True,
                            )
                        else:
                            (prefix / "share").symlink_to(
                                outside,
                                target_is_directory=True,
                            )
                    environment = os.environ.copy()
                    environment.update(
                        {
                            "AIGENT_HIVE_VERSION": "0.7.0",
                            "AIGENT_HIVE_PREFIX": str(prefix),
                            "PATH": (
                                f"{commands}{os.pathsep}{environment['PATH']}"
                            ),
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
                    self.assertEqual(result.returncode, 3, result)
                    self.assertIn(
                        "install path contains a symlink or non-directory",
                        result.stderr,
                    )
                    self.assertEqual(
                        sentinel.read_text(encoding="utf-8"),
                        "preserve",
                    )
                    self.assertEqual(
                        sorted(path.name for path in outside.iterdir()),
                        ["sentinel"],
                    )

    def test_macos_installer_creates_owned_directories_with_mode_0755(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            installer, commands = macos_installer_fixture(root, "FIXTURE123")
            prefix = root / "prefix"
            environment = os.environ.copy()
            environment.update(
                {
                    "AIGENT_HIVE_VERSION": "0.7.0",
                    "AIGENT_HIVE_PREFIX": str(prefix),
                    "PATH": f"{commands}{os.pathsep}{environment['PATH']}",
                }
            )
            result = subprocess.run(
                ["sh", "-c", 'umask 000; exec "$1"', "sh", str(installer)],
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(result.returncode, 0, result)
            for directory in (
                prefix,
                prefix / "bin",
                prefix / "share",
                prefix / "share/aigent-hive",
            ):
                self.assertEqual(
                    stat.S_IMODE(directory.stat().st_mode),
                    0o755,
                    directory,
                )
            receipt = prefix / "share/aigent-hive/install-receipt.json"
            self.assertEqual(stat.S_IMODE(receipt.stat().st_mode), 0o644)
            self.assertEqual(stat.S_IMODE(receipt.stat().st_mode) & 0o022, 0)

    def test_macos_installer_leaf_renames_do_not_follow_injected_symlinks(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            installer, commands = macos_installer_fixture(root, "FIXTURE123")
            attack_calls = {
                "staged receipt to pending": 2,
                "staged binary to destination": 3,
                "pending receipt to final receipt": 4,
            }
            for label, attack_call in attack_calls.items():
                with self.subTest(label=label):
                    case_root = root / f"leaf-{attack_call}"
                    case_root.mkdir()
                    prefix = case_root / "prefix"
                    outside = case_root / "outside"
                    outside.mkdir()
                    sentinel = outside / "sentinel"
                    sentinel.write_bytes(b"preserve")
                    environment = os.environ.copy()
                    environment.update(
                        {
                            "AIGENT_HIVE_VERSION": "0.7.0",
                            "AIGENT_HIVE_PREFIX": str(prefix),
                            "HIVE_MV_ATTACK_CALL": str(attack_call),
                            "HIVE_MV_COUNT_FILE": str(case_root / "mv-count"),
                            "HIVE_MV_OUTSIDE": str(outside),
                            "PATH": (
                                f"{commands}{os.pathsep}{environment['PATH']}"
                            ),
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
                    self.assertEqual(result.returncode, 0, result)
                    self.assertEqual(sentinel.read_bytes(), b"preserve")
                    self.assertEqual(
                        sorted(path.name for path in outside.iterdir()),
                        ["sentinel"],
                    )
                    self.assertTrue((prefix / "bin/hive").is_file())
                    self.assertFalse((prefix / "bin/hive").is_symlink())
                    receipt = prefix / "share/aigent-hive/install-receipt.json"
                    self.assertTrue(receipt.is_file())
                    self.assertFalse(receipt.is_symlink())

    def test_macos_recovery_promotion_does_not_follow_injected_receipt_symlink(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            installer, commands = macos_installer_fixture(root, "FIXTURE123")
            prefix = root / "prefix"
            base_environment = os.environ.copy()
            base_environment.update(
                {
                    "AIGENT_HIVE_VERSION": "0.7.0",
                    "AIGENT_HIVE_PREFIX": str(prefix),
                    "PATH": f"{commands}{os.pathsep}{base_environment['PATH']}",
                }
            )
            first = subprocess.run(
                [str(installer)],
                cwd=ROOT,
                env=base_environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(first.returncode, 0, first)
            receipt = prefix / "share/aigent-hive/install-receipt.json"
            pending = prefix / "share/aigent-hive/install-receipt.pending.json"
            receipt.replace(pending)
            outside = root / "outside"
            outside.mkdir()
            sentinel = outside / "sentinel"
            sentinel.write_bytes(b"preserve")
            recovery_environment = base_environment.copy()
            recovery_environment.update(
                {
                    "HIVE_MV_ATTACK_CALL": "1",
                    "HIVE_MV_COUNT_FILE": str(root / "recovery-mv-count"),
                    "HIVE_MV_OUTSIDE": str(outside),
                }
            )
            second = subprocess.run(
                [str(installer)],
                cwd=ROOT,
                env=recovery_environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(second.returncode, 0, second)
            self.assertEqual(sentinel.read_bytes(), b"preserve")
            self.assertEqual(
                sorted(path.name for path in outside.iterdir()),
                ["sentinel"],
            )
            self.assertTrue(receipt.is_file())
            self.assertFalse(receipt.is_symlink())

    def test_macos_installer_does_not_follow_precreated_staged_leaf_symlinks(
        self,
    ) -> None:
        for leaf in ("binary", "receipt"):
            with self.subTest(leaf=leaf), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                installer, commands = macos_installer_fixture(root, "FIXTURE123")
                prefix = root / "prefix"
                outside = root / "outside"
                outside.mkdir()
                external_leaf = outside / leaf
                external_leaf.write_bytes(b"preserve")
                environment = os.environ.copy()
                environment.update(
                    {
                        "AIGENT_HIVE_VERSION": "0.7.0",
                        "AIGENT_HIVE_PREFIX": str(prefix),
                        f"HIVE_STAGE_{leaf.upper()}_ATTACK": "1",
                        f"HIVE_STAGE_{leaf.upper()}_OUTSIDE": str(external_leaf),
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
                self.assertEqual(result.returncode, 0, result)
                self.assertEqual(external_leaf.read_bytes(), b"preserve")

    def test_windows_ownership_functions_match_powershell_5_and_7(self) -> None:
        if sys.platform != "win32":
            self.skipTest("Windows PowerShell qualification requires Windows")
        shells = [
            executable
            for executable in (
                shutil.which("powershell.exe"),
                shutil.which("pwsh.exe"),
            )
            if executable is not None
        ]
        self.assertGreaterEqual(len(shells), 1)
        command = r"""
$utilityModule = Join-Path $PSHOME (
    "Modules\Microsoft.PowerShell.Utility\Microsoft.PowerShell.Utility.psd1"
)
Import-Module $utilityModule -ErrorAction Stop
$errors = $null
$tokens = $null
$ast = [Management.Automation.Language.Parser]::ParseFile(
    $env:HIVE_INSTALLER_PATH,
    [ref]$tokens,
    [ref]$errors
)
if ($errors.Count -ne 0) { throw ($errors | Out-String) }
foreach ($name in @(
    "Get-ValidatedDirectReceipt",
    "Assert-ExistingDirectInstall",
    "Assert-AuthorizedAuthenticodeSignature",
    "Assert-SafeDirectoryChain",
    "Repair-PendingDirectInstall",
    "Move-InstallFile"
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
function Remove-TestJunction {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )
    [IO.Directory]::Delete($Path)
}
try {
    $binary = Join-Path $root "hive.cmd"
    $receipt = Join-Path $root "install-receipt.json"
    Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8
    Assert-Rejected -Label "receipt-only state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-Item -LiteralPath $receipt
    Set-Content -LiteralPath $binary -Value "not an executable" -Encoding utf8
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
    Set-Content -LiteralPath $binary -Value @(
        '@echo off',
        '> "%HIVE_EXECUTION_MARKER%" echo executed',
        'echo hive 0.7.0'
    ) -Encoding ascii
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8
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
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $receipt -Encoding utf8
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
    Set-Content -LiteralPath $receipt -Value "{}" -Encoding utf8
    Assert-Rejected -Label "binary reparse-point state" -Operation {
        Assert-ExistingDirectInstall -Destination $binary -ReceiptPath $receipt
    }
    Remove-TestJunction -Path $binary
    Remove-Item -LiteralPath $receipt -Force
    Set-Content -LiteralPath $binary -Value "not an executable" -Encoding utf8
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
    Remove-Item -LiteralPath $binary -Force
    Remove-TestJunction -Path $receipt

    $outside = Join-Path $root "outside"
    New-Item -ItemType Directory -Path $outside | Out-Null
    $sentinel = Join-Path $outside "sentinel"
    Set-Content -LiteralPath $sentinel -Value "preserve" -Encoding utf8
    $linkedPrefix = Join-Path $root "linked-prefix"
    New-Item -ItemType Junction -Path $linkedPrefix -Target $outside | Out-Null
    Assert-Rejected -Label "prefix reparse-point ancestor" -Operation {
        Assert-SafeDirectoryChain -Path (Join-Path $linkedPrefix "bin")
    }
    if (
        (Get-Content -LiteralPath $sentinel -Raw).Trim() -ne "preserve" -or
        (@(Get-ChildItem -LiteralPath $outside -Force).Count -ne 1) -or
        ((Get-ChildItem -LiteralPath $outside -Force).Name -ne "sentinel")
    ) {
        throw "prefix reparse-point ancestor changed external state"
    }
    Remove-TestJunction -Path $linkedPrefix

    $linkedAncestor = Join-Path $root "linked-ancestor"
    New-Item -ItemType Junction -Path $linkedAncestor -Target $outside | Out-Null
    Assert-Rejected -Label "reparse point above prefix" -Operation {
        Assert-SafeDirectoryChain -Path (Join-Path $linkedAncestor "prefix\bin")
    }
    if (
        (@(Get-ChildItem -LiteralPath $outside -Force).Count -ne 1) -or
        ((Get-ChildItem -LiteralPath $outside -Force).Name -ne "sentinel")
    ) {
        throw "reparse point above prefix changed external state"
    }
    Remove-TestJunction -Path $linkedAncestor

    $safePrefix = Join-Path $root "safe-prefix"
    New-Item -ItemType Directory -Path $safePrefix | Out-Null
    $linkedBin = Join-Path $safePrefix "bin"
    New-Item -ItemType Junction -Path $linkedBin -Target $outside | Out-Null
    Assert-Rejected -Label "bin reparse-point ancestor" -Operation {
        Assert-SafeDirectoryChain -Path $linkedBin
    }
    if ((Test-Path -LiteralPath (Join-Path $outside "hive.exe"))) {
        throw "bin reparse-point ancestor changed external state"
    }
    Remove-TestJunction -Path $linkedBin

    $linkedShare = Join-Path $safePrefix "share"
    New-Item -ItemType Junction -Path $linkedShare -Target $outside | Out-Null
    Assert-Rejected -Label "share reparse-point ancestor" -Operation {
        Assert-SafeDirectoryChain -Path (Join-Path $linkedShare "aigent-hive")
    }
    if ((Test-Path -LiteralPath (Join-Path $outside "aigent-hive"))) {
        throw "share reparse-point ancestor changed external state"
    }
    Remove-TestJunction -Path $linkedShare

    foreach ($leafCase in @(
        [pscustomobject]@{ Label = "staged receipt to pending"; Replace = $false },
        [pscustomobject]@{ Label = "staged binary to destination"; Replace = $true },
        [pscustomobject]@{ Label = "pending receipt to receipt"; Replace = $true }
    )) {
        $sourceLeaf = Join-Path $safePrefix ([Guid]::NewGuid().ToString())
        Set-Content -LiteralPath $sourceLeaf -Value "owned" -Encoding utf8
        $destinationLeaf = Join-Path $safePrefix ([Guid]::NewGuid().ToString())
        New-Item -ItemType Junction -Path $destinationLeaf -Target $outside | Out-Null
        Assert-Rejected -Label $leafCase.Label -Operation {
            Move-InstallFile `
                -Source $sourceLeaf `
                -Destination $destinationLeaf `
                -Replace $leafCase.Replace
        }
        if (
            -not (Test-Path -LiteralPath $sourceLeaf) -or
            (@(Get-ChildItem -LiteralPath $outside -Force).Count -ne 1) -or
            ((Get-ChildItem -LiteralPath $outside -Force).Name -ne "sentinel")
        ) {
            throw "$($leafCase.Label) changed external state"
        }
        Remove-Item -LiteralPath $sourceLeaf -Force
        Remove-TestJunction -Path $destinationLeaf
    }

    $replaceSource = Join-Path $safePrefix "replace-source"
    $replaceDestination = Join-Path $safePrefix "replace-destination"
    [IO.File]::WriteAllText($replaceSource, "new", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($replaceDestination, "old", [Text.UTF8Encoding]::new($false))
    Move-InstallFile `
        -Source $replaceSource `
        -Destination $replaceDestination `
        -Replace $true
    if (
        (Test-Path -LiteralPath $replaceSource) -or
        [IO.File]::ReadAllText($replaceDestination) -ne "new"
    ) {
        throw "regular replacement did not preserve atomic leaf semantics"
    }

    $utf8Probe = Join-Path $safePrefix "utf8-probe"
    $utf8Bytes = [Text.UTF8Encoding]::new($false).GetBytes('{"schema_version":1}')
    [IO.File]::WriteAllBytes($utf8Probe, $utf8Bytes)
    $writtenBytes = [IO.File]::ReadAllBytes($utf8Probe)
    if (
        $writtenBytes.Length -ge 3 -and
        $writtenBytes[0] -eq 0xEF -and
        $writtenBytes[1] -eq 0xBB -and
        $writtenBytes[2] -eq 0xBF
    ) {
        throw "UTF-8 receipt probe contains a BOM"
    }

    $transactionRoot = Join-Path $root "transaction"
    New-Item -ItemType Directory -Path $transactionRoot | Out-Null
    $transactionBinary = Join-Path $transactionRoot "hive.exe"
    $transactionReceipt = Join-Path $transactionRoot "install-receipt.json"
    $pendingReceipt = Join-Path $transactionRoot "install-receipt.pending.json"
    Set-Content -LiteralPath $transactionBinary -Value "new binary" -Encoding utf8
    $transactionDigest = (
        Get-FileHash -LiteralPath $transactionBinary -Algorithm SHA256
    ).Hash.ToLowerInvariant()
    @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = "0.7.0"
        artifact_sha256 = "sha256:$transactionDigest"
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $pendingReceipt -Encoding utf8
    Repair-PendingDirectInstall `
        -Destination $transactionBinary `
        -ReceiptPath $transactionReceipt `
        -PendingReceiptPath $pendingReceipt
    if (
        (Test-Path -LiteralPath $pendingReceipt) -or
        -not (Test-Path -LiteralPath $transactionReceipt)
    ) {
        throw "matching pending receipt was not promoted"
    }

    $retainedReceipt = Get-Content -LiteralPath $transactionReceipt -Raw
    @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = "0.7.1"
        artifact_sha256 = "sha256:" + ("0" * 64)
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $pendingReceipt -Encoding utf8
    Repair-PendingDirectInstall `
        -Destination $transactionBinary `
        -ReceiptPath $transactionReceipt `
        -PendingReceiptPath $pendingReceipt
    if (
        (Test-Path -LiteralPath $pendingReceipt) -or
        (Get-Content -LiteralPath $transactionReceipt -Raw) -ne $retainedReceipt
    ) {
        throw "valid old pair was not retained"
    }

    Set-Content -LiteralPath $pendingReceipt -Value "{}" -Encoding utf8
    Assert-Rejected -Label "malformed pending receipt" -Operation {
        Repair-PendingDirectInstall `
            -Destination $transactionBinary `
            -ReceiptPath $transactionReceipt `
            -PendingReceiptPath $pendingReceipt
    }
    Remove-Item -LiteralPath $pendingReceipt
    New-Item -ItemType Junction -Path $pendingReceipt -Target $outside | Out-Null
    Assert-Rejected -Label "pending receipt reparse point" -Operation {
        Repair-PendingDirectInstall `
            -Destination $transactionBinary `
            -ReceiptPath $transactionReceipt `
            -PendingReceiptPath $pendingReceipt
    }
    Remove-TestJunction -Path $pendingReceipt
    Remove-Item -LiteralPath $transactionReceipt
    @{
        schema_version = 1
        owner = "direct"
        product = "aigent-hive"
        version = "0.7.0"
        artifact_sha256 = "sha256:" + ("0" * 64)
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $pendingReceipt -Encoding utf8
    Assert-Rejected -Label "mismatched pending receipt" -Operation {
        Repair-PendingDirectInstall `
            -Destination $transactionBinary `
            -ReceiptPath $transactionReceipt `
            -PendingReceiptPath $pendingReceipt
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
        for shell in shells:
            with self.subTest(shell=shell):
                result = subprocess.run(
                    [
                        shell,
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        command,
                    ],
                    cwd=ROOT,
                    env=environment,
                    check=False,
                    text=True,
                    capture_output=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_windows_shell_install_boundaries_are_explicit(self) -> None:
        installer = (ROOT / "scripts/install.ps1").read_text(encoding="utf-8")
        dependency_setup = (
            ROOT / "scripts/setup-windows-dependencies.ps1"
        ).read_text(encoding="utf-8")
        guide = (
            ROOT / "docs/guides/signed-update-and-release.md"
        ).read_text(encoding="utf-8")
        for forbidden in ("pwsh", "winget", "Microsoft.PowerShell"):
            self.assertNotIn(forbidden, installer)
        for required in (
            'PackageId = "Microsoft.PowerShell"',
            'PackageVersion = "7.6.4.0"',
            '[switch]$ConfirmInstall',
            'ValidateSet("user", "machine")',
            "install-required",
            "already-satisfied",
            "requalification failed",
        ):
            self.assertIn(required, dependency_setup)
        self.assertIn("powershell.exe -NoLogo -NoProfile -NonInteractive", guide)
        self.assertIn('set "HIVE_VERSION=0.7.0"', guide)
        self.assertIn('set "HIVE_PREFIX=%LOCALAPPDATA%\\AigentHive"', guide)
        self.assertIn("$env:HIVE_VERSION", guide)
        self.assertIn("$env:HIVE_PREFIX", guide)
        self.assertNotIn("pwsh -", guide)

    def test_windows_source_dependency_preview_is_non_mutating(self) -> None:
        if sys.platform != "win32":
            self.skipTest("Windows dependency setup qualification requires Windows")
        powershell = shutil.which("powershell.exe")
        self.assertIsNotNone(powershell)
        script = ROOT / "scripts/setup-windows-dependencies.ps1"
        result = subprocess.run(
            [
                powershell,
                "-NoProfile",
                "-NonInteractive",
                "-File",
                str(script),
            ],
            cwd=ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertIn(
            payload["status"],
            {"already-satisfied", "install-required"},
        )
        self.assertFalse(payload["changed"])
        self.assertEqual(payload["package_id"], "Microsoft.PowerShell")
        self.assertEqual(payload["package_version"], "7.6.4.0")

    def test_windows_source_dependency_requires_consent_and_requalifies(
        self,
    ) -> None:
        if sys.platform != "win32":
            self.skipTest("Windows dependency setup qualification requires Windows")
        powershell = shutil.which("powershell.exe")
        self.assertIsNotNone(powershell)
        script = ROOT / "scripts/setup-windows-dependencies.ps1"

        with tempfile.TemporaryDirectory(prefix="hive-dependencies-") as temporary:
            root = Path(temporary)
            marker = root / "winget-invoked"
            fake_pwsh = root / "pwsh.cmd"
            fake_winget = root / "winget.cmd"
            fake_pwsh.write_text("@echo off\necho 7.5.0\n", encoding="ascii")
            fake_winget.write_text(
                "\n".join(
                    (
                        "@echo off",
                        '> "%HIVE_WINGET_MARKER%" echo invoked',
                        '> "%~dp0pwsh.cmd" echo @echo off',
                        '>> "%~dp0pwsh.cmd" echo echo 7.6.4',
                        "exit /b 0",
                        "",
                    )
                ),
                encoding="ascii",
            )
            environment = os.environ.copy()
            environment["PATH"] = str(root)
            environment["HIVE_WINGET_MARKER"] = str(marker)
            base_command = [
                powershell,
                "-NoProfile",
                "-NonInteractive",
                "-File",
                str(script),
            ]

            preview = subprocess.run(
                base_command,
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(preview.returncode, 0, preview.stderr)
            self.assertEqual(
                json.loads(preview.stdout)["status"],
                "install-required",
            )
            self.assertFalse(marker.exists())

            unconfirmed = subprocess.run(
                [*base_command, "-Apply"],
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertNotEqual(unconfirmed.returncode, 0)
            self.assertFalse(marker.exists())

            applied = subprocess.run(
                [*base_command, "-Apply", "-ConfirmInstall"],
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(applied.returncode, 0, applied.stderr)
            payload = json.loads(applied.stdout)
            self.assertEqual(payload["status"], "installed")
            self.assertTrue(payload["changed"])
            self.assertEqual(payload["detected_version"], "7.6.4")
            self.assertTrue(marker.is_file())

    def test_cmd_delegation_preserves_special_prefix_and_child_exit(self) -> None:
        if sys.platform != "win32":
            self.skipTest("cmd.exe delegation qualification requires Windows")
        cmd = shutil.which("cmd.exe")
        self.assertIsNotNone(cmd)
        with tempfile.TemporaryDirectory(prefix="hive cmd ") as temporary:
            root = Path(temporary)
            prefix = root / "prefix % value!"
            result_path = root / "result.txt"
            environment = os.environ.copy()
            environment["HIVE_PREFIX"] = str(prefix)
            environment["HIVE_RESULT"] = str(result_path)
            command = (
                'powershell.exe -NoLogo -NoProfile -NonInteractive '
                '-Command "[IO.File]::WriteAllText('
                "$env:HIVE_RESULT,$env:HIVE_PREFIX,"
                "[Text.UTF8Encoding]::new($false)); exit 23\""
            )
            command_line = f'"{cmd}" /D /V:OFF /C {command}'
            result = subprocess.run(
                command_line,
                cwd=ROOT,
                env=environment,
                check=False,
                text=True,
                capture_output=True,
            )
            self.assertEqual(result.returncode, 23, result.stderr)
            self.assertEqual(result_path.read_text(encoding="utf-8"), str(prefix))

    def test_release_shell_entrypoints_are_executable_and_reject_bad_versions(self) -> None:
        if sys.platform == "win32":
            self.skipTest("POSIX shell entrypoint checks are unavailable on Windows")
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
