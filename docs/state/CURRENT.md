# 현재 상태

- 제품 버전: `0.15.3`
- 공개 안정판·PC 설치: `0.15.2` (2026-11-16). [정식 근거](../research/overhaul/OV152-008.md)
- 작업: 재정비 단계 5 0.15.3 기억 회수, 브랜치 `feature/overhaul-5-0.15.3-memory`. [소유 계획](../plans/active/0.15.3-memory.md)·[시작 근거](../research/overhaul/OV153-001.md)
- 현재 요청: 유지관리자 "Start stage 5 in a cloud session". cloud 항목 진행, local 단계는 [실행 장소 규칙](../plans/overhaul/venue.md)의 넘기기
- 진행: OV153-001–007·009 완료. [구현 근거](../research/overhaul/OV153-002-006.md)·[회수율 재측정](../research/overhaul/OV153-007.md) 22/40→32/40(보류 3/10→5/10)·[단계 검토](../research/overhaul/stage-5-review.md)·[develop 병합](../research/overhaul/OV153-009.md)
- 공개 시험 0.15.3-test.1(OV153-008) 완료: npm test 태그·GitHub 시험판·세 OS 수용 통과. [공개 시험 근거](../research/overhaul/OV153-008.md). 워크플로 실행은 cloud 가능([확인 기록](../plans/overhaul/venue.md))
- **정식 0.15.3 보류**(2026-11-17 유지관리자): [에이전트 주도 프로젝트 설정 OV153-011](../research/overhaul/OV153-011-request.md) 포함 전 출시 금지. 시험판 재수용·이 PC 확인 뒤 출시
- **다음 local 작업: [OV153-010 이 PC 실제 호스트 확인과 정식 0.15.3](../research/overhaul/OV153-010-local.md)** — 0.15.2에서 test.1 갱신, Codex·Claude·Antigravity 표시, 실제 호스트의 묶음 기록·주제 회수 왕복, 통과 뒤 정식 게시·PC 설치. cloud 남은 작업 없음
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [0.15.2 종료 상태](../archive/state/0.15.2-closeout.md)·[0.15.2 수정 근거](../research/overhaul/OV152-002-005.md)·[앱 선택 안내](../research/overhaul/OV152-010.md)·[정식 0.15.1 근거와 한계](../research/overhaul/OV151-010.md) 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.3`
- 현재 등록 항목: 9/11 완료

- `agent-owned`: `OV153-011`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: `OV153-010`
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
