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
│   ├── hive-update/            # TUF/Ed25519, version/migration, backup/journal/recovery
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
├── LICENSE                    # primary Apache-2.0 전문
├── LICENSES/                  # REUSE용 Apache-2.0 canonical 전문
├── REUSE.toml                 # file-scope license mapping
├── copier.yml
└── hive-source.json            # consumer setup 거부 marker
```

## Source `.agents`와 출하물

루트 `.agents/`는 Hive 자체를 개발하는 에이전트 전용. 일부 external runtime이
`.agents/skills`를 자동 탐색할 수 있으므로 출하용 Skill을 루트 `.agents/skills`에
배치 금지. 유일한 source-only Skill인 `hive-usage-guard`는 source CodexBar watcher,
threshold와 current-session override만 관리하며 release bundle이나 consumer
projection에 포함 없음. Runtime state는 ignored `.agents/work/usage-guard/`에
두며 `.omx/` 수정 금지.

출하용 Skill과 directive는 `harness/`에서만 관리하고 release projection 단계에서 소비자 경로를 결정.
세부 계약:

- Role lifecycle: [`role-lifecycle.md`](role-lifecycle.md)
- Run lifecycle: [`run-lifecycle.md`](run-lifecycle.md)
- Skill consent: [`skill-consent.md`](skill-consent.md)
- Fallback hook 승인·활성화: [`hook-consent.md`](hook-consent.md)
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

`hive-judge-package`: implemented built-in 중 read-only data Skill.
Package 준비 뒤 independent judge invocation의 소유자: resolved host/OMX/OMC owner.
Hive CLI와 Skill의 model, judge, subagent 또는 provider process 실행 금지.

## Phase 2 knowledge와 index

`hive-wiki`는 `.hive/knowledge/Raw`, `Wiki`, `Schema`와 `suppression.yml`만 정본으로
읽기·쓰기 대상. Raw revision은 content-addressed immutable file이며 Wiki page는 typed
YAML frontmatter와 Markdown body. Deprecated/superseded state는 parser에서
active tree 진입을 거부.

`.hive/index/hive.sqlite3`은 FTS5, tag, alias, backlink, source/contradiction graph와
content hash의 disposable projection. Rebuild는 canonical tree를 scan해 같은
directory의 exclusive temp DB를 만들고 page count·logical digest를 검증한 뒤
교체. Query는 매번 canonical logical digest와 stale marker를 확인.

Canonical ingest/delete/suppress는 `.hive/index/.knowledge.lock`으로 직렬화.
삭제는 active page와 참조가 끝난 Raw revision을 제거하고 suppression ledger에는
fingerprint, source locator, reason, replacement, timestamp만 남김. `reason`은 삭제
prose를 복제할 수 없는 shipped stable reason-code enum이며 source/replacement는
`wiki:<id>`, `external:<id>` 또는 immutable Raw locator만 허용. Suppression
fingerprint 또는 locator가 active Wiki/Raw와 겹치면 direct suppression과 index
rebuild를 모두 거부해 active content와 suppression metadata의 공존 차단.

## Phase 3 Skill routing과 projection

`hive-projection`: 구현 완료 built-in 13개의 exact projection. Built-in 목록:
`setup-harness`, `hive-simple-question`,
`hive-prompt-refine`, `hive-knowledge-capture`, `hive-knowledge-query`,
`hive-knowledge-maintenance`, `hive-role-handoff`, `hive-run-checkpoint`,
`hive-run-resume`, `hive-judge-package`, `hive-update`, `hive-migrate`,
`hive-usage-guard`.

Active routing proof는 normalized routing fact, exact Skill content digest와 built-in
source 또는 optional Skill consent digest에 결합. 한 route는 Skill body를 최대
하나만 load하며 explicit Skill/direct answer, simple question, compatible OMX/OMC,
approved Hive Skill, host-native 순서의 precedence를 적용. `hive-prompt-refine`는
`refine-only`가 기본이고 명시된 `refine-and-run`만 허용하며 원문 intent, must,
must-not, scope, output과 authority 보존을 검증.

Fallback non-Stop hook activation은 exact
`.hive/runtime/current-capability-resolution.json`의 non-symlink regular file과
60초 이하 freshness를 요구. Setup은 이 ephemeral file이나 directory를 만들거나
추적하지 않으며 `.hive/.gitignore`의 `/runtime/` 규칙이 Git에서 제외. Missing,
stale, future, malformed 또는 non-absent evidence는 approval과 hook input 조회
전에 inactive neutral allow로 종료. `Stop`은 runtime evidence도 읽지 않는 neutral
fast path.

Codex와 Antigravity는 `.agents/skills/<skill>/SKILL.md`, Claude Code는
`.claude/skills/<skill>/SKILL.md`만 사용. Projection은 destination을 exclusive
claim해 검증한 뒤 destination-exclusive publication을 수행. Replace/delete
중 밀려난 기존 bytes는 same-directory quarantine에 보존하고, rollback 때 foreign
occupant를 overwrite하거나 삭제 없음. 자동 복원이 안전하지 않으면 prior
bytes의 recovery path를 diagnostic으로 보존.

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

Available OMX/OMC는 새 run owner로 자동 선택되며 absent, incompatible 또는 unknown은
truthful support 수준의 host-native owner로 resolve. Fallback hook은 이 셋 중
conclusive `absent`에만 별도 consent로 허용. Existing run은 missing,
incompatible, version 또는 evidence drift에서 owner 변경 금지. 세부 state와 exit
contract: [`run-lifecycle.md`](run-lifecycle.md).

## Phase 6 signed release와 update

`hive-update`는 local extracted TUF repository를 protected external public root로
검증. Root/targets/snapshot/timestamp role은 strict Ed25519 threshold, expiry,
metadata version, target length·SHA-256과 release rollback floor를 결합. Root
rotation은 이전 threshold와 candidate self-threshold를 모두 요구. Production
crate에는 signing/private-key API나 network downloader가 없음.

Release manifest는 exact version/classification, source commit/tag, surface inventory,
compiled migration table, provenance와 platform-signing evidence digest를 결합.
Signed classification은 `harness/release/historical-surfaces.yml`의 compiled baseline과
signed cumulative inventory의 observed delta와 일치 필수. Feature는 exact next
minor, compatible fix는 exact next patch만 허용. Same-major breaking change는
거부하고 major는 user-supplied exact target과 current plan,
compatibility/preservation report, signed migration-table digest를 결합한 별도 human
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

`hive-update`와 `hive-migrate` Skill은 signed CLI를 호출하는 thin data workflow.
Downloader, package manager, model, subagent, OMX/OMC와 release-provided executable
실행 금지. 세부 경계: [`release-update-trust-boundary.md`](release-update-trust-boundary.md).

## Crate 추가 원칙

빈 crate 사전 생성 금지. 다음 acceptance 구현 시 owning crate 추가:

- 결정적 setup renderer 구현 → `hive-render`
- Markdown/SQLite index 구현 → `hive-wiki`
- staged update와 migration 구현 → `hive-update`
- host projection compile 구현 → `hive-projection`

crate 이름만으로 미구현 capability를 지원하는 듯한 표시 금지.
