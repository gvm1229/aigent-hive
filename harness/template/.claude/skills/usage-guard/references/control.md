# Explicit usage control

Read before any threshold, disable, enable, or toggle operation.

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

## Intent rules

- Recognize clear threshold, disable/bypass, enable/restore, and toggle intent semantically; the examples below are illustrative rather than a finite phrase allowlist.
- Disable intent includes requests to turn off or bypass the usage guard, use quota below the stop line, or continue below the configured threshold for the current session.
- Enable intent includes requests to restore the guard, enforce the limit again, stop at the configured threshold, or remove the bypass.
- Threshold mutation requires the requested percentage. Never guess a value.
- A bare `continue`, `resume`, `finish`, urgency, or an active run does not authorize disable.
