# 현재 상태

- 제품 버전: `0.15.1`
- 공개 안정판·PC 설치: `0.15.0` 유지
- 작업: 단계 4 기존 구현 8/8, 검토 수정 012 완료, [소유 계획](../plans/active/0.15.1-detection.md)
- 현재 요청: [최우선 CLI·업데이트 설명·Discord 정정](../plans/active/0.15.1-human-output.md)의 구현·시험·커밋·단계 브랜치 push. Fable 추가 검토 전 태그·게시·병합 제외
- 구독자 글: 0.15.0 정정 원문 1,333자·0.15.1 미발행 초안 1,043자, 실제 전송 없음. 정정 워크플로는 0.15.1 정식과 함께 main 반영 뒤 정식 직후 실행
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [이전 정식 근거와 한계](../research/overhaul/OV150-013.md) 보존

- [검토 전 구현 근거](../research/overhaul/stage-4-implementation.md)와 [현재 검토 수정](../research/overhaul/stage-4-review.md) 구분. 현재 Rust 1,198개·Python 통과 집계 1,152개, 제외 2·45. 보류 종류 19/20·출력 사실 53/53, 문서 최종 162개 통과·2개 제외. 영어 콘솔 최초 실패의 원인 미규명과 재검증 통과 구분

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.1`
- 현재 등록 항목: 9/17 완료

- `agent-owned`: `OV151-013`, `OV151-014`, `OV151-015`, `OV151-016`, `OV151-017`
- `awaiting-user-authority`: `OV151-008`, `OV151-009`, `OV151-010`
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
