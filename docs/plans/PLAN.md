# Aigent Hive 활성 계획

> Revision: 492
> 기준일: 2026-11-10
> Product version: `0.14.1`
> 공개 Stable: `0.14.0`
> 현재 단계: 재정비 단계 1, 0.14.1 수정판 구현
> 공개 시험: `0.14.1-test.1` 수용(2026-11-11)
> 작업 브랜치: `fix/overhaul-1-0.14.1`([재정비 브랜치 규칙](overhaul/README.md#브랜치-규칙))

## 목표와 결정

- 2026-11-10 사용자 승인: 재정비 단계의 버전 시작·push·정식 출시를 사전 승인, 항목별 재승인 불필요
- 범위: 사용량 임계값 기록 오류, 공통 지식 연결 상태, 감시 실행 파일 보관 2개 제한과 미완료 갱신 복구, 저장 용량 보고와 정리 미리보기, README 출시일·표시
- 새 제품 바이트는 0.14.1과 `0.14.1-test.1`부터. 시험은 `tests/work/`의 임시 사용자·설치만 사용
- 0.14.0 직전 정식판의 프로젝트·사용자 원본을 공개 태그 바이트로 고정하고 갱신 인증에 등록

## Completion index

현재 집계는 0.14.1 수정판의 등록 항목 기준. 계획 문서 완료와 제품 완료의 구분.

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 0.14.1 수정판 | 10 | 1 | 90.9% |
| 제품 버전과 별개인 구독자 승인 자동화 | 3 | 0 | 100.0% |
| **현재 범위 합계** | **13** | **1** | **92.9%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [overhaul-0.14.1.md](active/overhaul-0.14.1.md) | `OV141-001–011` | 0.14.1 수정판 |
| [subscriber-approval-via-main.md](active/subscriber-approval-via-main.md) | `SMAIN-001–003` | 제품 버전과 별개인 구독자 승인 자동화 |

## 결정과 이전 근거

- [Hive 재정비 할 일](overhaul/README.md): 단계 0–7 체크 목록, 재개 절차, 단계별 전용 브랜치 규칙
- [0.14.0 완료 계획](../archive/plans/0.14.0-complete.md)·[완료 상태](../archive/state/0.14.0-complete.md) 보존
- 별도 조사: [프로젝트 지식 읽기·쓰기와 전역 읽기 전용](backlog/project-local-global-readonly.md). 0.14.1 범위 밖
- [공통 연결 작업](backlog/knowledge-attachment-consistency.md)·[임계값 확인 기록 오류](backlog/usage-threshold-provenance.md): 0.14.1 `OV141-002`–`004`의 근거
- [0.14.0 결정](../decisions/interactive-cli-0.14.0.md)·[조사 근거](../research/interactive-cli-0.14.0.md)
