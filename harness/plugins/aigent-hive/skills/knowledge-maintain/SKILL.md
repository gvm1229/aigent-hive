---
name: knowledge-maintain
description: Lint, rebuild, delete, or suppress canonical Hive knowledge through explicit CLI actions while preserving the disposable-index boundary. Use for requested knowledge repair, index rebuild, deletion, or re-ingest suppression; never combine knowledge deletion or garbage collection with a harness update transaction.
---

# Hive Knowledge Maintenance

Use the narrowest explicit maintenance action.

First route by target class. A valid `hive-source.json` selects `hive source-wiki lint --target
<source-root> --output json`. Its absence selects the consumer command below and is never a reason
to skip Wiki lint.

## Actions

- Validate without mutation:

  ```text
  hive knowledge lint --target <project-root> --user-root <user-root> --output json
  ```

- Rebuild the disposable index from tracked sources:

  ```text
  hive index rebuild --user-root <user-root> --output json
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
- Report changed paths, evidence digests, and recovery guidance from the CLI result.
