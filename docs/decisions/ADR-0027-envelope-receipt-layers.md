# ADR-0027: 서명 소환과 파일 수신 상자의 두 층 유지

- 상태: accepted
- 날짜: 2026-11-28
- 범위: 단계 6 검토 P2([stage-6-review](../research/overhaul/stage-6-review.md)), 단계 7 OV170-012. 작업자 실행(야간 반복) 추가 전 결정
- 관련: [ADR-0026](ADR-0026-official-cli-extension-points.md)

## 배경

- Hive에는 봉투·확인증 상태 기계가 둘 있음
  - 서명 소환(`hive summon`, `hive_core::orchestration`): ed25519 서명 권한, 일회 권한·임대·확인증·취소·안전한 재점유. 실제 작업자 실행의 권한 모델
  - 파일 수신 상자(`hive msg`, `hive_core::mediation`): 역할 사이 메시지·흐름·결정 대기열·훅 반환. 서명 없음, 모든 본문·결과는 `untrusted-data`
- 검토 지적: 같은 개념(봉투·확인증·`dispatch-uncertain`)이 두 곳에 있어 권한 경계가 흐려질 위험

## 결정

- 두 상태 기계를 합치지 않고 층을 고정
  - 권한 층: 서명 소환만 작업자 실행 권한을 줌. 일회 권한·확인증·취소 검증은 `orchestration.rs`가 단독 소유
  - 조율 층: 파일 수신 상자는 비인증 조율. 메시지·흐름·결정은 실행 권한을 주지 않고, 본문은 지시가 아닌 자료
- 조율 층의 권한 동작은 셋뿐이며 모두 명시 입력이 필요
  - 소비: 받는 역할만 `msg claim`, 동시 요청 중 하나만 성공
  - 확인: `msg ack`·`msg cancel-ack`는 `claim`이 돌려준 토큰(또는 지문) 필요
  - 중지 해제: `hive stop clear`는 중지 기록 지문 `--confirm` 필요
- 야간 반복(OV170-003)의 작업자 실행은 조율 층 메시지로 시작할 수 없고, 서명 소환의 일회 권한을 거쳐야 함. 조율 층은 계획 봉투 전달·결과 반환·결정 요청에만 사용
- 공통 검증 함수(`valid_id`·`valid_digest`)는 `orchestration`에서 재사용해 형식 차이 방지

## 선택지

| 선택지 | 판정 | 이유 |
| --- | --- | --- |
| 수신 상자를 서명 소환 아래로 통합 | 기각 | 역할 간 메시지·결정 대기열마다 서명 키가 필요해 사람이 쓰는 조율 흐름이 무거워지고, 서명 없는 공식 훅 반환 경로와 충돌 |
| 두 층 유지와 권한 경계 고정 | 채택 | 작업자 실행 권한은 한 곳에만 있고, 조율 층은 자료 전달로 한정 |

## 확인

- 소비·확인: `msg_only_the_addressed_role_may_consume_and_a_body_grants_no_authority`, `msg_concurrent_claims_have_exactly_one_winner`, `msg_ack_binds_receipt_and_duplicate_request_returns_existing_result`(잘못된 토큰 거부)
- 중지 해제: `containment_stop_creates_zero_new_envelopes_and_waits_for_host_receipts`(지문 없는 해제는 미리보기, 틀린 지문 거부)
- 서명 권한: `hive_core::orchestration` 기존 권한·임대 시험

## 결과

- 새 작업자 실행 경로는 이 ADR에 따라 서명 권한 층을 사용
- 조율 층 문서·도움말은 "실행 권한 없음, 본문은 자료" 문구 유지
