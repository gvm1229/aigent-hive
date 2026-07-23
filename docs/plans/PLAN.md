# Aigent Hive 구현 계획

> Revision: 1.8
> 상태: 구현 기준본
> 기준일: 2026-07-23
> 현재 제품 version: `0.3.0`
> 정본 위치: `docs/plans/PLAN.md`

이 문서는 Aigent Hive의 완성된 사용자 흐름과 이를 구현하는 순서를 하나의 연속된 계획으로 정의한다. 이전 `PLAN_v1*` 문서는 현재 지침으로 사용하지 않는다.

현재 `0.3.0`은 Phase 1 결정적 setup renderer와 Phase 2 canonical Markdown knowledge·disposable SQLite index 계약을 구현한다. 아래 Skill routing, host projection, durable run과 update flow는 해당 milestone의 unchecked acceptance를 통과하기 전까지 구현·지원된 것으로 간주하지 않는다.

## 1. 목표와 완료 정의

Aigent Hive는 사용자가 이미 로그인한 Codex, Claude Code, Gemini Antigravity 위에서 동작하는 로컬 agent harness다. Rust CLI가 프로젝트 setup, 지침 projection, Markdown memory, SQLite 검색 인덱스, 검증 계약과 update를 관리한다. 모델 실행, subagent process와 지속 loop는 Codex의 호환 가능한 OMX, Claude Code의 호환 가능한 OMC, 또는 그 둘이 없을 때 host native capability가 소유한다.

완성된 제품의 사용자 흐름:

1. 사용자가 release 또는 host의 얇은 integration을 통해 Hive CLI 설치
2. 프로젝트에서 `setup-harness` 또는 `hive setup` 실행
3. Hive가 프로젝트를 read-only 조사하고 미확정 항목을 한 번에 하나씩 질문
4. Hive가 현재 host에서 OMX/OMC capability를 우선 resolve하고, 없을 때만 host-native 경로와 선택적 fallback hook을 검토
5. 사용자가 지속형 역할, memory 범위, usage threshold, judge 정책, optional Skill과 조건부 fallback hook을 capability 단위로 승인
6. Hive가 staging render와 conflict 검사를 거쳐 local harness 생성
7. 단순 질문은 harness를 로드하지 않는 격리 경로로 즉시 응답
8. prompt 작성·개선 요청은 `hive-prompt-refine`가 의도를 보존한 copy-ready prompt로 정제
9. 일반 작업은 요청에 맞는 승인된 Skill을 자동 선택하며 OMX/OMC capability를 Hive duplicate보다 우선
10. 작업은 durable role/run Markdown를 사용하며 resolved owner가 subagent와 지속 실행 담당
11. 각 새 delegation 전에 local subscription usage guard 검사
12. deterministic verification 후 필요 시 독립 hostile judge quorum 실행
13. 검증된 현재 지식만 Karpathy식 Raw/Wiki/Schema에 반영하고 SQLite 재색인
14. 모든 필수 criterion 통과 후 완료 보고와 재개 가능한 handoff 저장
15. 사용자는 한 action으로 signed update를 적용하고 같은 major 호환 또는 cross-major 자동 migration 수행

### 1.1 제품 불변 조건

- Hive는 model-provider API를 직접 호출하지 않는다.
- Hive는 provider API key를 질문·저장·전달하지 않는다.
- subscription 인증, model call, model retry와 billing은 host 소유다.
- source workspace, release bundle, consumer harness는 별도 artifact다.
- 지식, role identity, run plan/status와 evidence manifest는 tracked Markdown가 정본이다.
- setup answers, typed config, optional Skill approval ledger와 suppression fingerprint는 tracked YAML/TOML이 정본이다.
- 작은 비기밀 Raw source object는 원본 format을 보존할 수 있다.
- SQLite는 삭제 가능한 FTS·tag·link projection이며 Git에 포함하지 않는다.
- SQLite에만 존재하는 durable fact는 금지한다.
- setup/update는 Hive-owned path, Hive marker와 별도 승인된 exact project-local hook entry 밖을 수정하지 않는다.
- optional Skill은 이름·source·revision·content digest·권한을 보여주고 개별 수동 승인한다.
- 승인된 Skill은 narrow description과 compact routing directive에 따라 사용자가 이름을 말하지 않아도 관련 task에서 자동 선택할 수 있다.
- simple-question gate는 Skill routing보다 먼저 실행하며 unrelated Skill, project memory와 hook context를 로드하지 않는다.
- Hive는 plan, Ralph, team, swarm 또는 provider session runtime을 재구현하지 않는다.
- Codex에서는 호환 가능한 OMX, Claude Code에서는 호환 가능한 OMC가 host-native duplicate보다 우선한다.
- orchestration mode를 setup preference로 묻지 않는다. 새 run마다 capability에서 owner 하나를 resolve하고 run 종료까지 고정한다.
- resolved runtime이 실패해도 다른 runtime으로 조용히 fallback하지 않는다.
- OMX/OMC가 감지되면 Hive lifecycle hook을 설치·활성화하지 않는다.
- OMX/OMC가 conclusively absent일 때만 hook capability·event·path·digest를 설명하고 사용자 승인을 요청한다. 거절은 완전히 지원되는 상태다.
- `hive-prompt-refine`는 명시적인 prompt 작성·정제 intent에서만 사용하며 모든 사용자 prompt를 자동 재작성하지 않는다.
- 지속형 전문가는 stable role identity이며 영구 process가 아니다.
- `100% complete`는 모든 필수 criterion의 boolean PASS를 뜻한다.
- 안전, 권한, 사용량 guard, 사용자 취소와 외부 blocker는 “계속 실행”보다 우선한다.
- 제작 agent의 reasoning transcript는 judge 입력에서 제외한다.
- elevated risk는 2/3 quorum, critical risk는 3/3 + human approval을 요구한다.
- deprecated 또는 superseded 지식은 active tree와 SQLite에서 삭제한다.
- 일반 삭제의 복구 이력은 Git이 소유하며, secret/legal erase만 별도 history purge를 사용한다.
- update backup은 최대 7일만 유지한다.
- 비기밀 canonical file은 Git 추적이 기본이며 runtime/cache/SQLite/backup은 제외한다.
- `X.Y.Z` version에서 같은 `X` 안의 upgrade만 non-breaking을 보장한다.
- 현재 product version은 Phase 2 Markdown knowledge와 disposable SQLite index milestone인 `0.3.0`이다.
- backward-compatible feature는 원칙적으로 `Y`, 빠른 호환 bugfix는 `Z`를 증가시킨다.
- `X` 증가는 사용자가 목표 major를 명시적으로 지시한 경우에만 준비·적용할 수 있으며 automation이 추론하거나 자동 증가하지 않는다.
- cross-major update는 경고, dry run, 자동 migration과 사용자 data 무손실 검증 없이는 commit하지 않는다.

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

Plan revision과 product version은 독립이다. 이 문서는 revision `1.8`이며 현재 구현 artifact는 `0.3.0`이다.

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
- versioned artifact 동작이 바뀌지 않는 plan-only edit는 product version을 올리지 않음
- `X`: breaking contract 또는 compatibility baseline 변경. 사용자가 정확한 major target을 명시해야 하며 release tooling은 그 지시와 human confirmation 없이는 거부

Hive는 SemVer의 일반적인 `0.y.z` 관행보다 강한 호환 정책을 사용한다. `0.1.0 → 0.n.z`도 같은 major `0`이므로 non-breaking이어야 한다. Breaking change는 자동 minor bump로 숨기지 않고 사용자가 승인한 next major에서만 수행한다.

## 2. Artifact와 source 구조

### 2.1 세 artifact

| Artifact | 정본 | 포함 | 금지 |
| --- | --- | --- | --- |
| Hive source | 이 Git 저장소 | Rust, schema, Copier source, Skill source, projection, fixture, docs | 실제 consumer state, credential |
| Release bundle | GitHub Release | signed binary, compiled template pack, schema, migration, manifest, provenance | source-only directive, mutable user data |
| Consumer harness | 사용자의 독립 프로젝트 | `.hive`, shared marker, approved projection, Markdown data | Hive source tree, plugin cache-only 정본 |

`hive-source.json`가 있는 target에는 consumer setup을 실행하지 않는다. Source root의 `.agents/`는 Hive 개발 전용이고 `harness/`만 출하 source다.

### 2.2 Durable state 정본

| Data class | 정본 format·위치 | SQLite 역할 |
| --- | --- | --- |
| knowledge | `.hive/knowledge/Raw`, `Wiki`, `Schema`의 tracked Markdown 또는 작은 원본 object | FTS, tag, alias, link, content-hash index |
| role identity | `.hive/team/roles/*.md` | 검색용 projection만 허용 |
| run plan/status/evidence manifest | `.hive/runs/**/*.md` | 검색·집계 cache만 허용 |
| setup answers | `.hive/setup-answers.yml` | 사용하지 않음 |
| typed config·role seed·knowledge scope | `.hive/config/*.{toml,yml}` | 사용하지 않음 |
| optional Skill approval | `.hive/config/approved-skills.yml` | 사용하지 않음 |
| fallback hook approval | `.hive/config/approved-hooks.yml` | 사용하지 않음 |
| deleted-content suppression | `.hive/knowledge/suppression.yml` | re-ingest filter projection 가능 |

Markdown body가 유리한 narrative state와 typed YAML/TOML이 유리한 configuration·consent state를 구분한다. 둘 다 tracked canonical source이며 어느 쪽도 SQLite에서 역으로 복구하지 않는다. 새 machine checkout은 이 tracked tree만으로 model call이나 network 없이 SQLite를 재구축할 수 있어야 한다.

### 2.3 Source workspace 목표 구조

```text
aigent-hive/
├── AGENTS.md
├── .agents/                       # Hive 개발 지침
├── crates/
│   ├── hive-core/                 # invariant와 ownership
│   ├── hive-cli/                  # user command
│   ├── hive-render/               # Phase 1
│   ├── hive-wiki/                 # Phase 2
│   ├── hive-projection/           # Phase 3
│   └── hive-update/               # Phase 6
├── harness/
│   ├── template/                  # Copier와 Rust가 공유하는 canonical template
│   ├── skills/                    # portable shipping Skill
│   ├── projections/               # host별 thin projection
│   ├── profiles/                  # general/custom과 검증된 domain 확장점
│   └── manifest.toml              # path ownership
├── schemas/
├── tests/
│   ├── fixtures/
│   ├── conformance/
│   └── work/                      # ignored disposable output
├── docs/
│   ├── plans/PLAN.md
│   ├── state/CURRENT.md
│   ├── decisions/
│   ├── architecture/
│   ├── research/
│   └── guides/
├── copier.yml
└── hive-source.json
```

빈 crate를 미리 생성하지 않는다. 구현과 acceptance가 함께 시작될 때 owning crate를 추가한다.

### 2.4 Source tracking

Git 추적:

- Rust source와 Cargo manifest/lock
- template, projection, profile, schema
- synthetic fixture와 normalized expected output
- `.agents`, `AGENTS.md`, thin host redirect
- plan, ADR, current state와 research
- CI와 release recipe

Git 제외:

- `target/`, `dist/`, `artifacts/`
- `.omx/`, `.omc/`, `.codex/`, `.claude/`
- `.agents/work/`
- `tests/work/`
- SQLite, WAL, SHM
- local backup, cache, temp file
- credential와 signing private key

## 3. 완성된 workflow와 구현

### Stage 0. 진입 라우팅

#### 완성 후 동작

사용자 요청을 다음 명시 action으로 분리:

- `SetupHarness`
- `AnswerSimpleQuestion`
- `RefinePrompt`
- `RunWork`
- `ResumeWork`
- `VerifyWork`
- `IngestKnowledge`
- `QueryKnowledge`
- `UpdateHarness`

자동 intent 추정은 보조 신호만 사용한다. 명시 action과 충돌하면 명시 action 우선.

Skill resolution은 action 분류 뒤 다음 precedence를 사용:

1. 사용자가 명시한 Skill 또는 명시한 direct/plain-answer 지시
2. simple-question gate
3. active host가 노출한 compatible OMX/OMC Skill
4. 승인되어 host discovery surface에 projection된 Hive 고유 Skill
5. host-native direct capability

Semantic routing은 Skill의 좁은 `description`과 generated `AGENTS.md`의 compact mapping으로 수행한다. Hive가 별도 `UserPromptSubmit` classifier hook을 만들지 않는다. Catalog가 크더라도 사용자가 승인한 Skill만 active projection에 들어가며, 한 task에는 필요한 최소 Skill 집합만 본문을 load한다.

#### 구현

- host projection은 action을 Hive namespace로 노출
- Rust CLI는 동일 action name과 `--output json`을 제공하고 `schemas/action-result.schema.json`을 준수
- action마다 허용 read/write/subagent/network capability 선언
- Skill catalog entry마다 `provided_by`, `superseded_by_external`, invocation intent와 side-effect class 선언
- unsupported action은 write 전에 종료

CLI process exit 의미:

| Exit | 의미 | `ActionResult.status` |
| ---: | --- | --- |
| `0` | 요청한 action 성공 | `success` |
| `2` | 잘못된 입력, schema 위반 또는 source-target 거부 | `blocked` 또는 `error` |
| `3` | 안전·승인 blocker 또는 ownership conflict | `blocked` 또는 `conflict` |
| `4` | 선택 host/runtime에서 capability 미지원 | `unsupported` |
| `5` | action 실행 후 필수 검증 실패 | `verification-failed` |
| `10` | 예기치 않은 내부 오류 | `error` |

`--output json`에서는 비정상 종료도 가능한 범위에서 schema-valid JSON 한 개만 stdout에 기록하고 human diagnostic은 stderr로 분리한다. Write 전 실패는 `changed_paths: []`, 모든 evidence는 locator와 `sha256` digest를 포함한다. 문자열 `code`는 exit number와 별개인 안정된 `hive.*` domain code다.

#### 완료 조건

- [ ] 세 host에서 같은 logical action이 같은 contract로 resolve
- [x] unknown action이 project write 또는 agent spawn 없이 실패
- [ ] host별 alias가 core state에 저장되지 않음
- [ ] investigate 요청에서 available OMX/OMC `analyze`가 Hive duplicate보다 우선
- [ ] approved Hive-only Skill은 이름을 직접 말하지 않아도 matching description에서 자동 선택
- [ ] simple-question fixture에서 Skill description 외 추가 Skill body load 0개
- [ ] explicit `plain answer`/`no workflow` 요청이 automatic Skill invocation보다 우선
- [x] 모든 exit class가 schema-valid `ActionResult`와 정확히 대응

### Stage 1. Read-only 조사와 setup 질문

#### 완성 후 동작

`setup-harness`는 먼저 repository를 조사:

- project root와 Git 상태
- 기존 `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- project manifest와 확인 가능한 domain
- 기존 Hive marker
- active host가 노출한 Skill/plugin capability metadata
- side-effect-free public `omx --version` 또는 `omc --version` 결과

그 뒤 preference만 한 번에 하나씩 질문:

1. project identity 확인
2. domain profile
3. primary host
4. persistent roles
5. knowledge ingest 범위
6. usage stop threshold
7. judge policy
8. optional Skills
9. OMX/OMC가 conclusively absent일 때만 optional fallback hooks
10. 최종 write preview

Orchestration owner는 preference가 아니다. Codex는 compatible OMX capability, Claude Code는 compatible OMC capability를 먼저 사용하고, 둘이 active host에서 확인되지 않을 때만 host-native로 resolve한다. Positive evidence는 host가 현재 session에 노출한 Skill/plugin metadata 또는 public executable의 side-effect-free `--version` 중 하나면 충분하다. Hive는 `.omx/`, `.omc/`, host-global config, session state와 plugin cache를 읽어 추론하지 않는다.

Detection 결과:

| 결과 | 의미 | setup 동작 |
| --- | --- | --- |
| `available` | compatible OMX/OMC capability 확인 | external owner 우선, Hive hook 질문·artifact 0개 |
| `absent` | host catalog와 public probe 모두 명확히 없음 | host-native, optional fallback hook consent 질문 가능 |
| `incompatible` | 설치 evidence는 있으나 지원 version/capability 아님 | 진단 후 해당 고급 기능 `unsupported`, hook fallback 금지 |
| `unknown` | probe surface가 없어 부재를 증명하지 못함 | host-native best-effort, hook fallback 금지 |

한 run이 시작되면 resolved owner와 evidence digest를 `STATUS.md`에 고정한다. Run 도중 environment가 바뀌어도 조용히 owner를 교체하지 않는다. 다음 새 run 또는 명시 reconfigure에서 다시 resolve한다.

Optional Skill은 자동 추천할 수 있지만 각 항목을 개별 승인받는다. 승인 화면은 name, source, immutable revision, content digest와 `requested_capabilities`를 모두 표시한다. 사용자는 capability별로 승인하며 `approved_capabilities ⊆ requested_capabilities`여야 한다. 승인 시각과 전체 consent payload digest를 함께 기록한다. 승인하지 않은 Skill 또는 capability는 download, render, discovery root 배치, hook 등록과 실행을 하지 않는다.

Consent v1은 `consent_version`, name, source, revision, `content_digest`, 정렬된 requested/approved capability와 UTC-seconds `approved_at`을 RFC 8785 JCS로 canonicalize한 UTF-8 bytes의 SHA-256이다. 정확한 계약은 `docs/architecture/skill-consent.md`를 따른다. Hive는 staging, projection, activation과 migration activation 전에 digest를 재계산한다. Field가 하나라도 바뀌거나 digest가 다르면 자동 재서명하지 않고 Skill을 inert로 두며 재승인을 요구한다.

Fallback hook은 Skill consent와 별도 approval object다. `absent`일 때만 다음과 같은 capability preview를 한 번에 보여준다.

> 이 프로젝트에서 compatible OMX/OMC installation을 감지하지 못했습니다. Aigent Hive는 선택적으로 project-local fallback hooks를 설치할 수 있습니다. 이 hooks는 `.hive/` ownership 위반 사전 경고, durable run checkpoint 누락 감지, update/migration 무결성 검사와 bounded diagnostic만 수행합니다. Skill routing, prompt 재작성, subagent orchestration, 자동 memory ingest 또는 Stop continuation은 수행하지 않습니다. 설치할 event, project-local path, executable command, requested capability와 content digest는 아래 preview와 같습니다. 설치하시겠습니까?

사용자는 전체 bundle이 아니라 표시된 hook capability별로 승인하거나 모두 거절할 수 있다. 거절해도 setup은 성공하며 host-native 기능만 사용한다. 승인 record는 `.hive/config/approved-hooks.yml`에 tracked canonical data로 저장한다. Approval이 없거나 digest/path/event가 변하면 hook을 render·등록·실행하지 않는다.

허용하는 fallback hook capability:

| Capability | 가능한 host event | 동작 |
| --- | --- | --- |
| `protect-hive-owned-state` | `PreToolUse` | manifest 밖에서 `.hive` protected state를 파괴·우회하는 명백한 mutation을 경고 또는 차단 |
| `update-integrity-guard` | `PreToolUse` | update/migration command가 dry-run·backup·staging gate를 우회하는지 확인 |
| `derived-state-invalidation` | `PostToolUse` | canonical file 변경 뒤 SQLite/evidence projection을 stale로 표시; model-visible 장문 출력 없음 |
| `checkpoint-reminder` | `PreCompact`가 있는 host, 그 외 non-blocking `Stop` | active durable run의 `STATUS.md` checkpoint 누락을 bounded diagnostic으로 알림 |

`UserPromptSubmit`, prompt rewrite, automatic Skill activation, automatic Wiki ingest, subagent spawn와 continuation decision은 fallback hook capability에 포함하지 않는다. `Stop` handler는 항상 neutral/empty allow 결과로 끝나며 `decision:block`, `continue`, 재호출 prompt를 반환하지 않는다. 동일 input digest의 재실행은 idempotent하고 timeout/error는 orchestration을 반복시키지 않은 채 진단만 남긴다.

OMX/OMC가 이후 감지되면 기존 Hive fallback hook은 즉시 neutral/inert 동작만 하고, 다음 `hive setup --reconfigure` 또는 `hive update` preview가 Hive-owned entry 제거를 제안한다. 제거 전후 foreign hook entry와 user byte는 보존한다.

Descriptor bytes, content/consent digest와 activation-time 재검증의 normative contract는 [`../architecture/hook-consent.md`](../architecture/hook-consent.md)를 따른다.

#### Copier 경계

Copier 9.17.0은 template authoring, 질문 UX 검토와 CI parity test에 사용한다.

- source root의 `copier.yml`이 question schema 정본
- `schemas/setup-answers.schema.json`이 answer의 machine contract
- `harness/template/`가 단일 template source
- CI가 Copier static render 결과와 Rust static renderer 결과 비교
- dynamic role materialization은 versioned role contract known-answer fixture와 Rust output 비교
- release는 compiled template pack 포함
- consumer는 Python이나 Copier 불필요
- Copier는 live project update authority가 아님

#### 구현

- `schemas/setup-answers.schema.json`으로 answer 검증
- setup answer에서 `orchestration_layer` preference 제거
- host capability probe를 `available|absent|incompatible|unknown`으로 정규화
- source-root guard를 모든 write보다 먼저 실행
- staging directory에 render
- manifest 기반 path·marker ownership 검증
- dry-run diff와 conflict 출력
- setup answers를 `.hive/setup-answers.yml`에 저장
- `persistent_roles`를 `.hive/config/role-seeds.yml`에, ingest include/exclude 범위를 `.hive/config/knowledge-scope.yml`에 projection
- 승인한 Skill만 `.hive/config/approved-skills.yml`에 immutable provenance, capability grant와 consent digest로 저장
- 승인한 fallback hook만 `.hive/config/approved-hooks.yml`과 project-local namespaced projection으로 저장
- role seed를 staging에서 `.hive/team/roles/<role-id>.md` canonical role로 materialize

#### 완료 조건

- [x] 같은 answer로 두 번 render한 normalized digest 동일
- [x] Codex+compatible OMX는 자동 OMX, Claude+compatible OMC는 자동 OMC로 resolve
- [x] Antigravity 또는 external capability 부재 host는 host-native로 resolve
- [x] `incompatible|unknown`에서 Hive fallback hook install 0개
- [x] external capability detected 상태에서 hook 질문·artifact·command 0개
- [x] external capability absent + hook 거절 setup 성공, hook artifact 0개
- [x] external capability absent + 일부 hook 승인 시 승인 event/capability만 projection
- [x] hook event/path/capability/digest tamper 시 activation 0회와 재승인 요구
- [x] fallback `Stop` fixture가 모든 상태에서 block/continue prompt 0개와 neutral output
- [x] 동일 hook input replay에서 canonical file 중복 mutation 0개
- [x] optional Skill 0개 승인 setup 성공
- [x] 승인하지 않은 Skill output 0개
- [x] `approved_capabilities`가 `requested_capabilities`를 벗어나면 staging 전 거부
- [x] Skill provenance/capability/timestamp 어느 한 field tamper도 기존 consent digest로 activation 불가
- [x] role seed와 knowledge scope가 setup answer에서 손실 없이 render
- [x] 모든 role seed가 schema-valid role file 하나로 materialize되고 두 번째 setup은 byte-identical
- [x] `hive-source.json` target write 0개
- [x] Copier/Rust static tree parity와 role materialization known-answer parity

### Stage 2. Harness 생성과 ownership 적용

#### 완성 후 동작

Hive는 consumer project에 다음만 생성:

- `.hive/config/`
- `.hive/knowledge/`
- `.hive/team/roles/`
- `.hive/runs/`
- `.hive/index/` runtime 위치
- root shared file의 exact Hive marker
- 사용자가 승인한 namespaced Skill/projection
- external capability absent와 별도 consent가 모두 성립할 때만 project-local Hive fallback hook projection

기존 root 문서가 있으면 전체 overwrite하지 않고 Hive marker만 merge. 손상·중첩 marker는 자동 추정하지 않고 conflict로 중지.

Setup-time role lifecycle:

1. `.hive/config/role-seeds.yml`은 승인한 초기 definition과 reconfigure preference
2. setup staging에서 각 seed를 `.hive/team/roles/<role-id>.md`로 materialize
3. materialize 후 role Markdown가 identity·assignment·handoff의 runtime 정본이며 runtime은 seed를 team member로 직접 사용하지 않음
4. 같은 seed 재적용은 no-op
5. 기존 role definition drift는 자동 overwrite하지 않고 conflict
6. 사용자가 reconfigure preview에서 명시 승인한 경우 definition field만 변경하고 assignment·handoff·body 보존
7. seed 제거만으로 role file을 삭제하지 않으며 명시 retire operation 필요

정확한 frontmatter/body, migration과 fixture 계약은 `docs/architecture/role-lifecycle.md`를 따른다.

#### 구현

- `harness/manifest.toml`을 compiled ownership manifest로 변환
- path traversal, absolute path, symlink escape 거부
- previous generated digest와 live bytes 비교
- shared marker 외 byte-preserving test
- generated file과 user-owned file 분류
- role ID 중복·path collision 거부와 role-profile schema 검증
- role file은 `canonical-data-protected`; update가 generated config처럼 overwrite/delete하지 않음
- host config는 foreign-owned가 기본이며 exact Hive hook entry만 `consented-shared-structure`로 merge
- JSON/TOML structured merge는 unknown key와 foreign array entry의 semantic identity·ordering·value를 보존하고 Hive-owned entry 외 삭제를 금지
- approved hook path는 project-local manifest allowlist 안에 있어야 하며 host-global path와 `.omx/.omc`는 항상 금지
- consumer `.hive/.gitignore`에는 SQLite와 short-lived backup만 제외

#### 완료 조건

- [x] 기존 user text와 external marker byte 동일
- [x] `.omx/.omc`, foreign runtime state와 host-global config read/write 0회
- [x] consent가 없는 `.codex/.claude/.agents` project hook projection read/write 0회
- [x] consented project-local hook merge 전후 foreign semantic tree와 entry digest 동일
- [x] generated path가 manifest 밖이면 setup 실패
- [x] canonical non-confidential files가 Git-visible
- [x] SQLite/WAL/SHM/journal, index stale/lock/temp와 backup만 consumer Git에서 제외
- [x] role reconfigure가 current assignment·handoff·user body를 보존
- [x] cross-major role candidate의 parse/schema 검증 실패 시 active role tree bytes 불변

### Stage 3. 단순 질문 격리

#### 완성 후 동작

명시 `AnswerSimpleQuestion` 또는 `hive-simple-question`은 다음 capability 없이 답변:

- project memory
- Wiki ingest/query
- repository mutation
- subagent
- external orchestration
- persistent run creation

질문 자체에 repository 정보가 필요하면 simple path를 거부하고 `RunWork` 전환을 제안. 자동 전환이나 write 없음.

#### 구현

- 최소 system contract만 포함한 portable Skill
- project root와 `.hive` mount 차단 가능한 host에서는 실제 차단
- 차단할 수 없는 host는 instruction-only로 표시하고 support matrix에서 구분
- simple response가 memory capture를 trigger하지 않게 함

#### 완료 조건

- [ ] simple fixture에서 project file read/write 0회
- [ ] subagent와 Skill 추가 load 0회
- [ ] project-dependent 질문은 명시 전환 전 실행 0회

### Stage 4. `hive-prompt-refine`

#### 완성 후 동작

`RefinePrompt` 또는 `hive-prompt-refine`은 사용자가 prompt를 작성·개선·구조화하거나 모호한 brief를 다른 agent가 실행할 copy-ready prompt로 바꾸려는 경우 자동 선택된다. 이름이나 역사적 별칭을 직접 말할 필요는 없지만, 일반 질문과 일반 작업 prompt를 몰래 가로채거나 재작성하지 않는다.

기본 mode는 `refine-only`다. Refined prompt를 실행하는 `refine-and-run`은 사용자가 같은 요청에서 명시한 경우에만 허용한다.

Flow:

1. 원문을 immutable input으로 보존하고 primary intent, target audience/agent와 원하는 결과를 추출
2. project-scoped prompt일 때만 허용된 read-only context에서 실제 path, command와 제약을 확인
3. objective, scope, inputs, constraints, acceptance, output shape와 stop condition의 ambiguity를 평가
4. 결과를 바꾸는 필수 정보가 없으면 한 번에 한 질문만 수행
5. 복잡한 요구사항 정제가 필요하고 OMX/OMC interview Skill이 available하면 기존 Skill을 사용하며, 없으면 host-native question surface만 사용
6. 사용자가 답하지 않은 비필수 항목은 명시적인 assumption 또는 placeholder로 남기고 사실을 창작하지 않음
7. provider-neutral prompt를 기본 생성하고 사용자가 target host를 지정했을 때만 Codex/Claude/Antigravity syntax로 얇게 projection
8. intent summary, assumptions/unresolved items와 copy-ready refined prompt를 분리해 반환
9. `refine-only`에서는 model execution, project mutation, subagent spawn, memory capture와 run 생성 0회
10. 사용자가 저장을 요청한 prompt만 지정한 tracked Markdown 또는 active run artifact에 기록

Refined prompt 최소 구조:

```text
Goal
Context and grounded inputs
Required workflow
Constraints and prohibited actions
Acceptance and verification
Output contract
Stop, blocker, and escalation conditions
```

이미 충분히 구체적인 prompt는 불필요하게 장문화하지 않는다. 원문의 tone, authority boundary, must/must-not와 명시된 tool/provider 선택을 보존하며, 개선 전후 meaning drift를 검사한다.

#### 구현

- `harness/skills/hive-prompt-refine/SKILL.md`에 narrow trigger와 `refine-only` default 정의
- provider-neutral `PromptRefinement` input/result schema와 normalized fixture 추가
- intent·constraint·acceptance preservation을 structural field와 text locator로 검증
- optional project grounding은 read-only capability로 분리하고 simple-question path와 memory ingest에서 격리
- OMX/OMC에 있는 analysis/interview capability는 호출 가능하지만 Hive가 classifier, interview engine 또는 prompt hook을 복제하지 않음
- prompt artifact에 original/refined text를 함께 저장할 때 secret scan과 explicit save consent 적용

#### 완료 조건

- [ ] “이 prompt를 개선해줘”, “agent에게 줄 prompt를 만들어줘” fixture에서 이름 언급 없이 `hive-prompt-refine` 선택
- [ ] 일반 질문·일반 coding request에서 automatic prompt rewrite 0회
- [ ] `refine-only`에서 project write, subagent, run creation과 memory capture 0회
- [ ] 원문 must/must-not, scope, target output과 user authority가 refined prompt에 손실 없이 존재
- [ ] 필수 ambiguity 질문은 한 번에 하나이며 답하지 않은 항목을 fabricated fact로 채우지 않음
- [ ] provider를 지정하지 않은 fixture에 provider-specific command·path 0개
- [ ] 이미 충분한 prompt는 normalized token/character budget을 불필요하게 확대하지 않음
- [ ] `refine-and-run`은 명시 intent 없이 activation 불가

### Stage 5. 지속형 역할과 host-owned orchestration

#### 완성 후 동작

지속형 팀원은 setup-time materialization을 거친 `.hive/team/roles/<role-id>.md`로 표현:

- 책임과 비책임
- context selector
- 허용 tool과 write scope
- 현재 assignment
- handoff와 verification 책임

Role identity는 session이 종료되어도 유지. 새 host session 또는 subagent가 같은 role document와 current handoff를 받아 역할을 계속 수행.

새 run의 실제 실행 owner는 사용자 선택 없이 다음 precedence로 resolve:

| 조건 | 실행 소유자 | Hive가 제공 | Hive가 제공하지 않음 |
| --- | --- | --- | --- |
| Codex + compatible OMX available | OMX | canonical project context와 coexistence contract | plan/Ralph/team 복제, OMX state 조작 |
| Claude + compatible OMC available | OMC | canonical project context와 coexistence contract | plan/Ralph/team 복제, OMC state 조작 |
| 그 외 | Codex/Claude/Antigravity native capability | role/run document, bounded brief, result schema | scheduler, model process |

Host-native capability가 부족하면 해당 기능은 `unsupported`. Hive가 hidden fallback을 만들지 않음.

#### 구현

- `.hive/team/roles/<role-id>.md`의 YAML frontmatter는 `schemas/role-profile.schema.json`으로 검증하고 Markdown body는 bounded handoff context로 사용
- task brief는 최소 required context와 exact acceptance만 포함
- result는 `schemas/action-result.schema.json`으로 changed artifact, evidence, blocker와 next action을 정규화
- session ID는 optional runtime binding일 뿐 role identity가 아님
- host/version/surface qualification은 `schemas/capability-matrix.schema.json`으로 기록
- owner resolution은 active host capability metadata와 public executable path/`--version` evidence로 결정하고 run status에 digest와 함께 pin
- Hive가 `omx setup/update`, `omc setup/update`, team lifecycle 또는 foreign state를 호출하지 않음
- resolved owner가 OMX/OMC이면 Hive hook과 duplicate Skill projection 0개
- resolved owner가 host-native여도 fallback hook은 Hive data integrity guard만 제공하고 orchestration 기능을 추가하지 않음
- non-clobber conformance fixture가 외부 namespace의 before/after checksum을 계산하며 Hive 제품 자체는 그 namespace를 읽지 않음

#### 완료 조건

- [ ] session 교체 후 role 책임·assignment·handoff 복구
- [ ] role document 없이 permanent team member claim 불가
- [ ] compatible external layer detected 시 Hive orchestration command/hook와 duplicate Skill 0개
- [ ] resolved external runtime이 run 도중 실패하면 host-native로 자동 전환하지 않음
- [ ] 새 run에서 external capability absent가 확인될 때만 host-native로 resolve
- [ ] fixture-side external namespace checksum 불변이고 Hive process의 foreign namespace read/write 0회

### Stage 6. Durable run과 100% completion

#### 완성 후 동작

장기 작업은 `.hive/runs/<run-id>/`에 저장:

```text
PLAN.md
STATUS.md
evidence/
```

`PLAN.md`:

- 목표와 범위
- 필수 acceptance checklist
- task dependency와 owner role
- verification method
- 위험 tier
- stop/block condition

`STATUS.md`:

- current revision
- 완료·진행·blocked task
- active role binding
- next action
- latest evidence locator
- resume note

사용자가 “100% 완료까지 계속”을 요청하면 resolved runtime이 plan의 미통과 criterion을 기준으로 execute→verify→repair를 지속. Harness는 durable run contract를 제공하고 runtime loop 자체는 구현하지 않음.

#### 구현

- checkbox와 criterion ID를 parser로 검증
- `STATUS.md` YAML frontmatter를 `schemas/run-status.schema.json`으로 검증
- 완료율은 필수 criterion PASS 수만 계산
- transcript 대신 bounded status/handoff 저장
- compaction, session 종료, handoff 전 `STATUS.md` 갱신
- 같은 artifact/evidence hash의 중복 result 멱등 처리
- runtime이 지속 loop를 지원하지 않으면 `resume-ready`까지만 제공하고 unattended completion을 지원했다고 표시하지 않음

#### 완료 조건

- [ ] criterion 하나가 unchecked/failed/unverified면 성공 불가
- [ ] fresh session이 PLAN+STATUS+evidence만으로 next action 복구
- [ ] transcript 없이 재개 가능
- [ ] 안전·승인·usage blocker에서 무한 반복하지 않음
- [ ] resolved runtime capability와 실제 제품 표시 일치

### Stage 7. Subscription usage guard

#### 완성 후 동작

기본 설정은 신뢰 가능한 sensor가 보고한 `remaining <= 10%`일 때 새 자율 delegation 또는 loop iteration을 시작하지 않는 것.

Hive는 provider API를 호출하지 않는다. Sensor 후보는 host가 로컬로 노출하는 command/file/status 또는 CodexBar 같은 별도 local tool.

Snapshot freshness:

- 새 delegation 직전 sample
- 한 번의 host dispatch가 발생하면 즉시 만료
- dispatch가 없더라도 sensor가 선언한 TTL 또는 Hive 최대 TTL 중 짧은 값 사용
- missing, stale, account/window 불일치, 역행 값은 `usage_unknown`

`usage_unknown`에서는 automatic continuation fail-closed. 이미 시작된 dispatch를 강제 종료한다고 주장하지 않으며, 다음 dispatch 전 새 snapshot이 필요하다.

#### 구현

`UsageSnapshot` 최소 필드:

- sensor ID/version
- host/account scope
- quota window
- remaining percent
- measured at
- expires at
- source confidence

CodexBar adapter는 pinned-qualified CLI의 `usage` JSON에서 active account와 quota window를 검증한다. `guard` 결과만으로는 account와 freshness를 증명할 수 없으므로 정본 snapshot으로 사용하지 않는다. Session window가 있으면 이를 우선하고, host가 session limit을 제공하지 않을 때 weekly window를 fallback으로 선택한다. 둘 다 없거나 선택된 window의 `remaining <= 10%`이면 새 permit을 발급하지 않는다.

Adapter는 side-effect-free local command만 실행. Hive는 model call retry를 하지 않으며 local sensor read도 bounded attempt 후 unknown 처리.

#### 완료 조건

- [ ] API endpoint와 provider SDK dependency 0개
- [ ] 10% 경계에서 새 automatic dispatch 0개
- [ ] stale/missing/mismatched sensor가 `usage_unknown`
- [ ] sensor가 없는 host에서 enforcement 가능하다고 표시하지 않음
- [ ] CodexBar가 없어도 core setup·memory·update 동작

### Stage 8. Deterministic verification과 hostile judge

#### 완성 후 동작

먼저 build, test, lint, schema, file ownership, link 검사처럼 결정론적 검증 수행. 그 뒤 사용자가 요청하거나 risk policy가 요구하면 독립 judge 실행.

Judge가 받는 context:

- 원래 목표와 acceptance
- artifact 또는 diff
- fresh verification evidence
- 알려진 제약

Judge가 받지 않는 context:

- task agent의 chain-of-thought 또는 reasoning
- task agent의 자기 점수·자기 칭찬
- 원하는 verdict를 암시하는 지시
- 다른 judge의 verdict

Risk tier:

| Tier | 예 | 판정 |
| --- | --- | --- |
| normal | 작은 문서·저위험 코드 | 요청 시 independent judge 1명 |
| elevated | cross-file architecture, migration, 보안 경계 | 3명 중 2명 PASS |
| critical | release signing, destructive migration, security-sensitive update | 3명 전원 PASS + human approval |

Judge는 `PASS`, `FAIL`, `INDETERMINATE`만 반환. `FAIL`은 재현 가능한 finding, `INDETERMINATE`는 부족한 evidence를 명시.

#### 구현

- host 또는 external runtime의 clean independent agent 사용
- 각 judge에 `schemas/judge-package.schema.json`을 따르는 동일 digest의 최소 context envelope 개별 전달
- 각 결과는 `schemas/judge-verdict.schema.json`으로 검증
- verdict 전에 다른 judge 결과 공개 금지
- quorum 계산은 deterministic code
- FAIL finding은 affected criterion/task에 연결

#### 완료 조건

- [ ] task agent가 자신의 결과를 최종 승인하지 않음
- [ ] judge 간 verdict leakage 0회
- [ ] 2/3와 3/3+human gate unit test
- [ ] missing evidence는 PASS가 아닌 INDETERMINATE
- [ ] verdict의 `package_digest`가 원본 judge package와 다르면 quorum 제외
- [ ] critical human approval 없이 completion 불가

### Stage 9. Karpathy Raw/Wiki/Schema memory

#### 완성 후 동작

```text
.hive/knowledge/
├── Raw/
├── Wiki/
│   ├── index.md
│   └── log.md
├── Schema/
│   └── schema.md
└── suppression.yml
```

#### Raw

- 원본·정규화 source
- 기존 source 내용 직접 수정 금지
- source 변경은 새 revision
- 폐기된 source는 active tree에서 삭제
- 기밀정보·credential 저장 금지

#### Wiki

- agent-maintained interlinked Markdown
- source-summary, entity, concept, comparison, synthesis, open-question 단위
- hashtag/tag와 Raw locator
- 새 source를 기존 관련 page에 누적 통합
- contradiction은 양쪽 source를 표시
- deprecated/superseded page는 active tree에서 삭제

#### Schema

- page kind, frontmatter, relation, tag와 operation contract
- provider instruction file과 분리
- 사용자와 agent가 versioned migration으로 공동 진화

#### 삭제와 suppression

삭제 순서:

1. obsolete Raw/Wiki/page/claim을 active tree에서 제거
2. backlink, index와 tag 제거
3. 최소 suppression entry 기록
4. SQLite rebuild 또는 incremental delete
5. stale reference lint

Suppression entry는 fingerprint, source locator, reason, replacement와 timestamp만 포함.
`reason`은 shipped stable reason-code enum이며 삭제 본문을 복제하지 않음.

일반 삭제는 Git history에서 복원 가능. Secret/legal erase만 별도 승인된 history rewrite와 backup purge 수행.

#### 병렬 처리

여러 source의 extraction은 독립 read-only agent로 병렬화 가능. Canonical Wiki integration은 한 curator가 serial하게 수행해 duplicate, contradiction, backlink와 index를 한 번에 정산.

#### SQLite

용도:

- FTS5 full-text search
- tag·alias index
- backlink·source graph
- incremental indexing용 content hash
- ranking cache

`hive index rebuild`:

1. canonical Markdown scan
2. frontmatter/tag/link/source parse
3. temp SQLite 생성
4. page count·logical digest·query fixture 검증
5. atomic replace

Model call과 network 불필요. SQLite byte hash 동일성은 요구하지 않음.

#### 완료 조건

- [x] SQLite 삭제 후 같은 page ID·tag·link·content digest 재구축
- [x] 동일 Markdown checkout에서 query result equivalence
- [x] deprecated/superseded content가 active Wiki와 search 결과에 없음
- [x] suppression ledger에 삭제 본문 없음
- [x] contradiction, orphan, broken link, missing citation, stale index 탐지
- [x] parallel extraction + serial integration에서 lost update 없음
- [x] Git LFS 없이 canonical knowledge 동기화 가능

### Stage 10. 완료 보고와 재개

#### 완성 후 동작

완료 조건:

- 모든 필수 criterion PASS
- deterministic verification PASS
- 필요한 judge quorum PASS
- active write/effect 없음
- current STATUS와 evidence locator 저장
- Wiki update 또는 skip reason 정산

보고:

- 생성·변경·삭제 artifact
- 실행한 검증과 결과
- judge verdict
- 사용량 상태
- memory 변경
- optional remaining work
- 재개가 필요할 때 정확한 next action

#### 구현

- completion report를 tracked Markdown로 생성
- 같은 report hash 중복 생성 방지
- blocker는 원인, 이미 시도한 안전한 대안과 resume condition 기록
- stale run은 current state를 다시 검증한 뒤 재개

#### 완료 조건

- [ ] completion claim과 fresh evidence 1:1 연결
- [ ] 새 session이 정확한 next action 복구
- [ ] blocked 상태를 succeeded로 표시하지 않음

### Stage 11. Update와 migration

#### 완성 후 동작

사용자는 CLI 또는 host의 thin action에서 `hive update` 한 번 실행:

1. current install·origin·source/release/harness version 확인
2. GitHub Release metadata와 signature/provenance 검증
3. compatibility 계산과 경고
4. protected canonical tree snapshot과 backup
5. shadow directory에 새 template/projection render
6. same-major 또는 cross-major migration
7. ownership·schema·link·query·host smoke test
8. atomic activation
9. SQLite rebuild
10. 최대 7일 후 backup 삭제

#### Same-major

`X.a.b → X.c.d`:

- canonical schema와 setup answer backward compatible
- project/user content rewrite 없음
- additive projection과 index rebuild 허용
- breaking change 발견 시 release 자체를 거부

이 규칙은 major `0`에도 동일하게 적용한다. `0.1.0 → 0.2.0`은 feature addition일 수 있지만 existing supported contract를 깨뜨릴 수 없다.

#### Cross-major

`X.* → Y.*`:

- breaking change 경고
- source version별 signed migration route
- canonical Markdown와 preferences snapshot
- shadow successor에서 자동 변환
- project file, docs와 user-authored body 보존
- deprecated system representation만 새 format으로 재구성
- SQLite는 migrate하지 않고 새 schema로 rebuild
- conflict는 active install을 바꾸지 않고 중지

#### Installer와 release

- GitHub Releases가 artifact 정본
- macOS: direct bootstrap + Homebrew 편의 경로
- Windows: signed PowerShell bootstrap + WinGet 편의 경로
- package manager install은 self-updater가 managed binary를 덮어쓰지 않음
- host plugin은 update action을 노출하는 thin surface이며 제품 정본이 아님
- feature implementation은 release-facing contract가 완성되고 acceptance evidence가 생긴 commit에서 `Y`를 증가
- 같은 feature contract의 작은 compatible fix는 `Z`를 증가
- plan/docs-only change는 shipped behavior가 바뀌지 않으면 product version 유지
- release tooling은 root Cargo version, CLI `--version`, bundle manifest, migration table과 generated harness version 불일치를 거부
- next-major prepare는 user-supplied exact target와 별도 human confirmation이 모두 없으면 거부하며 자동 계산·자동 증가 금지

#### Signing

- GitHub artifact attestation과 provenance
- macOS Developer ID signing/notarization
- Windows Artifact Signing 또는 hardware-backed Authenticode key
- offline threshold root와 online release role 분리
- private key repository 저장 금지
- protected environment와 human approval gate

#### 완료 조건

- [ ] tampered/expired/rollback release 거부
- [ ] root Cargo, CLI, bundle와 installed harness version parity
- [ ] feature fixture가 patch-only bump를 거부하고 compatible minor bump를 요구
- [ ] bugfix fixture가 같은 minor의 patch bump로 upgrade
- [ ] explicit exact target와 human confirmation 없는 major bump 0회
- [ ] same-major 모든 supported fixture non-breaking
- [ ] cross-major project/docs/preferences 무손실
- [ ] migration failure 시 active generation 불변
- [ ] SQLite file을 backup/migration input으로 요구하지 않음
- [ ] user/external bytes와 namespace checksum 불변
- [ ] backup 7일 초과 자동 정리
- [ ] update와 knowledge deletion/GC가 같은 transaction에 없음

## 4. Consumer harness 구조

```text
consumer-project/
├── AGENTS.md                         # shared file, Hive marker only
└── .hive/
    ├── setup-answers.yml             # tracked, non-secret
    ├── config/
    │   ├── harness.toml              # tracked
    │   ├── role-seeds.yml            # tracked setup projection
    │   ├── knowledge-scope.yml       # tracked setup projection
    │   ├── approved-skills.yml       # tracked Skill consent ledger
    │   └── approved-hooks.yml        # tracked fallback hook consent ledger
    ├── team/
    │   └── roles/*.md                # tracked
    ├── runs/
    │   └── <run-id>/                 # tracked unless user marks confidential
    ├── knowledge/
    │   ├── Raw/                      # tracked, non-confidential only
    │   ├── Wiki/                     # tracked
    │   ├── Schema/                   # tracked
    │   └── suppression.yml           # tracked, no deleted prose
    ├── index/
    │   └── hive.sqlite               # ignored, rebuildable
    └── backups/                      # ignored, maximum 7 days
```

Consumer project는 독립 `.gitignore`를 소유한다. Hive 기본 권장은 canonical non-confidential Markdown/YAML/TOML과 Raw source object를 모두 추적하고 SQLite/WAL/SHM/journal, index stale/lock/temp, generated backup과 runtime cache만 제외하는 것.

Host가 발견하는 Skill과 consented fallback hook은 이 canonical config에서 thin projection한다. Host-specific project path는 ownership manifest에 별도로 열거하며, external runtime detected 상태에서는 fallback hook projection이 존재하지 않아야 한다.

## 5. Rust 구현 경계

### 5.1 Crate 책임

| Crate | 책임 | 금지 |
| --- | --- | --- |
| `hive-core` | source guard, ownership, schema-neutral invariant | host SDK, filesystem mutation orchestration |
| `hive-cli` | command parsing, user-facing result, port wiring | model API |
| `hive-render` | answer validation, template render, staging manifest | live uncontrolled write |
| `hive-wiki` | Markdown parse/lint, SQLite index/rebuild | canonical fact를 DB에만 저장 |
| `hive-projection` | host capability matrix와 thin file projection | model/session runtime |
| `hive-update` | signature, compatibility, backup, migration, atomic activation | knowledge GC, external plugin update |

Dependency 방향:

```text
hive-core
  ↑
hive-render / hive-wiki / hive-projection
  ↑
hive-update
  ↑
hive-cli
```

Provider SDK, orchestration package와 OMX/OMC source를 dependency graph에 넣지 않는다.

### 5.2 Host projection

Projection은 다음만 담당:

- common `AGENTS.md` 진입점 발견
- Hive namespaced Skill/action 노출
- approved Skill metadata와 compact routing precedence 노출
- active host의 OMX/OMC capability resolution과 version-pinned capability matrix 조회
- external capability absent와 hook consent가 모두 성립할 때만 Hive data-integrity hook adapter 노출
- signed CLI 호출
- result 표시

Projection은 model call, persistent process, team state, external orchestration hook와 global config를 소유하지 않는다. Consented fallback hook은 project-local Hive entry만 소유하며 semantic routing이나 continuation을 수행하지 않는다. OMX/OMC coexistence 검증은 synthetic fixture가 외부 tree를 준비하고 checksum을 비교하는 방식이며, 출하된 Hive process가 foreign runtime state를 관찰하는 방식이 아니다.

### 5.3 Skill catalog

Built-in:

- `setup-harness`
- 향후 `hive-simple-question`
- 향후 `hive-prompt-refine`
- 향후 `hive-knowledge-capture`
- 향후 `hive-knowledge-query`
- 향후 `hive-knowledge-maintenance`
- 향후 `hive-run-checkpoint`
- 향후 `hive-run-resume`
- 향후 `hive-role-handoff`
- 향후 `hive-judge-package`
- 향후 `hive-update`
- 향후 `hive-migrate`

포함하지 않음:

- plan clone
- Ralph clone
- team/swarm clone
- OMX/OMC alias

Optional third-party Skill은 quarantine→provenance 검증→사용자 개별 승인→namespaced projection 순서. Catalog 등록이나 추천은 activation이 아니며, 승인되지 않은 Skill은 host discovery surface에 넣지 않는다. 승인된 model-invocable Skill은 description이 task와 명확히 일치할 때 자동 선택할 수 있지만 side effect는 capability policy와 별도 runtime approval을 따른다.

## 6. 구현 마일스톤

### Phase 0. Source bootstrap — `0.1.0`

- [x] root `AGENTS.md`와 `.agents/directives/` 생성
- [x] source/release/consumer boundary 문서화
- [x] `hive-source.json` 추가
- [x] Rust workspace와 `hive-core`/`hive-cli` skeleton 생성
- [x] source-target guard 구현
- [x] Copier question/template source 생성
- [x] `setup-harness` Skill source 생성
- [x] setup/action/role/run/judge/capability machine contract schema 초안 생성
- [x] plan, CURRENT, ADR, Git guide 생성
- [x] local 또는 CI `fmt/clippy/test` PASS
- [x] Copier smoke test PASS
- [x] Skill validator PASS
- [x] `main` initial commit과 `develop` branch push
- [x] workspace product version `0.1.0` barebone baseline

### Phase 1. Deterministic setup renderer — `0.2.0`

- [x] `hive-render` crate 추가
- [x] Copier/Rust parity corpus
- [x] `orchestration_layer` setup preference를 schema/Copier/template/Skill에서 제거
- [x] host capability resolver와 `available|absent|incompatible|unknown` evidence contract
- [x] fallback hook capability preview·consent schema와 `.hive/config/approved-hooks.yml`
- [x] staging render와 ownership validator
- [x] ownership-class 보존과 target read 전 no-follow guard
- [x] 충돌하지 않는 exclusive temp, transactional rollback과 rollback-failure code
- [x] shared marker three-way merge
- [x] setup answer migration
- [x] role seed materializer와 idempotent/reconfigure fixture
- [x] RFC 8785 Skill consent verifier와 tamper fixture
- [x] fallback hook typed execution·revoke와 complete installed validation
- [x] schema-valid `UnknownAction` JSON과 write-free failure
- [x] `hive setup --dry-run|apply|validate`

### Phase 2. Markdown knowledge와 SQLite — target `0.3.0`

- [x] Wiki/frontmatter schema 확정
- [x] ingest/query/lint
- [x] suppression/delete workflow
- [x] FTS5/tag/link index
- [x] deterministic rebuild와 logical digest
- [x] parallel extraction/serial integration fixture

### Phase 3. Portable Skills와 host projection — target `0.4.0`

- [ ] `hive-simple-question`
- [ ] `hive-prompt-refine` refine-only/refine-and-run contract와 meaning-preservation corpus
- [ ] Hive-owned knowledge/run/role/judge/update Skill catalog
- [ ] approved-only active projection과 automatic minimal Skill routing
- [ ] Codex projection
- [ ] Claude Code projection
- [ ] Antigravity projection
- [ ] external capability absent + explicit consent 전용 fallback data-integrity hooks
- [ ] detected external runtime과 fallback hook의 mutual exclusion
- [ ] host/version capability matrix에 automatic-Skill-routing, prompt-refine와 hook event별 support/evidence 추가

### Phase 4. Role/run contract와 interoperability — target `0.5.0`

- [ ] RoleProfile와 Run schema parser·fixture·conformance 확정
- [ ] fresh-session resume fixture
- [ ] host-native subagent conformance
- [ ] run별 owner resolution evidence pin과 no-mid-run-switch
- [ ] OMX non-clobber/coexistence
- [ ] OMC non-clobber/coexistence
- [ ] external runtime missing/version drift negative test

### Phase 5. Usage guard와 judge quorum — target `0.6.0`

- [x] `UsageSnapshot` adapter interface
- [x] local sensor TTL/freshness
- [x] 10% threshold fail-closed
- [ ] CodexBar candidate qualification
- [ ] judge clean-context envelope
- [ ] 2/3, 3/3+human quorum

### Phase 6. Update, migration과 release — target `0.7.0`

- [ ] `hive-update` crate
- [ ] same-major compatibility corpus
- [ ] cross-major migration fixture
- [ ] backup/restore/7-day retention
- [ ] atomic activation과 crash recovery
- [ ] GitHub Release packaging
- [ ] Cargo/CLI/bundle/installed harness version parity
- [ ] feature→minor, compatible bugfix→patch release classification gate
- [ ] explicit user target와 human confirmation 없는 major bump 거부
- [ ] macOS/Windows signing
- [ ] direct/Homebrew/WinGet install path

### Phase 7. Public qualification — target `0.8.0`; `1.0.0`은 explicit major approval 전 금지

- [ ] macOS arm64/x86_64
- [ ] Windows x86_64
- [ ] 세 host base workflow E2E
- [ ] host-native/OMX/OMC support matrix
- [ ] upgrade/migration fault injection
- [ ] supply-chain provenance
- [x] public license 확정 — 전체 source·harness `Apache-2.0`, GitHub 감지와 REUSE 검증 완료
- [ ] `0.8.x` release candidate qualification
- [ ] 사용자가 정확히 `1.0.0` release를 지시한 뒤에만 stable major prepare

## 7. 핵심 conformance와 fault injection

| Scenario | 기대 결과 |
| --- | --- |
| source root setup | write 0회, 명확한 거부 |
| user `AGENTS.md` + Hive marker | marker만 변경 |
| OMC/OMX marker 공존 | external bytes 불변 |
| Codex/Claude + compatible OMX/OMC | external owner 자동 resolve, Hive duplicate Skill·hook 0개 |
| external runtime absent | host-native resolve, hook consent 질문 가능 |
| external runtime incompatible/unknown | 진단 또는 best-effort native, hook install 0개 |
| fallback hook 거절 | setup 성공, hook artifact·command 0개 |
| fallback hook 일부 승인 | 승인 event/capability/path만 projection |
| fallback hook 승인 후 external runtime 등장 | hook neutral/inert, reconfigure가 Hive entry만 제거 |
| optional Skill 미승인 | artifact·hook 0개 |
| 승인 capability가 요청 범위를 초과 | staging 전 거부 |
| Skill consent payload field 변조 | projection/activation 0회, 재승인 요구 |
| investigate 요청 + OMX/OMC analyze available | external `analyze` 자동 선택 |
| prompt 작성·개선 요청 | `hive-prompt-refine` 자동 선택, refine-only default |
| 일반 prompt | hidden prompt rewrite 0회 |
| prompt refine에서 필수 정보 누락 | 한 번에 한 질문 또는 explicit placeholder |
| role seed 재적용 | role file byte 변경 0건 |
| role definition drift | 명시 승인 전 conflict, assignment·handoff·body 보존 |
| CLI 실패 | exit class와 schema-valid `ActionResult` 일치 |
| SQLite 삭제 | Markdown에서 동일 logical index 재구축 |
| deprecated Wiki 삭제 | active query 0건, suppression metadata만 유지 |
| stale usage sensor | automatic continuation 0회 |
| sensor 없음 | 20% enforcement claim 없음 |
| task-agent self review | final approval 거부 |
| judge disagreement | tier quorum에 따라 FAIL/INDETERMINATE |
| session 종료 | PLAN/STATUS/evidence로 재개 |
| resolved OMX/OMC가 run 중 실패 | hidden fallback·owner switch 0회 |
| feature release | `Y` 증가와 same-major compatibility 검증 |
| compatible quick bugfix | `Z` 증가 |
| major bump 요청 없음 | `X` 증가 0회 |
| same-major breaking template | release/update 거부 |
| cross-major migration crash | active generation 불변 또는 forward recovery |
| update 중 user edit | conflict, user bytes 보존 |
| backup age > 7 days | expired backup만 정리 |
| tampered release | install/update 거부 |
| provider API dependency 추가 | architecture/CI gate 실패 |

## 8. 완료 gate

v1 public release는 다음을 모두 충족해야 한다.

- [ ] source, release, consumer tree 분리
- [ ] macOS·Windows signed CLI
- [ ] 세 host의 실제 capability matrix
- [ ] model-provider API dependency와 credential path 0개
- [x] setup dry-run, ownership, conflict와 source guard
- [ ] action/role/run/judge/capability machine contract conformance
- [ ] simple-question negative capability test
- [ ] `hive-prompt-refine` automatic intent match, meaning preservation과 refine-only isolation
- [ ] approved Skill의 automatic minimal routing과 OMX/OMC precedence
- [ ] external absent + explicit consent에서만 fallback hook projection
- [ ] fallback hook이 routing, prompt rewrite, orchestration과 Stop continuation을 수행하지 않음
- [ ] persistent role/run fresh-session recovery
- [ ] host-native 또는 external orchestration truthful support 표시
- [ ] usage guard의 freshness와 fail-closed 증거
- [ ] hostile judge context isolation과 quorum
- [x] Karpathy Raw/Wiki/Schema와 SQLite rebuild
- [ ] same-major compatibility
- [ ] cross-major no-data-loss migration
- [ ] GitHub Release provenance와 signing
- [ ] product version parity, compatible minor/patch bump와 explicit-only major gate
- [x] public license — 전체 source·harness `Apache-2.0`, 전문, package metadata와 render fixture
- [ ] clean clone에서 전체 CI PASS

---

## Review needed

| 항목 | 현재 판정 | 다시 검토할 조건 |
| --- | --- | --- |
| OpenClaw | core와 초기 release 제외 | 세 host 기반이 안정된 뒤 별도 host adapter 수요와 conformance 증거 |
| CodexBar | local Codex usage sensor 후보 | machine-readable output, account/window/freshness semantics와 macOS test 통과 |
| usage-coach | dependency 제외, reference only | Hive usage guard에서 검증된 기능 결손 발생 |
| multi-agent-starter | reference only | role/run schema에서 검증된 결손 발생 |
| Copier | authoring·CI에 채택, runtime 제외 | Rust parity/ownership 방식보다 live Copier가 안전하다는 증거가 생길 때만 경계 재검토 |
| qmd/vector DB | 제외 | Markdown index+SQLite FTS recall/latency corpus 실패 |
| Obsidian integration | 유보 | local Markdown workflow 안정 후 실제 탐색 UX 수요 |
| cloud DB/VPS | 제외 | multi-machine concurrent writer 요구가 확정 |
| dashboard/desktop app | 유보 | CLI public release와 recovery gate 완료 |
| Rust TUF/signing library | 미선정 | Phase 6 spike에서 macOS/Windows, audit, rotation, offline verification 비교 |
| Antigravity projection surface | qualification 필요 | version-pinned public documentation과 L2/L3 fixture 확보 |
| web/unreal profile source | 아직 미이식 | 각 reference의 generic 부분을 별도 검토하고 domain fixture·precedence test 작성 |

## References

### 지식·지침·Skill

- [Andrej Karpathy — LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- [multica-ai — andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills)
- [Agent Skills specification](https://agentskills.io/specification)
- [OpenAI Codex — ExecPlans](https://developers.openai.com/cookbook/articles/codex_exec_plans)
- [Anthropic Claude Code — Memory](https://code.claude.com/docs/en/memory)

### Template와 update

- [Copier — Configuring a template](https://copier.readthedocs.io/en/stable/configuring/)
- [Copier — Updating a project](https://copier.readthedocs.io/en/stable/updating/)
- [RFC 8785 — JSON Canonicalization Scheme](https://www.rfc-editor.org/info/rfc8785)
- [The Update Framework](https://theupdateframework.io/)
- [GitHub artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/)

### 라이선스

- [Apache Software Foundation — Apache-2.0 적용 방법](https://www.apache.org/legal/apply-license)
- [REUSE Specification 3.3](https://reuse.software/spec/)

### Orchestration compatibility

- [Yeachan-Heo — oh-my-codex](https://github.com/Yeachan-Heo/oh-my-codex)
- [Yeachan-Heo — oh-my-claudecode](https://github.com/Yeachan-Heo/oh-my-claudecode)
- [OMX — Codex native hook mapping](https://github.com/Yeachan-Heo/oh-my-codex/blob/main/docs/codex-native-hooks.md)
- [OMC — Architecture and Skill routing](https://github.com/Yeachan-Heo/oh-my-claudecode/blob/main/docs/ARCHITECTURE.md)
- [netwaif — multi-agent-starter](https://github.com/netwaif/multi-agent-starter)

### Host Skills와 hooks

- [OpenAI Codex — Hooks](https://learn.chatgpt.com/docs/hooks)
- [Anthropic Claude Code — Skills](https://code.claude.com/docs/en/slash-commands)
- [Anthropic Claude Code — Hooks](https://code.claude.com/docs/en/hooks)
- [Google Antigravity — Skills](https://antigravity.google/docs/skills?app=antigravity-ide)
- [Google Antigravity — Hooks](https://antigravity.google/docs/hooks?app=antigravity)

### Usage

- [steipete — CodexBar](https://github.com/steipete/CodexBar)
- [netwaif — usage-coach](https://github.com/netwaif/usage-coach)

### 배경 영상

- [Video 1 — Claude Code project principles](https://youtu.be/KWrsLqnB6vA)
- [Video 2 — Obsidian second brain and AI team](https://youtu.be/R2aSqw7S3Ws)
- [Video 3 — Agentic OS](https://youtu.be/HRw-vP0j8OM)
