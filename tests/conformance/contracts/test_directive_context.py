"""Approved context wire replay; this does not simulate model adherence or real compaction."""

import hashlib
import json
import os
import subprocess

from tests.conformance.support.harness import Phase1CliTestCase, snapshot_tree


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


class DirectiveContextTests(Phase1CliTestCase):
    def setUp(self):
        super().setUp()
        self.target = self.work_root / "context-project"
        config = self.target / ".hive/config"
        config.mkdir(parents=True)
        (config / "harness.toml").write_bytes(b'schema_version = 1\nresolved_owner = "host-native"\n')
        self.rules = self.target / "AGENTS.md"
        self.rules.write_bytes(b"Reply in Korean.\nPreserve test behavior when editing tests.\n" + b"Unrelated reference.\n" * 500)
        self.spec = self.target / ".hive-context.toml"
        self.write_spec()

    def write_spec(self, source="AGENTS.md", first=1, last=1, core=b"Reply in Korean.\n"):
        text = f'''schema_version = 1
protected_paths = ["locked"]
[[rules]]
id = "language"
source = {json.dumps(source)}
first_line = {first}
last_line = {last}
digest = "{digest(core)}"
on = "restore"
[[rules]]
id = "tests"
source = "AGENTS.md"
first_line = 2
last_line = 2
digest = "{digest(b'Preserve test behavior when editing tests.' + bytes([10]))}"
on = "edit"
paths = ["tests"]
'''
        self.spec.write_bytes(text.encode())

    def configure(self, action="preview", confirm=None, expected=0, host="codex"):
        args = [str(self.hive_binary), "policy", "hooks", action, "--host", host,
                "--target", str(self.target), "--output", "json"]
        if confirm:
            args += ["--confirm", confirm]
        result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", timeout=20)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return json.loads(result.stdout)

    def invoke(self, event="SessionStart", payload=None, pinned=None, host="codex"):
        args = [str(self.hive_binary), "policy", "hook", "--host", host,
                "--event", event, "--target", str(self.target), "--stdin-json",
                "--expected-context", pinned or digest(self.spec.read_bytes())]
        payload = payload or {"hook_event_name": event, "source": "compact", "session_id": "shared-parent", "turn_id": "same-turn"}
        payload.setdefault("cwd", str(self.target))
        result = subprocess.run(args, input=json.dumps(payload),
            capture_output=True, text=True, encoding="utf-8", timeout=20)
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def edit(self, path, pinned=None):
        return self.invoke("PreToolUse", {"hook_event_name": "PreToolUse", "cwd": str(self.target),
            "tool_name": "apply_patch", "tool_input": {"command":
                f"*** Begin Patch\n*** Delete File: {path}\n*** End Patch"}}, pinned)

    def test_compactions_never_deduplicate_unchanged_policy_or_shared_parent(self):
        before = snapshot_tree(self.target)
        for count in (0, 1, 3, 5, 10):
            for _ in range(count):
                output = self.invoke()["hookSpecificOutput"]["additionalContext"]
                self.assertIn("Reply in Korean.", output)
                self.assertNotIn("Unrelated reference", output)
                self.assertNotIn("Preserve test behavior", output)
                self.assertLessEqual(len(output.encode()), 4096)
                self.assertLess(len(output.encode()), len(self.rules.read_bytes()) / 2)
        for source in ("startup", "resume", "compact", "clear"):
            output = self.invoke(payload={"hook_event_name": "SessionStart", "source": source})
            self.assertIn("Reply in Korean.", output["hookSpecificOutput"]["additionalContext"])
        self.assertEqual(snapshot_tree(self.target), before)

    def test_only_matching_paths_receive_detail_and_never_grant_permission(self):
        self.assertEqual(self.edit("src/app.rs"), {})
        self.assertEqual(self.edit("tests-other/test.py"), {})
        output = self.edit("tests/test.py")["hookSpecificOutput"]
        self.assertIn("Preserve test behavior", output["additionalContext"])
        self.assertNotIn("Reply in Korean", output["additionalContext"])
        self.assertNotIn("permissionDecision", output)
        self.assertEqual(self.edit("locked/file.txt")["hookSpecificOutput"]["permissionDecision"], "deny")
        self.assertEqual(self.edit(".hive/config/harness.toml")["hookSpecificOutput"]["permissionDecision"], "deny")

    def test_changed_source_is_not_promoted_and_does_not_block_unrelated_edits(self):
        self.rules.write_bytes(b"UNAPPROVED-SECRET\nPreserve test behavior when editing tests.\n")
        output = self.invoke()["hookSpecificOutput"]["additionalContext"]
        self.assertNotIn("UNAPPROVED-SECRET", output)
        self.assertIn("unavailable or changed", output)
        self.assertEqual(self.edit("src/app.rs"), {})
        self.assertEqual(self.edit("locked/a")["hookSpecificOutput"]["permissionDecision"], "deny")
        self.configure(expected=3)

    def test_changed_configuration_fails_closed_but_stop_and_remove_stay_available(self):
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        pinned = digest(self.spec.read_bytes())
        self.spec.write_bytes(b"invalid")
        self.assertEqual(self.edit("src/app.rs", pinned)["hookSpecificOutput"]["permissionDecision"], "deny")
        self.assertEqual(self.invoke("Stop", pinned=pinned), {})
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertFalse((self.target / ".codex/hooks.json").exists())

    def test_preview_pins_content_and_preserves_foreign_config_through_update_and_removal(self):
        directory = self.target / ".codex"
        directory.mkdir()
        config = directory / "hooks.json"
        original = b'{ "foreign": { "keep": 17 } }\n'
        config.write_bytes(original)
        preview = self.configure()["data"]
        self.assertEqual(preview["directive_context"]["model_calls"], 0)
        self.assertEqual(preview["preview"]["context_digest"], digest(self.spec.read_bytes()))
        self.assertFalse(preview["authorizes_model_execution"])
        self.configure("apply", preview["preview"]["approval_digest"])
        self.spec.write_bytes(self.spec.read_bytes() + b"\n# reviewed change\n")
        self.configure("apply", preview["preview"]["approval_digest"], expected=3)
        updated = self.configure()["data"]["preview"]
        self.configure("apply", updated["approval_digest"])
        self.assertEqual(json.loads(config.read_bytes())["foreign"], {"keep": 17})
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertEqual(config.read_bytes(), original)

    def test_context_hooks_can_be_reapplied_after_removal(self):
        directory = self.target / ".codex"
        directory.mkdir()
        config = directory / "hooks.json"
        foreign = b'{ "foreign": { "keep": 17 } }\n'
        config.write_bytes(foreign)
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertEqual(config.read_bytes(), foreign)
        self.configure("status")
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        self.assertEqual(json.loads(config.read_bytes())["foreign"], {"keep": 17})
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertEqual(config.read_bytes(), foreign)

    def test_context_disable_updates_keep_removal_and_reapply_available(self):
        directory = self.target / ".codex"
        directory.mkdir()
        config = directory / "hooks.json"
        foreign = b'{ "foreign": { "keep": 17 } }\n'
        config.write_bytes(foreign)
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        self.spec.unlink()
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        self.configure("status")
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertEqual(config.read_bytes(), foreign)
        self.configure("status")
        self.write_spec()
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        removal = self.configure("remove")["data"]["preview"]
        self.configure("remove", removal["approval_digest"])
        self.assertEqual(config.read_bytes(), foreign)

    def test_historical_container_allowance_rejects_other_namespaces(self):
        preview = self.configure()["data"]["preview"]
        self.configure("apply", preview["approval_digest"])
        receipt = self.target / ".hive/config/host-policy-hooks/codex.json"
        original = receipt.read_bytes()
        for path in (["foreign", "SubagentStart"], ["hooks", "SubagentStart", "foreign"],
                     ["hooks", "OtherEvent"]):
            intent = json.loads(original)
            intent["created_containers"][-1] = path
            receipt.write_text(json.dumps(intent), encoding="utf-8")
            before = snapshot_tree(self.target)
            result = self.configure("status", expected=3)
            expected_message = "schema" if len(path) > 2 else "container ownership"
            self.assertIn(expected_message, result["message"])
            self.assertEqual(snapshot_tree(self.target), before)
        receipt.write_bytes(original)

    def test_missing_context_invalid_paths_ranges_and_budget_are_not_approved(self):
        pinned = digest(self.spec.read_bytes())
        self.spec.unlink()
        self.assertIn("unavailable", self.invoke(pinned=pinned)["hookSpecificOutput"]["additionalContext"])
        for source in ("../escape.md", "C:/secret.md", ".git/secret.md", ".hive/runtime/transcript.md"):
            self.write_spec(source=source)
            self.configure(expected=3)
        self.write_spec(last=900)
        self.configure(expected=3)
        large = b"x" * 4096 + b"\n"
        self.rules.write_bytes(large + b"Preserve test behavior when editing tests.\n")
        self.write_spec(core=large)
        self.configure(expected=3)

    def test_unapproved_spec_is_inert_and_antigravity_is_not_claimed_as_supported(self):
        result = subprocess.run([str(self.hive_binary), "policy", "hook", "--host", "codex",
            "--event", "SessionStart", "--target", str(self.target), "--stdin-json"],
            input='{"hook_event_name":"SessionStart","source":"compact"}', capture_output=True, text=True)
        self.assertNotIn("Reply in Korean.", result.stdout)
        preview = self.configure(host="antigravity")["data"]
        self.assertIsNone(preview["directive_context"])
        self.assertEqual(preview["context_recovery"], "unsupported")

    def test_existing_hooks_gain_context_only_after_a_new_exact_preview(self):
        spec = self.spec.read_bytes()
        self.spec.unlink()
        old = self.configure()["data"]["preview"]
        self.configure("apply", old["approval_digest"])
        self.assertNotIn("context_digest", old)
        self.spec.write_bytes(spec)
        self.configure("apply", old["approval_digest"], expected=3)
        preview = self.configure()["data"]
        self.configure("apply", preview["preview"]["approval_digest"])
        command = next(item["value"]["hooks"][0]["command"]
                       for item in preview["preview"]["after"] if item["path"][-1] == "SessionStart")
        result = subprocess.run(command, shell=True,
            input=json.dumps({"hook_event_name":"SessionStart","source":"compact","cwd":str(self.target)}),
            capture_output=True, text=True, encoding="utf-8", timeout=20)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Reply in Korean.", json.loads(result.stdout)["hookSpecificOutput"]["additionalContext"])
        self.assertTrue(self.configure("status")["data"]["context_current"])

    def test_renames_and_windows_case_cannot_escape_custom_protection(self):
        payload = {"hook_event_name": "PreToolUse", "cwd": str(self.target),
            "tool_name": "apply_patch", "tool_input": {"command":
                "*** Begin Patch\n*** Update File: src/a\n*** Move to: locked/a\n@@\n-a\n+b\n*** End Patch"}}
        self.assertEqual(self.invoke("PreToolUse", payload)["hookSpecificOutput"]["permissionDecision"], "deny")
        if os.name == "nt":
            self.assertEqual(self.edit("locked./a")["hookSpecificOutput"]["permissionDecision"], "deny")
            self.assertEqual(self.edit("LOCKED/a")["hookSpecificOutput"]["permissionDecision"], "deny")

    def test_symlink_source_is_never_promoted(self):
        original = self.rules.read_bytes()
        outside = self.work_root / "outside.md"
        outside.write_bytes(original)
        self.rules.unlink()
        try:
            self.rules.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"host cannot create symlink: {type(error).__name__}")
        self.configure(expected=3)
        self.assertNotIn("Reply in Korean.", self.invoke()["hookSpecificOutput"]["additionalContext"])

    def test_child_context_restores_without_parent_dedup_and_outside_cwd_is_inert(self):
        for child in ("one", "two", "one"):
            output = self.invoke("SubagentStart", {"hook_event_name":"SubagentStart", "session_id":"shared-parent", "agent_id":child})["hookSpecificOutput"]
            self.assertEqual(output["hookEventName"], "SubagentStart")
            self.assertIn("Reply in Korean.", output["additionalContext"])
        self.assertEqual(self.invoke(payload={"hook_event_name":"SessionStart", "source":"compact", "cwd":str(self.work_root)}), {})
        preview = self.configure()["data"]["preview"]
        self.assertEqual(len(preview["after"]), 4)
        for entry in preview["after"]:
            handler = entry["value"]["hooks"][0]
            if entry["path"][-1] == "Stop": self.assertNotIn("additionalContextLimit", handler)
            else: self.assertEqual(handler["additionalContextLimit"], 0)

    def test_changed_policy_has_an_explicit_recovery_notice(self):
        result = subprocess.run([str(self.hive_binary), "policy", "hook", "--host", "codex",
            "--event", "SessionStart", "--target", str(self.target), "--stdin-json",
            "--expected-policy", "sha256:" + "0" * 64], input="{}", capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("policy changed", json.loads(result.stdout)["hookSpecificOutput"]["additionalContext"])
