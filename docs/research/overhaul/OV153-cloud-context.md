# 0.15.3 cloud 이어 받기 맥락

- 기준일: 2026-11-17
- 목적: 앞 cloud 세션("Overhaul stage 5: 0.15.3 memory recall")에서 새 cloud 세션("0.15.3: finish OV153-011/012/013 + test.2 + handoff")으로 맥락 손실 없는 이어 받기
- 앞 세션 상태: 2026-11-16 18:17 UTC 마지막 커밋 뒤 대기. 추가 범위 전달 메시지 미반영, 중지 요청 전달. 앞 세션 대화 기록은 새 세션이 읽을 수 없으므로 아래 저장소 기록이 정본

## 먼저 읽을 문서

1. [cloud 위임](OV153-cloud-mandate.md)과 [실행 장소 규칙](../../plans/overhaul/venue.md)의 진행 기본값·넘기기
2. [0.15.3 계획](../../plans/active/0.15.3-memory.md)
3. 요청 원문: [OV153-011](OV153-011-request.md)(추가 보고·A1–A4 포함), [OV153-012](OV153-012-request.md), [OV153-013](OV153-013-request.md)
4. [단계 5 검토](stage-5-review.md)와 [OV153-008](OV153-008.md)(test.1 수용 기록, test.2 절차의 본보기)
5. `origin/fix/ov153-011-agent-setup` 브랜치의 `docs/research/overhaul/OV153-011.md`, `OV153-011-test2.md`, `OV153-local-handoff.md`

## 앞 세션이 남긴 작업 상태

- 브랜치 `fix/ov153-011-agent-setup`: develop 미병합. 구현(`--capabilities` 선택, `--show --summary`의 `agent_setup` 제안, 손대지 않은 터미널 초안 대체, Skill 재작성)·시험·사실 페이지 지문 갱신, 출시 노트·구독자 안내·정식 공개 문서 준비(`5feef869`)까지 존재
- 같은 브랜치에서 `OV153-010-local.md` 삭제, 대신 `OV153-local-handoff.md`. 이 문서의 `TEST2_CANDIDATE`·`TEST2_COMMIT`·`TEST2_ACCEPTANCE` 자리표시 미기입
- `OV153-011-test2.md`: "진행 중" 표시만 있고 실행 기록 없음. test.2 워크플로 미실행
- 독립 검토 결과(브랜치 `OV153-011.md`): 1차 B1·B2·S1–S8, 2차 N1·N2 반영. N3(덜어 냄 시험·표지), N4, S4(저장 능력과 감지 능력 구분 표시), S6·S8 일부 미반영, 범위 밖으로 기록
- 브랜치 기록 기준 검증: Rust 921 통과·2 실패 뒤 제목 갱신 재실행, documentation·release·security·integration 통과, contract는 환경 전용 기존 실패 8개(권한·Copier 비교·시각 검증, 이전 실행과 동일)
- OV153-012·OV153-013: 구현 없음

## 앞 세션 이후 추가된 범위

- OV153-011 A1–A4: 능력 파일 없는 에이전트 미리보기·적용·검증 계약 시험, 대기 결과(`hive.project-setup-input-pending` 등)의 에이전트 경로 `next_action`, 능력 파일 선택 사항 명시, local 재현. 브랜치 `OV153-011.md`의 범위 밖 목록에 있던 "능력 파일 없는 CLI 미리보기·검증 경로"는 범위 안으로 이동
- OV153-013: 변경 0건 프로젝트의 `--scan` 갱신 표시와 `--validate` 실패. 진단 단서는 요청 원문

## 이 저장소의 함정

- 시험 묶음은 stdin을 닫고 실행(`</dev/null`). 열린 stdin에서 documentation 묶음 정지 사례
- 소스·등록부 변경 뒤 사실 페이지의 `repo:<path>#sha256:` 지문 갱신 필요. 미갱신 시 documentation 실패
- 계획 파일 8192바이트 상한(`scripts/plan_state.py`). metadata는 state·depends·evidence·owner·reason, depends는 공백 없는 쉼표 구분, `reason`에 `; ` 금지
- 한국어 문서는 `scripts/check-human-documentation-style.py --all` 통과 필요
- 출시 노트 영어 사실은 25단어 이하, we·you·please·can·may·might·easy·simply·very 금지(`scripts/check-release-notes.py`)
- 지침 파일 03·04는 읽기 예산 여유 없음. 새 규칙은 references 또는 overhaul 문서에 기록
- 공개 시험 게시 뒤 npm 전파 지연: 여섯 패키지 모두 `npm view` 확인 뒤 `recover_published_packages`
- 워크플로 실행은 GitHub MCP `actions_run_trigger`, 실패 재실행은 `rerun_failed_jobs`. `gh` CLI는 이 환경에서 검색 전용
- 커밋·PR에 AI 공동 작성자 표기 금지(분기 정책 거부)

## 끝 조건

- OV153-011(A1–A4 포함)·012·013 cloud 부분이 develop에 병합, CI 통과
- 0.15.3-test.2 게시·세 OS 수용 완료, `OV153-011-test2.md` 기록
- `OV153-local-handoff.md`에 test.2 값과 local 확인 전부(빈 폴더·기존 프로젝트 Skill 설정, A4 재현, OV153-012 실제 터미널, OV153-013 재현 B5)
- 정식 0.15.3 게시 금지
