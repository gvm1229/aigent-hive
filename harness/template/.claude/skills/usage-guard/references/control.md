# Explicit usage control

Read before any threshold, disable, enable, toggle, or reset acknowledgement operation.

7. Perform at most the explicitly requested control mutation:
   - A global threshold requires explicit global intent and an integer from 1 through 99:

     ```text
     hive usage threshold --user-root <user-root> --remaining-percent <percent> --output json
     ```

   - A project threshold requires an explicit integer and a valid installed project harness.
     Never reinterpret a non-Hive target request as a global change:

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
## Same-session policy recheck

   When a threshold mutation returns `session_recheck_required=true` for an active task, run
   `enforce` immediately with the same exact target, host, session id, process id, user root, and
   available account digest. This recheck also applies to manual source work. Continue only after
   fresh `hive.usage-allowed`; limited, unknown, or policy-changed results remain blocked. Never
   disable the session merely to acknowledge a changed threshold.

## Quota reset acknowledgement

- A `hive.usage-reset` stop persists across repeated enforcement and process restarts. Do not retry sensing to clear it.
- Only after the user explicitly acknowledges the reported reset and requests continuation, use the current halt's `reset_acknowledgement_digest` (or `status`'s `halt_digest`):

  ```text
  hive usage session --target <hive-target> --host <host> --user-root <user-root> --session-id <current-session-id> --process-id <current-process-id> --action acknowledge-reset --confirm-reset <halt-digest> --output json
  ```

- Use installed help first; older releases may not support this action. Never substitute session disable or direct marker editing.
- Immediately run `enforce` with the same exact binding and available account digest. Acknowledgement keeps the threshold guard enabled and does not authorize work or dispatch. Proceed only after fresh `hive.usage-allowed`; all blocked outcomes remain stops.

## Intent rules

- Recognize clear threshold, disable/bypass, enable/restore, and toggle intent semantically; the examples below are illustrative rather than a finite phrase allowlist.
- Disable intent includes requests to turn off or bypass the usage guard, use quota below the stop line, or continue below the configured threshold for the current session.
- Enable intent includes requests to restore the guard, enforce the limit again, stop at the configured threshold, or remove the bypass.
- Threshold mutation requires the requested percentage. Never guess a value.
- A bare `continue`, `resume`, `finish`, urgency, or an active run does not authorize disable.
