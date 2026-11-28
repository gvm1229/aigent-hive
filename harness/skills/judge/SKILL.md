---
name: judge
description: (judge) Prepare an explicit clean-context request for an independent host-owned Judge, or validate an existing Judge package and its signed acceptance evidence. Never produce a verdict, sign, or self-review. Formerly adversarial-judge, judge-evidence.
---

# Independent Judge (`judge`)

Keep independent review separate from the task agent. Choose exactly one mode, read only its
reference, and follow it.

## Modes

| Request | Mode | Read | Command |
| --- | --- | --- | --- |
| An explicitly requested Judge step or a Judge step in a saved plan | adversarial | [adversarial](references/adversarial.md) | `hive judge package`, host launch, `hive judge quorum` |
| Validate an existing Judge package and signed acceptance evidence | evidence | [evidence](references/evidence.md) | `hive judge package`, `hive judge quorum` |

The former names select modes of this Skill: `adversarial-judge` is adversarial and
`judge-evidence` is evidence. When the user names a former name, say the new name `judge` and its
mode once, then continue.

## Shared rules

- Keep the simple-question gate first. Do not load this Skill for a self-contained quick-answer,
  ordinary continuation, verdict production, signing, or task-agent self-review.
- Run `hive judge package --help` first. If unavailable, report the installed release as
  unsupported. Do not reproduce package, digest, filtering, or quorum logic in prose.
- Hive never calls a provider API, reads provider credentials, spawns a process, signs an
  artifact, or writes a Judge result. A private signing key is never requested, read, or stored.
- The task agent, requester, and any prior Judge result never become Judge context or approval
  authority. Findings are diagnostic; completion authority stays with the authenticated quorum.
- User cancellation and unavailable host capability stop the step without retry or hidden fallback.
