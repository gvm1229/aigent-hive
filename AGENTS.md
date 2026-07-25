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
- Do not implement a model runtime, scheduler, Ralph/plan/team clone, or provider API client. Prefer compatible OMX on Codex and OMC on Claude when their capabilities are available; otherwise use only what the host natively supports.
- Keep knowledge, role, and run state canonical in Markdown; keep setup/config authority in tracked typed YAML/TOML. Treat SQLite as a disposable, reproducible local index over those sources.
- Keep `AGENTS.md` short. Put detailed rules in the narrowest matching directive and load them only when relevant.
- Preserve user-authored and third-party bytes. Hive may mutate only paths and marker blocks declared in its ownership manifest.
- Require explicit user approval before activating any optional third-party or generated Skill.
- Resolve relevant approved Skills automatically from narrow descriptions and routing directives. Prefer an OMX/OMC capability over a duplicate Hive workflow, and keep the simple-question path free of unrelated Skill loading.
- Run the source `hive-usage-guard` session gate before every user task, including before the simple-question gate. When halted or unknown, only explicit guard control is allowed; obvious natural-language on/off intent does not require the user to name the Skill.
- After that gate allows work and before editing anything, read [`.agents/directives/00-editing-discipline.md`](.agents/directives/00-editing-discipline.md) in full. Apply all four sections as the highest-priority editing discipline within this repository contract; never compact, summarize, omit, or substitute any part. Higher-priority system, developer, and user instructions plus Hive security, ownership, credential, and production boundaries still control.
- Prefer existing, maintained solutions over new infrastructure. Record a concrete deficiency before adding a dependency or custom subsystem.
- Verify claims with fresh evidence before reporting completion.

## Mandatory Directive Loading

After the source usage gate returns `allowed` and before any repository action that may lead to edits, read in this order:

1. [`.agents/directives/00-editing-discipline.md`](.agents/directives/00-editing-discipline.md)
2. [`.agents/directives/01-behavior.md`](.agents/directives/01-behavior.md)
3. [`.agents/directives/02-architecture.md`](.agents/directives/02-architecture.md)
4. [`.agents/directives/03-workflow.md`](.agents/directives/03-workflow.md)
5. [`.agents/directives/06-session-coordination.md`](.agents/directives/06-session-coordination.md)
6. [`.agents/directives/07-source-usage-guard.md`](.agents/directives/07-source-usage-guard.md)

Also read:

- [`.agents/directives/04-documentation-state.md`](.agents/directives/04-documentation-state.md) before starting or resuming any `docs/plans/PLAN.md`-backed goal and before changing plans, decisions, research, or current-state documents.
- [`.agents/directives/05-security-safety.md`](.agents/directives/05-security-safety.md) before setup/update logic, filesystem mutation, release work, external tools, or credential-adjacent changes.
- [`.agents/directives/08-human-documentation-style.md`](.agents/directives/08-human-documentation-style.md) before creating or changing human-readable project documents.

## Canonical Project Documents

- Active implementation plan: [`docs/plans/PLAN.md`](docs/plans/PLAN.md)
- Current handoff state: [`docs/state/CURRENT.md`](docs/state/CURRENT.md)
- Source layout: [`docs/architecture/source-layout.md`](docs/architecture/source-layout.md)
- Persistent role lifecycle: [`docs/architecture/role-lifecycle.md`](docs/architecture/role-lifecycle.md)
- Optional Skill consent: [`docs/architecture/skill-consent.md`](docs/architecture/skill-consent.md)
- Fallback hook consent: [`docs/architecture/hook-consent.md`](docs/architecture/hook-consent.md)
- Consumer guidance marker: [`docs/guidance-schema.md`](docs/guidance-schema.md)
- Product decisions: [`docs/decisions/product-release-decisions.md`](docs/decisions/product-release-decisions.md)
- Product version lifecycle: [`docs/decisions/ADR-0006-version-lifecycle.md`](docs/decisions/ADR-0006-version-lifecycle.md)
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
