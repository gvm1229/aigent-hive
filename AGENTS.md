# Aigent Hive source

Canonical entrypoint; `CLAUDE.md` and `GEMINI.md` redirect here. Rust CLI and provider-neutral
local harness. Agent directives are English; human documentation is concise Korean.

## Always-on boundaries

- This is Hive source, never a consumer project. Keep source, release bundles and installed
  harnesses separate. Use `tests/work/` for disposable consumers; no consumer `.hive/` or host
  projections in this root. Preserve historical `harness/project-bases/` and `harness/user-bases/`.
- Preserve user/third-party bytes outside declared Hive paths and exact marker blocks.
- Hosts own subscription-authenticated models/subagents; no provider API calls or credentials.
  Hive runtime ownership is specified in 02.
- Canonical Markdown and typed YAML/TOML; SQLite is disposable. Data ownership: 02.
- Stable tags, protected main integration, publication and installation need current version-specific
  maintainer approval. Release/ship/continue/all todos authorize at most implementation, verification
  and numbered public tests. New product bytes after stable require a newer version and its test.1.
- Use [PLAN](docs/plans/PLAN.md)'s active version and next numbered public test for product work,
  unless the current user names another.
  Never invent a later destination. Stop continued work only for a user-owned manual blocker,
  required Codex restart, completed criteria or user interruption; otherwise continue safe work.
- Verify completion with fresh evidence. Work on develop; branch exceptions need explicit authority.
  Before branch creation/rename run `python scripts/branch-policy.py check <name>`.
  Only main, develop or approved work-class prefixes; never agent-name prefixes such as codex/.

## Read by phase

First source action: [installed usage guard](.agents/directives/07-installed-usage-guard.md).
Then read only matching rows. Linked references are conditional, not a recursive reading list.
Reuse unchanged material within a phase. Recover necessary rules after compaction; do not confuse
unchanged files with instructions still present in context. Bare directive filenames refer to `.agents/directives/`; other unlinked paths are repo-relative.

| When | Owner |
| --- | --- |
| Response/continuation | [01](.agents/directives/01-behavior.md) |
| Code/docs restructuring | [00](.agents/directives/00-editing-discipline.md) |
| Architecture/Skill boundaries | [02](.agents/directives/02-architecture.md) |
| Git/check selection | [03](.agents/directives/03-workflow.md) |
| Plan/state/knowledge/closure | [04](.agents/directives/04-documentation-state.md) |
| Filesystem/external tools | [05](.agents/directives/05-security-safety.md) |
| Edit reservations/concurrency | [06](.agents/directives/06-session-coordination.md) |
| Usage controls | [07](.agents/directives/07-installed-usage-guard.md) |
| Human document | [08](.agents/directives/08-human-documentation-style.md) |

For source knowledge use `hive source-wiki query --target <source-root>`, never consumer
`hive knowledge retrieve` at this root. An implementation plan carries decisions,
ordered file/symbol changes and verification; its implementer reads the owning step, not every
planning reference. See [handoff contract](.agents/directives/references/planning-contract.md)
when authoring or revising a plan.

## Navigation and shipping

[Home](docs/00-home.md) · [Index](docs/01-index.md) · [Current](docs/state/CURRENT.md) ·
[Architecture](docs/architecture/) · [Decisions](docs/decisions/) · [Facts](docs/facts/README.md) ·
[Branches](docs/guides/branching-rules.md) · [Commits](docs/guides/commit-rules.md).

`AGENTS.md`, `.agents/`, `docs/` and `tests/fixtures/` never ship to consumers.
`harness/` owns canonical consumer templates/directives/Skills; `crates/` compiles product artifacts.
Only `.agents/skills/update-summary/` and `.agents/skills/draft-devlog/` are explicitly authorized
nonshipping source-project Skills; no separate tracked Skill inventory beyond these exceptions.
