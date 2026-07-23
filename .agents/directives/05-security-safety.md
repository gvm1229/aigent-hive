# 05. Security and Safety Directive

This directive governs filesystem mutation, setup, update, releases, and external integrations.

## Credentials and Network

- Never request or store model-provider API keys.
- Never add a model-provider SDK or direct model endpoint.
- Treat subscription authentication as host-owned.
- Keep release credentials outside the repository and use protected CI environments or hardware-backed signing where available.
- Never commit tokens, certificates, private keys, local account data, or captured host sessions.

## Filesystem Ownership

- Refuse consumer setup when the target contains `hive-source.json`.
- Render into a staging directory before touching a consumer project.
- Validate every output path against an ownership manifest.
- Reject path traversal, absolute template paths, symlink escapes, and writes into external namespaces.
- Preserve non-Hive bytes in shared files; only mutate an exact Hive marker block.
- Never read, write, delete, or migrate `.omx/`, `.omc/`, `.codex/`, `.claude/`, or host-global configuration as part of Hive setup/update.

## Update Safety

- Verify release identity, compatibility, and content hashes before staging.
- Perform a dry run and create a recoverable backup before an update.
- Keep backups for at most seven days.
- Do not include backups or SQLite files in Git.
- Use an atomic activation boundary; on conflict or failed validation, leave the active installation unchanged.
- Never combine update with garbage collection or knowledge deletion.

## Destructive Operations

- Resolve exact targets before deletion.
- Deleting deprecated active knowledge is allowed only through the documented current-truth policy.
- Hard history erasure, branch deletion, force-push, release deletion, and key rotation require explicit user authority.
- Report what was removed and whether Git history or a time-limited backup can recover it.
