"""Native wire contracts in isolated CLI processes, not live-host acceptance."""

from __future__ import annotations

import json
import hashlib
import os
import subprocess
from jsonschema import Draft202012Validator

from tests.conformance.support.harness import Phase1CliTestCase, snapshot_tree, REPOSITORY_ROOT


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
                for path in ("src/application.rs", ".claude/user-owned-note.md"):
                    normal = self.invoke_native(host, self.payload(host, path))
                    if host == "antigravity":
                        self.assertEqual(normal["decision"], "ask")
                        self.assertNotIn("permissionOverrides", normal)
                    else:
                        self.assertEqual(normal, {})
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
            context = self.invoke_native(host, {"invocationNum": 0}, event)
            self.assertTrue(context)
            self.assertLess(len(json.dumps(context)), 1024)
            self.assertNotIn("permissionDecision", json.dumps(context))
        self.assertEqual(self.invoke_native("antigravity", {"invocationNum": 1}, "PreInvocation"), {})
        allowed = self.invoke_native("antigravity", self.payload("antigravity", "ordinary.txt"))
        self.assertEqual(allowed["decision"], "ask")
        self.assertEqual(snapshot_tree(self.target), before)


class NativePolicyConfigurationTests(Phase1CliTestCase):
    def setUp(self):
        super().setUp()
        self.target = self.work_root / "consumer"
        config = self.target / ".hive/config"
        config.mkdir(parents=True)
        (config / "harness.toml").write_text('schema_version = 1\nresolved_owner = "host-native"\n', encoding="utf-8")

    def configure(self, host, action, confirm=None, expected=0):
        args = [str(self.hive_binary), "policy", "hooks", action, "--host", host,
                "--target", str(self.target), "--output", "json"]
        if confirm is not None:
            args += ["--confirm", confirm]
        process = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", timeout=15)
        self.assertEqual(process.returncode, expected, process.stdout + process.stderr)
        result = json.loads(process.stdout)
        self.assertEqual(result["action"], "ConfigurePolicyHooks")
        if "preview" in result.get("data", {}):
            schema = json.loads((REPOSITORY_ROOT / "schemas/host-policy-intent.schema.json").read_text(encoding="utf-8"))
            Draft202012Validator(schema).validate(result["data"]["preview"])
        return result

    def test_three_hosts_preserve_foreign_settings_through_preview_apply_revoke(self):
        paths = {"codex": ".codex/hooks.json", "claude": ".claude/settings.local.json", "antigravity": ".agents/hooks.json"}
        foreign = b'{\r\n  "foreign" : {"values": [1,  2, 3]}\r\n}\n'
        for host, relative in paths.items():
            with self.subTest(host=host):
                path = self.target / relative
                path.parent.mkdir(exist_ok=True)
                path.write_bytes(foreign)
                before = snapshot_tree(self.target)
                preview = self.configure(host, "preview")["data"]["preview"]
                self.assertEqual(snapshot_tree(self.target), before)
                self.configure(host, "apply", preview["approval_digest"])
                installed = snapshot_tree(self.target)
                self.assertEqual(self.configure(host, "apply", preview["approval_digest"])["changed_paths"], [])
                self.assertEqual(snapshot_tree(self.target), installed)
                state = self.configure(host, "status")["data"]
                self.assertTrue(state["configured"])
                self.assertEqual(state["host_loaded"], "unverified")
                self.assertEqual(state["actual_effect"], "unverified")
                removal = self.configure(host, "remove")["data"]["preview"]
                self.configure(host, "remove", removal["approval_digest"])
                self.assertEqual(path.read_bytes(), foreign)
                self.assertFalse(self.configure(host, "status")["data"]["configured"])

    def test_missing_receipt_reports_unknown_configuration_without_inventing_host_effect(self):
        empty = self.configure("codex", "status")["data"]
        self.assertFalse(empty["configured"])
        self.assertFalse(empty["receipt_present"])
        preview = self.configure("codex", "preview")["data"]["preview"]
        self.configure("codex", "apply", preview["approval_digest"])
        (self.target / ".hive/config/host-policy-hooks/codex.json").unlink()
        before = snapshot_tree(self.target)
        unknown = self.configure("codex", "status")["data"]
        self.assertIsNone(unknown["configured"])
        self.assertEqual(unknown["configuration_state"], "unowned-or-receipt-missing")
        self.assertIsNone(unknown["policy_digest"])
        self.assertIsNone(unknown["host_version"])
        for stage in ("host_loaded", "event_matched", "checker_executed", "denial_observed", "actual_effect"):
            self.assertEqual(empty[stage], "unverified")
            self.assertEqual(unknown[stage], "unverified")
        self.assertEqual(before, snapshot_tree(self.target))

    def test_recovery_reuses_verified_claims_and_preserves_a_foreign_racer(self):
        path = self.target / ".codex/hooks.json"
        path.parent.mkdir()
        original = b'{"foreign": "preserve"}\n'
        path.write_bytes(original)
        preview = self.configure("codex", "preview")["data"]["preview"]
        self.configure("codex", "apply", preview["approval_digest"])
        installed = path.read_bytes()
        quarantine = path.parent / (".hive-user-claim-" + hashlib.sha256(b".codex/hooks.json").hexdigest())
        quarantine.mkdir()
        (quarantine / "claimed.bin").write_bytes(original)
        (quarantine / "replacement.bin").write_bytes(installed)
        if os.name != "nt":
            (quarantine / "claimed.bin").chmod(preview["config_mode"])
            (quarantine / "replacement.bin").chmod(preview["config_mode"])
        path.write_bytes(b'{"concurrent": "do not overwrite"}\n')
        before = snapshot_tree(self.target)
        self.configure("codex", "recover", expected=3)
        self.assertEqual(snapshot_tree(self.target), before)
        path.unlink()  # Reproduce a process exit immediately after claiming the old file.
        self.assertTrue(self.configure("codex", "recover")["data"]["configured"])
        self.assertEqual(path.read_bytes(), installed)
        self.assertFalse(quarantine.exists())
        quarantine.mkdir()  # Exit after removing claimed bytes but before directory cleanup.
        self.assertTrue(self.configure("codex", "recover")["data"]["configured"])
        self.assertEqual(path.read_bytes(), installed)
        self.assertFalse(quarantine.exists())

    def test_source_configuration_uses_no_consumer_state_and_removal_survives_owner_change(self):
        preview = self.configure("codex", "preview")["data"]["preview"]
        self.configure("codex", "apply", preview["approval_digest"])
        harness = self.target / ".hive/config/harness.toml"
        harness.write_text('schema_version = 1\nresolved_owner = "omx"\n', encoding="utf-8")
        self.configure("codex", "status")
        removal = self.configure("codex", "remove")["data"]["preview"]
        self.configure("codex", "remove", removal["approval_digest"])
        self.target = self.work_root / "source"
        self.target.mkdir()
        (self.target / "hive-source.json").write_text(json.dumps({"schema_version":1,
            "kind":"aigent-hive-source-workspace","consumer_setup_allowed":False}), encoding="utf-8")
        preview = self.configure("codex", "preview")["data"]["preview"]
        self.configure("codex", "apply", preview["approval_digest"])
        self.assertFalse((self.target / ".hive").exists())
        self.assertTrue((self.target / ".agents/policy-hooks/codex.json").is_file())

    def test_stale_preview_and_tampered_receipt_cannot_authorize_mutation(self):
        preview = self.configure("codex", "preview")["data"]["preview"]
        path = self.target / ".codex/hooks.json"
        path.parent.mkdir()
        path.write_bytes(b'{"foreign":true}\n')
        before = snapshot_tree(self.target)
        self.configure("codex", "apply", preview["approval_digest"], expected=3)
        self.assertEqual(snapshot_tree(self.target), before)
        fresh = self.configure("codex", "preview")["data"]["preview"]
        self.configure("codex", "apply", fresh["approval_digest"])
        receipt = self.target / ".hive/config/host-policy-hooks/codex.json"
        intent = json.loads(receipt.read_text(encoding="utf-8"))
        intent["target_digest"] = "sha256:" + "0" * 64
        receipt.write_text(json.dumps(intent), encoding="utf-8")
        before = snapshot_tree(self.target)
        self.configure("codex", "recover", expected=3)
        self.assertEqual(snapshot_tree(self.target), before)

    def test_approved_intent_recovers_interruption_before_configuration(self):
        preview = self.configure("codex", "preview")["data"]["preview"]
        receipt = self.target / ".hive/config/host-policy-hooks/codex.json"
        receipt.parent.mkdir()
        receipt.write_text(json.dumps(preview), encoding="utf-8")
        self.assertTrue(self.configure("codex", "status")["data"]["pending"])
        recovered = self.configure("codex", "recover")["data"]
        self.assertTrue(recovered["configured"])
        self.assertFalse(recovered["pending"])
        before = snapshot_tree(self.target)
        self.configure("codex", "recover")
        self.assertEqual(snapshot_tree(self.target), before)

    def test_generated_command_executes_the_native_protocol_without_a_model(self):
        preview = self.configure("codex", "preview")["data"]["preview"]
        command = preview["after"][0]["value"]["hooks"][0]["command"]
        payload = {"cwd": str(self.target), "hook_event_name": "PreToolUse", "tool_name": "apply_patch",
                   "tool_input": {"command": "*** Begin Patch\n*** Delete File: .hive/config/harness.toml\n*** End Patch"}}
        process = subprocess.run(command, shell=True, input=json.dumps(payload), capture_output=True,
                                 text=True, encoding="utf-8", timeout=20, cwd=self.target)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(process.stdout)["hookSpecificOutput"]["permissionDecision"], "deny")
