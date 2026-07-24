---
name: hive-usage-guard
description: Inspect or change the installed Aigent Hive usage threshold and current-session safeguard through the signed Hive CLI. Use automatically when the user clearly asks for usage-guard status, an explicit remaining-percentage threshold, current-session disable or bypass, enable or restore, or toggle; the user does not need to name this Skill. Do not infer bypass from urgency, an active run, or a bare continue or resume request.
---

# Hive Usage Guard

Enforce or control only the installed Hive usage policy and the current host session binding. This Skill does not run a model, continue a task, or own orchestration.

## Workflow

1. Run `hive usage enforce --help`. If unavailable, report the installed release as unsupported. Do not reconstruct the control format manually.
2. Obtain the exact current host session identifier and process ID from the active host context. Never invent, reuse, persist, or transfer a binding.
3. At every turn boundary, before simple-question detection, Skill routing, planning, answering, tools, mutations, delegation, or continuation, resolve only clear semantic usage-guard control intent. Apply an explicitly requested threshold/off/on control first, then enforce:

   ```text
   hive usage enforce --target <project-root> --session-id <current-session-id> --process-id <current-process-id> [--account-digest <active-account-digest>] --output json
   ```

   Supply the active account digest when the host exposes it. Omit it only when the qualified local sensor exposes exactly one unambiguous account. Exit `3`, `hive.usage-limited`, or `hive.usage-unknown` blocks every non-guard action in the bound host session.
4. Perform at most the explicitly requested control mutation:
   - Threshold requires an explicit integer from 1 through 99:

     ```text
     hive usage threshold --target <project-root> --remaining-percent <percent> --output json
     ```

   - Disable requires obvious current-session bypass intent and the confirmation flag:

     ```text
     hive usage session --target <project-root> --session-id <current-session-id> --process-id <current-process-id> --action disable --confirm-session-disable --output json
     ```

   - Enable restores enforcement for the current binding:

     ```text
     hive usage session --target <project-root> --session-id <current-session-id> --process-id <current-process-id> --action enable --output json
     ```

   - Toggle uses the same command with `--action toggle`. Include
     `--confirm-session-disable` whenever the result would disable enforcement.
5. `status` is inspection only and never substitutes for the turn-boundary `enforce` command. After a mutation, run `enforce` again. Treat `session_override=absent` or `stale` as enabled. Never copy an override to another host, session, or process.
6. Report the configured threshold, selected window, effective current-session state, changed Hive-owned path, and exact CLI result code.

## Intent rules

- Recognize clear threshold, disable/bypass, enable/restore, and toggle intent semantically; the examples below are illustrative rather than a finite phrase allowlist.
- Disable intent includes requests to turn off or bypass the usage guard, use quota below the stop line, or continue below the configured threshold for the current session.
- Enable intent includes requests to restore the guard, enforce the limit again, stop at the configured threshold, or remove the bypass.
- Threshold mutation requires the requested percentage. Never guess a value.
- A bare `continue`, `resume`, `finish`, urgency, or an active run does not authorize disable.

## Boundaries

- Mutate only `.hive/config/harness.toml` through `hive usage threshold` and the current binding under ignored `.hive/runtime/usage-guard/` through `hive usage enforce` or `hive usage session`.
- Never edit those files directly or persist the raw session identifier.
- Never install a fallback hook, rewrite a prompt, activate another Skill, start a watcher, spawn a subagent, create an orchestration loop, continue a stopped task, or invoke OMX/OMC.
- Treat any independently produced OMX/OMC cancellation result as auxiliary evidence only. It never substitutes for the bound halt marker or durable goal/task state.
- Never describe a disabled session as usage-enforced. Automatic dispatch still requires the independent `hive run resume` authorization contract.
- Refuse source-workspace use. The source-only development guard has a separate contract.
