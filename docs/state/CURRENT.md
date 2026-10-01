# 현재 상태

- 제품 버전: `0.11.1`
- 공개 안정판: `0.11.0`
- 작업 브랜치: `develop`
- 공개 시험 수용: `0.11.1-test.1`
- 다음 시험 대상: `0.11.1-test.2`
- 정본: [활성 계획](../plans/PLAN.md), [스킬 제공 계획](../plans/active/skill-delivery-0.11.1.md), [Claude 설치 계획](../plans/active/claude-user-install-0.11.1.md)
- 이전 완료와 검증: [0.11.0 상태](../archive/state/0.11.0-before-skill-delivery.md)

## 생성된 현재 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.11.1`
- 현재 등록 항목: 8/9 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: `SDP-005`
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 현재 근거

- 0.11.1 정식 출시 승인, [실행 계획](../plans/0.11.1-stable-release.md)의 test.2 준비·수용 진행
- [0.11.1 구독자 공지 초안](../releases/0.11.1.subscriber.ko.md) 문구·기존 Discord 채널 전송 승인, 정식 게시 뒤 발송 예정

- [Claude Windows 설치 검증](../research/claude-user-install-0.11.1.md): 실제 격리 CLI 세 경로 통과, 전체 Rust 964개 통과·4개 제외, 전체 Python 917개 통과·44개 조건 제외

- 0.11.0 Git 태그의 정확한 프로젝트·사용자 원본 보존
- [로컬 전체 검증](../research/skill-delivery-0.11.1.md): Windows Rust 959개 통과·4개 제외, Python 927개 통과·33개 운영체제 조건 제외
- [공개 수용](../research/skill-delivery-public-test-0.11.1.md): 실제 0.11.1-test.1 실행 파일의 Windows x64·Linux musl x64·macOS arm64 네 조건 통과, Codex 등록 응답은 모의
- 실제 Codex 서버·선택 목록·명시 호출·자연어 호출 미검증; 현재 사용자 설치·새 세션의 별도 승인 필요
- Claude Windows 설치 CUI-001–004 구현·검증 완료, 안정판 0.11.0·현재 사용자 설치 유지

## 승인 인계

- Claude 설치 `CUI-001–004` 수용 완료; 기존 `SDP-005`의 실제 환경 수용 승인 대기
- 승인 대상: 정확한 0.11.1-test.1 사용자 설치·갱신과 대상 소비자 프로젝트 갱신; 예상 결과는 동일 원본 자동 정리와 수정본 보존
- AGENTS.md·승인된 계획의 버전별 설치 승인 규칙 적용
- 승인 후 설치·정리 확인과 Codex 완전 재시작, 스킬 목록·명시 호출·자연어 호출의 실제 증거 필요
