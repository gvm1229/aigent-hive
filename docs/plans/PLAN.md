# Aigent Hive 활성 계획

> Revision: 518
> 기준일: 2026-11-28
> Product version: `0.17.0`
> 공개 Stable: `0.16.1`(2026-11-26 출시)
> 현재 단계: 단계 7 0.17.0 구현 시작(cloud 세션)
> 공개 시험: 0.17.0 시험판 없음
> 작업 브랜치: `feature/overhaul-7-0.17.0-overnight-loop`

## 목표와 결정

- [야간 루프](active/0.17.0-overnight-loop.md): 호스트 공식 반복·예약과 비대화형 작업자를 묶는 목표·점수표·예산·정지 조건, 아침 보고서, 0.16.0에서 넘어온 항목
- [신 역할 재배치](active/0.17.0-gods-realign.md): Apollon(지식)⇄Demeter(글쓰기), 웹 디자인 담당 Aphrodite와 동봉 디자인 묶음
- [자율 진행·정리 Skill·브라우저](active/0.17.0-skills-browser.md): `autonomous-workflow` 제품 Skill, `ai-slop-cleaner` 복원, 브라우저 선택, 상류 원본 동봉·주간 동기화
- 시작 승인: 유지관리자 2026-11-28 "yes, I approve of the version 0.17.0". 행동 기준은 `autonomous-workflow`. 정식·main 통합·게시·PC 설치는 0.17.0에 대한 별도 명시 승인 필요
- [실행 장소 규칙](overhaul/venue.md): 실제 호스트·PC 설치·자격 증명 단계는 local

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 야간 루프와 넘어온 항목 | 8 | 11 | 42.1% |
| 신 역할 재배치 | 5 | 0 | 100.0% |
| 자율 진행·정리 Skill·브라우저 | 3 | 1 | 75.0% |
| **현재 범위 합계** | **16** | **12** | **57.1%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [0.17.0-overnight-loop.md](active/0.17.0-overnight-loop.md) | `OV170-001–014`·`OV170-024–027` | 야간 루프와 넘어온 항목 |
| [0.17.0-gods-realign.md](active/0.17.0-gods-realign.md) | `OV170-015–019` | 신 역할 재배치 |
| [0.17.0-skills-browser.md](active/0.17.0-skills-browser.md) | `OV170-020–023` | 자율 진행·정리 Skill·브라우저 |

## 이전 근거

- [재정비 순서](overhaul/README.md)·[0.16.1 계획](active/0.16.1-knowledge-fixes.md)·[정식 0.16.1 근거](../research/overhaul/OV161-007.md)·[정식 0.16.0 근거](../research/overhaul/OV160-011.md)·[단계 5 근거 색인](../research/overhaul/stage-5-closeout.md)
- [이전 활성 계획 색인](../archive/plans/0.16.1-closeout.md)
