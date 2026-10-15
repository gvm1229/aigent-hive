# Aigent Hive 활성 계획

> Revision: 404
> 기준일: 2026-10-16
> Product version: `0.12.1`
> 공개 Stable: `0.12.0`
> 현재 단계: 첫 차수 세션 중단·60초 관측 공백 보호의 오프라인 수정
> 다음 공개 시험: `0.12.1-test.1`

## 현재 요청과 경계

- 사용자 범위: 0.12.1 첫 차수는 실제 초기화 감지 뒤 진행 중 세션 중단만 구현. 60초 연속 정상 조회 부재도 보호 중단, 앱 전체 종료 금지
- 현재 앱 제어·실제 시험·설치 비활성 유지. 나머지 예정 기능은 같은 버전의 후속 차수로 이관
- 작업 브랜치 develop, Windows Codex. 이전 작업의 변경 보존
- [결정](../decisions/steering-preservation.md)·[0.12.0 완료 계획](../archive/plans/0.12.0-complete.md)
- [이전 미완료 9개](backlog/0.12.0-deferred-host-acceptance.md): 기존 후속 이관 유지, 이번 범위 편입 없음

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 초기화 뒤 실행 중단 | 0 | 3 | 0.0% |
| **현재 범위 합계** | **0** | **3** | **0.0%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [reset-stop-0.12.1.md](active/reset-stop-0.12.1.md) | `RST121-001–003` | 초기화 뒤 실행 중단 |

## 초기화 뒤 중단 조사·계획

- [수정 계획](active/reset-stop-0.12.1.md): 두 대상 종료 확인·원인 조사·계획 작성 완료
- 현재 요청: 계획의 첫 차수 제품 구현·모의 검증. 기존 HCT112-002·003의 Windows 세션 중단 경로 대응, 다른 호스트 후속 유지
- [추가 지시 보존](backlog/steering-0.12.1.md): 이전 구현 근거 보존, 첫 차수 공개 범위 제외
