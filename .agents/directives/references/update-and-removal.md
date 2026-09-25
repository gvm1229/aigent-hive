# Update and removal safety

## Update Safety

- Verify release identity, compatibility, and content hashes before staging.
- Perform a dry run and create a recoverable backup before an update.
- Keep backups for at most seven days.
- Do not include backups or SQLite files in Git.
- Use an atomic activation boundary; on conflict or failed validation, leave the active installation unchanged.
- Never combine update with garbage collection or knowledge deletion.

## Destructive Operations

- Resolve exact targets before deletion.
- Documentation simplification, consolidation, or README streamlining is not deletion authority.
- Move valid knowledge and verify its tracked replacement locator before removing the original.
- Deleting deprecated, incorrect, or superseded active knowledge is allowed only through the
  documented current-truth policy.
- Hard history erasure, branch deletion, force-push, release deletion, and key rotation require explicit user authority.
- Report what was removed and whether Git history or a time-limited backup can recover it.
