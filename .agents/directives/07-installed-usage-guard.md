# 07. Installed Usage Guard

Use installed Hive policy; no source-local sensor, threshold, watcher or halt fork.

## Scope and preflight

- Authenticated projects: maximum global/project threshold. `hive-source.json`: global policy,
  explicit host, digest-keyed runtime in authenticated user root. Other folders: no usage
  controls, markers or configuration.
- Apply explicit controls first; source thresholds use `hive usage threshold --user-root`.
- Use exact current host session/process once at source-task start. Never invent, reuse,
  persist or transfer a binding:

```text
hive usage enforce --target <source-root> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --output json
```

- Exit 3, limited, unknown or reset blocks dispatch. Allow exact controls, consented fallback,
  same-binding threshold recheck or fresh enforcement for a new request. Require fresh
  `hive.usage-allowed`, or exit-0 v2 (`policy_schema_version=2`, `policy_evaluated=true`) with
  `hive.usage-disabled`, `hive.usage-preflight-disabled` or `hive.usage-observation-warning`.
  Evaluation grants no dispatch or policy-change authority; retain v1 safeguards.
  No per-tool recheck or session bypass.
- Observed exhaustion: no more tools, including checkpoints, knowledge, lint, Git, cleanup or
  handoff. Keep receipt; short halt notice only. New requests need fresh allowed usage.
  Run checks alone; inspect before queuing work.
- New sessions inherit installed policy, not prior opt-outs. Markers and exact binding govern.

## Controls and interruption

- Threshold: explicit global intent, integer 0–99 and installed support. Session disable requires
  explicit current-session intent plus `--confirm-session-disable`. Continue/resume, urgency or
  goals grant no bypass.
- Native sensor first; CodexBar requires separate failure-fallback consent. No credentials,
  provider calls, CLI reinstall or OMX/OMC.
- 2026-10-23 approval: two designated Windows chats, reset/5%/0% stops, app survival, direct resume.
  Synthetic readings: source-debug fixtures only. Keep failures, opt-in and the 60-second gap;
  never deliberately drain quota.
- No resume lock or acknowledgement. Stop runs present at detection once; never block or reinterrupt
  user-resumed work for that event. Official-logo notice: cause/status and close only.
  Keep cadence, deduplication, fresh sensing, thresholds and zero-quota checks.
- No app termination, unsupported-version fallback, ad-hoc watcher or other host-process signal.
