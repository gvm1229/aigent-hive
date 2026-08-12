# Machine-readable contracts

| Schema | 검증 대상 |
| --- | --- |
| `setup-answers.schema.json` | setup/reconfigure answer document |
| `action-result.schema.json` | CLI·host projection의 normalized action result |
| `role-profile.schema.json` | persistent role Markdown frontmatter |
| `run-status.schema.json` | run `STATUS.md` frontmatter |
| `run-checkpoint-request.schema.json` | optimistic durable run checkpoint 입력 |
| `dispatch-brief.schema.json` | host-owned 실행 전 prepare-only dispatch data |
| `role-handoff-request.schema.json` | optimistic persistent role handoff 입력 |
| `judge-package-request.schema.json` | clean-context judge package 생성 요청 |
| `judge-quorum-request.schema.json` | package와 독립 verdict quorum 집계 요청 |
| `judge-package.schema.json` | 독립 judge 입력 envelope |
| `judge-assignment.schema.json` | verdict 이전에 owner provenance와 exact slot roster를 고정하는 JCS digest artifact |
| `judge-verdict.schema.json` | 독립 judge 결과 |
| `judge-approval.schema.json` | critical quorum 이후 별도로 고정하는 JCS digest human approval artifact |
| `judge-attestation.schema.json` | assignment·verdict·approval의 detached Ed25519 signature sidecar |
| `judge-trust-root.schema.json` | consumer target 밖에서 보호하는 purpose-bound public-key trust root |
| `release-bundle-manifest.schema.json` | immutable release identity와 artifact length·SHA-256 binding |
| `migration-table.schema.json` | compiled Rust migration route |
| `release-surface-inventory.schema.json` | shipped schema·Skill·template·projection inventory |
| `historical-surfaces.schema.json` | updater에 compile되는 이전 release surface baseline |
| `update-state.schema.json` | 마지막 수락 release version·sequence·manifest digest |
| `update-journal.schema.json` | crash recovery의 exact mutation·backup·next-state binding |
| `backup-manifest.schema.json` | exact seven-day update recovery snapshot |
| `major-release-confirmation.schema.json` | explicit exact-target breaking-release authority |
| `capability-matrix.schema.json` | host/version/surface qualification |
| `knowledge-page.schema.json` | active Wiki Markdown frontmatter |
| `knowledge-suppression.schema.json` | deleted-content minimal suppression ledger |
| `knowledge-query-result.schema.json` | deterministic query result data |
| `knowledge-lint-result.schema.json` | link/citation/contradiction/stale diagnostics |

Markdown schema는 parser가 YAML frontmatter를 JSON object로 변환한 뒤 검증.
모든 schema는 versioned이며 breaking 변경은 새 major에서만 허용.

JSON Schema만으로 표현하기 어려운 cross-field invariant는 Rust semantic validator가 추가로 검사.

- optional Skill의 `approved_capabilities`는 `requested_capabilities`의 부분집합
- role·knowledge path는 project-relative이며 traversal, absolute path와 symlink escape 금지
- run의 passed/failed criterion은 required criterion의 부분집합이고 서로 disjoint
- `succeeded` run은 required criterion 전부가 passed이고 failed criterion이 없음
- schema v1 legacy run status는 진단용 parse만 허용하고, checkpoint·resume에는
  host/owner/runtime/evidence digest/subagent support 전체 pin과 criterion별 evidence가 필요
- dispatch brief는 provider-neutral data만 준비하며 runtime 호출과 subagent spawn은 범위 밖
- judge quorum에 포함되는 verdict의 `package_digest`는 입력 package와 동일
- authenticated quorum의 attestation은 artifact 전체의 JCS digest, trust-root ID,
  principal, key ID와 artifact kind를 exact domain-separated Ed25519 signature로 결합
- trust-root public key는 전역 unique이며 assignment/verdict/approval purpose를
  교차 사용 금지
- unsigned quorum schema v1은 diagnostic compatibility만 제공하고 PASS 권한 없음
- release artifact는 normalized path, exact length·SHA-256으로 local manifest에 결합
- accepted release version·sequence와 same-sequence manifest digest는
  downgrade/substitution floor를 형성
- release classification은 compiled historical surface와 cumulative inventory의
  observed set delta와 일치 필수
- cross-major confirmation은 exact source/target, current dry-run plan,
  compatibility/preservation report와 migration-table digest 전부에 결합
- migration metadata는 running Rust binary에 compile된 ID만 선택하며
  SQLite/runtime/backup 또는 executable payload의 input 사용 금지
- backup expiry는 exact `created_at_unix + 604800`, canonical-protected와
  manifest-owned entry만 허용하고 cleanup 전 manifest/file digest를 재검증
- Wiki tag/alias/source/link array는 lexicographic sort·unique이며 active source locator는 실제 Raw revision digest와 일치
- suppression entry의 deleted body field 표현 금지

Role seed의 setup-time materialization과 reconfigure/update ownership은
[`../docs/architecture/role-lifecycle.md`](../docs/architecture/role-lifecycle.md) 참조.
Optional Skill consent payload와 digest는
[`../docs/architecture/skill-consent.md`](../docs/architecture/skill-consent.md) 참조.
