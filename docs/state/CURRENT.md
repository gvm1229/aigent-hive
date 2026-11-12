# 현재 상태

- 제품·공개 안정판: `0.14.2`, [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.2)
- 재정비 단계 1b 완료, [활성 항목](../plans/active/overhaul-0.14.2.md) 8/8. [완료 근거](../research/overhaul/OV142-008.md)
- 공개 수용: test.2 소스 23107415·제품 지문 d79caba5, 세 운영체제 43170228400 success
- 정식: PR #92·main c34b56be, main CI 43185983734·후보 43185988867·복구 게시 43194391907 success. 구독자 안내 검증·전송 단계 성공, 수신자 열람 확인과 구분
- PC: Codex·Windows 11 실제 0.14.1→0.14.2, core/harness/settings verified·감시 0.14.2 running. 큰 사본 5개 자동 정리, bin 2개·prune 후보 0·건너뜀 0
- 선택 질문 usage-enforcement·usage-warnings 2개 보류, 기존 보호 선택·사용자 내용·프로젝트 파일 보존. 이 요청의 완료 조건 밖, 답변 대행 없음
- OV141-011 후속 조건 해결: [보관 기록](../research/overhaul/OV141-011.md), 0.14.1 당시 6개 실패 이력 보존
- 후속 조사: 기존 Windows knowledge_lock 간헐 실패, PR의 예약 작업 조회 시간초과·활성 state.lock 읽기 경합. 코드 변경·원인 해결 주장 없음, 현재 main의 기본 Windows 검사 통과
- 현재: 단계 3 `feature/overhaul-3-0.15.0-orchestrator`, [계획](../plans/overhaul/0.15.0-orchestrator.md). 단계 2 10/10 처리 완료·develop@24057f36 병합·분기 삭제
- [OV150-001](../research/overhaul/OV150-001.md): 합성 Claude 사건 → 호스트 처리기 → 실제 Codex `gpt-6.1-sol`·low·priority 1회, 종료 0·결과·확인증 확인. 같은 사건 재생 추가 호출 0회
- 실행: Codex 0.167.2·Windows 11 Pro 10.0.26300. 경계 자체 시험 8개 통과, 실제 Claude 사건·결과 소비는 미실행. Gemini·다른 운영체제 성공 근거 없음
- 잔여 `awaiting-external-evidence`: 조율자의 실험실 폴더 내 대화형 Claude Code 2.1.163 실행과 훅 반환 관측. [정확한 첫 입력·성공 파일](../research/overhaul/OV150-001.md#claude-첫-입력)
- OV150-001 미완료, 제품 0.14.2 유지. OV150-002 전환·OV150-003 이후 제품 구현은 현재 요청 밖
- 파일 봉투·프로젝트 한정 훅·호스트 소유 처리기: [ADR-0026](../decisions/ADR-0026-official-cli-extension-points.md). 전역 설정·훅·인증 접근·수정 없음, CLI 키 주입·화면 회수·Claude 비대화형 실행 없음
- 전면 위임 승인 적용: 추가 승인 대기 없음. 남은 조율자 대화형 시험의 실제 근거 대기와 권한 대기 구분

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.14.2`
- 현재 등록 항목: 8/8 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->
