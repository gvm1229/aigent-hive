# Hive 재정비 할 일

> 기준일: 2026-11-13
> 원본: Claude "aigent-hive 개편 계획" 초안. 실행 기준: 이 폴더
> 범위: 재정비 지정 요청. 활성 버전: PLAN.md

## 현재 위치

- 단계 1b: 8/8·0.14.2 정식·병합·분기 삭제
- 단계 2: 10/10(확인 7·대체 종료 3), `develop@24057f36` 병합·분기 삭제
- 단계 3: 19/19·0.15.0 정식·구독자 전송·PC 설치·단계 브랜치 원격/로컬 삭제 완료. [정식 근거](../../research/overhaul/OV150-013.md)·[Windows 실제 Claude 왕복](../../research/overhaul/OV150-001.md), 단계 4 구현 시작, 공개 시험 전 중단
- Fable (b) 채택. [ADR-0026](../../decisions/ADR-0026-official-cli-extension-points.md)·[반영·검증](../../research/overhaul/OVX-009.md)
- OVX-005–007: 창 제어 폐기로 대체 종료, 성공 없음·근거 보존. 대체 왕복: OV150-001
- [시작 근거](../../research/overhaul/stage-2-start.md) · [기존 결과·한계](../../research/overhaul/stage-2-closeout.md) · [검토](../../research/overhaul/stage-2-review.md)
- 창 제어 서면 확인: 비차단 별도 조사, 진입 조건·금지 해제 권한 아님

## 승인

- 현재 요청: 0.15.0 Windows 왕복 근거·참조 처리기·공개 시험·develop/main 병합·정식·게시·구독자 안내·PC 설치 전 과정 승인, 기본 비활성 유지

## 재개 절차

1. [07](../../../.agents/directives/07-installed-usage-guard.md) 사용량 사전 확인, 차단 시 즉시 중단
2. 단계 파일·브랜치 확인. 부재 시 아래 새 단계 절차
3. [재조정](../../../.agents/directives/references/plan-reconciliation.md): 계획·소스·시험·커밋 대조, 완료 근거 반영·불확실 항목 미완료
4. 첫 `- [ ]`·`state: agent-owned` 중 의존성 완료 항목 선택
5. [06](../../../.agents/directives/06-session-coordination.md) 세션 기록에 수정 경로 예약
6. 항목 범위 실행, 설계 문제는 소유자 보고·임의 확장 금지
7. 확인 실행. 실패는 미완료·중단 메모, 문서 변경은 documentation 검사와 문체 종료 0 필수
8. `- [x]`·`state: complete; evidence: repo:docs/research/overhaul/<ID>.md#sha256:<값>` 반영. 근거에 명령·결과·한계 기록
9. 현재 위치·진행 표 갱신, [03](../../../.agents/directives/03-workflow.md)에 따라 커밋 후 다음 항목

## 브랜치 규칙

2026-11-10 승인: 단계 브랜치·push·원격 삭제·버전 시작·정식 출시. [브랜치 규칙](../../guides/branching-rules.md) 승인 예외, 제공자 실행은 ADR-0026 적용.

2026-11-12 재정비 상시 승인: 정식 배포 포함 전 항목 사전 승인. 구조 선택도 권고안으로 진행·이유 기록, 응답 대기 없음. 검사·수용·계획 순서 통과 시 정식·main·게시·설치 승인 기록. 재정비 밖은 AGENTS·01 그대로. [승인 반영·검증](../../research/overhaul/autonomy-default.md)

- 단계별 브랜치·타 단계 구현 금지. 단계 3 왕복 검증 우선
- 새 단계: 이전 develop 병합 확인 → `git switch develop` → `git pull --ff-only` → `python scripts/branch-policy.py create <단계 브랜치>` → 현재 위치 갱신
- 작업·체크는 같은 브랜치. 전환 전 미완료 작업 커밋·깨끗한 작업 트리
- 병합 직전 Fable 5.2 advisor의 단계 전체 읽기 전용 검토. 부재 시 일반 Fable 기본 수준 대체, 이상 수정·확인 후 병합. [검토자 소유권 지침](../../../.agents/directives/references/agent-delegation.md#stage-review-and-release)의 관리자 원문을 모든 검토 요청에 포함
- 제품 목적·구조·큰 변경·위임 제안도 검토 근거 기록. 수용 제안은 계획·새 항목 연결, 누락 금지
- 단계 완료 항목에서 develop 병합·push. 제품 정식은 이후 develop에서 진행, main PR의 head는 develop. 단계 브랜치 삭제는 병합 뒤, 제품 단계는 정식 뒤
- 정식 뒤 삭제·다음 단계 시작의 추가 확인 생략. 병합 뒤 보고 대기 없이 다음 단계
- 구독자 안내: 2026-11-12부터 재정비 전체 완료까지 사전 승인. 원문·지문 파일 develop→main PR 승인과 `release-publication`의 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` 최초 설정 뒤 `send_subscriber_update=true` 게시, 추가 확인 제외
- 05 안전 유지, AI trailer·Codex Ultrafast 금지

| 단계 | 브랜치 |
| --- | --- |
| 0 | `chore/overhaul-0-repo-cleanup` |
| 1 | `fix/overhaul-1-0.14.1` |
| 1b | `fix/overhaul-1-0.14.2` |
| 2 | `test/overhaul-2-baseline` |
| 3 | `feature/overhaul-3-0.15.0-orchestrator` |
| 4 | `feature/overhaul-4-0.15.1-detection` |
| 5 | `feature/overhaul-5-0.15.2-memory` |
| 6 | `feature/overhaul-6-0.16.0-mediation` |
| 7 | `feature/overhaul-7-0.17.0-overnight-loop` |

## 중단 대비

- 항목당 커밋 하나, 확인 전 체크 금지
- 한도·종료 임박 시 새 항목 시작 금지. 여유가 있을 때 진행 메모 후 호스트 예약 재개 사용, 반복 확인 금지. 기존 사용자 요청의 5시간 한도 100% 복구 대기·자동 재개는 07의 차단 후 도구 금지·새 요청 사전 확인보다 낮은 우선순위
- 메모: `- <날짜> <ID>: 끝낸 단계 / 남은 단계 / 바뀐 파일`, 현재 위치에 기록
- 승인 대기는 건너뛰고 독립 작업, 승인 대기 절에 ID·질문 기록. 다른 세션과 동일 파일 동시 수정 금지
- 사용자 실제 설치·`~/.hive`·로그인 정보는 항목별 승인 범위만 접근

## 상태

| 표기 | 뜻 |
| --- | --- |
| `- [ ]`·`agent-owned` | 바로 진행할 작업 |
| `- [ ]`·`awaiting-user-authority` | 승인 대기, `owner`·`reason` 필수 |
| `- [ ]`·`awaiting-external-evidence` | 외부 근거 대기 |
| `- [ ]`·`blocked` | 진행 불가, 해결 경로 기록 |
| `- [x]`·`complete` | 근거 필수. 대체 종료는 실행 성공과 별도 표시 |

`scripts/plan_state.py` 형식. 제품 시작 시 `docs/plans/active/` 이동·PLAN 등록. `depends`는 같은 버전만, 단계 간 순서는 진입 조건.

## 단계와 진행

| 단계 | 파일 | 제품 변경 | 진입 조건 | 완료 |
| --- | --- | --- | --- | ---: |
| 0 저장소 정리 | [repo-cleanup.md](repo-cleanup.md) | 없음 | 없음 | 10/10 |
| 1 0.14.1 수정판 | [overhaul-0.14.1.md](../../archive/plans/overhaul-0.14.1.md) | 있음 | 단계 0 develop 병합·버전 승인 | 11/11 |
| 1b 0.14.2 수정판 | [overhaul-0.14.2.md](../active/overhaul-0.14.2.md) | 있음 | 단계 1 병합·2026-11-12 승인 | 8/8 |
| 2 기준선·공식 확장점 결정 | [baseline-experiments.md](baseline-experiments.md) | 없음 | 0.14.2 정식·정리 | 10/10 처리 |
| 3 0.15.0 작업 전달 | [0.15.0-orchestrator.md](../active/0.15.0-orchestrator.md) | 있음 | 단계 2 병합·OVX-009 결정, 첫 검증 OV150-001 | 6/6 |
| 3 0.15.0 하위 기능 | [0.15.0-gods.md](../active/0.15.0-gods.md) | 있음 | OV150-002 | 7/7 |
| 3 호스트 참조 처리기 | [0.15.0-host-reference.md](../active/0.15.0-host-reference.md) | 있음 | OV150-001·OV150-006, 정식 전 | 1/1 |
| 3 컴파일러 호환 | [0.15.0-qualification.md](../active/0.15.0-qualification.md) | 있음 | 첫 원격 CI | 1/1 |
| 3 운영체제 시험 도구 | [0.15.0-windows-qualification.md](../active/0.15.0-windows-qualification.md) | 없음 | 원격 CI 실패 | 4/4 |
| 4 0.15.1 판별 | [0.15.1-detection.md](../active/0.15.1-detection.md) | 있음 | 0.15.0 정식 | 4/11 |
| 5 0.15.2 기억 | [0.15.2-memory.md](0.15.2-memory.md) | 있음 | 0.15.1 정식 | 0/10 |
| 6 0.16.0 메시지 중재 | [0.16.0-mediation.md](0.16.0-mediation.md) | 있음 | 0.15.2 정식 | 0/11 |
| 7 0.17.0 호스트 야간 반복 | [0.17.0-overnight-loop.md](0.17.0-overnight-loop.md) | 있음 | 0.16.0 정식 | 0/11 |

## 불변 조건

- 공식 문서의 CLI 확장점만 사용. 제공자 CLI 키 주입·화면 캡처 금지, tmux·psmux `send-keys`/`capture-pane`·cmux 경로 제외
- Claude는 사람이 쓰는 대화형 호스트: 자체 훅·하위 에이전트·`/loop`·예약. 비대화형·`claude -p` 금지
- Codex·Gemini는 구독 로그인 공식 비대화형 작업자. Claude+Codex 먼저, Gemini도 동등한 작업자. Hive는 파일 봉투·수신 상자·훅 조율, 호스트가 실제 호출
- Hive 자체 모델·프로세스 실행·제공자 API·자격 증명 요청·파일 읽기·복사 없음. 공식 기능 문서와 실제 동작 수용 구분
- 실험: `tests/work/`·프로젝트 한정 훅. 전역 호스트 설정·훅·인증 변경 금지
- Markdown·YAML·TOML 정본, SQLite 재생성 파생물. 사용자 바이트·과거 기준본 보존, 적용 전 미리보기·확인
- 정식 이후 새 제품 바이트는 새 버전·test.1, 재정비 정식·게시·설치는 위 승인 적용
