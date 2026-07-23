# ADR-0004: 기존 orchestration runtime 사용

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Hive는 plan, Ralph, team, swarm, model session scheduler를 구현하지 않음.

- Codex: host native 또는 OMX
- Claude Code: host native 또는 OMC
- Gemini Antigravity: host native

## 역할 분리

- Hive: setup, durable role document, Markdown run state, Wiki/index, validation, update
- Host/OMX/OMC: model call, subagent spawn, session continuation, orchestration loop

## 선택과 관찰

- 사용자의 runtime 선택이 정본
- 사용자가 요청한 경우 public executable path와 side-effect-free `--version`만 advisory evidence로 확인
- Hive 제품은 `.omx/`, `.omc/`, `.codex/`, `.claude/`와 host-global runtime state를 읽거나 수정하지 않음
- 공존 검증은 synthetic fixture가 외부 tree의 before/after checksum을 계산하며 Hive process에 foreign tree 접근 권한을 주지 않는 방식으로 수행

## 실패 의미

선택 runtime에 capability가 없으면 `unsupported`. Hive가 다른 runtime으로 자동 fallback하거나 유사 기능을 생성하지 않음.
