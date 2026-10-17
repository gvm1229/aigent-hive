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

## Quota reset stop

- A reset stops the current request only. A new user request uses fresh sensing; no acknowledgement or unlock is required. Normal thresholds and zero-quota protection still apply.
- A monitor incident is a reason/status record, not dispatch authority or a resume lock. Users may resume directly in Codex. The notice only closes; it never starts or controls work.

## Reset-only session control

- For an explicit request to ignore quota resets only in the current task, use installed help to confirm support, then run:

  ```text
  hive usage session --target <hive-target> --host <host> --user-root <user-root> --session-id <current-session-id> --process-id <current-process-id> --action disable-reset-guard --confirm-reset-guard-disable --output json
  ```

- Restore reset detection with `--action enable-reset-guard`, without the disable confirmation. Both actions require the overall usage guard to remain enabled.
- These actions preserve the global settings and threshold guard. Missing or low usage still blocks work. Other hosts, sessions, and processes do not inherit the opt-out; changing the general guard does not silently clear a current reset-only preference.
- Reset-only disable never repairs a damaged marker. Use fresh enforce after repairing damaged state.
- After either successful action, run one same-binding `enforce`. Continue only after fresh `hive.usage-allowed`. Control success never grants dispatch authority or supplies periodic monitoring.
- Do not use general `disable` or `toggle` for a reset-only request. A bare continuation request authorizes neither kind of bypass.

## Zero-quota session control

- Zero-quota protection is independent of ordinary threshold and reset protection. Do not treat
  general disable, toggle, threshold changes, or continuation as permission to disable it.
- For an explicit current-session request to allow exhausted subscription usage, confirm installed
  command support and run:

  ```text
  hive usage session --target <hive-target> --host <host> --user-root <user-root> --session-id <current-session-id> --process-id <current-process-id> --action disable-zero-quota-guard --confirm-zero-quota-guard-disable --output json
  ```

- Restore with `--action enable-zero-quota-guard`, without the disable confirmation. General
  controls preserve this separate preference. New hosts, conversations, and processes default on.
- After either action, run one same-binding `enforce`. A successful control is not dispatch authority.
  An opt-out leaves an enabled ordinary threshold guard in force. Only disabling both allows sensing
  to be bypassed. Warn that the opt-out permits work that may use paid account credits.
- Exhaustion returns exit `3` and `hive.usage-quota-exhausted`. Stop Hive-controlled work, keep the
  halt receipt, and report the exhausted window. A bare continue does not remove the stop.
  A fresh positive observation can clear exhaustion; a newly detected quota increase can stop that
  request once while ordinary protection is enabled. The next request remeasures without a resume lock.
- Report `zero_quota_guard_enabled` and `active_turn_interruption_verified`. Do not promise live
  interruption, stop a host process, or change provider billing settings.

## Intent rules

- Recognize clear threshold, disable/bypass, enable/restore, and toggle intent semantically; the examples below are illustrative rather than a finite phrase allowlist.
- Disable intent includes requests to turn off or bypass the usage guard, use quota below the stop line, or continue below the configured threshold for the current session.
- Enable intent includes requests to restore the guard, enforce the limit again, stop at the configured threshold, or remove the bypass.
- Threshold mutation requires the requested percentage. Never guess a value.
- A bare `continue`, `resume`, `finish`, urgency, or an active run does not authorize disable.
