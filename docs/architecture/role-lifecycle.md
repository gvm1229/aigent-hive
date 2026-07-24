# Persistent role lifecycle

## 정본과 입력

- `.hive/config/role-seeds.yml`은 setup에서 사용자가 승인한 초기 role definition과 reconfigure preference를 보존한다.
- `.hive/team/roles/<role-id>.md`는 materialize된 뒤 해당 role identity, current assignment와 handoff의 runtime 정본이다.
- Runtime은 role seed를 team member identity로 직접 사용하지 않는다.

## Setup-time materialization

`hive setup`은 live tree를 쓰기 전에 staging에서 모든 role seed를 다음 순서로 materialize한다.

1. `schemas/setup-answers.schema.json`과 semantic path/capability rule 검증
2. 중복·case-fold collision `role_id` 거부
3. 출력 경로를 `.hive/team/roles/<role-id>.md`로 고정
4. seed field에 `schema_version: 1`, `current_assignment: null`, `handoff_path: null` 추가
5. `schemas/role-profile.schema.json`으로 frontmatter object 검증
6. canonical JSON object를 YAML-compatible frontmatter로 기록하고 초기 Markdown body 추가
7. 전체 staging tree 검증 후 한 번에 activation

초기 body:

```markdown
# <display_name>

## Current assignment

_Unassigned._

## Handoff

_No handoff yet._
```

Frontmatter JSON은 UTF-8, LF와 lexicographically sorted object key를 사용한다. Array order는 setup preview에서 사용자가 승인한 order를 보존한다.

## Idempotency와 reconfigure

- 같은 seed를 다시 적용하면 role file byte가 바뀌지 않는다.
- 기존 role file의 definition field가 seed와 다르면 자동 overwrite하지 않고 `hive.role-definition-conflict`로 중지한다.
- 사용자가 reconfigure preview에서 해당 role 변경을 명시 승인하면 definition field만 바꾸고 `current_assignment`, `handoff_path`와 Markdown body는 보존한다.
- seed 제거는 role file을 자동 삭제하지 않는다. 명시적인 retire operation과 preview가 있어야 active role을 제거할 수 있다.
- update는 role file을 일반 generated file로 취급하지 않는다.

## Runtime validation

`hive role validate --target <project> --role <role-id> --output json`은 exact
`.hive/team/roles/<role-id>.md`를 no-follow로 읽고 frontmatter를
`schemas/role-profile.schema.json`으로 검증한다. Filename의 role ID와 frontmatter의
`role_id`가 같아야 하며 Markdown body는 읽은 exact bytes로 digest를 계산한다.
Validation은 role file, project tree와 host-global namespace를 수정하지 않는다.

Role identity는 document가 소유한다. Session ID나 permanent process는 role identity가
아니며 valid role document, current assignment와 handoff 없이 지속형 team member라고
표시할 수 없다.

## Shared handoff transaction

한 run의 역할별 handoff는 각 role file에 복제하지 않고 exact
`.hive/runs/<run-id>/HANDOFF.md` 하나를 공유한다.

```markdown
---
{"handoffs":{"reviewer":{"markdown":"...","updated_at":"...Z"}},"run_id":"run-id","schema_version":1,"updated_at":"...Z"}
---
# Role handoffs
```

Frontmatter는 RFC 8785 canonical JSON이며 `handoffs` map의 key는 role ID다. Entry는
bounded Markdown와 RFC 3339 `updated_at`만 가진다. Body는 exact
`# Role handoffs\n`이다. 한 role handoff를 갱신할 때 다른 role entry와 body는
byte/semantic identity를 보존한다.

`hive role handoff` request는 caller가 읽은 `expected_current_assignment`,
`expected_handoff_path`, shared HANDOFF exact digest를 포함한다. Assignment, path 또는
digest가 stale이면 write 0건 conflict다. 성공 transaction은 shared HANDOFF entry와
role frontmatter의 assignment/handoff path를 함께 commit하며 role Markdown body를
byte-identical하게 보존한다. 중간 실패는 먼저 published한 HANDOFF를 rollback한다.
동일 desired entry와 role assignment retry는 byte-identical no-op다.

이 action은 사용자가 명시한 existing role/run handoff만 기록한다. Role을 자동
선택하거나 plan/team/subagent/continuation을 시작하지 않는다.

## Migration

Cross-major migration은 shadow tree에서 frontmatter를 parse·validate·transform한다. Current assignment, handoff와 Markdown body는 보존한다. Parse, schema 또는 conflict 검증이 실패하면 active role tree는 바뀌지 않는다.

Phase 1은 실제 update migration engine을 구현하지 않는다. 현재 conformance는 setup의 shadow render에 `schema_version: 999`인 JSON-parseable role candidate를 주입해 schema conflict 진단, changed path 0개와 active tree 전체 byte 불변을 검증한다. Source-version route, transform과 migration activation은 Phase 6 범위이며 지원된 것으로 표시하지 않는다.

## Conformance

- setup answer의 모든 seed가 정확히 하나의 role file을 생성
- duplicate와 case-fold collision 거부
- 두 번째 materialization byte-identical
- 명시 승인 없는 definition drift 거부
- 승인된 definition change가 assignment·handoff·body를 보존
- unsupported cross-major role candidate가 schema 검증에서 거부되고 active tree 전체 bytes를 보존
- runtime validation이 malformed, ID mismatch, traversal, symlink와 nonregular role을
  write 없이 거부
- handoff의 stale assignment/path/digest가 role과 shared HANDOFF를 모두 보존
- 여러 role entry를 shared envelope에 기록해도 기존 entry와 각 role body가 불변
- fresh-session resume가 role profile, exact body와 해당 shared handoff entry를 복구
