# Usage guard details

Read only the section that the Skill names. Before any threshold, disable, enable, or toggle
operation, read *Global policy and legacy compatibility* through *Intent rules*.

## Target and policy

Except for the Windows midway monitor, classify the target before any control. A valid installed project harness uses the global policy
plus its registered project override. A Hive source workspace uses the installed global policy
with an explicit `--host` and stores runtime state only under the user root. Every other folder,
including one with only its own `AGENTS.md` or no files, is non-Hive: do not invoke threshold,
session, status, or enforce, and create no halt or runtime state. The global enabled state applies to every project.
Project overrides may only raise the stop threshold: the active value is the maximum of
global, registered-project, and installed-project compatibility thresholds. The global
v2 `enforcement_enabled=false` disables all preflight and midway protection, including
zero-quota checks. It does not erase thresholds or dormant choices. In v1, ordinary disable
affects threshold protection and the independent zero-quota guard remains enabled.
Let the installed CLI interpret the schema; never treat missing v2 fields as an opt-out.
Guard inactivity never disables setup-free Hive Skills. A quick answer, prompt refinement, or
user-root knowledge workflow may still run in a non-Hive folder without project setup and
without a usage preflight. If a separately selected workflow requires project state, that
workflow owns one clear setup approval and automated bootstrap; do not expose capability or run
prerequisites as usage-guard errors.

## Automatic-dispatch preflight

Before automatic dispatch, apply explicit threshold/off/on control. `run resume` automatic
mode performs this shared preflight internally with the same session, process, and user root;
do not sample twice for that dispatch. For a standalone preflight, run:

```text
hive usage enforce --target <project-root> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> [--host codex|claude|antigravity] [--run <canonical-run-id>] [--account-digest <active-account-digest>] --output json
```

Supply an exposed active account digest when sensing is required; omit it for a CLI-confirmed
policy that needs no observation or one unambiguous locally sensed account. Enabled checks
remeasure previous halts. A reset stops one request without a resume lock. Exit `3`, limited,
unknown, or reset results block dispatch. A v2 `hive.usage-disabled` or
`hive.usage-preflight-disabled` result is a completed policy evaluation without sensing or
legacy halt/control reads. `hive.usage-observation-warning` is allowed only by the selected
v2 warning policy; it does not claim a valid observation. Do not run `enforce` for quick
answers, manual work, or other non-dispatch actions except a required same-session recheck.
Non-Codex automatic dispatch fails closed when required sensing has no qualified local sensor.

Treat exit `0` from `enforce` as a session-bound preflight only; it never authorizes dispatch.
Require a fresh same-binding `hive run resume --dispatch-intent automatic` result with
`data.usage_guard.policy_evaluated=true`, one authorization ID, and exactly one dispatch brief.
`enforced=true` with `outcome=authorized` means a valid observation was used. Accept
`outcome=disabled`, `enforced=false`, `observation_required=false` for checks the installed
policy disables. `enforcement_enabled=false` identifies the v2 master opt-out. Accept
`outcome=observation-unavailable` only with the CLI's successful v2 warning-policy evaluation.
Never fabricate an observation or enable a guard to satisfy this Skill. Legacy ordinary
disable still needs its independent zero-quota check unless separately opted out.

`status` is inspection only and never substitutes for an automatic-dispatch preflight. After mutation, run `enforce` only for pending automatic dispatch or the same-session recheck required by *Same-session policy recheck*. Treat `session_override=absent` or `stale` as enabled. Never copy an override to another host, session, or process.

Report saved global and project thresholds, active threshold, selected window, effective session state, changed Hive path, and exact CLI code. With a canonical `--run`, Discord receives only run title and checklist count; never raw prompts, session IDs, absolute paths, or credentials.

## Setup and manual inspection

For ordinary global preference setup, use user-setup's single status/CLI/result flow. Do not turn
this Skill into another questionnaire or infer an opt-out from a setup/resume request.
When a user asks to change which limits are guarded, route to
`hive setup --scope user --section usage`: it asks each selected app once, in app order, whether
its subscription has a 5-hour limit, then which limits to guard (5-hour, weekly, or both; weekly
only without a 5-hour limit), then one threshold per guarded limit. Antigravity reports only
provider quota pools, so it is asked one threshold that applies to every pool. A guarded limit is
checked against its own threshold; an unguarded limit never blocks. `threshold_remaining_percent`
reports the strictest guarded threshold and `guarded_windows` lists each one. A `hive.usage-unknown` result names
the missing guarded limits in `data.missing_windows` and lists resolutions in `next_action`.
After a confirmed session disable, a missing measurement returns `hive.usage-session-bypassed`
without a new halt and still authorizes automatic dispatch for that same disabled binding; a
measured zero quota still blocks until its separate opt-out. Existing
preflight, session and monitor authority below still applies to their exact operations.
`hive usage show --host <selected-host> --user-root <user-root> --output json` is a manual reading,
not a dispatch permit or policy change. A settings summary does not query a sensor.

## Boundaries detail

- Change global `.hive/config/user-setup.yml` only through explicit `hive usage threshold --user-root`
  or the confirmed `hive setup --scope user --section usage` flow with the exact user root.
  Change a configured project's `.hive/config/harness.toml` only through project threshold control,
  and ignored `.hive/runtime/usage-guard/` session bindings only for a configured Hive target.
- Never edit those files directly or persist the raw session identifier.
- Never install a fallback hook, rewrite a prompt, activate another Skill, start a watcher, spawn a subagent, create an orchestration loop, continue a stopped task, or invoke OMX/OMC.
- New v2 setups default all blocking and midway interruption off. Warnings default on and use
  already acquired observations only. Existing v1 behavior remains effective until consented
  migration; its reset default and independent zero-quota protection are preserved.
- When selected checks require it, a reset returns `hive.usage-reset` and exhausted subscription
  quota returns `hive.usage-quota-exhausted`, exit `3`. Missing/stale usage blocks only under the
  strict policy or legacy behavior. Paid credits never replace subscription quota.
- A bare continuation or reset acknowledgement cannot clear an exhaustion halt. Use a fresh
  observation after quota recovery or the separately confirmed opt-out in *Zero-quota session control*.
  This is a Hive preflight safeguard. It does not interrupt an already running host turn or change
  provider billing settings. Never claim that it prevents all account credit consumption.
- CodexBar installation is the sole optional fallback install action. It is allowed only through the exact consented CLI flow in the sensor reference and a qualified package-manager adapter.
- Treat any independently produced OMX/OMC cancellation result as auxiliary evidence only. It never substitutes for the bound halt marker or durable goal/task state.
- Source development uses this installed product contract and global threshold. Never create a second source Skill, Python guard, watcher, or source-local threshold state.

## Windows midway monitor

Handle explicit global Windows midway-monitor intent before folder classification. For example,
“중간 중단 감시를 꺼 줘”, “초기화 감지로 멈추는 기능을 꺼 줘”, or “turn off midway monitoring”
means the installed monitor's `disable`, never ordinary session-guard disable. This global
monitor control is allowed in non-Hive folders, requires no project setup or session binding,
and creates no project state. Read `hive usage monitor --help`, execute
`hive usage monitor disable --user-root <user-root> --output json` on the user's behalf, then
inspect `status`. Report whether configuration is disabled and the controller has stopped.
Do not ask the user to type PowerShell or add confirmation to their explicit disable request.
If the request could mean either safeguard, ask which one; do not change both.

`hive usage monitor --help` exposes the installed version's monitor commands. Only its confirmed
installation may create the current-user scheduled monitor. First show `preview --user-root
<user-root> --output json`; `enable` requires that exact `--confirm <preview_digest>` and
`--confirm-session-interruption`. Do not infer current-version installation authority.

This narrow Windows Codex protection exception permits the installed monitor, not this Skill,
to start observations every 15 seconds after separate consent. Interrupt only for selected
reset, threshold, or zero-quota causes. A continuous 60-second observation gap interrupts only
under the selected strict stop policy or preserved legacy behavior. A failed
persistent connection is discarded; an independent fresh-process read gets its own five-second
limit. Either valid route preserves observation health. Failure retries are two seconds; never
extend the 60-second rule to hide an error. Both routes use the same subscription provider,
so they are not independent services during a provider outage. Keep the app open with a persistent reason/status window. Never
terminate the app when control is unavailable. The authenticated current-user desktop must pass the supported protocol checks; its package version is diagnostic metadata, not an allowlist. Connection, valid observation and confirmed interruption are separate results. It never starts
model work, controls a remote/cloud run, or guarantees an account-percentage overshoot limit.

Use `status` to distinguish saved opt-in, the actual live controller/version, observation route
and health, primary/final errors, consecutive failure time, and verified interruption. An enabled
configuration alone does not prove the monitor is running. Updates preserve opt-in and history,
replace the fixed executable and exact owned task, and refuse to report success if transition
or fresh-observation verification fails. Disabled monitoring stays disabled.

The running monitor shares the existing maximum global/project floor, selected usage window,
current-session exclusions, and independent zero-quota protection. With saved per-host window
choices it checks only Codex's guarded windows, each at its own floor, exactly like preflight;
`status` and `hive setup --scope user --show` report the same choice in `usage_windows`. Children inherit their owning
conversation's policy. Unverified cwd/ownership is reported as unavailable, never replaced with
another project's higher floor. A first valid reading already at/below the floor also stops the
applicable active conversations. The notice states measured remaining quota, applied floor and
window; a sent request is not a verified stop.

A confirmed floor stop is deduplicated by conversation owner, policy, control, account, usage
pool and window. The user can resume that same conversation or send a new message at the same
low quota without another stop or manual preflight floor block. This is not an automatic-dispatch
permit and never transfers to another conversation, scheduled run, or independent zero-quota
check. Recovery above the floor, a new period/account, or policy change rearms detection.
Midway opt-out preserves the independently selected preflight policy. The v2 master opt-out
disables both. A threshold change or new session never enables an unselected guard.
A reset stops the sessions present at detection once. A continuous 60-second observation gap
also stops once until sensing recovers when strict failure protection is selected. Incident records persist as history, never as resume
locks. Users can resume or send a new message directly in Codex. The official-logo notice
only explains the cause and status and has one close button. It never executes a command,
acknowledges, unlocks, or resumes work. `disable` removes only the owned scheduled task and
retains incident evidence and staged bytes. Saved thresholds remain; the selected policy decides
whether threshold and zero-quota checks apply.

Already-started ordinary external commands can continue after a chat stops. This exception does
not cover local AI sessions or subagents: they remain interruption targets under their owner's
policy. Never report an AI child as stopped based only on its parent turn stopping.

## Global policy and legacy compatibility

- For an explicit request to change the whole v2 policy, use
  `hive setup --scope user --section usage --user-root <user-root>` and its confirmed CLI flow.
  If the installed CLI rejects this input, report unsupported and preserve the policy.
  Do not turn a global request into a session override or write settings by hand.
- New v2 protection defaults off. `enforcement_enabled=false` disables all blocking and midway
  interruption, including zero-quota checks; it preserves dormant settings. With the master
  enabled, only the selected preflight/midway causes apply. Legacy v1 ordinary disable still
  leaves independent zero-quota protection active. The installed CLI owns this interpretation.
- A fresh exit-0 `hive.usage-disabled` or `hive.usage-preflight-disabled` with
  `policy_schema_version=2` and `policy_evaluated=true` needs no sensor. A successful
  `hive.usage-observation-warning` is the selected v2 warning policy, not a valid measurement.
  These are valid preflight results, never dispatch authorization. Do not change policy merely
  to obtain `hive.usage-allowed` or `enforced=true`.

## Threshold and current-session controls

7. Perform at most the explicitly requested control mutation:
   - Percentages that a Claude user names are used percent, as on Claude's screen. Convert them:
     "stop at 96%" is `--remaining-percent 4`, or `--host claude --window all --stop-used-percent 96`.
     Never pass a used number as `--remaining-percent`. Codex and Antigravity numbers are already
     remaining. Tell the user both numbers.
   - A global threshold requires explicit global intent and an integer from 0 through 99:

     ```text
     hive usage threshold --user-root <user-root> --remaining-percent <percent> --output json
     ```

     This sets the default threshold and the threshold of every guarded limit of every app with
     saved per-host window choices; `data.host_windows_updated` lists those apps. To give one app
     or one limit a different threshold, use `hive setup --scope user --section usage` instead.

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
Threshold zero does not disable selected checks or enable unselected checks. With an enabled
zero-quota check, exhausted subscription quota stops work regardless of paid credits. Never
promise zero credit consumption from periodic client-side interruption.

## Same-session policy recheck

   Run each required usage check in its own tool call. Inspect its result before starting or
   queuing further work; do not batch it with other work, including parallel calls.

   When a threshold mutation returns `session_recheck_required=true` for an active task, run
   `enforce` immediately with the same exact target, host, session id, process id, user root, and
   available account digest. This recheck also applies to manual source work. Continue only after
   fresh `hive.usage-allowed` or a valid v2 policy result defined above; limited, unknown, or
   policy-changed failures remain blocked. Never
   disable the session merely to acknowledge a changed threshold.

## Quota reset stop

- A selected reset check stops the current request only. A new request reevaluates the installed
  policy; no acknowledgement or unlock is required. Other selected checks still apply.
- A monitor incident is a reason/status record, not dispatch authority or a resume lock. Users may resume directly in Codex. The notice only closes; it never starts or controls work.

## Reset-only session control

- For an explicit request to ignore quota resets only in the current task, use installed help to confirm support, then run:

  ```text
  hive usage session --target <hive-target> --host <host> --user-root <user-root> --session-id <current-session-id> --process-id <current-process-id> --action disable-reset-guard --confirm-reset-guard-disable --output json
  ```

- Restore reset detection with `--action enable-reset-guard`, without the disable confirmation. Both actions require the overall usage guard to remain enabled.
- These actions preserve global settings and other selected checks. Missing or low usage blocks
  only when the installed policy requires it. Other hosts, sessions, and processes do not inherit
  the opt-out; changing the general guard does not clear a current reset-only preference.
- Reset-only disable never repairs a damaged marker. Use fresh enforce after repairing damaged state.
- After either successful action, run one same-binding `enforce`. Continue only after fresh
  `hive.usage-allowed` or a valid v2 policy result defined above. Control success never grants
  dispatch authority or supplies periodic monitoring.
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
  controls preserve this separate preference. New hosts, conversations, and processes use the
  installed policy; they do not inherit a previous session's opt-out.
- After either action, run one same-binding `enforce`. A successful control is not dispatch authority.
  An opt-out leaves an enabled ordinary threshold guard in force. Only disabling both allows sensing
  to be bypassed in v1. A v2 master opt-out also skips sensing. Explain that disabled protection
  does not change the provider's billing rules.
- Exhaustion returns exit `3` and `hive.usage-quota-exhausted`. Stop Hive-controlled work, keep the
  halt receipt, and report the exhausted window. A bare continue does not remove the stop.
  After exhaustion is observed, do not call another tool for checkpoints, knowledge capture,
  lint, Git, cleanup, or handoff records. Defer these operations; the guard's existing receipt
  is sufficient. End with a short halt notice. New user requests require fresh allowed usage.
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
