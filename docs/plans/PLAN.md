# Aigent Hive 활성 계획

> Revision: 497
> 기준일: 2026-11-12
> Product version: `0.14.2`
> 공개 Stable: `0.14.2`
> 현재 단계: 재정비 단계 2 기준선 5/10 완료, 창 실험 차단
> 공개 시험: `0.14.2-test.1` 게시, 최종 `0.14.2-test.2` 세 운영체제 수용 완료
> 작업 브랜치: `test/overhaul-2-baseline`

## 목표와 결정

- 2026-11-12 승인: 단계 2 전에 0.14.2 추가. 전달된 검토 보완부터 정식·PC 갱신까지 승인
- 범위: 큰 파일의 정확 삭제·회귀·출시 안내·로컬 검사. 이 PC 제품 갱신·제품 자체 정리 포함
- [단계 결정](../decisions/monitor-cleanup-0.14.2.md)

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 0.14.2 수정판 | 8 | 0 | 100.0% |
| **현재 범위 합계** | **8** | **0** | **100.0%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [overhaul-0.14.2.md](active/overhaul-0.14.2.md) | `OV142-001–008` | 0.14.2 수정판 |

## 이전 근거

- [재정비 순서](overhaul/README.md)
- [0.14.1 보관 계획](../archive/plans/0.14.1-closeout.md)·[미완료 상태](../archive/state/0.14.1-incomplete.md)·[항목](../archive/plans/overhaul-0.14.1.md): OV141-011의 당시 미완료 기록, OV142-008에서 후속 해결 완료
- [완료된 구독자 승인 작업](../archive/plans/subscriber-approval-via-main.md)

- 완료: OV142-001–008 검증·정식·PC 실측. 단계 2는 [별도 source 계획](overhaul/baseline-experiments.md) 5/10, 제품 0.14.2 유지
- 단계 2: OVX-001·002·003·004·008 완료, OVX-005 부분 확인·실험 차단, OVX-006·007 차단. OVX-009·010은 사용자 지정 중단 경계
- [단계 2 결과·검증 한계](../research/overhaul/stage-2-closeout.md). 위 자동 집계는 기존 제품 계획 8개, 별도 단계 2 집계와 구분
