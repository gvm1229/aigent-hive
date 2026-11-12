"""God entry discovery metadata and unchanged delegation targets."""
import unittest
import json
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
    def test_new_entries_have_exact_projections(self):
        for name in CASES:
            canonical=(ROOT/f'harness/skills/{name}/SKILL.md').read_bytes()
            for base in ['harness/plugins/aigent-hive/skills','harness/template/.agents/skills','harness/template/.claude/skills']:
                self.assertEqual(canonical,(ROOT/base/name/'SKILL.md').read_bytes())
if __name__=='__main__': unittest.main()
