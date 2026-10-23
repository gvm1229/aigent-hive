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
- After observed exhaustion, no further tools: defer checkpoints, knowledge capture, lint, Git,
  cleanup and handoff. Keep the existing receipt; give only a short halt notice. New requests need
  fresh allowed usage. Run required checks alone; inspect the result before queuing work.
- New sessions default enabled. Installed markers and exact binding remain authoritative.

## Controls and interruption

- Threshold: explicit global intent and integer 0–99, subject to installed command support. Session disable: explicit current-session
  intent plus `--confirm-session-disable`. Continue/resume, urgency and goals grant no bypass.
- Native sensor first; CodexBar only as separately consented failure fallback. No credentials,
  provider calls, CLI reinstall or OMX/OMC.
- The 2026-10-23 approval covers two designated Windows chats: reset, 5% and 0% stops,
  app survival and direct resume. Synthetic readings are source-debug fixtures only.
  Preserve failed evidence, opt-in and the 60-second gap; never deliberately drain quota.
- Final correction: no resume lock or acknowledgement. Stop runs present at detection once;
  never block or reinterrupt user-resumed work for that event. The official-logo notice only explains
  cause/status and closes. Keep cadence, deduplication, fresh sensing, thresholds and zero-quota checks.
- No app termination, unsupported-version fallback, ad-hoc watcher or other host-process signal.
