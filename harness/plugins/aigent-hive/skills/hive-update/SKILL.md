---
name: hive-update
description: Verify, dry-run, back up, and activate an offline signed Aigent Hive release when the user explicitly requests a Hive update; never use for ordinary dependency updates, package-manager publication, or knowledge deletion.
---

# Hive Update

Use only the signed Hive CLI. Do not recreate release verification, migration, backup, or
activation logic in shell or host instructions.

## Workflow

1. Run `hive update --help`. If unavailable, report the installed release as unsupported.
2. Require a local extracted release repository and an independently protected TUF root.
   Do not download, trust, or execute a release script inside this Skill.
3. Run a dry-run first:

   ```text
   hive update --target <project-root> --bundle <release-dir> --trust-root <protected-root.json> --dry-run --output json
   ```

4. Show the exact source/target version, release manifest digest, plan digest, planned
   manifest-owned paths, migration route, and package-manager boundary.
5. Apply only after the requested update and exact dry-run plan remain current:

   ```text
   hive update --target <project-root> --bundle <release-dir> --trust-root <protected-root.json> --apply --output json
   ```

6. Accept success only when release thresholds, expiry, rollback floor, artifact digests,
   version/classification policy, migration route, backup, staged validation, activation,
   installed parity, and disposable-index rebuild all pass.
7. On an interrupted transaction, run only:

   ```text
   hive update --target <project-root> --recover --output json
   ```

## Boundaries

- Preserve user-authored project files, docs, canonical Markdown/YAML/TOML, foreign host
  entries, `.omx/`, and `.omc/` bytes.
- Never treat SQLite, runtime state, cache, or backup bytes as migration authority.
- Never combine update with knowledge delete, suppression, or garbage collection.
- Never generate, read, store, or invoke a release private key or model-provider credential.
- Never shell-execute package managers, downloaded migrations, models, subagents, OMX, or OMC.
- Homebrew and WinGet own their binaries; Hive may update the harness only after the installed
  CLI supports the signed route.
- A breaking release requires an exact user-supplied target and a separate bound human
  confirmation. Never infer a major version.
