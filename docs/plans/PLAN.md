# Aigent Hive 활성 계획

> Revision: 500
> 기준일: 2026-11-13
> Product version: `0.15.0`
> 공개 Stable: `0.14.2`
> 현재 단계: 재정비 단계 3 시작, OV150-001 처리기 실측·Claude 왕복 대기
> 공개 시험: `0.14.2-test.1` 게시, 최종 `0.14.2-test.2` 세 운영체제 수용 완료
> 작업 브랜치: `feature/overhaul-3-0.15.0-orchestrator`

## 목표와 결정

- 2026-11-12 승인: 단계 2 전에 0.14.2 추가. 전달된 검토 보완부터 정식·PC 갱신까지 승인
- 범위: 큰 파일의 정확 삭제·회귀·출시 안내·로컬 검사. 이 PC 제품 갱신·제품 자체 정리 포함
- [단계 결정](../decisions/monitor-cleanup-0.14.2.md)

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 작업 전달 | 4 | 2 | 66.7% |
| 신 입구와 수용 | 0 | 7 | 0.0% |
| **현재 범위 합계** | **4** | **9** | **30.8%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [0.15.0-orchestrator.md](active/0.15.0-orchestrator.md) | `OV150-001–006` | 작업 전달 |
| [0.15.0-gods.md](active/0.15.0-gods.md) | `OV150-007–013` | 신 입구와 수용 |

## 이전 근거

- [재정비 순서](overhaul/README.md)
- [0.14.1 보관 계획](../archive/plans/0.14.1-closeout.md)·[미완료 상태](../archive/state/0.14.1-incomplete.md)·[항목](../archive/plans/overhaul-0.14.1.md): OV141-011의 당시 미완료 기록, OV142-008에서 후속 해결 완료
- [완료된 구독자 승인 작업](../archive/plans/subscriber-approval-via-main.md)

- 완료: OV142-001–008 검증·정식·PC 실측. 단계 2 별도 계획 10/10 처리(확인 7·대체 종료 3), develop 병합·분기 삭제. 제품 0.14.2 유지
- 단계 3: [OV150-001](../research/overhaul/OV150-001.md) 실제 Codex 처리기 1회·재생 추가 호출 0회. Claude 대화형 왕복은 조율자 근거 대기, OV150-001 미완료
- OV150-002–010 구현 선행, 실제 왕복 수용은 OV150-011 전에 필수. 이번 요청의 OV150-011·태그·게시·develop 병합 제외
- [단계 2 결과·한계](../research/overhaul/stage-2-closeout.md) · [공식 확장점 결정](../decisions/ADR-0026-official-cli-extension-points.md)
