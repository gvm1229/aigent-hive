# 04. Documentation and State Directive

This directive governs durable project memory.

## Canonical Locations

- `docs/plans/PLAN.md` — sole active plan entrypoint and goal-parameter index
- `docs/plans/active/` — current-version implementation and release checklists
- `docs/plans/backlog/` — version-unbound candidates excluded from active completion
- `docs/archive/` — completed or superseded plans and historical state excluded from automatic loading
- `docs/plans/references.md` — non-normative external references
- `docs/state/CURRENT.md` — current evidence-backed handoff and next action
- `docs/decisions/` — decision ledger and ADRs
- `docs/research/` — bounded external research with version/date provenance
- `docs/architecture/` — current architecture
- `docs/guides/` — human-operable procedures

Do not use chat history, `.omx/`, `.agents/work/`, issue drafts, or generated SQLite data as durable project memory.

## Mandatory Turn Memory Gate

- On every user turn while Global Wiki is enabled, classify explicit reusable facts,
  preferences, workflows, corrections, and completed reusable outcomes before the final response.
- Write each safe durable item as one bounded normalized canonical claim through
  `hive knowledge remember`; require the canonical Markdown and derived-index receipt before
  reporting completion. Resolve `user-root|current-project|named-project` scope explicitly and
  fail closed on unknown or ambiguous named scope.
- Treat an identical current truth as an idempotent no-op and use an explicit supersede plan for
  a correction. Never append a correction diary or leave durable truth only in SQLite.
- Write nothing for credentials, likely secrets, confidential data without current-action
  authorization, ephemeral status, ambiguous inference, raw transcript, complete conversation,
  hook payload, tool output, hidden prompt, cache, database, or runtime state. Wiki disabled also
  means zero memory mutation.
- This turn gate is an agent-reviewed foreground step, never a hook, background recorder, or raw
  query capture. Source-product outcomes still use the bilingual atomic-fact gate below; do not
  duplicate one fact in both stores unless it also has a distinct user-global use.

## Agent-Reviewed Task-Fact Autocapture

- Before the final response for a material source task, decide whether the completed work created
  or materially revised a reusable artifact, product fact, decision, workflow, criterion, or
  continuation context.
- When it did, load `hive source-wiki` and update the smallest current-truth English/Korean pair.
- Capture only facts from the current authorized task and its reviewed local artifacts: outcome,
  tool or external project used, creation or acceptance criteria, and a bounded originating
  request summary.
- Preserve an exact originating request only when the user explicitly requests exact retention
  and the bounded text passes credential, confidentiality, and private-path review.
- Keep an external artifact outside the source corpus. Record only the Hive-relevant fact and a
  safe locator in a tracked source handoff or decision document, then cite that repository source
  from the Wiki pair.
- Treat an identical fact as an idempotent no-op. Update the existing topic instead of appending a
  correction diary or duplicating a session record.
- Never ingest a raw transcript, complete conversation, hook payload, tool output, hidden prompt,
  cache, database, or runtime state. Autocapture is an agent-reviewed completion step, not a hook
  or background recorder.
- Skip autocapture for editless/simple questions, tasks with no durable value, unsafe material, or
  explicit user opt-out. Report a safety skip only when it affects the requested result.

## Current-Truth Policy

- Update the existing canonical document instead of appending a correction diary.
- Before simplifying, consolidating, or removing a knowledge-bearing document, inventory every
  durable claim that would disappear from the active tree.
- Move still-valid knowledge into the smallest fitting current topic document or atomic fact note
  before removing it from the original surface. Record or verify the exact replacement locator.
- Removal from one presentation surface counts as a move only after the replacement is tracked and
  reachable from the documentation home or index.
- Remove deprecated, incorrect, or superseded knowledge from the active tree only after identifying
  the reason and replacement when one exists.
- Rely on Git history for ordinary historical recovery.
- Purge Git history only for secrets, legal erasure, or another explicitly approved exceptional reason.
- Keep a minimal suppression record when needed to prevent deleted knowledge from being re-ingested. Store only a locator or fingerprint, reason, and replacement; do not duplicate deleted prose.

## Plan Policy

- Keep exactly one active plan set rooted at `docs/plans/PLAN.md`.
- Unless the user explicitly opts out for the current request, write every plan to the
  appropriate canonical Markdown file before presenting or executing it. An opt-out does not
  override another rule that independently requires durable plan state.
- Never mirror a persisted plan one-for-one in the session. Reference it with a concise summary
  and its file path; when extensive review is appropriate, present the file path instead of
  reproducing the plan.
- Whenever a plan is created or materially revised to govern repository work, write it to the
  canonical tracked plan set before executing that plan. Chat text, a native plan tool, goal state,
  or an agent scratch file may mirror the plan but must never be its sole authority.
- Keep `PLAN.md` compact and free of checkboxes; it owns goal parameters, the completion index, load order, active-fragment links, and execution order.
- Keep `PLAN.md` and `docs/state/CURRENT.md` below 8 KiB. Move completed chronology to the tracked archive before shortening either current surface.
- Store actionable implementation and verification checklists only in documents listed under `PLAN.md`'s `Active fragments` section.
- Organize new active fragments by current concern, not historical phase or stage number.
- Keep every independently loaded plan fragment below 8 KiB. Split a large stage into numbered subfragments at an existing semantic boundary.
- Derive the `PLAN.md` completion index only from documents listed under `Active fragments`. Exclude backlog and archive documents.
- Update the completion index in the same edit as any counted checklist state change. The completed and remaining counts, percentage, and per-scope rows must match the linked fragments.
- Give every active checklist item a unique stable ID and exactly one owning fragment.
- Give each acceptance assertion one evidence owner. A release fragment owns only a distinct
  delivery transition such as candidate creation, public-test acceptance, protected integration,
  publication, or installed-artifact confirmation. It may name prerequisite IDs, but must not
  restate their implementation, test, documentation, or security assertion as another checkbox.
- Use a reference table or prerequisite line for evidence owned by another fragment. Do not count
  referenced evidence twice in `PLAN.md`, completion percentages, release gates, or a new test
  candidate decision.
- When a default-off or unsupported capability needs external runtime proof, move its remaining
  acceptance IDs intact to an explicitly named future-version candidate. Preserve the fail-closed
  implementation evidence and exclusion decision; do not leave a release blocked by a duplicate
  shadow checklist.
- Mark a checkbox complete only with evidence.
- Store version-unbound candidates under `docs/plans/backlog/` without release checklist IDs or completion percentages. Promote one only through an explicit versioned active fragment.
- Preserve completed or superseded plan and state records under `docs/archive/`; never treat archive bytes as current product authority or automatic task context.
- Put references at the bottom, outside the normative workflow.
- When the plan changes materially, update `docs/state/CURRENT.md` and the relevant ADR in the same concern.

## Plan Reconciliation Gate

Before starting or resuming any goal backed by `docs/plans/PLAN.md`:

1. Read the compact index and `docs/state/CURRENT.md`.
2. Resolve the active fragment that owns the current request. Load every fragment only when the request spans the complete active release.
3. Inspect every checklist item in the loaded owning scope, not only the unchecked subset.
4. Compare each item with current authoritative evidence from the worktree, tests, rendered artifacts, external state, and protected-boundary status.
5. Mark every already-proven unchecked item complete in its single owning active fragment, with evidence kept or recorded in canonical project documents.
6. Keep items unchecked when evidence is missing, indirect, stale, or contradictory.
7. Only after this reconciliation, derive the remaining execution queue from the still-unchecked items in the owning scope.

This gate is mandatory on every PLAN-backed goal start or resume and after material project-state changes. An unchecked box is not proof that work remains. Legacy native goal wording that refers to unchecked items in `docs/plans/PLAN.md` must resolve to the documents listed under `Active fragments`; the intentional absence of checkboxes in the compact index is not completion evidence. Chat history, native goal plans, `.omx/`, and `.agents/work/` never override the canonical index plus active fragments.

## Final Response Closure Gate

Before a final response for a material source task:

1. Reconcile the current request, its owning plan items, current worktree, verification results,
   and authorized remote actions.
2. Classify every known remaining item as `agent-owned`, `awaiting-user-authority`,
   `awaiting-external-evidence`, or `blocked`.
3. Continue execution when any `agent-owned` item remains. A prose progress report, test failure,
   CI observation, or release milestone does not satisfy this gate.
4. For every non-agent-owned item, record the exact scope, action or evidence required, owner,
   and reason the agent cannot obtain it in the active-session manifest. Update `CURRENT.md` when
   the state is a material source handoff.
5. Use `complete` only when no scoped item remains. Otherwise state the exact non-complete status
   and never describe the task as finished.

## Stable Release Plan Gate

- Keep stable publication blocked while any active in-scope checklist item is incomplete. A
  future-version candidate may be excluded only when `PLAN.md` names its exact checklist IDs and
  target version as deferred.
- Require evidence from a uniquely numbered public test version before stable publication. Bind
  acceptance to the exact source commit and public artifact; a dev build, CI-only candidate, or
  stable installation is not substitute evidence.
- A post-test change resets the affected acceptance item and requires the next numbered public
  test. Never mark a stable publication complete when the stable channel supplied the first or
  only evidence for product behavior, packaging, installation, performance, or recovery.

## Language

- Human-readable project documents use concise Korean.
- Agent directives use English.
- Keep code, paths, commands, schema keys, product names, and exact UI labels in their original form.
