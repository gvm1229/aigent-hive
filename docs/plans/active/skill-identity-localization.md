# Public Skill identity·localization plan

> Checklist owner: `SIL-*`
> Target: next independently published test release after naming approval
> Scope: all 22 consumer built-in Skills, selected-host plugin projections, global setup preview,
> and selected-interface-language descriptors

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

Source-only developer Skills remain outside this public rename. They do not appear in global
consumer setup or receive the `aigent-hive:` plugin namespace.

## Language contract

- `en`: English display name, short description, and frontmatter description
- `ko`: Korean display name, short description, and frontmatter description; `Aigent Hive`,
  `Skill`, host names, commands, paths, schema keys, and IDs unchanged
- Workflow body: English agent contract retained unless a separately approved full localized
  workflow body becomes necessary
- `both` applies only to Wiki knowledge language, not interface descriptors
- Reconfigure: selected-language marketplace package and user projection refresh; same semantic
  Skill ID, selected name, ownership manifest, and content digest validation

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
- [ ] [SIL-005] Add migration, dependency, ownership, Korean/English descriptor, host discovery,
  clean upgrade, and selected-language reconfigure regressions; qualify in a separate test
  publication while leaving npm `latest` on the stable release

## Acceptance

- Global setup and host discovery show short names only
- Codex plugin invocation identifies the provider as `aigent-hive:<short-name>`
- `record-knowledge` means one reviewed durable fact; `import-repository-knowledge` means a
  reviewed bulk repository scan
- Existing user selections, local modifications, Markdown knowledge, SQLite index, and project
  harnesses preserved through an authenticated migration or left unchanged on conflict
- Interface language controls every user-visible Skill descriptor generated by Hive
- Historical release bases remain immutable; unauthenticated predecessor or overlapping local
  modification produces a no-write conflict
