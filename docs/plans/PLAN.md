# Aigent Hive 활성 계획

> Revision: 515
> 기준일: 2026-11-20
> Product version: `0.16.0`
> 공개 Stable: `0.15.3`
> 현재 단계: 단계 6 cloud 부분 완료, OV160-012 이 PC 확인 대기
> 공개 시험: `0.16.0-test.3` 게시·세 OS 수용 완료([근거](../research/overhaul/OV160-013-test2.md)), test.2는 Windows 수용 실패
> 작업 브랜치: `develop`(단계 브랜치 병합 완료, 정식 뒤 삭제)

## 목표와 결정

- [소유 계획](active/0.16.0-mediation.md): 파일 수신 상자의 역할 간 메시지, 공식 훅 사건과 비동기 결과 반환, 작성·검토·자문 흐름, 피해 범위 통제, 결정 대기열, Skill 통합
- [시작 승인과 버전 전환](../research/overhaul/OV160-001.md): 재정비 상시 위임과 유지관리자 단계 6 시작 요청
- [실행 장소 규칙](overhaul/venue.md): 실제 호스트·PC 설치·자격 증명 단계는 local 넘기기

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 메시지·작업 중재 | 11 | 2 | 84.6% |
| local 확인 실패 수정 | 0 | 1 | 0.0% |
| **현재 범위 합계** | **11** | **3** | **78.6%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [0.16.0-mediation.md](active/0.16.0-mediation.md) | `OV160-001–013` | 메시지·작업 중재 |
| [0.16.0-local-fixes.md](active/0.16.0-local-fixes.md) | `OV160-014` | local 확인 실패 수정 |

## 이전 근거

- [재정비 순서](overhaul/README.md)·[정식 0.15.3 근거](../research/overhaul/OV153-010.md)
- [이전 활성 계획 색인](../archive/plans/0.15.3-closeout.md)
