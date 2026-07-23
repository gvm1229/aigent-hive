# 04. Documentation and State Directive

This directive governs durable project memory.

## Canonical Locations

- `docs/plans/PLAN.md` — only active implementation plan
- `docs/state/CURRENT.md` — current evidence-backed handoff and next action
- `docs/decisions/` — decision ledger and ADRs
- `docs/research/` — bounded external research with version/date provenance
- `docs/architecture/` — current architecture
- `docs/guides/` — human-operable procedures

Do not use chat history, `.omx/`, `.agents/work/`, issue drafts, or generated SQLite data as durable project memory.

## Current-Truth Policy

- Update the existing canonical document instead of appending a correction diary.
- Remove superseded or deprecated knowledge from the active tree.
- Rely on Git history for ordinary historical recovery.
- Purge Git history only for secrets, legal erasure, or another explicitly approved exceptional reason.
- Keep a minimal suppression record when needed to prevent deleted knowledge from being re-ingested. Store only a locator or fingerprint, reason, and replacement; do not duplicate deleted prose.

## Plan Policy

- Keep exactly one active plan at `docs/plans/PLAN.md`.
- Write actionable implementation and verification work as unchecked checkboxes.
- Mark a checkbox complete only with evidence.
- Put references and review-only candidates at the bottom, outside the normative workflow.
- When the plan changes materially, update `docs/state/CURRENT.md` and the relevant ADR in the same concern.

## Language

- Human-readable project documents use concise Korean.
- Agent directives use English.
- Keep code, paths, commands, schema keys, product names, and exact UI labels in their original form.
