# 현재 상태

- 제품 버전: `0.16.0`
- 공개 안정판·PC 설치: `0.15.3` (2026-11-20). [정식 근거](../research/overhaul/OV153-010.md)
- 작업: 재정비 단계 6 0.16.0 메시지·작업 중재, 브랜치 `feature/overhaul-6-0.16.0-mediation`. [소유 계획](../plans/active/0.16.0-mediation.md)·[시작 근거](../research/overhaul/OV160-001.md)
- 현재 요청(2026-11-20 유지관리자, cloud 세션): 단계 6 시작과 cloud 항목 구현, 단계 검토·develop 병합·0.16.0-test.1 세 OS 수용 뒤 local 단계만 남으면 인계 문서 작성과 종료. 원문 "Start and run overhaul stage 6 (product 0.16.0, message mediation)"
- 현재 위치: OV160-001–008·010 완료, [develop 병합](../research/overhaul/OV160-010.md)(003·004·007의 local 부분은 OV160-012). [수신 상자 `hive msg`](../research/overhaul/OV160-002.md)·[훅 사건·비동기 반환](../research/overhaul/OV160-003.md)·[작성·검토·자문 흐름](../research/overhaul/OV160-004.md)·[피해 범위 통제·`hive stop`](../research/overhaul/OV160-005.md)·[결정 대기열·Discord 발신](../research/overhaul/OV160-006.md)·[Skill 통합](../research/overhaul/OV160-007.md)·[usage-guard 축소·`hive advanced`](../research/overhaul/OV160-008.md). 다음은 0.16.0-test.1(OV160-009)
- [단계 6 검토](../research/overhaul/stage-6-review.md): Fable 5.2 1차 REQUEST CHANGES(B1·B2) → 수정 → 2차 REQUEST CHANGES(N1) → 수정 → 3차 APPROVE WITH FIXES, N4′ 수정 완료
- 계획 조정(2026-11-20, 상시 위임의 권고안): split 항목의 local 부분을 새 OV160-012로 모음([실행 장소](../plans/overhaul/venue.md) 제안 채택). 계획 검사가 local 대기 항목에 의존하는 cloud 항목 완료를 막기 때문. test.1은 develop 전용 워크플로라 병합(010) 뒤로 순서 정정
- 실행 장소: cloud 세션, 실행 가능한 `hive` 없음, 07 사전 확인 생략([실행 장소](../plans/overhaul/venue.md))
- **local 세션 시작점**: 아직 없음. cloud 항목 완료 뒤 인계 문서 작성
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [0.15.3 종료 상태](../archive/state/0.15.3-closeout.md)·[0.15.3 단계 검토](../research/overhaul/stage-5-review.md)·[단계 5·0.15.2 근거 색인](../research/overhaul/stage-5-closeout.md) 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.16.0`
- 현재 등록 항목: 9/12 완료

- `agent-owned`: `OV160-009`
- `awaiting-user-authority`: `OV160-011`
- `awaiting-external-evidence`: `OV160-012`
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
