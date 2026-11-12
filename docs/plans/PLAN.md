# Aigent Hive 활성 계획

> Revision: 502
> 기준일: 2026-11-13
> Product version: `0.15.0`
> 공개 Stable: `0.14.2`
> 현재 단계: 0.15.0 구현·검증, 실제 Claude 반환 근거 대기
> 공개 시험: `0.14.2-test.1` 게시, 최종 `0.14.2-test.2` 세 운영체제 수용 완료
> 작업 브랜치: `feature/overhaul-3-0.15.0-orchestrator`

## 목표와 결정

- 현재 요청: OV150-002–010 구현·검증·항목별 커밋·단계 브랜치 push, OV150-011 전 종료
- [소유 계획](active/0.15.0-orchestrator.md)·[9신과 역할 이전](active/0.15.0-gods.md), [전면 위임 원문](../research/overhaul/autonomy-default.md#2026-11-12-위임-확대)
- 0.15.0-test.1 수용: 세 운영체제의 역할 구성 병합·거부, 호스트 제공 출력의 `roster check`, 비활성 summon 미리보기·서명 Prepare, 신 입구·별칭 9개, 0.14.2 갱신. 이 빌드의 조율 비활성·활성화 경로 없음. 실제 Claude Stop→Codex→반환·Gemini 실행·실제 호스트 취소·재점유 미검증, 이 시험의 수용 대상 밖
- OV150-011의 OV150-001 의존성 명시·미완료 유지, 정식 OV150-013은 OV150-001·OV150-014 통과 필요
- [Fable 검토·수정·제안 배치](../research/overhaul/stage-3-review.md), P3 이번 적용·P1 단계 3·P2 단계 6·P4 단계 4
- 공개 안정판 0.14.2 보존, 이번 태그·게시·develop 병합 제외

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 작업 전달 | 5 | 1 | 83.3% |
| 신 입구와 수용 | 4 | 3 | 57.1% |
| 호스트 참조 처리기 | 0 | 1 | 0.0% |
| **현재 범위 합계** | **9** | **5** | **64.3%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [0.15.0-orchestrator.md](active/0.15.0-orchestrator.md) | `OV150-001–006`, `OV150-014` | 작업 전달 |
| [0.15.0-gods.md](active/0.15.0-gods.md) | `OV150-007–013` | 신 입구와 수용 |

| [0.15.0-host-reference.md](active/0.15.0-host-reference.md) | OV150-014 | 호스트 참조 처리기 |

## 이전 근거

- [재정비 순서](overhaul/README.md)
- [0.14.1 보관 계획](../archive/plans/0.14.1-closeout.md)·[미완료 상태](../archive/state/0.14.1-incomplete.md)·[항목](../archive/plans/overhaul-0.14.1.md): OV141-011의 당시 미완료 기록, OV142-008에서 후속 해결 완료
- [완료된 구독자 승인 작업](../archive/plans/subscriber-approval-via-main.md)

- 이전 완료: OV142-001–008 검증·정식·PC 실측, 단계 2 10/10 처리(확인 7·대체 종료 3)·develop 병합·분기 삭제. 공개 안정판 0.14.2 유지
- 단계 3: [OV150-001](../research/overhaul/OV150-001.md) 실제 Codex 처리기 1회·재생 추가 호출 0회. Claude 대화형 왕복은 조율자 근거 대기, OV150-001 미완료
- OV150-002–010 구현 선행, OV150-011 비활성 수용과 실제 왕복의 수용 범위 분리·명시적 의존성 유지. 이번 요청의 OV150-011·태그·게시·develop 병합 제외
- [단계 2 결과·한계](../research/overhaul/stage-2-closeout.md) · [공식 확장점 결정](../decisions/ADR-0026-official-cli-extension-points.md)
