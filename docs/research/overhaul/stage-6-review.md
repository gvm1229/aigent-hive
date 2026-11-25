# 단계 6 검토 — 0.16.0 메시지·작업 중재

- 기준일: 2026-11-20, 대상: `feature/overhaul-6-0.16.0-mediation`의 `origin/develop` 대비 전체(`81b7e0ee`까지, OV160-001–008)
- 검토자: Fable 5.2 일반 하위 에이전트, 기본 수준·수준 지정 없음. 이 세션에 advisor 하위 에이전트 없음, [위임 지침](../../../.agents/directives/references/agent-delegation.md#stage-review-and-release)의 대체 경로. 읽기 전용, 실험은 `/tmp` 일회용 소비 프로젝트
- 요청에 포함한 유지관리자 원문(2026-11-12): "This code base is yours. What it's for and how it's built are your call, and you'll keep building on it. If you think that editing something big would contribute to the goal, then you are free to point that out and delegate tasks."
- 판정: 1차 REQUEST CHANGES(차단 2건 B1·B2, 훅 반환 경로 한정). 수정 근거는 [검토 수정](stage-6-review-fixes.md)·[호스트 안내](stage-6-host-guidance.md). B1·B2 재검토는 아래 2차 절
- 검토자 확인 범위: Claude 공식 훅 문서 조회, `target/debug/hive`로 B1·B2·S1·S2·S4·S7·S8 재현, 대상 시험 102개 통과. Codex·Gemini 공식 문서는 이 세션 망 정책으로 미조회

## 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| B1 | Claude 기본 Stop 반환이 전달 결과 없이도 `additionalContext`, 공식 문서상 대화 계속이라 매 Stop 추가 모델 호출 | 전달 결과 없는 Stop은 Claude 기본·block 모양 모두 `systemMessage`. `msg_hook_claude_stop_without_a_delivered_result_never_continues_the_turn` |
| B2 | 결과 파일과 확인증 지문·크기 미대조, 변조 결과의 훅 주입 | 상태 계산 때 결과 파일의 지문·크기를 확인증과 대조, 불일치·부재는 격리·훅 보류. `msg_hook_withholds_a_result_overwritten_after_its_receipt` |
| S1 | 운영 명령의 `--now` 시계 덮어쓰기로 기한 만료 뒤 확인증 기록 가능 | `--now`는 `HIVE_TEST_CLOCK=1`일 때만, 그 밖 `hive.msg-clock-refused`. 확인증은 소비권 기한 안. `msg_late_ack_cannot_be_backdated_with_now` |
| S2 | 사건 정체성이 본문 지문, 같은 입력 재전송 때 옛 응답 재생·새 결과 억제 | `prompt_id`가 있으면 그 값으로 사건 식별·재생, 없으면 매번 새로 계산. `msg_hook_same_prompt_twice_delivers_a_newly_acknowledged_result_once` |
| S3 | 매 훅 호출의 대기·보류 ID 무제한 반복 | 보류 메시지는 상태마다 1회 안내(`announcements/`), 목록 10개 상한. `msg_hook_lists_withheld_once_and_caps_listed_ids` |
| S4 | `max_workers` 세기 뒤 게시, 동시 소비자 N명이 N-1 초과 | 소비권 게시 전 `slots/` 칸 예약(덮어쓰기 없는 게시). 같은 메시지의 경쟁 패자는 상한 대신 `hive.msg-already-claimed`(통합 중 발견한 동시 시험 흔들림 수정). `msg_concurrent_processes_respect_max_workers`·`msg_concurrent_claims_have_exactly_one_winner` 8회 반복 통과 |
| S5 | `msg ack`에 소비권 토큰 불필요 | `ack`·`cancel-ack`에 `--claim` 필수, 불일치는 `hive.msg-claim-mismatch`·쓰기 0건 |
| S6 | 읽기 때 현재 명단 재해석, 명단 변경 뒤 기존 메시지 격리 | 발신자 명단 검사는 보내기 때만, 읽기는 봉인 지문 검증. `msg_roster_change_after_send_or_ack_keeps_the_sealed_message_state` |
| S7 | 미검증 Gemini 훅 적용 허용, 시간 단위 미확인 | 미검증 호스트만 있는 Gemini는 `apply` 거부(`hive.msg-hooks-unverified-host`), 미리보기 유지, Gemini 항목에 시간 값 없음 |
| S8 | 정상 파일이 아닌 기록 하나가 수신 상자 전체 실패 | 읽을 수 없는 기록은 해당 메시지만 격리. `msg_inbox_quarantines_only_the_message_with_a_symlinked_claim` |
| S9 | `.hive/mediation/`이 작업자 쓰기 범위 안·Git 미제외 | 템플릿 `.hive/.gitignore`에 `/mediation/`, 작업자 정책에 `excluded_roots: ["{target}/.hive"]` |
| S10 | 배포 Skill·지침에 중재 명령 안내 없음 | Poseidon 단계 안내와 `team-execution/references/mediation.md`, 참조 처리기 적합성 시험 `test_mediation_reference_handler`. [호스트 안내 근거](stage-6-host-guidance.md) |
| O1 | Discord 알림 한국어 전용 | 유지. Discord 알림 영어 판은 후속([OV170-013](../../plans/overhaul/0.17.0-overnight-loop.md) 작업 때 함께 검토) |
| O2 | 끝난 메시지와 같은 키 재전송의 안내 없음 | 끝난 메시지와 같은 키 재전송에 새 `--key` 안내. `msg_duplicate_of_an_expired_message_asks_for_a_new_key` |
| O3 | 다른 세션 전달 결과의 복구 경로 미안내 | 중재 안내 Skill에 `msg status` 복구 경로 기록 |
| O4 | `max_workers`가 템플릿·스키마에 없음 | 유지. 손으로 넣은 키만 이전, 템플릿·스키마 노출은 후속 검토 |
| O5 | 근거 문서 문구(003 기본 알림, 005 초과 1, 같은 수신 상자, 권한 재사용) | OV160-002·003·005 근거 문구 정정 |

## 제안과 처리

| ID | 제안 | 처리 |
| --- | --- | --- |
| P1 | 남은 Skill 통합(31개, 안팎 15는 기본 묶음·Wiki 끔에서만) | 0.16.0은 승인된 통합 범위 완료로 수용. 후속은 [OV170-014](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P2 | 서명 소환과 파일 수신 상자의 두 봉투·확인증 모델 단일화 또는 ADR | 작업자 실행 추가 전 단계 7 첫 항목군 [OV170-012](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P3 | 호스트 쪽 안내 Skill과 참조 처리기 적합성 시험 | 정식 전 S10과 함께 이 단계에서 처리 |
| P4 | 수신 상자 보존 정리와 1,000개 지연 예산 | S3의 일회 안내·상한은 이 단계, 정리 명령은 [OV170-013](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P5 | 실제 수용 전 실험 기능 표시 | 0.16.0-test.1 출시 노트·local 인계에 "실제 호스트 수용 전" 한계 명시 |

## 2차 재검토

- 판정: REQUEST CHANGES. B1·B2 해소 확인(같은 검토자의 재현 재실행). S2 수정에서 생긴 새 차단 N1 1건, 같은 원인 N2, 안내 누락 N3, 경쟁 응답 코드 N4
- 확인 범위: `8953557f` 빌드 실행 파일로 S1–S9 재현 재실행, 같은 메시지·여러 메시지의 8–20개 동시 소비 프로세스, 흐름 확인증의 `--claim`

| ID | 지적 | 처리 |
| --- | --- | --- |
| N1 | Claude는 한 질문의 모든 Stop에 같은 `prompt_id`, 전달 응답이 계속 Stop에 재생 | 사건 식별에 본문 지문 추가(같은 바이트 재시도만 재생), `stop_hook_active`는 재생 전 중립 응답. `msg_hook_continuation_stops_of_one_prompt_never_replay_a_delivered_result` |
| N2 | 같은 질문의 뒤 Stop이 옛 대기 안내 재생, 새 결과 숨김 | N1 수정으로 해소, 같은 시험에서 새 결과 1회 전달 확인 |
| N3 | 안내의 `cancel-ack` 예시에 필수 `--claim` 없음 | 안내와 복사본 3곳에 `--claim`, `hive stop`의 `claim_digest` 안내 |
| N4 | 상한 1에서 같은 메시지 경쟁 패자가 `worker-limit` 응답 | 세대 게시 경쟁에서 진 경우 승자 기록이 같은 메시지면 `already-claimed`. 남은 경로는 3차 N4′ |
| N5 | `HIVE_TEST_CLOCK=1`은 인증 아닌 환경 스위치 | 유지. [검토 수정](stage-6-review-fixes.md)의 한계와 동일 |

## 3차 재검토

- 판정: APPROVE WITH FIXES. N1·N2·N3 해소 확인(별도 대상 폴더의 새 실행 파일, 계속 Stop 3회 `{}`·같은 질문 뒤 Stop 1회 전달·안내 4곳 지문 동일)
- 남은 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| N4′ | 상한 1·같은 메시지 소비자 8명에서 150회 중 31회 패자가 상한 응답(소비권 승인 전 상한 계산 경로) | 상한 거부 때 이 메시지의 소비권이 이미 있으면 `already-claimed`. `containment_same_message_racers_at_one_worker_lose_the_claim_not_the_limit`(8명·10회, 3회 반복 통과). 상한 초과·이중 승자는 원래 없음 |
| N6 | 같은 질문의 같은 바이트 Stop은 대기 안내 재생, 결과는 다음 다른 Stop에서 1회 전달 | 유지. 같은 바이트 재시도 재생 계약의 결과, 결과 유실 없음 |
