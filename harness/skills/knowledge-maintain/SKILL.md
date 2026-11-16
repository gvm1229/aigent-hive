---
name: knowledge-maintain
description: (knowledge-maintain) Keep Hive knowledge trustworthy by checking it, rebuilding its search index, or carrying out an explicitly requested cleanup.
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

- Give existing user-root facts a subject only on an explicit maintenance request. Preview
  writes nothing and pages with `--limit`/`--offset`; `subject` is `null` where no rule
  applies. Review every proposal yourself, keep facts that answer the same later question
  together, avoid one catch-all subject, and write the review file with the reported
  `preview_digest` and each `claim_id`, `expected_digest` and chosen `subject`. Apply only
  after showing the user the planned subjects; keep the returned `operation_id` for restore:

  ```text
  hive knowledge subjects list --user-root <user-root> --output json
  hive knowledge subjects preview --user-root <user-root> --limit 200 --offset 0 --output json
  hive knowledge subjects apply --user-root <user-root> --review <review.json> --confirm-subject-assignment yes --output json
  hive knowledge subjects restore --user-root <user-root> --operation <operation-id> --confirm-subject-restore yes --output json
  ```

## Optional vector maintenance

- After canonical knowledge changes, refresh the ordinary FTS index first. Vector freshness is
  checked per collection and visibility; report stale scopes instead of silently reinstalling.
- For an explicitly requested vector setup, use `hive knowledge vector preview` with the exact
  user root, target, collection, visibility and Python executable. Only the user's approval of
  that exact preview permits `enable --consent-digest <digest>`. Never auto-install Python or pip.
- For an already approved scope, use `status` then bounded `rebuild --max-seconds <1..60>
  --workers <1..16> --rebuild-mode resume`; continue verified checkpoints until ready or cancelled.
  A failed or unfinished generation must not replace the active one. Use `rollback` for the
  previous verified generation or `disable` to return to FTS-only operation; preserve canonical
  Markdown, registry and project settings.
- For several explicitly selected, already enabled shared scopes on the same runtime, replace
  `--collection` with `--collections <JSON-array>`. Keep `--visibility shared`; never include
  private/confidential or source scopes. Inspect each returned scope state and retry only
  unfinished scopes. `prepared-not-started` may have restored working files but no new calculation.
  Increase the bounded time budget if preparation prevented progress; a failed window is not success.
- Confidential construction needs its own current-action `authorize-build` grant for the exact
  operation and execution budget. Never reuse query consent, broaden visibility, or build another
  private collection. Follow CLI help for the current approval envelope rather than inventing it.
- A source workspace uses `hive source-wiki vector` with an explicit language, never consumer
  storage. Bundles contain canonical knowledge only; regenerate vector state on the destination
  after separate optional setup approval.

## Safety boundaries

- Require explicit deletion or suppression intent; lint or rebuild intent does not authorize canonical deletion.
- Preserve non-Hive files and all user-authored bytes outside declared Hive ownership.
- Never edit SQLite as canonical data or migrate it; rebuild it from tracked Markdown/YAML.
- Treat `hive index rebuild --target <legacy-project>` as a `0.7.x` compatibility action only,
  never as the operational shared-index route.
- Keep update/migration activation and knowledge deletion or garbage collection in separate transactions.
- Never use retrieval to trigger promotion; automatic promotion belongs only to reviewed scan,
  rescan, and maintenance apply flows.
- Report changed paths, evidence digests, and recovery guidance from the CLI result.
