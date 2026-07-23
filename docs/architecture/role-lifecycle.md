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

## Migration

Cross-major migration은 shadow tree에서 frontmatter를 parse·validate·transform한다. Current assignment, handoff와 Markdown body는 보존한다. Parse, schema 또는 conflict 검증이 실패하면 active role tree는 바뀌지 않는다.

## Conformance

- setup answer의 모든 seed가 정확히 하나의 role file을 생성
- duplicate와 case-fold collision 거부
- 두 번째 materialization byte-identical
- 명시 승인 없는 definition drift 거부
- 승인된 definition change가 assignment·handoff·body를 보존
- cross-major fixture가 role identity와 user body를 보존
