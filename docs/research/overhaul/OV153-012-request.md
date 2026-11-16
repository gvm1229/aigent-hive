# OV153-012 요청 — 대화형 설정 화면 가독성

- 기준일: 2026-11-17
- 계기: `korean-ai-job-search`에서 `hive setup --target … --user-root …` 빠른 설정 실행 화면
- 관찰: 질문 4개 뒤 미리보기 20여 줄이 같은 들여쓰기·같은 모양으로 붙어 출력. 질문·도움말·선택지·기본값·입력 줄 사이 구분 부족, 제목·구역·강조 없음. 미리보기는 `질문 문장 + 값`을 한 줄씩 이어 붙인 형태라 같은 질문의 반복과 비슷한 모양. 기능 목록은 한 줄 JSON, 빈 값 줄(`지식에 포함할 경로를 …`)도 그대로 표시
- 위치: `crates/hive-cli/src/user_setup/setup_engine/terminal.rs`(`say`·`ask`·`confirm`), `crates/hive-cli/src/user_setup/setup_engine/project.rs:504` 미리보기, `crates/hive-cli/src/user_setup/setup_engine/project/questions.rs:184` 기능·후크 요약. 사용자 설정(user-setup)도 같은 `Terminal` 사용

## 유지관리자 원문

> That is literally what a project setup via the hive cli looks like. Absolute horrid readability. The capability is good. The wordings are good. But the readability becomes horrid with just a few questions. They are glued together and not separated properly, and there are no decorations to the text whatsoever. Make this fix a 0.15.3 scope.

## 결정

- 0.15.3 범위에 포함. 기능·문구는 유지, 화면 배치와 꾸밈만 고침
- 정식 0.15.3은 OV153-011·OV153-012 완료와 공개 시험 재수용 뒤 출시
