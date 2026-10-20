# 07. Installed Usage Guard

Use installed Hive policy; no source-local sensor, threshold, watcher or halt fork.

## Scope and preflight

- Authenticated project harness: maximum global/project threshold. `hive-source.json`: global policy,
  explicit host, runtime under authenticated user root with digest keys. Other folders: no usage
  controls, markers or configuration.
- Apply explicit controls first. Source global thresholds use `hive usage threshold --user-root`.
- Obtain the exact current host session and process from host context; never invent, reuse,
  persist or transfer a binding. Once at source-task start:

```text
hive usage enforce --target <source-root> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --output json
```

- Exit 3, limited, unknown or reset blocks the current dispatch. Allow exact controls, consented
  fallback, a same-binding recheck after threshold change, or fresh enforcement for a new user request.
  Continue only on fresh `hive.usage-allowed`. No recheck per tool or session-disable bypass.
- New sessions default enabled. Installed markers and exact binding remain authoritative.

## Controls and interruption

- Threshold: explicit global intent and integer 1–99. Session disable: explicit current-session
  intent plus `--confirm-session-disable`. Continue/resume, urgency and goals grant no bypass.
- Native sensor first; CodexBar only as separately consented failure fallback. No credentials,
  provider calls, CLI reinstall or OMX/OMC.
- The 2026-10-18 approval covers current-Windows session-only qualification, then 0.12.1 installation
  and always-on monitoring after acceptance. The two live stops are complete; further checks must be
  read-only or display-only. Preserve failed aggregate evidence and installation acceptance gates.
- Final correction: no resume lock or acknowledgement. Stop runs present at detection once;
  never block or reinterrupt user-resumed work for that event. The official-logo notice only explains
  cause/status and closes. Keep cadence, deduplication, fresh sensing, thresholds and zero-quota checks.
- No app termination, unsupported-version fallback, ad-hoc watcher or other host-process signal.
- The 2026-10-21 approved 0.13.1 plan permits new scoped Windows threshold tests on designated
  test conversations, including long reasoning, tool cancellation, direct resume and 30-minute
  sensing. This supersedes the earlier test restriction only for those subjects. Preserve
  opt-in monitoring, the 60-second gap, app survival, prior evidence and installation authority.
