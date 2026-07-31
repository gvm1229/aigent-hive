from __future__ import annotations

import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
PACKAGER = ROOT / "scripts/package-npm.mjs"
NODE = shutil.which("node")
NPM = shutil.which("npm.cmd") or shutil.which("npm")
TARGETS = {
    "aarch64-apple-darwin": ("@aigent-hive/darwin-arm64", "darwin", "arm64", "hive"),
    "x86_64-apple-darwin": ("@aigent-hive/darwin-x64", "darwin", "x64", "hive"),
    "aarch64-unknown-linux-musl": ("@aigent-hive/linux-arm64", "linux", "arm64", "hive"),
    "x86_64-unknown-linux-musl": ("@aigent-hive/linux-x64", "linux", "x64", "hive"),
    "x86_64-pc-windows-msvc": ("@aigent-hive/win32-x64", "win32", "x64", "hive.exe"),
}


def run(*arguments: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        arguments,
        cwd=cwd or ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )


class NpmPackagingContract(unittest.TestCase):
    def setUp(self) -> None:
        if NODE is None or NPM is None:
            self.skipTest("Node.js and npm are unavailable")

    @staticmethod
    def write_installers(root: Path) -> Path:
        installers = root / "installers"
        installers.mkdir()
        for name in ("install.sh", "install.ps1", "install.cmd"):
            (installers / name).write_text(
                f"rendered aigent-hive 0.8.0 {name}\n",
                encoding="utf-8",
            )
        return installers

    def test_platform_and_umbrella_packages_use_exact_native_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            binary = work / "native-binary"
            binary.write_bytes(b"native-hive-test-binary")
            output = work / "packages"
            for target, (name, operating_system, cpu, executable) in TARGETS.items():
                run(
                    "node",
                    str(PACKAGER),
                    "platform",
                    "--product-version",
                    "0.8.0",
                    "--package-version",
                    "0.8.0-test.1",
                    "--output",
                    str(output),
                    "--target",
                    target,
                    "--binary",
                    str(binary),
                )
                directory = output / name.split("/", 1)[1]
                manifest = json.loads((directory / "package.json").read_text("utf-8"))
                self.assertEqual(manifest["name"], name)
                self.assertEqual(manifest["version"], "0.8.0-test.1")
                self.assertEqual(
                    manifest["aigentHive"],
                    {"productVersion": "0.8.0"},
                )
                self.assertEqual(manifest["os"], [operating_system])
                self.assertEqual(manifest["cpu"], [cpu])
                self.assertEqual((directory / "bin" / executable).read_bytes(), binary.read_bytes())
                self.assertNotIn("scripts", manifest)

            run(
                "node",
                str(PACKAGER),
                "umbrella",
                "--product-version",
                "0.8.0",
                "--package-version",
                "0.8.0-test.1",
                "--installer-dir",
                str(self.write_installers(work)),
                "--output",
                str(output),
            )
            umbrella = json.loads((output / "aigent-hive/package.json").read_text("utf-8"))
            self.assertEqual(umbrella["name"], "aigent-hive")
            self.assertEqual(umbrella["bin"], {"hive": "bin/hive.cjs"})
            self.assertEqual(
                umbrella["optionalDependencies"],
                {
                    definition[0]: "0.8.0-test.1"
                    for definition in TARGETS.values()
                },
            )
            self.assertEqual(
                umbrella["aigentHive"],
                {"productVersion": "0.8.0"},
            )
            self.assertNotIn("scripts", umbrella)
            for name in ("install.sh", "install.ps1", "install.cmd"):
                self.assertEqual(
                    (output / "aigent-hive" / name).read_text("utf-8"),
                    f"rendered aigent-hive 0.8.0 {name}\n",
                )

    def test_npm_pack_and_global_install_launch_native_binary(self) -> None:
        node_path = Path(run("node", "-p", "process.execPath").stdout.strip())
        machine = platform.machine().lower()
        current = {
            ("win32", "amd64"): "x86_64-pc-windows-msvc",
            ("win32", "x86_64"): "x86_64-pc-windows-msvc",
            ("darwin", "arm64"): "aarch64-apple-darwin",
            ("darwin", "x86_64"): "x86_64-apple-darwin",
            ("linux", "aarch64"): "aarch64-unknown-linux-musl",
            ("linux", "arm64"): "aarch64-unknown-linux-musl",
            ("linux", "x86_64"): "x86_64-unknown-linux-musl",
        }.get((sys.platform, machine))
        if current is None:
            self.skipTest(f"unsupported npm smoke host: {sys.platform}/{machine}")

        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            packages = work / "packages"
            dist = work / "dist"
            prefix = work / "prefix"
            dist.mkdir()
            run(
                "node",
                str(PACKAGER),
                "platform",
                "--product-version",
                "0.8.0",
                "--package-version",
                "0.8.0-test.1",
                "--output",
                str(packages),
                "--target",
                current,
                "--binary",
                str(node_path),
            )
            run(
                "node",
                str(PACKAGER),
                "umbrella",
                "--product-version",
                "0.8.0",
                "--package-version",
                "0.8.0-test.1",
                "--installer-dir",
                str(self.write_installers(work)),
                "--output",
                str(packages),
            )
            platform_name = TARGETS[current][0].split("/", 1)[1]
            platform_pack = json.loads(
                run(
                    NPM,
                    "pack",
                    "--json",
                    "--pack-destination",
                    str(dist),
                    cwd=packages / platform_name,
                ).stdout
            )[0]["filename"]
            with tarfile.open(dist / platform_pack, mode="r:gz") as archive:
                self.assertEqual(
                    sorted(archive.getnames()),
                    sorted(
                        (
                            f"package/bin/{TARGETS[current][3]}",
                            "package/LICENSE",
                            "package/package.json",
                            "package/README.md",
                        )
                    ),
                )

            umbrella_manifest_path = packages / "aigent-hive/package.json"
            umbrella_manifest = json.loads(umbrella_manifest_path.read_text("utf-8"))
            umbrella_manifest["optionalDependencies"] = {
                TARGETS[current][0]: f"file:{(dist / platform_pack).as_posix()}"
            }
            umbrella_manifest_path.write_text(
                json.dumps(umbrella_manifest, indent=2) + "\n",
                encoding="utf-8",
            )
            umbrella_pack = json.loads(
                run(
                    NPM,
                    "pack",
                    "--json",
                    "--pack-destination",
                    str(dist),
                    cwd=packages / "aigent-hive",
                ).stdout
            )[0]["filename"]
            run(
                NPM,
                "install",
                "--global",
                "--ignore-scripts",
                "--prefix",
                str(prefix),
                str(dist / umbrella_pack),
            )
            command = prefix / ("hive.cmd" if os.name == "nt" else "bin/hive")
            result = run(str(command), "--version")
            self.assertRegex(result.stdout.strip(), r"^v[0-9]+\.[0-9]+\.[0-9]+")


if __name__ == "__main__":
    unittest.main()
