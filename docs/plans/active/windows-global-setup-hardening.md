# Windows global setup hardening

- 상태: active
- Target: 다음 `0.9.0-test` numbered build
- 선행 조건: `BGR-*`, `GSS-*`, `SIL-*`
- 연결 gate: `KST-006`, `DIS9-002–010`, `REL9-011`
- 입력 증거: maintainer가 제공한 Windows 11 Codex setup 기록, `0.9.0-test.5`

## 목표

Windows에서 npm 전역 설치 경로가 Codex의 현재 `PATH`에 보이지 않아도 Hive가 설치된 실행
파일을 직접 찾아야 한다. 질문을 시작하기 전에 CLI와 설정 규칙을 확인하고, 사용자의 답을 한
번만 정확한 형식으로 저장하여 첫 `dry-run`에서 검증해야 한다. 중단 후에는 마지막 질문부터
이어갈 수 있어야 하며 사용자 홈에 임시 답안 파일을 남기지 않는다.

## 확인된 문제

1. Codex PowerShell이 `hive`를 찾지 못했는데도 설정 질문을 먼저 시작했다.
2. 사용자가 별도 `cmd.exe`에서 `where hive`를 실행해 경로를 전달해야 했다.
3. CLI가 설정 schema·정확한 답안 예시·내장 Skill catalog를 읽기 전용 명령으로 제공하지 않았다.
4. Agent가 실행 파일 byte와 npm package 폴더를 검색하고 답안 YAML을 추측했다.
5. 사용자 홈에 이름을 바꾼 임시 답안 파일을 최소 17개 만들고도 첫 검증에 성공하지 못했다.
6. 일반 질문의 진행 상황을 저장하지 않아 실패 뒤 처음부터 다시 답해야 하는 상태였다.
7. 기본 사용량 감지기의 실패 증거 없이 `CodexBar`를 물었고, 사용량 보호를 켰는데 Discord 질문은 하지 않았다.
8. 임시 파일 생성과 안전한 `dry-run` 전에 불필요한 승인을 요청했다.
9. 한국어 질문 끝에 다른 문자(`ર`)가 섞였고 일부 안내가 영어와 혼합됐다.
10. Codex에서 선택한 Antigravity가 실제로 쓸 수 있는지 확인하지 않은 채 최종 설정처럼 요약했다.

## 현재 소스와의 차이

현재 `develop`의 `configure` Skill에는 언어 우선 질문, 모든 내장 Skill 기본값, 한 줄에 하나인
목록, Discord 조건부 설정, `CodexBar` fallback 조건, 중단 재개 문구가 이미 있다. 첨부 기록만으로
이 후속 변경이 실패했다고 단정하지 않는다. 그러나 아래 구조적 결함은 현재 소스에도 남아 있다.

- 설치된 CLI의 Windows npm 경로를 자동으로 찾는 절차가 없다.
- CLI가 서명된 설정 계약과 canonical 답안 예시를 한 번에 출력하지 않는다.
- Discord 단계 외 일반 질문은 진행 상황 저장 대상이 아니다.
- 임시 답안 파일의 위치·단일 파일 사용·실패 시 정리 계약이 없다.
- host 가용성과 기본 사용량 감지기를 질문 전에 검증한 증거를 강제하지 않는다.

## 구현 순서

- [x] [WGS-001] 첨부 기록을 명령·질문·파일 쓰기·실패 결과 단위로 감사하고 현재 소스와 차이를 분류한다.
- [ ] [WGS-002] Windows에서 `Get-Command`, `where.exe`, `npm prefix -g` 순서로 `hive.cmd`를 찾고 exact version·package ownership을 확인하는 공통 resolver를 추가한다. 현재 process의 `PATH` 갱신이나 사용자의 수동 경로 전달을 요구하지 않는다.
- [ ] [WGS-003] `hive setup --scope user --describe --output json`을 추가하여 embedded schema, canonical 답안 예시, 질문 순서·조건, localized option, built-in Skill catalog, contract digest를 읽기 전용으로 출력한다.
- [ ] [WGS-004] canonical·plugin `configure` Skill이 `--describe` 결과만 사용하도록 바꾸고 binary byte 검색, npm 폴더 재귀 검색, 필드명·Skill ID 추측을 금지한다. CLI 확인 실패 전에는 질문을 시작하지 않는다.
- [ ] [WGS-005] 모든 답변 뒤 non-secret partial answer와 다음 질문을 저장하도록 progress schema와 command를 확장한다. 실패·재실행 때 `전체 다시 보기`, `일부만 변경`, `중단한 곳부터 계속`을 제공한다.
- [ ] [WGS-006] 답안 작업 파일은 운영체제 임시 폴더의 session별 단일 파일만 atomic 갱신하고 성공·실패·취소 시 삭제한다. user root의 기존 `.hive-user-setup-answers*.yml`은 Hive가 만든 것이 exact하게 증명될 때만 별도 cleanup preview에 포함한다.
- [ ] [WGS-007] 질문 전 preflight를 구현한다. 사용량 보호가 켜지면 Discord를 물으며, native sensor가 unavailable·unsupported·malformed일 때만 `CodexBar`를 물고, 선택 host는 authenticated·deferred·unsupported로 구분한다.
- [ ] [WGS-008] 명시적인 global setup 요청이 안전한 임시 파일·`dry-run`·conflict 없는 built-in apply를 승인한 것으로 처리한다. conflict, third-party Skill, 외부 설치, 비밀 접근, 파괴 작업만 별도 확인한다.
- [ ] [WGS-009] 한국어 exact prompt fixture를 보강하여 한 문장 안 언어 혼합, 예상 밖 문자, `Skill` 오역, 한 줄에 여러 항목인 목록을 차단한다.
- [ ] [WGS-010] Rust unit·CLI integration·Python static contract에 Windows PATH 불일치, individual Skill, 첫 YAML 검증 성공, 일반·Discord 단계 중단 재개, temp cleanup, conditional question 회귀를 추가한다.
- [ ] [WGS-011] 다음 numbered 시험판을 게시하고 실제 Windows 11의 clean npm install·fresh Codex session에서 한 번의 setup으로 `dry-run → apply → validate`를 완료한다. 사용자 수동 `where hive`, schema 추측, home 임시 파일, 조건 밖 질문이 모두 0건이어야 하며 이 증거 전 stable `0.9.0`은 중지한다.

## 수용 결과

- 처음 실행한 사용자는 질문을 다시 답하지 않고 setup을 끝낸다.
- CLI를 찾는 과정과 설정 파일 형식은 Agent가 추측하지 않는다.
- 개별 Skill 선택은 설치된 release의 정확한 목록으로 동작한다.
- Discord와 `CodexBar` 질문은 실제 조건에 맞을 때만 나온다.
- 실패 뒤 진행 상태는 보존되지만 webhook URL·raw prompt 같은 비밀은 저장하지 않는다.
- 이 컴퓨터의 macOS 검증은 구현 회귀만 증명한다. Windows 수용은 실제 Windows 11 fresh session에서 별도로 증명한다.

## 범위 밖

- Notion은 `0.10.0-test` 전까지 설정·도움말·질문에 노출하지 않는다.
- provider API key·Discord webhook URL 원문을 Hive 설정이나 진행 상태에 저장하지 않는다.
- stable `0.9.0` 게시 자체는 이 fragment가 아니라 `REL9-*`가 소유한다.
