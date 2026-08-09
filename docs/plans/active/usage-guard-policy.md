# 전역·프로젝트 사용량 보호 정책 계획

> Checklist owner: `UGP-*`
> Target: `0.9.0`
> Scope: global `usage-guard`, project별 조기 중지 한도, product 우선 Skill 구성

## 결정

- 사용량 보호 설정: global setup의 기본 항목
- 전역 기본 한도: 남은 사용량 `20%`
- 프로젝트 한도: 등록된 project별 선택 설정
- 적용 한도: `max(global, project override)`
- 예시: global `20%`, web `50%`, game `30%` → web `50%`, game `30%`에서 중지
- 프로젝트 한도: 전역 한도보다 낮은 값 설정 불가
- 전역 보호 비활성화: 모든 project 보호 비활성화
- 사용자 호출 Skill: product `usage-guard` 하나
- source의 guard: source directive·internal adapter 유지, 별도 active source Skill projection 제거

## 구현

- [ ] [UGP-001] user setup schema·config에 global usage guard enable·default remaining
  threshold와 project override map 추가, `1..99` 범위·등록 project ID·unknown project 거부
- [ ] [UGP-002] global·project 값을 읽는 단일 effective-policy resolver 구현. 설정값은
  `max(global, project override)`로 계산하며 project가 global safety를 낮추는 경로 0건
- [ ] [UGP-003] global setup의 한도 질문, project setup의 inherit·custom threshold 질문,
  `usage-guard` 재설정의 global·project scope preview·apply·validate와 선택 언어 안내 구현
- [ ] [UGP-004] 기존 단일 `usage_stop_remaining_percent`를 global 값으로 이관하고 기존
  project는 override 없음으로 보존. 인증 불가·변조 config는 write 0건 conflict
- [ ] [UGP-005] non-repo-specific source guard projection을 product `usage-guard`로 통합.
  source guard 실행은 같은 resolver를 쓰는 internal enforcement adapter로만 유지. source
  contributor bootstrap의 product plugin dependency·missing product 안내 추가
- [ ] [UGP-006] global `20%`, web `50%`, game `30%` effective threshold, disabled global,
  lower project override 거부, update migration, source·consumer projection, host별 guard와
  Discord payload의 project·effective threshold 회귀 검증

## 완료 기준

- 전역 설정만으로 모든 project의 최소 안전 한도 보장
- project override로 특정 project의 조기 중지 설정 가능
- 동일 session·project에서 매 guard 검사마다 동일 effective threshold 사용
- 사용자에게 global 값, project 값, 실제 적용 값을 구분해 표시
- source·product Skill 목록의 중복 usage guard 항목 0건
