# Synthetic fixtures

이 디렉터리에는 실제 사용자 데이터가 없는 소비자 프로젝트 fixture와 normalized expected output만 저장.

Live setup/render test는 `tests/work/`의 disposable directory에서 실행.

- `setup/`: deterministic setup, ownership, role과 consent
- `knowledge/`: synthetic Raw revisions와 prepared Wiki drafts
- `skills/`: Skill routing·prompt refinement·projection fixture
- `run/`: role·run lifecycle fixture
- `judge/`: Judge package·assignment·verdict fixture
- `usage/`: native·fallback usage sensor fixture
- `release/`: migration·platform signer·versioned release fixture
- `native-orchestration/`: host capability fixture
