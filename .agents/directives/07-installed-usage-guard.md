# 07. Installed Product Usage Guard Directive

Use the installed usage policy for Hive source work. No separate source sensor, threshold,
watcher, or halt implementation.

## Target classification

- A target with an authenticated project `.hive/config/harness.toml` uses the greater of the
  global threshold and its project threshold.
- This repository's `hive-source.json` marker authorizes the installed global policy without a
  project harness. Runtime state stays under the authenticated user root, keyed only by digests.
- Every other folder is non-Hive. Do not invoke status, enforce, threshold, or session control;
  create no halt marker, runtime file, or project configuration.

## Source task preflight

1. Resolve obvious threshold, disable, enable, or toggle intent before ordinary work.
2. Global threshold intent uses `hive usage threshold --user-root <user-root>`; a source task never
   writes a project threshold.
3. Obtain the exact active host session identifier and process ID from host context. Never invent,
   reuse, persist, or transfer a binding.
4. Run exactly one installed-product preflight at the start of the source task:

   ```text
   hive usage enforce --target <source-root> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --output json
   ```

5. Exit `3`, limited, unknown, or reset blocks work. Allow only exact guard control, consented
   fallback, or one same-binding `enforce` after threshold change or explicit reset acknowledgement.
   Acknowledgement requires installed-command support; never reconstruct state. Continue only
   on fresh `hive.usage-allowed`, never by session disable.

Do not repeat the preflight before each tool, mutation, push, or final response. Do not start a
background watcher. The installed marker and exact binding remain authoritative; a new session
defaults enabled.

## Control and safety

- A threshold change requires an explicit integer from 1 through 99 and explicit global intent.
- A session disable requires explicit current-session intent and `--confirm-session-disable`.
- A bare `continue`, `resume`, `finish`, urgency, or active goal does not authorize bypass.
- Native sensing remains primary. CodexBar remains a separately consented failure-only fallback.
- Never request credentials, reinstall a provider CLI, persist raw account or session identifiers,
  call a provider API, invoke OMX/OMC, or signal the host process.
