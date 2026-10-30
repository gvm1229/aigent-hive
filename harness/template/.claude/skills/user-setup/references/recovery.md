## Installation failure diagnosis

- Preserve the first CLI error and its safe diagnostics: host version, failing command phase,
  exit code, fixed reason classification, and output sizes/digests. Do not copy raw host output,
  credentials, or session state into knowledge or reports.
- A failed apply can restore files before returning its error. Missing files after failure do
  not prove that Hive called the host before writing them. Diagnose the actual failing command.
- On Windows, `invalid-marketplace-source` identifies a rejected marketplace path format.
  Use the corrected authenticated Hive release; do not prescribe administrator execution
  without evidence of a permission failure.
- Do not repeat apply with unchanged inputs after the same deterministic failure. Inspect only
  the exact user root and the authenticated CLI recovery result; global setup does not authorize
  project inspection. Preserve saved answers and independent completed setup steps.
- Recover an interrupted install only with the matching authenticated
  `hive install --scope user --host <host> --recover --user-root <user-root> --output json`.
  If recovery cannot attribute the pending host transition, preserve its journal and external
  state. Explain the exact unresolved action; never bypass the refusal with automatic uninstall,
  manual host edits, or a new apply.

## Projection refresh ownership

During an authenticated CLI projection refresh, remove a retired Hive Skill only when its
retired-name ledger entry and historical Hive digest prove ownership. Preserve modified or foreign
Skills with the same name. Only empty Hive-owned parents are eligible after successful deletion.
Never replace this verification with manual directory deletion.

## Preserving removal

- Use removal only when the user requests it. Structural
  validity of a manifest or a version string alone is not ownership proof. Unverified ownership
  requires read-only diagnosis while preserving files; never uninstall to make validation succeed.
- Inspect `hive uninstall --dry-run --user-root <user-root> --output json`. Show the program,
  host registrations, removal paths and preserved data. After authority for that exact scope,
  run `hive uninstall --confirm --user-root <user-root> --output json`. JSON alone is no consent.
  Changed ownership or files must stop removal; never substitute manual deletion.
- Check `completion_verified` and the exit code. A pending result is incomplete, including a
  Windows helper handoff or an unknown program owner. Use the returned `resume_command` and
  retained lifecycle receipt. Never infer completion from a child process starting.
- Preserve saved choices, knowledge, project files and lifecycle history. Disabled monitor stop
  receipts and lock files remain to prevent stale consent reuse. A retained removal helper stays
  outside deletion targets for result inspection and recovery.

## Clean reinstall

- Prefer authenticated CLI repair for an installed program; removal is not a validation bypass.
  Reinstall the selected saved host with `hive install --scope user --host <saved-host> --apply
  --user-root <user-root> --output json`. A valid saved preference file is reused without setup
  questions. Then run the saved-answer `dry-run`, `apply`, and `validate` sequence.
- If the program was removed, restore it with its authenticated npm or direct installer before
  running these commands. Keep the same user root and preserved choices.
- Hive provides no command to remove the knowledge base or saved preferences. Those files remain
  manual user-owned deletion targets outside this Skill.
