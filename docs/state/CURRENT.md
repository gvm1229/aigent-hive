# 현재 상태

- 제품 버전: `0.15.3`
- 공개 안정판·PC 설치: `0.15.2` (2026-11-16). [정식 근거](../research/overhaul/OV152-008.md)
- 작업: 재정비 단계 5 0.15.3 기억 회수, 브랜치 `feature/overhaul-5-0.15.3-memory`. [소유 계획](../plans/active/0.15.3-memory.md)·[시작 근거](../research/overhaul/OV153-001.md)
- 현재 요청: 유지관리자 "Start stage 5 in a cloud session". cloud 항목 진행, local 단계는 [실행 장소 규칙](../plans/overhaul/venue.md)의 넘기기
- 진행: OV153-001–007·009 완료. [구현 근거](../research/overhaul/OV153-002-006.md)·[회수율 재측정](../research/overhaul/OV153-007.md) 22/40→32/40(보류 3/10→5/10)·[단계 검토](../research/overhaul/stage-5-review.md)·[develop 병합](../research/overhaul/OV153-009.md)
- 공개 시험 0.15.3-test.1(OV153-008) 완료. [공개 시험 근거](../research/overhaul/OV153-008.md). 워크플로 실행은 cloud 가능([확인 기록](../plans/overhaul/venue.md))
- **cloud 부분 완료**(2026-11-17): [OV153-011](../research/overhaul/OV153-011.md)(추가 범위 A1–A4 포함)·[OV153-012](../research/overhaul/OV153-012.md)·[OV153-013](../research/overhaul/OV153-013.md) 구현·독립 검토·develop 병합, CI 통과. [0.15.3-test.2](../research/overhaul/OV153-011-test2.md) 게시·세 OS 수용 통과. 출시 노트·구독자 안내·정식 공개 문서 준비 완료
- **정식 0.15.3 보류**(2026-11-17 유지관리자): [에이전트 주도 프로젝트 설정 OV153-011](../research/overhaul/OV153-011-request.md) 포함 전 출시 금지. 시험판 재수용·이 PC 확인 뒤 출시
- **0.15.3 범위 추가**(2026-11-17 유지관리자): [대화형 설정 화면 가독성 OV153-012](../research/overhaul/OV153-012-request.md). 기능·문구 유지, 질문 구분·제목·구역·꾸밈 추가
- **cloud 위임**(2026-11-17): OV153-011 구현부터 test.2 수용·문서 준비까지 cloud 세션 담당, local 단계만 남으면 종료. [위임 순서](../research/overhaul/OV153-cloud-mandate.md)
- **local 세션 시작점**: [0.15.3 local 인계](../research/overhaul/OV153-local-handoff.md). test.2 설치, 실제 Claude·Codex 세션의 Skill 설정(빈 폴더·기존 프로젝트), A4 재현, OV153-012 실제 터미널, OV153-013 재현, 통과 뒤 정식 OV153-010. cloud 남은 작업 없음
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [0.15.2 종료 상태](../archive/state/0.15.2-closeout.md)·[0.15.2 수정 근거](../research/overhaul/OV152-002-005.md)·[앱 선택 안내](../research/overhaul/OV152-010.md)·[정식 0.15.1 근거와 한계](../research/overhaul/OV151-010.md) 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.3`
- 현재 등록 항목: 9/13 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: `OV153-011`, `OV153-012`, `OV153-013`, `OV153-010`
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
