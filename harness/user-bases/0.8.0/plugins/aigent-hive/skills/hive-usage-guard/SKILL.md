---
name: hive-usage-guard
description: Control the installed Hive usage guard and run its automatic-dispatch preflight; infer only explicit threshold or session control intent. Never infer bypass from urgency or a bare continue or resume request.
---

# Hive Usage Guard

Enforce or control only the installed Hive usage policy and the current host session binding. This Skill does not run a model, continue a task, or own orchestration.

## Workflow

1. Run `hive usage enforce --help`. If unavailable, report the installed release as unsupported. Do not reconstruct the control format manually.
2. Obtain the exact current host session identifier and process ID from the active host context. Never invent, reuse, persist, or transfer a binding.
3. Immediately before each new automatic dispatch, apply any explicitly requested threshold/off/on control, then run:

   ```text
   hive usage enforce --target <project-root> --session-id <current-session-id> --process-id <current-process-id> [--account-digest <active-account-digest>] --output json
   ```

   Supply the active account digest when the host exposes it. Omit it only when the qualified local sensor exposes exactly one unambiguous account. A current halt marker takes priority. Exit `3`, `hive.usage-limited`, or `hive.usage-unknown` blocks that automatic dispatch. Do not run `enforce` for ordinary answers, manual work, or other non-dispatch actions. Non-Codex automatic dispatch fails closed until a qualified local sensor exists.
4. If native sensing is unavailable or unsupported and CodexBar is missing, report the returned provider-specific notification and exact `next_action`. Do not substitute a different provider:

   ```text
   hive usage fallback-install --host codex|claude|antigravity --dry-run --output json
   ```

   Decline executes nothing, preserves core Hive use, and leaves automatic dispatch `hive.usage-unknown`. Package-manager unavailability remains a sanitized unsupported result.
5. On explicit install acceptance, run the provider-specific dry run first and show its fixed package-manager command. Apply only after fresh current-action confirmation:

   ```text
   hive usage fallback-install --host codex|claude|antigravity --apply --confirm-install --output json
   ```

   Never infer consent, install silently, request credentials, reinstall a provider CLI, or suggest CodexBar API-key or manual-cookie setup.
6. Perform at most the explicitly requested control mutation:
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
7. Treat exit `0` from `enforce` as a session-bound preflight only; it never authorizes dispatch. Require a separate `hive run resume --dispatch-intent automatic` result with `data.usage_guard.enforced=true`, `outcome=authorized`, one authorization ID, and exactly one dispatch brief. A confirmed session disable bypasses the preflight but does not authorize dispatch.
8. `status` is inspection only and never substitutes for an automatic-dispatch preflight. Run `enforce` after a mutation only when a new automatic dispatch is pending. Treat `session_override=absent` or `stale` as enabled. Never copy an override to another host, session, or process.
9. Report the configured threshold, selected window, effective current-session state, changed Hive-owned path, and exact CLI result code.

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
- CodexBar installation is the sole optional fallback install action. It is allowed only through the exact consented CLI flow above and a qualified package-manager adapter.
- Treat any independently produced OMX/OMC cancellation result as auxiliary evidence only. It never substitutes for the bound halt marker or durable goal/task state.
- Never describe a disabled session as usage-enforced. Automatic dispatch still requires the independent `hive run resume` authorization contract.
- Refuse source-workspace use. The source-only development guard has a separate contract.
