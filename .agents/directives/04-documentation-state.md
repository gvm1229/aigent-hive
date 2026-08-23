# 04. Documentation and State Directive

Owns plans, current state, archive, backlog, Source Wiki facts, and closure recording. Continuation
semantics belong to `01-behavior.md`; release execution belongs to `03-workflow.md`.

## Canonical locations

| Content | Location |
| --- | --- |
| Active plan index | `docs/plans/PLAN.md` |
| Active checklists | `docs/plans/active/` |
| Version-unbound candidates | `docs/plans/backlog/` |
| Completed or superseded history | `docs/archive/` |
| Current handoff | `docs/state/CURRENT.md` |
| Decisions, research, architecture, guides | matching `docs/` directory |
| Atomic source facts | `docs/facts/en/`, `docs/facts/ko/` |

Chat history, runtime scratch state, issue drafts, and SQLite are not durable project memory.

## User and source fact gates

- On a Wiki-enabled turn, classify explicit reusable user facts, preferences, workflows,
  corrections, conventions, and verified outcomes. Use one bounded `hive knowledge remember`
  request with an explicit `user-root|current-project|named-project` scope. Identical truth is a
  no-op; ambiguous scope fails closed.
- Never capture credentials, unauthorized confidential content, private paths, ephemeral state,
  ambiguous inference, raw transcript, complete conversation, hook payload, tool output, hidden
  prompt, cache, database, or runtime state. Wiki disabled means no memory mutation.
- A material source task uses agent-reviewed task fact capture before final response. Update the
  smallest current-truth English/Korean pair with outcome, artifact or tool, criteria, and a bounded
  originating request summary. Do not duplicate a user fact without distinct user-global value.
- External artifacts stay outside the source corpus. Record only a safe locator and Hive-relevant
  reviewed fact. An editless or non-durable task needs no source fact.

## Current-truth preservation

- Update the existing canonical topic instead of appending a correction diary.
- Before shortening or removing knowledge, inventory every durable claim and move each valid claim
  to the smallest current canonical locator. Verify reachability from the documentation index.
- Remove active knowledge only when deprecated, incorrect, or superseded; record reason and
  replacement. Use Git history for normal recovery. History erasure requires exceptional authority.
- A suppression record contains only a locator or fingerprint, reason, and replacement.

## Plan contract

- Keep one active plan set rooted at `PLAN.md`. Persist a plan before execution unless the user
  explicitly opts out and no independent rule requires it.
- `PLAN.md` stays below 8KiB, contains no checkboxes, and owns goal parameters, completion index,
  load order, active-fragment links, and execution order.
- `CURRENT.md` stays below 8KiB and contains current evidence, blockers, unrun verification, and
  next work; move chronology to archive.
- Each active fragment stays below 8KiB and owns unique checklist IDs for one concern. Backlog and
  archive never contribute to completion percentages.
- Give each acceptance assertion one owner. Release checklists may reference implementation proof
  but never count it again.
- Update the completion index with every checklist transition. Mark complete only with current
  evidence. Keep unsupported runtime acceptance intact as an explicitly deferred future candidate.
- A material plan change updates `CURRENT.md` and its owning ADR in the same concern.

## Reconciliation

Before starting or resuming a plan-backed goal:

1. Read `PLAN.md`, `CURRENT.md`, and the owning active fragment.
2. Inspect every item in that fragment, including checked items.
3. Compare current source, tests, rendered artifacts, external evidence, and protected boundaries.
4. Mark already-proven work complete and leave indirect, stale, or contradictory evidence open.
5. Derive the execution queue only from the reconciled fragment.

Load every active fragment only for a complete-release audit. Archive, backlog, native goal text,
chat history, and runtime scratch never override the active plan set.

## Final Response Closure Gate

Apply the status meanings from `01-behavior.md`:

1. Reconcile the request, owning checklist, worktree, evidence, and authorized remote actions.
2. Record remaining `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, and
   `blocked` items in the active-session manifest.
3. Continue execution when any `agent-owned` item remains.
4. For a non-agent-owned item, record exact action or evidence, owner, and why the active session
   cannot obtain it. Reflect a material handoff in `CURRENT.md`.
5. Claim completion only when the scoped criteria and evidence are complete.

## Stable Release Plan Gate

- Keep stable publication blocked while any active in-scope checklist item is incomplete. A future
  candidate is excluded only when `PLAN.md` names its exact IDs and target version.
- Require evidence from a uniquely numbered public test version bound to exact source and public
  artifacts before stable publication.
- A post-test change resets the affected acceptance item and requires the next numbered test.
- Never use a stable channel that supplied the first or only evidence for product behavior,
  packaging, installation, performance, migration, or recovery.

## Language

Human-readable project documents use concise Korean under `08-human-documentation-style.md`.
Agent directives use English. Preserve exact identifiers, commands, paths, and schema keys.
