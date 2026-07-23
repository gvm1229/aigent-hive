# AGENTS.md

This is the canonical agent manifest for developing Aigent Hive.
Claude Code, OpenAI Codex, and Gemini-compatible hosts must treat this file as the common entrypoint.
Hive development directives live under `.agents/`; files under `harness/` are source for the product shipped to consumer projects and must not be confused with this repository's own instructions.

## Project Context

- Project: `aigent-hive`
- Type: Rust CLI and provider-neutral agent harness source workspace
- Purpose: build, test, package, and update local project harnesses for subscription-authenticated agent hosts
- Runtime boundary: Hive never calls model-provider APIs and never stores provider API credentials
- Repository language: human-facing documents use concise Korean; agent directives use English

## Prime Directives

- Treat this repository as Hive source, never as a consumer project with an installed Hive harness.
- Keep source, release artifacts, and installed consumer harnesses physically and logically separate.
- Do not implement a model runtime, scheduler, Ralph/plan/team clone, or provider API client. Use host-native orchestration or an explicitly selected external layer such as OMX or OMC.
- Keep knowledge, role, and run state canonical in Markdown; keep setup/config authority in tracked typed YAML/TOML. Treat SQLite as a disposable, reproducible local index over those sources.
- Keep `AGENTS.md` short. Put detailed rules in the narrowest matching directive and load them only when relevant.
- Preserve user-authored and third-party bytes. Hive may mutate only paths and marker blocks declared in its ownership manifest.
- Require explicit user approval before activating any optional third-party or generated Skill.
- Prefer existing, maintained solutions over new infrastructure. Record a concrete deficiency before adding a dependency or custom subsystem.
- Verify claims with fresh evidence before reporting completion.

## Mandatory Directive Loading

Before any repository action that may lead to edits, read:

1. [`.agents/directives/01-behavior.md`](.agents/directives/01-behavior.md)
2. [`.agents/directives/02-architecture.md`](.agents/directives/02-architecture.md)
3. [`.agents/directives/03-workflow.md`](.agents/directives/03-workflow.md)
4. [`.agents/directives/06-session-coordination.md`](.agents/directives/06-session-coordination.md)

Also read:

- [`.agents/directives/04-documentation-state.md`](.agents/directives/04-documentation-state.md) before changing plans, decisions, research, or current-state documents.
- [`.agents/directives/05-security-safety.md`](.agents/directives/05-security-safety.md) before setup/update logic, filesystem mutation, release work, external tools, or credential-adjacent changes.

## Canonical Project Documents

- Active implementation plan: [`docs/plans/PLAN.md`](docs/plans/PLAN.md)
- Current handoff state: [`docs/state/CURRENT.md`](docs/state/CURRENT.md)
- Source layout: [`docs/architecture/source-layout.md`](docs/architecture/source-layout.md)
- Persistent role lifecycle: [`docs/architecture/role-lifecycle.md`](docs/architecture/role-lifecycle.md)
- Optional Skill consent: [`docs/architecture/skill-consent.md`](docs/architecture/skill-consent.md)
- Product decisions: [`docs/decisions/product-release-decisions.md`](docs/decisions/product-release-decisions.md)
- Git workflow: [`docs/guides/branching-rules.md`](docs/guides/branching-rules.md)
- Commit rules: [`docs/guides/commit-rules.md`](docs/guides/commit-rules.md)

## Source and Shipping Boundaries

| Surface | Purpose | Shipped to consumer projects |
| --- | --- | --- |
| Root `AGENTS.md` and `.agents/` | Develop Hive itself | Never |
| `harness/` | Canonical setup questions, templates, skills, and projections | Compiled or projected through a release |
| `crates/` | Rust implementation | Compiled binaries/libraries only |
| `docs/` | Maintainer knowledge and active plan | Never as consumer instructions |
| `tests/fixtures/` | Synthetic consumer projects and expected output | Never |

Do not generate consumer `.hive/`, consumer `AGENTS.md`, or host projections in the source root. Use disposable directories under `tests/work/`.

## Tool-Specific Entry Points

- Codex reads this file directly.
- `CLAUDE.md` redirects Claude Code here.
- `GEMINI.md` redirects Gemini-compatible hosts here.
