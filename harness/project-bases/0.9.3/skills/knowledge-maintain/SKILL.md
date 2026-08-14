---
name: knowledge-maintain
description: "(knowledge-maintain) Keep Hive knowledge trustworthy by checking it, rebuilding its search index, or carrying out an explicitly requested cleanup."
---

# Maintain Knowledge (`knowledge-maintain`)

Use the narrowest explicit maintenance action.

First route by target class. A valid `hive-source.json` selects `hive source-wiki lint --target
<source-root> --output json`. Its absence selects consumer lint and is never a reason to skip Wiki
lint. Use the project target only for an enabled registered project. For an unregistered project,
use the user root as both `--target` and `--user-root`.

## Actions

- Validate an enabled registered project without mutation:

  ```text
  hive knowledge lint --target <project-root> --user-root <user-root> --output json
  ```

- Validate user-root knowledge from an unregistered project without mutation:

  ```text
  hive knowledge lint --target <user-root> --user-root <user-root> --output json
  ```

- Rebuild the disposable index from tracked sources:

  ```text
  hive index rebuild --user-root <user-root> --output json
  ```

- Rescan a previously imported repository with a new digest-bound review. The apply flow updates
  source claims, automatically promotes reviewed safe-general claims, and invalidates a promoted
  derivative when its exact source claim no longer applies:

  ```text
  hive knowledge scan --target <directory> --apply <review.json> --user-root <user-root> --output json
  ```

- Delete a canonical Wiki page and record minimal suppression metadata:

  ```text
  hive knowledge delete --target <project-root> --user-root <user-root> --page-id <id> --reason <reason> [--replacement <locator>] --timestamp <RFC3339> --output json
  ```

- Suppress a deleted source fingerprint from re-ingest:

  ```text
  hive knowledge suppress --target <project-root> --user-root <user-root> --fingerprint <sha256:digest> --source-locator <locator> --reason <reason> [--replacement <locator>] --timestamp <RFC3339> --output json
  ```

## Safety

- Require explicit deletion or suppression intent; lint or rebuild intent does not authorize canonical deletion.
- Preserve non-Hive files and all user-authored bytes outside declared Hive ownership.
- Never edit SQLite as canonical data or migrate it; rebuild it from tracked Markdown/YAML.
- Treat `hive index rebuild --target <legacy-project>` as a `0.7.x` compatibility action only,
  never as the operational shared-index route.
- Keep update/migration activation and knowledge deletion or garbage collection in separate transactions.
- Never use retrieval to trigger promotion; automatic promotion belongs only to reviewed scan,
  rescan, and maintenance apply flows.
- Report changed paths, evidence digests, and recovery guidance from the CLI result.
