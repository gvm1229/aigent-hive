# Skill identity·localization plan

> Checklist owner: `SIL-*`
> Target: next independently published test release after naming approval
> Scope: all source and consumer Hive-owned Skills, selected-host plugin projections, global setup
> preview, selected-interface-language descriptors, and durable rename-ledger cleanup

## Finding

- Oh My Codex keeps its Skill directory names short and receives provenance from its plugin ID
  `oh-my-codex`; its local package manifest exposes `skills: "./skills/"`
- Hive already receives the equivalent host namespace from the plugin ID `aigent-hive`, but its
  internal `hive-` prefixes duplicate that identity: `aigent-hive:hive-knowledge-scan`
- Current `SKILL.md` frontmatter and Codex metadata contain fixed English descriptions. Global
  setup writes the selected response language into user guidance, but it does not render
  descriptors in that language
- Hive owns the selected-host marketplace source package under `~/.hive/marketplaces/`; a safe
  selected-language plugin projection can therefore be generated without treating a host cache
  as canonical user data

## Proposed public names

Host-facing invocation form: `$aigent-hive:<name>`.

| Current public ID | Proposed short name | Purpose label |
| --- | --- | --- |
| `setup-hive` | `configure` | Global Hive preferences |
| `setup-harness` | `setup-project` | Project harness setup |
| `auto-setup-harness` | `auto-setup-project` | Minimal-question project setup |
| `hive-simple-question` | `answer` | Self-contained answer |
| `hive-prompt-refine` | `refine-prompt` | Prompt refinement before execution |
| `hive-knowledge-capture` | `record-knowledge` | One reviewed durable fact |
| `hive-knowledge-query` | `search-knowledge` | Bounded knowledge retrieval |
| `hive-knowledge-promote` | `share-knowledge` | Approved cross-project promotion |
| `hive-knowledge-maintenance` | `maintain-knowledge` | Lint, rebuild, delete, or suppress knowledge |
| `hive-knowledge-scan` | `import-repository-knowledge` | Reviewed bulk repository onboarding |
| `hive-wiki` | `manage-wiki` | Explicit Wiki command routing |
| `hive-run-checkpoint` | `save-progress` | Durable run checkpoint |
| `hive-run-resume` | `resume-work` | Validated fresh-session resume |
| `hive-usage-guard` | `manage-usage` | Usage safeguard control |
| `hive-role-handoff` | `handoff-role` | Persistent role handoff |
| `hive-judge-package` | `verify-package` | Judge package provenance validation |
| `hive-update` | `update-hive` | Signed Hive update |
| `hive-project-upgrade` | `upgrade-project` | Project projection upgrade |
| `hive-migrate` | `migrate-project` | Supported project migration |
| `hive-loop-engineering` | `engineer-run` | Evidence-gated run-graph engineering |
| `ai-slop-cleaner` | `clean-ai-slop` | Behavior-preserving cleanup |
| `best-practice-research` | `research-practices` | Bounded primary-source research |

The completed public rename covered consumer Skills and shared source copies; five source-only
`hive-*` IDs remained. The maintainer superseded that exclusion. Approved naming now covers the
source·consumer union: shared Skills use one exact short ID, surface-only Skills keep their location
but follow the same policy, and only consumer host invocation receives `aigent-hive:`.

## Language contract

- `en`: English display name, short description, and frontmatter description
- `ko`: Korean display name, short description, and frontmatter description; `Aigent Hive`,
  `Skill`, host names, commands, paths, schema keys, and IDs unchanged
- Workflow body: English agent contract retained unless a separately approved full localized
  workflow body becomes necessary
- `both` applies only to Wiki knowledge language, not interface descriptors
- Reconfigure: selected-language marketplace package and user projection refresh; same semantic
  Skill ID, selected name, ownership manifest, and content digest validation

## Rename ledger contract

- Canonical catalog records every retired public or legacy Skill ID against its current ID
- Resolution follows the ledger transitively, rejects duplicate retired IDs or cycles, and emits
  only the current ID
- The ledger resolves names and reserves retired IDs; authenticated historical release inventory or
  an installed ownership manifest is the sole authority for a retired projected path and its deletion
- A retired path with changed, unknown, or foreign bytes blocks the operation without writing;
  Hive never deletes it by name alone
- The same ledger drives saved selection migration, dependency closure, collision reservation, and
  future rename regressions. Every future rename also adds an authenticated historical-base or
  ownership-manifest cleanup regression; a name match alone never authorizes deletion

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
- [ ] [SIL-007] Present one deduplicated source·consumer Skill inventory with current name, scope,
  function, and example use cases; record maintainer approval for every retained or replaced name
- [ ] [SIL-008] Apply the approved names to source-only, shared, and consumer-only Skills in one
  migration; update source routing, consumer namespace metadata, transitive retired-name cleanup,
  references, selected-language descriptions, and source·consumer·upgrade regressions

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
- Every active source and consumer Skill follows the approved short action-name policy; a shared
  Skill has one exact ID on both surfaces and no source-only Skill remains on an unapproved
  `hive-*` exception
- Historical release bases remain immutable; unauthenticated predecessor or overlapping local
  modification produces a no-write conflict
