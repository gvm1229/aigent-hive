# 1. 목표와 완료 정의

Aigent Hive는 사용자가 이미 로그인한 Codex, Claude Code, Gemini Antigravity 위에서 동작하는 로컬 agent harness. Rust CLI가 프로젝트 setup, 지침 projection, Markdown memory, SQLite 검색 인덱스, 검증 계약과 update를 관리. 모델 실행, subagent process와 지속 loop는 Codex의 호환 가능한 OMX, Claude Code의 호환 가능한 OMC, 또는 그 둘이 없을 때 host native capability가 소유.

완성된 제품의 사용자 흐름:

1. 사용자가 release 또는 host의 얇은 integration을 통해 Hive CLI 설치
2. 프로젝트에서 `setup-harness` 또는 `hive setup` 실행
3. Hive가 프로젝트를 read-only 조사하고 미확정 항목을 한 번에 하나씩 질문
4. Hive가 현재 host에서 OMX/OMC capability를 우선 resolve하고, `absent|incompatible|unknown`이면 truthful host-native 경로를 사용하며 conclusive `absent`에서만 선택적 fallback hook을 검토
5. 사용자가 지속형 역할, memory 범위, usage threshold, judge 정책, optional Skill과 조건부 fallback hook을 capability 단위로 승인
6. Hive가 staging render와 conflict 검사를 거쳐 local harness 생성
7. 단순 질문은 harness를 로드하지 않는 격리 경로로 즉시 응답
8. prompt 작성·개선 요청은 `hive-prompt-refine`가 의도를 보존한 copy-ready prompt로 정제
9. 일반 작업은 요청에 맞는 승인된 Skill을 자동 선택하며 OMX/OMC capability를 Hive duplicate 대비 우선 적용
10. 작업은 durable role/run Markdown을 사용하며 resolved owner가 subagent와 지속 실행 담당
11. Source 개발에서는 매 turn과 action 경계에서 session-wide usage guard를 검사하고,
    consumer harness에서는 각 새 automatic dispatch 전에 local subscription usage
    guard 검사
12. deterministic verification 후 필요 시 독립 hostile judge quorum 실행
13. 검증된 현재 지식만 Karpathy식 Raw/Wiki/Schema에 반영하고 SQLite 재색인
14. 모든 필수 criterion 통과 후 완료 보고와 재개 가능한 handoff 저장
15. 사용자는 한 action으로 signed update를 적용하고 같은 major 호환 또는 cross-major 자동 migration 수행

### 1.1 제품 불변 조건

- Hive는 model-provider API를 직접 호출 금지.
- Hive는 provider API key를 질문·저장·전달 금지.
- subscription 인증, model call, model retry와 billing은 host 소유.
- source workspace, release bundle, consumer harness는 별도 artifact.
- 지식, role identity, run plan/status와 evidence manifest는 tracked Markdown이 정본.
- setup answers, typed config, optional Skill approval ledger와 suppression fingerprint는 tracked YAML/TOML이 정본.
- 작은 비기밀 Raw source object의 원본 format 보존 가능.
- SQLite는 삭제 가능한 FTS·tag·link projection이며 Git에 포함 금지.
- SQLite에만 존재하는 durable fact는 금지.
- setup/update는 Hive-owned path, Hive marker와 별도 승인된 exact project-local hook entry 밖을 수정 금지.
- optional Skill은 이름·source·revision·content digest·권한을 보여주고 개별 수동 승인.
- 승인된 Skill은 narrow description과 compact routing directive에 따라 이름 지정 없이 관련 task에서 자동 선택 가능.
- simple-question gate는 Skill routing보다 먼저 실행하며 unrelated Skill, project memory와 hook context를 로드 금지.
- 사람용 프로젝트 문서 기본값: 별도 언어 요청이 없으면 간결한 한국어와 명사형 중심, declarative·polite sentence-form 종결 회피
- Hive는 plan, Ralph, team, swarm 또는 provider session runtime을 재구현 금지.
- Codex에서는 호환 가능한 OMX, Claude Code에서는 호환 가능한 OMC가 host-native duplicate 대비 우선.
- orchestration mode를 setup preference로 질문 금지. 각 새 run에서 capability 기반 owner 하나를 resolve하고 run 종료까지 고정.
- resolved runtime이 실패해도 다른 runtime으로 조용히 fallback 금지.
- OMX/OMC가 감지되면 Hive lifecycle hook을 설치·활성화 금지.
- OMX/OMC가 conclusively absent일 때만 hook capability·event·path·digest를 설명하고 사용자 승인을 요청. 거절은 완전히 지원되는 상태.
- `hive-prompt-refine`는 명시적인 prompt 작성·정제 intent에서만 사용하며 모든 사용자 prompt를 자동 재작성 금지.
- 지속형 전문가는 stable role identity. 영구 process 없음.
- `100% complete`는 모든 필수 criterion의 boolean PASS를 뜻.
- 안전, 권한, 사용량 guard, 사용자 취소와 외부 blocker는 “계속 실행”보다 우선.
- 제작 agent의 reasoning transcript는 judge 입력에서 제외.
- elevated risk는 2/3 quorum, critical risk는 3/3 + human approval을 요구.
- deprecated 또는 superseded 지식은 active tree와 SQLite에서 삭제.
- 일반 삭제의 복구 이력은 Git이 소유하며, secret/legal erase만 별도 history purge를 사용.
- update backup은 최대 7일만 유지.
- 비기밀 canonical file은 Git 추적이 기본이며 runtime/cache/SQLite/backup은 제외.
- `X.Y.Z` version에서 같은 `X` 안의 upgrade만 non-breaking을 보장.
- 현재 product version은 마지막 완료 milestone인 Phase 6 signed release/update와 safe migration `0.7.0`.
- backward-compatible feature는 원칙적으로 `Y`, 빠른 호환 bugfix는 `Z`를 증가시킴.
- `X` 증가는 사용자가 목표 major를 명시적으로 지시한 경우에만 준비·적용할 수 있으며 automation이 추론하거나 자동 증가 금지.
- cross-major update는 경고, dry run, 자동 migration과 사용자 data 무손실 검증 없이는 commit 금지.

### 1.2 명시적 비목표

- OpenClaw 같은 상시 control plane
- cloud DB와 Hive 운영 서버
- provider API SDK
- 자체 model router
- 자체 subagent launcher/scheduler
- OMX/OMC의 plan·Ralph·team 복제
- web dashboard와 별도 desktop app
- vector DB 기본 도입
- 사용자의 `.omx`, `.omc`, host-global config 또는 foreign host entry 관리
- 사용자 동의 없는 Skill 자동 수집·활성화
- 모든 prompt를 가로채는 hidden prompt rewrite
- OMX/OMC와 중복되는 Skill classifier 또는 Stop continuation hook

### 1.3 Product version 정책

Plan revision과 product version은 독립. 현재 plan revision: `1.28`. 현재 완료 artifact: `0.7.0`.

Version 정본과 projection:

| Surface | 역할 |
| --- | --- |
| root `Cargo.toml`의 `workspace.package.version` | source와 Rust package version 정본 |
| `hive --version` | compiled CLI version |
| release manifest와 provenance | binary/template/schema/migration bundle version |
| consumer `.hive/config/harness.toml` | 설치된 harness version과 source release identity |

`X.Y.Z` 증가 규칙:

- `Y`: backward-compatible user-facing feature, 새 Skill, 새 schema capability 또는 새 host projection
- `Z`: 같은 feature contract 안의 작은 bugfix, security fix, packaging·documentation correction
- versioned artifact 동작이 바뀌지 않는 plan-only edit는 product version을 증가 금지
- `X`: breaking contract 또는 compatibility baseline 변경. 사용자가 정확한 major target을 명시해야 하며 release tooling은 그 지시와 human confirmation 없이는 거부

Hive 호환 정책은 SemVer의 일반적인 `0.y.z` 관행 대비 강한 기준. `0.1.0 → 0.n.z`도 같은 major `0`이므로 non-breaking 필수. Breaking change는 자동 minor bump가 아닌 사용자 승인 next major에서만 수행.
