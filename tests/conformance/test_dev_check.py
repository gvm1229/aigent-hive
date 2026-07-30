from __future__ import annotations

import importlib.util
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/dev-check.py"
SPEC = importlib.util.spec_from_file_location("dev_check", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class DevCheckTest(unittest.TestCase):
    def test_conformance_requirements_keep_exact_pins(self) -> None:
        self.assertEqual(
            (ROOT / "requirements-conformance.txt")
            .read_text(encoding="utf-8")
            .splitlines(),
            [
                "copier==9.17.0",
                "jsonschema[format]==4.25.1",
                "PyYAML==6.0.2",
            ],
        )

    def test_ci_install_steps_consume_the_shared_requirements(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertEqual(
            workflow.count("-r requirements-conformance.txt"),
            2,
        )
        for pin in (
            "copier==9.17.0",
            "jsonschema[format]==4.25.1",
            "PyYAML==6.0.2",
        ):
            self.assertNotIn(pin, workflow)

    def test_rust_tools_fall_back_to_stable_rustup_toolchain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            toolchain = home / ".rustup/toolchains/stable-test/bin"
            toolchain.mkdir(parents=True)
            cargo = toolchain / MODULE.executable_name("cargo")
            rustc = toolchain / MODULE.executable_name("rustc")
            cargo.touch()
            rustc.touch()
            with (
                mock.patch.object(MODULE.shutil, "which", return_value=None),
                mock.patch.object(MODULE.Path, "home", return_value=home),
            ):
                self.assertEqual(
                    MODULE.resolve_tool("cargo", rust_tool=True), cargo
                )
                self.assertEqual(
                    MODULE.resolve_tool("rustc", rust_tool=True), rustc
                )

    def test_tool_environment_does_not_mutate_caller_environment(self) -> None:
        original = os.environ.get("PATH")
        tool = Path("/tools/cargo")
        environment = MODULE.tool_environment(tool)
        self.assertEqual(os.environ.get("PATH"), original)
        self.assertEqual(
            environment["PATH"].split(os.pathsep)[0],
            str(tool.parent),
        )

    def test_python_mode_uses_uv_ephemeral_requirements_and_passthrough(self) -> None:
        commands: list[list[str]] = []
        environments: list[dict[str, str]] = []
        uv = Path("/tools/uv")
        with (
            mock.patch.object(
                MODULE, "resolve_tool", return_value=uv
            ),
            mock.patch.object(
                MODULE,
                "run",
                side_effect=lambda command, **options: (
                    commands.append(list(command)),
                    environments.append(dict(options["environment"])),
                ),
            ),
        ):
            MODULE.run_python(["tests.conformance.test_dev_check", "-v"])

        self.assertEqual(commands[0][0:7], [
            str(uv),
            "run",
            "--python",
            "3.13",
            "--isolated",
            "--no-project",
            "--with-requirements",
        ])
        self.assertEqual(
            commands[0][7:],
            [
                str(ROOT / "requirements-conformance.txt"),
                "python",
                "-m",
                "unittest",
                "tests.conformance.test_dev_check",
                "-v",
            ],
        )
        if os.name == "nt":
            self.assertEqual(
                environments[0]["HIVE_WINDOWS_SOURCE_USAGE_GUARD_SUBSET"],
                "1",
            )

    def test_main_supports_targeted_rust_and_pre_push_modes(self) -> None:
        with (
            mock.patch.object(MODULE, "run_rust") as run_rust,
            mock.patch.object(MODULE, "run_python") as run_python,
        ):
            self.assertEqual(
                MODULE.main(["rust", "--", "test", "-p", "hive-core"]),
                0,
            )
            run_rust.assert_called_once_with(["test", "-p", "hive-core"])
            run_python.assert_not_called()

            run_rust.reset_mock()
            self.assertEqual(MODULE.main(["pre-push"]), 0)
            run_rust.assert_called_once_with(())
            run_python.assert_called_once_with(())

    def test_missing_tool_error_is_actionable(self) -> None:
        with (
            mock.patch.object(MODULE.shutil, "which", return_value=None),
            mock.patch.object(
                MODULE.Path, "home", return_value=Path("/missing-home")
            ),
        ):
            with self.assertRaisesRegex(
                MODULE.DevCheckError,
                r"Required tool 'uv' was not found.*Install uv.*PATH",
            ):
                MODULE.resolve_tool("uv")


if __name__ == "__main__":
    unittest.main()
