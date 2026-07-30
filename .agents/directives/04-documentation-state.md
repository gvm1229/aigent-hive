# 04. Documentation and State Directive

This directive governs durable project memory.

## Canonical Locations

- `docs/plans/PLAN.md` — sole active plan entrypoint and goal-parameter index
- `docs/plans/phases/` — numbered Phase 0–7 milestone fragments; the index identifies active phases
- `docs/plans/stages/` — numbered Stage 0–11 workflow fragments
- `docs/plans/contracts/`, `active/`, and `references.md` — lazy-loaded contracts, non-phase active checklists, and references
- `docs/state/CURRENT.md` — current evidence-backed handoff and next action
- `docs/decisions/` — decision ledger and ADRs
- `docs/research/` — bounded external research with version/date provenance
- `docs/architecture/` — current architecture
- `docs/guides/` — human-operable procedures

Do not use chat history, `.omx/`, `.agents/work/`, issue drafts, or generated SQLite data as durable project memory.

## Agent-Reviewed Task-Fact Autocapture

- Before the final response for a material source task, decide whether the completed work created
  or materially revised a reusable artifact, product fact, decision, workflow, criterion, or
  continuation context.
- When it did, load `hive-source-wiki` and update the smallest current-truth English/Korean pair.
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
- Whenever a plan is created or materially revised to govern repository work, write it to the
  canonical tracked plan set before executing that plan. Chat text, a native plan tool, goal state,
  or an agent scratch file may mirror the plan but must never be its sole authority.
- Keep `PLAN.md` compact and free of checkboxes; it owns goal parameters, the completion index, load order, active-fragment links, and execution order.
- Store actionable implementation and verification checklists only in documents listed under `PLAN.md`'s `Active fragments` section.
- Split phases and workflow stages into numbered documents. Do not recombine multiple phases or unrelated stages into an aggregate reference file.
- Keep every independently loaded plan fragment below 8 KiB. Split a large stage into numbered subfragments at an existing semantic boundary.
- Derive the `PLAN.md` completion index only from Phase milestone checklists and non-phase documents listed under `Active fragments`. Exclude Stage checklists because they restate workflow acceptance and would double-count implementation.
- Update the completion index in the same edit as any counted checklist state change. The completed and remaining counts, percentage, and per-scope rows must match the linked fragments.
- Give every active checklist item a unique stable ID and exactly one owning fragment.
- Mark a checkbox complete only with evidence.
- Put references and review-only candidates at the bottom, outside the normative workflow.
- When the plan changes materially, update `docs/state/CURRENT.md` and the relevant ADR in the same concern.

## Plan Reconciliation Gate

Before starting or resuming any goal backed by `docs/plans/PLAN.md`:

1. Read the compact index and `docs/state/CURRENT.md`.
2. Load every fragment listed under the index's `Active fragments` section, but do not preload history, stable contracts, or references.
3. Inspect every active checklist item, not only the unchecked subset.
4. Compare each item with current authoritative evidence from the worktree, tests, rendered artifacts, external state, and protected-boundary status.
5. Mark every already-proven unchecked item complete in its single owning active fragment, with evidence kept or recorded in canonical project documents.
6. Keep items unchecked when evidence is missing, indirect, stale, or contradictory.
7. Only after this reconciliation, derive the remaining execution queue from the still-unchecked active items.

This gate is mandatory on every PLAN-backed goal start or resume and after material project-state changes. An unchecked box is not proof that work remains. Legacy native goal wording that refers to unchecked items in `docs/plans/PLAN.md` must resolve to the documents listed under `Active fragments`; the intentional absence of checkboxes in the compact index is not completion evidence. Chat history, native goal plans, `.omx/`, and `.agents/work/` never override the canonical index plus active fragments.

## Language

- Human-readable project documents use concise Korean.
- Agent directives use English.
- Keep code, paths, commands, schema keys, product names, and exact UI labels in their original form.
