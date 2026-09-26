# Knowledge And Preservation

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

## Language

Human-readable project documents use concise Korean under `08-human-documentation-style.md`.
Agent directives use English. Preserve exact identifiers, commands, paths, and schema keys.
