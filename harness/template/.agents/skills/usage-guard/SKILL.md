---
name: usage-guard
description: (usage-guard) Control the installed Hive usage guard and run its automatic-dispatch preflight; infer only explicit threshold or session control intent. Never infer bypass from urgency or a bare continue or resume request.
---

# Hive Usage Guard

Enforce or control only the installed Hive usage policy and the current host session binding. This Skill does not run a model, continue a task, or own orchestration.

## Workflow

1. Classify the target before any control. A valid installed project harness uses the global policy
   plus its registered project override. A Hive source workspace uses the installed global policy
   with an explicit `--host` and stores runtime state only under the user root. Every other folder,
   including one with only its own `AGENTS.md` or no files, is non-Hive: do not invoke threshold,
   session, status, or enforce, and create no halt or runtime state. The global enabled state applies to every project.
   Project overrides may only raise the stop threshold: the active value is the maximum of
   global, registered-project, and installed-project compatibility thresholds. A disabled global guard disables every project.
   Guard inactivity never disables setup-free Hive Skills. A quick answer, prompt refinement, or
   user-root knowledge workflow may still run in a non-Hive folder without project setup and
   without a usage preflight. If a separately selected workflow requires project state, that
   workflow owns one clear setup approval and automated bootstrap; do not expose capability or run
   prerequisites as usage-guard errors.
2. Run `hive usage enforce --help`. If unavailable, report the installed release as unsupported. Do not reconstruct the control format manually.
3. Obtain the exact current host session identifier and process ID from the active host context. Never invent, reuse, persist, or transfer a binding.
4. Immediately before each new automatic dispatch, apply any explicitly requested threshold/off/on control, then run:

   ```text
   hive usage enforce --target <project-root> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> [--host codex|claude|antigravity] [--run <canonical-run-id>] [--account-digest <active-account-digest>] --output json
   ```

   Supply the exposed active account digest; omit only for one unambiguous locally sensed account. `enforce` remeasures ordinary halts, including earlier-process and legacy markers. A reset halt requires explicit acknowledgement under [control procedures](references/control.md). Exit `3`, limited, unknown, or reset results block dispatch. Do not run `enforce` for quick answers, manual work, or other non-dispatch actions except the required same-session recheck after a threshold change or reset acknowledgement. Non-Codex automatic dispatch fails closed until a qualified local sensor exists.
5. If native sensing is unavailable or unsupported, read [sensor fallback](references/sensors.md). Never install a fallback without explicit current-action acceptance.
6. Before any explicitly requested threshold or session control, read [control procedures](references/control.md). Never guess a percentage or infer bypass from a bare continue, resume, finish, urgency, or an active run.
7. Treat exit `0` from `enforce` as a session-bound preflight only; it never authorizes dispatch. Require a separate `hive run resume --dispatch-intent automatic` result with `data.usage_guard.enforced=true`, `outcome=authorized`, one authorization ID, and exactly one dispatch brief. A confirmed session disable bypasses the preflight but does not authorize dispatch.
8. `status` is inspection only and never substitutes for an automatic-dispatch preflight. After mutation, run `enforce` only for pending automatic dispatch or the same-session recheck required by the control reference. Treat `session_override=absent` or `stale` as enabled. Never copy an override to another host, session, or process.
9. Report saved global and project thresholds, active threshold, selected window, effective session state, changed Hive path, and exact CLI code. With a canonical `--run`, Discord receives only run title and checklist count; never raw prompts, session IDs, absolute paths, or credentials.

## Boundaries

- Mutate global `.hive/config/user-setup.yml` only through explicit `hive usage threshold --user-root`, a configured project's `.hive/config/harness.toml` only through project threshold control, and the current binding under ignored `.hive/runtime/usage-guard/` only for a configured Hive target.
- Never edit those files directly or persist the raw session identifier.
- Never install a fallback hook, rewrite a prompt, activate another Skill, start a watcher, spawn a subagent, create an orchestration loop, continue a stopped task, or invoke OMX/OMC.
- `quota_reset_guard_enabled` defaults to true. A reset observation blocks automatic dispatch with
  `hive.usage-reset`; it does not authorize a watcher, a provider call, or a host-process signal.
- CodexBar installation is the sole optional fallback install action. It is allowed only through the exact consented CLI flow in the sensor reference and a qualified package-manager adapter.
- Treat any independently produced OMX/OMC cancellation result as auxiliary evidence only. It never substitutes for the bound halt marker or durable goal/task state.
- Never describe a disabled session as usage-enforced. Automatic dispatch still requires the independent `hive run resume` authorization contract.
- Source development uses this installed product contract and global threshold. Never create a second source Skill, Python guard, watcher, or source-local threshold state.
