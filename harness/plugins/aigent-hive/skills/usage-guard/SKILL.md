---
name: usage-guard
description: (usage-guard) Control the installed Hive usage guard and run its automatic-dispatch preflight; infer only explicit threshold or session control intent. Never infer bypass from urgency or a bare continue or resume request.
---

# Hive Usage Guard

Enforce or control only the installed Hive usage policy and the current host session binding. It never runs a model, continues a task, or owns orchestration. Details: [control reference](references/control.md); read only the named section.

## Workflow

1. An explicit global Windows midway-monitor request, such as “중간 중단 감시를 꺼 줘” or “turn off
   midway monitoring”, means the installed monitor's `disable`, never session-guard disable. Read
   *Windows midway monitor*. If the request could mean either safeguard, ask which one.
2. Otherwise classify the target first: installed project harness, Hive source workspace, or
   non-Hive folder. In a non-Hive folder do not invoke threshold, session, status, or enforce,
   and create no halt or runtime state. Read *Target and policy*.
3. Run `hive usage enforce --help`. If unavailable, report the release unsupported; never
   reconstruct the control format. Take the exact current host session ID and process ID
   from the active host context. Never invent, reuse, persist, or transfer a binding.
4. Run `hive usage enforce` only before automatic dispatch; `run resume` automatic mode runs it
   internally. Skip it for quick answers, ordinary manual work, and other non-dispatch actions,
   except a required same-session recheck. Exit `3`, limited, unknown, or reset results block
   dispatch. Non-Codex automatic dispatch fails closed when required sensing has no qualified
   local sensor. Read *Automatic-dispatch preflight*.
5. Exit `0` from `enforce` never authorizes dispatch. Require a fresh same-binding
   `hive run resume --dispatch-intent automatic` result with
   `data.usage_guard.policy_evaluated=true`, one authorization ID, and exactly one dispatch brief.
   Accept `enforced=true` with `outcome=authorized`; `outcome=disabled`, `enforced=false`,
   `observation_required=false` for disabled checks (`enforcement_enabled=false` is the v2 master
   opt-out); or `outcome=observation-unavailable` only under the v2 warning policy. Never
   fabricate an observation or enable a guard for this Skill.
6. If native sensing is unavailable or unsupported, read [sensor fallback](references/sensors.md).
   Never install a fallback without explicit current-action acceptance.
7. Before any explicitly requested threshold or session control, read the control sections. Never
   guess a percentage or infer bypass from a bare continue, resume, finish, urgency, or an active run.
8. `status` is inspection only. Treat `session_override=absent` or `stale` as enabled.
9. Report saved and active thresholds, window, session state, changed path, and exact CLI code.
   Discord receives only run title and checklist count, never raw prompts or credentials.

## Boundaries

- Change usage settings and bindings only through installed CLI flows; never edit them directly
  or persist the raw session identifier.
- Never install a fallback hook, rewrite a prompt, activate another Skill, start a watcher, spawn a
  subagent, create an orchestration loop, continue a stopped task, or invoke OMX/OMC.
- A bare continuation or reset acknowledgement cannot clear an exhaustion halt. Paid credits never
  replace subscription quota. Never claim this guard prevents all credit consumption.
- Other rules: *Setup and manual inspection* and *Boundaries detail*.
