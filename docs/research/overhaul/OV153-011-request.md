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
