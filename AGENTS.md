# AGENTS.md

Canonical entrypoint for Aigent Hive source development. `CLAUDE.md` and `GEMINI.md` redirect here.
Repository directives live under `.agents/`; shipping consumer sources live under `harness/`.

## Project

- Product: Rust CLI and provider-neutral local agent harness
- Runtime: subscription-authenticated hosts; no model-provider API or provider credential path
- Canonical state: Markdown for knowledge, plans, roles, and runs; typed YAML/TOML for setup authority
- SQLite: disposable derived index
- Human documentation: concise Korean; agent directives: English

## Always-on boundaries

- Treat this repository as Hive source, never as a consumer project.
- Keep source, release bundles, and installed consumer harnesses separate.
- Preserve user-authored and third-party bytes outside declared Hive paths and marker blocks.
- Use host-owned model and subagent execution. Hive owns deterministic state, envelopes, receipts,
  validation, migration, and rollback; it never calls provider APIs or stores provider credentials.
- Stable `tag`, protected `main` integration, publication, and installation require the
  maintainer's current, version-specific approval. `release`, `ship`, `continue`, and `all todos`
  authorize at most implementation, verification, and numbered public tests.
- Abort continued work only for an exact user-owned manual blocker, a required Codex restart, or
  completed criteria. Every other failure remains agent-owned work.
- Preserve historical `harness/project-bases/` and `harness/user-bases/` bytes.
- Verify completion claims with fresh evidence.

## Entry sequence

1. For a source task, run the installed session-bound `hive usage enforce` contract from
   [`.agents/directives/07-installed-usage-guard.md`](.agents/directives/07-installed-usage-guard.md).
2. Before an edit, read [`.agents/directives/00-editing-discipline.md`](.agents/directives/00-editing-discipline.md)
   in full.
3. Load only the directive rows that match the task.

| Task | Directive |
| --- | --- |
| Response, prompt routing, continuation, final status | [`01-behavior.md`](.agents/directives/01-behavior.md) |
| Runtime, orchestration, artifact, or Skill architecture | [`02-architecture.md`](.agents/directives/02-architecture.md) |
| Branch, commit, push, worktree, CI, test or stable release | [`03-workflow.md`](.agents/directives/03-workflow.md) |
| Plan, state, Wiki, fact, archive or backlog | [`04-documentation-state.md`](.agents/directives/04-documentation-state.md) |
| Setup, update, filesystem mutation, external tool or credential-adjacent work | [`05-security-safety.md`](.agents/directives/05-security-safety.md) |
| Concurrent automated edits or session manifest | [`06-session-coordination.md`](.agents/directives/06-session-coordination.md) |
| Usage threshold or session control | [`07-installed-usage-guard.md`](.agents/directives/07-installed-usage-guard.md) |
| Human-readable project document | [`08-human-documentation-style.md`](.agents/directives/08-human-documentation-style.md) |

Source Wiki lookup is target-specific. For this root, use `hive source-wiki query --target
<source-root>`; never pass the source root to consumer `hive knowledge retrieve`.

## Canonical navigation

- Documentation: [`docs/00-home.md`](docs/00-home.md), [`docs/01-index.md`](docs/01-index.md)
- Plan and handoff: [`docs/plans/PLAN.md`](docs/plans/PLAN.md), [`docs/state/CURRENT.md`](docs/state/CURRENT.md)
- Architecture and decisions: [`docs/architecture/`](docs/architecture/), [`docs/decisions/`](docs/decisions/)
- Git rules: [`docs/guides/branching-rules.md`](docs/guides/branching-rules.md), [`docs/guides/commit-rules.md`](docs/guides/commit-rules.md)
- Atomic facts: [`docs/facts/README.md`](docs/facts/README.md)

## Shipping boundary

| Surface | Source purpose | Consumer shipping |
| --- | --- | --- |
| `AGENTS.md`, `.agents/` | Develop Hive | Never |
| `harness/` | Canonical consumer templates, directives and Skills | Rendered projection |
| `crates/` | Product implementation | Compiled artifacts |
| `docs/`, `tests/fixtures/` | Source documentation and synthetic evidence | Never |

Use disposable consumer targets only under `tests/work/`. Do not create consumer `.hive/` or
consumer host projections in the source root. The explicit maintainer-authorized nonshipping
source-project Skills are `.agents/skills/update-summary/` and `.agents/skills/draft-devlog/`;
there is no separate tracked Skill inventory beyond these explicit exceptions.
