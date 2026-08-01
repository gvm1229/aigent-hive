# ADR-0004: 기존 orchestration runtime 사용

- 상태: superseded by ADR-0015
- 날짜: 2026-07-23
- v0.9 대체 제안: [`ADR-0015`](ADR-0015-host-native-skill-composition.md)
- 현재 효력: 새 `0.9.x` 실행에는 없음. 고정된 `0.8.x` 실행의 역사적 호환 계약만 유지

## 역사적 `0.8.x` 결정

결정: Hive의 plan, Ralph, team, swarm, model session scheduler 구현 금지.

- Codex: compatible OMX capability를 external owner로 resolve
- Claude Code: compatible OMC capability를 external owner로 resolve
- Gemini Antigravity: host native

Pure Hive와 OMX/OMC 사이의 setup 선택지 없음. 새 run owner는 active host capability evidence에서 자동 resolve한 뒤 해당 run의 `STATUS.md`에 evidence digest와 함께 고정.

## 역할 분리

- Hive: setup, durable role document, Markdown run state, Wiki/index, validation, update
- Host/OMX/OMC: model call, subagent spawn, session continuation, orchestration loop

## Resolution과 관찰

- active host가 노출한 Skill/plugin capability metadata 또는 public executable path와 side-effect-free `--version`을 evidence로 확인
- `available`은 active host에 맞는 compatible evidence 하나로 확정했던 `0.8.x` 규칙
- `absent`는 host catalog와 public executable 양쪽의 명시적 absent evidence가 모두 있어야 확정
- `incompatible`과 `unknown`: host-native resolve, Hive fallback hook 금지
- `evidence_digest`는 `evidence_digest` field 자체만 제외한 normalized capability resolution object 전체의 RFC 8785 JCS bytes를 SHA-256한 값
- Hive 제품의 `.omx/`, `.omc/`, plugin cache, host-global runtime state read·write 금지
- 공존 검증은 synthetic fixture가 외부 tree의 before/after checksum을 계산하며 Hive process에 foreign tree 접근 권한을 주지 않는 방식으로 수행

## 실패 의미

Resolved runtime의 capability 부재·실행 중 실패 결과: `unsupported` 또는 `blocked`. 다른 runtime으로의 자동 fallback과 유사 기능 생성 금지.

첫 checkpoint는 host, host version, surface, external runtime, resolved owner, full
resolution evidence digest와 subagent support를 `STATUS.md`에 함께 pin. 이후
missing, incompatible, version 또는 evidence drift는 exit `3|4|5`로 중지하며 기존
run owner와 canonical artifact 변경 금지. Fresh-session resume는 host가 실행할
provider-neutral brief만 `prepared_only: true`, `spawned: false`로 준비.

`0.8.x`에서 OMX/OMC가 선택된 host는 Hive lifecycle hook과 duplicate orchestration Skill 설치 금지. `0.9.x` optional hook은 ADR-0015에 따라 host가 지원하는 exact integrity event와 별도 동의가 있을 때만 허용.

`hive-role-handoff`, `hive-run-checkpoint`, `hive-run-resume`는 role/run Markdown
정본을 기록·검증·복구하는 data Skills. Compatible OMX/OMC와 함께 projection 가능. Plan, Ralph, team, retry, persistent loop 실행·복제 금지.

Hook descriptor는 `{schema_version, capability, event, path, command}`만 포함한 RFC 8785 JCS object의 UTF-8 bytes와 trailing LF. Content digest는 이 설치 bytes를, consent digest는 content digest와 승인 시각을 포함한 approval payload 전체를 결합. Activation은 ledger, 현재 resolution, descriptor bytes와 두 digest를 다시 검증.

Detection이 `available`, `incompatible` 또는 `unknown`이면 기존 Hive hook도 neutral/inert. Fallback hook의 `UserPromptSubmit` classification, prompt rewrite, Skill activation, orchestration, automatic memory ingest, continuation 수행 금지. `Stop`: 모든 입력·승인 상태에서 neutral allow, continuation loop 생성 금지. 상세 계약: [`../architecture/hook-consent.md`](../architecture/hook-consent.md).

Semantic Skill routing은 host Skill discovery, narrow descriptions와 compact `AGENTS.md` precedence가 담당. OMX/OMC의 keyword detector, classifier, Stop continuation 복제 금지.
