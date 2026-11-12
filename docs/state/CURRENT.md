# 현재 상태

- 현재: Windows Claude 왕복·참조 처리기 완료, 조율 비활성·활성화 경로 없음. 첫 CI의 Rust 1.100 지적 보완·Windows 1,434개 통과, 새 CI·공개 시험·정식·설치 진행
- 아래 선행 구현·검토 기록의 당시 011 전 종료·반환 미검증 경계는 현재 요청과 실증으로 대체


- 제품 버전: `0.15.0`
- 공개 안정판: `0.14.2`, [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.2)
- 재정비 단계 1b 완료, [활성 항목](../plans/active/overhaul-0.14.2.md) 8/8. [완료 근거](../research/overhaul/OV142-008.md)
- 공개 수용: test.2 소스 23107415·제품 지문 d79caba5, 세 운영체제 43170228400 success
- 정식: PR #92·main c34b56be, main CI 43185983734·후보 43185988867·복구 게시 43194391907 success. 구독자 안내 검증·전송 단계 성공, 수신자 열람 확인과 구분
- PC: Codex·Windows 11 실제 0.14.1→0.14.2, core/harness/settings verified·감시 0.14.2 running. 큰 사본 5개 자동 정리, bin 2개·prune 후보 0·건너뜀 0
- 선택 질문 usage-enforcement·usage-warnings 2개 보류, 기존 보호 선택·사용자 내용·프로젝트 파일 보존. 이 요청의 완료 조건 밖, 답변 대행 없음
- OV141-011 후속 조건 해결: [보관 기록](../research/overhaul/OV141-011.md), 0.14.1 당시 6개 실패 이력 보존
- 후속 조사: 기존 Windows knowledge_lock 간헐 실패, PR의 예약 작업 조회 시간초과·활성 state.lock 읽기 경합. 코드 변경·원인 해결 주장 없음, 현재 main의 기본 Windows 검사 통과
- 현재: 단계 3 `feature/overhaul-3-0.15.0-orchestrator`, [계획](../plans/active/0.15.0-orchestrator.md). 단계 2 10/10 처리 완료·develop@24057f36 병합·분기 삭제
- [OV150-001](../research/overhaul/OV150-001.md): 합성 Claude 사건 → 호스트 처리기 → 실제 Codex `gpt-6.1-sol`·low·priority 1회, 종료 0·결과·확인증 확인. 같은 사건 재생 추가 호출 0회
- 실행: Codex 0.167.2·Windows 11 Pro 10.0.26300. 경계 자체 시험 10개 통과, 실제 Claude 입력 훅 확인·Stop 반환 미검증. Gemini·다른 운영체제 성공 근거 없음
- 잔여 `awaiting-external-evidence`: 사용자의 실험실 내 구독 인증 Claude Code 데스크톱 세션에서 첫 입력과 Stop 반환 관측. [정확한 첫 입력·성공 파일](../research/overhaul/OV150-001.md#claude-첫-입력)
- OV150-001 미완료·기본 비활성 유지. 선행 요청의 OV150-002–010 구현·검증 완료, OV150-011 전 종료
- 파일 봉투·프로젝트 한정 훅·호스트 소유 처리기: [ADR-0026](../decisions/ADR-0026-official-cli-extension-points.md). 전역 설정·훅·인증 접근·수정 없음, CLI 키 주입·화면 회수·Claude 비대화형 실행 없음
- 전면 위임 승인 적용: 추가 승인 대기 없음. 남은 조율자 대화형 시험의 실제 근거 대기와 권한 대기 구분

## 선행 구현의 검증 경계

- OV150-002–010 구현·회귀 보완 완료. [최종 로컬 검사](../research/overhaul/OV150-010.md#최종-로컬-검사-범위): Python 81개 모듈·1137개 통과·45개 조건부 건너뜀, Rust 1429개 통과·5개 수동 제외
- 공개 0.14.2 원본/태그 검증과 17개 공개 이전 버전의 실제 Hive CLI 보존·갱신·실패 복구 통과. 실제 모델·호스트 왕복·0.15.0 공개 세 OS 수용 증명 제외
- 이번 종료 경계: OV150-011 전. 태그·게시·develop/main 병합·실제 사용자 설치 미실행, 조율 기본 비활성

- 단계 브랜치 원격 반영 확인: 제품 검증 8e4909bd·브랜치 규칙 43267922439 success. 전체 CI는 PR/main/develop 조건으로 미실행, 이번 새 PR·병합·태그·게시 없음

- 후속 문서 반영 0e26087f·브랜치 규칙 43268257440 success. 산출물 점검의 미검토 471·후속 실험 참조 1 보존, 원본 결과 Markdown 편집·삭제·커밋 없음

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.0`
- 현재 등록 항목: 13/18 완료

- `agent-owned`: `OV150-011`, `OV150-012`, `OV150-013`, `OV150-017`, `OV150-018`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 단계 3 검토 후 수정

- [Fable 5.2 검토](../research/overhaul/stage-3-review.md): COMMENT·경계 위반 0·높은 위험 0, 일반 Fable 기본 수준 대체
- M1–M4·L5–L11 수정·검증 완료, P3 정본 표의 67개 생성 표면 확인. P1 OV150-014·P2 OV160-002·P4 OV151-011 배치
- Codex·Windows: Rust 1434개 통과·수동 제외 5, Python 81개 모듈·1140개 통과·조건부 제외 45. 후속 관련 시험으로 최초 목록 누락 보완, 중복 합산 없음
- 최종 fmt·workspace/all-targets/all-features clippy·문서 묶음 통과, Source Wiki 정본·파생 색인 274쪽 확인. [정확한 근거·한계](../research/overhaul/stage-3-review.md#최종-수정과-검증)
- 산출물 관리: 검토 대기 470·실험실 보존 1. 검토 완료한 이전 버전 재현 폴더 99개 파일 정리, 기록 보존. 일일 점검은 최소 20.306GiB·미측정 21개로 주의 필요, 공유 빌드·다른 작업 원본의 전체 정리 완료 주장 없음
- 원격 검증: 단계 분기 `75dc4c16`·[브랜치 검사 43285223978](https://github.com/gvm1229/aigent-hive/actions/runs/43285223978) success·로컬/원격 일치. Ubuntu 분기·커밋 규칙만 확인, 제품 전체 CI는 PR/main/develop 조건으로 미실행
- 0.15.0-test.1 수용: 세 운영체제의 역할 구성 병합·거부, 호스트 제공 출력의 `roster check`, 비활성 summon 미리보기·서명 Prepare, 신 입구·별칭 9개, 0.14.2 갱신. 이 빌드의 조율 비활성·활성화 경로 없음. Windows 11·Claude Code 2.1.322·Codex의 실제 왕복 확인. Gemini·다른 OS 왕복·실제 호스트 취소·재점유 미검증, 이 시험의 수용 대상 밖
- OV150-001 완료, 정식 OV150-013의 OV150-014 통과 필요. 현재 요청의 태그·게시·병합·설치 승인
