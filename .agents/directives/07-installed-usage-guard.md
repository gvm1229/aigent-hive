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

- Exit 3, limited, unknown or reset blocks work. Allow exact controls, consented fallback, or one
  same-binding recheck after threshold change or installed-command reset acknowledgement.
  Continue only on fresh `hive.usage-allowed`. No recheck per tool or session-disable bypass.
- New sessions default enabled. Installed markers and exact binding remain authoritative.

## Controls and interruption

- Threshold: explicit global intent and integer 1–99. Session disable: explicit current-session
  intent plus `--confirm-session-disable`. Continue/resume, urgency and goals grant no bypass.
- Native sensor first; CodexBar only as separately consented failure fallback. No credentials,
  provider calls, CLI reinstall or OMX/OMC.
- Windows desktop control in the 0.12.1 prototype is unqualified and disabled. Keep monitor and
  acceptance tasks off; do not run current-app probes, interrupt tests or automatic installation.
  App termination, ad-hoc watchers and other host-process signals remain forbidden. Preserve exact
  incident acknowledgement and fresh usage requirements; session-only acknowledgement cannot clear
  shared incidents. Existing installed preflight, thresholds and zero-quota checks remain active.
