---
name: run
description: (run) Save, hand off, or resume an existing Hive run through the signed CLI. Checkpoint progress and evidence, record an explicitly requested role handoff, or load and validate a run after interruption. Formerly run-checkpoint, run-handoff, run-resume.
---

# Hive Run (`run`)

Keep the durable state of one existing Hive run. Choose exactly one mode, read only its
reference, and follow it. Execution, orchestration, and continuation stay with the run's
pinned owner: host-native by default for new v0.9 runs, explicitly selected external
compatibility when requested, or the preserved owner of an existing 0.8.x run.

## Modes

| Request | Mode | Read | Command |
| --- | --- | --- | --- |
| Save progress and evidence before handoff, interruption, or verification | checkpoint | [checkpoint](references/checkpoint.md) | `hive run checkpoint` |
| Explicitly assign a role or save its bounded shared handoff entry | handoff | [handoff](references/handoff.md) | `hive role handoff` |
| Load and validate a run after interruption, handoff, or a new session | resume | [resume](references/resume.md) | `hive run resume` |
| Review real policy-checker evidence of a run | policy review | [policy review](references/policy-review.md) | `hive run policy-review` |

The former names select modes of this Skill: `run-checkpoint` is checkpoint, `run-handoff` is
handoff, and `run-resume` is resume. When the user names a former name, say the new name `run`
and its mode once, then continue.

## Shared rules

- Run the mode's `--help` first. If unavailable, report the installed release as unsupported.
  Never write `STATUS.md`, `PLAN.md`, role files, or `HANDOFF.md` directly, and never reconstruct
  CLI behavior manually.
- Accept only a schema-valid result. An identical retry is a no-op; a stale revision, assignment,
  digest, owner evidence, or unsafe path is a stop condition.
- Keep the simple-question path isolated; do not load this Skill for self-contained quick-answers.
- Never create a plan, Ralph loop, team workflow, retry loop, subagent, model call, provider API
  request, runtime process, or automatic continuation.
- Never select, replace, install, configure, or invoke OMX/OMC, and never switch or fall back to
  another owner when capability evidence changes, is missing, or is unknown.
