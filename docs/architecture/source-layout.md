# Source 구조

## 경계

```text
Hive source ──build/test──> release bundle ──setup/update──> consumer harness
```

- Hive source: 이 Git 저장소
- release bundle: 컴파일된 Rust binary, schema, template pack, projection metadata
- consumer harness: 사용자의 독립 프로젝트에 생성되는 로컬 파일

세 tree 사이의 runtime path 역참조 금지.

Hive source, release bundle과 생성된 Aigent Hive 소유 material에는 모두 `Apache-2.0`을 적용. 생성된 harness는 `.hive/` 안에 자체 라이선스 전문을 두며 소비자 프로젝트 root의 license 변경 없음.

## 현재 구조

```text
aigent-hive/
├── AGENTS.md
├── .agents/                    # Hive 개발 지침, 출하 금지
├── crates/
│   ├── hive-core/              # provider-neutral invariant와 usage permit policy
│   ├── hive-render/            # 결정적 staging, ownership, consent와 role materialization
│   ├── hive-wiki/              # Markdown ingest/lint와 disposable SQLite FTS5 projection
│   ├── hive-projection/        # portable Skill routing, prompt 검증과 thin host projection
│   ├── hive-update/            # local integrity, version/migration, backup/journal/recovery
│   └── hive-cli/               # setup/knowledge/index/hook/usage/role/run/judge/update adapter
├── packaging/                  # Homebrew·WinGet source manifest template
├── scripts/                    # release version gate와 direct signed bootstrap
├── harness/
│   ├── template/               # Copier authoring·CI source
│   ├── skills/                 # portable shipping Skill source
│   ├── release/                # compiled historical public-surface baseline
│   ├── projections/            # host별 얇은 projection 확장점
│   ├── profiles/               # domain profile 확장점
│   ├── LICENSE                 # Apache-2.0 전문
│   └── manifest.toml           # ownership·금지 경로
├── schemas/
├── tests/                     # schema/render/materializer conformance
├── docs/
│   ├── 00-home.md             # 사람·agent 공통 진입점
│   ├── 01-index.md            # 전체 문서 catalog
│   └── facts/{en,ko}/         # bilingual atomic source fact
├── LICENSE                    # primary Apache-2.0 전문
├── LICENSES/                  # REUSE용 Apache-2.0 canonical 전문
├── REUSE.toml                 # file-scope license mapping
├── copier.yml
└── hive-source.json            # consumer setup 거부 marker
```

## Source `.agents`와 출하물

루트 `.agents/`: Hive 자체 개발 directive와 ignored runtime state 전용. 명시 유지보수자 요청의
비출하 source-project Skill `update-summary|draft-devlog` 2건만 `.agents/skills/`에 유지. 제품
Skill·consumer projection은 `0건`.

Source 개발의 pre-task gate: 설치된 `hive usage enforce` 1회. Source 전용 Python
guard·watcher·threshold state 없음. Product Skill은 `harness/skills/<name>/` 정본을 설치 product namespace로 사용. current product ID와
retired-name migration: [`../skills.md`](../skills.md). Installed consumer copy, `.hive/`
state와 user knowledge의 source import 금지.

출하용 canonical Skill과 directive는 `harness/`에서 관리하고 release projection
단계에서 consumer 경로를 결정. Runtime state는 ignored `.agents/work/`에 두며
`.omx/`·`.omc/` 수정 금지.

구현된 source docs Wiki: tracked human topic document와
`docs/facts/en/`·`docs/facts/ko/` atomic Markdown.
Derived SQLite: ignored disposable `.agents/work/source-wiki/index.sqlite3`.
Coordination marker: ignored persistent noncanonical
`.agents/work/source-wiki/.index.lock`. Rebuild는 marker regular file의 exclusive OS
advisory lock, `lint`·`query`는 shared lock 사용. Reader는 writer 완료 뒤 live index를
bounded read하고 in-memory 검증 종료까지 lock을 유지하여 in-flight claim gap 관찰
0건. SQLite는 ambient target path에서 직접 open하지 않고 in-memory
생성→serialize→in-memory deserialize 검증 뒤 pinned source-root capability 내부의
recoverable two-phase CAS로 publication. Phase 1은 expected live identity 확인과 unique
Hive-owned claim 이동, Phase 2는 synced temporary의 live 이동과 exact prior claim 정리.
Crash residue는 missing live index와 exact Hive-owned orphan claim·temporary 가능.
다음 explicit rebuild만 canonical Markdown에서 disposable index를 재생성하고 exact
regular Hive-owned claim·temporary path를 정리. `lint`·`query`는 missing·stale·corrupt·
crash-interrupted index에서 implicit repair 없이 fail-closed. Consumer
`.hive/knowledge/`, `omx_wiki/`와 `.omx/wiki/` 사용 금지.
현재 source run orchestration: host-native prepare-only baseline. 후속 목표:
Hive-native event·scheduler·receipt·cancel·team·multi-goal control과 host executor 분리.
신규 OMX·OMC dependency 없음, legacy pinned owner는 read-only provenance. Source Wiki는
orchestration control과 독립이며 knowledge migration `0건`. 결정:
[`ADR-0011`](../decisions/ADR-0011-source-wiki-independence.md)과
[`ADR-0014`](../decisions/ADR-0014-docs-wiki-architecture.md). 구현 checklist:
[`source-docs-wiki.md`](../archive/plans/foundations/source-docs-wiki.md).
세부 계약:

- Role lifecycle: [`role-lifecycle.md`](role-lifecycle.md)
- Run lifecycle: [`run-lifecycle.md`](run-lifecycle.md)
- Skill consent: [`skill-consent.md`](skill-consent.md)
- Optional host-native hook 승인·활성화: [`hook-consent.md`](hook-consent.md)
- Consumer shared guidance marker: [`../guidance-schema.md`](../guidance-schema.md)

## Phase 1 ownership와 activation

`harness/manifest.toml`의 ownership class가 setup의 write/delete 권한을 제한.

| Class | Phase 1 동작 |
| --- | --- |
| `hive-managed-*`, `hive-generated-config` | renderer가 재현한 exact bytes만 관리 |
| `user-answer-protected` | 현재 setup answer에서 재현한 projection만 갱신 |
| `user-consent-protected` | 유효한 approval에 결합된 ledger/descriptor만 생성·철회 |
| `canonical-data-protected` | 기존 canonical bytes를 보존; role definition은 명시 reconfigure 때만 갱신하며 assignment·handoff·body는 보존 |
| `shared-marker` | shared file의 exact Hive marker만 교체 |
| `rebuildable-runtime`, `ephemeral-backup` | canonical source 취급 금지 |

`.omx/**`, `.omc/**`, `.codex/**`와 manifest 밖 path는 setup write 대상으로 사용할 수
없음. `.claude/**`와 `.agents/**`도 기본적으로 foreign-owned이며 Phase 3 manifest가
열거한 exact Skill projection path만 예외. Consumer target 자체뿐 아니라 사용자가
지정한 target path의 기존 ancestor도 symlink와 entry type을 확인하고,
target-relative managed file은 별도로 같은 검사를 반복하므로 외부 symlink target을
추적 금지.

`execute_setup`은 시작 시 ambient parent capability에서 consumer root를 no-follow로 열어 pin. Protected seed, shared marker, role, hook 철회 ownership과 changed-path 계산은 이 pinned handle에서 처리. Apply는 source tree와 consumer tree 밖의 sibling staging directory에서 전체 planned tree를 먼저 검증하고, activation snapshot·parent directory 생성·exclusive temp replacement·삭제·rollback·설치 bytes 재검증을 모두 같은 target handle 아래에서 수행.

Core `cap-std`의 stable public surface만으로는 cross-platform no-follow directory open과 Windows handle-derived identity를 함께 표현할 수 없어, 같은 Bytecode Alliance project의 exact-pinned companion `cap-fs-ext 4.0.2`를 사용. Root·child directory·file entry는 stable no-follow로 열고, mutation 직전과 post-validation 뒤 ambient parent에서 current target을 다시 no-follow-open해 pinned handle과 `(device, inode)`를 exact 비교. Windows 값은 handle-derived volume/file identity. 따라서 ambient target retarget은 conflict와 pinned-tree rollback으로 끝나며 symlink ancestor 교체는 외부 경로 mutation 전에 차단. ReFS의 128-bit file identifier를 companion이 64-bit inode로 표현하는 제한은 Windows matrix에서 계속 감시할 known risk.

각 live replacement는 handle-relative `create_new`로 만든 충돌하지 않는 임의 이름의 temp를 사용. Windows의 기존 destination 교체는 같은 parent capability 아래 backup/복원을 거치며, operation snapshot 기준 rollback과 staged exact bytes·삭제 부재·known hook 집합의 post-validation까지 통과해야 transaction이 성공. Read-only `--validate`와 activation 전 render/preflight는 기존 lexical/no-follow 검사를 유지.

## Phase 5 usage guard와 judge

`hive-core::usage_guard`는 provider-neutral snapshot 검증, configured inclusive 중지선과 one-shot dispatch permit만 소유. Session window가 있으면 우선하고, host가 session limit을 노출하지 않을 때 weekly를 fallback으로 선택. `hive-cli`의 CodexBar adapter는 optional local executable을 fixed argv로 bounded 실행해 snapshot을 정규화하며 provider SDK, API key와 model call은 소유 범위 밖.

Session은 weekly의 low, malformed 또는 duplicate 상태보다 우선. Session
없을 때만 exact 단일 weekly를 fallback으로 선택. `hive run resume`의 automatic
intent는 installed harness threshold를 권위값으로 사용하고 mismatch override를
거부. 이전 selected snapshot과 issued authorization은 Git에서 제외된 Hive-owned
`.hive/runtime/usage-history/`와 `.hive/runtime/dispatch-authorizations/`에만
bounded하게 저장. Measurement/reset 역행과 같은 reset의 remaining 증가는
fail closed. Exact run revision·selected active role·brief 하나에 authorization
하나만 발급하며 permit을 brief 준비 closure 직전에 한 번 소비. Manual intent는
sensor·runtime record read/write 없음. Sensor 부재·stale·scope mismatch,
limited·expired·reused authorization은 brief 없이 `usage_unknown|usage-limited`
recovery로 종료. Hive 밖에서 capture된 JSON replay는 막을 수 없으므로 실제
host/orchestration owner의 authorization ID 단일 소비 필수.

Loop dispatch의 추가 gate: `hive loop checkpoint`의 authenticated usage evidence와
exact run authorization locator·digest 결합 뒤 `hive loop prepare` 재검증. 결과는
`.hive/runs/<run-id>/graph/prepared/`의 `prepared_only: true`, `spawned: false` record.
Hive의 host process·model·subagent spawn 0건.

`hive-core::judge`는 clean-context `JudgePackage`, verdict 전 digest-bound
`JudgeAssignment`, assignment-bound final verdict와 verdict 후 별도
`HumanApproval`을 검증. `hive-core::judge_auth`는 consumer target 밖의
agent-write-denied TOML trust root를 기준으로 assignment, verdict와 approval의
detached Ed25519 signature를 strict 검증. Owner·judge·human key purpose와
principal을 분리하고 public-key bytes 재사용을 거부한 뒤 normal 1명, elevated 2/3,
critical 3/3+human quorum을 결정론적으로 계산.

`hive judge package|quorum`은 target 안의 explicit target-relative file만 bounded
no-follow read하며 project 수정 0건. Task-agent reasoning, self-evaluation,
verdict-leading instruction과 prior judge verdict는 package에서 거부. Quorum
출력은 identity·slot·finding·개별 verdict를 숨긴 aggregate count/status,
authentication algorithm과 approval 유효성만 포함. Unsigned v1은 diagnostic
compatibility만 제공하고 completion-authorizing PASS 권한 없음. Ed25519는 trusted-key possession과
exact artifact binding을 증명하지만 judge 판단의 진실성, 실제 사람의 생체 presence와
전역 replay 방지는 증명 범위 밖. 상세 trust boundary와 fail-closed 조건:
[`judge-trust-boundary.md`](judge-trust-boundary.md).

`package-review`: implemented built-in 중 read-only data Skill.
Package 준비 뒤 independent judge invocation의 소유자: host-native 기본 owner 또는
명시적 compatibility·기존 pinned OMX/OMC owner.
Hive CLI와 Skill의 model, judge, subagent 또는 provider process 실행 금지.

## Phase 2 knowledge와 index

`hive-wiki::RagStore`: capability-pinned user root의 단일 canonical/derived writer.
Canonical source:

- `.hive/config/projects.yml`, `.hive/config/collections.yml`
- `.hive/knowledge/Raw`, `Wiki`, `Claims`, `Schema`, `suppression.yml`

Derived·recovery metadata:

- `.hive/index/rag-generation.json`: published generation manifest
- `.hive/index/rag-dirty.json`: interrupted canonical/derived transaction journal

Raw revision: content-addressed immutable file. Wiki document와 typed claim: typed YAML
frontmatter와 Markdown body. Deprecated·superseded state의 active retrieval 제외.

Collection model:

| Field | Contract |
| --- | --- |
| kind | `user-root|registered-project|directory|imported` |
| state | `attached|detached` |
| visibility | `shared|project-private|confidential` |
| identity | stable collection ID; absolute local path와 source project ID는 identity seed 제외 |

Canonical document: document·collection ID, locator, title, kind, category, body digest,
visibility, language, revision, tag, alias, link, source와 replacement. Canonical claim:
claim ID·key, collection·document ID, locator, kind, assertion status, normalized fact,
provenance, revision, source·supersede·replacement, observed·verified time와 digest.
Assertion status: `user-stated|observed|verified|inferred|conflicted|superseded`.

`.hive/index/hive.sqlite3`: 사용자 root에 하나만 존재하는 disposable normalized RAG
projection. Table family: collection, document, deterministic hash-derived chunk, claim,
FTS, tag, alias, link, source, replacement, generation manifest와 dirty journal. Project별
SQLite, directory별 table, canonical SQLite 소유권 0건.

`RagStore`의 single capability lock 범위: project·collection registry, canonical write,
dirty journal, serialized SQLite와 generation manifest. Rebuild: in-memory 생성·검증 뒤
recoverable publish. Query: registry·manifest·serialized SQLite의 bounded read만 사용하며
canonical Markdown full scan·implicit repair 0건. Missing·dirty·stale·corrupt projection:
explicit rebuild 전 fail-closed.

Wiki-enabled turn의 memory·retrieval gate:

1. 질문·research·knowledge-dependent work의 routing 전 `knowledge-recall` 기반
   bounded retrieval 최대 1회
2. Automatic default top 5와 result byte budget; explicit query만 확대
3. No-hit 시 기존 simple/task route 유지; retrieved instruction·command는 untrusted data
4. Final response 전 모든 user turn의 agent-reviewed memory classification
5. Reusable fact·preference·workflow만 `hive knowledge remember`의 normalized current-truth
   claim으로 기록
6. Secret·confidential-without-scope·ephemeral·ambiguous·raw transcript·hook·tool output·
   runtime state: canonical write 0건

Portable bundle: deterministic stored ZIP `.hivekb`, canonical `manifest.json`과 manifest
digest, path-sorted canonical Markdown·portable metadata·suppression·provenance. SQLite,
runtime state, absolute path, credential와 unauthorized confidential bytes 제외. Import:
full validation → staging → conflict plan → backup → atomic activation → disposable index
rebuild. 다른 machine의 unmapped project collection: stable identity를 보존한 `detached`,
explicit local mapping 전 private auto-query 제외.

Directory scan: explicit `hive knowledge scan`, Git tracked-first, optional nonignored
untracked, non-Git narrow allowlist, deterministic count·size budget. Binary, license,
secret·credential candidate, vendor·generated·runtime path, special file와 external symlink
제외. Inventory는 raw content 없이 path·digest·size·decision·reason만 보존. Claim apply:
exact inventory digest와 1–16개 evidence entry에 결합된 `agent_reviewed: true` atomic
claim만 허용. Reusable global promotion: 별도 consolidated consent·redaction·dedup·
contradiction gate.

Canonical delete·suppress는 active Wiki/Raw와 suppression fingerprint·locator의 동시
존재를 거부. Suppression ledger: fingerprint, source locator, stable reason code,
replacement와 timestamp만 보존.

## Phase 3 Skill routing과 projection

`harness/skills/catalog.yml`: implemented built-in 22개. `user-setup` 1개는 user-scope
전용, project projection 대상 21개. v0.9 신규 built-in 5개:

- `ralph-loop`
- `knowledge-maintain`
- `code-polish`
- `research-best-practices`
- `knowledge-import`

Active routing proof: normalized routing fact, exact Skill content digest, built-in source
또는 optional Skill consent digest의 결합. 한 route의 loaded Skill body 최대 1개.
Current precedence:

1. explicit direct/plain answer
2. explicit Skill
3. simple-question isolation gate
4. Hive run data contract (`run-checkpoint|run-resume|run-handoff`)
5. approved Hive orchestration·task Skill candidate
6. host-native direct capability
7. legacy migration·recovery의 explicit foreign provenance reader

신규 OMX·OMC routing 없음. `prompt-refine`의 기본 mode:
`refine-only`; same request의 명시적 실행 intent가 있는 `refine-and-run`만 허용.

Optional lifecycle hook activation: exact
`.hive/runtime/current-capability-resolution.json`, 60초 이하 freshness,
`resolved_owner: host-native`, requested event의 exact `support: supported` 필수. Legacy
external owner, `best-effort|unsupported|unverified`, missing event 또는 absent surface:
approval·hook input 조회 전 inert. Exact target·event head·control epoch·one-time authority
필수, selected session pointer authority `0건`.

Codex와 Antigravity는 `.agents/skills/<skill>/SKILL.md`, Claude Code는
`.claude/skills/<skill>/SKILL.md`만 사용. Projection은 destination을 exclusive
claim해 검증한 뒤 destination-exclusive publication을 수행. Replace/delete
중 밀려난 기존 bytes는 same-directory quarantine에 보존하고, rollback 때 foreign
occupant를 overwrite하거나 삭제 없음. 자동 복원이 안전하지 않으면 prior
bytes의 recovery path를 diagnostic으로 보존.

소비자 자동 편집 조정과 host Skill 발견: 분리. 현재 소비자 projection: 모두
`.agents/directives/03-session-coordination.md`. `AGENTS.md`: Codex·Claude Code·
Antigravity 공통 계약 연결. `hive session`: Git 제외
`.hive/runtime/active-sessions/*.md`의 bounded host/session ID·process ID·project-relative
reservation path 전용 저장. 검증된 live foreign session의 동일·상위·하위 reservation:
원자적 거부. 직접 사용자·editor write: best-effort 경계 제외.
`project-setup`과 `project-refresh`: 인증된 three-way directive update preview,
user·foreign·비충돌 local byte 보존. 변경 대상: 직접 모순 Hive-owned rule만.

## Phase 4 role/run과 recovery

`hive-core::role`은 persistent role frontmatter/body를, `hive-core::run`은 PLAN
criterion, STATUS state, capability owner pin과 prepare-only `DispatchBrief`를
provider-neutral하게 검증. `hive-cli`의 role/run adapter는 consumer root를
no-follow로 pin하고 explicit request, canonical artifact와 evidence를 bounded read.

`hive role handoff`는 shared `HANDOFF.md`와 selected role assignment를 optimistic
two-file transaction으로 기록. `hive run checkpoint`는 PLAN에서 criterion을
파생하고 첫 capability resolution의 full-object JCS digest를 owner pin으로 저장.
`hive run resume`는 canonical PLAN/STATUS/role/handoff/evidence만 읽어 recovery data와
`prepared_only: true`, `spawned: false` brief를 반환.

현재 새 v0.9 run owner: verified host-native prepare-only 기본값. 신규 Hive-native control은
ADR-0019 feasibility 전 default-off. Legacy OMX·OMC owner: read-only provenance만 허용.
Required capability의
`unsupported|unverified`: exact `host_capability_unsupported`, 다른 runtime 자동 전환과
mutation 0건. Existing run의 missing·incompatible·version·evidence drift: owner 변경
없음.

Canonical loop graph:

```text
.hive/runs/<run-id>/graph/
├── CURRENT.md
├── revisions/<16-digit-revision>.md
└── prepared/<usage-authorization-id>.json
```

`hive loop initialize|validate|checkpoint|steer|prepare|recover`: immutable DAG revision,
cycle·self-edge·unreachable·orphan criterion gate, evidence-bound success edge, independent
verification role, bounded retry·backoff·failure fingerprint, explicit steering과 terminal
`blocked|failed|complete` 계약. `prepare`: one-time usage/run authorization, current STATUS,
role, fresh capability와 전체 evidence의 optimistic validation 뒤 data-only record 생성.
현재 baseline의 scheduler·process spawn·tmux·OMX/OMC command·Stop continuation 0건.
후속 native logical scheduler는 direct process spawn 없이 host envelope 사용. 세부 state와
exit contract: [`run-lifecycle.md`](run-lifecycle.md).

## Phase 6 attested release와 update

`hive-update`는 이미 받은 local bundle의 manifest, artifact length와 SHA-256을 검증.
Npm registry integrity·OIDC provenance 또는 GitHub exact tag·SHA-256·artifact attestation이
획득 출처를 증명하고 local verifier가 다운로드 뒤 byte 변경을 차단. Production crate에는
signing/private-key API나 network downloader가 없음.

Release manifest는 exact version·sequence, source commit/tag와 path별 length·SHA-256을 결합.
Classification은 `harness/release/historical-surfaces.yml`의 compiled baseline과
cumulative inventory의 observed delta와 일치 필수. Feature는 exact next
minor, compatible fix는 exact next patch만 허용. Same-major breaking change는
거부하고 major는 user-supplied exact target과 current plan,
compatibility/preservation report, migration-table digest를 결합한 별도 human
confirmation 없이는 진행 없음.

Update는 verification과 renderer dry-run 뒤 canonical config/team/run/knowledge 및
changed projection bytes를 `.hive/backups/`에 snapshot하고 ignored durable journal을
첫 mutation 전에 기록. SQLite/runtime/backup/`.omx/.omc`는 migration/backup
authority에서 제외. Atomic activation 뒤 exact after digest를 확인하고 update state를
commit marker로 기록한 다음 SQLite rebuild. Prepared transaction은 journaled
before/after digest일 때만 rollback하며 concurrent user bytes는 보존. Valid
unreferenced backup만 7일 초과 뒤 exact file set을 검증해 정리.

Cross-major route는 project/docs/preference/user-Markdown/symlink snapshot과 shared
`AGENTS.md` marker 밖 foreign-byte digest를 activation 전후 비교하고 compiled Hive
system representation만 mutable path로 허용.

`product-update`와 `project-transition` Skill은 verified CLI를 호출하는 thin data workflow.
Downloader, package manager, model, subagent, OMX/OMC와 release-provided executable
실행 금지. 세부 경계: [`release-update-trust-boundary.md`](release-update-trust-boundary.md).

## Crate 추가 원칙

빈 crate 사전 생성 금지. 다음 acceptance 구현 시 owning crate 추가:

- 결정적 setup renderer 구현 → `hive-render`
- Markdown/SQLite index 구현 → `hive-wiki`
- staged update와 migration 구현 → `hive-update`
- host projection compile 구현 → `hive-projection`

crate 이름만으로 미구현 capability를 지원하는 듯한 표시 금지.
