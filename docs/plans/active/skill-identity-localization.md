# Skill identity·localization plan

> Checklist owner: `SIL-*`
> Target: next independently published test release after naming approval
> Scope: product-only Hive-owned Skills, source projection retirement, selected-host plugin
> projections, global setup preview, selected-language descriptors, and rename-ledger cleanup

## Finding

- Plugin ID `aigent-hive`: host provenance 제공. 내부 `hive-` 접두사와 source Skill 목록: 중복
- Fixed English descriptor: 선택 언어와 불일치. Hive-owned marketplace source에서 localized
  package 생성 가능
- Source Skill: 설치 product Skill과 기능·context 중복. 저장소별 behavior 정본은
  `AGENTS.md`·`.agents/directives/`

## Approved naming catalog

Canonical human catalog: [`docs/skills.md`](../../skills.md). Consumer invocation:
`$aigent-hive:<product-name>`. Aigent Hive source development uses the same installed product
Skills. Final tracked source Skill count: `0`.

Maintainer corrections: product `prompt-refine`, `research-best-practices`, universal `ship`,
user-amendable `amend-directive`. Historical `hive-loop-engineering` remains current
`engineer-run`; approved target: product `ralph-loop`.

Source-specific Git, release, documentation, safety, and Windows rules remain tracked repository
directives. Product Skills discover and apply those rules without embedding Aigent Hive branch or
release policy.

## Language contract

- `en|ko`: display name·short description·frontmatter description localization. Product name,
  `Skill`, host, command, path, schema key, ID 불변. `both`: Wiki 전용
- Workflow body: English contract 유지
- Reconfigure: selected-language package·user projection refresh와 identity·digest 검증

## Rename ledger contract

- Retired ID route: one-to-one·merge·split closure·no-Skill base tool. Transitive resolution,
  duplicate·cycle 거부, current product ID만 출력
- 적용 범위: saved selection, dependency closure, collision reservation, future rename regression
- 삭제 authority: authenticated historical inventory 또는 installed ownership manifest의 exact
  byte. Changed·unknown·foreign path: write 없는 conflict

## Product workflow decisions

`usage-guard`·`ship`·`amend-directive`와 폐기 source route의 상세 정본:
[`docs/skills.md`](../../skills.md)·[`ADR-0012`](../../decisions/ADR-0012-global-onboarding-shared-index.md).

## Checklist

- [x] [SIL-001] Confirm the public-name mapping and document the exact host invocation
  syntax, including the selected-host `aigent-hive:` namespace
- [x] [SIL-002] Add a typed legacy-to-short-name migration for saved selections, dependency
  closure, ownership manifests, optional-Skill collision checks, update three-way bases, and
  incoming custom edits; old IDs accepted only as migration input and never emitted after apply
- [x] [SIL-003] Render selected-language display names and concise descriptions from canonical
  `en|ko` locale data into Hive-owned user and selected-host marketplace projections; no host
  cache mutation or foreign-byte overwrite
- [x] [SIL-004] Rename canonical consumer Skills and all generated projections, preserve the
  `aigent-hive:<short-name>` host namespace, and revise global setup, README, schemas, release
  metadata, and migration guidance
- [x] [SIL-006] Add a canonical, transitive retired-ID ledger; use it to migrate selections and
  safely remove only authenticated retired projection paths during `0.8.x` and future rename
  updates, refusing modified or foreign retired paths without writes. Evidence: `retired-names.yml`,
  `hive-projection` ledger validation, and `historical_080_install_supports_retirement_and_rejects_byte_tamper`
  exercise every historic user Skill path on all three hosts
- [x] [SIL-005] Add migration, dependency, ownership, Korean/English descriptor, host discovery,
  clean upgrade, retired-path cleanup/conflict, and selected-language reconfigure regressions;
  qualify in a separate test publication while leaving npm `latest` on the stable release. Evidence:
  candidate `31183471023` all five native targets·npm umbrella PASS; publication `31184578205`
  PASS, `test=0.9.0-test.5`, `latest=0.8.0`, `v0.9.0-test.5` GitHub prerelease
- [x] [SIL-007] Record the reviewed source·product Skill catalog and lineage in `docs/skills.md`
- [ ] [SIL-008] Replace that interim catalog with the approved product-only 22-Skill catalog;
  final `.agents/skills/` tracked Skill count `0`, source behavior retained in repository directives
- [ ] [SIL-009] Add product `ship` with repository-rule discovery, concern map, scoped staging,
  nearest verification, commit inspection, explicit push authority, and no Aigent Hive hardcoding
- [ ] [SIL-010] Add product `amend-directive` for global·project·Hive source scopes. Preview exact
  owned paths·markers, preserve local edits, update canonical producer and projections together,
  move ordinary behavior constants from Rust into amendable directives or typed preferences, and
  refuse signed cache·foreign byte·compiled safety-boundary mutation
- [ ] [SIL-011] Route source Wiki tasks through product knowledge Skills and `hive source-wiki` on
  `hive-source.json`; remove `source-review` in favor of Wiki lookup plus ordinary read-only tools
- [ ] [SIL-012] Move every reusable source workflow to its canonical product Skill, update source
  AGENTS·directives to require the installed product plugin, then delete tracked `.agents/skills/`
  only after source setup·guard·Wiki·prompt·commit routes pass
- [ ] [SIL-013] Extend the retired-name ledger for one-to-one, merge, split, and no-Skill routes;
  migrate saved selections and remove only authenticated retired source projections
- [ ] [SIL-014] Regenerate selected-language catalog, plugin packages, host projections, setup
  preview, README, dependency closure, and all references from the product-only canonical list
- [ ] [SIL-015] Verify clean source bootstrap with installed product, missing-product guidance,
  combined host discovery collision 0, local amendment preservation, historic upgrade cleanup,
  universal Git fixtures, source Wiki routing, and all three host projections

## Acceptance

- Global setup and host discovery show short names only
- Codex plugin invocation identifies the provider as `aigent-hive:<short-name>`
- `record-knowledge` means one reviewed durable fact; `import-repository-knowledge` means a
  reviewed bulk repository scan
- Existing user selections, local modifications, Markdown knowledge, SQLite index, and project
  harnesses preserved through an authenticated migration or left unchanged on conflict
- A verified retired Skill projection is removed when its replacement is selected; a modified or
  foreign retired path remains untouched and produces a no-write conflict
- Interface language controls every user-visible Skill descriptor generated by Hive
- Every active Hive Skill follows the product-only list in `docs/skills.md`; tracked source Skill
  directory and source-specific host discovery entries: `0`
- Aigent Hive source remains fully operable through installed product Skills plus tracked repository
  directives; missing product dependency yields exact installation guidance before work
- `ship` contains no Aigent Hive branch, release, version, or test constant
- `amend-directive` changes ordinary Hive behavior without modifying signed cache or weakening
  compiled ownership·credential·provider boundaries
- Historical release bases remain immutable; unauthenticated predecessor or overlapping local
  modification produces a no-write conflict
