# Agent 자율 실행 지속 계획

> Checklist owner: `AAC-*`
> Target: source agent directive·session coordination·static contract
> Origin: 남은 Agent 소유 회귀 정리·검증·push·시험 게시 작업이 있는 중간 보고 종료

## 목표

- `all todos`, `until completion`, `do not stop` 요청의 Agent 소유 작업 지속
- 진행 보고와 최종 응답의 구분
- 사용자 권한·외부 증거 대기만 정확한 종료 경계로 허용

## Checklist

- [x] [AAC-001] `01-behavior`에 Agent가 직접 실행 가능한 조사·수정·검사·commit·push·CI 감시·release
  action이 남으면 final response 금지 규칙 추가
- [x] [AAC-002] `01-behavior`에 `complete`, `awaiting-user-authority`,
  `awaiting-external-evidence`, `blocked` 종료 상태와 허용 표현 추가
- [x] [AAC-003] `04-documentation-state`에 final response 전 현재 요청의 남은 항목을
  Agent 실행·사용자 권한·외부 증거·차단으로 분류하는 closure gate 추가
- [x] [AAC-004] `06-session-coordination` manifest에 task status·남은 Agent 소유 action·최종
  evidence 요구 기록 추가. `active` 상태의 final completion 금지
- [x] [AAC-005] static contract에 “old Skill regression 발견 → fix → verify → push → candidate →
  publication” fixture와 terminal-state directive 필수 문구 추가
- [x] [AAC-006] 소비자 프로젝트 `AGENTS.md` 템플릿과 Rust 렌더링 결과에 같은 Agent 소유 작업
  지속·중간 보고 종료 금지 계약 추가. Codex·Claude·Antigravity alias 동일 적용
- [x] [AAC-007] 소비자 전역 설정의 영어·한국어 Hive 지침에 같은 종료 상태 분류·자동 수행 지속
  계약 추가
- [x] [AAC-008] template·Rust 렌더러·전역 설치 지침의 계약 일치 정적 회귀와 Source Wiki 사실 갱신

## 완료 기준

- Agent 소유 action이 남은 진행 보고의 종료 0건
- 사용자 action만 남은 경우에만 정확한 권한·행동·기대 증거 인계
- `engineer-run` 호출 여부와 무관한 source directive 적용
- source·consumer directive static contract·Source Wiki lint 통과

## Evidence

- `cargo test -p hive-render --no-fail-fast`: 66 tests PASS
- `cargo test -p hive-cli --bin hive --no-fail-fast`: 328 tests PASS
- `python3 -m unittest tests.conformance.test_phase3_static_contracts -q`: PASS

## 범위 밖

- 사용자 Windows·GitHub protected action·외부 account의 자동 조작
- provider API·credential·host process 제어
