# ADR-0004: 기존 orchestration runtime 사용

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Hive는 plan, Ralph, team, swarm, model session scheduler를 구현하지 않음.

- Codex: compatible OMX capability를 우선, 없을 때 host native
- Claude Code: compatible OMC capability를 우선, 없을 때 host native
- Gemini Antigravity: host native

사용자에게 pure Hive와 OMX/OMC 중 하나를 고르게 하지 않는다. 새 run의 owner는 active host capability에서 자동 resolve하고 해당 run의 `STATUS.md`에 evidence digest와 함께 고정한다.

## 역할 분리

- Hive: setup, durable role document, Markdown run state, Wiki/index, validation, update
- Host/OMX/OMC: model call, subagent spawn, session continuation, orchestration loop

## Resolution과 관찰

- active host가 노출한 Skill/plugin capability metadata 또는 public executable path와 side-effect-free `--version`을 evidence로 확인
- Hive 제품은 `.omx/`, `.omc/`, plugin cache와 host-global runtime state를 읽거나 수정하지 않음
- 공존 검증은 synthetic fixture가 외부 tree의 before/after checksum을 계산하며 Hive process에 foreign tree 접근 권한을 주지 않는 방식으로 수행

## 실패 의미

Resolved runtime에 capability가 없거나 run 도중 실패하면 `unsupported` 또는 `blocked`. Hive가 다른 runtime으로 자동 fallback하거나 유사 기능을 생성하지 않음.

OMX/OMC가 감지된 host에는 Hive lifecycle hook과 duplicate orchestration Skill을 설치하지 않는다. External capability가 conclusively absent일 때만 Hive-owned data-integrity fallback hook을 capability·event·path·digest별로 설명하고 사용자 승인을 받을 수 있다. 거절은 정상 지원 상태다.

Semantic Skill routing은 host Skill discovery, narrow descriptions와 compact `AGENTS.md` precedence가 담당한다. Hive가 OMX/OMC의 keyword detector, classifier 또는 Stop continuation을 복제하지 않는다.
