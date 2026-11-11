# Hive 재정비 할 일

> 기준일: 2026-11-10
> 설계 원본: Claude 문서 "aigent-hive 개편 계획"(검토용 초안). 실행 기준은 이 폴더
> 범위: 사용자가 "재정비 계속"처럼 이 작업을 지정할 때의 실행 기준. 0.14.0 활성 계획은 그대로 유지

## 현재 위치

- 현재 단계 브랜치: `fix/overhaul-1-0.14.1`
- 다음 항목: `OV141-011`(정식 출시와 이 PC 갱신) ([overhaul-0.14.1.md](../active/overhaul-0.14.1.md))
- 마지막 완료: `OV141-010`(develop 병합)
- 승인 대기: `OV141-011` — 구독자 안내 초안(`docs/releases/0.14.1.subscriber.ko.md`) 승인 또는 수정, 저장소 비밀값 `AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST`에 승인 지문 등록(현재 초안 `sha256:1088be71de63a9c8bb1a5f15dd341142cd7a5d2b0327c71dfc979f429bc50c2c`)
- 중단 메모: 2026-11-11 OV141-011: main 병합(PR #90, `ade346b2`)·정식 후보 run `43031639589` 성공 / 게시 run `43034744767`은 구독자 안내 비활성으로 거부, npm `latest`는 0.14.0 유지. 남은 단계: 안내 승인 → 안내 파일을 develop→main 병합 → 정식 후보 재빌드(`promoted_test_version=0.14.1-test.1`, `acceptance_run_id=43019363819`) → 비밀값 등록 → `send_subscriber_update=true`로 게시 → 이 PC 갱신(사용자 PC 필요) / 바뀐 파일: `docs/releases/0.14.1.subscriber.ko.md`·`.sha256`, `docs/01-index.md`

## 재개 절차

어느 AI든, 언제 다시 시작하든 같은 순서.

1. 사용량 확인: [07](../../../.agents/directives/07-installed-usage-guard.md)의 사전 확인. 차단이면 즉시 중단
2. 위치·브랜치 확인: 위 "현재 위치"의 단계 파일 열기, `git switch <현재 단계 브랜치>`. 브랜치가 없으면 아래 브랜치 규칙의 새 단계 시작 절차
3. 대조: 그 파일의 모든 항목과 현재 소스·시험·커밋 비교([재조정](../../../.agents/directives/references/plan-reconciliation.md)). 이미 끝난 일은 근거와 함께 완료 처리, 불확실한 일은 미완료 유지
4. 선택: 위에서부터 첫 `- [ ]` 항목 중 `state: agent-owned`이고 `depends`가 모두 완료인 항목 하나
5. 예약: `.agents/work/active-sessions/`의 세션 기록에 수정 경로 등록([06](../../../.agents/directives/06-session-coordination.md))
6. 실행: 항목의 "변경"만 수행. 범위 확장·설계 변경 금지, 설계 문제는 계획 소유자에게 보고
7. 확인: 항목의 "확인" 실행. 실패 시 미완료 유지와 중단 메모 기록. 문서 변경 항목은 `python scripts/test-lanes.py --lane documentation`과 문체 검사기의 종료 코드 0까지 확인
8. 체크: `- [x]`와 `state: complete; evidence: repo:docs/research/overhaul/<ID>.md#sha256:<값>`으로 변경. 근거 문서에 실행 명령·결과·한계 기록
9. 갱신: "현재 위치"와 아래 진행 표 갱신, 커밋([03](../../../.agents/directives/03-workflow.md)), 다음 항목

## 브랜치 규칙

2026-11-10 사용자 지시. push·원격 브랜치 삭제·버전 시작·정식 출시는 사용자 사전 승인(2026-11-10), 항목별 재승인 불필요. [브랜치 규칙](../../guides/branching-rules.md)의 "특정 작업에 대한 사용자 명시 승인" 예외로 아래 단계 브랜치 생성 허용.

- 단계마다 전용 브랜치 하나. 다른 단계의 일은 그 브랜치에 커밋 금지
- 새 단계 시작: 이전 단계의 develop 병합 확인 → `git switch develop` → `git pull --ff-only` → `python scripts/branch-policy.py create <단계 브랜치>` → 이 문서 "현재 위치"의 브랜치 갱신
- 단계 작업의 모든 커밋은 단계 브랜치에. 이 문서와 단계 파일의 체크도 같은 브랜치에서 갱신
- 병합 전 검토(2026-11-10 사용자 지시): 단계의 develop 병합 항목 직전에 Fable 5.2 advisor 하위 에이전트로 단계 전체 변경 검토, 발견한 이상은 같은 단계 브랜치에서 수정·확인 뒤 병합. 검토 결과는 병합 항목의 근거 문서에 기록
- 다음 단계 자동 진행(2026-11-10 사용자 지시): 병합 뒤 사용자 보고를 기다리지 않고 다음 단계 브랜치 생성과 진행
- 단계 완료: 각 단계 파일의 "develop 병합과 단계 브랜치 종료" 항목으로 `develop`에 병합과 push. 제품 단계의 정식 출시는 병합 뒤 `develop`에서 진행(`main` PR의 head는 `develop`)
- 단계 브랜치 삭제는 병합 뒤(제품 단계는 정식 출시 뒤)
- 브랜치 전환 전 작업 트리는 깨끗한 상태. 미완료 일은 단계 브랜치에 커밋 뒤 전환

| 단계 | 브랜치 |
| --- | --- |
| 0 | `chore/overhaul-0-repo-cleanup` |
| 1 | `fix/overhaul-1-0.14.1` |
| 2 | `test/overhaul-2-baseline` |
| 3 | `feature/overhaul-3-0.15.0-orchestrator` |
| 4 | `feature/overhaul-4-0.15.1-detection` |
| 5 | `feature/overhaul-5-0.15.2-memory` |
| 6 | `feature/overhaul-6-0.16.0-mediation` |
| 7 | `feature/overhaul-7-0.17.0-overnight-loop` |

## 중단 대비 규칙

- 한 항목은 커밋 하나 크기. 확인 통과 전 체크 금지
- 사용량 한도·종료 임박 시 새 항목 시작 금지, 진행 중 항목의 중단 메모 기록 뒤 5시간 한도가 다시 100% 사용 가능해질 때까지 대기하고 자동 재개(2026-11-10 사용자 지시, 수동 재시작 최소화). 대기는 예약 재개(`send_later` 등 세션의 예약 기능) 사용, 반복 확인 금지
- 중단 메모 형식: `- <날짜> <ID>: 끝낸 단계 / 남은 단계 / 바뀐 파일`. 이 문서 "현재 위치"에 한 줄
- `awaiting-user-authority` 항목은 건너뛰고 다음 독립 항목 진행. "승인 대기"에 ID와 질문 기록
- 다른 세션과 같은 파일 동시 수정 금지(06)
- 사용자 실제 설치·`~/.hive`·로그인 정보는 항목에 적힌 승인 없이 읽기·쓰기 금지

## 상태 표기

| 표기 | 뜻 |
| --- | --- |
| `- [ ]` + `agent-owned` | AI가 바로 진행할 일 |
| `- [ ]` + `awaiting-user-authority` | 사용자 승인 대기, `owner`·`reason` 필수 |
| `- [ ]` + `awaiting-external-evidence` | 외부 근거 대기 |
| `- [ ]` + `blocked` | 진행 불가, 해결 경로 기록 |
| `- [x]` + `complete` | 확인 완료, `evidence` 필수 |

항목 형식은 `scripts/plan_state.py`와 동일. 제품 단계 시작 때 해당 파일을 `docs/plans/active/`로 옮겨 `PLAN.md`에 등록하면 그대로 집계. `depends`는 같은 버전 파일 안에서만 사용, 단계 사이 순서는 아래 표의 진입 조건.

## 단계와 진행

| 단계 | 파일 | 제품 변경 | 진입 조건 | 완료 |
| --- | --- | --- | --- | ---: |
| 0 저장소 정리 | [repo-cleanup.md](repo-cleanup.md) | 없음 | 없음 | 10/10 |
| 1 0.14.1 수정판 | [overhaul-0.14.1.md](../active/overhaul-0.14.1.md) | 있음 | 단계 0 develop 병합, 버전 승인 | 10/11 |
| 2 기준선·창 실험 | [baseline-experiments.md](baseline-experiments.md) | 없음 | 0.14.1 정식 | 0/10 |
| 3 0.15.0 오케스트레이터 | [0.15.0-orchestrator.md](0.15.0-orchestrator.md) | 있음 | 단계 2 develop 병합, OVX-009 승인 | 0/7 |
| 3 0.15.0 하위 기능 | [0.15.0-gods.md](0.15.0-gods.md) | 있음 | OV150-001 | 0/7 |
| 4 0.15.1 판별 | [0.15.1-detection.md](0.15.1-detection.md) | 있음 | 0.15.0 정식 | 0/10 |
| 5 0.15.2 기억 | [0.15.2-memory.md](0.15.2-memory.md) | 있음 | 0.15.1 정식 | 0/10 |
| 6 0.16.0 중재 | [0.16.0-mediation.md](0.16.0-mediation.md) | 있음 | 0.15.2 정식 | 0/12 |
| 7 0.17.0 야간 루프 | [0.17.0-overnight-loop.md](0.17.0-overnight-loop.md) | 있음 | 0.16.0 정식 | 0/11 |

## 불변 조건

- 모델·제공자 API 직접 호출, API 키·자격 증명 파일 읽기·복사 금지
- 공식 CLI(`claude`·`codex`·`gemini`)는 다중화 창(tmux·psmux·cmux) 안의 대화형 세션으로만 실행. 헤드리스는 보조 경로
- 정본은 Markdown·YAML·TOML, SQLite는 재생성 가능한 파생물
- 사용자 바이트·과거 기준본 보존, 적용은 미리보기·확인 뒤
- 새 제품 바이트는 새 버전과 test.1. 정식 태그·게시·설치는 버전별 사용자 승인
