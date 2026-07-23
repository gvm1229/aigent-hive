# Machine-readable contracts

| Schema | 검증 대상 |
| --- | --- |
| `setup-answers.schema.json` | setup/reconfigure answer document |
| `action-result.schema.json` | CLI·host projection의 normalized action result |
| `role-profile.schema.json` | persistent role Markdown frontmatter |
| `run-status.schema.json` | run `STATUS.md` frontmatter |
| `judge-package.schema.json` | 독립 judge 입력 envelope |
| `judge-verdict.schema.json` | 독립 judge 결과 |
| `capability-matrix.schema.json` | host/version/surface qualification |

Markdown schema는 parser가 YAML frontmatter를 JSON object로 변환한 뒤 검증.
모든 schema는 versioned이며 breaking 변경은 새 major에서만 허용.

JSON Schema만으로 표현하기 어려운 cross-field invariant는 Rust semantic validator가 추가로 검사한다.

- optional Skill의 `approved_capabilities`는 `requested_capabilities`의 부분집합
- role·knowledge path는 project-relative이며 traversal, absolute path와 symlink escape 금지
- run의 passed/failed criterion은 required criterion의 부분집합이고 서로 disjoint
- `succeeded` run은 required criterion 전부가 passed이고 failed criterion이 없음
- judge quorum에 포함되는 verdict의 `package_digest`는 입력 package와 동일

Role seed의 setup-time materialization과 reconfigure/update ownership은
[`../docs/architecture/role-lifecycle.md`](../docs/architecture/role-lifecycle.md)를 따른다.
Optional Skill consent payload와 digest는
[`../docs/architecture/skill-consent.md`](../docs/architecture/skill-consent.md)를 따른다.
