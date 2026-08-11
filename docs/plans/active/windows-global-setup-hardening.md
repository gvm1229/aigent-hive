# Global setup release recovery·Windows hardening

- 상태: active
- Target: 다음 numbered build와 stable `0.9.0` 출시 gate
- 완료된 선행 조건: Mac release 원본 복구 `BGR-008–013`
- 미완료 선행 조건: `GSS-*`, `SIL-*`, `UGP-*`
- 연결 gate: `KST-006`, `DIS9-002–010`, `REL9-011`
- 입력 증거: 이 Mac의 과거 developer/public build 원본 불일치, 현재 local validation,
  maintainer가 제공한 Windows 11 Codex setup 기록과 `0.9.0-test.5`

## 목표

- Mac: 완료된 developer build·public 시험판 원본 복구의 회귀 방지와 사용자 데이터 보존 확인
- Windows: npm 전역 설치 경로가 Codex의 현재 `PATH`에 없어도 설치된 실행 파일 자동 탐색
- 공통 setup: 질문 전 CLI·설정 계약 확인, 답안 단일 저장, 첫 `dry-run` 검증, 중단 지점 재개
- 사용자 홈의 임시 답안 파일: `0건`

## 완료된 Mac 복구

- 과거 문제: developer build와 공개 시험판의 설치 원본 기록 불일치, 오래된 `0.7.0` Hive 파일 잔존
- 구현 정본: [`bootstrap-global-setup-recovery.md`](bootstrap-global-setup-recovery.md)의
  `BGR-008–013`
- 복구 계약: authenticated historical base·live byte 확인, vanilla 교체, local edit 보존 merge,
  unknown·tampered byte의 write 없는 conflict, knowledge·preference 보존
- 현재 증거: `/Users/hojin/.local/bin/hive`의 `AIgent Hive v0.9.0-dev`와
  `hive install --scope user --host codex --validate --output json` 성공
- 결론: 동일 기능 재구현 없음. Windows 수정 뒤 Mac 회귀검사만 재실행

## 확인된 문제

1. Codex PowerShell CLI 탐색 전 설정 질문 시작
2. 별도 `cmd.exe`의 수동 `where hive` 실행·경로 전달 요구
3. 설정 schema·정확한 답안 예시·내장 Skill catalog의 읽기 전용 CLI 부재
4. 실행 파일 byte·npm package 폴더 검색과 답안 YAML 추측
5. 사용자 홈 임시 답안 파일 최소 17개 생성 뒤 첫 검증 실패
6. 일반 질문 진행 상태 미보존과 실패 뒤 처음부터 재답변
7. 기본 감지기 실패 증거 없는 `CodexBar` 질문과 Discord 질문 누락
8. 임시 파일 생성·안전한 `dry-run` 전 불필요 승인 요청
9. 한국어 질문의 예기치 않은 문자(`ર`)와 영어 혼합
10. Codex Antigravity 가용성 확인 없는 최종 설정 요약

## 현재 소스와의 차이

현재 `develop`의 `configure` Skill: 언어 우선 질문, 모든 내장 Skill 기본값, 한 줄당 하나의
목록, Discord 조건부 설정, `CodexBar` fallback 조건, 중단 재개 문구 보유. 첨부 기록만으로
후속 변경 실패 단정 불가. 아래 구조적 결함의 당시 소스 잔존

- 설치된 CLI의 Windows npm 경로 자동 탐색 절차 부재
- 서명된 설정 계약·canonical 답안 예시 일괄 출력 부재
- Discord 외 일반 질문의 진행 상태 저장 부재
- 임시 답안 파일 위치·단일 파일 사용·실패 시 정리 계약 부재
- 질문 전 host 가용성·기본 사용량 감지기 검증 증거 강제 부재

## 구현 순서

- [x] [WGS-001] 첨부 기록의 명령·질문·파일 쓰기·실패 결과 감사와 현행 소스 차이 분류 완료
- [x] [WGS-002] Windows `Get-Command` → `where.exe` → `npm prefix -g` 기반 `hive.cmd` resolver, exact version·package ownership 확인, process `PATH` 갱신·사용자 수동 경로 전달 요구 없음
- [x] [WGS-003] `hive setup --scope user --describe --output json`의 embedded schema·canonical 답안 template·질문 순서·조건·localized option·built-in Skill catalog·contract digest 읽기 전용 출력. 사용량 한도: 사용자 입력 placeholder, 기본값 없음
- [x] [WGS-004] canonical·plugin `configure`의 `--describe` 전용 입력, binary byte·npm 재귀 검색·필드명·Skill ID 추측 금지, CLI 확인 실패 전 질문 시작 금지
- [x] [WGS-005] non-secret partial answer·다음 질문 progress 저장과 재실행 시 `전체 다시 보기`·`일부만 변경`·`중단한 곳부터 계속` 제공
- [x] [WGS-006] 운영체제 임시 폴더의 session별 단일 답안 파일 atomic 갱신·성공/실패/취소 시 삭제, persisted progress의 비밀 없는 partial answer 한정, Hive 생성 입증 파일만 cleanup preview 대상
- [x] [WGS-007] 질문 전 preflight, 조건부 Discord·`CodexBar` 질문, host 상태 authenticated·deferred·unsupported 구분
- [x] [WGS-008] 명시 global setup 요청의 안전한 임시 파일·`dry-run`·무충돌 built-in apply 승인 처리, conflict·third-party Skill·외부 설치·비밀 접근·파괴 작업만 별도 확인
- [x] [WGS-009] 한국어 exact prompt fixture의 언어 혼합·예기치 않은 문자·`Skill` 오역·다중 항목 목록 차단
- [x] [WGS-010] Windows PATH 불일치·individual Skill·첫 YAML 검증·일반/Discord 중단 재개·temp cleanup·조건부 질문의 Rust unit·CLI integration·Python static 회귀 추가
- [x] [WGS-012] authenticated pending Codex marketplace transaction이 Hive-owned canonical root를
  가리키지만 host manifest가 사라져 structured probe가 실패할 때, `hive install --recover`가
  knowledge·저장 preference·foreign host entry를 보존하고 exact Hive marketplace entry만 제거한 뒤
  재설치 허용. source·product workflow의 deterministic recovery 우선 실행
- [x] [WGS-013] `hive uninstall`이 Hive-managed host activation·projection·package·index·backup·runtime만
  제거하고 `.hive/knowledge/`·saved user preferences를 항상 보존. `--full`·`-f` 파괴 경로 제공 없음.
  저장 preference를 읽은 재설치의 setup 질문 0건, Rust unit·product Skill projection static regression 통과
- [x] [WGS-011] `0.9.0-test.12` 시험판을 게시한 뒤 maintainer의 실제 Windows 11 machine에서
  clean npm install·fresh Codex session·product-only Skill catalog·global/project usage guard를
  포함한 한 번의 setup으로 `dry-run → apply → validate` 완료. 사용자 수동 `where hive`, schema
  추측, home 임시 파일, 조건 밖 질문 모두 0건. 이 Mac에서 Windows 설치·setup 실행 또는 대체
  수용 판정 금지. `test.12` actual clean reinstall·saved preference 재사용·setup sequence·Skill·data
  보존 증거와 유지보수자 확인의 새 Codex session 자동 CLI 탐색·Discord 실제 전달 확보

## 수용 결과

- Mac developer build·public 시험판 전환: ownership 오류 없음, preference·knowledge 보존,
  질문 모음 반복 0건
- 첫 실행 사용자: 재답변 없는 setup 완료
- CLI 탐색 과정·설정 파일 형식: Agent 추측 없음
- 개별 Skill 선택: 설치 release의 정확한 목록 기반
- Discord·`CodexBar` 질문: 실제 조건 충족 시에만 표시
- 실패 뒤 진행 상태 보존, webhook URL·raw prompt 비밀 저장 없음
- 이 Mac: 완료된 원본 복구와 새 source unit·static·cross-platform 회귀 실행
- Windows global install·setup 수용: maintainer의 실제 Windows 11 fresh session에서만 실행·증명
- dangling Hive Codex marketplace: Hive backup·pending transaction·canonical root 일치 시 자동 recovery,
  knowledge·저장 preference·foreign host entry 변경 0건
- clean reinstall: `hive uninstall` 뒤 saved preference·knowledge 보존, 설치 재개 시 setup 질문 0건
- `test.12` Windows actual: `hive uninstall → install → dry-run → apply → validate → install validate` PASS,
  saved setup digest·knowledge file 5개 aggregate digest 동일, Hive active Skill 22개·retired ID `0건`,
  home temporary answer `0건`, `--full` unknown-option rejection. Discord persisted 설정·usage guard `20%`
  확인. 유지보수자 Windows fresh session: 자동 CLI 탐색·Discord 실제 전달 확인

## 범위 밖

- Notion: `0.10.0-test` 전까지 설정·도움말·질문 노출 없음
- provider API key·Discord webhook URL 원문: Hive 설정·진행 상태 저장 없음
- stable `0.9.0` 게시 소유: 이 fragment가 아닌 `REL9-*`
