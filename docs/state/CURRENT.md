# 현재 상태

- 제품 버전: `0.16.0`
- 공개 안정판: `0.16.0`(2026-11-25 출시). PC 설치: `0.16.0`(2026-11-25 갱신, 자료 보존 확인)
- 작업: 재정비 단계 6 0.16.0 메시지·작업 중재, 브랜치 `feature/overhaul-6-0.16.0-mediation`. [소유 계획](../plans/active/0.16.0-mediation.md)·[시작 근거](../research/overhaul/OV160-001.md)
- 현재 요청(2026-11-23 유지관리자, local 세션): 이 세션만 사용량 보호를 끄고 정식 0.16.0까지 자율 진행. [권한과 범위](../research/overhaul/OV160-resume.md)
- 현재 위치: OV160-001–010·013 완료. **OV160-012 test.3 local 확인 실패**([근거](../research/overhaul/OV160-012.md)): 0.15.x에서 `hive update` 인증 실패(Claude·Antigravity), 보존 재설치의 `projects.yml` 삭제, 옛 Skill 이름 안내 없음, 교차 검토 기본 공급자. 사용량 상태 표시줄(터미널)·Claude 훅 왕복·5초 측정은 통과. 정식 0.16.0 보류. 다음은 [0.16.0 재계획](../research/overhaul/OV160-replan.md)(Fable 5.2 2차 APPROVE WITH FIXES 반영) OV160-014–021과 test.4
- [단계 6 검토](../research/overhaul/stage-6-review.md): Fable 5.2 1차 REQUEST CHANGES(B1·B2) → 수정 → 2차 REQUEST CHANGES(N1) → 수정 → 3차 APPROVE WITH FIXES, N4′ 수정 완료
- 계획 조정(2026-11-20, 상시 위임의 권고안): split 항목의 local 부분을 새 OV160-012로 모음([실행 장소](../plans/overhaul/venue.md) 제안 채택). 계획 검사가 local 대기 항목에 의존하는 cloud 항목 완료를 막기 때문. test.1은 develop 전용 워크플로라 병합(010) 뒤로 순서 정정
- 실행 장소: 2026-11-21 local 세션(이 PC, 0.16.0-test.3 설치, 07 사전 확인 실행). 다음 수정은 cloud
- **다음 시작점**: 단계 6 종료. 정식 0.16.0 출시 완료([근거](../research/overhaul/OV160-011.md)). 다음은 [재정비 순서](../plans/overhaul/README.md)의 단계 7
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [0.15.3 종료 상태](../archive/state/0.15.3-closeout.md)·[0.15.3 단계 검토](../research/overhaul/stage-5-review.md)·[단계 5·0.15.2 근거 색인](../research/overhaul/stage-5-closeout.md) 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.16.0`
- 현재 등록 항목: 27/27 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
- `reopened`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
