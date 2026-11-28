# OV170-022 진행 기록 — 브라우저 선택과 기존 탭 사용

- 기준일: 2026-11-28, 실행 장소: cloud 세션. 항목은 아직 미완료

## 완료

- 전역 설정 `tools` 절의 `user-browser`: `default-browser`(기본)·`in-app`. 사용자 설정 `browser.target`, `browser.reuse_active_tab`(기본 `true`), `browser.detected_default`
- 판별: 설정 Skill이 운영체제 기본 브라우저를 읽기 전용 질의로 한 번 읽어 기록(Windows `UserChoice` ProgId, macOS LaunchServices `https` 처리기, Linux `xdg-settings`). `crates/hive-cli/src/browser.rs`가 Chromium 계열(Chrome·Chromium·Edge·Brave·Whale·Vivaldi·Opera·Arc)과 대조, 아니거나 실패면 Chrome. Hive 자체 프로세스 실행 없음
- 지침 겹: 전역 Hive 블록에 미리 보기 브라우저, 새 탭 대신 보고 있는 탭에서 작업, Whale의 Claude in Chrome 지원 미검증 안내
- 시험: 세 OS 고정물(Chromium·비 Chromium·판별 실패), 지침 문구(기본·Whale·앱 내장·탭 재사용 끔), 빈 설정 기본값, 설정 화면 스냅샷

## 남은 범위

- 강제 겹(Claude): 2026-11-28 local 구현 완료(아래). 이 PC의 실제 `~/.claude/settings.json` 적용은 사용자 승인 대기
- 강제 겹(Codex): MCP 도구별 끄기 지원 조사 미완료. 확인 전에는 지침 겹만
- 실제 확인(local): Whale 기본 설정으로 Claude·Codex 동작([local 인계](OV170-local-handoff.md))
- 실행 파일 위치 기록: 판별 값만 기록, 실행 파일 경로 미기록

## local 조사(2026-11-28, 이 Windows 11 Pro PC)

- 기본 브라우저: `UserChoice` `ProgId` 값 `WhaleHTM`(읽기 전용 조회). `browser::recognize`의 `whale` 표지와 맞아 Whale로 판별
- Claude in Chrome 연결: 연결 확인. `list_connected_browsers` 결과 Windows 로컬 브라우저 1개 연결·사용 중, 실행 중인 브라우저는 `whale` 프로세스뿐(`chrome` 없음), Claude 확장(`fcoeoabgfenejglbffodgkkbkcdhcgfn`)은 Whale `Profile 1`에만 설치. 탭 조작은 미실행: 이 세션의 탭 묶음이 없어 확인하려면 새 탭을 만들어야 하므로 보류
- 빈틈: 지침에 Whale이 나오려면 `browser.detected_default`가 기록돼야 함. 이 값은 `user-setup` Skill이 OS 조회 결과를 답변 파일에 넣을 때만 기록. 사람이 터미널에서 `hive setup --scope user --section tools`를 직접 실행하면 기록되지 않아 지침이 Chrome으로 나옴. Hive 안에서 직접 읽으려면 작업 공간 규칙(`unsafe_code = "forbid"`) 때문에 레지스트리 읽기용 새 의존성(예: `winreg`) 필요. 결정 대기
- Codex 강제 겹 조사: 설치된 `codex-cli 0.171.0` 실행 파일의 설정 구조(`RawMcpServerConfig`)에 MCP 서버별 `enabled_tools`·`disabled_tools` 키 있음. `codex features list`에 `browser_use`·`browser_use_external`·`browser_use_full_cdp_access`(모두 stable·활성) 있음. Codex의 브라우저 조작은 개별 MCP 도구가 아니라 `node_repl`·`cua_repl` 서버와 기능 스위치로 제공
  - 지원되는 방식: 앱 내장 브라우저 끄기 `features.browser_use=false`, 외부 Chrome 조작 끄기 `features.browser_use_external=false`, MCP 서버 단위 `disabled_tools`
  - 지원되지 않는 방식: 새 탭 열기만 따로 막기. 탭 도구가 별도 MCP 도구 이름으로 노출되지 않아 지침 겹만 적용
  - 미검증: 두 기능 스위치가 실제로 어느 브라우저를 끄는지의 실행 확인. 문자열과 기능 목록만 확인

## Claude 강제 겹(2026-11-28 local)

- 명령: `hive setup --scope user --section tools --claude-browser-deny preview`, `... apply --confirm <digest>`, `... revert`(미리 보기)와 `... revert --confirm <digest>`
- 추가 항목: `default-browser`는 `mcp__Claude_Browser__*`, `in-app`은 `mcp__claude-in-chrome__*`. 탭 재사용이 켜져 있으면 반대쪽 새 탭 도구 하나 추가
- 적용 순서: 미리 보기 지문 일치 확인, `<user-root>/hive-backups/`에 검증된 백업, 되돌리기 기록 `.hive/state/claude-browser-deny.json`, `permissions.deny`만 고치고 다른 바이트 보존
- 되돌리기: 적용 직후 그대로면 원래 바이트 복원, 바뀌었으면 Hive가 넣은 항목만 제거. 브라우저 선택이 바뀐 뒤에는 먼저 되돌리기 요구
- 시험: `cargo test -p hive-cli --bin hive claude_browser_deny` 7개 통과(Windows 11, 이 PC, 임시 폴더만 사용)
- 지침 문구에는 명령 줄 미추가. 설정 `tools` 절 결과에 선택 제안으로 표시

## 시험판 설치 뒤 확인(2026-11-28, `0.17.0-test.2`)

- 갱신 뒤 세 호스트 전역 지침의 미리 보기 줄: "항상 Chrome에서 열기". 갱신은 기본 브라우저 판별 값을 기록하지 않으므로 Whale 대신 Chrome으로 표시. 위 빈틈이 실제 설치에서도 재현
- 판별 값 기록 시도: `user-setup` Skill과 같은 답변 묶음(`browser.detected_default: WhaleHTM`)의 미리 보기는 통과. 적용은 2026-11-20에 미리 보기 단계에서 멈춘 사용자 설정(사용량 감시 켜기 단계) 때문에 충돌로 거부. 이 PC의 설정 변경 0건
- 제품 빈틈: 오래 멈춘 설정은 `--resume` 말고 버리는 방법 부재. 관계없는 설정 절 변경까지 계속 차단

## Whale 기록(2026-11-29, `0.17.0-test.3`)

- 멈춘 초안 정리 뒤 `browser.detected_default: WhaleHTM` 적용 성공. Claude·Codex·Antigravity 전역 지침 모두 "항상 Whale에서 열기", Whale의 Claude in Chrome 안내 한 줄 포함
- 남은 범위: 사람이 터미널에서 설정할 때의 기본 브라우저 판별(`winreg` 결정 대기), 새 세션에서 실제 미리 보기 요청 확인

