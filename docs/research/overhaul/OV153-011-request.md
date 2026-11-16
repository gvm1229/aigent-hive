# OV153-011 요청 — 에이전트 주도 프로젝트 설정

- 기준일: 2026-11-17
- 계기: `korean-ai-job-search`에서 `/aigent-hive:project-setup` 호출 뒤 에이전트가 설정 대신 대화형 CLI 실행을 사용자에게 넘김
- 원인: project-setup·user-setup Skill이 보이는 터미널이 없으면 명령만 안내하고 AI 질문 대체를 금지. 에이전트 경로 `--answers --input-channel agent --apply`는 대화형만 만드는 `--capabilities` 파일 필수

## 유지관리자 원문

> The CLI design choice was meant to be optional. It must not take away the ability for AI agents to do the setup and the work autonomously. In fact, most users will just ask their AI agent to set up hive for them.

> 1. The user triggers the skill (the user called the skill, not the cli command, meaning that they want the AI agent to take over the setup in the first place)
> 2. Via the skill, the AI agent will look over the project and the global user hive settings and determine what project setup is most appropriate
> 3. If the project is an empty folder, it'd be the user wanting to install hive as a starting point to their project. Then the AI agent will ask the user what project they would like, one detail after another. What the project is about, the goal for the project, how the user wants the project to be developed, and so on. Those cannot, and must not be determined by the AI agent blindly and unannounced.
> 4. If the project already has something, then that must be scanned to determine what the project is. To take a few examples: a mountain of markdown files (library or archive esque project), typescript code but with react installed (web project), cpp files with uasset files (unreal game project). Those examples are not exact as the cpp files with uasset files might be an unreal simulation. All the more reason for the AI agent to actually scan the project before determining.

> Ensure this is the case in the 0.15.3 version. Do not release 0.15.3 in the current state. The above fix must be included in its scope as well.

## 결정

- 정식 0.15.3 보류. OV153-011 완료·공개 시험 재수용·이 PC 실제 확인 뒤 출시

## 추가 보고(2026-11-17, 0.15.2 Windows 11 Claude Code 새 빈 프로젝트)

- 현상 1: `hive setup --target <project> --user-root <user-root> --answers <patch.json> --input-channel agent --dry-run --output json`이 `hive.setup-invalid-input` "missing required option --capabilities"로 미리보기부터 거부, 바뀐 파일 없음
- 현상 2: `--non-interactive --dry-run`은 `hive.project-setup-input-pending`과 "터미널에서 이어서 진행" 안내만 반환
- 문제: Skill은 신뢰할 수 있는 능력 자료만 쓰라고 요구하지만 새 프로젝트·`~/.hive`에 그 파일이 없고 만드는 명령도 부재. 결과적으로 자율 설정 불가, 남는 길은 사용자 터미널뿐
- 보고자 요청: 능력 자료 생성 명령, 또는 `--capabilities` 없이 호스트 기본 능력으로 미리보기·적용, 둘 다 어렵다면 오류·Skill 문서에 제약 명시

## 추가 범위(0.15.3 필수)

- A1 `--capabilities` 없는 `--input-channel agent --dry-run`·`--apply`·`--validate`가 새 빈 프로젝트에서 성공. 위 현상 1 명령 그대로 계약 시험
- A2 에이전트가 받는 모든 대기·오류 결과(`hive.project-setup-input-pending`, `hive.setup-input-pending`, `hive.setup-needs-input`, `--non-interactive` 결과 포함)에 터미널 안내 단독 금지. 에이전트 경로 명령(`--show --summary`, `--answers ... --input-channel agent --dry-run`)을 `next_action`에 함께 제시
- A3 별도 능력 생성 명령은 범위 밖. 능력 파일은 선택 사항이며 저장값·감지값을 쓴다는 사실을 Skill·사용법·오류 문구에 명시
- A4 local 확인에 현상 1·2 명령 재현 포함. test.2에서 미리보기·적용 성공, 터미널만 안내하는 결과 없음
