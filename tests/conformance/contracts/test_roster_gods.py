"""God entry discovery metadata and unchanged delegation targets."""
import unittest
import json
import re
from pathlib import Path
import yaml
ROOT=Path(__file__).resolve().parents[3]
CASES={'athena': {'en': ['plan the work', 'design a workflow', 'review the strategy'], 'ko': ['작업 계획', '흐름 설계', '전략 검토'], 'target': 'verified-workflow'}, 'artemis': {'en': ['research best practices', 'find official sources', 'compare approaches'], 'ko': ['구현 관행 조사', '공식 자료 찾기', '방법 비교'], 'target': 'research-best-practices'}, 'hephaistos': {'en': ['implement a change', 'review generated code', 'polish selected code'], 'ko': ['변경 구현', '생성 코드 검토', '선택 코드 정리'], 'target': 'code-polish'}, 'apollon': {'en': ['rewrite selected Korean', 'improve this writing', 'preserve meaning while editing'], 'ko': ['선택한 한국어 윤문', '글 표현 개선', '뜻을 지키며 편집'], 'target': 'humanize-kor'}, 'eileithyia': {'en': ['set up a project', 'configure Hive', 'refresh project guidance'], 'ko': ['프로젝트 설정', 'Hive 구성', '프로젝트 지침 갱신'], 'target': 'project-setup'}, 'poseidon': {'en': ['coordinate a team', 'run bounded lanes', 'organize parallel work'], 'ko': ['팀 조율', '제한된 작업 분담', '병렬 작업 정리'], 'target': 'team-execution'}, 'demeter': {'en': ['recall project knowledge', 'find a saved fact', 'search reviewed knowledge'], 'ko': ['프로젝트 지식 조회', '저장된 사실 찾기', '검토 지식 검색'], 'target': 'knowledge-recall'}, 'aither': {'en': ['check usage controls', 'inspect remaining usage', 'review usage policy'], 'ko': ['사용량 보호 확인', '남은 사용량 조회', '사용량 정책 검토'], 'target': 'usage-guard'}, 'hades': {'en': ['clean completed artifacts', 'inspect cleanup candidates', 'prepare focused commits'], 'ko': ['완료 산출물 정리', '정리 후보 확인', '작은 단위 커밋 준비'], 'target': 'code-polish'}}
class GodEntryTests(unittest.TestCase):
    def test_three_korean_and_english_discovery_phrases_per_god(self):
        for name,case in CASES.items():
            text=(ROOT/f'harness/skills/{name}/SKILL.md').read_text(encoding='utf-8')
            header=yaml.safe_load(text.split('---',2)[1])
            for phrase in case['en']+case['ko']:
                with self.subTest(god=name,phrase=phrase): self.assertIn(phrase,header['description'])
            target=ROOT/f"harness/skills/{case['target']}/SKILL.md"
            self.assertTrue(target.is_file())
            self.assertIn('../'+case['target']+'/SKILL.md',text)
            policy=yaml.safe_load((ROOT/f'harness/skills/{name}/agents/openai.yaml').read_text(encoding='utf-8'))
            self.assertTrue(policy['policy']['allow_implicit_invocation'])
    def test_each_setup_schema_accepts_god_selection(self):
        for path in ['user-setup.schema.json','user-setup-catalog.schema.json','setup-answers.schema.json']:
            schema=json.loads((ROOT/'schemas'/path).read_text(encoding='utf-8'))
            self.assertTrue(set(CASES).issubset(schema['$defs']['skill_name']['enum']))
    def test_ownership_table_covers_every_public_command_and_skill_once(self):
        text=(ROOT/'docs/architecture/god-ownership.md').read_text(encoding='utf-8')
        rows=[line.split('|') for line in text.splitlines() if line.startswith('| ') and line.split('|')[1].strip() in CASES]
        self.assertEqual(len(rows),9)
        commands=[name for row in rows for name in re.findall(r'`([^`]+)`',row[3])]
        skills=[name for row in rows for name in re.findall(r'`([^`]+)`',row[4])]
        catalog=yaml.safe_load((ROOT/'harness/skills/catalog.yml').read_text(encoding='utf-8'))
        self.assertEqual(set(skills),{entry['name'] for entry in catalog['skills']})
        self.assertEqual(len(skills),len(set(skills)))
        main=(ROOT/'crates/hive-cli/src/main.rs').read_text(encoding='utf-8')
        dispatch=main.split('fn dispatch(',1)[1].split('struct UsageArguments',1)[0]
        public=set(re.findall(r'^        Some\("([a-z][a-z-]+)"',dispatch,re.M))|{'doctor','check-target'}
        self.assertEqual(set(commands),public)
        self.assertEqual(len(commands),len(set(commands)))
    def test_new_entries_have_exact_projections(self):
        for name in CASES:
            canonical=(ROOT/f'harness/skills/{name}/SKILL.md').read_bytes()
            for base in ['harness/plugins/aigent-hive/skills','harness/template/.agents/skills','harness/template/.claude/skills']:
                self.assertEqual(canonical,(ROOT/base/name/'SKILL.md').read_bytes())
import subprocess
from tests.conformance.support.harness import Phase1CliTestCase, read_yaml, write_yaml, snapshot_tree
class GodCliTests(Phase1CliTestCase):
    def call(self,*args):
        return subprocess.run([str(self.hive_binary),*map(str,args)],capture_output=True,text=True,encoding='utf-8',cwd=ROOT)
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
            payload=json.loads(result.stdout);self.assertEqual(payload['announcement'],expected)
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
