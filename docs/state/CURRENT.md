# 현재 상태

- 제품·공개 안정판: `0.14.2`, [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.2)
- 재정비 단계 1b 완료, [활성 항목](../plans/active/overhaul-0.14.2.md) 8/8. [완료 근거](../research/overhaul/OV142-008.md)
- 공개 수용: test.2 소스 23107415·제품 지문 d79caba5, 세 운영체제 43170228400 success
- 정식: PR #92·main c34b56be, main CI 43185983734·후보 43185988867·복구 게시 43194391907 success. 구독자 안내 검증·전송 단계 성공, 수신자 열람 확인과 구분
- PC: Codex·Windows 11 실제 0.14.1→0.14.2, core/harness/settings verified·감시 0.14.2 running. 큰 사본 5개 자동 정리, bin 2개·prune 후보 0·건너뜀 0
- 선택 질문 usage-enforcement·usage-warnings 2개 보류, 기존 보호 선택·사용자 내용·프로젝트 파일 보존. 이 요청의 완료 조건 밖, 답변 대행 없음
- OV141-011 후속 조건 해결: [보관 기록](../research/overhaul/OV141-011.md), 0.14.1 당시 6개 실패 이력 보존
- 후속 조사: 기존 Windows knowledge_lock 간헐 실패, PR의 예약 작업 조회 시간초과·활성 state.lock 읽기 경합. 코드 변경·원인 해결 주장 없음, 현재 main의 기본 Windows 검사 통과
- 현재: 단계 2 `test/overhaul-2-baseline`, [기준선 계획](../plans/overhaul/baseline-experiments.md) 9/10 처리 완료(확인 6·대체 종료 3). 제품 0.14.2 유지
- OVX-001: 30종 기본값 general 100%, 실제 자동 선택값 없음. OVX-002: 정답 상위 다섯 개 적중 22/40, 보류 3/10
- OVX-003: 관련 사실 3/3, 범위 한정. Claims 685개 지문 보존. OVX-004: 프로젝트 파일 81개·내장 Skill 28개·14,527단어·임시 사용자 198,235바이트
- OVX-008: 세 제공자 외부 자동 입력 허용 미확인. OVX-005–007은 공식 확장점 재범위화로 대체 종료, 원래 근거·실제 미실행 보존
- 단계 1b 로컬·원격 브랜치 삭제 완료. [시작 근거](../research/overhaul/stage-2-start.md) · [현재 결과와 해결 경로](../research/overhaul/stage-2-closeout.md)
- 실행 호스트 Codex·Windows 11. Windows psmux 3.3.2, WSL 로그인 셸의 tmux·Windows 연결 Codex/Gemini 경로 확인, Claude 경로 결과 없음, macOS 환경 없음. 모델 호출·새 pane 0개
- 단계 2 검토 보완: [Fable 5.2 판정·수정·확장점 조사](../research/overhaul/stage-2-review.md), 추적 코퍼스 재측정 30/30 general·null, 고정 검색 입력 JSON 보존
- OVX-009: [ADR-0026](../decisions/ADR-0026-official-cli-extension-points.md)의 2026-11-12 승인 반영. 공식 확장점·파일 봉투·훅 연결, Claude 대화형·Codex/Gemini 비대화형 작업자. Hive 직접 모델·프로세스 실행 금지 유지
- 단계 3 첫 검증: 프로젝트 한정 Claude→훅·수신 상자→Codex 왕복. 이번 요청의 실제 제공자 CLI·모델 실행 없음. [반영 계획·검증](../research/overhaul/OVX-009.md)
- OVX-010 develop 병합은 현재 사용자 지시로 실행 제외. 창 제어 제공자 서면 확인은 비차단 별도 조사. 단계 브랜치 보존·push, 단계 전체 완료 주장 제외

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.14.2`
- 현재 등록 항목: 8/8 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->
