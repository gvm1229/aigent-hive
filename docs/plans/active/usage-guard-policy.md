# 전역·프로젝트 사용량 보호 정책 계획

> Checklist owner: `UGP-*`
> Target: `0.9.1` 이후 source 정본 수렴
> Scope: global `usage-guard`, project별 조기 중지 한도, 설치 product 단일 정본

## 결정

- 사용량 보호 설정: global setup의 핵심 기본 항목, 활성화 권장
- 전역 한도: global setup에서 사용자가 선택
- 프로젝트 한도: 등록된 project별 선택 설정, project 유형별 자동값 없음
- 적용 한도: `max(global, project override)`
- 프로젝트 한도: 전역 한도보다 낮은 값 설정 불가
- 전역 보호 비활성화: 모든 project 보호 비활성화
- Custom setup threshold: 사용자가 valid range 안에서 직접 선택
- 신속 기본 profile: 활성화, 남은 사용량 `20%`
- 사용자 호출 Skill: product `usage-guard` 하나
- Aigent Hive source의 pre-task gate: 설치된 `hive usage`가 단일 정본
- Source 전용 Python gate·watcher·Skill·adapter·threshold state: `0건`
- Non-Hive guard 비활성: setup-free Hive Skill 비활성과 무관

## 구현

- [x] [UGP-001] user setup schema·config에 global usage guard enable·user-selected remaining
  threshold와 project override map 추가. global setup의 threshold 선택 필수, project 유형별
  자동값·hard-coded percentage 없음. `1..99` 범위·등록 project ID·unknown project 거부
- [x] [UGP-002] global·project 값을 읽는 단일 effective-policy resolver 구현. 설정값은
  `max(global, project override)`로 계산하며 project가 global safety를 낮추는 경로 0건
- [x] [UGP-003] global setup의 한도 질문, project setup의 inherit·custom threshold 질문,
  `usage-guard` 재설정의 global·project scope preview·apply·validate와 선택 언어 안내 구현.
  숫자는 사용자 입력·기존 설정만 표시하고 web·game 등 profile별 자동 제안 없음
- [x] [UGP-004] 기존 단일 `usage_stop_remaining_percent`를 global 값으로 이관하고 기존
  project는 override 없음으로 보존. 인증 불가·변조 config는 write 0건 conflict
- [x] [UGP-005] source의 mandatory pre-task gate를 repository directive·script로 유지하되,
  source Skill·adapter·user threshold state 생성 금지. 소비자 노출 guard: product
  `usage-guard` 하나. source gate: project 설정·user policy 미생성
- [x] [UGP-006] 서로 다른 임의 global·project 값의 effective threshold, disabled global,
  lower project override 거부, update migration, source·consumer projection, host별 guard와
  Discord payload의 project·effective threshold 회귀 검증. profile별 hard-coded value 0건
- [x] [UGP-007] Custom setup의 사용량 보호 선택을 `활성화 (권장)`·`비활성화`로 변경하고,
  신속 기본 profile을 enabled·remaining `20%`로 변경. 기존 명시적 사용자 선택 보존과
  disable 경로 회귀 추가
- [x] [UGP-008] Setup 설명·preview·summary·schema fixture·saved preference migration을 새
  기본값과 일치시키고 clean install·reconfigure·preserving reinstall 수용
- [x] [UGP-009] Source workspace의 Python gate·15초 watcher·별도 scratch policy를 제거하고,
  repository directive가 설치된 `hive usage`의 session-bound one-shot 보호만 호출. VS Code를
  방해하는 background watcher·tool 경계별 중복 gate `0건`
- [x] [UGP-010] Global과 project threshold mutation을 분리. 명시적 global 변경만 `--user-root`에
  저장하고 이번 사용자 설정 `5%` 적용. Project `harness.toml`이 있는 target의 변경은
  `max(global, project)`로 적용. `hive-source.json` source workspace는 별도 구현 없이 설치 product의
  global threshold 사용. 자체 `AGENTS.md`만 있거나 빈 folder인 non-Hive target은 guard 전체 비활성:
  halt·threshold 변경·session override·runtime file `0건`
- [x] [UGP-011] Non-Hive guard 비활성과 Hive 기능 사용 가능 여부를 분리. `quick-answer`, prompt
  개선, user-root 지식은 project setup·usage preflight 없이 사용. Project state가 필요한 별도
  workflow만 한 번의 활성화 승인과 자동 capability·run bootstrap 소유. 내부 실행 전제를
  usage guard 오류로 노출하는 경로 `0건`
- [x] [UGP-012] Source 전용 Python guard 제거와 함께 CI의 삭제된
  `tests.conformance.test_source_usage_guard` 호출 제거. Linux·macOS·Windows 적합성 작업에서
  stale module 호출 `0건`, exact `develop` CI 재검증

## 완료 기준

- 전역 설정만으로 모든 project의 최소 안전 한도 보장
- project override로 특정 project의 조기 중지 설정 가능
- 동일 session·project에서 매 guard 검사마다 동일 effective threshold 사용
- 사용자에게 global 값, project 값, 실제 적용 값을 구분해 표시
- Source Python gate·watcher·별도 설정 복제 0건, 설치 product `usage-guard`만 활성
