---
name: hive-knowledge-maintenance
description: Lint, rebuild, delete, or suppress canonical Hive knowledge through explicit signed CLI actions while preserving the disposable-index boundary. Use for requested knowledge repair, index rebuild, deletion, or re-ingest suppression; never combine knowledge deletion or garbage collection with a harness update transaction.
---

# Hive Knowledge Maintenance

Use the narrowest explicit maintenance action.

## Actions

- Validate without mutation:

  ```text
  hive knowledge lint --target <project-root> --output json
  ```

- Rebuild the disposable index from tracked sources:

  ```text
  hive index rebuild --target <project-root> --output json
  ```

- Delete a canonical Wiki page and record minimal suppression metadata:

  ```text
  hive knowledge delete --target <project-root> --page-id <id> --reason <reason> [--replacement <locator>] --timestamp <RFC3339> --output json
  ```

- Suppress a deleted source fingerprint from re-ingest:

  ```text
  hive knowledge suppress --target <project-root> --fingerprint <sha256:digest> --source-locator <locator> --reason <reason> [--replacement <locator>] --timestamp <RFC3339> --output json
  ```

## Safety

- Require explicit deletion or suppression intent; lint or rebuild intent does not authorize canonical deletion.
- Preserve non-Hive files and all user-authored bytes outside declared Hive ownership.
- Never edit SQLite as canonical data or migrate it; rebuild it from tracked Markdown/YAML.
- Keep update/migration activation and knowledge deletion or garbage collection in separate transactions.
- Report changed paths, evidence digests, and recovery guidance from the CLI result.
