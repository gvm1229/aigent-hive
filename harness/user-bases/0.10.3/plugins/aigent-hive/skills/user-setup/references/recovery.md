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
