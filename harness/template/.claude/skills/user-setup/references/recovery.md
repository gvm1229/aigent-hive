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

## Clean reinstall

- Use this route for explicitly requested reinstall or authenticated CLI recovery. Structural
  validity of a manifest or a version string alone is not ownership proof. Unverified ownership
  requires read-only diagnosis while preserving files; never uninstall to make validation succeed.
- Run `hive uninstall --user-root <user-root> --output json` only after an exact CLI-owned removal
  plan and explicit destructive-scope authority exist. If the CLI cannot prove the affected files
  and registrations, preserve them and do not substitute manual deletion. This removes Hive-managed host
  activation, projections, packages, indexes, backups, and runtime state while preserving
  `.hive/knowledge/` and saved user preferences.
- Reinstall the selected saved host with `hive install --scope user --host <saved-host> --apply
  --user-root <user-root> --output json`. A valid saved preference file is reused without setup
  questions. Then run the saved-answer `dry-run`, `apply`, and `validate` sequence.
- Hive provides no command to remove the knowledge base or saved preferences. Those files remain
  manual user-owned deletion targets outside this Skill.
