# 07. Source Usage Guard Directive

This directive governs quota safety while developing Aigent Hive itself. It does not define
consumer harness behavior.

## Session-wide activation

For every user turn in this source workspace:

1. Read [`.agents/skills/hive-usage-guard/SKILL.md`](../skills/hive-usage-guard/SKILL.md).
2. Resolve only obvious usage-guard control intent before ordinary task routing:
   - turn off: explicit natural-language intent to disable/bypass the usage guard, use remaining
     quota, continue below the threshold, or ignore the usage limit for this session;
   - turn on: explicit natural-language intent to enable/restore the guard, enforce the threshold,
     stop at the configured limit, or remove the current bypass;
   - threshold: an explicit new percentage for the usage guard.
3. Apply that control even when the user does not write `$hive-usage-guard`.
4. Run `gate`. It starts the watcher when needed and evaluates the current session.
5. Only after exit `0` may simple-question routing or any other task execution begin.

A bare `continue`, `resume`, `finish`, urgency, or an active goal does not authorize bypass unless
the prompt also clearly refers to the guard, quota, usage limit, threshold, or remaining allowance.

## Halt semantics

- Exit `10` means the selected session-first quota window is at or below the inclusive configured
  remaining percentage.
- Exit `11` means current quota safety cannot be established.
- On either exit, finish only the already-running safe boundary. Block every subsequent ordinary
  task in the same session, including simple answers, plans, Skills, tools, delegations, writes,
  pushes, and final task answers.
- While blocked, permit only guard status, threshold, enable, disable, or toggle control and the
  minimal response explaining how to bypass or restore the guard.
- A watcher marker is authoritative for the guarded session until a fresh `check` clears it.
- `omx cancel` success is not halt evidence for Codex goal mode or durable Ultragoal artifacts.
- The watcher does not signal or kill the Codex App process.

## User override

- A threshold change requires an explicit percentage from the user.
- A session disable requires explicit user intent and `--confirm-session-disable`.
- Obvious natural-language disable/restore intent is explicit intent; naming the Skill is not
  required.
- A disable applies only to the current session identifier and process.
- New sessions default to enabled. Never copy or infer an override.
- Re-enable immediately when the user asks, then run a fresh check.

## Boundary checks

After the turn gate succeeds, run `gate` again before each tool, mutation, delegation, external
write, push, and final task answer. Keep individual blocking commands bounded. The watcher polls
between boundaries. The guard does not claim to interrupt a single in-flight model inference;
it prevents the next observable execution boundary and all later tasks until bypass.

All source-guard state is Hive-owned scratch data under ignored
`.agents/work/usage-guard/`. Never store credentials, account identifiers, or raw CodexBar output.
