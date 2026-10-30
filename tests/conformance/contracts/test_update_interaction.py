"""Update input contracts reject missing consent before any installation work."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess

from jsonschema import Draft202012Validator
from tests.conformance.support.harness import ACTION_RESULT_SCHEMA, Phase1CliTestCase, REPOSITORY_ROOT, read_yaml, snapshot_tree


class UpdateInteractionConformance(Phase1CliTestCase):
    def invoke(self, *flags: str, machine: bool = True) -> tuple[subprocess.CompletedProcess, dict | None]:
        root = self.work_root / "untouched-user"
        root.mkdir(exist_ok=True)
        process = subprocess.run(
            [str(self.hive_binary), "update", *flags, "--user-root", str(root),
             *(["--output", "json"] if machine else [])],
            input="yes\nyes\nyes\n", capture_output=True, text=True, encoding="utf-8",
            timeout=10, env={**os.environ, "PATH": str(self.work_root / "no-external-tools")},
        )
        self.assertEqual(list(root.iterdir()), [])
        if machine:
            result = json.loads(process.stdout)
            Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
            self.assertEqual(process.returncode, result["exit_code"])
            self.assertEqual(result["changed_paths"], [])
            return process, result
        self.assertEqual(process.stdout, "")
        return process, None

    def test_json_and_noninteractive_do_not_take_pipe_input_as_update_consent(self) -> None:
        for flags in [(), ("--non-interactive",)]:
            with self.subTest(flags=flags):
                _, result = self.invoke(*flags)
                self.assertEqual(result["code"], "hive.update-needs-input")
                self.assertEqual(result["status"], "needs-input")

    def test_explicit_terminal_without_a_terminal_never_applies(self) -> None:
        process, _ = self.invoke("--confirm", "--interactive", machine=False)
        self.assertEqual(process.returncode, 2)
        self.assertIn("usable terminal", process.stderr)

    def test_conflicting_and_duplicate_options_fail_before_mutation(self) -> None:
        for flags in [
            ("--interactive",),
            ("--confirm", "--interactive", "--non-interactive"),
            ("--non-interactive", "--non-interactive"),
            ("--output", "json"),
        ]:
            with self.subTest(flags=flags):
                _, result = self.invoke(*flags)
                self.assertEqual(result["status"], "error")
                self.assertEqual(result["exit_code"], 2)

    def test_input_redirection_alone_does_not_confirm_an_update(self) -> None:
        process, _ = self.invoke(machine=False)
        self.assertEqual(process.returncode, 2)
        self.assertIn("approval is required", process.stderr)


class ApprovedUpdateResumeConformance(Phase1CliTestCase):
    def test_confirmed_setup_resumes_before_fresh_software_approval_or_package_discovery(self):
        config = self.setup_user_root / ".hive/config/user-setup.yml"
        self.assertTrue(config.resolve().is_relative_to(self.work_root))
        config.unlink()  # Discard only the synthetic baseline created by setUp.
        commands = self.work_root / "commands"
        commands.mkdir()
        fixture = REPOSITORY_ROOT / "tests/fixtures/native-orchestration/agy-stub.py"
        wrapper = '''import json,os,runpy,sys
from pathlib import Path
scope=Path(os.environ['HIVE_TEST_SANDBOX_ROOT']).resolve()
def within(path):
    resolved=path.resolve()
    return any(parent.exists() and parent.samefile(scope) for parent in (resolved,*resolved.parents))
state=Path(os.environ['HIVE_TEST_HOST_LOG']+'.antigravity.state')
assert within(state)
if sys.argv[1:3]==['plugin','install']:
    source=Path(sys.argv[3]).resolve()
    assert within(source)
    hive=next(p for p in source.parents if p.name=='.hive')
    stage=hive.parent/'.gemini/config/plugins/aigent-hive'
    assert within(stage) and not stage.is_symlink()
if sys.argv[1:]==['plugin','uninstall','aigent-hive'] and state.exists():
    saved=json.loads(state.read_text()).get('stage')
    if saved:
        stage=Path(saved)
        assert within(stage) and not stage.is_symlink()
'''+f"runpy.run_path({str(fixture)!r},run_name='__main__')\n"
        if os.name == "nt":
            (commands / "agy.py").write_text(wrapper, encoding="utf-8")
            python = subprocess.list2cmdline([os.sys.executable])
            (commands / "agy.cmd").write_text(f'@{python} "%~dp0\\agy.py" %*\r\n', encoding="utf-8")
        else:
            program = commands / "agy"
            program.write_text(f"#!{os.sys.executable}\n"+wrapper, encoding="utf-8")
            program.chmod(0o700)
        environment = {**os.environ, "PATH": str(commands),
            "HIVE_TEST_HOST_LOG": str(self.work_root / "host-actions"),
            "HIVE_TEST_SANDBOX_ROOT": str(self.work_root)}
        document = self.work_root / "answers.json"
        document.write_text(json.dumps({"schema_version":2,"interface_language":"en",
            "wiki":{"enabled":False,"language":"en","backend":"markdown"},
            "profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},
            "selected_hosts":["antigravity"],"skills":{"mode":"all"},"update_check":{"enabled":False},
            "judge_invocation":"explicit","preference_decisions":{},
            "usage_guard":{"enforcement_enabled":False,"warnings_enabled":False,"stop_remaining_percent":20,
                "preflight":{"threshold_enabled":False,"reset_enabled":False,"zero_quota_enabled":False,"observation_failure":"warn"},
                "midway":{"enabled":False,"threshold_enabled":False,"reset_enabled":False,"zero_quota_enabled":False,"observation_failure":"warn"}}}),encoding="utf-8")
        foreign = self.setup_user_root / ".gemini/GEMINI.md"
        foreign.parent.mkdir()
        foreign.write_text("User-owned preface.\n", encoding="utf-8")
        foreign_before = foreign.read_bytes()
        preview_answers = self.work_root / "preview.json"
        draft = json.loads(document.read_text())
        draft["wiki"]["enabled"] = True
        preview_answers.write_text(json.dumps(draft), encoding="utf-8")
        preview_before = snapshot_tree(self.setup_user_root)
        self.assertFalse((self.setup_user_root / ".hive/index").exists())
        preview = subprocess.run([str(self.hive_binary),"setup","--scope","user","--user-root",str(self.setup_user_root),
            "--answers",str(preview_answers),"--dry-run","--output","json"],
            env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(preview.returncode,0,(preview.stdout,preview.stderr))
        self.assertEqual(json.loads(preview.stdout)["code"],"hive.user-setup-dry-run-complete")
        self.assertEqual(snapshot_tree(self.setup_user_root),preview_before)
        interrupted = subprocess.run([str(self.hive_binary),"setup","--scope","user","--user-root",str(self.setup_user_root),
            "--answers",str(document),"--apply","--output","json"],
            env={**environment,"HIVE_USER_SETUP_INTERRUPT_AFTER":"0"},capture_output=True,text=True,timeout=30)
        self.assertEqual(interrupted.returncode,87,(interrupted.stdout,interrupted.stderr))
        progress_path = self.setup_user_root / ".hive/config/setup-progress/user.yml"
        progress = read_yaml(progress_path)
        self.assertEqual(progress["phase"], "applying")
        document.unlink()
        command = [str(self.hive_binary),"update","--user-root",str(self.setup_user_root),"--output","json"]
        resumed = subprocess.run(command,env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(resumed.returncode,0,(resumed.stdout,resumed.stderr))
        result=json.loads(resumed.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
        self.assertEqual(result["action"],"UpdateHiveUser")
        self.assertEqual(result["code"],"hive.user-update-complete")
        self.assertTrue(result["data"]["recovered"])
        self.assertEqual(result["data"]["steps"], {"core":"not-run","settings":"verified","harness":"verified","monitor":"inspected"})
        self.assertFalse(result["data"]["effective_usage_policy"]["preflight"]["zero_quota_enabled"])
        self.assertEqual(result["data"]["monitor_state"]["controller_state"],"not-configured")
        self.assertLessEqual(len(resumed.stdout.encode("utf-8")),8192)
        completed = read_yaml(progress_path)
        self.assertEqual(completed["phase"], "complete")
        self.assertEqual(completed["operation_id"], progress["operation_id"])
        self.assertTrue(foreign.read_text().startswith("User-owned preface.\n"))
        saved = subprocess.run([str(self.hive_binary),"__complete-user-update","--user-root",str(self.setup_user_root),
            "--interaction","non-interactive"],env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(saved.returncode,0,(saved.stdout,saved.stderr))
        saved_result=json.loads(saved.stdout)
        self.assertEqual(saved_result["data"]["settings_changes"]["changed_keys"],[])
        self.assertEqual(saved_result["data"]["pending_questions"],[])
        self.assertFalse(saved_result["data"]["effective_usage_policy"]["authorizes_dispatch"])
        # The source debug binary has no install-owner receipt. Success therefore came from
        # pending setup recovery before package discovery, not from installation/reconciliation.
        before=snapshot_tree(self.setup_user_root)
        repeated=subprocess.run(command,env=environment,input="",capture_output=True,text=True,timeout=10)
        self.assertEqual(repeated.returncode,2,(repeated.stdout,repeated.stderr))
        self.assertEqual(json.loads(repeated.stdout)["code"],"hive.update-needs-input")
        self.assertEqual(snapshot_tree(self.setup_user_root),before)
        settings_path = self.setup_user_root / ".hive/config/user-setup.yml"
        old_choices = read_yaml(settings_path)["preference_decisions"]
        document.write_text(json.dumps({"schema_version":1,"kind":"hive.setup-answer-patch",
            "answers":{"persona":{"id":"friendly"}}}),encoding="utf-8")
        patched = subprocess.run([str(self.hive_binary),"setup","--scope","user","--user-root",str(self.setup_user_root),
            "--answers",str(document),"--input-channel","agent","--apply","--summary","--output","json"],
            env={**environment,"HIVE_USER_SETUP_INTERRUPT_AFTER":"0"},capture_output=True,text=True,timeout=30)
        self.assertEqual(patched.returncode,87,(patched.stdout,patched.stderr))
        patch_progress = read_yaml(progress_path)
        self.assertNotEqual(patch_progress["operation_id"],progress["operation_id"])
        document.unlink()
        recovered = subprocess.run([str(self.hive_binary),"setup","--scope","user","--user-root",str(self.setup_user_root),
            "--resume","--summary","--output","json"],env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(recovered.returncode,0,(recovered.stdout,recovered.stderr))
        patch_result = json.loads(recovered.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(patch_result)
        self.assertTrue(patch_result["data"]["completion_verified"])
        self.assertLessEqual(len(recovered.stdout.encode("utf-8")),8192)
        self.assertEqual(read_yaml(progress_path)["operation_id"],patch_progress["operation_id"])
        settings = read_yaml(settings_path)
        self.assertEqual(settings["persona"]["id"],"friendly")
        self.assertEqual(settings["preference_decisions"]["/persona/id"]["origin"],"external-change")
        self.assertIsNone(settings["preference_decisions"]["/persona/id"]["decided_at"])
        for key, value in old_choices.items():
            if key != "/persona/id":
                self.assertEqual(settings["preference_decisions"][key],value,key)
        after_apply = snapshot_tree(self.setup_user_root)
        inspected = subprocess.run([str(self.hive_binary),"setup","--scope","user","--user-root",str(self.setup_user_root),
            "--show","--summary","--output","json"],env=environment,input="",capture_output=True,text=True,timeout=15)
        self.assertEqual(inspected.returncode,0,(inspected.stdout,inspected.stderr))
        self.assertEqual(json.loads(inspected.stdout)["data"]["settings"]["persona"]["id"],"friendly")
        self.assertLessEqual(len(inspected.stdout.encode("utf-8")),4096)
        self.assertEqual(snapshot_tree(self.setup_user_root),after_apply)
        settings_before = settings_path.read_bytes()
        knowledge = self.setup_user_root / ".hive/knowledge/retained.md"
        knowledge.parent.mkdir(parents=True, exist_ok=True)
        knowledge.write_bytes(b"User-owned knowledge.\n")
        removal = subprocess.run([str(self.hive_binary),"uninstall","--confirm","--user-root",str(self.setup_user_root),
            "--output","json"],env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(removal.returncode,2,(removal.stdout,removal.stderr))
        removal_result=json.loads(removal.stdout)
        self.assertEqual(removal_result["code"],"hive.user-uninstall-pending")
        self.assertEqual(removal_result["data"]["core"]["owner"],"unverified")
        self.assertEqual(removal_result["data"]["steps"]["host_activation"],"removed")
        self.assertFalse((self.setup_user_root / ".hive/install/antigravity.json").exists())
        self.assertFalse(json.loads((self.work_root / "host-actions.antigravity.state").read_text())["plugin"])
        self.assertEqual(settings_path.read_bytes(),settings_before)
        self.assertEqual(foreign.read_bytes(),foreign_before)
        self.assertEqual(knowledge.read_bytes(),b"User-owned knowledge.\n")
        document.write_text(json.dumps(read_yaml(settings_path)),encoding="utf-8")
        restored=subprocess.run([str(self.hive_binary),"setup","--scope","user","--answers",str(document),
            "--apply","--user-root",str(self.setup_user_root),"--output","json"],env=environment,input="",capture_output=True,text=True,timeout=30)
        self.assertEqual(restored.returncode,0,(restored.stdout,restored.stderr))
        self.assertEqual(settings_path.read_bytes(),settings_before)
        self.assertEqual(knowledge.read_bytes(),b"User-owned knowledge.\n")

    def test_failed_harness_is_incomplete_and_does_not_report_effective_policy(self):
        process=subprocess.run([str(self.hive_binary),"__complete-user-update","--user-root",str(self.setup_user_root),
            "--interaction","non-interactive"],env={**os.environ,"PATH":str(self.work_root / "absent-commands")},
            input="",capture_output=True,text=True,timeout=30)
        self.assertNotEqual(process.returncode,0,(process.stdout,process.stderr))
        result=json.loads(process.stdout)
        Draft202012Validator(ACTION_RESULT_SCHEMA).validate(result)
        self.assertEqual(result["data"]["steps"]["settings"],"incomplete")
        self.assertEqual(result["data"]["steps"]["harness"],"unverified")
        self.assertIsNone(result["data"]["effective_usage_policy"])
        self.assertEqual(result["data"]["monitor_state"]["sensor_queries"],0)
        self.assertEqual(result["data"]["resume_command"],result["next_action"])
