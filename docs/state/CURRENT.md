# 현재 상태

- 제품·공개 안정판: `0.14.2`, [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.2)
- 재정비 단계 1b 완료, [활성 항목](../plans/active/overhaul-0.14.2.md) 8/8. [완료 근거](../research/overhaul/OV142-008.md)
- 공개 수용: test.2 소스 23107415·제품 지문 d79caba5, 세 운영체제 43170228400 success
- 정식: PR #92·main c34b56be, main CI 43185983734·후보 43185988867·복구 게시 43194391907 success. 구독자 안내 검증·전송 단계 성공, 수신자 열람 확인과 구분
- PC: Codex·Windows 11 실제 0.14.1→0.14.2, core/harness/settings verified·감시 0.14.2 running. 큰 사본 5개 자동 정리, bin 2개·prune 후보 0·건너뜀 0
- 선택 질문 usage-enforcement·usage-warnings 2개 보류, 기존 보호 선택·사용자 내용·프로젝트 파일 보존. 이 요청의 완료 조건 밖, 답변 대행 없음
- OV141-011 후속 조건 해결: [보관 기록](../research/overhaul/OV141-011.md), 0.14.1 당시 6개 실패 이력 보존
- 후속 조사: 기존 Windows knowledge_lock 간헐 실패, PR의 예약 작업 조회 시간초과·활성 state.lock 읽기 경합. 코드 변경·원인 해결 주장 없음, 현재 main의 기본 Windows 검사 통과
- 현재: 단계 2 `test/overhaul-2-baseline`, [baseline-experiments.md](../plans/overhaul/baseline-experiments.md)의 OVX-001 시작. 제품 0.14.2 유지
- 단계 1b 로컬·원격 브랜치 삭제 완료. [단계 시작 근거](../research/overhaul/stage-2-start.md)
- 단계 2 실험은 2026-11-12 명시 승인 범위. OVX-009 구조 결정과 OVX-010 병합은 이번 작업 제외

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.14.2`
- 현재 등록 항목: 8/8 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->
