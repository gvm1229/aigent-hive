# 단계 6 검토 수정 — 0.16.0 메시지 중계

- 기준일: 2026-11-20, 대상: `81b7e0ee`(OV160-005)까지의 단계 6 변경
- 검토: Fable 5.2 단계 검토의 B1·B2·S1–S9·O2·O5 지적. 이 문서는 수정과 시험 근거
- 실제 호스트·제공자 호출 0회. 가짜 사건·임시 소비자 폴더만 사용
- 같은 시기의 다른 작업(Poseidon 진입 Skill, 중계 참고 문서, `hive-core` `decision.rs`의 영어 알림)과 파일 겹침 없음

## 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| B1 | Claude 기본 Stop 반환(`StopContext`)이 결과 없이 대기·미전달 ID만 있어도 `additionalContext`를 보내 Stop마다 모델 턴 추가 | 넘긴 결과가 없으면 `StopBlock`과 같이 `systemMessage`만 반환. `hook_events_render_each_official_return_shape`, `msg_hook_claude_stop_without_a_delivered_result_never_continues_the_turn`, 프로세스 `msg_hook_process_exits_zero_with_one_json_document_and_no_stderr` |
| B2 | `msg status`·`msg inbox`·훅 전달이 결과 파일과 확인증의 `result_digest`·`result_bytes` 대조 없음 | 상태 계산(`derive_state`)에 결과 바이트 추가, `MessageReceipt::verify_result`로 지문·크기 대조. 불일치·누락은 `quarantined`, 훅은 `load`가 검증한 바이트만 전달. `mediation_state_derivation_covers_loss_and_quarantine`, `msg_hook_withholds_a_result_overwritten_after_its_receipt` |
| S1 | `--now`가 운영 명령의 무인증 시계 변경, 기한 뒤 확인증을 옛 시각으로 `acknowledged` 처리 가능 | `--now`는 `HIVE_TEST_CLOCK=1`(기존 `HIVE_TEST_*` 관례)일 때만 수락, 아니면 `hive.msg-clock-refused`. `MessageReceipt::validate`에 `acknowledged_at <= lease_until` 추가. 이 crate의 단위 시험은 `cfg!(test)`로 허용, 프로세스 시험은 환경 변수를 명시. `msg_now_is_a_test_only_clock`, `msg_late_ack_cannot_be_backdated_with_now` |
| S2 | 사건 ID가 입력 지문이라 같은 `UserPromptSubmit` 두 번째가 낡은 응답을 재생하고 새 결과를 숨김 | `prompt_id`가 있으면 그 지문으로 사건 ID, 같은 턴 재시도만 재생. 없으면 호출마다 새 ID로 응답 다시 계산(전달은 메시지당 1회 유지). 사건 기록에 `turn_digest`. `msg_hook_same_prompt_twice_delivers_a_newly_acknowledged_result_once`, `msg_hook_duplicate_and_recursive_stop_record_once_without_dispatch` |
| S3 | 훅마다 대기·미전달 ID 전부를 무제한 반복 | 미전달은 상태마다 1회 안내(`announcements/<메시지 ID>-<상태>.json`, 덮어쓰기 없음). 대기·미전달 각각 10개까지 나열하고 나머지는 "and N more". `msg_hook_lists_withheld_once_and_caps_listed_ids` |
| S4 | `max_workers`가 세고 나서 게시하는 방식이라 동시 소비 N건이 N-1개 초과 가능 | 소비권 게시 전에 `slots/<칸>/<세대>.json`을 덮어쓰기 없이 예약. 칸의 최신 세대 보유자가 끝났을 때만(다른 토큰의 소비권·종료·불확실 상태·기한 지난 미게시 예약) 다음 세대 게시. 삭제 없음, 재시작 안전. `msg_concurrent_processes_respect_max_workers`(프로세스 8개·`max_workers = 1`·메시지 2개 → 소비권 1개), `containment_released_worker_slots_are_reused_by_generation` |
| S5 | `msg ack`가 메시지 ID만 요구, `claim_token` 불필요 | `ack`·`cancel-ack`에 `--claim <소비권 토큰 또는 지문>` 필수. 틀리면 `hive.msg-claim-mismatch`, 결과·확인증·격리 파일 0개. `msg claim` 결과에 `claim_digest` 추가, 사용법·사람용 코드·문서 갱신. `msg_ack_binds_receipt_and_duplicate_request_returns_existing_result`, `containment_stop_creates_zero_new_envelopes_and_waits_for_host_receipts` |
| S6 | 읽을 때 보낸 사람을 현재 역할 설정으로 다시 해석해 모델·노력 변경 뒤 확인된 메시지까지 격리 | 보낸 사람은 보낼 때만 확인, 읽을 때는 봉인된 봉투 지문·본문·키 기록만 확인. 흐름의 `hive.msg-flow-profile-changed`는 유지. `msg_roster_change_after_send_or_ack_keeps_the_sealed_message_state` |
| S7 | Gemini가 미검증인데 `msg hooks apply --host gemini`가 `.gemini/settings.json`을 쓰고, 단위 미확인 `timeout: 10` 기록 | 사건이 모두 미검증인 호스트는 `apply`를 `hive.msg-hooks-unverified-host`로 거부, 미리보기만 허용. Gemini 항목은 `timeout` 없음, Claude·Codex는 공식 문서대로 초 단위 10. `msg_hook_return_shapes_follow_each_host`, `msg_hooks_preview_apply_requires_exact_digest_and_stays_in_the_project` |
| S8 | 소비권·확인증·취소 기록 하나가 일반 파일이 아니면 `msg inbox` 전체 실패, 훅은 모든 메시지에 `{}` | 메시지별 기록 읽기 실패는 그 메시지만 `quarantined`, 나머지 계속. 훅은 읽을 수 없는 메시지를 건너뜀. `msg_inbox_quarantines_only_the_message_with_a_symlinked_claim`, `msg_hook_one_symlinked_claim_does_not_silence_other_messages` |
| S9 | `.hive/mediation/`이 작업자 쓰기 루트 안에 있고 git 제외 아님 | 템플릿 `.hive/.gitignore`에 `/mediation/`(기존 `/runtime/`과 같은 실행 기록 관례). 작업자 정책에 `excluded_roots: ["{target}/.hive"]` 추가, 검증이 고정값 요구. 템플릿은 `hive-render`가 직접 포함하고 투영 대상 아님, 0.16.0 출시 표면·이전 표는 템플릿 바이트를 기록하지 않아 변경 없음. `containment_hive_state_is_excluded_from_the_worker_and_git`, `containment_policy_is_least_privilege_for_every_default_role` |
| O2 | 끝난 메시지의 중복 보내기에 다음 행동 없음 | 기존 메시지가 `acknowledged`·`expired`·`quarantined`·`cancelled`·`dispatch-uncertain`이면 `next_action`에 새 `--key` 안내. 대기 중 중복은 안내 없음. `msg_duplicate_of_an_expired_message_asks_for_a_new_key`, `msg_duplicate_send_returns_one_message_and_conflicting_key_is_refused` |
| O5 | 증거 문서와 구현 불일치 | [OV160-003](OV160-003.md)에 B1·S2·S3·S7·S8 변경과 새 기본 반환, [OV160-005](OV160-005.md)의 "상한을 하나 넘을 수 있음"을 S4 칸 예약으로 교체, [OV160-002](OV160-002.md)에 소환 봉투는 `.hive/runs/*/inbox/`에 남고 `msg inbox`가 함께 나열할 뿐이라는 점과 `orchestration.rs`에서 ID·지문 검사 함수만 재사용한다는 점 기록. [OV160-004](OV160-004.md)의 "호출자 시계 또는 `--now`" 문구는 계획이 그 문서 지문을 고정하고 있어 이번 수정 범위 밖, S1 뒤에는 시험용 시계로 해석 |

## 바뀐 계약 요약

- 훅 반환: 결과가 있을 때만 대화를 잇는 형식(`additionalContext`·`decision: block`), 그 밖에는 `systemMessage` 또는 `{}`
- 사건 ID: `prompt_id` 기준 재생, 없으면 호출마다 새 사건
- 상태: 확인증과 결과 파일의 지문·크기가 다르면 `quarantined`
- 시계: `--now`는 시험 전용, 확인증 시각은 소비 기한 이하
- 소비권: 실행 칸 예약 뒤 게시, `ack`·`cancel-ack`는 `--claim` 필수
- 공유 파일: `msg.rs`·`cli_help.rs`(`--claim` 선택지)·`human_output/codes.rs`(새 코드 3개)

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test -p hive-core --locked` | 200 통과 |
| `cargo test -p hive-cli --locked --bin hive msg` | 49 통과(새 단위 9) |
| `cargo test -p hive-cli --locked --test msg_inbox` | 5 통과(새 프로세스 2) |
| `cargo test -p hive-cli --locked --bin hive -- hook summon stop human_output cli_help` | 99 통과, 도움말 스냅숏 변화 없음 |
| `cargo test -p hive-cli --locked --test cli_help --test human_output --test historical_project_upgrade` | 9·10·6 통과 |
| `cargo test -p hive-render -p hive-projection -p hive-update --locked` | 96·46·65 통과 |
| `HIVE_BIN=<빌드한 hive> python -m unittest tests.conformance.contracts.test_roster_gods tests.conformance.contracts.test_action_result_contracts tests.conformance.contracts.test_hive_skills` | 27 통과 |
| `python scripts/generate-gods.py --check` | 67 일치 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 경고 0, Rust 1.100 |
| `cargo fmt --all --check` | 통과 |
| `python scripts/test-lanes.py --lane documentation` | 164개 중 통과 161·건너뜀 2·오류 1. 오류는 `test_registered_plan_and_generated_views_are_current`의 낡은 완료 증거 지문: 계획 파일이 OV160-002·003·005 문서 지문을 고정하는데 O5가 그 문서를 고침. 계획 파일 갱신은 이 수정 범위 밖 |
| `check-markdown-links.py`·`check-human-documentation-style.py --all` | 링크 실패 0, 문체 실패 0. 바뀐 문서의 새 지적 0 |

## 한계

- `--now`를 받지 않아도 시스템 시계 자체의 조작은 막지 못함. 같은 사용자의 악의적 파일 변조는 여전히 증명 밖
- 소비권을 게시하지 못하고 멈춘 칸 예약은 그 소비 기한까지 칸 점유. `max_workers`를 줄이면 줄어든 번호 밖의 칸은 보지 않고, 기존 소비권 수 확인이 함께 상한을 지킴
- `prompt_id`가 없는 호스트에서 같은 턴의 진짜 재시도는 새 사건이 되어, 처음 응답을 잃었다면 이미 넘긴 결과는 다시 받지 못함. 결과는 `results/`와 `msg status`에 그대로 있음
- `excluded_roots`는 호스트가 지켜야 하는 정책 자료. Codex `workspace-write`의 실제 경로 제외 방식은 local 확인 대상
