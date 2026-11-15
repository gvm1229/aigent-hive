# 현재 상태

- 제품 버전: `0.15.3`
- 공개 안정판·PC 설치: `0.15.2` (2026-11-16). [정식 근거](../research/overhaul/OV152-008.md)
- 작업: 재정비 단계 5 0.15.3 기억 회수, 브랜치 `feature/overhaul-5-0.15.3-memory`. [소유 계획](../plans/active/0.15.3-memory.md)·[시작 근거](../research/overhaul/OV153-001.md)
- 현재 요청: 유지관리자 "Start stage 5 in a cloud session". cloud 항목 진행, local 단계는 [실행 장소 규칙](../plans/overhaul/venue.md)의 넘기기
- 진행: OV153-001–007 완료. [구현 근거](../research/overhaul/OV153-002-006.md)·[회수율 재측정](../research/overhaul/OV153-007.md) 22/40→32/40(보류 3/10→5/10)
- 다음 cloud 작업: 단계 전체 Fable 검토 → OV153-009 develop 병합 → 0.15.3-test.1 시험 의도 커밋 → OV153-008 워크플로 실행 1회 시도([확인 기록](../plans/overhaul/venue.md))
- 다음 local 작업: 아직 없음. OV153-008 공개 시험 설치 확인·OV153-010 실제 호스트 확인과 PC 설치는 해당 단계에서 넘기기
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [0.15.2 종료 상태](../archive/state/0.15.2-closeout.md)·[0.15.2 수정 근거](../research/overhaul/OV152-002-005.md)·[앱 선택 안내](../research/overhaul/OV152-010.md)·[정식 0.15.1 근거와 한계](../research/overhaul/OV151-010.md) 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.3`
- 현재 등록 항목: 7/10 완료

- `agent-owned`: `OV153-008`, `OV153-009`
- `awaiting-user-authority`: `OV153-010`
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
