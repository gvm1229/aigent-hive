"""Native wire contracts in isolated CLI processes, not live-host acceptance."""

from __future__ import annotations

import json
import subprocess

from tests.conformance.support.harness import Phase1CliTestCase, snapshot_tree


class NativePolicyProtocolTests(Phase1CliTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.target = self.work_root / "consumer"
        self.target.mkdir()
        protected = self.target / ".hive/config"
        protected.mkdir(parents=True)
        (protected / "harness.toml").write_bytes(b"preserve these exact bytes\n")

    def invoke_native(self, host, payload, event="PreToolUse"):
        process = subprocess.run(
            [str(self.hive_binary), "policy", "hook", "--host", host,
             "--event", event, "--target", str(self.target), "--stdin-json"],
            input=payload if isinstance(payload, str) else json.dumps(payload),
            capture_output=True, text=True, encoding="utf-8", timeout=15,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertNotIn("PRIVATE-SENTINEL", process.stdout + process.stderr)
        return json.loads(process.stdout)

    def payload(self, host, path):
        if host == "codex":
            return {"cwd": str(self.target), "hook_event_name": "PreToolUse",
                    "tool_name": "apply_patch", "tool_input": {"command":
                        f"*** Begin Patch\n*** Delete File: {path}\n*** End Patch"}}
        if host == "claude":
            return {"cwd": str(self.target), "hook_event_name": "PreToolUse",
                    "tool_name": "Write", "tool_input": {
                        "file_path": path, "content": "PRIVATE-SENTINEL"}}
        return {"workspacePaths": [str(self.target)], "toolCall": {
            "name": "write_to_file", "args": {
                "TargetFile": path, "CodeContent": "PRIVATE-SENTINEL"}}}

    def assert_denied(self, host, result):
        if host == "antigravity":
            self.assertEqual(result["decision"], "deny")
        else:
            output = result["hookSpecificOutput"]
            self.assertEqual(output["hookEventName"], "PreToolUse")
            self.assertEqual(output["permissionDecision"], "deny")

    def test_three_protocols_deny_protected_targets_and_leave_normal_permissions_intact(self):
        before = snapshot_tree(self.target)
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                self.assert_denied(host, self.invoke_native(
                    host, self.payload(host, ".hive/config/harness.toml")))
                self.assertEqual(self.invoke_native(
                    host, self.payload(host, "src/application.rs")), {})
        self.assertEqual(snapshot_tree(self.target), before)

    def test_malformed_and_oversized_input_cannot_be_reported_as_allow(self):
        for host in ("codex", "claude", "antigravity"):
            for raw in ("{PRIVATE-SENTINEL", " " * (1024 * 1024 + 1)):
                with self.subTest(host=host, length=len(raw)):
                    self.assert_denied(host, self.invoke_native(host, raw))

    def test_relative_patch_paths_follow_native_working_directory(self):
        child = self.target / "child"
        child.mkdir()
        payload = self.payload("codex", ".hive/config/harness.toml")
        payload["cwd"] = str(child)
        # A child-local .hive is outside the root-owned state namespace.
        self.assertEqual(self.invoke_native("codex", payload), {})
        payload["tool_input"]["command"] = (
            "*** Begin Patch\n*** Delete File: ../.hive/config/harness.toml\n*** End Patch"
        )
        self.assert_denied("codex", self.invoke_native("codex", payload))

    def test_stop_is_neutral_and_startup_only_delivers_bounded_context(self):
        before = snapshot_tree(self.target)
        for host in ("codex", "claude", "antigravity"):
            expected_stop = {"decision": "allow"} if host == "antigravity" else {}
            self.assertEqual(self.invoke_native(host, "not JSON", "Stop"), expected_stop)
            event = "PreInvocation" if host == "antigravity" else "SessionStart"
            context = self.invoke_native(host, {}, event)
            self.assertLess(len(json.dumps(context)), 1024)
            self.assertNotIn("permissionDecision", json.dumps(context))
        self.assertEqual(snapshot_tree(self.target), before)
