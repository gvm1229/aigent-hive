# OV170-031 local 인계 — 실제 세션에서 자동으로 맞추기 확인

- 작성: 2026-11-29 cloud 세션. 필요 조건: 이 기능을 담은 0.17.0 시험판 설치
- 금지: 자동 커밋 확인 외의 Git 조작, `claude -p`, 전역 `~/.claude`·`~/.codex` 직접 수정

## 단계

1. `tests/work/`에 공개 0.16.1 고정물(`tests/fixtures/project-predecessors/0.16.1/public-stable`)을 풀고 Git 저장소로 커밋
2. `.hive/project.toml`에 `range = "0.17"`을 쓰고 커밋
3. Claude Code로 그 폴더를 열고 첫 화면의 한 줄 안내 확인: `Eileithyia called · project Hive 0.16.1 → 0.17.0, files: N`
4. `git status`로 Hive 파일과 `.hive/project.lock` 변경 확인, `git log`에 새 커밋이 없는지 확인
5. Codex: 설치본의 훅 지원 위치(사용자 훅 설정 또는 플러그인)를 확인하고, 있으면 `hive project sync --hook codex`를 등록해 같은 확인. 없으면 이유 기록
6. `.hive/`가 없는 폴더를 Claude·Codex로 열어 안내·파일 변경 0건 확인

## 결과 기록

- 통과하면 [OV170-031](OV170-031.md)과 계획 항목을 `complete`로, 이어서 OV170-032 완료 표시
- 실패하면 `agent-owned`로 되돌리고 화면·명령 출력 첨부
