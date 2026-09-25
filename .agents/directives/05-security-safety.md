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
- Never read, write, delete, or migrate `.omx/`, `.omc/`, foreign runtime state, or host-global configuration as part of Hive setup/update.
- Project-local host configuration remains foreign-owned by default. The only exception is an exact Hive-namespaced data-integrity hook projection when the host-native hook surface is supported, no explicit external compatibility owner is active, the user has approved the displayed events/capabilities/paths/digest, and the ownership manifest authorizes a non-clobbering structured merge.
- If external orchestration later becomes available, Hive-owned fallback hooks must remain inert until a consented reconfigure removes only Hive-owned entries; foreign entries and bytes stay untouched.

For installation/update, file or knowledge removal, recursive moves/deletion, history erasure,
release deletion or key rotation, read [update/removal safety](references/update-and-removal.md) first.
