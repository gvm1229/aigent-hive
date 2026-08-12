# 전역·프로젝트 사용량 보호 정책 계획

> Checklist owner: `UGP-*`
> Target: `0.9.0`
> Scope: global `usage-guard`, project별 조기 중지 한도, 단일 product Skill

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
- Aigent Hive source의 pre-task gate: repository directive가 소유. 설치 product `usage-guard`와
  별개 user policy 미생성
- Source 전용 Skill·adapter·user threshold state: `0건`

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

## 완료 기준

- 전역 설정만으로 모든 project의 최소 안전 한도 보장
- project override로 특정 project의 조기 중지 설정 가능
- 동일 session·project에서 매 guard 검사마다 동일 effective threshold 사용
- 사용자에게 global 값, project 값, 실제 적용 값을 구분해 표시
- Source Skill·adapter·설정 복제 0건, product `usage-guard`만 활성
