# OV170-022 진행 기록 — 브라우저 선택과 기존 탭 사용

- 기준일: 2026-11-28, 실행 장소: cloud 세션. 항목은 아직 미완료

## 완료

- 전역 설정 `tools` 절의 `user-browser`: `default-browser`(기본)·`in-app`. 사용자 설정 `browser.target`, `browser.reuse_active_tab`(기본 `true`), `browser.detected_default`
- 판별: 설정 Skill이 운영체제 기본 브라우저를 읽기 전용 질의로 한 번 읽어 기록(Windows `UserChoice` ProgId, macOS LaunchServices `https` 처리기, Linux `xdg-settings`). `crates/hive-cli/src/browser.rs`가 Chromium 계열(Chrome·Chromium·Edge·Brave·Whale·Vivaldi·Opera·Arc)과 대조, 아니거나 실패면 Chrome. Hive 자체 프로세스 실행 없음
- 지침 겹: 전역 Hive 블록에 미리 보기 브라우저, 새 탭 대신 보고 있는 탭에서 작업, Whale의 Claude in Chrome 지원 미검증 안내
- 시험: 세 OS 고정물(Chromium·비 Chromium·판별 실패), 지침 문구(기본·Whale·앱 내장·탭 재사용 끔), 빈 설정 기본값, 설정 화면 스냅샷

## 남은 범위

- 강제 겹(Claude): 사용자 확인과 `~/hive-backups` 백업 뒤 `~/.claude/settings.json`의 `permissions.deny` 추가, 미리보기·되돌리기 명령. 미구현
- 강제 겹(Codex): MCP 도구별 끄기 지원 조사 미완료. 확인 전에는 지침 겹만
- 실제 확인(local): Whale 기본 설정으로 Claude·Codex 동작([local 인계](OV170-local-handoff.md))
- 실행 파일 위치 기록: 판별 값만 기록, 실행 파일 경로 미기록
