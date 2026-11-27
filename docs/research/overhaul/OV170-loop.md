# OV170-002·003·004·006·007 — 야간 루프 입력·반복·정지·복구·명령

- 기준일: 2026-11-28, 실행 장소: cloud 세션
- 구현: `crates/hive-core/src/overnight.rs`(순수 상태·검증, 입출력·프로세스·제공자 호출 없음), `crates/hive-cli/src/overnight.rs`(`hive loop start|status|record|stop`)
- 권한 경계: [ADR-0027](../../decisions/ADR-0027-envelope-receipt-layers.md). 작업자 실행 권한은 서명 소환, 루프 기록은 계획 봉투·결과·결정만

## OV170-002 루프 입력

- 목표 문서: 제목·수용 문장·문서 지문(`sha256`). Hive의 문서 재작성 없음
- 점수표: 항목 1–32개, 종류 `build·test·lint·accessibility·visual-diff·critique·custom`, 가중치 합 1.00, 통과 기준 0.00–1.00(소수 둘째 자리), 결정적 검사 1개 이상
- 예산: 반복 1–200회, 전체 시간 60초–24시간, 사용량 하한 0–99%, 점수 정체 N·같은 실패 K 1–20
- 사람 결정 지점: 최대 32개, 되돌릴 수 없는 단계 표시
- 시험 `invalid_scorecards_and_budgets_are_refused`: 가중치 합·소수 자리·중복 id·결정적 검사 없음·반복 0·시간 10초·하한 100·지문 형식·NaN·모르는 필드 거부

## OV170-003 반복 단위

- 단계: 호스트 이어가기(`continued`) → 계획 봉투(`planned`) → 작업자 확인증(`receipt`) → 결정적 검사(`checked`) → 다른 제공자 비평(`critiqued`) → 판정(`judged`)
- 반복마다 점수·변경 지문·비평 제공자·심각한 비평 수·중복 방지 키(`loop_id`·입력 지문·번호의 지문) 기록. 같은 키 두 번째 이어가기·확인증, 다른 반복의 확인증, 같은 제공자 비평 거부
- 확인증 유실: `receipt-missing` → `dispatch-uncertain`으로 정지, 자동 재실행 없음
- 시험 `three_iterations_advance_and_duplicates_are_refused`, `lost_receipt_is_dispatch_uncertain_and_never_redispatched`(가짜 호스트·작업자 사건)

## OV170-004 정지 조건

- 우선순위: 모든 검사 통과·심각한 비평 0 → 되돌릴 수 없는 단계 → 사용량 하한 → 시간 → 반복 수 → 같은 실패 K회 → 점수 N회 정체. 사람의 중지(`Cancel`)
- 정지 뒤 새 계획 봉투 거부. 진행 중 작업의 취소 근거: 호스트 취소 확인증이 있으면 `receipt`, 공식 취소가 없으면 `unproven`, 진행 작업이 없으면 `nothing-in-flight`. 키 입력으로 중단 보완 없음
- 시험 `six_stop_conditions_each_stop_the_loop`(통과·반복·시간·사용량·정체·같은 실패와 되돌릴 수 없는 단계), `stop_creates_no_envelope_and_labels_cancel_proof_honestly`

## OV170-006 복구와 루프 전용 브랜치

- `recover` 판단: 진행 작업 0이면 다음 키로 이어가기, 디스크의 확인증은 채택(재실행 0), 임대 안이면 대기, 임대가 지나면 사람의 안전한 재점유 확인, 정지된 루프는 보고서만
- 끝난 반복 재실행 0건(재시작 뒤 새 키와 기존 키 불일치)
- 브랜치: `loop/<id>`만 커밋 허용(`commit_allowed`), push·병합·배포는 사람 승인

## OV170-007 사용자 명령과 아침 보고서

- `hive loop start --goal --scorecard`: 입력·상태 기록과 호스트 공식 이어가기 안내만(Claude `/loop`·예약, Codex·Gemini 호스트 소유 처리기). `launched_processes: 0`
- `hive loop status`: 기록 조회와 복구 판단, `hive loop record`: 호스트 보고 사건 반영, `hive loop stop`: 취소 사건과 취소 근거, `new_envelopes: 0`
- 모든 출구에서 `.hive/loops/<id>/report.md` 아침 보고서(상태·반복 수·점수 추이·대기 결정·변경·남은 사용량)
- 기존 준비 전용 명령(`initialize·validate·checkpoint·steer·prepare·recover`)은 이미 `Advanced` 묶음. 하위 명령 이동은 하지 않고 도움말에서 새 명령을 앞에 배치(판단 기록)
- 시험 `start_status_record_and_stop_never_launch_a_process`

## 확인

| 항목 | 결과 |
| --- | --- |
| Rust | `hive-core` overnight 5, `hive-cli` overnight·도움말 39 통과, 전체 clippy(1.100) 경고 0, fmt 통과 |
| 도움말 | `hive loop` 등록 갱신과 스냅샷 재생성 |

## 한계

- 모의 사건 시험은 실제 모델·호스트 반복 성공 근거가 아님. 실제 야간 시나리오는 OV170-008(local, 사용자 동의)
- Claude 예약 작업 해제의 실제 확인과 Discord 결정 수신은 OV170-005(사용자 승인)
