# Aigent Hive 활성 계획

> Revision: 401
> 기준일: 2026-10-15
> Product version: `0.12.1`
> 공개 Stable: `0.12.0`
> 현재 단계: 앱 유지와 실행 중 세션 중단 구현
> 다음 공개 시험: `0.12.1-test.1`

## 현재 요청과 경계

- 사용자 승인: 0.12.1 수정 버전의 하네스 규칙 구현·검증. 앱 중단 시험 통과 뒤 0.12.1 로컬 설치·상시 감시 승인, 게시 제외
- 작업 브랜치 develop, Windows Codex. 이전 작업의 변경 보존
- [결정](../decisions/steering-preservation.md)·[0.12.0 완료 계획](../archive/plans/0.12.0-complete.md)
- [이전 미완료 9개](backlog/0.12.0-deferred-host-acceptance.md): 기존 후속 이관 유지, 이번 범위 편입 없음

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 초기화 뒤 실행 중단 | 1 | 2 | 33.3% |
| 추가 지시 보존·검증 | 2 | 0 | 100.0% |
| **현재 범위 합계** | **3** | **2** | **60.0%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [reset-stop-0.12.1.md](active/reset-stop-0.12.1.md) | `RST121-001–003` | 초기화 뒤 실행 중단 |
| [steering-0.12.1.md](active/steering-0.12.1.md) | `STR121-001–002` | 추가 지시 보존·검증 |

## 초기화 뒤 중단 조사·계획

- [수정 계획](active/reset-stop-0.12.1.md): 두 대상 종료 확인·원인 조사·계획 작성 완료
- 현재 요청: 계획의 제품 구현·검증. 기존 HCT112-002·003의 Windows 세션 중단 경로 대응, 다른 호스트 후속 유지
