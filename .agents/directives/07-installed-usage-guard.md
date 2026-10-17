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
- The 2026-10-18 user request authorizes session-only qualification on the current Windows app,
  then installation and always-on monitoring for 0.12.1 only after acceptance passes. Use the saved
  qualification plan: static protocol check, a scoped adapter test, then isolated monitor acceptance.
  Keep global monitoring off until acceptance. Unsupported app versions fail closed without app exit.
  App termination, ad-hoc watchers and other host-process signals remain forbidden. Preserve exact
  incident acknowledgement and fresh usage requirements; session-only acknowledgement cannot clear
  shared incidents. Existing installed preflight, thresholds and zero-quota checks remain active.
- After the two 2026-10-18 live stops, the user ended further live interruption tests and requested
  the production notice UI. Use display-only and read-only checks; never interrupt current work
  to test its appearance. Preserve failed automatic-confirmation evidence and conditional install gates.
