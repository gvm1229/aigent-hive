# Source 구조

## 경계

```text
Hive source ──build/test──> release bundle ──setup/update──> consumer harness
```

- Hive source: 이 Git 저장소
- release bundle: 컴파일된 Rust binary, schema, template pack, projection metadata
- consumer harness: 사용자의 독립 프로젝트에 생성되는 로컬 파일

세 tree는 서로를 runtime path로 역참조하지 않는다.

Hive source, release bundle과 생성된 Aigent Hive 소유 material에는 모두 `Apache-2.0`을 적용한다. 생성된 harness는 `.hive/` 안에 자체 라이선스 전문을 두며 소비자 프로젝트 root의 license를 변경하지 않는다.

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
│   └── hive-cli/               # setup/knowledge/index/hook/usage/role/run adapter
├── harness/
│   ├── template/               # Copier authoring·CI source
│   ├── skills/                 # portable shipping Skill source
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

루트 `.agents/`는 Hive 자체를 개발하는 에이전트 전용이다. 일부 external runtime이 `.agents/skills`를 자동 탐색할 수 있으므로 출하용 Skill을 루트 `.agents/skills`에 두지 않는다.

출하용 Skill과 directive는 `harness/`에서만 관리하고 release projection 단계에서 소비자 경로를 결정한다.
Role/run lifecycle과 Skill consent의 normative contract는 각각
[`role-lifecycle.md`](role-lifecycle.md), [`run-lifecycle.md`](run-lifecycle.md)와
[`skill-consent.md`](skill-consent.md)에 둔다.
Fallback hook 승인·활성화 경계는 [`hook-consent.md`](hook-consent.md), consumer
shared guidance marker 계약은 [`../guidance-schema.md`](../guidance-schema.md)에 둔다.

## Phase 1 ownership와 activation

`harness/manifest.toml`의 ownership class가 setup의 write/delete 권한을 제한한다.

| Class | Phase 1 동작 |
| --- | --- |
| `hive-managed-*`, `hive-generated-config` | renderer가 재현한 exact bytes만 관리 |
| `user-answer-protected` | 현재 setup answer에서 재현한 projection만 갱신 |
| `user-consent-protected` | 유효한 approval에 결합된 ledger/descriptor만 생성·철회 |
| `canonical-data-protected` | 기존 canonical bytes를 보존; role definition은 명시 reconfigure 때만 갱신하며 assignment·handoff·body는 보존 |
| `shared-marker` | shared file의 exact Hive marker만 교체 |
| `rebuildable-runtime`, `ephemeral-backup` | canonical source로 취급하지 않음 |

`.omx/**`, `.omc/**`, `.codex/**`와 manifest 밖 path는 setup write 대상으로 사용할 수
없다. `.claude/**`와 `.agents/**`도 기본적으로 foreign-owned이며 Phase 3 manifest가
열거한 exact Skill projection path만 예외다. Consumer target 자체뿐 아니라 사용자가
지정한 target path의 기존 ancestor도 symlink와 entry type을 확인하고,
target-relative managed file은 별도로 같은 검사를 반복하므로 외부 symlink target을
따라가지 않는다.

`execute_setup`은 시작 시 ambient parent capability에서 consumer root를 no-follow로 열어 pin한다. Protected seed, shared marker, role, hook 철회 ownership과 changed-path 계산은 이 pinned handle에서 읽는다. Apply는 source tree와 consumer tree 밖의 sibling staging directory에서 전체 planned tree를 먼저 검증하고, activation snapshot·parent directory 생성·exclusive temp replacement·삭제·rollback·설치 bytes 재검증을 모두 같은 target handle 아래에서 수행한다.

Core `cap-std`의 stable public surface만으로는 cross-platform no-follow directory open과 Windows handle-derived identity를 함께 표현할 수 없어, 같은 Bytecode Alliance project의 exact-pinned companion `cap-fs-ext 4.0.2`를 사용한다. Root·child directory·file entry는 stable no-follow로 열고, mutation 직전과 post-validation 뒤 ambient parent에서 current target을 다시 no-follow-open해 pinned handle과 `(device, inode)`를 exact 비교한다. Windows 값은 handle-derived volume/file identity다. 따라서 ambient target retarget은 conflict와 pinned-tree rollback으로 끝나며 symlink ancestor 교체는 외부 경로 mutation 전에 차단된다. ReFS의 128-bit file identifier를 companion이 64-bit inode로 표현하는 제한은 Windows matrix에서 계속 감시할 known risk다.

각 live replacement는 handle-relative `create_new`로 만든 충돌하지 않는 임의 이름의 temp를 사용한다. Windows의 기존 destination 교체는 같은 parent capability 아래 backup/복원을 거치며, operation snapshot 기준 rollback과 staged exact bytes·삭제 부재·known hook 집합의 post-validation까지 통과해야 transaction이 성공한다. Read-only `--validate`와 activation 전 render/preflight는 기존 lexical/no-follow 검사를 유지한다.

## Usage guard 경계

`hive-core::usage_guard`는 provider-neutral snapshot 검증, 10% inclusive 중지선과 one-shot dispatch permit만 소유한다. Session window가 있으면 우선하고, host가 session limit을 노출하지 않을 때 weekly를 fallback으로 선택한다. `hive-cli`의 CodexBar adapter는 optional local executable을 fixed argv로 bounded 실행해 snapshot을 정규화하며 provider SDK, API key와 model call을 소유하지 않는다.

Hive는 현재 dispatch를 종료하거나 자체 loop를 실행하지 않는다. Resolved OMX/OMC 또는 host-native owner가 새 dispatch 직전에 permit을 요청하고 한 번 소비해야 하며, sensor 부재·stale·scope mismatch는 `usage_unknown`으로 fail-closed한다.

## Phase 2 knowledge와 index

`hive-wiki`는 `.hive/knowledge/Raw`, `Wiki`, `Schema`와 `suppression.yml`만 정본으로
읽고 쓴다. Raw revision은 content-addressed immutable file이며 Wiki page는 typed
YAML frontmatter와 Markdown body다. Deprecated/superseded state는 parser에서
active tree 진입을 거부한다.

`.hive/index/hive.sqlite3`은 FTS5, tag, alias, backlink, source/contradiction graph와
content hash의 disposable projection이다. Rebuild는 canonical tree를 scan해 같은
directory의 exclusive temp DB를 만들고 page count·logical digest를 검증한 뒤
교체한다. Query는 매번 canonical logical digest와 stale marker를 확인한다.

Canonical ingest/delete/suppress는 `.hive/index/.knowledge.lock`으로 직렬화한다.
삭제는 active page와 참조가 끝난 Raw revision을 제거하고 suppression ledger에는
fingerprint, source locator, reason, replacement, timestamp만 남긴다. `reason`은 삭제
prose를 복제할 수 없는 shipped stable reason-code enum이며 source/replacement는
`wiki:<id>`, `external:<id>` 또는 immutable Raw locator만 허용한다. Suppression
fingerprint 또는 locator가 active Wiki/Raw와 겹치면 direct suppression과 index
rebuild를 모두 거부해 active content와 suppression metadata가 공존하지 않게 한다.

## Phase 3 Skill routing과 projection

`hive-projection`은 exact 9개 implemented built-in과 3개 catalog-only future entry를
구분한다. Implemented built-in은 `setup-harness`, `hive-simple-question`,
`hive-prompt-refine`, `hive-knowledge-capture`, `hive-knowledge-query`,
`hive-knowledge-maintenance`, `hive-role-handoff`, `hive-run-checkpoint`,
`hive-run-resume`다. `hive-judge-package`, `hive-update`, `hive-migrate`는 catalog
metadata만 가지며 host discovery root에 Skill body를 만들지 않는다.

Active routing proof는 normalized routing fact, exact Skill content digest와 built-in
source 또는 optional Skill consent digest에 결합된다. 한 route는 Skill body를 최대
하나만 load하며 explicit Skill/direct answer, simple question, compatible OMX/OMC,
approved Hive Skill, host-native 순서의 precedence를 따른다. `hive-prompt-refine`는
`refine-only`가 기본이고 명시된 `refine-and-run`만 허용하며 원문 intent, must,
must-not, scope, output과 authority 보존을 검증한다.

Fallback non-Stop hook activation은 exact
`.hive/runtime/current-capability-resolution.json`의 non-symlink regular file과
60초 이하 freshness를 요구한다. Setup은 이 ephemeral file이나 directory를 만들거나
추적하지 않으며 `.hive/.gitignore`의 `/runtime/` 규칙이 Git에서 제외한다. Missing,
stale, future, malformed 또는 non-absent evidence는 approval과 hook input을 읽기
전에 inactive neutral allow로 끝난다. `Stop`은 runtime evidence도 읽지 않는 neutral
fast path다.

Codex와 Antigravity는 `.agents/skills/<skill>/SKILL.md`, Claude Code는
`.claude/skills/<skill>/SKILL.md`만 사용한다. Projection은 destination을 exclusive
claim해 검증한 뒤 destination-exclusive publication을 수행한다. Replace/delete
중 밀려난 기존 bytes는 same-directory quarantine에 보존하고, rollback 때 foreign
occupant를 overwrite하거나 삭제하지 않는다. 자동 복원이 안전하지 않으면 prior
bytes의 recovery path를 diagnostic으로 남긴다.

## Phase 4 role/run과 recovery

`hive-core::role`은 persistent role frontmatter/body를, `hive-core::run`은 PLAN
criterion, STATUS state, capability owner pin과 prepare-only `DispatchBrief`를
provider-neutral하게 검증한다. `hive-cli`의 role/run adapter는 consumer root를
no-follow로 pin하고 explicit request, canonical artifact와 evidence를 bounded하게
읽는다.

`hive role handoff`는 shared `HANDOFF.md`와 selected role assignment를 optimistic
two-file transaction으로 기록한다. `hive run checkpoint`는 PLAN에서 criterion을
파생하고 첫 capability resolution의 full-object JCS digest를 owner pin으로 저장한다.
`hive run resume`는 canonical PLAN/STATUS/role/handoff/evidence만 읽어 recovery data와
`prepared_only: true`, `spawned: false` brief를 반환한다.

Available OMX/OMC는 새 run owner로 자동 선택되며 absent, incompatible 또는 unknown은
truthful support 수준의 host-native owner로 resolve한다. Fallback hook은 이 셋 중
conclusive `absent`에만 별도 consent로 허용한다. Existing run은 missing,
incompatible, version 또는 evidence drift에서 owner를 바꾸지 않는다. 자세한 state와
exit contract는 [`run-lifecycle.md`](run-lifecycle.md)에 둔다.

## Crate 추가 원칙

빈 crate를 미리 만들지 않는다. 다음 acceptance를 구현할 때 owning crate 추가:

- 결정적 setup renderer 구현 → `hive-render`
- Markdown/SQLite index 구현 → `hive-wiki`
- staged update와 migration 구현 → `hive-update`
- host projection compile 구현 → `hive-projection`

crate 이름만으로 미구현 capability를 지원하는 것처럼 보이게 하지 않는다.
