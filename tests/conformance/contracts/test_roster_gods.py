"""Generated god surfaces, name-only discovery, and exact ownership."""
import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("generate_gods", ROOT / "scripts/generate-gods.py")
GENERATOR = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = GENERATOR
SPEC.loader.exec_module(GENERATOR)
GODS = GENERATOR.load_table()
CASES = {god.name: god for god in GODS}


class GodEntryTests(unittest.TestCase):
    def test_descriptions_trigger_only_explicit_god_names(self):
        guide = (ROOT / "docs/skills.md").read_text(encoding="utf-8")
        for god in GODS:
            rows = [line for line in guide.splitlines() if line.startswith(f"| `{god.name}` |")]
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0].split("|")[-2].strip(), f"{god.korean} 호출")
            text = (ROOT / f"harness/skills/{god.name}/SKILL.md").read_text(encoding="utf-8")
            header = yaml.safe_load(text.split("---", 2)[1])
            self.assertEqual(header["description"], god.description_en)
            self.assertIn("only when the user names", header["description"])
            for name in [god.name, god.korean, *god.aliases]:
                self.assertIn(name, header["description"])
            for skill in god.skills:
                if skill != god.name:
                    self.assertIn(f"../{skill}/SKILL.md", text)
            policy = yaml.safe_load((ROOT / f"harness/skills/{god.name}/agents/openai.yaml").read_text(encoding="utf-8"))
            self.assertEqual(policy["interface"]["short_description"], god.description_en)
            self.assertTrue(policy["policy"]["allow_implicit_invocation"])
        hades = (ROOT / "harness/skills/hades/SKILL.md").read_text(encoding="utf-8")
        self.assertNotIn("code-polish", hades)
        self.assertNotIn("ship", hades)
        poseidon = (ROOT / "harness/skills/poseidon/SKILL.md").read_text(encoding="utf-8")
        self.assertNotIn("cleanup", poseidon)
        self.assertNotIn("implementation requests", poseidon)

    def test_every_current_skill_description_says_what_the_skill_does(self):
        # Names alone only decide when a Skill is called; the description must state its work.
        for path in sorted((ROOT / "harness/skills").glob("*/SKILL.md")):
            header = yaml.safe_load(path.read_text(encoding="utf-8").split("---", 2)[1])
            summary = re.sub(r"^\([a-z0-9-]+\)\s*", "", header["description"])
            work = summary.split("Use only when", 1)[0].strip()
            self.assertGreaterEqual(len(work.split()), 4, f"{path.parent.name}: {header['description']}")
        rust = (ROOT / "crates/hive-core/src/gods_generated.rs").read_text(encoding="utf-8")
        for god in GODS:
            self.assertIn(god.summary_en, god.description_en)
            self.assertIn(f'"({god.name}) {god.summary_ko}.', rust)
            self.assertNotIn("명시한 신 이름에만 사용", rust)
        guide = (ROOT / "docs/skills.md").read_text(encoding="utf-8")
        for god in GODS:
            self.assertIn(f"| `{god.name}` | {god.summary_ko} |", guide)

    def test_generated_surfaces_match_the_canonical_typed_table(self):
        result = subprocess.run([sys.executable, str(ROOT / "scripts/generate-gods.py"), "--check"], capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_typed_table_rejects_unknown_fields_wrong_types_and_duplicate_ownership(self):
        original = (ROOT / "harness/gods.toml").read_text(encoding="utf-8")
        for changed in [original.replace('name = "athena"', 'name = 3', 1), original.replace('name = "athena"', 'unknown = "athena"', 1), original.replace('commands = ["doctor", "check-target", "route"]', 'commands = ["roster"]', 1), original.replace('"athena"', '"../escape"')]:
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / "gods.toml"
                path.write_text(changed, encoding="utf-8")
                with self.assertRaises(ValueError):
                    GENERATOR.load_table(path)

    def test_each_setup_schema_accepts_god_selection(self):
        for path in ["user-setup.schema.json", "user-setup-catalog.schema.json", "setup-answers.schema.json"]:
            schema = json.loads((ROOT / "schemas" / path).read_text(encoding="utf-8"))
            self.assertTrue(set(CASES).issubset(schema["$defs"]["skill_name"]["enum"]))

    def test_ownership_table_covers_every_public_command_and_skill_once(self):
        text = (ROOT / "docs/architecture/god-ownership.md").read_text(encoding="utf-8")
        rows = [line.split("|") for line in text.splitlines() if line.startswith("| ") and line.split("|")[1].strip() in CASES]
        self.assertEqual(len(rows), 9)
        commands = [name for row in rows for name in re.findall(r"`([^`]+)`", row[3])]
        skills = [name for row in rows for name in re.findall(r"`([^`]+)`", row[4])]
        catalog = yaml.safe_load((ROOT / "harness/skills/catalog.yml").read_text(encoding="utf-8"))
        self.assertEqual(set(skills), {entry["name"] for entry in catalog["skills"]})
        self.assertEqual(len(skills), len(set(skills)))
        main = (ROOT / "crates/hive-cli/src/main.rs").read_text(encoding="utf-8")
        dispatch = main.split("fn dispatch(", 1)[1].split("struct UsageArguments", 1)[0]
        public = set(re.findall(r'^        Some\("([a-z][a-z-]+)"', dispatch, re.M)) | {"doctor", "check-target"}
        self.assertEqual(set(commands), public)
        self.assertEqual(len(commands), len(set(commands)))

    def test_new_entries_have_exact_projections(self):
        for name in CASES:
            canonical = (ROOT / f"harness/skills/{name}/SKILL.md").read_bytes()
            for base in ["harness/plugins/aigent-hive/skills", "harness/template/.agents/skills", "harness/template/.claude/skills"]:
                self.assertEqual(canonical, (ROOT / base / name / "SKILL.md").read_bytes())
from tests.conformance.support.harness import Phase1CliTestCase, read_yaml, write_yaml, snapshot_tree
class GodCliTests(Phase1CliTestCase):
    def call(self,*args):
        return subprocess.run([str(self.hive_binary),*map(str,args)],capture_output=True,text=True,encoding='utf-8',cwd=ROOT)
    def test_summon_notice_failure_still_emits_action_result(self):
        project = self.work_root / 'consumer'
        project.mkdir()
        missing_home = self.work_root / 'missing-home'
        result = self.call('summon', 'athena', '--task', 'bounded task', '--target', project,
                           '--user-root', missing_home, '--run', 'run-1', '--action', 'task-1',
                           '--expected-head', 'none', '--control-epoch', '0', '--authority-id',
                           'auth-1', '--output', 'json')
        self.assertEqual(result.returncode, 2, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload['action'], 'Summon')
        self.assertEqual(payload['status'], 'error')
        self.assertEqual(payload['exit_code'], 2)
        self.assertFalse(payload['data']['enabled'])
        self.assertFalse((project / '.hive').exists())
    def test_roster_and_korean_alias_summon_are_inert_in_both_languages(self):
        project=self.work_root/'consumer';project.mkdir()
        result=self.call('roster','--target',project,'--user-root',self.setup_user_root,'--output','json')
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(len(json.loads(result.stdout)['roster']['roles']),6)
        digests=[]
        for language,expected in [('en','Calling upon Hades — cleanup'),('ko','하데스 호출 — 정리')]:
            config_path=self.setup_user_root/'.hive/config/user-setup.yml'
            config=read_yaml(config_path);config['interface_language']=language;write_yaml(config_path,config)
            result=self.call('summon','하데스','--task','bounded task','--target',project,'--user-root',self.setup_user_root,'--run','run-1','--action','task-1','--expected-head','none','--control-epoch','0','--authority-id','auth-1','--output','json')
            self.assertEqual(result.returncode,0,result.stderr)
            action=json.loads(result.stdout);self.assertEqual(action['action'],'Summon');payload=action['data'];self.assertEqual(payload['announcement'],expected)
            self.assertIn('[Hades]',result.stderr);self.assertFalse(payload['enabled']);self.assertEqual(payload['mutation_count'],0)
            digests.append(payload['request_digest']);self.assertFalse((project/'.hive').exists())
        self.assertEqual(digests[0],digests[1])
    def test_hades_plan_preserves_files_and_rejects_confirm(self):
        before=snapshot_tree(self.setup_user_root)
        result=self.call('hades','plan','--user-root',self.setup_user_root,'--output','json')
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(json.loads(result.stdout)['changed_paths'],[])
        rejected=self.call('하데스','plan','--user-root',self.setup_user_root,'--confirm','--output','json')
        self.assertNotEqual(rejected.returncode,0)
        self.assertEqual(snapshot_tree(self.setup_user_root),before)
if __name__=='__main__': unittest.main()
