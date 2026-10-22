# Aigent Hive 활성 계획

> Revision: 429
> 기준일: 2026-10-23
> Product version: `0.13.2`
> 공개 Stable: `0.13.1`
> 현재 단계: 승인 계획 구현
> 다음 공개 시험: `0.13.2-test.1`

## 현재 요청과 경계

- 앱 버전 대신 실제 통신 호환 검사, 하한0~99%, 최종 전역0%, 초기화·5%·0% 실제 중단 수용
- 사용자 승인:0.13.2 정식 출시·main 통합·로컬 설치, 지정 시험 채팅2개와 실제 초기화 관찰
- 이번 구현 채팅의 일반 보호 임시 예외·독립0% 보호 유지, 최종 설치 뒤 일반 보호 복원과 새 검사
- 앱 유지·지식/설정/기록 보존·같은 사건의 직접 재개. 크레딧 차감0 보장·시험 밖 불필요한 중단·계정 고의 소진 제외
- 작업 브랜치develop. 이전 미추적 조사2개와 완료 출시 근거 보존

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 기능 호환·0% 보호·실제 수용·출시·설치 | 0 | 6 | 0.0% |
| **현재 범위 합계** | **0** | **6** | **0.0%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [usage-compatibility-0.13.2.md](active/usage-compatibility-0.13.2.md) | `UC132-001–006` | 기능 호환·0% 보호·실제 수용·출시·설치 |

## 이전 근거

- [0.13.1 계획 전체](../archive/plans/0.13.1-before-compatibility.md)·[정식 게시](0.13.1-stable-release.md)·[이전 상태](../archive/state/0.13.1-before-compatibility.md)
- 이전 후속9개는 기존 이관 유지, 이번 기능 범위 편입 없음
