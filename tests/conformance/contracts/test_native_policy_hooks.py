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
                for protected in (".hive/config/harness.toml", ".hive/LICENSE-AIGENT-HIVE.txt", ".hive/README.md", ".hive/index/hive.sqlite3",
                                  ".hive/backups/restore.json", ".hive/runtime/usage-guard/halt.json",
                                  ".hive/language-packs/policy.json", ".hive/directives/00-editing-discipline.md"):
                    self.assert_denied(host, self.invoke_native(host, self.payload(host, protected)))
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

    def configure(self, host, action, confirm=None, expected=0, review_run=None):
        args = [str(self.hive_binary), "policy", "hooks", action, "--host", host,
                "--target", str(self.target), "--output", "json"]
        if confirm is not None:
            args += ["--confirm", confirm]
        if review_run is not None:
            args += ["--review-run", review_run]
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

    def test_generated_command_denies_checker_startup_and_process_failure(self):
        before = snapshot_tree(self.target)
        for host in ("codex", "claude", "antigravity"):
            with self.subTest(host=host):
                preview = self.configure(host, "preview")["data"]["preview"]
                command = preview["after"][0]["value"]["hooks"][0]["command"]
                self.assertIn(str(self.hive_binary), command)
                def quote(value):
                    return "'" + value.replace("'", "''" if os.name == "nt" else "'\\''") + "'"

                executable = quote(str(self.hive_binary))
                python = quote(os.sys.executable)
                failures = (
                    command.replace(str(self.hive_binary), str(self.work_root / "missing checker.exe"), 1),
                    command.replace(f"'--host' '{host}'", "'--host' 'unsupported-fixture'", 1),
                    command.replace(executable, f"{python} '-c' 'pass'", 1),
                    command.replace(executable, f"{python} '-c' 'import sys;print(sys.stdin.read());sys.exit(3)'", 1),
                )
                for failing_command in failures:
                    self.assertNotEqual(failing_command, command)
                    process = subprocess.run(failing_command, shell=True, input="PRIVATE-SENTINEL",
                        capture_output=True, text=True, encoding="utf-8", timeout=20, cwd=self.target)
                    self.assertEqual(process.returncode, 0, process.stderr)
                    self.assertEqual(process.stderr, "")
                    response = json.loads(process.stdout)
                    if host == "antigravity":
                        self.assertEqual(response["decision"], "deny")
                    else:
                        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "deny")
                    self.assertIn("repair the registered checker", process.stdout)
                    self.assertNotIn("PRIVATE-SENTINEL", process.stdout)
                    self.assertEqual(snapshot_tree(self.target), before)

    def test_generated_command_executes_the_native_protocol_without_a_model(self):
        before = snapshot_tree(self.target)
        for host in ("codex", "claude", "antigravity"):
            preview = self.configure(host, "preview")["data"]["preview"]
            command = preview["after"][0]["value"]["hooks"][0]["command"]
            for path in ("ordinary.txt", ".hive/config/harness.toml"):
                payload = NativePolicyProtocolTests.payload(self, host, path)
                process = subprocess.run(command, shell=True, input=json.dumps(payload), capture_output=True,
                                         text=True, encoding="utf-8", timeout=20, cwd=self.target)
                self.assertEqual(process.returncode, 0, process.stderr)
                response = json.loads(process.stdout)
                if path == "ordinary.txt":
                    if host == "antigravity":
                        self.assertEqual(response["decision"], "ask")
                        self.assertNotIn("permissionOverrides", response)
                    else:
                        self.assertEqual(response, {})
                else:
                    NativePolicyProtocolTests.assert_denied(self, host, response)
        self.assertEqual(snapshot_tree(self.target), before)

    def test_explicit_run_notice_binds_host_session_and_never_requests_continuation(self):
        base = self.target
        digest = lambda value: "sha256:" + hashlib.sha256(value).hexdigest()
        for host in ("codex", "claude", "antigravity"):
            self.target = base / host
            config = self.target / ".hive/config"
            config.mkdir(parents=True)
            (config / "harness.toml").write_text('schema_version = 1\nresolved_owner = "host-native"\n', encoding="utf-8")
            run = self.target / ".hive/runs/review-1"
            (run / "evidence").mkdir(parents=True)
            (run / "PLAN.md").write_text("# Plan\n\n- [ ] [build] Build succeeds\n", encoding="utf-8")
            state = {"schema_version":1,"run_id":"review-1","revision":1,"state":"executing",
                "required_criteria":["build"],"passed_criteria":[],"failed_criteria":[],"active_roles":[],
                "next_action":"verify","latest_evidence":[],"blocker":None,"updated_at":"2026-09-19T00:00:00Z",
                "host":host,"host_version":"fixture","surface":"cli","external_runtime":None,
                "resolved_owner":"host-native","resolution_evidence_digest":digest(b"capability"),
                "subagent_support":"supported","criterion_evidence":{},
                "continuation":{"session_binding_digest":digest(b"fixture-session"),"max_retry_attempts":3,
                                "attempts_used":0,"cancel_requested":False}}
            def write_status():
                (run / "STATUS.md").write_text("---\n"+json.dumps(state)+"\n---\n# Status\n", encoding="utf-8")
            def review(action, *extra):
                process = subprocess.run([str(self.hive_binary), "run", "policy-review", action,
                    "--target",str(self.target),"--run","review-1","--output","json",*extra],
                    capture_output=True,text=True,encoding="utf-8",timeout=15)
                self.assertEqual(process.returncode,0,process.stdout+process.stderr)
                return json.loads(process.stdout)["data"]
            write_status()
            target_digest = review("list")["target_digest"]
            binding = {"operation_id":"review-1","policy_digest":digest(b"policy"),"target_digest":target_digest}
            evaluation = {"schema_version":1,"binding":binding,
                "requirements":[{"rule_id":"exact-authority","mandatory":True}],
                "results":[{"rule_id":"exact-authority","binding":binding,"decision":"deny","code":"hive.denied"}]}
            evaluation_bytes = json.dumps(evaluation).encode()
            (run / "evidence/policy.json").write_bytes(evaluation_bytes)
            state["latest_evidence"] = [".hive/runs/review-1/evidence/policy.json#"+digest(evaluation_bytes)]
            write_status()
            args = ("--evaluation","evidence/policy.json","--rule","exact-authority","--class","non-compliance")
            preview = review("preview", *args)
            added = review("add", *args, "--confirm", preview["preview_digest"])
            before = snapshot_tree(self.target)
            if host == "antigravity":
                refusal = self.configure(host,"preview",expected=3,review_run="review-1")
                self.assertIn("non-continuing",refusal["message"])
                self.assertEqual(snapshot_tree(self.target),before)
                continue
            ordinary = self.configure(host,"preview")["data"]["preview"]
            self.assertNotIn("review_run",ordinary)
            self.configure(host,"apply",ordinary["approval_digest"])
            hooks = self.configure(host,"preview",review_run="review-1")["data"]["preview"]
            self.configure(host,"apply",hooks["approval_digest"],review_run="review-1")
            self.assertEqual(self.configure(host,"status")["data"]["review_run"],"review-1")
            command = next(entry for entry in hooks["after"] if entry["path"][-1]=="Stop")["value"]["hooks"][0]["command"]
            def notice(session):
                process = subprocess.run(command,shell=True,input=json.dumps({"hook_event_name":"Stop",
                    "session_id":session,"transcript_path":"PRIVATE-SENTINEL"}),capture_output=True,
                    text=True,encoding="utf-8",timeout=20,cwd=self.target)
                self.assertEqual(process.returncode,0,process.stderr)
                self.assertNotIn("PRIVATE-SENTINEL",process.stdout+process.stderr)
                return json.loads(process.stdout)
            unchanged = snapshot_tree(self.target)
            self.assertEqual(notice("other-session"),{})
            for _ in range(2):
                output = notice("fixture-session")
                self.assertEqual(set(output),{"systemMessage"})
                self.assertIn("1 policy review",output["systemMessage"])
            self.assertEqual(snapshot_tree(self.target),unchanged)
            review("reject","--candidate",added["book"]["candidates"][0]["id"],"--confirm",added["book_digest"])
            self.assertEqual(notice("fixture-session"),{})
            removal = self.configure(host,"remove")["data"]["preview"]
            self.configure(host,"remove",removal["approval_digest"])
            self.assertFalse(self.configure(host,"status")["data"]["configured"])
